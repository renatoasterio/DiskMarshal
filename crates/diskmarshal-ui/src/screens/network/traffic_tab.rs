//! The traffic tab: how fast this Mac is receiving and sending, which
//! programs are doing it, and where the connections of one of them go.

use diskmarshal_domain::network::{BitRate, Connection, ProgramTraffic, TrafficSnapshot};
use egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Response, RichText, Role, ScrollArea,
    Sense, Stroke, Ui, WidgetInfo, pos2, vec2,
};

use super::view::small;
use super::view_model::NetworkViewModel;
use crate::components::{
    CellWidth, HudPanel, Trace, cell_row, key_value, line_plot, paint_segmented_bar,
};
use crate::i18n::pt_br;
use crate::screens::ScreenContext;
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

const PLOT_HEIGHT: f32 = 96.0;
/// Height of the two panels at the top, so they end on the same line.
const TOP_ROW_HEIGHT: f32 = 186.0;
const RATE_FONT_SIZE: f32 = 16.0;
const HEADER_HEIGHT: f32 = 22.0;
const PROGRAM_ROW_HEIGHT: f32 = 30.0;
const CONNECTION_ROW_HEIGHT: f32 = 38.0;
/// Width of a column of figures.
const FIGURE_WIDTH: f32 = 82.0;
const ANSWER_WIDTH: f32 = 58.0;
const BAR_WIDTH: f32 = 48.0;
const BAR_HEIGHT: f32 = 8.0;
const BAR_CELLS: usize = 8;
const SELECTED_FILL_OPACITY: f32 = 0.1;
const HOVER_FILL_OPACITY: f32 = 0.05;
const MARKER_WIDTH: f32 = 2.0;
/// Space the title and the frame of a panel take from its height.
const PANEL_CHROME: f32 = 50.0;

/// Draws the tab for this frame.
pub(super) fn show(model: &mut NetworkViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    let Some(snapshot) = model.traffic().cloned() else {
        let (title, body, color) = match model.traffic_failure() {
            Some(failure) => (pt_br::NETWORK_UNREADABLE_TITLE, failure, tokens.color_alert),
            None => (
                pt_br::NETWORK_WAITING_TITLE,
                pt_br::NETWORK_WAITING_BODY,
                tokens.color_text_secondary,
            ),
        };
        HudPanel::new(title).show(ui, tokens, |ui| {
            ui.label(RichText::new(body).color(color));
        });
        return;
    };

    let plot = model.plot();
    cell_row(
        ui,
        &[CellWidth::Share(1.5), CellWidth::Share(1.0)],
        |index, ui| {
            if index == 0 {
                HudPanel::new(pt_br::NETWORK_NOW)
                    .min_height(TOP_ROW_HEIGHT)
                    .show(ui, tokens, |ui| {
                        rates(ui, tokens, &snapshot);
                        let traces = [
                            Trace {
                                values: &plot.upload,
                                color: tokens.color_ok,
                            },
                            Trace {
                                values: &plot.download,
                                color: tokens.color_accent,
                            },
                        ];
                        let glow = context.appearance.glow_strength;
                        line_plot(ui, tokens, &traces, PLOT_HEIGHT, glow);
                        let scale = pt_br::network_scale(plot.top);
                        ui.label(small(&scale, tokens.color_text_muted));
                    });
            } else {
                session_panel(ui, tokens, &snapshot);
            }
        },
    );
    if let Some(failure) = model.traffic_failure() {
        ui.add_space(SPACE_1);
        ui.label(small(failure, tokens.color_alert));
    }
    ui.add_space(SPACE_3);

    let shown = model.shown_program().cloned();
    let remaining = ui.available_height();
    let mut picked: Option<String> = None;
    cell_row(
        ui,
        &[CellWidth::Share(1.0), CellWidth::Share(1.2)],
        |index, ui| {
            if index == 0 {
                let shown_name = shown.as_ref().map(|program| program.name.as_str());
                picked = programs_panel(ui, tokens, &snapshot, shown_name, remaining);
            } else {
                connections_panel(ui, tokens, shown.as_ref(), remaining);
            }
        },
    );
    if let Some(program) = picked {
        model.select_program(&program);
    }
}

