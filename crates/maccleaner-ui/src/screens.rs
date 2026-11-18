//! The seven screens. Each lives in its own folder as a view and a view model.

mod cleanup;
mod dashboard;
mod developer;
mod disk_map;
mod duplicates;
mod group_review;
mod monitor;
mod photos;

use std::path::Path;

use egui::Ui;
use maccleaner_application::ports::FileSystemWalker;

use crate::app::UiServices;
use crate::components::AppearanceSettings;
use crate::navigation::Screen;
use crate::theme::Tokens;

/// What every screen receives when it draws a frame.
pub(crate) struct ScreenContext<'a> {
    pub tokens: &'a Tokens,
    pub appearance: &'a AppearanceSettings,
    pub services: &'a UiServices,
    /// Seconds on the frame clock.
    pub now: f64,
}

/// The state of every screen, kept while the user moves between them.
#[derive(Default)]
pub(crate) struct Screens {
    cleanup: cleanup::CleanupViewModel,
    dashboard: dashboard::DashboardViewModel,
    developer: developer::DeveloperViewModel,
    disk_map: disk_map::DiskMapViewModel,
    duplicates: duplicates::DuplicatesViewModel,
    monitor: monitor::MonitorViewModel,
    photos: photos::PhotosViewModel,
}

impl Screens {
    /// Draws the given screen and returns the screen the user asked to open, if any.
    pub(crate) fn show(
        &mut self,
        screen: Screen,
        ui: &mut Ui,
        context: &ScreenContext<'_>,
    ) -> Option<Screen> {
        // Readings keep arriving whatever screen is open, so the trace has no gaps.
        self.monitor.poll();
        match screen {
            Screen::Dashboard => return self.show_dashboard(ui, context),
            Screen::Cleanup => cleanup::show(&mut self.cleanup, ui, context),
            Screen::Developer => developer::show(&mut self.developer, ui, context),
            Screen::DiskMap => disk_map::show(&mut self.disk_map, ui, context),
            Screen::Duplicates => duplicates::show(&mut self.duplicates, ui, context),
            Screen::Monitor => monitor::show(&mut self.monitor, ui, context),
            Screen::Photos => photos::show(&mut self.photos, ui, context),
        }
        None
    }

    fn show_dashboard(&mut self, ui: &mut Ui, context: &ScreenContext<'_>) -> Option<Screen> {
        // The analysis can end while the dashboard is on screen.
        self.cleanup.poll();
        let recoverable = dashboard::Recoverable {
            cleanup: self.cleanup.recoverable(),
            developer: self.developer.recoverable(),
            duplicates: self.duplicates.recoverable(),
            photos: self.photos.recoverable(),
            is_analyzing: self.cleanup.progress().is_some(),
        };
        let action = dashboard::show(
            &mut self.dashboard,
            &mut self.monitor,
            recoverable,
            ui,
            context,
        );
        match action? {
            dashboard::DashboardAction::GoTo(screen) => Some(screen),
            dashboard::DashboardAction::AnalyzeCleanup => {
                self.cleanup.analyze(ui.ctx(), context.services);
                None
            }
        }
    }

    /// Takes one reading of the system before the first frame.
    pub(crate) fn preload_monitor(&mut self, services: &UiServices) {
        self.monitor.sample_now(services);
    }

    /// Analyzes the cleanup rules before the first frame, so that screen
    /// opens filled in.
    pub(crate) fn preload_cleanup(&mut self, services: &UiServices) {
        self.cleanup.analyze_now(services);
    }

    /// Searches `root` for build folders before the first frame, so the
    /// developer screen opens filled in.
    pub(crate) fn preload_developer(&mut self, services: &UiServices, root: &Path) {
        self.developer.search_now(services, root);
    }

    /// Searches `root` for duplicates before the first frame, so that screen
    /// opens filled in.
    pub(crate) fn preload_duplicates(&mut self, services: &UiServices, root: &Path) {
        self.duplicates.search_now(services, root);
    }

    /// Searches `root` for similar photos before the first frame, so that
    /// screen opens filled in.
    pub(crate) fn preload_photos(
        &mut self,
        ctx: &egui::Context,
        services: &UiServices,
        root: &Path,
    ) {
        self.photos.search_now(ctx, services, root);
    }

    /// Measures `root` before the first frame, so the disk map opens filled in.
    pub(crate) fn preload_scan(&mut self, walker: &dyn FileSystemWalker, root: &Path) {
        self.disk_map.scan_now(walker, root);
    }
}

/// Writes a path the way people read it: the home folder becomes `~`.
pub(crate) fn display_path(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

/// Cuts the middle out of a text longer than `max_chars`, keeping its
/// beginning and its end: the two parts of a path that tell it apart.
pub(crate) fn shorten_middle(text: &str, max_chars: usize) -> String {
    let length = text.chars().count();
    if length <= max_chars || max_chars < 3 {
        return text.to_owned();
    }
    let kept = max_chars - 1;
    let (head, tail) = (kept / 2, kept - kept / 2);
    let start: String = text.chars().take(head).collect();
    let end: String = text.chars().skip(length - tail).collect();
    format!("{start}…{end}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{display_path, shorten_middle};

    #[test]
    fn shortens_long_text_in_the_middle_and_leaves_short_text_alone() {
        assert_eq!(shorten_middle("~/Projetos", 20), "~/Projetos");
        let long = "/Volumes/Arquivo/Clientes/2024/Relatórios/Fotos";
        let short = shorten_middle(long, 21);
        assert_eq!(short.chars().count(), 21);
        assert!(short.starts_with("/Volumes/A") && short.ends_with("rios/Fotos"));
        assert!(short.contains('…'));
    }

    #[test]
    fn shortens_the_home_folder_to_a_tilde() {
        let home = Path::new("/Users/tester");
        assert_eq!(display_path(Path::new("/Users/tester"), home), "~");
        assert_eq!(
            display_path(Path::new("/Users/tester/code/web"), home),
            "~/code/web"
        );
        assert_eq!(
            display_path(Path::new("/Library/Caches"), home),
            "/Library/Caches"
        );
        assert_eq!(
            display_path(Path::new("/Users/tester-two/x"), home),
            "/Users/tester-two/x"
        );
    }
}
