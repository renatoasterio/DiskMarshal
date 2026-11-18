//! Drawing of the monitor screen.

use std::path::Path;

use diskmarshal_domain::monitoring::{SensorReading, SystemSnapshot, ThermalLevel};
use egui::{
    Align, Align2, Color32, FontFamily, FontId, Layout, Rect, RichText, Sense, Ui, pos2, vec2,
};

use super::view_model::MonitorViewModel;
use crate::components::{
    CellWidth, HudPanel, cell_row, key_value, line_plot, readout, segmented_bar,
};
use crate::i18n::pt_br;
use crate::screens::ScreenContext;
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

const PLOT_HEIGHT: f32 = 132.0;
const CORE_BARS_HEIGHT: f32 = PLOT_HEIGHT;
/// Height of the memory, disk and temperature panels: room for five sensors.
const MIDDLE_ROW_HEIGHT: f32 = 176.0;
const BAR_CELLS: usize = 18;

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut MonitorViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.ensure_started(ui.ctx(), context.services);
    model.poll();
    let tokens = context.tokens;
    let Some(snapshot) = model.latest().cloned() else {
        HudPanel::new(pt_br::MONITOR_WAITING_TITLE).show(ui, tokens, |ui| {
            ui.label(RichText::new(pt_br::MONITOR_WAITING_BODY).color(tokens.color_text_secondary));
        });
        return;
    };

    cell_row(
        ui,
        &[CellWidth::Share(1.5), CellWidth::Share(1.0)],
        |index, ui| {
            if index == 0 {
                let percent = format!("{:.0}%", snapshot.cpu_usage * 100.0);
                HudPanel::new(pt_br::MONITOR_CPU)
                    .meta(&percent)
                    .show(ui, tokens, |ui| {
                        let history = model.cpu_history();
                        line_plot(
                            ui,
                            tokens,
                            &history,
                            PLOT_HEIGHT,
                            context.appearance.glow_strength,
                        );
                    });
            } else {
                let cores = pt_br::MONITOR_CORES.of(snapshot.core_usage.len());
                HudPanel::new(pt_br::MONITOR_PER_CORE)
                    .meta(&cores)
                    .show(ui, tokens, |ui| {
                        core_bars(ui, tokens, &snapshot.core_usage);
                    });
            }
        },
    );
    ui.add_space(SPACE_3);

    cell_row(ui, &[CellWidth::Share(1.0); 3], |index, ui| match index {
        0 => memory_panel(ui, tokens, &snapshot),
        1 => disk_panel(ui, context),
        _ => temperature_panel(ui, tokens, &snapshot.sensors),
    });
    ui.add_space(SPACE_3);

    let remaining = ui.available_height();
    HudPanel::new(pt_br::MONITOR_PROCESSES)
        .meta(pt_br::MONITOR_PROCESSES_META)
        .min_height(remaining)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1;
            let busiest = snapshot
                .processes
                .first()
                .map_or(1.0, |process| process.cpu_cores.max(0.01));
            for process in &snapshot.processes {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&process.name));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_3;
                        let percent = format!("{:>5.0}%", process.cpu_cores * 100.0);
                        ui.label(RichText::new(percent).family(FontFamily::Monospace));
                        ui.allocate_ui(vec2(150.0, 8.0), |ui| {
                            let ratio = process.cpu_cores / busiest;
                            segmented_bar(ui, tokens, ratio, BAR_CELLS, tokens.color_accent);
                        });
                        ui.label(
                            RichText::new(process.memory.to_string())
                                .size(FONT_SIZE_SMALL)
                                .color(tokens.color_text_muted),
                        );
                    });
                });
            }
        });
}

