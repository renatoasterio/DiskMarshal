//! What a screen shows while a search runs: the totals so far and a way out.

use egui::{RichText, Ui};
use maccleaner_application::progress::ScanProgress;

use crate::components::{ButtonKind, HudPanel, activity_bar, hud_button};
use crate::i18n::pt_br;
use crate::theme::{FONT_SIZE_DATA, SPACE_2, SPACE_3, Tokens};

/// What the panel says and how it moves.
pub(crate) struct ProgressPanel<'a> {
    pub title: &'a str,
    /// A short note at the right of the title.
    pub meta: Option<&'a str>,
    pub progress: ScanProgress,
    /// Seconds on the frame clock.
    pub now: f64,
    /// Whether the activity bar moves.
    pub is_moving: bool,
}

/// Shows the panel. Returns `true` when the user asked to stop the search.
pub(crate) fn progress_panel(ui: &mut Ui, tokens: &Tokens, panel: &ProgressPanel<'_>) -> bool {
    let mut frame = HudPanel::new(panel.title);
    if let Some(meta) = panel.meta {
        frame = frame.meta(meta);
    }
    frame.show(ui, tokens, |ui| {
        let counted = format!(
            "{} {} · {}",
            pt_br::group_digits(panel.progress.items_seen),
            pt_br::ITEMS,
            panel.progress.bytes_seen
        );
        ui.label(RichText::new(counted).size(FONT_SIZE_DATA + 5.0));
        ui.add_space(SPACE_2);
        activity_bar(ui, tokens, panel.now, panel.is_moving);
        ui.add_space(SPACE_3);
        hud_button(ui, tokens, pt_br::CANCEL, ButtonKind::Quiet).clicked()
    })
}
