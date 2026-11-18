//! The short start-up sequence shown when the app opens.

use egui::{Align2, FontFamily, FontId, Id, Rect, Sense, Ui, pos2, vec2};

use crate::animation::Timeline;
use crate::components::segmented_bar;
use crate::effects::glow::{self, TextStyle};
use crate::i18n::pt_br;
use crate::theme::{
    FONT_SIZE_SMALL, MOTION_BOOT_HOLD_SECONDS, MOTION_BOOT_SECONDS, SPACE_3, SPACE_4, Tokens,
    display_bold,
};

const BLOCK_WIDTH: f32 = 520.0;
const TITLE_FONT_SIZE: f32 = 30.0;
const TITLE_LETTER_SPACING: f32 = 7.0;
const LINE_FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 20.0;
/// Horizontal position of the result ("ok") of each line.
const RESULT_COLUMN: f32 = 360.0;
const BAR_HEIGHT: f32 = 10.0;
const BAR_CELLS: usize = 24;

/// The start-up sequence: a few status lines and a bar that fills up.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BootSequence {
    timeline: Timeline,
}

impl BootSequence {
    /// Starts the sequence at `now`, in seconds on the frame clock.
    pub(crate) const fn start(now: f64) -> Self {
        Self {
            timeline: Timeline::start(now),
        }
    }

    /// Paints the sequence over the whole of `ui` and returns `true` once it
    /// is over, either because it ran to the end or because the user clicked.
    pub(crate) fn show(&self, ui: &mut Ui, tokens: &Tokens, now: f64, glow_strength: f32) -> bool {
        let rect = ui.max_rect();
        let response = ui.interact(rect, Id::unique("boot_sequence"), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, tokens.color_surface_void);

        let progress = self.timeline.progress(now, MOTION_BOOT_SECONDS);
        let lines = pt_br::BOOT_LINES;
        let block_height = TITLE_FONT_SIZE + SPACE_4 + LINE_HEIGHT * lines.len() as f32;
        let origin = pos2(
            rect.center().x - BLOCK_WIDTH / 2.0,
            rect.center().y - block_height / 2.0 - LINE_HEIGHT,
        );
        glow::text(
            &painter,
            origin,
            Align2::LEFT_TOP,
            pt_br::BRAND,
            &TextStyle {
                font: FontId::new(TITLE_FONT_SIZE, display_bold()),
                color: tokens.color_accent,
                letter_spacing: TITLE_LETTER_SPACING,
                glow: glow_strength,
            },
        );

        let line_font = FontId::new(LINE_FONT_SIZE, FontFamily::Monospace);
        let lines_top = origin.y + TITLE_FONT_SIZE + SPACE_4;
        let visible = (progress * lines.len() as f32).ceil() as usize;
        for (index, (step, result)) in lines.iter().take(visible).enumerate() {
            let y = lines_top + LINE_HEIGHT * index as f32;
            painter.text(
                pos2(origin.x, y),
                Align2::LEFT_TOP,
                step,
                line_font.clone(),
                tokens.color_text_secondary,
            );
            painter.text(
                pos2(origin.x + RESULT_COLUMN, y),
                Align2::LEFT_TOP,
                result,
                line_font.clone(),
                tokens.color_ok,
            );
        }

        let bar_top = lines_top + LINE_HEIGHT * lines.len() as f32 + SPACE_3;
        let bar = Rect::from_min_size(pos2(origin.x, bar_top), vec2(BLOCK_WIDTH, BAR_HEIGHT));
        segmented_bar::paint(
            &painter,
            bar,
            progress,
            BAR_CELLS,
            tokens.color_accent,
            tokens.color_line,
        );
        painter.text(
            pos2(origin.x, bar_top + BAR_HEIGHT + SPACE_3),
            Align2::LEFT_TOP,
            pt_br::BOOT_SKIP_HINT,
            FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace),
            tokens.color_text_muted,
        );

        response.clicked()
            || self.timeline.elapsed(now) >= MOTION_BOOT_SECONDS + MOTION_BOOT_HOLD_SECONDS
    }
}
