//! The analysis tab: the speed test, the wireless link, the way to the
//! internet and what all of it adds up to.

use diskmarshal_domain::network::{
    Finding, NetworkReport, Severity, SignalQuality, SpeedTest, WifiLink,
};
use egui::{Align, Color32, Layout, RichText, ScrollArea, Ui, vec2};

use super::view::{badge, meter_row, small};
use super::view_model::NetworkViewModel;
use crate::components::{
    ButtonKind, CellWidth, HudPanel, activity_bar, cell_row, hud_button, key_value, readout,
    segmented_bar,
};
use crate::i18n::pt_br;
use crate::screens::ScreenContext;
use crate::theme::{FONT_SIZE_DATA, SPACE_1, SPACE_2, SPACE_3, Tokens};

/// Height of the four speed panels, so they end on the same line.
const SPEED_PANEL_HEIGHT: f32 = 92.0;
const SPEED_BAR_CELLS: usize = 18;
/// Height of the wireless and path panels.
const LINK_PANEL_HEIGHT: f32 = 172.0;
/// Width kept for the buttons beside the note of the speed test.
const CONTROLS_WIDTH: f32 = 390.0;
/// The top of each speed bar: a gigabit, a tenth of a second and a
/// thousand round trips per minute.
const FULL_SPEED_MEGABITS: f64 = 1_000.0;
const FULL_LATENCY_MILLIS: f64 = 100.0;
const FULL_RESPONSIVENESS_RPM: f64 = 1_000.0;
/// The top of the bar of the wireless transmit rate, in megabits per second.
const FULL_WIFI_RATE_MEGABITS: f64 = 2_400.0;
/// Space the title and the frame of a panel take from its height.
const PANEL_CHROME: f32 = 50.0;

/// What the user asked for in this frame.
enum Request {
    MeasureSpeed,
    Analyze,
}

/// Draws the tab for this frame.
pub(super) fn show(model: &mut NetworkViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    speed_panels(ui, tokens, model.speed_test());
    ui.add_space(SPACE_2);
    let mut request = speed_controls(model, ui, tokens);
    ui.add_space(SPACE_3);

    if model.is_analyzing() {
        HudPanel::new(pt_br::NETWORK_ANALYZE_TITLE).show(ui, tokens, |ui| {
            ui.label(RichText::new(pt_br::NETWORK_ANALYZING).color(tokens.color_text_secondary));
            ui.add_space(SPACE_2);
            activity_bar(ui, tokens, context.now, true);
        });
    } else if let Some(report) = model.report().cloned() {
        cell_row(ui, &[CellWidth::Share(1.0); 2], |index, ui| {
            if index == 0 {
                wifi_panel(ui, tokens, report.profile.wifi.as_ref());
            } else {
                path_panel(ui, tokens, &report);
            }
        });
        ui.add_space(SPACE_3);
        diagnosis_panel(ui, tokens, &model.findings());
    } else {
        HudPanel::new(pt_br::NETWORK_ANALYZE_TITLE).show(ui, tokens, |ui| {
            ui.label(RichText::new(pt_br::NETWORK_ANALYZE_BODY).color(tokens.color_text_secondary));
            if let Some(failure) = model.analysis_failure() {
                ui.add_space(SPACE_2);
                ui.label(RichText::new(failure).color(tokens.color_alert));
            }
            ui.add_space(SPACE_3);
            if hud_button(ui, tokens, pt_br::NETWORK_ANALYZE, ButtonKind::Primary).clicked() {
                request = Some(Request::Analyze);
            }
        });
    }

    match request {
        Some(Request::MeasureSpeed) => model.measure_speed(ui.ctx(), context.services),
        Some(Request::Analyze) => model.analyze(ui.ctx(), context.services),
        None => {}
    }
}

