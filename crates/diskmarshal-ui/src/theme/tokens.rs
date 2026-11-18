//! Design tokens: every color, size and duration the interface uses.

use egui::Color32;

/// Size of the window, and the smallest size it can be resized to.
pub const WINDOW_SIZE: [f32; 2] = [1100.0, 720.0];

// Spacing scale (`--space-1` to `--space-5`).
pub(crate) const SPACE_1: f32 = 4.0;
pub(crate) const SPACE_2: f32 = 8.0;
pub(crate) const SPACE_3: f32 = 12.0;
pub(crate) const SPACE_4: f32 = 16.0;
pub(crate) const SPACE_5: f32 = 24.0;

// Frame of the window.
pub(crate) const STATUS_BAR_HEIGHT: f32 = 36.0;
pub(crate) const RAIL_WIDTH: f32 = 216.0;
pub(crate) const RAIL_ITEM_HEIGHT: f32 = 36.0;
/// Length of each arm of the corner brackets on a panel (`--corner-size`).
pub(crate) const CORNER_SIZE: f32 = 10.0;
/// Distance between the lines of the background grid.
pub(crate) const GRID_CELL: f32 = 40.0;

// Type scale.
pub(crate) const FONT_SIZE_SMALL: f32 = 11.0;
pub(crate) const FONT_SIZE_DATA: f32 = 13.0;
pub(crate) const FONT_SIZE_RAIL: f32 = 14.0;
pub(crate) const FONT_SIZE_BRAND: f32 = 15.0;
pub(crate) const FONT_SIZE_TITLE: f32 = 26.0;

// Motion (`--motion-*`).
pub(crate) const MOTION_BOOT_SECONDS: f64 = 1.6;
/// How long the finished boot sequence stays on screen before it closes.
pub(crate) const MOTION_BOOT_HOLD_SECONDS: f64 = 0.35;
pub(crate) const MOTION_ENTER_SECONDS: f32 = 0.35;
/// Time the faint scan band takes to cross the window from top to bottom.
pub(crate) const MOTION_SCAN_BAND_SECONDS: f64 = 7.0;
pub(crate) const MOTION_TYPE_SECONDS_PER_CHARACTER: f64 = 0.022;

/// The colors of one palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tokens {
    pub color_surface_void: Color32,
    pub color_surface_window: Color32,
    pub color_surface_panel: Color32,
    pub color_surface_raised: Color32,
    pub color_line: Color32,
    pub color_line_strong: Color32,
    pub color_text_primary: Color32,
    pub color_text_secondary: Color32,
    pub color_text_muted: Color32,
    pub color_accent: Color32,
    pub color_accent_contrast: Color32,
    pub color_ok: Color32,
    pub color_warn: Color32,
    pub color_alert: Color32,
    pub color_category_system: Color32,
    pub color_category_code: Color32,
    pub color_category_photo: Color32,
    pub color_category_media: Color32,
    pub color_category_document: Color32,
    pub color_category_archive: Color32,
}
