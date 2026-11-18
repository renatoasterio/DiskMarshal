//! The panel that opens from the sliders icon: palette, motion and glow.

use egui::{RichText, Slider, Ui};

use crate::animation::MotionPreference;
use crate::components::{HudPanel, chip};
use crate::i18n::pt_br;
use crate::theme::{FONT_SIZE_SMALL, Palette, SPACE_1, SPACE_2, SPACE_3, Tokens};

/// Width of the panel.
pub(crate) const APPEARANCE_PANEL_WIDTH: f32 = 380.0;
const GLOW_RANGE: std::ops::RangeInclusive<f32> = 0.0..=2.0;
const GLOW_STEP: f64 = 0.25;
const LABEL_LETTER_SPACING: f32 = 1.3;

/// How the interface looks and moves, as chosen by the user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AppearanceSettings {
    pub palette: Palette,
    pub motion: MotionPreference,
    /// Glow strength: 0.0 is off, 1.0 is the default, 2.0 is the strongest.
    pub glow_strength: f32,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            palette: Palette::default(),
            motion: MotionPreference::default(),
            glow_strength: 1.0,
        }
    }
}

/// What the user did in the panel during this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AppearanceResponse {
    /// The palette changed, so the theme must be applied again.
    pub palette_changed: bool,
    /// The user asked to see the start-up sequence again.
    pub replay_boot: bool,
}

/// Shows the panel and applies the user's choices to `settings`.
pub(crate) fn appearance_panel(
    ui: &mut Ui,
    tokens: &Tokens,
    settings: &mut AppearanceSettings,
) -> AppearanceResponse {
    let mut response = AppearanceResponse::default();
    HudPanel::new(pt_br::APPEARANCE)
        .meta(pt_br::APPEARANCE_CLOSE_HINT)
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_1 + 2.0;

            section_label(ui, tokens, pt_br::APPEARANCE_PALETTE);
            ui.horizontal(|ui| {
                for palette in Palette::ALL {
                    let swatch = Some(palette.tokens().color_accent);
                    let is_selected = settings.palette == palette;
                    if chip(ui, tokens, palette.label(), is_selected, swatch).clicked()
                        && !is_selected
                    {
                        settings.palette = palette;
                        response.palette_changed = true;
                    }
                }
            });
            ui.add_space(SPACE_2);

            section_label(ui, tokens, pt_br::APPEARANCE_MOTION);
            ui.horizontal(|ui| {
                for motion in MotionPreference::ALL {
                    if chip(ui, tokens, motion.label(), settings.motion == motion, None).clicked() {
                        settings.motion = motion;
                    }
                }
            });
            ui.add_space(SPACE_2);

            section_label(ui, tokens, pt_br::APPEARANCE_GLOW);
            ui.spacing_mut().slider_width = ui.available_width();
            ui.add(
                Slider::new(&mut settings.glow_strength, GLOW_RANGE)
                    .step_by(GLOW_STEP)
                    .show_value(false),
            );
            ui.add_space(SPACE_3);

            if chip(ui, tokens, pt_br::APPEARANCE_REPLAY_BOOT, false, None).clicked() {
                response.replay_boot = true;
            }
        });
    response
}

fn section_label(ui: &mut Ui, tokens: &Tokens, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(FONT_SIZE_SMALL)
            .extra_letter_spacing(LABEL_LETTER_SPACING)
            .color(tokens.color_text_muted),
    );
}
