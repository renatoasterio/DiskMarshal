//! Use case: find out what Docker holds that nothing uses. Removes nothing.

use maccleaner_domain::docker::DockerInventory;

use crate::ports::{ContainerEngine, EngineError};

/// Builds the preview of a Docker cleanup.
pub struct InspectDocker<'a> {
    engine: &'a dyn ContainerEngine,
}

impl<'a> InspectDocker<'a> {
    /// Creates the use case over the Docker engine.
    #[must_use]
    pub fn new(engine: &'a dyn ContainerEngine) -> Self {
        Self { engine }
    }

    /// Asks Docker what it holds and lists what no container uses, in the
    /// order it is shown.
    ///
    /// # Errors
    ///
    /// Fails when Docker is not installed, is not running, or cannot answer.
    pub fn execute(&self) -> Result<DockerInventory, EngineError> {
        let mut inventory = self.engine.inventory()?;
        inventory.sort_unused();
        Ok(inventory)
    }
}

#[cfg(test)]
mod tests {
    use maccleaner_domain::docker::{DockerInventory, DockerResource, EngineStatus, ResourceKind};
    use maccleaner_domain::shared::ByteSize;

    use super::InspectDocker;
    use crate::ports::{ContainerEngine, EngineError};

    /// An engine that answers with a fixed result.
    struct FixedEngine(Result<DockerInventory, EngineError>);

    impl ContainerEngine for FixedEngine {
        fn status(&self) -> EngineStatus {
            EngineStatus::NotRunning
        }

        fn inventory(&self) -> Result<DockerInventory, EngineError> {
            self.0.clone()
        }

        fn disk_usage(&self) -> Result<ByteSize, EngineError> {
            Ok(ByteSize::ZERO)
        }

        fn remove(&self, _resource: &DockerResource) -> Result<(), EngineError> {
            Ok(())
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
    fn lists_what_is_unused_in_the_order_it_is_shown() {
        let engine = FixedEngine(Ok(DockerInventory {
            unused: vec![
                resource(ResourceKind::Volume, "data", 10),
                resource(ResourceKind::Image, "web:dev", 500),
            ],
            ..DockerInventory::default()
        }));

        let inventory = InspectDocker::new(&engine).execute().unwrap();

        let kinds: Vec<_> = inventory.unused.iter().map(|found| found.kind).collect();
        assert_eq!(kinds, [ResourceKind::Image, ResourceKind::Volume]);
    }

    #[test]
    fn says_why_docker_could_not_be_asked() {
        let engine = FixedEngine(Err(EngineError::NotRunning));
        assert_eq!(
            InspectDocker::new(&engine).execute().unwrap_err(),
            EngineError::NotRunning
        );
    }
}
