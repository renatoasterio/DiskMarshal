//! Applies the tokens to egui's own widgets.

use egui::{FontFamily, FontId, Stroke, TextStyle, Theme, Visuals};

use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, FONT_SIZE_TITLE, Tokens, display_bold};

/// Makes egui's built-in widgets (sliders, text, selection) follow `tokens`.
pub(crate) fn apply(ctx: &egui::Context, tokens: &Tokens) {
    ctx.set_theme(Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = visuals(tokens);
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(FONT_SIZE_SMALL, FontFamily::Proportional),
            ),
            (
                TextStyle::Body,
                FontId::new(FONT_SIZE_DATA, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(FONT_SIZE_DATA, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(FONT_SIZE_DATA, FontFamily::Monospace),
            ),
            (
                TextStyle::Heading,
                FontId::new(FONT_SIZE_TITLE, display_bold()),
            ),
        ]
        .into();
    });
}

fn visuals(tokens: &Tokens) -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(tokens.color_text_primary);
    visuals.panel_fill = tokens.color_surface_window;
    visuals.window_fill = tokens.color_surface_window;
    visuals.extreme_bg_color = tokens.color_surface_void;
    visuals.selection.bg_fill = tokens.color_accent;
    visuals.selection.stroke = Stroke::new(1.0, tokens.color_accent_contrast);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, tokens.color_line);
    visuals.widgets.inactive.bg_fill = tokens.color_line;
    visuals.widgets.inactive.weak_bg_fill = tokens.color_surface_raised;
    visuals.widgets.hovered.bg_fill = tokens.color_line_strong;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, tokens.color_accent);
    visuals.widgets.active.bg_fill = tokens.color_accent;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, tokens.color_accent);
    visuals
}
