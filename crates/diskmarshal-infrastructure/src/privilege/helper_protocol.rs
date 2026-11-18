//! What the app and the privileged helper say to each other.
//!
//! The app asks on the command line, in one of two forms: the ids of the
//! administrator rules to clean, or the identifier of an app to uninstall
//! followed by the items of it to remove. Neither form carries a command,
//! and the items only choose among what the helper finds by itself. The
//! helper answers with one line of JSON listing what happened to each item.

use std::path::{Component, Path, PathBuf};

use diskmarshal_domain::deletion::{DeletionOutcome, DeletionStatus, SkipReason};
use diskmarshal_domain::shared::ByteSize;
use serde::{Deserialize, Serialize};

/// File name of the helper, which sits beside the app's own executable.
pub const HELPER_NAME: &str = "diskmarshal-privileged-helper";
/// Argument that makes the helper check everything and remove nothing.
pub const DRY_RUN_FLAG: &str = "--dry-run";
/// Argument that starts an uninstall: the bundle identifier follows it, then
/// the items to remove.
pub const UNINSTALL_FLAG: &str = "--uninstall";

/// Most rules one request may name.
const MAX_RULES: usize = 16;
/// Longest rule id accepted.
const MAX_ID_LENGTH: usize = 64;
/// Longest bundle identifier accepted, the limit Apple sets for them.
const MAX_BUNDLE_ID_LENGTH: usize = 155;
/// Most items one uninstall may name.
const MAX_ITEMS: usize = 64;
/// Longest item path accepted, the limit macOS sets for paths.
const MAX_PATH_LENGTH: usize = 1_024;

/// The one thing a request asks the helper to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperTask {
    /// Clean the administrator rules with these ids.
    CleanRules(Vec<String>),
    /// Remove items of one app.
    Uninstall {
        /// Bundle identifier of the app.
        bundle_id: String,
        /// The items to remove, each an absolute path.
        items: Vec<PathBuf>,
    },
}

/// What the helper is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperRequest {
    /// What to do.
    pub task: HelperTask,
    /// Whether to check everything and remove nothing.
    pub is_dry_run: bool,
}

impl HelperRequest {
    /// Reads a request from command-line arguments.
    ///
    /// # Errors
    ///
    /// Fails, naming nothing the caller sent, when the arguments are not
    /// one of the two forms. For a cleanup: at least one rule id, none
    /// repeated, nothing shaped like a path or an option. For an uninstall:
    /// a bundle identifier and at least one absolute, plain path.
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, &'static str> {
        let mut is_dry_run = false;
        let mut words = Vec::new();
        for argument in arguments {
            if argument == DRY_RUN_FLAG {
                is_dry_run = true;
            } else {
                words.push(argument);
            }
        }
        let task = match words.split_first() {
            Some((flag, details)) if flag == UNINSTALL_FLAG => Self::uninstall(details)?,
            _ => Self::clean_rules(words)?,
        };
        Ok(Self { task, is_dry_run })
    }

    fn clean_rules(words: Vec<String>) -> Result<HelperTask, &'static str> {
        let mut rule_ids: Vec<String> = Vec::new();
        for word in words {
            if !is_rule_id(&word) || rule_ids.contains(&word) {
                return Err("argument is not a rule id");
            }
            rule_ids.push(word);
        }
        match rule_ids.len() {
            0 => Err("no rule to clean"),
            count if count > MAX_RULES => Err("too many rules"),
            _ => Ok(HelperTask::CleanRules(rule_ids)),
        }
    }

    fn uninstall(details: &[String]) -> Result<HelperTask, &'static str> {
        let Some((bundle_id, items)) = details.split_first() else {
            return Err("no app to uninstall");
        };
        if !is_bundle_id(bundle_id) {
            return Err("argument is not a bundle identifier");
        }
        if items.is_empty() {
            return Err("no item to remove");
        }
        if items.len() > MAX_ITEMS {
            return Err("too many items");
        }
        if !items.iter().all(|item| is_item_path(item)) {
            return Err("argument is not an item path");
        }
        Ok(HelperTask::Uninstall {
            bundle_id: bundle_id.clone(),
            items: items.iter().map(PathBuf::from).collect(),
        })
    }
}

/// A rule id is lowercase words joined by hyphens, such as `system-caches`.
#[must_use]
pub(crate) fn is_rule_id(text: &str) -> bool {
    let is_word_character =
        |character: char| character.is_ascii_lowercase() || character.is_ascii_digit();
    !text.is_empty()
        && text.len() <= MAX_ID_LENGTH
        && text
            .split('-')
            .all(|word| !word.is_empty() && word.chars().all(is_word_character))
}

/// A bundle identifier is letters, digits, dots, hyphens and underscores,
/// starting with a letter or a digit, such as `com.vendor.editor`. Nothing
/// that could be read as an option or as a path fits.
#[must_use]
pub(crate) fn is_bundle_id(text: &str) -> bool {
    let is_allowed =
        |character: char| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_');
    text.len() <= MAX_BUNDLE_ID_LENGTH
        && text.starts_with(|first: char| first.is_ascii_alphanumeric())
        && text.chars().all(is_allowed)
}

