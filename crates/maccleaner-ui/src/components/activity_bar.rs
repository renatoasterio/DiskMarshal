//! A bar that shows work is under way when its total is not known.

use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::theme::Tokens;

const BAR_HEIGHT: f32 = 8.0;
const CELLS: usize = 30;
const CELL_GAP: f32 = 2.0;
/// Share of the bar that is lit at a time.
const LIT_WINDOW: f32 = 0.2;
/// Time the lit window takes to cross the bar.
const CYCLE_SECONDS: f64 = 1.6;

/// Shows the bar. With `is_moving` the lit window travels from left to
/// right; without it the window stays in the middle.
pub(crate) fn activity_bar(ui: &mut Ui, tokens: &Tokens, now: f64, is_moving: bool) {
    let position = if is_moving {
        ((now % CYCLE_SECONDS) / CYCLE_SECONDS) as f32
    } else {
        0.4
    };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), BAR_HEIGHT), Sense::hover());
    let cell_count = CELLS as f32;
    let cell_width = (rect.width() - CELL_GAP * (cell_count - 1.0)) / cell_count;
    for index in 0..CELLS {
        let distance = (index as f32 / cell_count - position).rem_euclid(1.0);
        let left = rect.left() + index as f32 * (cell_width + CELL_GAP);
        let cell = Rect::from_min_size(pos2(left, rect.top()), vec2(cell_width, rect.height()));
        let color = if distance < LIT_WINDOW {
            tokens.color_accent
        } else {
            tokens.color_line
        };
        ui.painter().rect_filled(cell, 0.0, color);
    }
}
