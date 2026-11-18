//! A figure read at a glance: a large value followed by its unit.

use egui::{FontId, RichText, Ui};

use crate::theme::{FONT_SIZE_SMALL, SPACE_1, Tokens, display_bold};

const READOUT_FONT_SIZE: f32 = 28.0;

/// Shows `value` large and `unit` small after it, on one line.
pub(crate) fn readout(ui: &mut Ui, tokens: &Tokens, value: &str, unit: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_1 + 2.0;
        ui.label(
            RichText::new(value)
                .font(FontId::new(READOUT_FONT_SIZE, display_bold()))
                .color(tokens.color_text_primary),
        );
        ui.label(
            RichText::new(unit)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_secondary),
        );
    });
}
