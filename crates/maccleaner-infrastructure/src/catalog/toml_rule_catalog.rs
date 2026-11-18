//! Rule catalog read from the TOML files embedded in the binary.

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use maccleaner_application::ApplicationError;
use maccleaner_application::ports::RuleCatalog;
use maccleaner_domain::cleanup::{CleanupRule, PrivilegeScope, RiskLevel};
use maccleaner_domain::deletion::DeletionMethod;
use maccleaner_domain::dev_artifacts::{ArtifactFolder, ArtifactRule, Ecosystem};
use serde::Deserialize;

const CLEANUP_RULES: &str = include_str!("../../resources/cleanup_rules.toml");
const ARTIFACT_RULES: &str = include_str!("../../resources/artifact_rules.toml");
const HOME_PREFIX: &str = "~/";
const TEMPORARY_FOLDER: &str = "$TMPDIR";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    rule: Vec<RuleEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleEntry {
    id: String,
    name: String,
    description: String,
    locations: Vec<String>,
    risk: RiskEntry,
    scope: ScopeEntry,
    method: MethodEntry,
    #[serde(default)]
    minimum_age_days: Option<u32>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum RiskEntry {
    Safe,
    Review,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ScopeEntry {
    User,
    Administrator,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum MethodEntry {
    Permanent,
    Trash,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactFile {
    project: Vec<ProjectEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectEntry {
    ecosystem: EcosystemEntry,
    markers: Vec<String>,
    #[serde(default)]
    safe: Vec<String>,
    #[serde(default)]
    review: Vec<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum EcosystemEntry {
    Node,
    Dotnet,
    Rust,
    Jvm,
    Python,
    Swift,
    Flutter,
    Go,
    Php,
    Terraform,
}

/// Reads the catalog of code projects.
fn parse_artifact_rules(source: &str) -> Result<Vec<ArtifactRule>, String> {
    let file: ArtifactFile = toml::from_str(source).map_err(|error| error.to_string())?;
    file.project
        .into_iter()
        .map(|entry| {
            if entry.markers.is_empty() || (entry.safe.is_empty() && entry.review.is_empty()) {
                return Err("a project needs at least one marker and one folder".to_owned());
            }
            let folders = |names: Vec<String>, risk: RiskLevel| {
                names
                    .into_iter()
                    .map(move |name| ArtifactFolder { name, risk })
            };
            Ok(ArtifactRule {
                ecosystem: match entry.ecosystem {
                    EcosystemEntry::Node => Ecosystem::Node,
                    EcosystemEntry::Dotnet => Ecosystem::DotNet,
                    EcosystemEntry::Rust => Ecosystem::Rust,
                    EcosystemEntry::Jvm => Ecosystem::Jvm,
                    EcosystemEntry::Python => Ecosystem::Python,
                    EcosystemEntry::Swift => Ecosystem::Swift,
                    EcosystemEntry::Flutter => Ecosystem::Flutter,
                    EcosystemEntry::Go => Ecosystem::Go,
                    EcosystemEntry::Php => Ecosystem::Php,
                    EcosystemEntry::Terraform => Ecosystem::Terraform,
                },
                markers: entry.markers,
                folders: folders(entry.safe, RiskLevel::Safe)
                    .chain(folders(entry.review, RiskLevel::Review))
                    .collect(),
            })
        })
        .collect()
}

/// Supplies the embedded catalogs, with `~` and `$TMPDIR` replaced by the
/// folders of one user.
#[derive(Debug, Clone)]
pub struct TomlRuleCatalog {
    home: PathBuf,
    temporary: Option<PathBuf>,
}

impl TomlRuleCatalog {
    /// Creates a catalog for a user whose home folder is `home` and whose
    /// temporary folder is `temporary`.
    #[must_use]
    pub fn new(home: PathBuf, temporary: Option<PathBuf>) -> Self {
        Self { home, temporary }
    }

    /// Creates a catalog for the current user, from `HOME` and `TMPDIR`.
    /// Returns `None` when the home folder is unknown.
    #[must_use]
    pub fn for_current_user() -> Option<Self> {
        let home = env::var_os("HOME").map(PathBuf::from)?;
        Some(Self::new(home, env::var_os("TMPDIR").map(PathBuf::from)))
    }

    fn parse(&self, source: &str) -> Result<Vec<CleanupRule>, String> {
        let file: CatalogFile = toml::from_str(source).map_err(|error| error.to_string())?;
        let mut seen = HashSet::new();
        let mut rules = Vec::with_capacity(file.rule.len());
        for entry in file.rule {
            if !seen.insert(entry.id.clone()) {
                return Err(format!("rule `{}` appears twice", entry.id));
            }
            if entry.locations.is_empty() {
                return Err(format!("rule `{}` has no folder", entry.id));
            }
            let locations = entry
                .locations
                .iter()
                .map(|template| self.expand(template, &entry.id))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect();
            rules.push(CleanupRule {
                id: entry.id,
                name: entry.name,
                description: entry.description,
                locations,
                risk: match entry.risk {
                    RiskEntry::Safe => RiskLevel::Safe,
                    RiskEntry::Review => RiskLevel::Review,
                },
                scope: match entry.scope {
                    ScopeEntry::User => PrivilegeScope::User,
                    ScopeEntry::Administrator => PrivilegeScope::Administrator,
                },
                method: match entry.method {
                    MethodEntry::Permanent => DeletionMethod::Permanent,
                    MethodEntry::Trash => DeletionMethod::Trash,
                },
                minimum_age_days: entry.minimum_age_days,
            });
        }
        Ok(rules)
    }

    /// Makes a folder of the catalog absolute. `Ok(None)` drops a folder that
    /// does not exist for this user, such as `$TMPDIR` when it is not set.
    fn expand(&self, template: &str, rule: &str) -> Result<Option<PathBuf>, String> {
        if template == TEMPORARY_FOLDER {
            return Ok(self.temporary.clone());
        }
        if let Some(relative) = template.strip_prefix(HOME_PREFIX) {
            return Ok(Some(self.home.join(relative)));
        }
        if Path::new(template).is_absolute() {
            return Ok(Some(PathBuf::from(template)));
        }
        Err(format!(
            "rule `{rule}`: `{template}` is not an absolute folder"
        ))
    }
}

impl RuleCatalog for TomlRuleCatalog {
    fn cleanup_rules(&self) -> Result<Vec<CleanupRule>, ApplicationError> {
        self.parse(CLEANUP_RULES)
            .map_err(ApplicationError::InvalidCatalog)
    }

    fn artifact_rules(&self) -> Result<Vec<ArtifactRule>, ApplicationError> {
        parse_artifact_rules(ARTIFACT_RULES).map_err(ApplicationError::InvalidCatalog)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use maccleaner_application::ports::RuleCatalog;
    use maccleaner_domain::cleanup::{PrivilegeScope, RiskLevel};
    use maccleaner_domain::deletion::{DeletionMethod, ProtectedPaths};

    use super::TomlRuleCatalog;

    const HOME: &str = "/Users/tester";

    fn catalog() -> TomlRuleCatalog {
        TomlRuleCatalog::new(
            PathBuf::from(HOME),
            Some(PathBuf::from("/private/tmp/tester")),
        )
    }

    #[test]
    fn the_embedded_catalog_is_valid_and_fully_absolute() {
        let rules = catalog().cleanup_rules().unwrap();

        assert!(rules.len() >= 10);
        for rule in &rules {
            assert!(
                !rule.name.is_empty() && !rule.locations.is_empty(),
                "{}",
                rule.id
            );
            assert!(
                rule.locations.iter().all(|path| path.is_absolute()),
                "{}",
                rule.id
            );
        }
        let caches = rules.iter().find(|rule| rule.id == "app-caches").unwrap();
        assert_eq!(
            caches.locations,
            [PathBuf::from("/Users/tester/Library/Caches")]
        );
        let temporary = rules
            .iter()
            .find(|rule| rule.id == "temporary-files")
            .unwrap();
        assert_eq!(temporary.locations, [PathBuf::from("/private/tmp/tester")]);
        assert_eq!(temporary.minimum_age_days, Some(7));
    }

    #[test]
    fn no_folder_is_inside_the_folder_of_another_rule() {
        let rules = catalog().cleanup_rules().unwrap();
        let folders: Vec<(&str, &Path)> = rules
            .iter()
            .flat_map(|rule| {
                rule.locations
                    .iter()
                    .map(|path| (rule.id.as_str(), path.as_path()))
            })
            .collect();
        for (index, (rule, folder)) in folders.iter().enumerate() {
            for (other_rule, other) in folders.iter().skip(index + 1) {
                assert!(
                    !folder.starts_with(other) && !other.starts_with(folder),
                    "{rule} and {other_rule} overlap: {} / {}",
                    folder.display(),
                    other.display()
                );
            }
        }
    }

    #[test]
    fn what_every_rule_removes_is_allowed_by_the_deletion_policy() {
        let policy = ProtectedPaths::new(Path::new(HOME));
        for rule in catalog().cleanup_rules().unwrap() {
            for folder in &rule.locations {
                let item = folder.join("some-item");
                assert_eq!(
                    policy.check(&item, std::slice::from_ref(folder)),
                    Ok(()),
                    "{}",
                    rule.id
                );
            }
        }
    }

    #[test]
    fn emptying_the_trash_is_never_offered_as_safe() {
        // The Trash is where duplicates and photos go so they can be
        // recovered. A safe rule is selected by default; this one must not be.
        let rules = catalog().cleanup_rules().unwrap();
        let trash = rules.iter().find(|rule| rule.id == "trash").unwrap();
        assert_eq!(trash.risk, RiskLevel::Review);
    }

    #[test]
    fn review_rules_and_administrator_rules_are_marked() {
        let rules = catalog().cleanup_rules().unwrap();
        let archives = rules
            .iter()
            .find(|rule| rule.id == "xcode-archives")
            .unwrap();
        assert_eq!(archives.risk, RiskLevel::Review);
        assert_eq!(archives.method, DeletionMethod::Trash);
        let system = rules
            .iter()
            .find(|rule| rule.id == "system-caches")
            .unwrap();
        assert_eq!(system.scope, PrivilegeScope::Administrator);
    }

    #[test]
    fn a_missing_temporary_folder_drops_only_that_folder() {
        let rules = TomlRuleCatalog::new(PathBuf::from(HOME), None)
            .cleanup_rules()
            .unwrap();
        let temporary = rules
            .iter()
            .find(|rule| rule.id == "temporary-files")
            .unwrap();
        assert!(temporary.locations.is_empty());
    }

    #[test]
    fn rejects_unknown_fields_duplicates_and_relative_folders() {
        let rule = |extra: &str, folder: &str| {
            format!(
                "[[rule]]\nid = \"a\"\nname = \"A\"\ndescription = \"\"\nlocations = [\"{folder}\"]\n\
                 risk = \"safe\"\nscope = \"user\"\nmethod = \"permanent\"\n{extra}\n"
            )
        };
        let catalog = catalog();
        assert!(catalog.parse(&rule("", "~/x")).is_ok());
        assert!(catalog.parse(&rule("command = \"rm\"", "~/x")).is_err());
        assert!(catalog.parse(&rule("", "relative/x")).is_err());
        assert!(
            catalog
                .parse(&(rule("", "~/x") + &rule("", "~/y")))
                .is_err()
        );
    }

    #[test]
    fn the_embedded_project_catalog_is_valid() {
        let rules = catalog().artifact_rules().unwrap();
        assert!(rules.len() >= 10);
        let node = rules
            .iter()
            .find(|rule| rule.is_marker("package.json"))
            .unwrap();
        assert_eq!(node.folder("node_modules").unwrap().risk, RiskLevel::Safe);
        assert_eq!(node.folder("dist").unwrap().risk, RiskLevel::Review);
        let dotnet = rules
            .iter()
            .find(|rule| rule.is_marker("Api.csproj"))
            .unwrap();
        assert!(dotnet.folder("obj").is_some());
    }

    #[test]
    fn a_project_without_markers_or_folders_is_rejected() {
        let project = |body: &str| format!("[[project]]\n{body}\n");
        let parse = super::parse_artifact_rules;
        assert!(
            parse(&project(
                "ecosystem = \"rust\"\nmarkers = []\nsafe = [\"target\"]"
            ))
            .is_err()
        );
        assert!(parse(&project("ecosystem = \"rust\"\nmarkers = [\"Cargo.toml\"]")).is_err());
        assert!(
            parse(&project(
                "ecosystem = \"cobol\"\nmarkers = [\"x\"]\nsafe = [\"y\"]"
            ))
            .is_err()
        );
    }
}
