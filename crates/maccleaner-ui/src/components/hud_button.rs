//! The action button: a rectangle with two corners cut off.

use egui::{
    Align2, FontId, Pos2, Response, Role, Sense, Shape, Stroke, Ui, WidgetInfo, pos2, vec2,
};

use crate::effects::glow::{self, TextStyle};
use crate::theme::{SPACE_4, Tokens, display};

const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_FONT_SIZE: f32 = 13.0;
const BUTTON_LETTER_SPACING: f32 = 1.8;
/// Length of the cut on the top-left and bottom-right corners.
const CORNER_CUT: f32 = 8.0;
const HOVER_FILL_OPACITY: f32 = 0.14;

/// How much a button stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonKind {
    /// The one main action of a screen: filled with the accent color.
    Primary,
    /// A secondary action: accent outline.
    Outline,
    /// A way out, such as cancel: muted outline.
    Quiet,
}

/// Shows a button and returns its response.
pub(crate) fn hud_button(ui: &mut Ui, tokens: &Tokens, label: &str, kind: ButtonKind) -> Response {
    let text = label.to_uppercase();
    let mut style = TextStyle {
        font: FontId::new(BUTTON_FONT_SIZE, display()),
        color: tokens.color_text_primary,
        letter_spacing: BUTTON_LETTER_SPACING,
        glow: 0.0,
    };
    let text_width = glow::size(ui.painter(), &text, &style).x;
    let (rect, response) = ui.allocate_exact_size(
        vec2(text_width + SPACE_4 * 2.0, BUTTON_HEIGHT),
        Sense::click(),
    );
    let is_lit = response.hovered() || response.has_focus();

    let (fill, border, text_color) = match kind {
        ButtonKind::Primary => {
            let fill = if is_lit {
                tokens.color_accent.gamma_multiply(0.85)
            } else {
                tokens.color_accent
            };
            (fill, tokens.color_accent, tokens.color_accent_contrast)
        }
        ButtonKind::Outline => (
            hover_fill(tokens, is_lit),
            tokens.color_accent,
            tokens.color_accent,
        ),
        ButtonKind::Quiet => (
            hover_fill(tokens, is_lit),
            tokens.color_line_strong,
            tokens.color_text_secondary,
        ),
    };
    let (min, max) = (rect.min, rect.max);
    let outline: Vec<Pos2> = vec![
        pos2(min.x + CORNER_CUT, min.y),
        pos2(max.x, min.y),
        pos2(max.x, max.y - CORNER_CUT),
        pos2(max.x - CORNER_CUT, max.y),
        pos2(min.x, max.y),
        pos2(min.x, min.y + CORNER_CUT),
    ];
    ui.painter().add(Shape::convex_polygon(
        outline,
        fill,
        Stroke::new(1.0, border),
    ));
    style.color = text_color;
    glow::text(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        &text,
        &style,
    );

    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    response
}

fn hover_fill(tokens: &Tokens, is_lit: bool) -> egui::Color32 {
    if is_lit {
        tokens.color_accent.gamma_multiply(HOVER_FILL_OPACITY)
    } else {
        egui::Color32::TRANSPARENT
    }
}
