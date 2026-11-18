//! One thing Docker holds that nothing is using.

use std::path::PathBuf;

use crate::cleanup::RiskLevel;
use crate::docker::ResourceKind;
use crate::shared::ByteSize;

/// A Docker resource no container uses, offered for removal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerResource {
    /// What the resource is.
    pub kind: ResourceKind,
    /// What Docker removes the resource by: an image reference or
    /// identifier, a container identifier, a volume name. Empty for the
    /// build cache, which is removed as a whole.
    pub reference: String,
    /// Name shown to the user. Empty when the resource has none, as with an
    /// image left behind by a build.
    pub name: String,
    /// Space Docker reports for the resource. Images share layers, so
    /// removing one can free less than this.
    pub size: ByteSize,
    /// When it was created, in seconds since the Unix epoch, when known.
    pub created_unix_seconds: Option<i64>,
}

impl DockerResource {
    /// How careful the user should be before removing the resource.
    ///
    /// The build cache and images without a name are leftovers of builds
    /// and are made again by the next one. Everything else may hold
    /// something the user wants: an image that exists nowhere else, the
    /// files a container wrote, the database inside a volume.
    #[must_use]
    pub fn risk(&self) -> RiskLevel {
        match self.kind {
            ResourceKind::BuildCache => RiskLevel::Safe,
            ResourceKind::Image if self.name.is_empty() => RiskLevel::Safe,
            ResourceKind::Image | ResourceKind::Container | ResourceKind::Volume => {
                RiskLevel::Review
            }
        }
    }

    /// The label under which the removal of this resource is recorded in
    /// the history, such as `docker/image/web:dev`. It is not a path on disk.
    #[must_use]
    pub fn history_label(&self) -> PathBuf {
        let name = if self.name.is_empty() {
            self.reference.as_str()
        } else {
            self.name.as_str()
        };
        let mut label = PathBuf::from("docker");
        label.push(self.kind.slug());
        if !name.is_empty() {
            // The name goes in as one piece: it may hold `/` and must not be
            // read as folders, nor as an absolute path that replaces the label.
            label.push(name.replace('/', ":"));
        }
        label
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::DockerResource;
    use crate::cleanup::RiskLevel;
    use crate::docker::ResourceKind;
    use crate::shared::ByteSize;

    fn resource(kind: ResourceKind, reference: &str, name: &str) -> DockerResource {
        DockerResource {
            kind,
            reference: reference.into(),
            name: name.into(),
            size: ByteSize::from_bytes(1_000),
            created_unix_seconds: None,
        }
    }

    #[test]
    fn only_build_leftovers_are_safe() {
        let risk = |kind, name| resource(kind, "ref", name).risk();
        assert_eq!(risk(ResourceKind::BuildCache, ""), RiskLevel::Safe);
        assert_eq!(risk(ResourceKind::Image, ""), RiskLevel::Safe);
        assert_eq!(risk(ResourceKind::Image, "web:dev"), RiskLevel::Review);
        assert_eq!(risk(ResourceKind::Container, "web-1"), RiskLevel::Review);
        assert_eq!(risk(ResourceKind::Volume, "pgdata"), RiskLevel::Review);
    }

    #[test]
    fn labels_the_history_without_pretending_to_be_a_path_on_disk() {
        assert_eq!(
            resource(ResourceKind::Image, "web:dev", "web:dev").history_label(),
            PathBuf::from("docker/image/web:dev")
        );
        assert_eq!(
            resource(ResourceKind::Image, "sha256:ab12", "").history_label(),
            PathBuf::from("docker/image/sha256:ab12")
        );
        assert_eq!(
            resource(ResourceKind::Image, "/etc/x", "registry.test/team/web:1").history_label(),
            PathBuf::from("docker/image/registry.test:team:web:1")
        );
        assert_eq!(
            resource(ResourceKind::BuildCache, "", "").history_label(),
            PathBuf::from("docker/build-cache")
        );
    }
}
