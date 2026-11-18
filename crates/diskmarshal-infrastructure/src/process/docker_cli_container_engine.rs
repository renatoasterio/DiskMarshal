//! Container engine that talks to Docker through its command line program.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use diskmarshal_application::ports::{ContainerEngine, EngineError};
use diskmarshal_domain::docker::{
    DockerInventory, DockerResource, EngineStatus, KindUsage, ResourceKind,
};
use diskmarshal_domain::shared::ByteSize;
use serde::{Deserialize, Deserializer};

use crate::process::bounded_command::{CommandFailure, Stdout, run_bounded};

/// Where the installers of Docker put its command line program. Each one is
/// an absolute path, so a different `docker` in `PATH` can never run instead.
const SYSTEM_LOCATIONS: [&str; 3] = [
    "/usr/local/bin/docker",
    "/opt/homebrew/bin/docker",
    "/Applications/Docker.app/Contents/Resources/bin/docker",
];
/// The same, for installers that put it inside the home folder: Docker
/// Desktop, OrbStack and Rancher Desktop.
const HOME_LOCATIONS: [&str; 3] = [
    ".docker/bin/docker",
    ".orbstack/bin/docker",
    ".rd/bin/docker",
];

const STATUS_TIMEOUT: Duration = Duration::from_secs(10);
/// Docker measures every image and volume to answer, which takes a while
/// when there are many.
const MEASURE_TIMEOUT: Duration = Duration::from_secs(120);
const REMOVE_TIMEOUT: Duration = Duration::from_secs(120);
const PRUNE_TIMEOUT: Duration = Duration::from_secs(600);

/// Makes Docker answer one JSON object per line instead of a table.
const JSON_FORMAT: &str = "{{json .}}";
const VERSION_FORMAT: &str = "{{.Server.Version}}";
/// What Docker prints in place of a name an image does not have.
const NO_NAME: &str = "<none>";
/// What Docker prints as the use count of a resource nothing uses.
const UNUSED: &str = "0";
/// States of a container that is not running and will not start by itself.
const STOPPED_STATES: [&str; 3] = ["exited", "created", "dead"];
/// Ends the options, so a name starting with `-` is never read as one.
const END_OF_OPTIONS: &str = "--";

/// Asks the `docker` program found on this Mac.
///
/// The program runs as the logged-in user, with no shell in between, and
/// receives nothing from the environment but the home folder, where Docker
/// keeps the address of its engine. No removal is ever forced: Docker
/// refuses a resource that is in use, and it stays.
#[derive(Debug, Clone)]
pub struct DockerCliContainerEngine {
    program: Option<PathBuf>,
    home: PathBuf,
}

impl DockerCliContainerEngine {
    /// Looks for Docker's program in the places its installers use, for the
    /// user whose home folder is `home`.
    #[must_use]
    pub fn for_home(home: &Path) -> Self {
        let program = SYSTEM_LOCATIONS
            .iter()
            .map(PathBuf::from)
            .chain(HOME_LOCATIONS.iter().map(|relative| home.join(relative)))
            .find(|candidate| candidate.is_file());
        Self::new(program, home.to_path_buf())
    }

    /// Uses the program at `program`, or reports Docker as not installed
    /// when there is none.
    #[must_use]
    pub fn new(program: Option<PathBuf>, home: PathBuf) -> Self {
        Self { program, home }
    }

    /// Runs Docker's program and returns what it printed.
    fn ask(&self, arguments: &[&str], timeout: Duration) -> Result<String, EngineError> {
        let program = self.program.as_ref().ok_or(EngineError::NotInstalled)?;
        let mut command = Command::new(program);
        command.args(arguments).env_clear().env("HOME", &self.home);
        let output =
            run_bounded(&mut command, Stdout::Capture, timeout).map_err(
                |failure| match failure {
                    CommandFailure::NotStarted(error) => EngineError::Failed(error.to_string()),
                    CommandFailure::TimedOut => EngineError::TimedOut,
                },
            )?;
        if output.code == Some(0) {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(EngineError::Failed(first_line(&output.stderr)))
        }
    }

    fn usage(&self) -> Result<Vec<KindUsage>, EngineError> {
        let answer = self.ask(&["system", "df", "--format", JSON_FORMAT], MEASURE_TIMEOUT)?;
        parse_usage(&answer).map_err(EngineError::Failed)
    }
}

