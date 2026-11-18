//! Content hasher built on BLAKE3.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use maccleaner_application::ports::ContentHasher;
use maccleaner_domain::duplicates::ContentHash;

/// Bytes read from each end of a file for the quick digest.
const QUICK_SAMPLE_BYTES: u64 = 16 * 1024;

/// Fingerprints files with BLAKE3.
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3ContentHasher;

impl ContentHasher for Blake3ContentHasher {
    fn quick_hash(&self, path: &Path) -> io::Result<ContentHash> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        let mut hasher = blake3::Hasher::new();
        // The length is part of the digest, so a file that is a prefix of
        // another never matches it.
        hasher.update(&length.to_le_bytes());
        if length <= QUICK_SAMPLE_BYTES * 2 {
            hasher.update_reader(&mut file)?;
        } else {
            hasher.update_reader((&mut file).take(QUICK_SAMPLE_BYTES))?;
            file.seek(SeekFrom::Start(length - QUICK_SAMPLE_BYTES))?;
            hasher.update_reader((&mut file).take(QUICK_SAMPLE_BYTES))?;
        }
        Ok(ContentHash(*hasher.finalize().as_bytes()))
    }

    fn full_hash(&self, path: &Path) -> io::Result<ContentHash> {
        let mut hasher = blake3::Hasher::new();
        hasher.update_reader(File::open(path)?)?;
        Ok(ContentHash(*hasher.finalize().as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use maccleaner_application::ports::ContentHasher;
    use maccleaner_test_support::TempTree;

    use super::Blake3ContentHasher;

    #[test]
    fn identical_content_gives_identical_digests() {
        let fixture = TempTree::new().unwrap();
        let a = fixture.file("a.bin", 100_000).unwrap();
        let b = fixture.file("copy/b.bin", 100_000).unwrap();

        let hasher = Blake3ContentHasher;
        assert_eq!(
            hasher.quick_hash(&a).unwrap(),
            hasher.quick_hash(&b).unwrap()
        );
        assert_eq!(hasher.full_hash(&a).unwrap(), hasher.full_hash(&b).unwrap());
    }

    #[test]
    fn the_full_digest_sees_a_difference_in_the_middle_that_the_quick_one_misses() {
        let fixture = TempTree::new().unwrap();
        let a = fixture.file("a.bin", 100_000).unwrap();
        let b = fixture.file("b.bin", 100_000).unwrap();
        let mut content = fs::read(&b).unwrap();
        content[50_000] ^= 0xFF;
        fs::write(&b, content).unwrap();

        let hasher = Blake3ContentHasher;
        assert_eq!(
            hasher.quick_hash(&a).unwrap(),
            hasher.quick_hash(&b).unwrap()
        );
        assert_ne!(hasher.full_hash(&a).unwrap(), hasher.full_hash(&b).unwrap());
    }

    #[test]
    fn the_quick_digest_tells_apart_files_that_differ_at_either_end_or_in_length() {
        let fixture = TempTree::new().unwrap();
        let base = fixture.file("base.bin", 100_000).unwrap();
        let hasher = Blake3ContentHasher;
        let expected = hasher.quick_hash(&base).unwrap();

        for (name, position) in [("start.bin", 0), ("end.bin", 99_999)] {
            let other = fixture.file(name, 100_000).unwrap();
            let mut content = fs::read(&other).unwrap();
            content[position] ^= 0xFF;
            fs::write(&other, content).unwrap();
            assert_ne!(hasher.quick_hash(&other).unwrap(), expected, "{name}");
        }
        let longer = fixture.file("longer.bin", 100_001).unwrap();
        assert_ne!(hasher.quick_hash(&longer).unwrap(), expected);
    }

    #[test]
    fn a_missing_file_is_an_error() {
        let fixture = TempTree::new().unwrap();
        assert!(
            Blake3ContentHasher
                .full_hash(&fixture.path("gone"))
                .is_err()
        );
    }
}