/// The two speeds, each in the color of its trace.
fn rates(ui: &mut Ui, tokens: &Tokens, snapshot: &TrafficSnapshot) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_3 + SPACE_2;
        let rate = |text: String, color: Color32| {
            RichText::new(text)
                .font(FontId::new(RATE_FONT_SIZE, FontFamily::Monospace))
                .color(color)
        };
        ui.label(rate(
            format!("↓ {}", snapshot.download),
            tokens.color_accent,
        ));
        ui.label(rate(format!("↑ {}", snapshot.upload), tokens.color_ok));
    });
    ui.add_space(SPACE_1);
}

fn session_panel(ui: &mut Ui, tokens: &Tokens, snapshot: &TrafficSnapshot) {
    HudPanel::new(pt_br::NETWORK_SESSION)
        .min_height(TOP_ROW_HEIGHT)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1 + 2.0;
            let resent = format!(
                "{}%",
                pt_br::decimal(snapshot.retransmitted_ratio() * 100.0, 1)
            );
            let rows = [
                (pt_br::NETWORK_RECEIVED, snapshot.received.to_string()),
                (pt_br::NETWORK_SENT, snapshot.sent.to_string()),
                (pt_br::NETWORK_PEAK, snapshot.peak_download.to_string()),
                (
                    pt_br::NETWORK_CONNECTIONS_WITH_DATA,
                    pt_br::group_digits(snapshot.connection_count() as u64),
                ),
                (
                    pt_br::NETWORK_PROGRAMS_ON,
                    pt_br::group_digits(snapshot.programs.len() as u64),
                ),
                (pt_br::NETWORK_RESENT, resent),
            ];
            for (label, value) in rows {
                key_value(ui, tokens, label, &value);
            }
        });
}

/// Lists the programs and returns the one the user clicked, if any.
fn programs_panel(
    ui: &mut Ui,
    tokens: &Tokens,
    snapshot: &TrafficSnapshot,
    shown: Option<&str>,
    height: f32,
) -> Option<String> {
    let mut picked = None;
    HudPanel::new(pt_br::NETWORK_PROGRAMS)
        .meta(pt_br::NETWORK_PROGRAMS_META)
        .min_height(height)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            if snapshot.programs.is_empty() {
                ui.label(small(pt_br::NETWORK_NO_TRAFFIC, tokens.color_text_muted));
                return;
            }
            let (header, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), HEADER_HEIGHT), Sense::hover());
            let columns = ProgramColumns::of(header);
            let painter = ui.painter_at(header);
            heading(
                &painter,
                tokens,
                pos2(columns.name_left, header.center().y),
                Align2::LEFT_CENTER,
                pt_br::NETWORK_COLUMN_PROGRAM,
            );
            heading(
                &painter,
                tokens,
                pos2(columns.download_right, header.center().y),
                Align2::RIGHT_CENTER,
                pt_br::NETWORK_COLUMN_RECEIVING,
            );
            heading(
                &painter,
                tokens,
                pos2(columns.upload_right, header.center().y),
                Align2::RIGHT_CENTER,
                pt_br::NETWORK_COLUMN_SENDING,
            );

            let busiest = snapshot
                .programs
                .iter()
                .map(ProgramTraffic::rate)
                .max()
                .unwrap_or(BitRate::ZERO);
            ScrollArea::vertical()
                .id_salt("network_programs")
                .max_height((height - PANEL_CHROME - HEADER_HEIGHT).max(PROGRAM_ROW_HEIGHT))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for program in &snapshot.programs {
                        let is_shown = shown == Some(program.name.as_str());
                        if program_row(ui, tokens, program, busiest, is_shown).clicked() {
                            picked = Some(program.name.clone());
                        }
                    }
                });
        });
    picked
}

/// Where each column of the list of programs is.
struct ProgramColumns {
    name_left: f32,
    download_right: f32,
    upload_right: f32,
    bar: Rect,
}

