//! Finding removable project folders in a scanned tree.

use std::path::Path;

use crate::dev_artifacts::{ArtifactRule, ProjectArtifact};
use crate::scan::{EntryKind, FileNode, FileTree, NodeId};

/// Finds every removable folder under the root of `tree`.
///
/// `root_path` is the absolute path the tree was scanned from. The search
/// never looks inside a folder it has just offered, inside packages, or
/// inside hidden folders, so the dependencies of a project are never
/// mistaken for projects of their own.
#[must_use]
pub fn find_artifacts(
    tree: &FileTree,
    root_path: &Path,
    rules: &[ArtifactRule],
) -> Vec<ProjectArtifact> {
    let mut found = Vec::new();
    let mut pending = vec![FileTree::ROOT];
    while let Some(folder) = pending.pop() {
        let children: Vec<(NodeId, &FileNode)> = tree
            .children(folder)
            .filter_map(|id| tree.node(id).map(|node| (id, node)))
            .collect();
        let matched: Vec<(&ArtifactRule, Option<i64>)> = rules
            .iter()
            .filter_map(|rule| marker_of(rule, &children).map(|modified| (rule, modified)))
            .collect();

        for (id, node) in &children {
            if node.kind() != EntryKind::Directory {
                continue;
            }
            let offered = matched.iter().find_map(|(rule, modified)| {
                rule.folder(node.name())
                    .map(|artifact| (rule, artifact, *modified))
            });
            match offered {
                Some((rule, artifact, modified)) => found.push(ProjectArtifact {
                    project: tree
                        .node(folder)
                        .map_or_else(String::new, |n| last_name(n.name())),
                    ecosystem: rule.ecosystem,
                    folder: artifact.name.clone(),
                    path: path_of(tree, *id, root_path),
                    size: node.allocated_size(),
                    project_modified_unix_seconds: modified,
                    risk: artifact.risk,
                }),
                None if !node.name().starts_with('.') => pending.push(*id),
                None => {}
            }
        }
    }
    found.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.path.cmp(&b.path)));
    found
}

/// When one of the rule's markers is among `children`, returns `Some` with
/// that file's modification time.
fn marker_of(rule: &ArtifactRule, children: &[(NodeId, &FileNode)]) -> Option<Option<i64>> {
    children
        .iter()
        .find(|(_, node)| node.kind() == EntryKind::File && rule.is_marker(node.name()))
        .map(|(_, node)| node.modified_unix_seconds())
}

/// The root node is named by its full path; everything else by its own name.
fn last_name(name: &str) -> String {
    Path::new(name).file_name().map_or_else(
        || name.to_owned(),
        |last| last.to_string_lossy().into_owned(),
    )
}

