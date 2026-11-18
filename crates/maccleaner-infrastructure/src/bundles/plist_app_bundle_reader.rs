//! App bundle reader built on the `Info.plist` every bundle carries.

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use maccleaner_application::ports::{AppBundleFacts, AppBundleReader};
use maccleaner_domain::cleanup::PrivilegeScope;
use plist::Value;

/// Where a bundle keeps its description.
const INFO_FILE: &str = "Contents/Info.plist";
/// A description larger than this is not one: real ones are a few kilobytes.
const INFO_FILE_LIMIT_BYTES: u64 = 1_048_576;
const IDENTIFIER_KEY: &str = "CFBundleIdentifier";
/// Keys that hold the name of the app, the one shown to the user first.
const NAME_KEYS: [&str; 2] = ["CFBundleDisplayName", "CFBundleName"];
/// Keys that hold the version, the one shown to the user first.
const VERSION_KEYS: [&str; 2] = ["CFBundleShortVersionString", "CFBundleVersion"];

/// Reads `Contents/Info.plist`, in either of the formats macOS writes it.
///
/// A bundle owned by the user this reader was created for can be removed by
/// that user. Any other bundle, such as one installed by the App Store or by
/// an installer package, needs an administrator.
#[derive(Debug, Clone, Copy)]
pub struct PlistAppBundleReader {
    user_id: Option<u32>,
}

impl PlistAppBundleReader {
    /// Creates a reader for the user who owns the folder `home`.
    #[must_use]
    pub fn for_home(home: &Path) -> Self {
        Self {
            user_id: fs::metadata(home).ok().map(|metadata| metadata.uid()),
        }
    }
}

impl AppBundleReader for PlistAppBundleReader {
    fn read(&self, bundle: &Path) -> Option<AppBundleFacts> {
        let owner = fs::symlink_metadata(bundle)
            .ok()
            .filter(fs::Metadata::is_dir)?
            .uid();
        let info = bundle.join(INFO_FILE);
        if fs::metadata(&info).ok()?.len() > INFO_FILE_LIMIT_BYTES {
            return None;
        }
        let description = Value::from_file(&info).ok()?;
        let description = description.as_dictionary()?;
        let text = |keys: &[&str]| {
            keys.iter()
                .filter_map(|key| description.get(key)?.as_string())
                .map(str::trim)
                .find(|text| !text.is_empty())
                .map(str::to_owned)
        };
        Some(AppBundleFacts {
            bundle_id: text(&[IDENTIFIER_KEY])?,
            declared_name: text(&NAME_KEYS),
            version: text(&VERSION_KEYS),
            scope: if self.user_id == Some(owner) {
                PrivilegeScope::User
            } else {
                PrivilegeScope::Administrator
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use maccleaner_application::ports::AppBundleReader;
    use maccleaner_domain::cleanup::PrivilegeScope;
    use maccleaner_test_support::TempTree;

    use super::PlistAppBundleReader;

    const DESCRIPTION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.vendor.editor</string>
    <key>CFBundleName</key>
    <string>Vendor Editor</string>
    <key>CFBundleShortVersionString</key>
    <string>4.2</string>
    <key>CFBundleVersion</key>
    <string>4200</string>
</dict>
</plist>
"#;

    fn bundle(fixture: &TempTree, name: &str, description: &str) -> std::path::PathBuf {
        let info = fixture
            .file(&format!("{name}/Contents/Info.plist"), 0)
            .unwrap();
        fs::write(info, description).unwrap();
        fixture.path(name)
    }

    #[test]
    fn reads_the_identifier_the_name_and_the_version_shown_to_the_user() {
        let fixture = TempTree::new().unwrap();
        let editor = bundle(&fixture, "Editor.app", DESCRIPTION);

        let facts = PlistAppBundleReader::for_home(fixture.root())
            .read(&editor)
            .unwrap();

        assert_eq!(facts.bundle_id, "com.vendor.editor");
        assert_eq!(facts.declared_name.as_deref(), Some("Vendor Editor"));
        assert_eq!(facts.version.as_deref(), Some("4.2"));
        assert_eq!(
            facts.scope,
            PrivilegeScope::User,
            "owned by who owns the home"
        );
    }

    #[test]
    fn a_bundle_of_someone_else_needs_an_administrator() {
        let fixture = TempTree::new().unwrap();
        let editor = bundle(&fixture, "Editor.app", DESCRIPTION);

        // The root folder of the disk belongs to the system, not to the user.
        let facts = PlistAppBundleReader::for_home(Path::new("/"))
            .read(&editor)
            .unwrap();

        assert_eq!(facts.scope, PrivilegeScope::Administrator);
    }

    #[test]
    fn refuses_what_is_not_an_app_bundle() {
        let fixture = TempTree::new().unwrap();
        let reader = PlistAppBundleReader::for_home(fixture.root());

        let nameless = bundle(
            &fixture,
            "Nameless.app",
            &DESCRIPTION.replace("com.vendor.editor", "  "),
        );
        let broken = bundle(&fixture, "Broken.app", "not a property list");
        let empty = fixture.directory("Empty.app").unwrap();
        let link = fixture
            .symlink(&bundle(&fixture, "Real.app", DESCRIPTION), "Link.app")
            .unwrap();

        assert!(reader.read(&nameless).is_none(), "no identifier");
        assert!(reader.read(&broken).is_none());
        assert!(reader.read(&empty).is_none(), "no description");
        assert!(
            reader.read(&link).is_none(),
            "a link is not the bundle itself"
        );
        assert!(reader.read(&fixture.path("gone.app")).is_none());
    }
}