impl ProgramColumns {
    fn of(row: Rect) -> Self {
        let bar_right = row.right() - SPACE_2;
        let bar = Rect::from_min_max(
            pos2(bar_right - BAR_WIDTH, row.center().y - BAR_HEIGHT / 2.0),
            pos2(bar_right, row.center().y + BAR_HEIGHT / 2.0),
        );
        let upload_right = bar.left() - SPACE_3;
        Self {
            name_left: row.left() + SPACE_2,
            download_right: upload_right - FIGURE_WIDTH,
            upload_right,
            bar,
        }
    }
}

fn program_row(
    ui: &mut Ui,
    tokens: &Tokens,
    program: &ProgramTraffic,
    busiest: BitRate,
    is_shown: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), PROGRAM_ROW_HEIGHT),
        Sense::click(),
    );
    let painter = ui.painter_at(rect);
    row_background(&painter, tokens, rect, is_shown, response.hovered());
    let columns = ProgramColumns::of(rect);
    let middle = rect.center().y;
    let name_area = Rect::from_min_max(
        pos2(columns.name_left, rect.top()),
        pos2(columns.download_right - FIGURE_WIDTH, rect.bottom()),
    );
    painter.with_clip_rect(name_area).text(
        pos2(columns.name_left, middle),
        Align2::LEFT_CENTER,
        &program.name,
        FontId::new(FONT_SIZE_DATA - 1.0, FontFamily::Proportional),
        tokens.color_text_primary,
    );
    figure(
        &painter,
        pos2(columns.download_right, middle),
        &program.download.to_string(),
        tokens.color_accent,
    );
    figure(
        &painter,
        pos2(columns.upload_right, middle),
        &program.upload.to_string(),
        tokens.color_ok,
    );
    let share = if busiest.is_zero() {
        0.0
    } else {
        (program.rate().as_bits_per_second() as f64 / busiest.as_bits_per_second() as f64) as f32
    };
    paint_segmented_bar(
        &painter,
        columns.bar,
        share,
        BAR_CELLS,
        tokens.color_accent,
        tokens.color_line,
    );
    response.widget_info(|| WidgetInfo::selected(Role::Button, true, is_shown, &program.name));
    response
}

fn connections_panel(ui: &mut Ui, tokens: &Tokens, program: Option<&ProgramTraffic>, height: f32) {
    let title = program.map_or_else(
        || pt_br::NETWORK_CONNECTIONS.to_owned(),
        |program| pt_br::network_connections_title(&program.name),
    );
    let count = program.map(|program| pt_br::NETWORK_OPEN.of(program.connections.len()));
    let mut panel = HudPanel::new(&title).min_height(height);
    if let Some(count) = &count {
        panel = panel.meta(count);
    }
    panel.show(ui, tokens, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let Some(program) = program else {
            ui.label(small(
                pt_br::NETWORK_NO_CONNECTIONS,
                tokens.color_text_muted,
            ));
            return;
        };
        let (header, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), HEADER_HEIGHT), Sense::hover());
        let columns = ConnectionColumns::of(header);
        let painter = ui.painter_at(header);
        let middle = header.center().y;
        heading(
            &painter,
            tokens,
            pos2(columns.destination_left, middle),
            Align2::LEFT_CENTER,
            pt_br::NETWORK_COLUMN_DESTINATION,
        );
        heading(
            &painter,
            tokens,
            pos2(columns.answer_right, middle),
            Align2::RIGHT_CENTER,
            pt_br::NETWORK_COLUMN_ANSWER,
        );
        heading(
            &painter,
            tokens,
            pos2(columns.received_right, middle),
            Align2::RIGHT_CENTER,
            pt_br::NETWORK_RECEIVED,
        );
        heading(
            &painter,
            tokens,
            pos2(columns.sent_right, middle),
            Align2::RIGHT_CENTER,
            pt_br::NETWORK_SENT,
        );
        ScrollArea::vertical()
            .id_salt("network_connections")
            .max_height((height - PANEL_CHROME - HEADER_HEIGHT).max(CONNECTION_ROW_HEIGHT))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for connection in &program.connections {
                    connection_row(ui, tokens, connection);
                }
            });
    });
}