fn path_of(tree: &FileTree, id: NodeId, root_path: &Path) -> std::path::PathBuf {
    let mut path = root_path.to_path_buf();
    path.extend(tree.path_segments(id).into_iter().skip(1));
    path
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::find_artifacts;
    use crate::cleanup::RiskLevel;
    use crate::dev_artifacts::{ArtifactFolder, ArtifactRule, Ecosystem};
    use crate::scan::{FileEntry, FileTree, NodeId};
    use crate::shared::ByteSize;

    fn rule(ecosystem: Ecosystem, markers: &[&str], folders: &[(&str, RiskLevel)]) -> ArtifactRule {
        ArtifactRule {
            ecosystem,
            markers: markers.iter().map(|marker| (*marker).to_owned()).collect(),
            folders: folders
                .iter()
                .map(|(name, risk)| ArtifactFolder {
                    name: (*name).to_owned(),
                    risk: *risk,
                })
                .collect(),
        }
    }

    fn rules() -> Vec<ArtifactRule> {
        vec![
            rule(
                Ecosystem::Node,
                &["package.json"],
                &[
                    ("node_modules", RiskLevel::Safe),
                    ("dist", RiskLevel::Review),
                ],
            ),
            rule(
                Ecosystem::DotNet,
                &["*.csproj", "*.sln"],
                &[
                    ("bin", RiskLevel::Safe),
                    ("obj", RiskLevel::Safe),
                    ("packages", RiskLevel::Review),
                ],
            ),
        ]
    }

    fn folder(tree: &mut FileTree, parent: NodeId, name: &str) -> NodeId {
        tree.add_child(parent, FileEntry::directory(name)).unwrap()
    }

    fn file(tree: &mut FileTree, parent: NodeId, name: &str, bytes: u64) {
        tree.add_child(parent, FileEntry::file(name, ByteSize::from_bytes(bytes)))
            .unwrap();
    }

    fn find(mut tree: FileTree) -> Vec<(String, String, u64)> {
        tree.aggregate();
        find_artifacts(&tree, Path::new("/work"), &rules())
            .into_iter()
            .map(|a| (a.project, a.folder, a.size.as_bytes()))
            .collect()
    }

    #[test]
    fn offers_a_folder_only_next_to_its_marker() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        let web = folder(&mut tree, FileTree::ROOT, "web");
        file(&mut tree, web, "package.json", 1);
        let modules = folder(&mut tree, web, "node_modules");
        file(&mut tree, modules, "lib.js", 500);
        let loose = folder(&mut tree, FileTree::ROOT, "notes");
        let stray = folder(&mut tree, loose, "node_modules");
        file(&mut tree, stray, "x", 900);

        assert_eq!(
            find(tree),
            [("web".to_owned(), "node_modules".to_owned(), 500)]
        );
    }

    #[test]
    fn never_treats_a_dependency_as_a_project() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        let web = folder(&mut tree, FileTree::ROOT, "web");
        file(&mut tree, web, "package.json", 1);
        let modules = folder(&mut tree, web, "node_modules");
        let dependency = folder(&mut tree, modules, "left-pad");
        file(&mut tree, dependency, "package.json", 1);
        let nested = folder(&mut tree, dependency, "node_modules");
        file(&mut tree, nested, "y", 50);

        let found = find(tree);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "web");
    }

    #[test]
    fn markers_can_match_by_extension_and_ignore_case() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        let api = folder(&mut tree, FileTree::ROOT, "api");
        file(&mut tree, api, "Billing.CSPROJ", 1);
        let bin = folder(&mut tree, api, "bin");
        file(&mut tree, bin, "app.dll", 300);
        let other = folder(&mut tree, FileTree::ROOT, "scripts");
        let scripts_bin = folder(&mut tree, other, "bin");
        file(&mut tree, scripts_bin, "deploy.sh", 10);

        assert_eq!(find(tree), [("api".to_owned(), "bin".to_owned(), 300)]);
    }

    #[test]
    fn reports_absolute_paths_risk_and_largest_first() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        let web = folder(&mut tree, FileTree::ROOT, "web");
        file(&mut tree, web, "package.json", 1);
        let dist = folder(&mut tree, web, "dist");
        file(&mut tree, dist, "bundle.js", 100);
        let modules = folder(&mut tree, web, "node_modules");
        file(&mut tree, modules, "lib.js", 700);
        tree.aggregate();

        let found = find_artifacts(&tree, Path::new("/work"), &rules());

        assert_eq!(found[0].path, PathBuf::from("/work/web/node_modules"));
        assert_eq!(found[0].risk, RiskLevel::Safe);
        assert_eq!(found[1].path, PathBuf::from("/work/web/dist"));
        assert_eq!(found[1].risk, RiskLevel::Review);
    }

    #[test]
    fn does_not_search_inside_hidden_folders() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        let hidden = folder(&mut tree, FileTree::ROOT, ".nvm");
        let version = folder(&mut tree, hidden, "v20");
        file(&mut tree, version, "package.json", 1);
        let modules = folder(&mut tree, version, "node_modules");
        file(&mut tree, modules, "npm.js", 400);

        assert!(find(tree).is_empty());
    }

    #[test]
    fn the_scanned_folder_itself_can_be_a_project() {
        let mut tree = FileTree::new(FileEntry::directory("/work"));
        file(&mut tree, FileTree::ROOT, "package.json", 1);
        let modules = folder(&mut tree, FileTree::ROOT, "node_modules");
        file(&mut tree, modules, "lib.js", 60);

        assert_eq!(
            find(tree),
            [("work".to_owned(), "node_modules".to_owned(), 60)]
        );
    }
}
