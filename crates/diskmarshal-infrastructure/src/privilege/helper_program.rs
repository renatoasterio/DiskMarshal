//! Deciding whether the helper on disk may be given administrator rights.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Absolute path, so a different program earlier in `PATH` can never run.
const CODESIGN: &str = "/usr/bin/codesign";
/// Folders of an app bundle between the bundle and its executables.
const BUNDLE_EXECUTABLES: [&str; 2] = ["Contents", "MacOS"];
/// Permission bits that would let someone other than the owner change a file.
const WRITABLE_BY_OTHERS: u32 = 0o022;
const EXECUTABLE_BY_OWNER: u32 = 0o100;

/// Whether `path` is a real program file that only its owner can change.
/// A link is refused: what it points to could be swapped.
pub(crate) fn is_unmodifiable_program(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.file_type().is_file()
            && metadata.mode() & WRITABLE_BY_OTHERS == 0
            && metadata.mode() & EXECUTABLE_BY_OWNER != 0
    })
}

/// When the helper is inside an app bundle, asks macOS whether the bundle is
/// still exactly what was signed. Outside a bundle, as in a development
/// build, there is no signature to check.
pub(crate) fn has_valid_signature(helper: &Path) -> bool {
    let Some(bundle) = bundle_of(helper) else {
        return true;
    };
    Command::new(CODESIGN)
        .args(["--verify", "--strict"])
        .arg(bundle)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// The app bundle whose `Contents/MacOS` folder holds `program`, if any.
fn bundle_of(program: &Path) -> Option<&Path> {
    let mut folder = program.parent()?;
    for expected in BUNDLE_EXECUTABLES.iter().rev() {
        if folder.file_name()?.to_str()? != *expected {
            return None;
        }
        folder = folder.parent()?;
    }
    folder
        .extension()
        .is_some_and(|extension| extension == "app")
        .then_some(folder)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::bundle_of;

    #[test]
    fn finds_the_bundle_only_from_its_executables_folder() {
        let inside = Path::new("/Applications/DiskMarshal.app/Contents/MacOS/helper");
        assert_eq!(
            bundle_of(inside),
            Some(Path::new("/Applications/DiskMarshal.app"))
        );
        assert_eq!(bundle_of(Path::new("/work/target/debug/helper")), None);
        assert_eq!(
            bundle_of(Path::new("/x/Folder/Contents/MacOS/helper")),
            None
        );
    }
}
