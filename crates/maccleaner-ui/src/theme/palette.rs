//! The three palettes the user can choose from.

use egui::Color32;

use crate::i18n::pt_br;
use crate::theme::Tokens;

/// A color scheme of the interface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Palette {
    /// Cyan on near-black. The default.
    #[default]
    Agency,
    /// Green on near-black, like a classic terminal.
    Terminal,
    /// Amber on warm black.
    Tactical,
}

impl Palette {
    /// Every palette, in the order they are offered.
    pub const ALL: [Self; 3] = [Self::Agency, Self::Terminal, Self::Tactical];

    /// Name shown to the user.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Agency => pt_br::PALETTE_AGENCY,
            Self::Terminal => pt_br::PALETTE_TERMINAL,
            Self::Tactical => pt_br::PALETTE_TACTICAL,
        }
    }

    pub(crate) const fn tokens(self) -> Tokens {
        match self {
            Self::Agency => AGENCY,
            Self::Terminal => TERMINAL,
            Self::Tactical => TACTICAL,
        }
    }
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const AGENCY: Tokens = Tokens {
    color_surface_void: rgb(0x04_07_0a),
    color_surface_window: rgb(0x07_0c_10),
    color_surface_panel: rgb(0x0a_13_19),
    color_surface_raised: rgb(0x0f_1c_24),
    color_line: rgb(0x17_30_39),
    color_line_strong: rgb(0x24_50_5e),
    color_text_primary: rgb(0xd5_ee_f3),
    color_text_secondary: rgb(0x84_a7_b1),
    color_text_muted: rgb(0x56_72_7b),
    color_accent: rgb(0x2f_e6_ff),
    color_accent_contrast: rgb(0x02_13_17),
    color_ok: rgb(0x45_f0_a0),
    color_warn: rgb(0xff_b0_20),
    color_alert: rgb(0xff_4d_5a),
};

const TERMINAL: Tokens = Tokens {
    color_surface_void: rgb(0x03_07_03),
    color_surface_window: rgb(0x05_0b_06),
    color_surface_panel: rgb(0x08_13_0a),
    color_surface_raised: rgb(0x0d_1d_11),
    color_line: rgb(0x15_30_19),
    color_line_strong: rgb(0x22_59_2d),
    color_text_primary: rgb(0xd3_f5_d9),
    color_text_secondary: rgb(0x84_b0_8e),
    color_text_muted: rgb(0x55_77_5d),
    color_accent: rgb(0x3d_ff_7a),
    color_accent_contrast: rgb(0x02_15_05),
    color_ok: rgb(0xc8_ff_70),
    ..AGENCY
};

const TACTICAL: Tokens = Tokens {
    color_surface_void: rgb(0x08_06_03),
    color_surface_window: rgb(0x0c_09_05),
    color_surface_panel: rgb(0x14_10_08),
    color_surface_raised: rgb(0x1e_18_0c),
    color_line: rgb(0x33_29_0f),
    color_line_strong: rgb(0x5c_4a_1a),
    color_text_primary: rgb(0xf5_ea_d3),
    color_text_secondary: rgb(0xb3_a2_7c),
    color_text_muted: rgb(0x7a_6d_50),
    color_accent: rgb(0xff_b0_00),
    color_accent_contrast: rgb(0x1a_10_00),
    color_warn: rgb(0xff_7a_1a),
    ..AGENCY
};
