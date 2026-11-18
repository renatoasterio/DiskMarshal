//! Drawing of the dashboard.

use std::path::Path;

use egui::{
    Align, Align2, CursorIcon, FontFamily, FontId, Layout, Rect, RichText, Role, Sense, Ui,
    WidgetInfo, pos2, vec2,
};
use maccleaner_domain::deletion::HistoryAction;
use maccleaner_domain::monitoring::ThermalLevel;
use maccleaner_domain::shared::ByteSize;

use super::view_model::{DashboardAction, DashboardViewModel, Recoverable};
use crate::components::{
    ButtonKind, CellWidth, HudPanel, RadialGauge, cell_row, hud_button, key_value,
    paint_segmented_bar, radar, radial_gauge,
};
use crate::i18n::pt_br;
use crate::navigation::Screen;
use crate::screens::monitor::MonitorViewModel;
use crate::screens::{ScreenContext, display_path};
use crate::theme::{
    FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_4, Tokens, display_bold,
};

const GAUGE_PANEL_WIDTH: f32 = 172.0;
const TOP_ROW_HEIGHT: f32 = 208.0;
const TOTAL_FONT_SIZE: f32 = 40.0;
const RADAR_DIAMETER: f32 = 140.0;
const BAR_CELLS: usize = 18;
const BAR_HEIGHT: f32 = 8.0;
/// Geometry of a row of the recoverable list: name, bar, size.
const ROW_HEIGHT: f32 = 26.0;
const ROW_LABEL_WIDTH: f32 = 168.0;
const ROW_VALUE_WIDTH: f32 = 104.0;
/// Below this width the bar says nothing, so the row goes without it.
const ROW_BAR_MIN_WIDTH: f32 = 48.0;
/// Time the radar sweep takes to go around.
const SWEEP_SECONDS: f64 = 4.0;
/// Where the sweep rests when motion is off.
const SWEEP_AT_REST: f32 = 0.875;

/// Draws the screen for this frame and returns what the user asked for.
pub(crate) fn show(
    model: &mut DashboardViewModel,
    monitor: &mut MonitorViewModel,
    recoverable: Recoverable,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
) -> Option<DashboardAction> {
    monitor.ensure_started(ui.ctx(), context.services);
    monitor.poll();
    model.refresh(context.services, context.now);
    let (model, monitor) = (&*model, &*monitor);
    let tokens = context.tokens;
    let mut action = None;

    let gauge = CellWidth::Fixed(GAUGE_PANEL_WIDTH);
    cell_row(
        ui,
        &[gauge, gauge, gauge, CellWidth::Share(1.0)],
        |index, ui| match index {
            0 => disk_gauge(ui, context),
            1 => memory_gauge(ui, monitor, context),
            2 => heat_gauge(ui, monitor, context),
            _ => {
                HudPanel::new(pt_br::DASH_RECOVERABLE)
                    .min_height(TOP_ROW_HEIGHT)
                    .show(ui, tokens, |ui| {
                        action = recoverable_panel(ui, tokens, recoverable);
                    });
            }
        },
    );
    ui.add_space(crate::theme::SPACE_3);

    let remaining = ui.available_height();
    cell_row(
        ui,
        &[CellWidth::Share(1.35), CellWidth::Share(1.0)],
        |index, ui| {
            if index == 0 {
                HudPanel::new(pt_br::DASH_HISTORY)
                    .meta(pt_br::DASH_HISTORY_META)
                    .min_height(remaining)
                    .show(ui, tokens, |ui| history(ui, model, context));
            } else {
                HudPanel::new(pt_br::DASH_SYSTEM)
                    .min_height(remaining)
                    .show(ui, tokens, |ui| {
                        if system_panel(ui, monitor, recoverable, context) {
                            action = Some(DashboardAction::AnalyzeCleanup);
                        }
                    });
            }
        },
    );
    action
}

fn gauge_panel(ui: &mut Ui, context: &ScreenContext<'_>, title: &str, gauge: &RadialGauge<'_>) {
    HudPanel::new(title)
        .min_height(TOP_ROW_HEIGHT)
        .show(ui, context.tokens, |ui| {
            radial_gauge(ui, context.tokens, context.appearance, gauge);
        });
}

/// A gauge with nothing to show yet.
const fn empty_gauge(name: &str) -> RadialGauge<'_> {
    RadialGauge {
        name,
        ratio: 0.0,
        value: "—",
        unit: "",
        is_warning: false,
    }
}

