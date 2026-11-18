//! The window frame: status bar, navigation rail, working area and overlays.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use diskmarshal_application::ports::{
    AccessStatus, AppBundleReader, ContainerEngine, ContentHasher, ExternalOpener, FileInspector,
    FileRemover, FileSystemWalker, FolderLister, FolderPicker, HistoryRepository, ImageThumbnailer,
    MetricsProvider, PermissionProbe, PrivilegedCleaner, RuleCatalog, RunningAppProbe, TrashBin,
    VersionControlProbe, VolumeStats, WallClock,
};
use diskmarshal_domain::monitoring::VolumeUsage;
use egui::{
    Align, Align2, Area, CentralPanel, Context, FontFamily, FontId, Frame, Id, Key, LayerId,
    Layout, Order, Panel, Rect, RichText, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2,
    WidgetInfo, pos2, vec2,
};

use crate::animation::{MotionPreference, Timeline, ease_out_cubic};
use crate::components::{
    APPEARANCE_PANEL_WIDTH, AppearanceSettings, BootSequence, Icon, appearance_panel, icon_button,
    segmented_bar,
};
use crate::effects::glow::{self, TextStyle};
use crate::effects::{paint_grid, paint_scanlines};
use crate::i18n::pt_br;
use crate::navigation::Screen;
use crate::screens::{ScreenContext, Screens};
use crate::theme::{
    FONT_SIZE_BRAND, FONT_SIZE_DATA, FONT_SIZE_RAIL, FONT_SIZE_SMALL, FONT_SIZE_TITLE,
    MOTION_ENTER_SECONDS, MOTION_SCAN_BAND_SECONDS, MOTION_TYPE_SECONDS_PER_CHARACTER, Palette,
    RAIL_ITEM_HEIGHT, RAIL_WIDTH, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, STATUS_BAR_HEIGHT,
    Tokens, apply_style, display, display_bold, install_fonts,
};

/// The volume whose usage is shown in the status bar and the rail.
const STARTUP_VOLUME: &str = "/";
/// How often the disk usage is read again, in seconds.
const DISK_REFRESH_SECONDS: f64 = 5.0;
/// Repaint interval while nothing is animating: enough for the clock.
const IDLE_REPAINT: Duration = Duration::from_secs(1);
/// Repaint interval of the animations that loop in the background: twenty
/// frames per second. Measured on an M2 Pro, drawing every frame kept the
/// app at 46% of a core while it sat idle; this pace brings it near 10%.
const AMBIENT_REPAINT: Duration = Duration::from_millis(50);
/// For how long after the user does something every frame is drawn.
const SMOOTH_SECONDS: f64 = 1.5;

const BRAND_WIDTH: f32 = 132.0;
const BRAND_LETTER_SPACING: f32 = 2.7;
const RAIL_ICON_SIZE: f32 = 18.0;
const RAIL_LETTER_SPACING: f32 = 1.1;
const RAIL_SHORTCUT_SIZE: f32 = 16.0;
const RAIL_MARKER_WIDTH: f32 = 3.0;
const RAIL_ACTIVE_FILL_OPACITY: f32 = 0.1;
const RAIL_FOOTER_HEIGHT: f32 = 72.0;
const DISK_BAR_CELLS: usize = 18;
const TITLE_LETTER_SPACING: f32 = 3.6;
const TITLE_GLOW_SHARE: f32 = 0.6;
/// How far the content of a screen slides up as it enters, in points.
const ENTER_OFFSET: f32 = 6.0;
const STATUS_LETTER_SPACING: f32 = 1.1;
const BLINKS_PER_SECOND: f64 = 2.0;