/// Where each column of the list of connections is.
struct ConnectionColumns {
    destination_left: f32,
    answer_right: f32,
    received_right: f32,
    sent_right: f32,
}

impl ConnectionColumns {
    fn of(row: Rect) -> Self {
        let sent_right = row.right() - SPACE_2;
        let received_right = sent_right - FIGURE_WIDTH;
        Self {
            destination_left: row.left() + SPACE_2,
            answer_right: received_right - FIGURE_WIDTH,
            received_right,
            sent_right,
        }
    }
}

fn connection_row(ui: &mut Ui, tokens: &Tokens, connection: &Connection) {
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), CONNECTION_ROW_HEIGHT),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    row_background(&painter, tokens, rect, false, response.hovered());
    let columns = ConnectionColumns::of(rect);
    let destination_area = Rect::from_min_max(
        pos2(columns.destination_left, rect.top()),
        pos2(columns.answer_right - ANSWER_WIDTH, rect.bottom()),
    );
    let clipped = painter.with_clip_rect(destination_area);
    clipped.text(
        pos2(columns.destination_left, rect.top() + SPACE_1 + 1.0),
        Align2::LEFT_TOP,
        connection.remote.to_string(),
        FontId::new(FONT_SIZE_DATA - 1.0, FontFamily::Monospace),
        tokens.color_text_primary,
    );
    let kind = match connection.service() {
        Some(service) => format!("{service} · {}", connection.transport.label()),
        None => connection.transport.label().to_owned(),
    };
    let (note, note_color) = if connection.retransmitted.is_zero() {
        (kind, tokens.color_text_muted)
    } else {
        let resent = pt_br::network_resent(connection.retransmitted);
        (format!("{kind} · {resent}"), tokens.color_warn)
    };
    clipped.text(
        pos2(columns.destination_left, rect.bottom() - SPACE_1),
        Align2::LEFT_BOTTOM,
        note,
        FontId::new(FONT_SIZE_SMALL - 1.0, FontFamily::Monospace),
        note_color,
    );
    let middle = rect.center().y;
    let answer = connection
        .round_trip
        .map_or_else(|| pt_br::NETWORK_NOT_MEASURED.to_owned(), pt_br::millis);
    figure(
        &painter,
        pos2(columns.answer_right, middle),
        &answer,
        tokens.color_text_secondary,
    );
    figure(
        &painter,
        pos2(columns.received_right, middle),
        &connection.received.to_string(),
        tokens.color_accent,
    );
    figure(
        &painter,
        pos2(columns.sent_right, middle),
        &connection.sent.to_string(),
        tokens.color_ok,
    );
}

/// The line above a row and what tells the chosen or pointed row apart.
fn row_background(
    painter: &Painter,
    tokens: &Tokens,
    rect: Rect,
    is_chosen: bool,
    is_pointed: bool,
) {
    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, tokens.color_line),
    );
    if is_chosen {
        let fill = tokens.color_accent.gamma_multiply(SELECTED_FILL_OPACITY);
        painter.rect_filled(rect, 0.0, fill);
        painter.vline(
            rect.left() + MARKER_WIDTH / 2.0,
            rect.y_range(),
            Stroke::new(MARKER_WIDTH, tokens.color_accent),
        );
    } else if is_pointed {
        let fill = tokens.color_accent.gamma_multiply(HOVER_FILL_OPACITY);
        painter.rect_filled(rect, 0.0, fill);
    }
}

/// The name of a column.
fn heading(painter: &Painter, tokens: &Tokens, position: Pos2, anchor: Align2, text: &str) {
    painter.text(
        position,
        anchor,
        text.to_uppercase(),
        FontId::new(FONT_SIZE_SMALL - 1.0, FontFamily::Monospace),
        tokens.color_text_muted,
    );
}

/// A figure, aligned at its right end.
fn figure(painter: &Painter, right_middle: Pos2, text: &str, color: Color32) {
    painter.text(
        right_middle,
        Align2::RIGHT_CENTER,
        text,
        FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace),
        color,
    );
}
