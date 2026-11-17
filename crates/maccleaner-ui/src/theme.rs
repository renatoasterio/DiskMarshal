//! Visual identity. The single source for colors, sizes, durations and fonts.
//!
//! Token names mirror the CSS variables of `design/prototype/index.html`:
//! `--color-surface-panel` is `Tokens::color_surface_panel`, and so on.

mod fonts;
mod palette;
mod style;
mod tokens;

pub use palette::Palette;
pub use tokens::WINDOW_SIZE;

pub(crate) use fonts::{display, display_bold, install as install_fonts};
pub(crate) use style::apply as apply_style;
pub(crate) use tokens::{
    CORNER_SIZE, FONT_SIZE_BRAND, FONT_SIZE_DATA, FONT_SIZE_RAIL, FONT_SIZE_SMALL, FONT_SIZE_TITLE,
    GRID_CELL, MOTION_BOOT_HOLD_SECONDS, MOTION_BOOT_SECONDS, MOTION_ENTER_SECONDS,
    MOTION_SCAN_BAND_SECONDS, MOTION_TYPE_SECONDS_PER_CHARACTER, RAIL_ITEM_HEIGHT, RAIL_WIDTH,
    SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_5, STATUS_BAR_HEIGHT, Tokens,
};
