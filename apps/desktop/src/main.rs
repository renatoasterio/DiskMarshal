//! DiskMarshal desktop application. Composition root for the graphical
//! interface: wires the macOS adapters into the interface and opens the window.

#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context as _, anyhow};
use diskmarshal_application::ports::{
    ConnectionInspector, ContainerEngine, MetricsProvider, NetworkTuner, ReachabilityProbe,
    SpeedTester, TrafficProbe,
};
use diskmarshal_infrastructure::bundles::{PlistAppBundleReader, application_folders};
use diskmarshal_infrastructure::catalog::TomlRuleCatalog;
use diskmarshal_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, SafeFileRemover, StatfsVolumeStats,
    disable_dataless_materialization, icloud_sync_roots,
};
use diskmarshal_infrastructure::hashing::Blake3ContentHasher;
use diskmarshal_infrastructure::imaging::ImageIoThumbnailer;
use diskmarshal_infrastructure::metrics::{SysinfoMetricsProvider, UnavailableMetrics};
use diskmarshal_infrastructure::network::{
    MacosConnectionInspector, NettopTrafficProbe, NetworkQualitySpeedTester, PingReachabilityProbe,
};
use diskmarshal_infrastructure::permissions::FullDiskAccessProbe;
use diskmarshal_infrastructure::persistence::JsonlHistoryRepository;
use diskmarshal_infrastructure::privilege::{MacosNetworkTuner, MacosPrivilegedCleaner};
use diskmarshal_infrastructure::process::{DockerCliContainerEngine, GitVersionControlProbe};
use diskmarshal_infrastructure::system::{
    LocaltimeWallClock, NativeFolderPicker, OpenCommandOpener, SysinfoRunningAppProbe,
};
use diskmarshal_infrastructure::trash::NsFileManagerTrashBin;
use diskmarshal_ui::{DiskMarshalUi, UiOptions, UiServices, WINDOW_SIZE};
use eframe::egui;

const APP_NAME: &str = "DiskMarshal";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("erro: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    // Before any file is touched: without this, reading a cloud-only item
    // would make macOS download it.
    disable_dataless_materialization()
        .context("não foi possível impedir downloads do iCloud; o app não foi aberto")?;

    let viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_inner_size(WINDOW_SIZE)
        .with_min_inner_size(WINDOW_SIZE);
    let options = eframe::NativeOptions {
        viewport,
        ..capture::native_options()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|creation| Ok(Box::new(DesktopApp::new(&creation.egui_ctx)))),
    )
    .map_err(|error| anyhow!("não foi possível abrir a janela: {error}"))
}

/// The eframe application: owns the interface and forwards every frame to it.
struct DesktopApp {
    interface: DiskMarshalUi,
}

impl DesktopApp {
    fn new(ctx: &egui::Context) -> Self {
        let home = env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from);
        let temporary = env::var_os("TMPDIR").map(PathBuf::from);
        let network = capture::network().unwrap_or_else(system_network);
        let services = UiServices {
            volume_stats: Arc::new(StatfsVolumeStats),
            wall_clock: Arc::new(LocaltimeWallClock),
            walker: Arc::new(ReaddirWalker::new(icloud_sync_roots())),
            opener: Arc::new(OpenCommandOpener),
            inspector: Arc::new(LstatFileInspector),
            remover: Arc::new(SafeFileRemover),
            trash_bin: Arc::new(NsFileManagerTrashBin::new()),
            history: Arc::new(JsonlHistoryRepository::for_home(&home)),
            rules: Arc::new(TomlRuleCatalog::new(home.clone(), temporary)),
            version_control: Arc::new(GitVersionControlProbe),
            folder_picker: Arc::new(NativeFolderPicker),
            hasher: Arc::new(Blake3ContentHasher),
            thumbnailer: Arc::new(ImageIoThumbnailer),
            privileged: Arc::new(MacosPrivilegedCleaner::beside_current_executable()),
            permissions: Arc::new(FullDiskAccessProbe::for_home(&home)),
            bundles: Arc::new(PlistAppBundleReader::for_home(&home)),
            running_apps: Arc::new(SysinfoRunningAppProbe),
            folder_lister: Arc::new(ReaddirFolderLister),
            container_engine: capture::container_engine().unwrap_or_else(|| docker(&home)),
            application_folders: capture::application_folders()
                .unwrap_or_else(|| application_folders(&home)),
            metrics: capture::metrics().unwrap_or_else(system_metrics),
            traffic_probe: network.traffic_probe,
            connection_inspector: network.connection_inspector,
            reachability: network.reachability,
            speed_tester: network.speed_tester,
            network_tuner: network.network_tuner,
            home,
        };
        Self {
            interface: DiskMarshalUi::new(ctx, services, &capture::ui_options()),
        }
    }
}

