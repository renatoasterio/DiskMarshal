//! Cleaning Docker through a stand-in for its command line program: a script
//! that answers like Docker and writes down what it was asked to do. No real
//! Docker is asked anything, so no image or volume of this Mac is touched.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use diskmarshal_application::ports::{ContainerEngine, EngineError};
use diskmarshal_application::use_cases::{InspectDocker, PruneDocker};
use diskmarshal_domain::cleanup::RiskLevel;
use diskmarshal_domain::deletion::{DeletionStatus, SkipReason};
use diskmarshal_domain::docker::{EngineStatus, ResourceKind};
use diskmarshal_domain::shared::ByteSize;
use diskmarshal_infrastructure::persistence::JsonlHistoryRepository;
use diskmarshal_infrastructure::process::DockerCliContainerEngine;
use diskmarshal_test_support::TempTree;

/// Answers like `docker`. Every removal is appended to `calls.log`; the
/// build cache disappears from the totals once `pruned` exists; the image
/// `busy:1` is refused the way Docker refuses an image in use.
const STAND_IN: &str = r#"#!/bin/sh
here="$(/usr/bin/dirname "$0")"
case "$1 $2" in
  "version --format")
    [ -e "$here/stopped" ] && { echo "Cannot connect to the Docker daemon" >&2; exit 1; }
    echo "24.0.2"
    echo "$HOME" > "$here/home.log" ;;
  "system df")
    if [ "$3" = "--verbose" ]; then /bin/cat "$here/details.json"
    elif [ -e "$here/pruned" ]; then /bin/cat "$here/usage-after.jsonl"
    else /bin/cat "$here/usage.jsonl"; fi ;;
  "builder prune")
    echo "$*" >> "$here/calls.log"
    : > "$here/pruned" ;;
  "image rm"|"container rm"|"volume rm")
    echo "$*" >> "$here/calls.log"
    [ "$4" = "busy:1" ] && { echo "Error response from daemon: conflict: unable to remove repository reference" >&2; exit 1; }
    exit 0 ;;
  *)
    echo "unexpected: $*" >&2; exit 64 ;;
esac
"#;

const USAGE: &str = r#"{"Active":"1","Reclaimable":"2.1GB (84%)","Size":"2.5GB","TotalCount":"4","Type":"Images"}
{"Active":"1","Reclaimable":"4MB (25%)","Size":"16MB","TotalCount":"2","Type":"Containers"}
{"Active":"1","Reclaimable":"480MB (80%)","Size":"600MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"8.4GB","Size":"8.4GB","TotalCount":"212","Type":"Build Cache"}
"#;

const USAGE_AFTER: &str = r#"{"Active":"1","Reclaimable":"1.2GB (75%)","Size":"1.6GB","TotalCount":"3","Type":"Images"}
{"Active":"1","Reclaimable":"4MB (25%)","Size":"16MB","TotalCount":"2","Type":"Containers"}
{"Active":"1","Reclaimable":"480MB (80%)","Size":"600MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"0B","Size":"0B","TotalCount":"0","Type":"Build Cache"}
"#;

const DETAILS: &str = r#"{"Images":[
{"Containers":"1","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:aaa","Repository":"postgres","Tag":"16","Size":"400MB"},
{"Containers":"0","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:bbb","Repository":"web","Tag":"dev","Size":"1.2GB"},
{"Containers":"0","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:ccc","Repository":"<none>","Tag":"<none>","Size":"900MB"},
{"Containers":"0","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:ddd","Repository":"busy","Tag":"1","Size":"30MB"}],
"Containers":[
{"ID":"c1","Names":"db-1","State":"running","Size":"12MB","CreatedAt":"2024-02-29 12:00:00 -0300 -03"},
{"ID":"c2","Names":"job-7","State":"exited","Size":"4MB","CreatedAt":"2024-02-29 12:00:00 -0300 -03"}],
"Volumes":[
{"Name":"pgdata","Links":"1","Size":"120MB"},
{"Name":"-old data","Links":"0","Size":"480MB"}],
"BuildCache":[]}"#;

/// A folder holding the stand-in and the files it answers from.
struct FakeDocker {
    fixture: TempTree,
}

impl FakeDocker {
    fn new() -> io::Result<Self> {
        let fixture = TempTree::new()?;
        for (name, content) in [
            ("usage.jsonl", USAGE),
            ("usage-after.jsonl", USAGE_AFTER),
            ("details.json", DETAILS),
            ("docker", STAND_IN),
        ] {
            fs::write(fixture.file(name, 0)?, content)?;
        }
        fs::set_permissions(fixture.path("docker"), fs::Permissions::from_mode(0o755))?;
        Ok(Self { fixture })
    }

