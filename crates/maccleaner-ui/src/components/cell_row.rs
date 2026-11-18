//! A row of cells side by side, each with its own width.

use egui::{Align, Layout, Rect, Ui, UiBuilder, pos2, vec2};

use crate::theme::SPACE_3;

/// How wide one cell of a row is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CellWidth {
    /// Exactly this many points.
    Fixed(f32),
    /// A share of what the fixed cells leave free, relative to the other shares.
    Share(f32),
}

/// Works out the width of every cell for a row `total` points wide.
fn resolve(widths: &[CellWidth], total: f32, gap: f32) -> Vec<f32> {
    let gaps = gap * widths.len().saturating_sub(1) as f32;
    let (fixed, shares) = widths
        .iter()
        .fold((0.0, 0.0), |(fixed, shares), width| match width {
            CellWidth::Fixed(points) => (fixed + points, shares),
            CellWidth::Share(share) => (fixed, shares + share),
        });
    let free = (total - gaps - fixed).max(0.0);
    widths
        .iter()
        .map(|width| match width {
            CellWidth::Fixed(points) => *points,
            CellWidth::Share(share) if shares > 0.0 => free * share / shares,
            CellWidth::Share(_) => 0.0,
        })
        .collect()
}

/// Lays cells side by side, aligned at the top, and calls `add_cell` with the
/// position of each one. The row is as tall as its tallest cell.
pub(crate) fn cell_row(
    ui: &mut Ui,
    widths: &[CellWidth],
    mut add_cell: impl FnMut(usize, &mut Ui),
) {
    let total = ui.available_width();
    let top_left = ui.cursor().min;
    let bottom_limit = ui.max_rect().bottom();
    let mut left = top_left.x;
    let mut height: f32 = 0.0;
    for (index, width) in resolve(widths, total, SPACE_3).into_iter().enumerate() {
        let rect = Rect::from_min_max(pos2(left, top_left.y), pos2(left + width, bottom_limit));
        let mut cell = ui.new_child(
            UiBuilder::new()
                .id_salt(("cell_row", index))
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        cell.set_width(width);
        add_cell(index, &mut cell);
        height = height.max(cell.min_size().y);
        left += width + SPACE_3;
    }
    ui.advance_cursor_after_rect(Rect::from_min_size(top_left, vec2(total, height)));
}

#[cfg(test)]
mod tests {
    use super::{CellWidth, resolve};

    #[test]
    fn fixed_cells_keep_their_width_and_shares_split_the_rest() {
        let widths = [
            CellWidth::Fixed(100.0),
            CellWidth::Share(1.0),
            CellWidth::Share(3.0),
        ];
        // 520 wide, two gaps of 10: 400 left for the shares.
        assert_eq!(resolve(&widths, 520.0, 10.0), vec![100.0, 100.0, 300.0]);
    }

    #[test]
    fn a_row_too_narrow_for_its_fixed_cells_gives_shares_nothing() {
        let widths = [CellWidth::Fixed(300.0), CellWidth::Share(1.0)];
        assert_eq!(resolve(&widths, 200.0, 10.0), vec![300.0, 0.0]);
    }
}