/// What the interface needs from the outside world.
#[derive(Clone)]
pub struct UiServices {
    /// Reads how full the startup disk is.
    pub volume_stats: Arc<dyn VolumeStats>,
    /// Reads the time shown in the status bar.
    pub wall_clock: Arc<dyn WallClock>,
    /// Reads folders from disk for every scan.
    pub walker: Arc<dyn FileSystemWalker>,
    /// Shows items in Finder and opens System Settings.
    pub opener: Arc<dyn ExternalOpener>,
    /// Reads the identity and real location of an item.
    pub inspector: Arc<dyn FileInspector>,
    /// Removes items permanently.
    pub remover: Arc<dyn FileRemover>,
    /// Moves items to the Trash.
    pub trash_bin: Arc<dyn TrashBin>,
    /// Records every removal.
    pub history: Arc<dyn HistoryRepository>,
    /// Supplies the catalog of what can be cleaned.
    pub rules: Arc<dyn RuleCatalog>,
    /// Tells whether a folder holds files under version control.
    pub version_control: Arc<dyn VersionControlProbe>,
    /// Lets the user choose a folder with the system dialog.
    pub folder_picker: Arc<dyn FolderPicker>,
    /// Fingerprints file content, to find duplicates.
    pub hasher: Arc<dyn ContentHasher>,
    /// Reads processor, memory, sensor and process figures.
    pub metrics: Arc<dyn MetricsProvider>,
    /// Makes small pictures of image files, to compare and to show.
    pub thumbnailer: Arc<dyn ImageThumbnailer>,
    /// Cleans system areas after macOS authenticates the user.
    pub privileged: Arc<dyn PrivilegedCleaner>,
    /// Finds out which permissions the user gave the app.
    pub permissions: Arc<dyn PermissionProbe>,
    /// Reads what an app bundle says about itself.
    pub bundles: Arc<dyn AppBundleReader>,
    /// Tells which programs are running.
    pub running_apps: Arc<dyn RunningAppProbe>,
    /// Lists what is directly inside a folder.
    pub folder_lister: Arc<dyn FolderLister>,
    /// Talks to the Docker engine installed on this Mac.
    pub container_engine: Arc<dyn ContainerEngine>,
    /// The folders where apps are installed.
    pub application_folders: Vec<PathBuf>,
    /// The user's home folder.
    pub home: PathBuf,
}

/// How the interface starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiOptions {
    /// Whether the start-up sequence plays before the first screen.
    pub shows_boot: bool,
    /// How much animation to show.
    pub motion: MotionPreference,
    /// The screen shown first.
    pub initial_screen: Screen,
    /// The color scheme the interface starts with.
    pub palette: Palette,
    /// A folder to measure before the first frame, so the disk map opens
    /// filled in. `None` waits for the user to choose.
    pub preload_scan: Option<PathBuf>,
    /// Whether to analyze the cleanup rules before the first frame.
    pub preload_cleanup: bool,
    /// A folder to search for build folders before the first frame.
    pub preload_developer: Option<PathBuf>,
    /// A folder to search for duplicates before the first frame.
    pub preload_duplicates: Option<PathBuf>,
    /// Whether to take one reading of the system before the first frame.
    pub preload_monitor: bool,
    /// A folder to search for similar photos before the first frame.
    pub preload_photos: Option<PathBuf>,
    /// Whether to list the installed apps before the first frame.
    pub preload_apps: bool,
    /// Whether to ask Docker what it holds before the first frame.
    pub preload_docker: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            shows_boot: true,
            motion: MotionPreference::default(),
            initial_screen: Screen::default(),
            palette: Palette::default(),
            preload_scan: None,
            preload_cleanup: false,
            preload_developer: None,
            preload_duplicates: None,
            preload_monitor: false,
            preload_photos: None,
            preload_apps: false,
            preload_docker: false,
        }
    }
}

/// The whole DiskMarshal interface. The binary creates one and calls
/// [`show`](Self::show) every frame.
pub struct DiskMarshalUi {
    services: UiServices,
    screen: Screen,
    appearance: AppearanceSettings,
    tokens: Tokens,
    boot: Option<BootSequence>,
    screen_entered: Timeline,
    is_appearance_open: bool,
    disk_usage: Option<VolumeUsage>,
    full_disk_access: AccessStatus,
    /// When the disk usage and the permissions were last read.
    system_read_at: Option<f64>,
    /// When the user last moved the pointer, clicked or typed.
    last_input_at: f64,
    screens: Screens,
}

