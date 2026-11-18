//! A bar made of small cells, lit from the left in proportion to a value.

use egui::{Color32, Painter, Rect, Sense, Ui, pos2, vec2};

use crate::theme::Tokens;

const BAR_HEIGHT: f32 = 8.0;
const CELL_GAP: f32 = 2.0;

/// Shows a bar as wide as the space available, with `cells` cells.
pub(crate) fn segmented_bar(ui: &mut Ui, tokens: &Tokens, ratio: f32, cells: usize, lit: Color32) {
    let size = vec2(ui.available_width(), BAR_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    paint(ui.painter(), rect, ratio, cells, lit, tokens.color_line);
}

/// Paints a segmented bar inside `rect`. `ratio` is clamped to 0.0..=1.0.
pub(crate) fn paint(
    painter: &Painter,
    rect: Rect,
    ratio: f32,
    cells: usize,
    lit: Color32,
    unlit: Color32,
) {
    if cells == 0 {
        return;
    }
    let cell_count = cells as f32;
    let cell_width = (rect.width() - CELL_GAP * (cell_count - 1.0)) / cell_count;
    let lit_cells = (ratio.clamp(0.0, 1.0) * cell_count).round() as usize;
    for index in 0..cells {
        let left = rect.left() + index as f32 * (cell_width + CELL_GAP);
        let cell = Rect::from_min_size(pos2(left, rect.top()), vec2(cell_width, rect.height()));
        painter.rect_filled(cell, 0.0, if index < lit_cells { lit } else { unlit });
    }
}
