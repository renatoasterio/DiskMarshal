//! MacCleaner desktop application. Composition root for the graphical
//! interface: wires the macOS adapters into the interface and opens the window.

#![forbid(unsafe_code)]

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context as _, anyhow};
use eframe::egui;
use maccleaner_infrastructure::filesystem::{StatfsVolumeStats, disable_dataless_materialization};
use maccleaner_infrastructure::system::LocaltimeWallClock;
use maccleaner_ui::{MacCleanerUi, UiOptions, UiServices, WINDOW_SIZE};

const APP_NAME: &str = "MacCleaner";

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
    interface: MacCleanerUi,
}

impl DesktopApp {
    fn new(ctx: &egui::Context) -> Self {
        let services = UiServices {
            volume_stats: Arc::new(StatfsVolumeStats),
            wall_clock: Arc::new(LocaltimeWallClock),
        };
        Self {
            interface: MacCleanerUi::new(ctx, services, capture::ui_options()),
        }
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

/// Normal start-up: default renderer, start-up sequence and full motion.
#[cfg(not(feature = "capture"))]
mod capture {
    use super::UiOptions;

    pub(super) fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions::default()
    }

    pub(super) fn ui_options() -> UiOptions {
        UiOptions::default()
    }
}

/// Development build used to check the interface against the prototype.
///
/// With `EFRAME_SCREENSHOT_TO=<file>.png` set, eframe saves the second frame
/// and exits. That hook exists only in the OpenGL renderer, and the frame has
/// to be still, so this build skips the start-up sequence and all motion.
/// `MACCLEANER_SCREEN=<slug>` and `MACCLEANER_PALETTE=<slug>` choose what to capture.
#[cfg(feature = "capture")]
mod capture {
    use maccleaner_ui::{MotionPreference, Palette, Screen};

    use super::UiOptions;

    pub(super) fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        }
    }

    pub(super) fn ui_options() -> UiOptions {
        let initial_screen = std::env::var("MACCLEANER_SCREEN")
            .ok()
            .and_then(|slug| Screen::from_slug(&slug))
            .unwrap_or_default();
        let palette = std::env::var("MACCLEANER_PALETTE")
            .ok()
            .and_then(|slug| Palette::from_slug(&slug))
            .unwrap_or_default();
        UiOptions {
            shows_boot: false,
            motion: MotionPreference::Off,
            initial_screen,
            palette,
        }
    }
}
