//! The bar at the bottom of a list: what is selected and the one action.

use egui::{Align, Color32, Layout, RichText, Stroke, Ui};

use crate::components::{ButtonKind, hud_button};
use crate::theme::{SPACE_2, Tokens};

/// Shows the footer and returns `true` when its button was clicked.
///
/// `summary` says what is selected. `notice`, when present, replaces it in
/// the alert color, to tell the user why the action did not happen.
pub(crate) fn action_footer(
    ui: &mut Ui,
    tokens: &Tokens,
    summary: &str,
    notice: Option<&str>,
    action: &str,
) -> bool {
    let top = ui.cursor().top();
    ui.painter().hline(
        ui.max_rect().x_range(),
        top,
        Stroke::new(1.0, tokens.color_line_strong),
    );
    ui.add_space(SPACE_2);
    let mut clicked = false;
    ui.horizontal(|ui| {
        let (text, color): (&str, Color32) = match notice {
            Some(notice) => (notice, tokens.color_alert),
            None => (summary, tokens.color_text_secondary),
        };
        ui.label(RichText::new(text).color(color));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            clicked = hud_button(ui, tokens, action, ButtonKind::Primary).clicked();
        });
    });
    clicked
}