impl ContainerEngine for DockerCliContainerEngine {
    fn status(&self) -> EngineStatus {
        if self.program.is_none() {
            return EngineStatus::NotInstalled;
        }
        match self.ask(&["version", "--format", VERSION_FORMAT], STATUS_TIMEOUT) {
            Ok(version) if !version.trim().is_empty() => EngineStatus::Running {
                version: version.trim().to_owned(),
            },
            Ok(_) | Err(_) => EngineStatus::NotRunning,
        }
    }

    fn inventory(&self) -> Result<DockerInventory, EngineError> {
        let engine_version = match self.status() {
            EngineStatus::NotInstalled => return Err(EngineError::NotInstalled),
            EngineStatus::NotRunning => return Err(EngineError::NotRunning),
            EngineStatus::Running { version } => version,
        };
        let usage = self.usage()?;
        let details = self.ask(
            &["system", "df", "--verbose", "--format", JSON_FORMAT],
            MEASURE_TIMEOUT,
        )?;
        let mut unused = parse_unused(&details).map_err(EngineError::Failed)?;
        unused.extend(unused_build_cache(&usage));
        Ok(DockerInventory {
            engine_version,
            usage,
            unused,
        })
    }

    fn disk_usage(&self) -> Result<ByteSize, EngineError> {
        Ok(self.usage()?.iter().map(|usage| usage.size).sum())
    }

    fn remove(&self, resource: &DockerResource) -> Result<(), EngineError> {
        let reference = resource.reference.as_str();
        let by_reference = |noun: &'static str| {
            if reference.is_empty() {
                return Err(EngineError::Failed("nothing to remove".to_owned()));
            }
            self.ask(&[noun, "rm", END_OF_OPTIONS, reference], REMOVE_TIMEOUT)
        };
        match resource.kind {
            ResourceKind::BuildCache => {
                self.ask(&["builder", "prune", "--all", "--force"], PRUNE_TIMEOUT)
            }
            ResourceKind::Image => by_reference("image"),
            ResourceKind::Container => by_reference("container"),
            ResourceKind::Volume => by_reference("volume"),
        }
        .map(drop)
    }
}

/// One line of `docker system df`. Docker prints every figure as text.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UsageLine {
    #[serde(rename = "Type")]
    kind: String,
    #[serde(default, deserialize_with = "text")]
    total_count: String,
    #[serde(default, deserialize_with = "text")]
    active: String,
    #[serde(default, deserialize_with = "text")]
    size: String,
    #[serde(default, deserialize_with = "text")]
    reclaimable: String,
}

