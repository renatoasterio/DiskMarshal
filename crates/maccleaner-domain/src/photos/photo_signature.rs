//! What an image looks like, reduced to a few numbers that can be compared.

use crate::photos::{SimilarityTolerance, Thumbnail};

/// Columns and rows of the grid the outline hash is taken from. Each row
/// gives eight comparisons between neighbours, 64 bits in all.
const HASH_COLUMNS: usize = 9;
const HASH_ROWS: usize = 8;
const HASH_BITS: u32 = 64;
/// Side of the brightness map compared cell by cell.
const MAP_SIDE: usize = 32;
/// Most each color channel of the average color may differ.
const MAX_COLOR_DIFFERENCE: u8 = 20;
/// Most the proportions may differ, as a share of the larger one.
const MAX_ASPECT_DIFFERENCE: f32 = 0.05;

/// The look of an image: its outline, its brightness map, its average color
/// and its proportions.
///
/// The outline hash alone cannot tell apart images with little detail, such
/// as two pages of text, so two images match only when the brightness map,
/// the color and the proportions agree as well.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoSignature {
    hash: u64,
    brightness: Box<[u8]>,
    mean_color: [u8; 3],
    aspect: f32,
}

impl PhotoSignature {
    /// Describes the image a thumbnail was made from.
    #[must_use]
    pub fn of(thumbnail: &Thumbnail) -> Self {
        let luma: Vec<u8> = thumbnail.colors().map(luma).collect();
        let (width, height) = (thumbnail.width() as usize, thumbnail.height() as usize);
        let grid = shrink(&luma, width, height, HASH_COLUMNS, HASH_ROWS);
        let hash = grid
            .as_chunks::<HASH_COLUMNS>()
            .0
            .iter()
            .fold(0u64, |hash, row| {
                row.windows(2).fold(hash, |hash, pair| {
                    (hash << 1) | u64::from(pair[0] > pair[1])
                })
            });
        Self {
            hash,
            brightness: shrink(&luma, width, height, MAP_SIDE, MAP_SIDE).into_boxed_slice(),
            mean_color: mean_color(thumbnail),
            aspect: width as f32 / height as f32,
        }
    }

    /// How alike the outlines of two images are, from 0.0 to 1.0.
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f32 {
        1.0 - self.hash_distance(other) as f32 / HASH_BITS as f32
    }

    /// Whether the two images count as the same photo under `tolerance`.
    #[must_use]
    pub fn matches(&self, other: &Self, tolerance: SimilarityTolerance) -> bool {
        self.hash_distance(other) <= tolerance.max_hash_distance()
            && self.has_proportions_of(other)
            && self.has_color_of(other)
            && self.mean_difference(other) <= tolerance.max_mean_difference()
    }

    fn hash_distance(&self, other: &Self) -> u32 {
        (self.hash ^ other.hash).count_ones()
    }

    fn has_proportions_of(&self, other: &Self) -> bool {
        let larger = self.aspect.max(other.aspect);
        larger > 0.0 && (self.aspect - other.aspect).abs() / larger <= MAX_ASPECT_DIFFERENCE
    }

    fn has_color_of(&self, other: &Self) -> bool {
        self.mean_color
            .iter()
            .zip(other.mean_color)
            .all(|(mine, theirs)| mine.abs_diff(theirs) <= MAX_COLOR_DIFFERENCE)
    }

    /// Average difference between the brightness maps, on the 0 to 255 scale.
    fn mean_difference(&self, other: &Self) -> u32 {
        let total: u32 = self
            .brightness
            .iter()
            .zip(other.brightness.iter())
            .map(|(mine, theirs)| u32::from(mine.abs_diff(*theirs)))
            .sum();
        total / (MAP_SIDE * MAP_SIDE) as u32
    }
}

/// Perceived brightness of a color, with the usual weights for each channel.
fn luma([red, green, blue]: [u8; 3]) -> u8 {
    ((u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114) / 1000) as u8
}

fn mean_color(thumbnail: &Thumbnail) -> [u8; 3] {
    let mut totals = [0u64; 3];
    let mut count = 0u64;
    for color in thumbnail.colors() {
        for (total, channel) in totals.iter_mut().zip(color) {
            *total += u64::from(channel);
        }
        count += 1;
    }
    totals.map(|total| (total / count.max(1)) as u8)
}