impl DiskMarshalUi {
    /// Creates the interface and installs its fonts and theme on `ctx`.
    #[must_use]
    pub fn new(ctx: &Context, services: UiServices, options: &UiOptions) -> Self {
        let appearance = AppearanceSettings {
            motion: options.motion,
            palette: options.palette,
            ..Default::default()
        };
        let tokens = appearance.palette.tokens();
        install_fonts(ctx);
        apply_style(ctx, &tokens);

        let mut screens = Screens::default();
        if let Some(root) = &options.preload_scan {
            screens.preload_scan(services.walker.as_ref(), root);
        }
        if options.preload_cleanup {
            screens.preload_cleanup(&services);
        }
        if let Some(root) = &options.preload_developer {
            screens.preload_developer(&services, root);
        }
        if let Some(root) = &options.preload_duplicates {
            screens.preload_duplicates(&services, root);
        }
        if options.preload_monitor {
            screens.preload_monitor(&services);
        }
        if let Some(root) = &options.preload_photos {
            screens.preload_photos(ctx, &services, root);
        }
        if options.preload_apps {
            screens.preload_apps(&services);
        }
        if options.preload_docker {
            screens.preload_docker(&services);
        }
        let now = ctx.input(|input| input.time);
        let plays_boot = options.shows_boot && options.motion.allows_ambient();
        Self {
            services,
            screen: options.initial_screen,
            appearance,
            tokens,
            boot: plays_boot.then(|| BootSequence::start(now)),
            screen_entered: Timeline::start(now),
            is_appearance_open: false,
            disk_usage: None,
            full_disk_access: AccessStatus::Unknown,
            system_read_at: None,
            last_input_at: now,
            screens,
        }
    }

    /// Color the window is cleared with before anything is painted.
    #[must_use]
    pub fn clear_color(&self) -> [f32; 4] {
        self.tokens.color_surface_void.to_normalized_gamma_f32()
    }

