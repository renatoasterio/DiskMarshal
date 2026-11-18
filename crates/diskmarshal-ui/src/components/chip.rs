//! A small toggle for choosing one option among a few.

use egui::{
    Color32, FontFamily, FontId, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, pos2, vec2,
};

use crate::theme::{SPACE_2, Tokens};

const CHIP_HEIGHT: f32 = 24.0;
const CHIP_FONT_SIZE: f32 = 12.0;
const SWATCH_SIZE: f32 = 8.0;
/// Opacity of the accent fill behind a selected chip.
const SELECTED_FILL_OPACITY: f32 = 0.1;

/// Shows a chip. `swatch` adds a small color sample before the label.
pub(crate) fn chip(
    ui: &mut Ui,
    tokens: &Tokens,
    label: &str,
    is_selected: bool,
    swatch: Option<Color32>,
) -> Response {
    let font = FontId::new(CHIP_FONT_SIZE, FontFamily::Proportional);
    let swatch_width = if swatch.is_some() {
        SWATCH_SIZE + SPACE_2
    } else {
        0.0
    };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let size = vec2(galley.size().x + swatch_width + SPACE_2 * 2.5, CHIP_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let is_lit = is_selected || response.hovered() || response.has_focus();
    let border = if is_lit {
        tokens.color_accent
    } else {
        tokens.color_line_strong
    };
    let text_color = if is_selected {
        tokens.color_accent
    } else {
        tokens.color_text_secondary
    };
    let painter = ui.painter();
    if is_selected {
        painter.rect_filled(
            rect,
            0.0,
            tokens.color_accent.gamma_multiply(SELECTED_FILL_OPACITY),
        );
    }
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);

    let mut text_left = rect.left() + SPACE_2 * 1.25;
    if let Some(color) = swatch {
        let center = pos2(text_left + SWATCH_SIZE / 2.0, rect.center().y);
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::splat(SWATCH_SIZE)),
            0.0,
            color,
        );
        text_left += swatch_width;
    }
    let text_top = rect.center().y - galley.size().y / 2.0;
    painter.galley(pos2(text_left, text_top), galley, text_color);

    response.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, is_selected, label));
    response
}