fn speed_panels(ui: &mut Ui, tokens: &Tokens, test: Option<&SpeedTest>) {
    let figure = |value: Option<f64>, full: f64| match value {
        Some(value) => (pt_br::decimal(value, 0), (value / full) as f32),
        None => (pt_br::NETWORK_NOT_MEASURED.to_owned(), 0.0),
    };
    let panels = [
        (
            pt_br::NETWORK_DOWNLOAD,
            pt_br::NETWORK_UNIT_MEGABITS,
            figure(
                test.map(|test| test.download.megabits_per_second()),
                FULL_SPEED_MEGABITS,
            ),
        ),
        (
            pt_br::NETWORK_UPLOAD,
            pt_br::NETWORK_UNIT_MEGABITS,
            figure(
                test.map(|test| test.upload.megabits_per_second()),
                FULL_SPEED_MEGABITS,
            ),
        ),
        (
            pt_br::NETWORK_LATENCY,
            pt_br::NETWORK_UNIT_MILLIS,
            figure(
                test.and_then(|test| test.idle_latency)
                    .map(|latency| latency.as_secs_f64() * 1_000.0),
                FULL_LATENCY_MILLIS,
            ),
        ),
        (
            pt_br::NETWORK_RESPONSIVENESS,
            pt_br::NETWORK_UNIT_RPM,
            figure(
                test.and_then(|test| test.responsiveness)
                    .map(|responsiveness| f64::from(responsiveness.rpm())),
                FULL_RESPONSIVENESS_RPM,
            ),
        ),
    ];
    cell_row(ui, &[CellWidth::Share(1.0); 4], |index, ui| {
        let Some((title, unit, (value, ratio))) = panels.get(index) else {
            return;
        };
        HudPanel::new(title)
            .min_height(SPEED_PANEL_HEIGHT)
            .show(ui, tokens, |ui| {
                readout(ui, tokens, value, unit);
                segmented_bar(ui, tokens, *ratio, SPEED_BAR_CELLS, tokens.color_accent);
            });
    });
}

/// The note about the speed test and the buttons that start a measurement.
fn speed_controls(model: &NetworkViewModel, ui: &mut Ui, tokens: &Tokens) -> Option<Request> {
    let (note, color) = if model.is_measuring_speed() {
        (
            pt_br::NETWORK_SPEED_RUNNING.to_owned(),
            tokens.color_text_secondary,
        )
    } else if let Some(failure) = model.speed_failure() {
        (failure.to_owned(), tokens.color_alert)
    } else if let Some(test) = model.speed_test() {
        (pt_br::network_speed_done(test), tokens.color_text_muted)
    } else {
        (
            pt_br::NETWORK_SPEED_NOTE.to_owned(),
            tokens.color_text_muted,
        )
    };
    let mut request = None;
    ui.horizontal(|ui| {
        let note_width = (ui.available_width() - CONTROLS_WIDTH).max(0.0);
        ui.allocate_ui_with_layout(vec2(note_width, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.label(small(&note, color));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
            let label = if model.speed_test().is_some() {
                pt_br::NETWORK_MEASURE_AGAIN
            } else {
                pt_br::NETWORK_MEASURE_SPEED
            };
            ui.add_enabled_ui(!model.is_measuring_speed(), |ui| {
                if hud_button(ui, tokens, label, ButtonKind::Outline).clicked() {
                    request = Some(Request::MeasureSpeed);
                }
            });
            if model.report().is_some() {
                ui.add_enabled_ui(!model.is_analyzing(), |ui| {
                    if hud_button(ui, tokens, pt_br::NETWORK_ANALYZE_AGAIN, ButtonKind::Quiet)
                        .clicked()
                    {
                        request = Some(Request::Analyze);
                    }
                });
            }
        });
    });
    request
}

