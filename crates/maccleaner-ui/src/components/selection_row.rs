//! One line of a list the user picks items from.

use egui::{
    Align2, Color32, FontFamily, FontId, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, pos2, vec2,
};

use crate::components::segmented_bar;
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_2, SPACE_3, Tokens};

const ROW_HEIGHT: f32 = 38.0;
const CHECK_SIZE: f32 = 14.0;
const BADGE_WIDTH: f32 = 76.0;
const BADGE_HEIGHT: f32 = 18.0;
const BAR_WIDTH: f32 = 150.0;
const BAR_HEIGHT: f32 = 8.0;
const BAR_CELLS: usize = 16;
const TRAILING_WIDTH: f32 = 84.0;
const HOVER_FILL_OPACITY: f32 = 0.05;
const DISABLED_OPACITY: f32 = 0.45;

/// What a row shows.
pub(crate) struct SelectionRow<'a> {
    pub is_checked: bool,
    /// A disabled row is dimmed and cannot be toggled.
    pub is_enabled: bool,
    pub title: &'a str,
    /// A second, quieter line under the title.
    pub detail: &'a str,
    /// A short label in a colored outline, such as the risk.
    pub badge: Option<(&'a str, Color32)>,
    /// How full the bar is, from 0.0 to 1.0. `None` leaves the bar out.
    pub ratio: Option<f32>,
    /// Right-aligned text at the end of the row, such as the size.
    pub trailing: &'a str,
}

/// Shows the row. The response is clicked when the user toggles it.
pub(crate) fn selection_row(ui: &mut Ui, tokens: &Tokens, row: &SelectionRow<'_>) -> Response {
    let sense = if row.is_enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), sense);
    let painter = ui.painter_at(rect);
    let dim = |color: Color32| {
        if row.is_enabled {
            color
        } else {
            color.gamma_multiply(DISABLED_OPACITY)
        }
    };

    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, tokens.color_line),
    );
    if row.is_enabled && (response.hovered() || response.has_focus()) {
        painter.rect_filled(
            rect,
            0.0,
            tokens.color_accent.gamma_multiply(HOVER_FILL_OPACITY),
        );
    }

    let check = Rect::from_center_size(
        pos2(rect.left() + SPACE_2 + CHECK_SIZE / 2.0, rect.center().y),
        Vec2::splat(CHECK_SIZE),
    );
    if row.is_checked {
        painter.rect_filled(check, 0.0, dim(tokens.color_accent));
    } else {
        painter.rect_stroke(
            check,
            0.0,
            Stroke::new(1.0, dim(tokens.color_line_strong)),
            StrokeKind::Inside,
        );
    }

    let mut right = rect.right() - SPACE_2;
    painter.text(
        pos2(right, rect.center().y),
        Align2::RIGHT_CENTER,
        row.trailing,
        FontId::new(FONT_SIZE_DATA, FontFamily::Monospace),
        dim(tokens.color_text_primary),
    );
    right -= TRAILING_WIDTH;
    if let Some(ratio) = row.ratio {
        let bar = Rect::from_min_size(
            pos2(right - BAR_WIDTH, rect.center().y - BAR_HEIGHT / 2.0),
            vec2(BAR_WIDTH, BAR_HEIGHT),
        );
        segmented_bar::paint(
            &painter,
            bar,
            ratio,
            BAR_CELLS,
            dim(tokens.color_accent),
            tokens.color_line,
        );
        right -= BAR_WIDTH + SPACE_3;
    }
    if let Some((label, color)) = row.badge {
        let badge = Rect::from_min_size(
            pos2(right - BADGE_WIDTH, rect.center().y - BADGE_HEIGHT / 2.0),
            vec2(BADGE_WIDTH, BADGE_HEIGHT),
        );
        painter.rect_stroke(badge, 0.0, Stroke::new(1.0, dim(color)), StrokeKind::Inside);
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            label.to_uppercase(),
            FontId::new(FONT_SIZE_SMALL - 1.0, FontFamily::Monospace),
            dim(color),
        );
        right -= BADGE_WIDTH + SPACE_3;
    }

    let text_left = check.right() + SPACE_3;
    let text_area = Rect::from_min_max(pos2(text_left, rect.top()), pos2(right, rect.bottom()));
    let text = painter.with_clip_rect(text_area);
    let title_y = if row.detail.is_empty() {
        rect.center().y
    } else {
        rect.center().y - 8.0
    };
    text.text(
        pos2(text_left, title_y),
        Align2::LEFT_CENTER,
        row.title,
        FontId::new(FONT_SIZE_DATA, FontFamily::Proportional),
        dim(tokens.color_text_primary),
    );
    if !row.detail.is_empty() {
        text.text(
            pos2(text_left, rect.center().y + 9.0),
            Align2::LEFT_CENTER,
            row.detail,
            FontId::new(FONT_SIZE_SMALL, FontFamily::Proportional),
            dim(tokens.color_text_muted),
        );
    }

    response.widget_info(|| {
        WidgetInfo::selected(Role::CheckBox, row.is_enabled, row.is_checked, row.title)
    });
    response
}