    fn engine(&self) -> DockerCliContainerEngine {
        DockerCliContainerEngine::new(Some(self.fixture.path("docker")), self.home())
    }

    fn home(&self) -> PathBuf {
        self.fixture.path("home")
    }

    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.fixture.path("calls.log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn lists_what_nothing_uses_and_how_much_docker_holds() {
    let docker = FakeDocker::new().unwrap();
    let engine = docker.engine();

    assert_eq!(
        engine.status(),
        EngineStatus::Running {
            version: "24.0.2".into()
        }
    );
    let inventory = InspectDocker::new(&engine).execute().unwrap();

    assert_eq!(inventory.engine_version, "24.0.2");
    assert_eq!(
        inventory.size(),
        ByteSize::from_bytes(2_500_000_000 + 16_000_000 + 600_000_000 + 8_400_000_000)
    );
    let seen: Vec<(ResourceKind, &str, RiskLevel)> = inventory
        .unused
        .iter()
        .map(|found| (found.kind, found.name.as_str(), found.risk()))
        .collect();
    assert_eq!(
        seen,
        [
            (ResourceKind::BuildCache, "", RiskLevel::Safe),
            (ResourceKind::Image, "web:dev", RiskLevel::Review),
            (ResourceKind::Image, "", RiskLevel::Safe),
            (ResourceKind::Image, "busy:1", RiskLevel::Review),
            (ResourceKind::Container, "job-7", RiskLevel::Review),
            (ResourceKind::Volume, "-old data", RiskLevel::Review),
        ],
        "what a container uses is not listed"
    );
    assert!(docker.calls().is_empty(), "inspecting removes nothing");
    assert_eq!(
        fs::read_to_string(docker.fixture.path("home.log"))
            .unwrap()
            .trim(),
        docker.home().to_str().unwrap(),
        "docker finds its engine through the home folder"
    );
}

#[test]
fn removes_what_was_approved_and_leaves_what_docker_refuses() {
    let docker = FakeDocker::new().unwrap();
    let engine = docker.engine();
    let history_file = docker.fixture.path("history.jsonl");
    let history = JsonlHistoryRepository::new(history_file.clone());
    let inventory = InspectDocker::new(&engine).execute().unwrap();

    let report = PruneDocker::new(&engine, &history).execute(&inventory.unused);

    assert_eq!(
        docker.calls(),
        [
            "builder prune --all --force",
            "image rm -- web:dev",
            "image rm -- sha256:ccc",
            "image rm -- busy:1",
            "container rm -- c2",
            "volume rm -- -old data",
        ],
        "nothing is forced, and a name starting with a dash stays a name"
    );
    let refused = &report.outcomes[3];
    assert_eq!(refused.target, PathBuf::from("docker/image/busy:1"));
    assert!(
        matches!(&refused.status, DeletionStatus::Skipped(SkipReason::Failed(reason)) if reason.contains("conflict")),
        "{:?}",
        refused.status
    );
    assert_eq!(
        report
            .outcomes
            .iter()
            .filter(|outcome| outcome.is_removed())
            .count(),
        5
    );
    assert_eq!(
        report.reclaimed,
        ByteSize::from_bytes(900_000_000 + 8_400_000_000),
        "what docker's own totals dropped by, not the sizes added up"
    );
    let recorded = fs::read_to_string(history_file).unwrap();
    assert_eq!(recorded.lines().count(), 5);
    assert!(recorded.contains("docker/build-cache") && !recorded.contains("busy"));
}

#[test]
fn a_docker_that_is_closed_is_told_apart_from_one_that_is_missing() {
    let docker = FakeDocker::new().unwrap();
    docker.fixture.file("stopped", 0).unwrap();
    let engine = docker.engine();

    assert_eq!(engine.status(), EngineStatus::NotRunning);
    assert_eq!(
        InspectDocker::new(&engine).execute().unwrap_err(),
        EngineError::NotRunning
    );

    let missing = DockerCliContainerEngine::new(None, docker.home());
    assert_eq!(missing.status(), EngineStatus::NotInstalled);
    assert_eq!(
        InspectDocker::new(&missing).execute().unwrap_err(),
        EngineError::NotInstalled
    );
    assert!(docker.calls().is_empty());
}