/// The readings of this machine, or a stand-in when it cannot be queried.
fn system_metrics() -> Arc<dyn MetricsProvider> {
    SysinfoMetricsProvider::new().map_or_else(
        || Arc::new(UnavailableMetrics) as Arc<dyn MetricsProvider>,
        |provider| Arc::new(provider),
    )
}

/// The Docker installed on this Mac, when there is one.
fn docker(home: &Path) -> Arc<dyn ContainerEngine> {
    Arc::new(DockerCliContainerEngine::for_home(home))
}

/// What the network screen needs from the outside world.
struct NetworkAdapters {
    traffic_probe: Arc<dyn TrafficProbe>,
    connection_inspector: Arc<dyn ConnectionInspector>,
    reachability: Arc<dyn ReachabilityProbe>,
    speed_tester: Arc<dyn SpeedTester>,
    network_tuner: Arc<dyn NetworkTuner>,
}

/// The network of this machine, read and changed through the programs
/// macOS ships.
fn system_network() -> NetworkAdapters {
    NetworkAdapters {
        traffic_probe: Arc::new(NettopTrafficProbe::new()),
        connection_inspector: Arc::new(MacosConnectionInspector),
        reachability: Arc::new(PingReachabilityProbe),
        speed_tester: Arc::new(NetworkQualitySpeedTester),
        network_tuner: Arc::new(MacosNetworkTuner::beside_current_executable()),
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.interface.show(ui);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.interface.clear_color()
    }
}

/// Normal start-up: default renderer, start-up sequence, and as much motion
/// as the user asked macOS for.
#[cfg(not(feature = "capture"))]
mod capture {
    use std::path::PathBuf;
    use std::sync::Arc;

    use diskmarshal_application::ports::{ContainerEngine, MetricsProvider};
    use diskmarshal_infrastructure::system::prefers_reduced_motion;
    use diskmarshal_ui::MotionPreference;

    use super::{NetworkAdapters, UiOptions};

    pub(super) fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions::default()
    }

    /// The normal app always shows the readings of this machine.
    pub(super) fn metrics() -> Option<Arc<dyn MetricsProvider>> {
        None
    }

    /// The normal app always asks the Docker of this machine.
    pub(super) fn container_engine() -> Option<Arc<dyn ContainerEngine>> {
        None
    }

    /// The normal app always lists the apps of this machine.
    pub(super) fn application_folders() -> Option<Vec<PathBuf>> {
        None
    }

    /// The normal app always shows the network of this machine.
    pub(super) fn network() -> Option<NetworkAdapters> {
        None
    }

    pub(super) fn ui_options() -> UiOptions {
        // "Reduce motion" in System Settings is where the app starts from;
        // the appearance panel still lets the user choose otherwise.
        let motion = if prefers_reduced_motion() {
            MotionPreference::Reduced
        } else {
            MotionPreference::Full
        };
        UiOptions {
            motion,
            ..UiOptions::default()
        }
    }
}