    /// Draws one frame of the interface.
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|input| input.time);
        self.refresh_system_facts(now);
        self.schedule_repaint(&ctx, now);

        if let Some(boot) = self.boot {
            let is_over = CentralPanel::no_frame()
                .show(ui, |ui| {
                    boot.show(ui, &self.tokens, now, self.appearance.glow_strength)
                })
                .inner;
            if is_over {
                self.boot = None;
                self.screen_entered = Timeline::start(now);
            }
            self.paint_scanlines(&ctx, now);
            return;
        }

        self.handle_shortcuts(&ctx, now);
        let window_fill = Frame::NONE.fill(self.tokens.color_surface_window);
        let toggled_appearance = Panel::top("status_bar")
            .exact_size(STATUS_BAR_HEIGHT)
            .resizable(false)
            .show_separator_line(false)
            .frame(window_fill)
            .show(ui, |ui| self.status_bar(ui))
            .inner;
        Panel::left("rail")
            .exact_size(RAIL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(window_fill)
            .show(ui, |ui| self.rail(ui, now));
        CentralPanel::no_frame()
            .frame(window_fill)
            .show(ui, |ui| self.stage(ui, now));

        if toggled_appearance {
            self.is_appearance_open = !self.is_appearance_open;
        }
        self.appearance_overlay(&ctx, now, toggled_appearance);
        self.paint_scanlines(&ctx, now);
    }

    /// Reads again, every few seconds, the facts about the system the frame
    /// shows: how full the disk is and what macOS lets the app read.
    fn refresh_system_facts(&mut self, now: f64) {
        let is_stale = self
            .system_read_at
            .is_none_or(|read_at| now - read_at >= DISK_REFRESH_SECONDS);
        if is_stale {
            self.disk_usage = self
                .services
                .volume_stats
                .usage(Path::new(STARTUP_VOLUME))
                .ok();
            self.full_disk_access = self.services.permissions.full_disk_access();
            self.system_read_at = Some(now);
        }
    }

    /// Paces the frames to what is on screen.
    ///
    /// Right after the user does something, every frame is drawn, so
    /// transitions are smooth. After that, the animations that loop in the
    /// background get a slower, steady pace. Without focus, or with looping
    /// motion turned off, one frame per second keeps the clock right without
    /// spending processor or battery.
    fn schedule_repaint(&mut self, ctx: &Context, now: f64) {
        let (is_focused, had_input) = ctx.input(|input| {
            (
                input.focused,
                !input.events.is_empty() || input.pointer.is_moving(),
            )
        });
        if had_input {
            self.last_input_at = now;
        }
        let motion = self.appearance.motion;
        let is_settling = self.boot.is_some()
            || now - self.last_input_at < SMOOTH_SECONDS
            || self.screen_entered.elapsed(now) < SMOOTH_SECONDS;
        if is_focused && motion.allows_transitions() && is_settling {
            ctx.request_repaint();
        } else if is_focused && motion.allows_ambient() {
            ctx.request_repaint_after(AMBIENT_REPAINT);
        } else {
            ctx.request_repaint_after(IDLE_REPAINT);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context, now: f64) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let pressed = Screen::ALL
            .into_iter()
            .find(|screen| ctx.input(|input| input.key_pressed(screen.shortcut().0)));
        if let Some(screen) = pressed {
            self.go_to(screen, now);
        }
    }

    fn go_to(&mut self, screen: Screen, now: f64) {
        if screen != self.screen {
            self.screen = screen;
            self.screen_entered = Timeline::start(now);
        }
    }

    /// Paints the status bar and returns `true` when the appearance icon was clicked.
    fn status_bar(&self, ui: &mut Ui) -> bool {
        let tokens = self.tokens;
        let rect = ui.max_rect();
        let mut toggled = false;
        ui.horizontal_centered(|ui| {
            ui.add_space(SPACE_3);
            let (brand, _) =
                ui.allocate_exact_size(vec2(BRAND_WIDTH, STATUS_BAR_HEIGHT), Sense::hover());
            glow::text(
                ui.painter(),
                brand.left_center(),
                Align2::LEFT_CENTER,
                pt_br::BRAND,
                &TextStyle {
                    font: FontId::new(FONT_SIZE_BRAND, display_bold()),
                    color: tokens.color_accent,
                    letter_spacing: BRAND_LETTER_SPACING,
                    glow: self.appearance.glow_strength,
                },
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(SPACE_3);
                toggled = icon_button(
                    ui,
                    &tokens,
                    Icon::Sliders,
                    self.is_appearance_open,
                    pt_br::APPEARANCE,
                )
                .clicked();
                ui.add_space(SPACE_2);
                ui.label(status_text(
                    self.services.wall_clock.now().to_string(),
                    &tokens,
                ));
                if let Some(usage) = self.disk_usage {
                    ui.add_space(SPACE_3);
                    let free = format!("{} {}", usage.available, pt_br::FREE_SPACE_SUFFIX);
                    ui.label(status_text(free.to_uppercase(), &tokens));
                }
            });
        });
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() - 0.5,
            Stroke::new(1.0, tokens.color_line),
        );
        toggled
    }

    fn rail(&mut self, ui: &mut Ui, now: f64) {
        let tokens = self.tokens;
        let rect = ui.max_rect();
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.add_space(SPACE_3);

        let mut active_top = rect.top();
        for screen in Screen::ALL {
            let (row, response) =
                ui.allocate_exact_size(vec2(RAIL_WIDTH, RAIL_ITEM_HEIGHT), Sense::click());
            let is_active = screen == self.screen;
            if is_active {
                active_top = row.top();
            }
            self.rail_item(
                ui,
                row,
                screen,
                is_active,
                response.hovered() || response.has_focus(),
            );
            response.widget_info(|| {
                WidgetInfo::selected(Role::Button, true, is_active, screen.label())
            });
            if response.clicked() {
                self.go_to(screen, now);
            }
        }

        let marker_top = if self.appearance.motion.allows_transitions() {
            ui.ctx().animate_value_with_time(
                Id::unique("rail_marker"),
                active_top,
                MOTION_ENTER_SECONDS,
            )
        } else {
            active_top
        };
        let marker_x = rect.left() + RAIL_MARKER_WIDTH / 2.0;
        glow::line(
            ui.painter(),
            [
                pos2(marker_x, marker_top),
                pos2(marker_x, marker_top + RAIL_ITEM_HEIGHT),
            ],
            tokens.color_accent,
            RAIL_MARKER_WIDTH,
            self.appearance.glow_strength,
        );

        let footer = Rect::from_min_max(
            pos2(rect.left() + SPACE_4, rect.bottom() - RAIL_FOOTER_HEIGHT),
            pos2(rect.right() - SPACE_4, rect.bottom() - SPACE_3),
        );
        ui.painter().hline(
            rect.x_range(),
            footer.top() - SPACE_3,
            Stroke::new(1.0, tokens.color_line),
        );
        ui.scope_builder(UiBuilder::new().max_rect(footer), |ui| self.disk_footer(ui));
        ui.painter().vline(
            rect.right() - 0.5,
            rect.y_range(),
            Stroke::new(1.0, tokens.color_line),
        );
    }

    fn rail_item(&self, ui: &Ui, row: Rect, screen: Screen, is_active: bool, is_lit: bool) {
        let tokens = self.tokens;
        let painter = ui.painter();
        let color = if is_active {
            tokens.color_accent
        } else if is_lit {
            tokens.color_text_primary
        } else {
            tokens.color_text_secondary
        };
        if is_active {
            painter.rect_filled(
                row,
                0.0,
                tokens.color_accent.gamma_multiply(RAIL_ACTIVE_FILL_OPACITY),
            );
        }

        let icon_center = pos2(row.left() + SPACE_4 + RAIL_ICON_SIZE / 2.0, row.center().y);
        screen.icon().paint(
            painter,
            Rect::from_center_size(icon_center, Vec2::splat(RAIL_ICON_SIZE)),
            color,
        );
        glow::text(
            painter,
            pos2(
                row.left() + SPACE_4 + RAIL_ICON_SIZE + SPACE_3,
                row.center().y,
            ),
            Align2::LEFT_CENTER,
            &screen.label().to_uppercase(),
            &TextStyle {
                font: FontId::new(FONT_SIZE_RAIL, display()),
                color,
                letter_spacing: RAIL_LETTER_SPACING,
                glow: 0.0,
            },
        );

        let shortcut_center = pos2(
            row.right() - SPACE_4 - RAIL_SHORTCUT_SIZE / 2.0,
            row.center().y,
        );
        let shortcut = Rect::from_center_size(shortcut_center, Vec2::splat(RAIL_SHORTCUT_SIZE));
        painter.rect_stroke(
            shortcut,
            0.0,
            Stroke::new(1.0, tokens.color_line),
            StrokeKind::Inside,
        );
        painter.text(
            shortcut.center(),
            Align2::CENTER_CENTER,
            screen.shortcut().1,
            FontId::new(FONT_SIZE_SMALL - 1.0, FontFamily::Monospace),
            tokens.color_text_muted,
        );
    }

    fn disk_footer(&self, ui: &mut Ui) {
        let tokens = self.tokens;
        ui.spacing_mut().item_spacing.y = SPACE_1 + 2.0;
        let Some(usage) = self.disk_usage else {
            ui.label(small_text(pt_br::DISK_UNKNOWN, tokens.color_text_muted));
            return;
        };

        let ratio = usage.used_ratio() as f32;
        ui.horizontal(|ui| {
            ui.label(small_text(
                &pt_br::STARTUP_DISK.to_uppercase(),
                tokens.color_text_secondary,
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let percent = format!("{:.0}%", ratio * 100.0);
                ui.label(small_text(&percent, tokens.color_text_muted));
            });
        });
        let bar_color = if usage.is_critical() {
            tokens.color_warn
        } else {
            tokens.color_accent
        };
        segmented_bar(ui, &tokens, ratio, DISK_BAR_CELLS, bar_color);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_1;
            let amounts = format!("{} {} {}", usage.used(), pt_br::DISK_OF, usage.total);
            ui.label(small_text(&amounts, tokens.color_text_secondary));
            if usage.is_critical() {
                ui.label(small_text(pt_br::DISK_CRITICAL, tokens.color_warn));
            }
        });
    }

    fn stage(&mut self, ui: &mut Ui, now: f64) {
        let tokens = self.tokens;
        let rect = ui.max_rect();
        paint_grid(ui.painter(), rect, &tokens);

        let motion = self.appearance.motion;
        let entered = if motion.allows_transitions() {
            ease_out_cubic(
                self.screen_entered
                    .progress(now, f64::from(MOTION_ENTER_SECONDS)),
            )
        } else {
            1.0
        };
        let content = rect
            .shrink2(vec2(SPACE_5, SPACE_4))
            .translate(vec2(0.0, (1.0 - entered) * ENTER_OFFSET));
        let requested = ui.scope_builder(UiBuilder::new().max_rect(content), |ui| {
            ui.set_opacity(entered);
            self.stage_title(ui, now);
            ui.add_space(SPACE_1);
            ui.label(
                RichText::new(self.screen.subtitle())
                    .size(FONT_SIZE_DATA - 1.0)
                    .color(tokens.color_text_secondary),
            );
            ui.add_space(SPACE_3);
            self.permission_notice(ui);
            let context = ScreenContext {
                tokens: &tokens,
                appearance: &self.appearance,
                services: &self.services,
                now,
            };
            self.screens.show(self.screen, ui, &context)
        });
        if let Some(screen) = requested.inner {
            self.go_to(screen, now);
        }
    }

    /// Says, on the screens that read the disk, that macOS is hiding part of
    /// it, with the way to fix that one click away.
    fn permission_notice(&self, ui: &mut Ui) {
        if self.full_disk_access != AccessStatus::Denied || !self.screen.reads_personal_folders() {
            return;
        }
        let tokens = self.tokens;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
            ui.label(
                RichText::new(pt_br::FULL_DISK_ACCESS_MISSING)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_warn),
            );
            let link = ui.add(
                egui::Label::new(
                    RichText::new(pt_br::OPEN_FULL_DISK_ACCESS)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_accent),
                )
                .sense(Sense::click()),
            );
            if link
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                // If System Settings does not open, the text still says what is missing.
                let _ = self.services.opener.open_full_disk_access_settings();
            }
        });
        ui.add_space(SPACE_2);
    }

    /// Paints the screen title, typed out letter by letter, with a blinking cursor.
    fn stage_title(&self, ui: &mut Ui, now: f64) {
        let tokens = self.tokens;
        let motion = self.appearance.motion;
        let title = self.screen.title().to_uppercase();
        let length = title.chars().count();
        let typed = if motion.allows_transitions() {
            let duration = MOTION_TYPE_SECONDS_PER_CHARACTER * length as f64;
            (self.screen_entered.progress(now, duration) * length as f32).ceil() as usize
        } else {
            length
        };
        let visible: String = title.chars().take(typed).collect();

        let (row, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), FONT_SIZE_TITLE), Sense::hover());
        let font = FontId::new(FONT_SIZE_TITLE, display_bold());
        let text_rect = glow::text(
            ui.painter(),
            row.left_center(),
            Align2::LEFT_CENTER,
            &visible,
            &TextStyle {
                font: font.clone(),
                color: tokens.color_text_primary,
                letter_spacing: TITLE_LETTER_SPACING,
                glow: self.appearance.glow_strength * TITLE_GLOW_SHARE,
            },
        );
        let is_cursor_on = !motion.allows_ambient() || (now * BLINKS_PER_SECOND) as i64 % 2 == 0;
        if is_cursor_on {
            ui.painter().text(
                pos2(text_rect.right() + SPACE_1, row.center().y),
                Align2::LEFT_CENTER,
                "_",
                font,
                tokens.color_accent,
            );
        }
    }

    fn appearance_overlay(&mut self, ctx: &Context, now: f64, toggled_this_frame: bool) {
        if !self.is_appearance_open {
            return;
        }
        let window = ctx.content_rect();
        let position = pos2(
            window.right() - APPEARANCE_PANEL_WIDTH - SPACE_3,
            window.top() + STATUS_BAR_HEIGHT + SPACE_2,
        );
        let tokens = self.tokens;
        let shown = Area::new(Id::unique("appearance_panel"))
            .order(Order::Foreground)
            .fixed_pos(position)
            .show(ctx, |ui| {
                ui.set_width(APPEARANCE_PANEL_WIDTH);
                appearance_panel(ui, &tokens, &mut self.appearance)
            });

        if shown.inner.palette_changed {
            self.tokens = self.appearance.palette.tokens();
            apply_style(ctx, &self.tokens);
        }
        if shown.inner.replay_boot {
            self.boot = Some(BootSequence::start(now));
            self.is_appearance_open = false;
        }
        let clicked_outside = shown.response.clicked_elsewhere() && !toggled_this_frame;
        if clicked_outside || ctx.input(|input| input.key_pressed(Key::Escape)) {
            self.is_appearance_open = false;
        }
    }

    fn paint_scanlines(&self, ctx: &Context, now: f64) {
        let painter = ctx.layer_painter(LayerId::new(Order::Debug, Id::unique("scanline_overlay")));
        let band = self
            .appearance
            .motion
            .allows_ambient()
            .then(|| ((now % MOTION_SCAN_BAND_SECONDS) / MOTION_SCAN_BAND_SECONDS) as f32);
        paint_scanlines(&painter, ctx.content_rect(), &self.tokens, band);
    }
}

fn status_text(text: String, tokens: &Tokens) -> RichText {
    RichText::new(text)
        .size(FONT_SIZE_SMALL)
        .extra_letter_spacing(STATUS_LETTER_SPACING)
        .color(tokens.color_text_secondary)
}

fn small_text(text: &str, color: egui::Color32) -> RichText {
    RichText::new(text).size(FONT_SIZE_SMALL).color(color)
}
