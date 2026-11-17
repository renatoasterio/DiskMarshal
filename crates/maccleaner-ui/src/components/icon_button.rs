//! A square button that shows only an icon.

use egui::{Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo};

use crate::components::Icon;
use crate::theme::Tokens;

const BUTTON_SIZE: f32 = 24.0;
const ICON_SIZE: f32 = 14.0;

/// Shows an icon button. `label` names it for screen readers and the tooltip.
pub(crate) fn icon_button(
    ui: &mut Ui,
    tokens: &Tokens,
    icon: Icon,
    is_active: bool,
    label: &str,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON_SIZE), Sense::click());
    let is_lit = is_active || response.hovered() || response.has_focus();
    let (border, color) = if is_lit {
        (tokens.color_accent, tokens.color_accent)
    } else {
        (tokens.color_line_strong, tokens.color_text_secondary)
    };
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    icon.paint(
        ui.painter(),
        egui::Rect::from_center_size(rect.center(), Vec2::splat(ICON_SIZE)),
        color,
    );

    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    response.on_hover_text(label)
}