/// Shrinks a `width` by `height` map to `columns` by `rows`, averaging the
/// source cells each new cell covers.
fn shrink(source: &[u8], width: usize, height: usize, columns: usize, rows: usize) -> Vec<u8> {
    let mut shrunk = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        let top = row * height / rows;
        let bottom = ((row + 1) * height / rows).clamp(top + 1, height);
        for column in 0..columns {
            let left = column * width / columns;
            let right = ((column + 1) * width / columns).clamp(left + 1, width);
            let total: u64 = (top..bottom)
                .flat_map(|y| source.iter().skip(y * width + left).take(right - left))
                .map(|value| u64::from(*value))
                .sum();
            let count = ((bottom - top) * (right - left)) as u64;
            shrunk.push((total / count) as u8);
        }
    }
    shrunk
}

#[cfg(test)]
mod tests {
    use super::PhotoSignature;
    use crate::photos::{SimilarityTolerance, Thumbnail};

    /// A picture drawn from a formula, so tests need no image files.
    fn picture(width: u32, height: u32, color_at: impl Fn(f32, f32) -> [u8; 3]) -> Thumbnail {
        let mut rgba = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let [red, green, blue] =
                    color_at(x as f32 / width as f32, y as f32 / height as f32);
                rgba.extend([red, green, blue, 255]);
            }
        }
        Thumbnail::new(width, height, rgba).unwrap()
    }

    /// A bright sun over a dark hill: enough structure for an outline.
    fn landscape(x: f32, y: f32) -> [u8; 3] {
        let sun = ((x - 0.7).powi(2) + (y - 0.3).powi(2)).sqrt() < 0.18;
        let hill = y > 0.6 + 0.2 * (x * 6.0).sin();
        match (sun, hill) {
            (true, _) => [250, 220, 90],
            (_, true) => [30, 70, 40],
            _ => [90, 150, 220],
        }
    }

    #[test]
    fn the_same_picture_at_another_size_matches_strictly() {
        let large = PhotoSignature::of(&picture(64, 48, landscape));
        let small = PhotoSignature::of(&picture(48, 36, landscape));

        assert!(large.matches(&small, SimilarityTolerance::Strict));
        assert!(large.similarity(&small) > 0.95);
    }

    #[test]
    fn a_slightly_darker_copy_matches_only_with_some_tolerance() {
        let original = PhotoSignature::of(&picture(64, 48, landscape));
        let darker =
            PhotoSignature::of(&picture(64, 48, |x, y| landscape(x, y).map(|c| c - c / 25)));

        assert!(original.matches(&darker, SimilarityTolerance::Balanced));
    }

    #[test]
    fn a_mirrored_picture_does_not_match() {
        let original = PhotoSignature::of(&picture(64, 48, landscape));
        let mirrored = PhotoSignature::of(&picture(64, 48, |x, y| landscape(1.0 - x, y)));

        assert!(!original.matches(&mirrored, SimilarityTolerance::Loose));
    }

    #[test]
    fn flat_pictures_of_different_colors_do_not_match() {
        let red = PhotoSignature::of(&picture(32, 32, |_, _| [200, 30, 30]));
        let blue = PhotoSignature::of(&picture(32, 32, |_, _| [30, 30, 200]));

        assert!(!red.matches(&blue, SimilarityTolerance::Loose));
    }

    #[test]
    fn pages_with_different_text_do_not_match() {
        // Mostly white, with dark lines in different places: the outline
        // hash barely differs, the brightness map does.
        let page = |lines: [f32; 3]| {
            picture(64, 64, move |x, y| {
                let on_line = lines
                    .iter()
                    .any(|line| (y - line).abs() < 0.06 && x > 0.1 && x < 0.9);
                if on_line {
                    [40, 40, 40]
                } else {
                    [250, 250, 250]
                }
            })
        };
        let first = PhotoSignature::of(&page([0.15, 0.35, 0.55]));
        let second = PhotoSignature::of(&page([0.25, 0.6, 0.85]));

        assert!(!first.matches(&second, SimilarityTolerance::Loose));
    }

    #[test]
    fn the_same_picture_in_other_proportions_does_not_match() {
        let wide = PhotoSignature::of(&picture(64, 32, landscape));
        let square = PhotoSignature::of(&picture(48, 48, landscape));

        assert!(!wide.matches(&square, SimilarityTolerance::Loose));
    }

    #[test]
    fn a_thumbnail_smaller_than_the_grids_still_gets_a_signature() {
        let tiny = PhotoSignature::of(&picture(3, 2, landscape));
        assert!(tiny.matches(&tiny.clone(), SimilarityTolerance::Strict));
    }
}
