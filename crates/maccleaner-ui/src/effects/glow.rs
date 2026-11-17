//! Glow built from layered translucent strokes, the only kind egui can draw
//! without a shader.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

/// Strongest halo opacity, reached at the top of the glow slider.
const MAX_HALO_OPACITY: f32 = 0.5;
/// Halo opacity per unit of glow strength.
const HALO_OPACITY_PER_STRENGTH: f32 = 0.16;
/// How far the halo copies of a text sit from it, in points.
const HALO_OFFSETS: [Vec2; 4] = [
    Vec2::new(-1.5, 0.0),
    Vec2::new(1.5, 0.0),
    Vec2::new(0.0, -1.5),
    Vec2::new(0.0, 1.5),
];

/// How a piece of text is drawn.
pub(crate) struct TextStyle {
    pub font: FontId,
    pub color: Color32,
    /// Extra space between letters, in points.
    pub letter_spacing: f32,
    /// Glow strength: 0.0 draws plain text.
    pub glow: f32,
}

/// Paints `text` anchored at `position` and returns the rectangle it covers.
pub(crate) fn text(
    painter: &Painter,
    position: Pos2,
    anchor: Align2,
    text: &str,
    style: &TextStyle,
) -> Rect {
    let layout = |color: Color32| {
        let format = TextFormat {
            font_id: style.font.clone(),
            color,
            extra_letter_spacing: style.letter_spacing,
            ..TextFormat::default()
        };
        painter.layout_job(LayoutJob::single_section(text.to_owned(), format))
    };

    let galley = layout(style.color);
    let rect = anchor.anchor_size(position, galley.size());
    if style.glow > 0.0 {
        let halo_color = halo(style.color, style.glow);
        let halo_galley = layout(halo_color);
        for offset in HALO_OFFSETS {
            painter.galley(rect.min + offset, halo_galley.clone(), halo_color);
        }
    }
    painter.galley(rect.min, galley, style.color);
    rect
}

/// Paints a line with a soft halo around it.
pub(crate) fn line(painter: &Painter, points: [Pos2; 2], color: Color32, width: f32, glow: f32) {
    if glow > 0.0 {
        painter.line_segment(
            points,
            Stroke::new(width + 4.0 * glow, halo(color, glow * 0.5)),
        );
        painter.line_segment(points, Stroke::new(width + 2.0 * glow, halo(color, glow)));
    }
    painter.line_segment(points, Stroke::new(width, color));
}

fn halo(color: Color32, strength: f32) -> Color32 {
    color.gamma_multiply((HALO_OPACITY_PER_STRENGTH * strength).min(MAX_HALO_OPACITY))
}
