//! The two typefaces of the interface, embedded in the binary.
//!
//! JetBrains Mono carries all data. Rajdhani is the display face, used for
//! titles and navigation. Both are licensed under the SIL Open Font License;
//! the license texts are in `assets/fonts/`.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const DATA_FONT: &str = "jetbrains_mono_regular";
const DISPLAY_FONT: &str = "rajdhani_semibold";
const DISPLAY_BOLD_FONT: &str = "rajdhani_bold";

const DISPLAY_FAMILY: &str = "display";
const DISPLAY_BOLD_FAMILY: &str = "display_bold";

macro_rules! font_file {
    ($name:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/fonts/",
            $name
        ))
    };
}

/// Family for navigation and labels set in the display face.
pub(crate) fn display() -> FontFamily {
    FontFamily::Name(DISPLAY_FAMILY.into())
}

/// Family for titles and the brand, in the bold display face.
pub(crate) fn display_bold() -> FontFamily {
    FontFamily::Name(DISPLAY_BOLD_FAMILY.into())
}

/// Registers the embedded fonts. JetBrains Mono becomes the default for both
/// proportional and monospace text, with egui's own fonts as fallback.
pub(crate) fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut embed = |name: &str, bytes: &'static [u8]| {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    embed(DATA_FONT, font_file!("JetBrainsMono-Regular.ttf"));
    embed(DISPLAY_FONT, font_file!("Rajdhani-SemiBold.ttf"));
    embed(DISPLAY_BOLD_FONT, font_file!("Rajdhani-Bold.ttf"));

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, DATA_FONT.to_owned());
    }
    fonts.families.insert(
        display(),
        vec![DISPLAY_FONT.to_owned(), DATA_FONT.to_owned()],
    );
    fonts.families.insert(
        display_bold(),
        vec![DISPLAY_BOLD_FONT.to_owned(), DATA_FONT.to_owned()],
    );
    ctx.set_fonts(fonts);
}