fn wifi_panel(ui: &mut Ui, tokens: &Tokens, link: Option<&WifiLink>) {
    let Some(link) = link else {
        HudPanel::new(pt_br::NETWORK_WIFI)
            .min_height(LINK_PANEL_HEIGHT)
            .show(ui, tokens, |ui| {
                ui.label(small(pt_br::NETWORK_NO_WIFI, tokens.color_text_muted));
            });
        return;
    };
    let standard = pt_br::network_wifi_standard(link);
    HudPanel::new(pt_br::NETWORK_WIFI)
        .meta(&standard)
        .min_height(LINK_PANEL_HEIGHT)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1 + 1.0;
            let signal_color = match link.quality() {
                SignalQuality::Strong => tokens.color_ok,
                SignalQuality::Fair => tokens.color_warn,
                SignalQuality::Weak => tokens.color_alert,
            };
            let signal = format!("{} dBm", link.signal_dbm);
            meter_row(
                ui,
                tokens,
                pt_br::NETWORK_SIGNAL,
                link.signal_ratio(),
                &signal,
                signal_color,
            );
            let noise = format!("{} dBm", link.noise_dbm);
            meter_row(
                ui,
                tokens,
                pt_br::NETWORK_NOISE,
                link.noise_ratio(),
                &noise,
                tokens.color_ok,
            );
            let rate_ratio =
                (link.transmit_rate.megabits_per_second() / FULL_WIFI_RATE_MEGABITS) as f32;
            let rate = link.transmit_rate.to_string();
            meter_row(
                ui,
                tokens,
                pt_br::NETWORK_RATE,
                rate_ratio,
                &rate,
                tokens.color_accent,
            );
            ui.add_space(SPACE_1);
            let channel = pt_br::network_wifi_channel(link);
            ui.label(small(&channel, tokens.color_text_muted));
        });
}

fn path_panel(ui: &mut Ui, tokens: &Tokens, report: &NetworkReport) {
    HudPanel::new(pt_br::NETWORK_PATH)
        .min_height(LINK_PANEL_HEIGHT)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1;
            let profile = &report.profile;
            let rows = [
                (
                    pt_br::NETWORK_ROUTER,
                    pt_br::network_router(profile.gateway, report.gateway.as_ref()),
                ),
                (
                    pt_br::NETWORK_INTERNET,
                    pt_br::network_internet(report.internet.as_ref()),
                ),
                (pt_br::NETWORK_DNS_IN_USE, pt_br::network_dns_in_use(report)),
                (
                    pt_br::NETWORK_PROXY,
                    pt_br::network_proxy(profile.proxy).to_owned(),
                ),
                (pt_br::NETWORK_VPN, pt_br::network_vpn(profile.vpn)),
            ];
            for (label, value) in rows {
                key_value(ui, tokens, label, &value);
            }
        });
}

fn diagnosis_panel(ui: &mut Ui, tokens: &Tokens, findings: &[Finding]) {
    let attention = findings
        .iter()
        .filter(|finding| finding.severity() == Severity::Attention)
        .count();
    let meta = pt_br::NETWORK_ATTENTION_POINTS.of(attention);
    let height = ui.available_height();
    HudPanel::new(pt_br::NETWORK_DIAGNOSIS)
        .meta(&meta)
        .min_height(height)
        .show(ui, tokens, |ui| {
            ScrollArea::vertical()
                .id_salt("network_findings")
                .max_height((height - PANEL_CHROME).max(SPEED_PANEL_HEIGHT))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = SPACE_2;
                    for finding in findings {
                        let severity = finding.severity();
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = SPACE_3;
                            badge(
                                ui,
                                pt_br::network_severity(severity),
                                severity_color(tokens, severity),
                            );
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(pt_br::network_finding(finding))
                                        .size(FONT_SIZE_DATA - 1.0),
                                );
                            });
                        });
                    }
                });
        });
}

/// The color that tells how much a finding asks of the user.
fn severity_color(tokens: &Tokens, severity: Severity) -> Color32 {
    match severity {
        Severity::Attention => tokens.color_warn,
        Severity::Info => tokens.color_text_secondary,
        Severity::Good => tokens.color_ok,
    }
}