fn disk_gauge(ui: &mut Ui, context: &ScreenContext<'_>) {
    let Ok(usage) = context.services.volume_stats.usage(Path::new("/")) else {
        return gauge_panel(ui, context, pt_br::DASH_DISK, &empty_gauge("disk"));
    };
    let value = format!("{:.0}%", usage.used_ratio() * 100.0);
    let unit = format!("{} {}", usage.available, pt_br::FREE_SPACE_SUFFIX);
    let gauge = RadialGauge {
        name: "disk",
        ratio: usage.used_ratio() as f32,
        value: &value,
        unit: &unit,
        is_warning: usage.is_critical(),
    };
    gauge_panel(ui, context, pt_br::DASH_DISK, &gauge);
}

fn memory_gauge(ui: &mut Ui, monitor: &MonitorViewModel, context: &ScreenContext<'_>) {
    let Some(snapshot) = monitor.latest() else {
        return gauge_panel(ui, context, pt_br::DASH_MEMORY, &empty_gauge("memory"));
    };
    let value = format!("{:.0}%", snapshot.memory_ratio() * 100.0);
    let unit = format!("{} {}", snapshot.memory_used, pt_br::MONITOR_IN_USE);
    let gauge = RadialGauge {
        name: "memory",
        ratio: snapshot.memory_ratio(),
        value: &value,
        unit: &unit,
        is_warning: false,
    };
    gauge_panel(ui, context, pt_br::DASH_MEMORY, &gauge);
}

/// The hottest component when a sensor answers; processor use otherwise.
fn heat_gauge(ui: &mut Ui, monitor: &MonitorViewModel, context: &ScreenContext<'_>) {
    let snapshot = monitor.latest();
    if let Some(sensor) = snapshot.and_then(|snapshot| snapshot.hottest()) {
        let value = format!("{:.0} °C", sensor.celsius);
        let gauge = RadialGauge {
            name: "temperature",
            ratio: sensor.scale_ratio(),
            value: &value,
            unit: &sensor.label,
            is_warning: sensor.level() != ThermalLevel::Normal,
        };
        return gauge_panel(ui, context, pt_br::DASH_TEMPERATURE, &gauge);
    }
    let usage = snapshot.map_or(0.0, |snapshot| snapshot.cpu_usage);
    let value = snapshot.map_or_else(|| "—".to_owned(), |_| format!("{:.0}%", usage * 100.0));
    let gauge = RadialGauge {
        name: "cpu",
        ratio: usage,
        value: &value,
        unit: pt_br::DASH_CPU_USE,
        is_warning: false,
    };
    gauge_panel(ui, context, pt_br::MONITOR_CPU, &gauge);
}

/// The total that can be recovered and where it comes from. Clicking a row
/// opens its screen.
fn recoverable_panel(
    ui: &mut Ui,
    tokens: &Tokens,
    recoverable: Recoverable,
) -> Option<DashboardAction> {
    let total = recoverable.total();
    let headline = total.map_or_else(|| "—".to_owned(), |total| total.to_string());
    ui.label(
        RichText::new(headline)
            .font(FontId::new(TOTAL_FONT_SIZE, display_bold()))
            .color(tokens.color_ok),
    );
    ui.add_space(SPACE_1);

    let whole = total.map_or(0, ByteSize::as_bytes);
    let rows = [
        (
            Screen::Cleanup,
            pt_br::DASH_ROW_CLEANUP,
            recoverable.cleanup,
        ),
        (
            Screen::Developer,
            pt_br::DASH_ROW_DEVELOPER,
            recoverable.developer,
        ),
        (
            Screen::Duplicates,
            pt_br::DASH_ROW_DUPLICATES,
            recoverable.duplicates,
        ),
        (Screen::Photos, pt_br::DASH_ROW_PHOTOS, recoverable.photos),
    ];
    ui.spacing_mut().item_spacing.y = 0.0;
    let mut action = None;
    for (screen, label, size) in rows {
        if recoverable_row(ui, tokens, label, size, whole) {
            action = Some(DashboardAction::GoTo(screen));
        }
    }
    action
}

