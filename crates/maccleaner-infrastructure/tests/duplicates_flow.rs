//! Finding duplicates among real files on disk.

use std::fs;
use std::path::PathBuf;

use maccleaner_application::progress::{CancellationToken, NoProgress};
use maccleaner_application::use_cases::FindDuplicates;
use maccleaner_domain::deletion::DeletionMethod;
use maccleaner_domain::shared::ByteSize;
use maccleaner_infrastructure::filesystem::{LstatFileInspector, ReaddirWalker};
use maccleaner_infrastructure::hashing::Blake3ContentHasher;
use maccleaner_test_support::TempTree;

#[test]
fn groups_only_files_whose_whole_content_is_identical() {
    let fixture = TempTree::new().unwrap();
    fixture.file("videos/trip.mov", 80_000).unwrap();
    fixture.file("downloads/trip copy.mov", 80_000).unwrap();
    fixture.file("desktop/trip-2.mov", 80_000).unwrap();
    // Same size as the videos, but different in the middle.
    let different = fixture.file("downloads/other.mov", 80_000).unwrap();
    let mut content = fs::read(&different).unwrap();
    content[40_000] ^= 0xFF;
    fs::write(&different, content).unwrap();
    // A second name for the same file is not a second copy.
    fixture
        .hard_link("videos/trip.mov", "videos/trip-link.mov")
        .unwrap();
    fixture.file("notes/a.txt", 10).unwrap();
    fixture.file("notes/b.txt", 10).unwrap();

    let report = FindDuplicates::new(
        &ReaddirWalker::default(),
        &Blake3ContentHasher,
        &LstatFileInspector,
    )
    .execute(
        fixture.root(),
        ByteSize::from_bytes(1_000),
        &NoProgress,
        &CancellationToken::new(),
    )
    .unwrap();

    assert_eq!(
        report.groups.len(),
        1,
        "small files and near-copies are not groups"
    );
    let found = &report.groups[0];
    assert_eq!(found.group.size, ByteSize::from_bytes(80_000));
    let mut names: Vec<String> = found
        .group
        .copies
        .iter()
        .map(|copy| {
            copy.path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(names.len(), 3);
    assert!(!names.contains(&"other.mov".to_owned()));
    assert_eq!(found.requests.len(), 3);
    assert!(
        found
            .requests
            .iter()
            .all(|request| request.method == DeletionMethod::Trash)
    );
    let targets: Vec<&PathBuf> = found
        .requests
        .iter()
        .map(|request| &request.target)
        .collect();
    let copies: Vec<&PathBuf> = found.group.copies.iter().map(|copy| &copy.path).collect();
    assert_eq!(targets, copies, "each removal matches its copy, in order");
}
