//! Photos that look the same, gathered so the user keeps one of each.

use std::cmp::Reverse;
use std::path::PathBuf;

use crate::photos::{PhotoSignature, SimilarityTolerance};
use crate::scan::LocalFile;
use crate::shared::ByteSize;

/// One of the photos of a group.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarPhoto {
    /// Absolute path of the file.
    pub path: PathBuf,
    /// Size of the content.
    pub logical_size: ByteSize,
    /// Disk space the file uses.
    pub allocated: ByteSize,
    /// Last modification, in seconds since the Unix epoch, when known.
    pub modified_unix_seconds: Option<i64>,
    /// How alike this photo and the one to keep are, from 0.0 to 1.0.
    pub similarity: f32,
}

/// Two or more photos that look the same.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityGroup {
    /// The photos, with the one to keep first.
    pub photos: Vec<SimilarPhoto>,
}

impl SimilarityGroup {
    /// Space freed by removing every photo but the one to keep.
    #[must_use]
    pub fn reclaimable(&self) -> ByteSize {
        self.photos
            .iter()
            .skip(1)
            .map(|photo| photo.allocated)
            .sum()
    }
}

/// Gathers the photos that match each other under `tolerance`.
///
/// Photos are linked in chains: when A matches B and B matches C, the three
/// form one group. In each group the largest file comes first, because it is
/// most likely the original; among equal sizes, the oldest. Groups come out
/// with the one that frees the most space first.
#[must_use]
pub fn group_similar(
    photos: &[(LocalFile, PhotoSignature)],
    tolerance: SimilarityTolerance,
) -> Vec<SimilarityGroup> {
    let mut leaders: Vec<usize> = (0..photos.len()).collect();
    for (first, (_, signature)) in photos.iter().enumerate() {
        for (second, (_, other)) in photos.iter().enumerate().skip(first + 1) {
            if signature.matches(other, tolerance) {
                let (a, b) = (
                    leader_of(&mut leaders, first),
                    leader_of(&mut leaders, second),
                );
                if let Some(slot) = leaders.get_mut(a.max(b)) {
                    *slot = a.min(b);
                }
            }
        }
    }

    let mut members: Vec<Vec<usize>> = vec![Vec::new(); photos.len()];
    for index in 0..photos.len() {
        let leader = leader_of(&mut leaders, index);
        if let Some(group) = members.get_mut(leader) {
            group.push(index);
        }
    }
    let mut groups: Vec<SimilarityGroup> = members
        .into_iter()
        .filter(|group| group.len() > 1)
        .filter_map(|group| group_of(photos, group))
        .collect();
    groups.sort_by_key(|group| Reverse(group.reclaimable()));
    groups
}

/// Follows the links up to the photo that leads the group, shortening the
/// path for the next search.
fn leader_of(leaders: &mut [usize], mut index: usize) -> usize {
    while let Some(&parent) = leaders.get(index) {
        if parent == index {
            break;
        }
        let grandparent = leaders.get(parent).copied().unwrap_or(parent);
        if let Some(slot) = leaders.get_mut(index) {
            *slot = grandparent;
        }
        index = parent;
    }
    index
}

fn group_of(
    photos: &[(LocalFile, PhotoSignature)],
    mut members: Vec<usize>,
) -> Option<SimilarityGroup> {
    members.sort_by(|a, b| {
        let key = |index: &usize| {
            photos.get(*index).map(|(file, _)| {
                (
                    Reverse(file.logical_size),
                    file.modified_unix_seconds.unwrap_or(i64::MAX),
                    file.path.as_os_str().len(),
                    file.path.clone(),
                )
            })
        };
        key(a).cmp(&key(b))
    });
    let (_, reference) = photos.get(*members.first()?)?;
    let photos = members
        .iter()
        .filter_map(|index| photos.get(*index))
        .map(|(file, signature)| SimilarPhoto {
            path: file.path.clone(),
            logical_size: file.logical_size,
            allocated: file.allocated,
            modified_unix_seconds: file.modified_unix_seconds,
            similarity: reference.similarity(signature),
        })
        .collect();
    Some(SimilarityGroup { photos })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::group_similar;
    use crate::photos::{PhotoSignature, SimilarityTolerance, Thumbnail};
    use crate::scan::LocalFile;
    use crate::shared::ByteSize;

    /// A picture with a bright band whose position is `shift`.
    fn signature(shift: f32) -> PhotoSignature {
        let mut rgba = Vec::new();
        for y in 0..48 {
            for x in 0..64 {
                let (x, y) = (x as f32 / 64.0, y as f32 / 48.0);
                let lit = (x - shift).abs() < 0.15 && y > 0.2;
                let value = if lit { 230 } else { (40.0 + y * 60.0) as u8 };
                rgba.extend([value, value, value, 255]);
            }
        }
        PhotoSignature::of(&Thumbnail::new(64, 48, rgba).unwrap())
    }

    fn photo(path: &str, bytes: u64, modified: i64, shift: f32) -> (LocalFile, PhotoSignature) {
        let file = LocalFile {
            path: PathBuf::from(path),
            logical_size: ByteSize::from_bytes(bytes),
            allocated: ByteSize::from_bytes(bytes),
            modified_unix_seconds: Some(modified),
        };
        (file, signature(shift))
    }

    #[test]
    fn gathers_matching_photos_and_keeps_the_largest_first() {
        let photos = [
            photo("/p/small.jpg", 200, 5, 0.3),
            photo("/p/other.jpg", 900, 5, 0.8),
            photo("/p/original.heic", 700, 9, 0.3),
            photo("/p/export.png", 700, 2, 0.3),
        ];

        let groups = group_similar(&photos, SimilarityTolerance::Strict);

        assert_eq!(groups.len(), 1);
        let paths: Vec<&str> = groups[0]
            .photos
            .iter()
            .filter_map(|photo| photo.path.to_str())
            .collect();
        // Largest first; among equal sizes, the oldest.
        assert_eq!(paths, ["/p/export.png", "/p/original.heic", "/p/small.jpg"]);
        assert_eq!(groups[0].reclaimable(), ByteSize::from_bytes(900));
        assert!((groups[0].photos[0].similarity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn photos_that_match_nothing_form_no_group() {
        let photos = [
            photo("/p/a.jpg", 100, 1, 0.2),
            photo("/p/b.jpg", 100, 1, 0.8),
        ];
        assert!(group_similar(&photos, SimilarityTolerance::Loose).is_empty());
    }

    #[test]
    fn orders_groups_by_the_space_they_free() {
        let photos = [
            photo("/p/a1.jpg", 100, 1, 0.2),
            photo("/p/a2.jpg", 100, 1, 0.2),
            photo("/p/b1.jpg", 900, 1, 0.8),
            photo("/p/b2.jpg", 900, 1, 0.8),
        ];

        let groups = group_similar(&photos, SimilarityTolerance::Strict);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].reclaimable(), ByteSize::from_bytes(900));
    }
}