/// Development build used to check the interface against the prototype.
///
/// With `EFRAME_SCREENSHOT_TO=<file>.png` set, eframe saves the second frame
/// and exits. That hook exists only in the OpenGL renderer, and the frame has
/// to be still, so this build skips the start-up sequence and all motion.
/// `DISKMARSHAL_SCREEN=<slug>` and `DISKMARSHAL_PALETTE=<slug>` choose what to capture,
/// `DISKMARSHAL_SCAN=<folder>` measures a folder first so the disk map is filled in, and
/// `DISKMARSHAL_ANALYZE=1` analyzes the cleanup rules first. `DISKMARSHAL_DEV`,
/// `DISKMARSHAL_DUP` and `DISKMARSHAL_PHOTOS` each name a folder to search
/// first for build folders, duplicates and similar photos, and
/// `DISKMARSHAL_DEMO=1` replaces the readings of the monitor with made-up
/// ones, and what Docker holds too. `DISKMARSHAL_APPS=<folder>` lists the apps
/// of that folder instead of the ones installed, and `DISKMARSHAL_DOCKER=1`
/// asks Docker first. `DISKMARSHAL_NETWORK=<tab>` fills in the network screen
/// first and opens it on that tab (`traffic`, `analysis` or `tuning`); with
/// `DISKMARSHAL_DEMO` the network it shows is made up too, speed test
/// included. Run with `HOME` pointing at a demonstration folder to keep
/// every personal path out of the picture.
#[cfg(feature = "capture")]
mod capture {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use diskmarshal_application::ports::{ContainerEngine, EngineError, MetricsProvider};
    use diskmarshal_domain::docker::{
        DockerInventory, DockerResource, EngineStatus, KindUsage, ResourceKind,
    };
    use diskmarshal_domain::monitoring::{ProcessUsage, SensorReading, SystemSnapshot};
    use diskmarshal_domain::shared::ByteSize;
    use diskmarshal_ui::{MotionPreference, Palette, Screen};

    use super::{NetworkAdapters, UiOptions};

    /// With `DISKMARSHAL_DEMO` set, the network screen shows a made-up network.
    pub(super) fn network() -> Option<NetworkAdapters> {
        std::env::var_os("DISKMARSHAL_DEMO").map(|_| demo_network::adapters())
    }

    const GIGABYTE: u64 = 1_000_000_000;
    const DEMO_CORES: u32 = 10;

    /// Made-up readings for documentation screenshots, so they show no
    /// program that happens to be running on the machine that takes them.
    #[derive(Default)]
    struct DemoMetrics {
        readings: AtomicU32,
    }

    impl MetricsProvider for DemoMetrics {
        fn sample(&self) -> SystemSnapshot {
            let step = self.readings.fetch_add(1, Ordering::Relaxed) as f32;
            let wave = |speed: f32, offset: f32| (step * speed + offset).sin() * 0.5 + 0.5;
            let process = |name: &str, cpu_cores: f32, megabytes: u64| ProcessUsage {
                name: name.to_owned(),
                cpu_cores,
                memory: ByteSize::from_bytes(megabytes * 1_000_000),
            };
            let sensor = |label: &str, celsius: f32| SensorReading {
                label: label.to_owned(),
                celsius,
            };
            SystemSnapshot {
                cpu_usage: 0.14 + 0.16 * wave(0.45, 0.0) + 0.06 * wave(1.7, 1.0),
                core_usage: (0..DEMO_CORES)
                    .map(|core| {
                        let busy = 0.75 / (core as f32 + 1.0);
                        (busy * (0.5 + 0.5 * wave(0.6, core as f32))).clamp(0.02, 1.0)
                    })
                    .collect(),
                memory_used: ByteSize::from_bytes(19 * GIGABYTE + 600_000_000),
                memory_total: ByteSize::from_bytes(32 * GIGABYTE),
                swap_used: ByteSize::from_bytes(GIGABYTE + 200_000_000),
                sensors: vec![
                    sensor("CPU", 52.0),
                    sensor("GPU", 47.0),
                    sensor("SSD", 38.0),
                    sensor("Bateria", 33.0),
                    sensor("Placa", 41.0),
                ],
                processes: vec![
                    process("WindowServer", 0.21, 512),
                    process("Xcode", 0.18, 2_400),
                    process("Safari", 0.12, 1_100),
                    process("node", 0.09, 640),
                    process("mds_stores", 0.06, 180),
                    process("kernel_task", 0.05, 96),
                ],
            }
        }
    }

    /// A made-up Docker for documentation screenshots, so they show no image
    /// or volume of the machine that takes them.
    struct DemoEngine;

    impl DemoEngine {
        const VERSION: &'static str = "27.3.1";
        const DAY: i64 = 86_400;

