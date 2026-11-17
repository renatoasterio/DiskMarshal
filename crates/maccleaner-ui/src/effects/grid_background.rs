//! Faint grid behind the working area.

use egui::{Painter, Rect, Stroke};

use crate::theme::{GRID_CELL, Tokens};

/// Opacity of the grid lines over the window surface.
const GRID_OPACITY: f32 = 0.28;

/// Paints the grid over `rect`.
pub(crate) fn paint(painter: &Painter, rect: Rect, tokens: &Tokens) {
    let stroke = Stroke::new(1.0, tokens.color_line.gamma_multiply(GRID_OPACITY));
    let mut x = rect.left();
    while x <= rect.right() {
        painter.vline(x, rect.y_range(), stroke);
        x += GRID_CELL;
    }
    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.hline(rect.x_range(), y, stroke);
        y += GRID_CELL;
    }
}