/// An item path is absolute, names something below the root folder, and has
/// no `.` or `..` in it and no character that is not printed.
#[must_use]
pub(crate) fn is_item_path(text: &str) -> bool {
    let path = Path::new(text);
    let mut components = path.components();
    text.len() <= MAX_PATH_LENGTH
        && !text.chars().any(char::is_control)
        && components.next() == Some(Component::RootDir)
        && components.clone().next().is_some()
        && components.all(|component| matches!(component, Component::Normal(_)))
        // `Path` drops a `.` in the middle, so the text is checked too.
        && !text.split('/').any(|name| name == ".")
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StatusRecord {
    Deleted,
    Trashed,
    WouldDelete,
    Missing,
    Changed,
    Protected,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeRecord {
    path: String,
    bytes: u64,
    status: StatusRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

/// Writes outcomes as one line of JSON.
#[must_use]
pub fn encode_outcomes(outcomes: &[DeletionOutcome]) -> String {
    let records: Vec<OutcomeRecord> = outcomes
        .iter()
        .map(|outcome| {
            let (status, detail) = match &outcome.status {
                DeletionStatus::Deleted => (StatusRecord::Deleted, None),
                DeletionStatus::Trashed => (StatusRecord::Trashed, None),
                DeletionStatus::WouldDelete => (StatusRecord::WouldDelete, None),
                DeletionStatus::Skipped(SkipReason::Missing) => (StatusRecord::Missing, None),
                DeletionStatus::Skipped(SkipReason::ChangedSincePreview) => {
                    (StatusRecord::Changed, None)
                }
                DeletionStatus::Skipped(SkipReason::Protected(reason)) => {
                    (StatusRecord::Protected, Some(format!("{reason:?}")))
                }
                DeletionStatus::Skipped(SkipReason::Failed(reason)) => {
                    (StatusRecord::Failed, Some(reason.clone()))
                }
            };
            OutcomeRecord {
                path: outcome.target.to_string_lossy().into_owned(),
                bytes: outcome.reclaimable.as_bytes(),
                status,
                detail,
            }
        })
        .collect();
    // Serializing plain strings and numbers cannot fail; an empty list is the
    // honest answer if it ever does.
    serde_json::to_string(&records).unwrap_or_else(|_| "[]".to_owned())
}

/// Reads the outcomes written by [`encode_outcomes`].
///
/// A protected item comes back as a failed one that carries the reason as
/// text: the app only needs to know it was left untouched and why.
///
/// # Errors
///
/// Fails when the text is not in the expected format.
pub fn decode_outcomes(text: &str) -> Result<Vec<DeletionOutcome>, String> {
    let records: Vec<OutcomeRecord> =
        serde_json::from_str(text.trim()).map_err(|error| error.to_string())?;
    Ok(records
        .into_iter()
        .map(|record| {
            let detail = record.detail.unwrap_or_default();
            let status = match record.status {
                StatusRecord::Deleted => DeletionStatus::Deleted,
                StatusRecord::Trashed => DeletionStatus::Trashed,
                StatusRecord::WouldDelete => DeletionStatus::WouldDelete,
                StatusRecord::Missing => DeletionStatus::Skipped(SkipReason::Missing),
                StatusRecord::Changed => DeletionStatus::Skipped(SkipReason::ChangedSincePreview),
                StatusRecord::Protected => {
                    DeletionStatus::Skipped(SkipReason::Failed(format!("protected: {detail}")))
                }
                StatusRecord::Failed => DeletionStatus::Skipped(SkipReason::Failed(detail)),
            };
            DeletionOutcome {
                target: PathBuf::from(record.path),
                status,
                reclaimable: ByteSize::from_bytes(record.bytes),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use diskmarshal_domain::deletion::{
        DeletionOutcome, DeletionStatus, ProtectionReason, SkipReason,
    };
    use diskmarshal_domain::shared::ByteSize;

    use super::{
        DRY_RUN_FLAG, HelperRequest, HelperTask, UNINSTALL_FLAG, decode_outcomes, encode_outcomes,
    };

    fn arguments(list: &[&str]) -> Vec<String> {
        list.iter().map(|argument| (*argument).to_owned()).collect()
    }

    #[test]
    fn reads_rule_ids_and_the_dry_run_flag() {
        let request =
            HelperRequest::parse(arguments(&["system-caches", DRY_RUN_FLAG, "system-logs"]))
                .unwrap();
        assert_eq!(
            request.task,
            HelperTask::CleanRules(vec!["system-caches".to_owned(), "system-logs".to_owned()])
        );
        assert!(request.is_dry_run);
    }

    #[test]
    fn reads_an_uninstall_with_the_app_and_its_items() {
        let request = HelperRequest::parse(arguments(&[
            UNINSTALL_FLAG,
            "com.vendor.Editor-2",
            "/Applications/Editor Pro.app",
            "/Library/LaunchDaemons/com.vendor.Editor-2.plist",
        ]))
        .unwrap();

        assert_eq!(
            request.task,
            HelperTask::Uninstall {
                bundle_id: "com.vendor.Editor-2".to_owned(),
                items: vec![
                    PathBuf::from("/Applications/Editor Pro.app"),
                    PathBuf::from("/Library/LaunchDaemons/com.vendor.Editor-2.plist"),
                ],
            }
        );
        assert!(!request.is_dry_run);
    }

    #[test]
    fn refuses_an_uninstall_that_is_not_an_identifier_and_plain_absolute_paths() {
        let uninstall = |details: &[&str]| {
            let mut words = vec![UNINSTALL_FLAG];
            words.extend(details);
            HelperRequest::parse(arguments(&words))
        };
        let app = "/Applications/Editor.app";

        assert!(uninstall(&["com.vendor.editor", app]).is_ok());
        assert!(uninstall(&[]).is_err(), "no app");
        assert!(uninstall(&["com.vendor.editor"]).is_err(), "no item");
        for bad_identifier in [
            "",
            "-x",
            ".hidden",
            "com vendor",
            "com/vendor",
            "a;b",
            "$(id)",
        ] {
            assert!(
                uninstall(&[bad_identifier, app]).is_err(),
                "{bad_identifier:?}"
            );
        }
        for bad_item in [
            "",
            "/",
            "Applications/Editor.app",
            "/Applications/../etc",
            "/Applications/./Editor.app",
            "/Applications/Editor.app\n/etc",
            "--force",
        ] {
            assert!(
                uninstall(&["com.vendor.editor", bad_item]).is_err(),
                "{bad_item:?}"
            );
        }
        let long = format!("/{}", "a".repeat(1_100));
        assert!(uninstall(&["com.vendor.editor", &long]).is_err());
        let many: Vec<String> = (0..65).map(|index| format!("/Library/x/{index}")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        assert!(uninstall(&[&["com.vendor.editor"], many.as_slice()].concat()).is_err());
        assert!(
            HelperRequest::parse(arguments(&["system-caches", UNINSTALL_FLAG])).is_err(),
            "the two forms do not mix"
        );
    }

    #[test]
    fn refuses_anything_that_is_not_a_rule_id() {
        for bad in [
            "/Library/Caches",
            "../etc",
            "system caches",
            "System-Caches",
            "--force",
            "-x",
            "a;rm",
            "",
            "double--hyphen",
            "$(id)",
        ] {
            assert!(
                HelperRequest::parse(arguments(&["system-caches", bad])).is_err(),
                "{bad:?}"
            );
        }
        assert!(
            HelperRequest::parse(arguments(&[])).is_err(),
            "nothing to do"
        );
        assert!(
            HelperRequest::parse(arguments(&[DRY_RUN_FLAG])).is_err(),
            "still nothing to do"
        );
        assert!(
            HelperRequest::parse(arguments(&["a", "a"])).is_err(),
            "a rule named twice"
        );
        let many: Vec<String> = (0..17).map(|index| format!("rule-{index}")).collect();
        assert!(HelperRequest::parse(many).is_err());
    }

    #[test]
    fn outcomes_survive_the_trip_between_helper_and_app() {
        let outcome = |name: &str, status| DeletionOutcome {
            target: PathBuf::from("/Library/Caches").join(name),
            status,
            reclaimable: ByteSize::from_bytes(4_096),
        };
        let sent = [
            outcome("a", DeletionStatus::Deleted),
            outcome("b", DeletionStatus::WouldDelete),
            outcome("c", DeletionStatus::Skipped(SkipReason::Missing)),
            outcome(
                "d",
                DeletionStatus::Skipped(SkipReason::ChangedSincePreview),
            ),
            outcome(
                "e",
                DeletionStatus::Skipped(SkipReason::Failed("Permission denied".to_owned())),
            ),
            outcome(
                "f",
                DeletionStatus::Skipped(SkipReason::Protected(ProtectionReason::InsidePackage)),
            ),
        ];

        let received = decode_outcomes(&encode_outcomes(&sent)).unwrap();

        assert_eq!(received[..5], sent[..5]);
        assert_eq!(received[5].target, sent[5].target);
        assert!(!received[5].is_removed());
        assert!(matches!(
            &received[5].status,
            DeletionStatus::Skipped(SkipReason::Failed(reason)) if reason.contains("InsidePackage")
        ));
    }

    #[test]
    fn refuses_an_answer_in_another_format() {
        assert!(decode_outcomes("not json").is_err());
        assert!(
            decode_outcomes(r#"[{"path":"/x","bytes":1,"status":"deleted","extra":1}]"#).is_err()
        );
        assert!(decode_outcomes(r#"[{"path":"/x","bytes":1,"status":"exploded"}]"#).is_err());
        assert!(decode_outcomes("[]").unwrap().is_empty());
    }
}