        /// The moment of the capture. Ages are counted back from it, so a
        /// resource made 150 days ago reads the same in every capture.
        fn now() -> i64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| {
                    i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
                })
        }

        fn usage(kind: ResourceKind, counts: (u64, u64), megabytes: (u64, u64)) -> KindUsage {
            KindUsage {
                kind,
                total_count: counts.0,
                active_count: counts.1,
                size: ByteSize::from_bytes(megabytes.0 * 1_000_000),
                reclaimable: ByteSize::from_bytes(megabytes.1 * 1_000_000),
            }
        }

        fn unused(kind: ResourceKind, name: &str, megabytes: u64, days: i64) -> DockerResource {
            DockerResource {
                kind,
                reference: format!("demo-{name}-{megabytes}"),
                name: name.to_owned(),
                size: ByteSize::from_bytes(megabytes * 1_000_000),
                created_unix_seconds: (days > 0).then(|| Self::now() - days * Self::DAY),
            }
        }
    }

    impl ContainerEngine for DemoEngine {
        fn status(&self) -> EngineStatus {
            EngineStatus::Running {
                version: Self::VERSION.to_owned(),
            }
        }

        fn inventory(&self) -> Result<DockerInventory, EngineError> {
            use ResourceKind::{BuildCache, Container, Image, Volume};
            Ok(DockerInventory {
                engine_version: Self::VERSION.to_owned(),
                usage: vec![
                    Self::usage(Image, (31, 12), (14_200, 5_400)),
                    Self::usage(Container, (17, 5), (82, 50)),
                    Self::usage(Volume, (28, 9), (2_600, 570)),
                    Self::usage(BuildCache, (212, 0), (8_400, 8_400)),
                ],
                unused: vec![
                    Self::unused(BuildCache, "", 8_400, 0),
                    Self::unused(Image, "portal-web:dev", 1_400, 150),
                    Self::unused(Image, "", 900, 95),
                    Self::unused(Image, "", 600, 160),
                    Self::unused(Image, "postgres:14", 420, 400),
                    Self::unused(Image, "node:18-alpine", 170, 240),
                    Self::unused(Container, "api-faturamento-db-1", 31, 210),
                    Self::unused(Container, "etl-relatorios-run-3f2a", 4, 270),
                    Self::unused(Volume, "api-faturamento_pgdata", 480, 0),
                    Self::unused(Volume, "4f1c9e07a27b", 83, 0),
                ],
            })
        }

        fn disk_usage(&self) -> Result<ByteSize, EngineError> {
            Ok(self.inventory()?.size())
        }

        fn remove(&self, _resource: &DockerResource) -> Result<(), EngineError> {
            Err(EngineError::Failed("demonstration".to_owned()))
        }
    }

    /// With `DISKMARSHAL_DEMO` set, the Docker screen shows a made-up Docker.
    pub(super) fn container_engine() -> Option<Arc<dyn ContainerEngine>> {
        std::env::var_os("DISKMARSHAL_DEMO")
            .map(|_| Arc::new(DemoEngine) as Arc<dyn ContainerEngine>)
    }

    /// With `DISKMARSHAL_APPS` set, the apps of that folder are the ones listed.
    pub(super) fn application_folders() -> Option<Vec<PathBuf>> {
        std::env::var_os("DISKMARSHAL_APPS").map(|folder| vec![PathBuf::from(folder)])
    }

    /// With `DISKMARSHAL_DEMO` set, the monitor shows made-up readings.
    pub(super) fn metrics() -> Option<Arc<dyn MetricsProvider>> {
        std::env::var_os("DISKMARSHAL_DEMO")
            .map(|_| Arc::new(DemoMetrics::default()) as Arc<dyn MetricsProvider>)
    }

    pub(super) fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        }
    }

    pub(super) fn ui_options() -> UiOptions {
        let initial_screen = std::env::var("DISKMARSHAL_SCREEN")
            .ok()
            .and_then(|slug| Screen::from_slug(&slug))
            .unwrap_or_default();
        let palette = std::env::var("DISKMARSHAL_PALETTE")
            .ok()
            .and_then(|slug| Palette::from_slug(&slug))
            .unwrap_or_default();
        UiOptions {
            shows_boot: false,
            motion: MotionPreference::Off,
            initial_screen,
            palette,
            preload_scan: std::env::var_os("DISKMARSHAL_SCAN").map(std::path::PathBuf::from),
            preload_cleanup: std::env::var_os("DISKMARSHAL_ANALYZE").is_some(),
            preload_developer: std::env::var_os("DISKMARSHAL_DEV").map(std::path::PathBuf::from),
            preload_duplicates: std::env::var_os("DISKMARSHAL_DUP").map(std::path::PathBuf::from),
            preload_monitor: true,
            preload_photos: std::env::var_os("DISKMARSHAL_PHOTOS").map(std::path::PathBuf::from),
            preload_apps: std::env::var_os("DISKMARSHAL_APPS").is_some(),
            preload_docker: std::env::var_os("DISKMARSHAL_DOCKER").is_some(),
            preload_network: std::env::var_os("DISKMARSHAL_NETWORK").is_some(),
            // Only the made-up tester: a real test spends real data.
            preload_speed_test: std::env::var_os("DISKMARSHAL_DEMO").is_some(),
            network_tab: std::env::var("DISKMARSHAL_NETWORK").ok(),
        }
    }

    /// A made-up network for documentation screenshots, so they show no
    /// program, address or network of the machine that takes them. The
    /// addresses come from the ranges reserved for documentation.
    mod demo_network {
        use std::net::{IpAddr, Ipv4Addr};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Duration;

        use diskmarshal_application::ports::{
            ConnectionInspector, InspectionError, NetworkTuner, PrivilegeError, ReachabilityProbe,
            SpeedTestError, SpeedTester, TrafficProbe, TrafficProbeError,
        };
        use diskmarshal_domain::network::{
            BitRate, Connection, ConnectionProfile, Endpoint, NetworkChange, PingResult,
            ProcessTraffic, ProxySettings, Responsiveness, SpeedTest, TrafficReading, Transport,
            VpnProfiles, WifiBand, WifiLink,
        };
        use diskmarshal_domain::shared::ByteSize;

        use super::NetworkAdapters;

        const ROUTER: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1));
        const THIS_MAC: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 24));
        const FASTEST_RESOLVER: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        /// Seconds between two made-up readings.
        const STEP_SECONDS: u64 = 2;
        const BYTES_PER_MEGABIT_SECOND: f64 = 125_000.0;
        const NAME_SERVICE_PORT: u16 = 53;

        /// A program, the megabits per second it receives and sends, and
        /// the addresses and ports its connections go to.
        type DemoProgram = (&'static str, f64, f64, &'static [([u8; 4], u16)]);

        const PROGRAMS: [DemoProgram; 6] = [
            (
                "Safari",
                38.0,
                2.1,
                &[
                    ([203, 0, 113, 24], 443),
                    ([203, 0, 113, 80], 443),
                    ([198, 51, 100, 7], 443),
                ],
            ),
            (
                "Backup",
                0.3,
                11.2,
                &[([198, 51, 100, 44], 443), ([198, 51, 100, 45], 443)],
            ),
            ("Música", 2.4, 0.1, &[([203, 0, 113, 140], 443)]),
            ("Mensagens", 0.6, 0.2, &[([192, 0, 2, 200], 443)]),
            ("mDNSResponder", 0.05, 0.03, &[([192, 168, 0, 1], 53)]),
            ("apsd", 0.01, 0.01, &[([203, 0, 113, 5], 5223)]),
        ];

        pub(super) fn adapters() -> NetworkAdapters {
            NetworkAdapters {
                traffic_probe: Arc::new(DemoTraffic::default()),
                connection_inspector: Arc::new(DemoConnection),
                reachability: Arc::new(DemoConnection),
                speed_tester: Arc::new(DemoConnection),
                network_tuner: Arc::new(DemoConnection),
            }
        }

        /// Counters that grow a little differently at each reading.
        #[derive(Default)]
        struct DemoTraffic {
            readings: AtomicU64,
        }

        impl DemoTraffic {
            /// The connections of the program at `index` of the list, with
            /// what they had moved by reading number `step`.
            fn connections(index: usize, step: u64) -> Vec<Connection> {
                let (_, download, upload, destinations) = PROGRAMS[index];
                // The usual speed of the program, swinging by a third.
                let moved = |megabits: f64| {
                    let seconds = (step * STEP_SECONDS) as f64;
                    let swing = (step as f64 * 0.7 + index as f64).sin() * 0.33;
                    let bytes = megabits * BYTES_PER_MEGABIT_SECOND * (seconds + swing);
                    (bytes.max(0.0) / destinations.len() as f64) as u64
                };
                destinations
                    .iter()
                    .enumerate()
                    .map(|(place, (address, port))| Connection {
                        transport: if *port == NAME_SERVICE_PORT {
                            Transport::Udp
                        } else {
                            Transport::Tcp
                        },
                        local: Endpoint {
                            address: THIS_MAC.to_string(),
                            port: Some(50_000 + (index * 10 + place) as u16),
                        },
                        remote: Endpoint {
                            address: Ipv4Addr::from(*address).to_string(),
                            port: Some(*port),
                        },
                        interface: "en0".to_owned(),
                        received: ByteSize::from_bytes(40_000_000 + moved(download)),
                        sent: ByteSize::from_bytes(2_000_000 + moved(upload)),
                        retransmitted: ByteSize::from_bytes(if place == 2 { 8_400 } else { 0 }),
                        round_trip: Some(Duration::from_millis(
                            11 + (index * 7 + place * 5) as u64,
                        )),
                    })
                    .collect()
            }
        }

        impl TrafficProbe for DemoTraffic {
            fn read(&self) -> Result<TrafficReading, TrafficProbeError> {
                let step = self.readings.fetch_add(1, Ordering::Relaxed);
                let processes = PROGRAMS
                    .iter()
                    .enumerate()
                    .map(|(index, (program, ..))| ProcessTraffic {
                        pid: 500 + index as u32,
                        program: (*program).to_owned(),
                        connections: Self::connections(index, step),
                    })
                    .collect();
                Ok(TrafficReading {
                    taken_at: Duration::from_secs(step * STEP_SECONDS),
                    processes,
                })
            }
        }

        /// A wireless connection with two things worth fixing: a proxy
        /// search nobody answers and a slow name server.
        struct DemoConnection;

        impl ConnectionInspector for DemoConnection {
            fn profile(&self) -> Result<ConnectionProfile, InspectionError> {
                Ok(ConnectionProfile {
                    interface: "en0".to_owned(),
                    service: "Wi-Fi".to_owned(),
                    local_address: Some(THIS_MAC),
                    gateway: Some(ROUTER),
                    dns_servers: vec![ROUTER],
                    configured_dns: Vec::new(),
                    proxy: ProxySettings {
                        is_auto_discovery_on: true,
                        is_discovery_answered: Some(false),
                        has_configured_proxy: false,
                    },
                    vpn: VpnProfiles {
                        installed: 3,
                        connected: 0,
                    },
                    wifi: Some(WifiLink {
                        signal_dbm: -52,
                        noise_dbm: -93,
                        channel: 44,
                        band: Some(WifiBand::FiveGigahertz),
                        width_mhz: Some(80),
                        standard: "802.11ax".to_owned(),
                        transmit_rate: BitRate::from_bits_per_second(1_201_000_000),
                        neighbors_on_channel: 3,
                    }),
                })
            }
        }

        impl ReachabilityProbe for DemoConnection {
            fn ping(&self, target: IpAddr) -> PingResult {
                PingResult {
                    sent: 10,
                    received: 10,
                    average: Some(Duration::from_millis(if target == ROUTER { 3 } else { 14 })),
                }
            }

            fn time_dns(&self, server: IpAddr) -> Option<Duration> {
                let millis = if server == ROUTER {
                    38
                } else if server == FASTEST_RESOLVER {
                    9
                } else {
                    13
                };
                Some(Duration::from_millis(millis))
            }
        }

        impl SpeedTester for DemoConnection {
            fn measure(&self) -> Result<SpeedTest, SpeedTestError> {
                Ok(SpeedTest {
                    download: BitRate::from_bits_per_second(468_000_000),
                    upload: BitRate::from_bits_per_second(383_000_000),
                    idle_latency: Some(Duration::from_millis(13)),
                    responsiveness: Some(Responsiveness::from_rpm(440)),
                    transferred: ByteSize::from_bytes(1_400_000_000),
                    duration: Duration::from_secs(18),
                })
            }
        }

        impl NetworkTuner for DemoConnection {
            fn is_available(&self) -> bool {
                true
            }

            /// A demonstration changes nothing.
            fn apply(
                &self,
                _changes: &[NetworkChange],
                _prompt: &str,
            ) -> Result<(), PrivilegeError> {
                Err(PrivilegeError::Failed("demonstration".to_owned()))
            }
        }
    }
}
