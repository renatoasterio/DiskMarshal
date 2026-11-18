//! A line over a grid, like an oscilloscope trace.

use egui::{Color32, Pos2, Sense, Stroke, Ui, pos2, vec2};

use crate::effects::glow;
use crate::theme::Tokens;

const VERTICAL_DIVISIONS: usize = 12;
const HORIZONTAL_DIVISIONS: usize = 4;
const TRACE_WIDTH: f32 = 1.5;

/// One line of a plot.
pub(crate) struct Trace<'a> {
    /// The readings, each from 0.0 to 1.0, oldest first.
    pub values: &'a [f32],
    pub color: Color32,
}

/// Shows each trace over the same grid, oldest reading at the left, in a
/// plot `height` points tall. The first trace is drawn first, so the last
/// one stays on top.
pub(crate) fn line_plot(
    ui: &mut Ui,
    tokens: &Tokens,
    traces: &[Trace<'_>],
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
    for trace in traces.iter().filter(|trace| trace.values.len() >= 2) {
        let last = (trace.values.len() - 1) as f32;
        let points: Vec<Pos2> = trace
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                pos2(
                    rect.left() + rect.width() * index as f32 / last,
                    rect.bottom() - rect.height() * value.clamp(0.0, 1.0),
                )
            })
            .collect();
        glow::path(&painter, &points, trace.color, TRACE_WIDTH, glow_strength);
    }
}