/// One source of recoverable space. Returns `true` when it was clicked.
fn recoverable_row(
    ui: &mut Ui,
    tokens: &Tokens,
    label: &str,
    size: Option<ByteSize>,
    whole: u64,
) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let painter = ui.painter_at(rect);
    let font = FontId::new(FONT_SIZE_DATA, FontFamily::Monospace);
    painter.hline(rect.x_range(), rect.top(), (1.0, tokens.color_line));

    let is_highlighted = response.hovered() || response.has_focus();
    let label_color = if is_highlighted {
        tokens.color_accent
    } else {
        tokens.color_text_primary
    };
    painter.text(
        rect.left_center(),
        Align2::LEFT_CENTER,
        label,
        font.clone(),
        label_color,
    );

    let (value, value_color) = match size {
        Some(size) => (size.to_string(), tokens.color_text_primary),
        None => (pt_br::DASH_NOT_ANALYZED.to_owned(), tokens.color_text_muted),
    };
    painter.text(
        rect.right_center(),
        Align2::RIGHT_CENTER,
        value,
        font,
        value_color,
    );

    let bar = Rect::from_min_max(
        pos2(
            rect.left() + ROW_LABEL_WIDTH,
            rect.center().y - BAR_HEIGHT / 2.0,
        ),
        pos2(
            rect.right() - ROW_VALUE_WIDTH,
            rect.center().y + BAR_HEIGHT / 2.0,
        ),
    );
    if bar.width() >= ROW_BAR_MIN_WIDTH {
        let ratio = match size {
            Some(size) if whole > 0 => (size.as_bytes() as f64 / whole as f64) as f32,
            _ => 0.0,
        };
        paint_segmented_bar(
            &painter,
            bar,
            ratio,
            BAR_CELLS,
            tokens.color_accent,
            tokens.color_line,
        );
    }
    response.clicked()
}

fn history(ui: &mut Ui, model: &DashboardViewModel, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    if model.history().is_empty() {
        ui.label(RichText::new(pt_br::DASH_HISTORY_EMPTY).color(tokens.color_text_secondary));
        return;
    }
    let now = context.services.wall_clock.unix_seconds();
    ui.spacing_mut().item_spacing.y = SPACE_1;
    for entry in model.history() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
            let (verb, color) = match entry.action {
                HistoryAction::Deleted => (pt_br::DASH_DELETED, tokens.color_accent),
                HistoryAction::Trashed => (pt_br::DASH_TRASHED, tokens.color_warn),
            };
            let age = pt_br::age(now - entry.timestamp_unix_seconds);
            ui.label(
                RichText::new(age)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_text_muted),
            );
            ui.label(RichText::new(verb).size(FONT_SIZE_SMALL).color(color));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(entry.reclaimable.to_string()).size(FONT_SIZE_SMALL));
                let path = display_path(&entry.path, &context.services.home);
                ui.add(
                    egui::Label::new(
                        RichText::new(path)
                            .size(FONT_SIZE_SMALL)
                            .color(tokens.color_text_secondary),
                    )
                    .truncate(),
                );
            });
        });
    }
}

/// The radar and the one-click analysis. Returns `true` when the user asked
/// for the analysis.
fn system_panel(
    ui: &mut Ui,
    monitor: &MonitorViewModel,
    recoverable: Recoverable,
    context: &ScreenContext<'_>,
) -> bool {
    let tokens = context.tokens;
    let mut wants_analysis = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_4;
        let sweep = if context.appearance.motion.allows_ambient() {
            // The sweep goes faster while an analysis is running.
            let period = if recoverable.is_analyzing {
                SWEEP_SECONDS / 4.0
            } else {
                SWEEP_SECONDS
            };
            ((context.now % period) / period) as f32
        } else {
            SWEEP_AT_REST
        };
        radar(ui, tokens, RADAR_DIAMETER, Some(sweep));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1;
            if let Some(snapshot) = monitor.latest() {
                let cpu = format!("{:.0}%", snapshot.cpu_usage * 100.0);
                key_value(ui, tokens, pt_br::MONITOR_CPU, &cpu);
                key_value(
                    ui,
                    tokens,
                    pt_br::MONITOR_MEMORY,
                    &snapshot.memory_used.to_string(),
                );
                if let Some(sensor) = snapshot.hottest() {
                    let degrees = format!("{:.0} °C · {}", sensor.celsius, sensor.label);
                    key_value(ui, tokens, pt_br::DASH_TEMPERATURE, &degrees);
                }
            }
            key_value(ui, tokens, "iCloud", pt_br::DASH_NO_DOWNLOADS);
            ui.add_space(SPACE_2);
            if recoverable.is_analyzing {
                ui.label(RichText::new(pt_br::CLEANUP_ANALYZING).color(tokens.color_accent));
            } else if hud_button(ui, tokens, pt_br::DASH_ANALYZE, ButtonKind::Primary).clicked() {
                wants_analysis = true;
            }
        });
    });
    wants_analysis
}
