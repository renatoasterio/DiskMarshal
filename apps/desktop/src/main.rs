//! DiskMarshal desktop application. Composition root for the graphical
//! interface: wires the macOS adapters into the interface and opens the window.

#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context as _, anyhow};
use diskmarshal_application::ports::{ContainerEngine, MetricsProvider};
use diskmarshal_infrastructure::bundles::{PlistAppBundleReader, application_folders};
use diskmarshal_infrastructure::catalog::TomlRuleCatalog;
use diskmarshal_infrastructure::filesystem::{
    LstatFileInspector, ReaddirFolderLister, ReaddirWalker, SafeFileRemover, StatfsVolumeStats,
    disable_dataless_materialization, icloud_sync_roots,
};
use diskmarshal_infrastructure::hashing::Blake3ContentHasher;
use diskmarshal_infrastructure::imaging::ImageIoThumbnailer;
use diskmarshal_infrastructure::metrics::{SysinfoMetricsProvider, UnavailableMetrics};
use diskmarshal_infrastructure::permissions::FullDiskAccessProbe;
use diskmarshal_infrastructure::persistence::JsonlHistoryRepository;
use diskmarshal_infrastructure::privilege::OsascriptPrivilegedCleaner;
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
            privileged: Arc::new(OsascriptPrivilegedCleaner::beside_current_executable()),
            permissions: Arc::new(FullDiskAccessProbe::for_home(&home)),
            bundles: Arc::new(PlistAppBundleReader::for_home(&home)),
            running_apps: Arc::new(SysinfoRunningAppProbe),
            folder_lister: Arc::new(ReaddirFolderLister),
            container_engine: capture::container_engine().unwrap_or_else(|| docker(&home)),
            application_folders: capture::application_folders()
                .unwrap_or_else(|| application_folders(&home)),
            metrics: capture::metrics().unwrap_or_else(system_metrics),
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

    use super::UiOptions;

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
/// asks Docker first. Run with `HOME` pointing at a demonstration folder to
/// keep every personal path out of the picture.
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

    use super::UiOptions;

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
        }
    }
}
