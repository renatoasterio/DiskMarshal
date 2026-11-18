//! Finding similar photos among real image files on disk.

use std::fs;

use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::{FindSimilarPhotos, SimilarPhotosOptions};
use maccleaner_domain::deletion::DeletionMethod;
use maccleaner_domain::photos::SimilarityTolerance;
use maccleaner_infrastructure::filesystem::{LstatFileInspector, ReaddirWalker};
use maccleaner_infrastructure::imaging::ImageIoThumbnailer;
use maccleaner_test_support::{TempTree, bitmap_bytes};

/// A bright sun over a dark hill.
fn landscape(x: f32, y: f32) -> [u8; 3] {
    let sun = ((x - 0.7).powi(2) + (y - 0.3).powi(2)).sqrt() < 0.18;
    let hill = y > 0.6 + 0.2 * (x * 6.0).sin();
    match (sun, hill) {
        (true, _) => [250, 220, 90],
        (_, true) => [30, 70, 40],
        _ => [90, 150, 220],
    }
}

/// Dark stripes on a light wall: nothing like the landscape.
fn stripes(x: f32, _y: f32) -> [u8; 3] {
    if ((x * 7.0) as u32).is_multiple_of(2) {
        [40, 30, 60]
    } else {
        [230, 225, 210]
    }
}

#[test]
fn gathers_the_same_picture_saved_at_two_sizes_and_nothing_else() {
    let fixture = TempTree::new().unwrap();
    let write = |relative: &str, bytes: Vec<u8>| {
        let path = fixture.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    };
    write("trip/beach.bmp", bitmap_bytes(240, 180, landscape));
    write("export/beach-small.bmp", bitmap_bytes(200, 150, landscape));
    write("trip/wall.bmp", bitmap_bytes(240, 180, stripes));
    // Too small to be a photo, and not an image at all.
    write("trip/icon.bmp", bitmap_bytes(16, 16, landscape));
    fixture.file("trip/notes.txt", 200_000).unwrap();
    // Images inside a Photos library are never read.
    write(
        "Fotos.photoslibrary/originals/beach.bmp",
        bitmap_bytes(240, 180, landscape),
    );

    let options = SimilarPhotosOptions {
        tolerance: SimilarityTolerance::Strict,
        previewed_groups: 10,
        preview_side: 96,
    };
    let report = FindSimilarPhotos::new(
        &ReaddirWalker::default(),
        &ImageIoThumbnailer,
        &LstatFileInspector,
    )
    .execute(
        fixture.root(),
        options,
        &NoProgress,
        &CancellationToken::new(),
    )
    .unwrap();

    assert_eq!(report.photos_compared, 3, "two beaches and one wall");
    assert_eq!(report.groups.len(), 1);
    let found = &report.groups[0];
    let names: Vec<String> = found
        .group
        .photos
        .iter()
        .map(|photo| {
            photo
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names,
        ["beach.bmp", "beach-small.bmp"],
        "the larger file is kept"
    );
    assert!(found.group.photos[1].similarity > 0.95);
    assert!(
        found
            .requests
            .iter()
            .all(|request| request.method == DeletionMethod::Trash)
    );
    assert_eq!(found.requests.len(), 2);

    assert_eq!(found.previews.len(), 2);
    let preview = found.previews[0].as_ref().unwrap();
    assert_eq!((preview.width(), preview.height()), (96, 72));
}
