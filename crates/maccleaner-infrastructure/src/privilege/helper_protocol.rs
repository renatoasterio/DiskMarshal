//! What the app and the privileged helper say to each other.
//!
//! The app passes rule ids on the command line, never paths or commands. The
//! helper answers with one line of JSON listing what happened to each item.

use std::path::PathBuf;

use maccleaner_domain::deletion::{DeletionOutcome, DeletionStatus, SkipReason};
use maccleaner_domain::shared::ByteSize;
use serde::{Deserialize, Serialize};

/// File name of the helper, which sits beside the app's own executable.
pub const HELPER_NAME: &str = "maccleaner-privileged-helper";
/// Argument that makes the helper check everything and remove nothing.
pub const DRY_RUN_FLAG: &str = "--dry-run";
/// Most rules one request may name.
const MAX_RULES: usize = 16;
/// Longest rule id accepted.
const MAX_ID_LENGTH: usize = 64;

/// What the helper is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperRequest {
    /// The administrator rules to clean.
    pub rule_ids: Vec<String>,
    /// Whether to check everything and remove nothing.
    pub is_dry_run: bool,
}

impl HelperRequest {
    /// Reads a request from command-line arguments.
    ///
    /// # Errors
    ///
    /// Fails, naming nothing the caller sent, when there is no rule, there
    /// are too many, or an argument is not a rule id or the dry-run flag.
    /// Anything shaped like a path or an option is refused.
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, &'static str> {
        let mut request = Self {
            rule_ids: Vec::new(),
            is_dry_run: false,
        };
        for argument in arguments {
            if argument == DRY_RUN_FLAG {
                request.is_dry_run = true;
            } else if is_rule_id(&argument) && !request.rule_ids.contains(&argument) {
                request.rule_ids.push(argument);
            } else {
                return Err("argument is not a rule id");
            }
        }
        match request.rule_ids.len() {
            0 => Err("no rule to clean"),
            count if count > MAX_RULES => Err("too many rules"),
            _ => Ok(request),
        }
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

    use maccleaner_domain::deletion::{
        DeletionOutcome, DeletionStatus, ProtectionReason, SkipReason,
    };
    use maccleaner_domain::shared::ByteSize;

    use super::{DRY_RUN_FLAG, HelperRequest, decode_outcomes, encode_outcomes};

    fn arguments(list: &[&str]) -> Vec<String> {
        list.iter().map(|argument| (*argument).to_owned()).collect()
    }

    #[test]
    fn reads_rule_ids_and_the_dry_run_flag() {
        let request =
            HelperRequest::parse(arguments(&["system-caches", DRY_RUN_FLAG, "system-logs"]))
                .unwrap();
        assert_eq!(request.rule_ids, ["system-caches", "system-logs"]);
        assert!(request.is_dry_run);
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
