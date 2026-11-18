//! A line over a grid, like an oscilloscope trace.

use egui::{Pos2, Sense, Stroke, Ui, pos2, vec2};

use crate::effects::glow;
use crate::theme::Tokens;

const VERTICAL_DIVISIONS: usize = 12;
const HORIZONTAL_DIVISIONS: usize = 4;
const TRACE_WIDTH: f32 = 1.5;

/// Shows `values`, each from 0.0 to 1.0, oldest at the left, as a trace
/// `height` points tall.
pub(crate) fn line_plot(
    ui: &mut Ui,
    tokens: &Tokens,
    values: &[f32],
    height: f32,
    glow_strength: f32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let painter = ui.painter_at(rect);
    let grid = Stroke::new(1.0, tokens.color_line);
    for division in 0..=VERTICAL_DIVISIONS {
        let x = rect.left() + rect.width() * division as f32 / VERTICAL_DIVISIONS as f32;
        painter.vline(x, rect.y_range(), grid);
    }
    for division in 0..=HORIZONTAL_DIVISIONS {
        let y = rect.top() + rect.height() * division as f32 / HORIZONTAL_DIVISIONS as f32;
        painter.hline(rect.x_range(), y, grid);
    }
    if values.len() < 2 {
        return;
    }

    let last = (values.len() - 1) as f32;
    let trace: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            pos2(
                rect.left() + rect.width() * index as f32 / last,
                rect.bottom() - rect.height() * value.clamp(0.0, 1.0),
            )
        })
        .collect();
    glow::path(
        &painter,
        &trace,
        tokens.color_accent,
        TRACE_WIDTH,
        glow_strength,
    );
}
