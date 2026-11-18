//! One line of a fact sheet: a muted name followed by its value.

use egui::{RichText, Ui};

use crate::theme::{FONT_SIZE_SMALL, SPACE_2, Tokens};

/// Shows `label` and `value` on one line, in the small data size.
pub(crate) fn key_value(ui: &mut Ui, tokens: &Tokens, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        ui.label(
            RichText::new(label)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
        ui.label(
            RichText::new(value)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_primary),
        );
    });
}
