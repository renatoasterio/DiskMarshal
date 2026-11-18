//! Everything Docker reports about its own disk use.

use crate::docker::{DockerResource, ResourceKind};
use crate::shared::ByteSize;

/// How much Docker holds of one kind of resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindUsage {
    /// The kind these figures are about.
    pub kind: ResourceKind,
    /// How many resources of the kind exist.
    pub total_count: u64,
    /// How many of them are in use.
    pub active_count: u64,
    /// Space all of them occupy.
    pub size: ByteSize,
    /// Space Docker says removing the unused ones would free.
    pub reclaimable: ByteSize,
}

impl KindUsage {
    /// The share of the space that nothing uses, from 0.0 to 1.0.
    #[must_use]
    pub fn unused_ratio(&self) -> f64 {
        if self.size.is_zero() {
            return 0.0;
        }
        (self.reclaimable.as_bytes() as f64 / self.size.as_bytes() as f64).min(1.0)
    }
}

/// What Docker holds, and what of it no container uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DockerInventory {
    /// Version of the Docker engine that answered.
    pub engine_version: String,
    /// One entry per kind Docker reported.
    pub usage: Vec<KindUsage>,
    /// Every resource nothing uses.
    pub unused: Vec<DockerResource>,
}

impl DockerInventory {
    /// Space everything Docker holds occupies inside it.
    #[must_use]
    pub fn size(&self) -> ByteSize {
        self.usage.iter().map(|usage| usage.size).sum()
    }

    /// The figures of one kind, when Docker reported them.
    #[must_use]
    pub fn usage_of(&self, kind: ResourceKind) -> Option<&KindUsage> {
        self.usage.iter().find(|usage| usage.kind == kind)
    }

    /// Puts the unused resources in the order they are shown: by kind, and
    /// the largest first within each kind.
    pub fn sort_unused(&mut self) {
        self.unused.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then(b.size.cmp(&a.size))
                .then_with(|| a.name.cmp(&b.name))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{DockerInventory, KindUsage};
    use crate::docker::{DockerResource, ResourceKind};
    use crate::shared::ByteSize;

    fn usage(kind: ResourceKind, size: u64, reclaimable: u64) -> KindUsage {
        KindUsage {
            kind,
            total_count: 4,
            active_count: 1,
            size: ByteSize::from_bytes(size),
            reclaimable: ByteSize::from_bytes(reclaimable),
        }
    }

    fn resource(kind: ResourceKind, name: &str, size: u64) -> DockerResource {
        DockerResource {
            kind,
            reference: name.into(),
            name: name.into(),
            size: ByteSize::from_bytes(size),
            created_unix_seconds: None,
        }
    }

    #[test]
    fn adds_up_the_kinds_and_finds_one_by_kind() {
        let inventory = DockerInventory {
            engine_version: "24.0.2".into(),
            usage: vec![
                usage(ResourceKind::Image, 800, 200),
                usage(ResourceKind::Volume, 200, 0),
            ],
            unused: Vec::new(),
        };
        assert_eq!(inventory.size(), ByteSize::from_bytes(1_000));
        assert_eq!(
            inventory
                .usage_of(ResourceKind::Image)
                .map(KindUsage::unused_ratio),
            Some(0.25)
        );
        assert!(inventory.usage_of(ResourceKind::Container).is_none());
    }

    #[test]
    fn the_unused_share_of_nothing_is_nothing_and_never_passes_the_whole() {
        assert!(usage(ResourceKind::Image, 0, 0).unused_ratio().abs() < f64::EPSILON);
        assert!((usage(ResourceKind::Image, 100, 250).unused_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn shows_the_build_cache_first_and_the_largest_of_each_kind_before_the_rest() {
        let mut inventory = DockerInventory {
            unused: vec![
                resource(ResourceKind::Volume, "data", 900),
                resource(ResourceKind::Image, "small", 100),
                resource(ResourceKind::BuildCache, "", 50),
                resource(ResourceKind::Image, "large", 700),
            ],
            ..DockerInventory::default()
        };
        inventory.sort_unused();
        let names: Vec<&str> = inventory
            .unused
            .iter()
            .map(|resource| resource.name.as_str())
            .collect();
        assert_eq!(names, ["", "large", "small", "data"]);
    }
}
