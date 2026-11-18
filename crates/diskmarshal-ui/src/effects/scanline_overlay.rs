//! Scanlines over the whole window, with an optional band that drifts down.

use egui::{Color32, Mesh, Painter, Rect, Shape, Stroke, pos2};

use crate::theme::Tokens;

const LINE_SPACING: f32 = 3.0;
const LINE_OPACITY: f32 = 0.03;
const BAND_HEIGHT: f32 = 90.0;
const BAND_OPACITY: f32 = 0.05;

/// Paints the scanlines over `rect`.
///
/// `band_progress`, from 0.0 to 1.0, places the drifting band between the top
/// and the bottom of `rect`. `None` leaves the band out.
pub(crate) fn paint(painter: &Painter, rect: Rect, tokens: &Tokens, band_progress: Option<f32>) {
    let line = Stroke::new(1.0, tokens.color_accent.gamma_multiply(LINE_OPACITY));
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.hline(rect.x_range(), y, line);
        y += LINE_SPACING;
    }

    let Some(progress) = band_progress else {
        return;
    };
    let center = rect.top() - BAND_HEIGHT / 2.0 + progress * (rect.height() + BAND_HEIGHT);
    let peak = tokens.color_accent.gamma_multiply(BAND_OPACITY);
    let rows = [
        (center - BAND_HEIGHT / 2.0, Color32::TRANSPARENT),
        (center, peak),
        (center + BAND_HEIGHT / 2.0, Color32::TRANSPARENT),
    ];
    let mut mesh = Mesh::default();
    for (row_y, color) in rows {
        mesh.colored_vertex(pos2(rect.left(), row_y), color);
        mesh.colored_vertex(pos2(rect.right(), row_y), color);
    }
    for first in [0, 2] {
        mesh.add_triangle(first, first + 1, first + 2);
        mesh.add_triangle(first + 1, first + 2, first + 3);
    }
    painter.add(Shape::mesh(mesh));
}