/// One vertical bar per processor core.
fn core_bars(ui: &mut Ui, tokens: &Tokens, cores: &[f32]) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), CORE_BARS_HEIGHT), Sense::hover());
    if cores.is_empty() {
        return;
    }
    let painter = ui.painter_at(rect);
    let label_height = 14.0;
    let gap = 5.0;
    let count = cores.len() as f32;
    let width = (rect.width() - gap * (count - 1.0)) / count;
    let top = rect.top();
    let bottom = rect.bottom() - label_height;
    for (index, usage) in cores.iter().enumerate() {
        let left = rect.left() + index as f32 * (width + gap);
        let track = Rect::from_min_max(pos2(left, top), pos2(left + width, bottom));
        painter.rect_filled(track, 0.0, tokens.color_line.gamma_multiply(0.5));
        let height = track.height() * usage.clamp(0.0, 1.0);
        let bar = Rect::from_min_max(pos2(left, bottom - height), pos2(left + width, bottom));
        painter.rect_filled(bar, 0.0, tokens.color_accent);
        painter.text(
            pos2(left + width / 2.0, rect.bottom()),
            Align2::CENTER_BOTTOM,
            (index + 1).to_string(),
            FontId::new(FONT_SIZE_SMALL - 2.0, FontFamily::Monospace),
            tokens.color_text_muted,
        );
    }
}

fn memory_panel(ui: &mut Ui, tokens: &Tokens, snapshot: &SystemSnapshot) {
    HudPanel::new(pt_br::MONITOR_MEMORY)
        .min_height(MIDDLE_ROW_HEIGHT)
        .show(ui, tokens, |ui| {
            let unit = format!("{} {}", pt_br::DISK_OF, snapshot.memory_total);
            readout(ui, tokens, &snapshot.memory_used.to_string(), &unit);
            segmented_bar(
                ui,
                tokens,
                snapshot.memory_ratio(),
                BAR_CELLS,
                tokens.color_accent,
            );
            ui.add_space(SPACE_2);
            let free = snapshot.memory_total.saturating_sub(snapshot.memory_used);
            key_value(ui, tokens, pt_br::MONITOR_FREE, &free.to_string());
            key_value(
                ui,
                tokens,
                pt_br::MONITOR_SWAP,
                &snapshot.swap_used.to_string(),
            );
        });
}

fn disk_panel(ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::MONITOR_DISK)
        .min_height(MIDDLE_ROW_HEIGHT)
        .show(ui, tokens, |ui| {
            let Ok(usage) = context.services.volume_stats.usage(Path::new("/")) else {
                ui.label(RichText::new(pt_br::DISK_UNKNOWN).color(tokens.color_text_muted));
                return;
            };
            let unit = format!("{} {}", pt_br::DISK_OF, usage.total);
            readout(ui, tokens, &usage.used().to_string(), &unit);
            let color = if usage.is_critical() {
                tokens.color_warn
            } else {
                tokens.color_accent
            };
            segmented_bar(ui, tokens, usage.used_ratio() as f32, BAR_CELLS, color);
            ui.add_space(SPACE_2);
            key_value(
                ui,
                tokens,
                pt_br::MONITOR_FREE,
                &usage.available.to_string(),
            );
            key_value(
                ui,
                tokens,
                pt_br::MONITOR_VOLUME,
                pt_br::MONITOR_STARTUP_VOLUME,
            );
        });
}

fn temperature_panel(ui: &mut Ui, tokens: &Tokens, sensors: &[SensorReading]) {
    HudPanel::new(pt_br::MONITOR_TEMPERATURE)
        .min_height(MIDDLE_ROW_HEIGHT)
        .show(ui, tokens, |ui| {
            if sensors.is_empty() {
                ui.label(
                    RichText::new(pt_br::MONITOR_NO_SENSORS)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_text_muted),
                );
            }
            ui.spacing_mut().item_spacing.y = SPACE_1 + 1.0;
            for sensor in sensors {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_2;
                    ui.allocate_ui(vec2(58.0, FONT_SIZE_DATA + 4.0), |ui| {
                        ui.label(RichText::new(&sensor.label).size(FONT_SIZE_DATA - 1.0));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let degrees = format!("{:.0} °C", sensor.celsius);
                        ui.label(RichText::new(degrees).family(FontFamily::Monospace));
                        let color = thermal_color(tokens, sensor.level());
                        segmented_bar(ui, tokens, sensor.scale_ratio(), 12, color);
                    });
                });
            }
        });
}

/// The color that tells how warm a reading is.
fn thermal_color(tokens: &Tokens, level: ThermalLevel) -> Color32 {
    match level {
        ThermalLevel::Normal => tokens.color_ok,
        ThermalLevel::Warm => tokens.color_warn,
        ThermalLevel::Hot => tokens.color_alert,
    }
}