/// The answer of `docker system df --verbose`. An empty list comes as `null`.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Details {
    #[serde(default)]
    images: Option<Vec<ImageLine>>,
    #[serde(default)]
    containers: Option<Vec<ContainerLine>>,
    #[serde(default)]
    volumes: Option<Vec<VolumeLine>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ImageLine {
    #[serde(rename = "ID", default, deserialize_with = "text")]
    id: String,
    #[serde(default, deserialize_with = "text")]
    repository: String,
    #[serde(default, deserialize_with = "text")]
    tag: String,
    /// How many containers use the image.
    #[serde(default, deserialize_with = "text")]
    containers: String,
    #[serde(default, deserialize_with = "text")]
    size: String,
    #[serde(default, deserialize_with = "text")]
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerLine {
    #[serde(rename = "ID", default, deserialize_with = "text")]
    id: String,
    #[serde(default, deserialize_with = "text")]
    names: String,
    #[serde(default, deserialize_with = "text")]
    state: String,
    #[serde(default, deserialize_with = "text")]
    size: String,
    #[serde(default, deserialize_with = "text")]
    created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VolumeLine {
    #[serde(default, deserialize_with = "text")]
    name: String,
    /// How many containers use the volume.
    #[serde(default, deserialize_with = "text")]
    links: String,
    #[serde(default, deserialize_with = "text")]
    size: String,
}

/// Reads a figure whether Docker printed it as text or as a bare value,
/// which differs between its versions.
fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Printed {
        Text(String),
        Number(serde_json::Number),
        Flag(bool),
        Nothing,
    }
    Ok(match Printed::deserialize(deserializer)? {
        Printed::Text(text) => text,
        Printed::Number(number) => number.to_string(),
        Printed::Flag(flag) => flag.to_string(),
        Printed::Nothing => String::new(),
    })
}

/// Reads `docker system df`: one JSON object per line, one line per kind.
fn parse_usage(answer: &str) -> Result<Vec<KindUsage>, String> {
    let mut usage = Vec::new();
    for line in answer.lines().filter(|line| !line.trim().is_empty()) {
        let line: UsageLine = serde_json::from_str(line).map_err(|error| error.to_string())?;
        let kind = match line.kind.as_str() {
            "Images" => ResourceKind::Image,
            "Containers" => ResourceKind::Container,
            "Local Volumes" => ResourceKind::Volume,
            "Build Cache" => ResourceKind::BuildCache,
            // A kind this app does not know is not offered for removal.
            _ => continue,
        };
        usage.push(KindUsage {
            kind,
            total_count: line.total_count.trim().parse().unwrap_or(0),
            active_count: line.active.trim().parse().unwrap_or(0),
            size: parse_size(&line.size).unwrap_or(ByteSize::ZERO),
            reclaimable: parse_size(&line.reclaimable).unwrap_or(ByteSize::ZERO),
        });
    }
    Ok(usage)
}

/// Reads `docker system df --verbose` and keeps what no container uses.
fn parse_unused(answer: &str) -> Result<Vec<DockerResource>, String> {
    let details: Details = serde_json::from_str(answer).map_err(|error| error.to_string())?;
    let images = details
        .images
        .unwrap_or_default()
        .into_iter()
        .filter(|image| image.containers.trim() == UNUSED && !image.id.is_empty())
        .map(|image| {
            let named = |part: &str| !part.is_empty() && part != NO_NAME;
            let (name, reference) = match (named(&image.repository), named(&image.tag)) {
                (true, true) => {
                    let name = format!("{}:{}", image.repository, image.tag);
                    (name.clone(), name)
                }
                // Downloaded by digest: it has a name but no tag to remove it by.
                (true, false) => (image.repository, image.id),
                (false, _) => (String::new(), image.id),
            };
            DockerResource {
                kind: ResourceKind::Image,
                reference,
                name,
                size: parse_size(&image.size).unwrap_or(ByteSize::ZERO),
                created_unix_seconds: parse_created(&image.created_at),
            }
        });
    let containers = details
        .containers
        .unwrap_or_default()
        .into_iter()
        .filter(|container| {
            STOPPED_STATES.contains(&container.state.trim()) && !container.id.is_empty()
        })
        .map(|container| DockerResource {
            kind: ResourceKind::Container,
            reference: container.id,
            name: container.names,
            size: parse_size(&container.size).unwrap_or(ByteSize::ZERO),
            created_unix_seconds: parse_created(&container.created_at),
        });
    let volumes = details
        .volumes
        .unwrap_or_default()
        .into_iter()
        .filter(|volume| volume.links.trim() == UNUSED && !volume.name.is_empty())
        .map(|volume| DockerResource {
            kind: ResourceKind::Volume,
            reference: volume.name.clone(),
            name: volume.name,
            size: parse_size(&volume.size).unwrap_or(ByteSize::ZERO),
            created_unix_seconds: None,
        });
    Ok(images.chain(containers).chain(volumes).collect())
}

/// The build cache as one resource, when there is any of it to remove.
fn unused_build_cache(usage: &[KindUsage]) -> Option<DockerResource> {
    let cache = usage
        .iter()
        .find(|usage| usage.kind == ResourceKind::BuildCache)?;
    (!cache.reclaimable.is_zero()).then(|| DockerResource {
        kind: ResourceKind::BuildCache,
        reference: String::new(),
        name: String::new(),
        size: cache.reclaimable,
        created_unix_seconds: None,
    })
}

/// Reads a size the way Docker prints it, such as `4.7GB` or `1.2GB (44%)`.
fn parse_size(printed: &str) -> Option<ByteSize> {
    let figure = printed.split_whitespace().next()?;
    let unit_at = figure.find(|letter: char| !(letter.is_ascii_digit() || letter == '.'))?;
    let (number, unit) = figure.split_at(unit_at);
    let scale = match unit.to_ascii_lowercase().as_str() {
        "b" => 1.0,
        "kb" => 1e3,
        "mb" => 1e6,
        "gb" => 1e9,
        "tb" => 1e12,
        "kib" => 1_024.0,
        "mib" => 1_048_576.0,
        "gib" => 1_073_741_824.0,
        _ => return None,
    };
    let bytes = number.parse::<f64>().ok()? * scale;
    (bytes.is_finite() && bytes >= 0.0).then(|| ByteSize::from_bytes(bytes.round() as u64))
}

/// Reads a moment the way Docker prints it, such as
/// `2024-02-29 12:00:00 -0300 -03`, as seconds since the Unix epoch.
fn parse_created(printed: &str) -> Option<i64> {
    const DAY: i64 = 86_400;
    let mut parts = printed.split_whitespace();
    let (date, time, offset) = (parts.next()?, parts.next()?, parts.next()?);

    let mut date = date.splitn(3, '-').map(str::parse::<i64>);
    let (year, month, day) = (date.next()?.ok()?, date.next()?.ok()?, date.next()?.ok()?);
    let mut time = time.splitn(3, ':');
    let hour: i64 = time.next()?.parse().ok()?;
    let minute: i64 = time.next()?.parse().ok()?;
    // Docker sometimes adds a fraction of a second, which nobody needs here.
    let second: i64 = time.next()?.split('.').next()?.parse().ok()?;
    let in_range = (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && (0..24).contains(&hour)
        && (0..60).contains(&minute)
        && (0..=60).contains(&second);
    if !in_range {
        return None;
    }

    let (sign, digits) = offset.split_at_checked(1)?;
    let sign = match sign {
        "+" => 1,
        "-" => -1,
        _ => return None,
    };
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let (hours, minutes) = digits.split_at(2);
    let ahead_of_utc =
        sign * (hours.parse::<i64>().ok()? * 3_600 + minutes.parse::<i64>().ok()? * 60);

    Some(
        days_since_epoch(year, month, day) * DAY + hour * 3_600 + minute * 60 + second
            - ahead_of_utc,
    )
}

/// Days from 1970-01-01 to a date of the Gregorian calendar, with the
/// formula that counts in years starting in March.
fn days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let (era, year_of_era) = (year.div_euclid(400), year.rem_euclid(400));
    let month_from_march = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The first line of what a program printed as its error, which is where
/// Docker says what went wrong.
fn first_line(printed: &[u8]) -> String {
    String::from_utf8_lossy(printed)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("no message")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use diskmarshal_application::ports::{ContainerEngine, EngineError};
    use diskmarshal_domain::docker::{EngineStatus, ResourceKind};
    use diskmarshal_domain::shared::ByteSize;

    use super::{
        DockerCliContainerEngine, parse_created, parse_size, parse_unused, parse_usage,
        unused_build_cache,
    };

    const USAGE: &str = r#"{"Active":"2","Reclaimable":"1.5GB (60%)","Size":"2.5GB","TotalCount":"5","Type":"Images"}
{"Active":"1","Reclaimable":"12MB (50%)","Size":"24MB","TotalCount":"3","Type":"Containers"}
{"Active":"1","Reclaimable":"480MB (80%)","Size":"600MB","TotalCount":"4","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"8.4GB","Size":"8.4GB","TotalCount":"212","Type":"Build Cache"}
"#;

    const DETAILS: &str = r#"{"Images":[
{"Containers":"1","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:aaa","Repository":"postgres","Tag":"16","Size":"400MB"},
{"Containers":"0","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:bbb","Repository":"web","Tag":"dev","Size":"1.2GB"},
{"Containers":"0","CreatedAt":"2024-02-29 12:00:00 -0300 -03","ID":"sha256:ccc","Repository":"<none>","Tag":"<none>","Size":"900MB"},
{"Containers":"0","CreatedAt":"","ID":"sha256:ddd","Repository":"registry.test/tool","Tag":"<none>","Size":"50MB"},
{"Containers":"N/A","ID":"sha256:eee","Repository":"unknown","Tag":"1","Size":"10MB"}],
"Containers":[
{"ID":"c1","Names":"db-1","State":"running","Size":"12MB","CreatedAt":"2024-02-29 12:00:00 -0300 -03"},
{"ID":"c2","Names":"job-7","State":"exited","Size":"4MB (virtual 1.2GB)","CreatedAt":"2024-02-29 12:00:00 -0300 -03"},
{"ID":"c3","Names":"paused-1","State":"paused","Size":"1MB","CreatedAt":""}],
"Volumes":[
{"Name":"pgdata","Links":"1","Size":"120MB"},
{"Name":"old_data","Links":"0","Size":"480MB"},
{"Name":"remote","Links":"0","Size":"N/A"}],
"BuildCache":[{"ID":"x","Size":"175kB","InUse":"false"}]}"#;

    #[test]
    fn reads_sizes_the_way_docker_prints_them() {
        let bytes = |printed| parse_size(printed).map(ByteSize::as_bytes);
        assert_eq!(bytes("0B"), Some(0));
        assert_eq!(bytes("175kB"), Some(175_000));
        assert_eq!(bytes("46.69MB"), Some(46_690_000));
        assert_eq!(bytes("4.703GB (44%)"), Some(4_703_000_000));
        assert_eq!(bytes("1.5TB"), Some(1_500_000_000_000));
        assert_eq!(bytes("2GiB"), Some(2_147_483_648));
        assert_eq!(bytes("N/A"), None);
        assert_eq!(bytes(""), None);
        assert_eq!(bytes("12"), None, "a number without a unit says nothing");
        assert_eq!(bytes("1.2.3GB"), None);
    }

    #[test]
    fn reads_moments_in_any_time_zone() {
        assert_eq!(parse_created("1970-01-01 00:00:00 +0000 UTC"), Some(0));
        assert_eq!(
            parse_created("2000-03-01 00:00:00 +0000 UTC"),
            Some(951_868_800)
        );
        assert_eq!(
            parse_created("2024-02-29 12:00:00 -0300 -03"),
            Some(1_709_218_800)
        );
        assert_eq!(
            parse_created("2024-02-29 20:30:00.800008045 +0530 IST"),
            Some(1_709_218_800)
        );
        for broken in [
            "",
            "yesterday",
            "2024-02-29",
            "2024-13-01 00:00:00 +0000",
            "2024-02-29 25:00:00 +0000",
            "2024-02-29 12:00:00 0300",
            "2024-02-29 12:00:00 +03",
        ] {
            assert_eq!(parse_created(broken), None, "{broken}");
        }
    }

    #[test]
    fn reads_how_much_docker_holds_of_each_kind() {
        let usage = parse_usage(USAGE).unwrap();

        let kinds: Vec<_> = usage.iter().map(|usage| usage.kind).collect();
        assert_eq!(
            kinds,
            [
                ResourceKind::Image,
                ResourceKind::Container,
                ResourceKind::Volume,
                ResourceKind::BuildCache
            ]
        );
        let images = &usage[0];
        assert_eq!((images.total_count, images.active_count), (5, 2));
        assert_eq!(images.size, ByteSize::from_bytes(2_500_000_000));
        assert_eq!(images.reclaimable, ByteSize::from_bytes(1_500_000_000));

        let cache = unused_build_cache(&usage).unwrap();
        assert_eq!(cache.kind, ResourceKind::BuildCache);
        assert_eq!(cache.size, ByteSize::from_bytes(8_400_000_000));
    }

    #[test]
    fn a_kind_docker_adds_later_is_ignored_and_figures_may_come_as_numbers() {
        let answer = concat!(
            r#"{"Type":"Images","TotalCount":5,"Active":2,"Size":"1GB","Reclaimable":"0B"}"#,
            "\n",
            r#"{"Type":"Something New","TotalCount":"1","Active":"0","Size":"9GB","Reclaimable":"9GB"}"#,
            "\n"
        );
        let usage = parse_usage(answer).unwrap();
        assert_eq!(usage.len(), 1);
        assert_eq!((usage[0].total_count, usage[0].active_count), (5, 2));
        assert!(unused_build_cache(&usage).is_none());
        assert!(parse_usage("not json").is_err());
    }

    #[test]
    fn keeps_only_what_no_container_uses() {
        let unused = parse_unused(DETAILS).unwrap();

        let seen: Vec<(ResourceKind, &str, &str, u64)> = unused
            .iter()
            .map(|found| {
                (
                    found.kind,
                    found.name.as_str(),
                    found.reference.as_str(),
                    found.size.as_bytes(),
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                (ResourceKind::Image, "web:dev", "web:dev", 1_200_000_000),
                (ResourceKind::Image, "", "sha256:ccc", 900_000_000),
                (
                    ResourceKind::Image,
                    "registry.test/tool",
                    "sha256:ddd",
                    50_000_000
                ),
                (ResourceKind::Container, "job-7", "c2", 4_000_000),
                (ResourceKind::Volume, "old_data", "old_data", 480_000_000),
                (ResourceKind::Volume, "remote", "remote", 0),
            ]
        );
        assert_eq!(unused[0].created_unix_seconds, Some(1_709_218_800));
        assert_eq!(unused[2].created_unix_seconds, None);
    }

    #[test]
    fn empty_lists_come_as_null_or_not_at_all() {
        assert!(
            parse_unused(r#"{"Images":null,"Containers":[],"BuildCache":null}"#)
                .unwrap()
                .is_empty()
        );
        assert!(parse_unused("{}").unwrap().is_empty());
        assert!(parse_unused("not json").is_err());
    }

    #[test]
    fn without_a_program_docker_is_not_installed() {
        let engine = DockerCliContainerEngine::new(None, PathBuf::from("/Users/nobody"));
        assert_eq!(engine.status(), EngineStatus::NotInstalled);
        assert_eq!(engine.inventory().unwrap_err(), EngineError::NotInstalled);
        assert_eq!(engine.disk_usage().unwrap_err(), EngineError::NotInstalled);
    }
}
