//! The rule that decides whether an item in a library folder belongs to an app.

use crate::applications::InstalledApp;
use crate::cleanup::RiskLevel;

/// Decides which items of a library folder belong to one app.
///
/// An item belongs to the app when its name is the app's bundle identifier,
/// alone or followed by a dot and something else. Names are compared without
/// regard to letter case. The answer says how sure the match is:
///
/// - [`RiskLevel::Safe`]: the name is the identifier itself, with at most an
///   extension macOS adds (`.plist`, `.savedState`, `.binarycookies`) or a
///   prefix macOS adds to shared containers (`group.`, a team identifier).
/// - [`RiskLevel::Review`]: the name only starts with the identifier, or is
///   the name of the app. Something else could be using that name.
#[derive(Debug, Clone)]
pub struct LeftoverMatcher {
    identifier: String,
    names: Vec<String>,
    /// Identifiers of other installed apps that start with this app's own.
    longer_identifiers: Vec<String>,
}

impl LeftoverMatcher {
    /// A shorter identifier says too little to tell an app's items apart.
    const MINIMUM_IDENTIFIER_LENGTH: usize = 3;
    /// Extensions macOS adds to an identifier to name a file of the app.
    const SYSTEM_EXTENSIONS: [&'static str; 3] = ["plist", "savedstate", "binarycookies"];
    /// What macOS puts before the identifier of a container shared by apps.
    const GROUP_PREFIX: &'static str = "group.";
    /// Length of the team identifier Apple gives each developer.
    const TEAM_IDENTIFIER_LENGTH: usize = 10;

    /// Builds the rule for `app`. `installed` are all the apps on this Mac:
    /// an item that carries the longer identifier of another one of them
    /// belongs to that app, not to this one.
    #[must_use]
    pub fn new(app: &InstalledApp, installed: &[InstalledApp]) -> Self {
        let identifier = app.bundle_id.to_lowercase();
        let extension = format!("{identifier}.");
        let longer_identifiers = installed
            .iter()
            .map(|other| other.bundle_id.to_lowercase())
            .filter(|other| other.starts_with(&extension))
            .collect();
        let names = std::iter::once(&app.name)
            .chain(&app.declared_name)
            .map(|name| name.to_lowercase())
            .filter(|name| !name.is_empty())
            .collect();
        Self {
            identifier,
            names,
            longer_identifiers,
        }
    }

    /// Says whether the item called `item_name` belongs to the app and how
    /// sure that is. `is_matched_by_name` tells whether the folder the item
    /// is in also accepts items named like the app.
    #[must_use]
    pub fn classify(&self, item_name: &str, is_matched_by_name: bool) -> Option<RiskLevel> {
        let item = item_name.to_lowercase();
        self.classify_by_identifier(&item).or_else(|| {
            (is_matched_by_name && self.names.contains(&item)).then_some(RiskLevel::Review)
        })
    }

    fn classify_by_identifier(&self, item: &str) -> Option<RiskLevel> {
        if self.identifier.len() < Self::MINIMUM_IDENTIFIER_LENGTH {
            return None;
        }
        Self::spellings(item)
            .into_iter()
            .find_map(|spelling| self.classify_spelling(spelling))
    }

    /// The name as it is and without the prefixes macOS puts before the
    /// identifier of a shared container.
    fn spellings(item: &str) -> Vec<&str> {
        let without_team = Self::without_team_identifier(item);
        let without_group = [Some(item), without_team]
            .into_iter()
            .flatten()
            .filter_map(|spelling| spelling.strip_prefix(Self::GROUP_PREFIX));
        std::iter::once(item)
            .chain(without_team)
            .chain(without_group)
            .collect()
    }

    fn without_team_identifier(item: &str) -> Option<&str> {
        let (team, rest) = item.split_at_checked(Self::TEAM_IDENTIFIER_LENGTH)?;
        let rest = rest.strip_prefix('.')?;
        team.bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
            .then_some(rest)
    }

    fn classify_spelling(&self, spelling: &str) -> Option<RiskLevel> {
        let rest = spelling.strip_prefix(self.identifier.as_str())?;
        if rest.is_empty() {
            return Some(RiskLevel::Safe);
        }
        let suffix = rest.strip_prefix('.')?;
        if Self::SYSTEM_EXTENSIONS.contains(&suffix) {
            return Some(RiskLevel::Safe);
        }
        let belongs_to_another_app = self.longer_identifiers.iter().any(|other| {
            spelling
                .strip_prefix(other.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
        });
        (!belongs_to_another_app).then_some(RiskLevel::Review)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LeftoverMatcher;
    use crate::applications::InstalledApp;
    use crate::cleanup::{PrivilegeScope, RiskLevel};
    use crate::shared::ByteSize;

    fn app(name: &str, bundle_id: &str) -> InstalledApp {
        InstalledApp {
            name: name.into(),
            declared_name: None,
            bundle_id: bundle_id.into(),
            version: None,
            path: PathBuf::from(format!("/Applications/{name}.app")),
            size: ByteSize::ZERO,
            scope: PrivilegeScope::User,
            is_running: false,
        }
    }

    fn editor() -> LeftoverMatcher {
        let editor = app("Editor", "com.vendor.Editor");
        LeftoverMatcher::new(&editor, std::slice::from_ref(&editor))
    }

    #[test]
    fn the_identifier_itself_is_a_sure_match() {
        for item in [
            "com.vendor.Editor",
            "com.vendor.editor",
            "com.vendor.Editor.plist",
            "com.vendor.Editor.savedState",
            "com.vendor.Editor.binarycookies",
        ] {
            assert_eq!(
                editor().classify(item, false),
                Some(RiskLevel::Safe),
                "{item}"
            );
        }
    }

    #[test]
    fn shared_containers_of_the_app_are_a_sure_match() {
        for item in [
            "group.com.vendor.Editor",
            "AB12CD34EF.com.vendor.Editor",
            "AB12CD34EF.group.com.vendor.Editor",
        ] {
            assert_eq!(
                editor().classify(item, false),
                Some(RiskLevel::Safe),
                "{item}"
            );
        }
        assert_eq!(editor().classify("AB12CD34EF.Suite", false), None);
        assert_eq!(
            editor().classify("too-short.com.vendor.Editor", false),
            None
        );
    }

    #[test]
    fn a_name_that_only_starts_with_the_identifier_needs_review() {
        assert_eq!(
            editor().classify("com.vendor.Editor.helper", false),
            Some(RiskLevel::Review)
        );
        assert_eq!(
            editor().classify("com.vendor.Editor.0F1E2D3C.plist", false),
            Some(RiskLevel::Review)
        );
    }

    #[test]
    fn a_different_identifier_never_matches() {
        for item in [
            "com.vendor.Editor2",
            "com.vendor.EditorPro.plist",
            "com.vendor",
            "com.vendor.Viewer",
            "org.other.com.vendor.Editor",
        ] {
            assert_eq!(editor().classify(item, false), None, "{item}");
        }
    }

    #[test]
    fn an_item_of_another_installed_app_is_left_to_that_app() {
        let editor = app("Editor", "com.vendor.Editor");
        let beta = app("Editor Beta", "com.vendor.Editor.beta");
        let matcher = LeftoverMatcher::new(&editor, &[editor.clone(), beta]);

        assert_eq!(matcher.classify("com.vendor.Editor.beta", false), None);
        assert_eq!(
            matcher.classify("com.vendor.Editor.beta.plist", false),
            None
        );
        assert_eq!(
            matcher.classify("com.vendor.Editor.betamax", false),
            Some(RiskLevel::Review),
            "not the other app's identifier, only its beginning"
        );
        assert_eq!(
            matcher.classify("com.vendor.Editor.plist", false),
            Some(RiskLevel::Safe)
        );
    }

    #[test]
    fn the_name_of_the_app_counts_only_where_the_folder_allows_and_needs_review() {
        assert_eq!(editor().classify("Editor", false), None);
        assert_eq!(editor().classify("editor", true), Some(RiskLevel::Review));
        assert_eq!(editor().classify("Editor Backup", true), None);

        let mut renamed = app("Editor", "com.vendor.Editor");
        renamed.declared_name = Some("Vendor Editor".into());
        let matcher = LeftoverMatcher::new(&renamed, &[]);
        assert_eq!(
            matcher.classify("Vendor Editor", true),
            Some(RiskLevel::Review)
        );
    }

    #[test]
    fn an_identifier_too_short_to_tell_apps_apart_matches_nothing() {
        let tiny = app("Go", "go");
        let matcher = LeftoverMatcher::new(&tiny, &[]);
        assert_eq!(matcher.classify("go", false), None);
        assert_eq!(matcher.classify("go.plist", false), None);
        assert_eq!(matcher.classify("Go", true), Some(RiskLevel::Review));
    }
}
