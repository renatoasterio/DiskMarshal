//! The framed panel every block of content sits in.

use egui::{Align, Frame, Label, Layout, Painter, Rect, RichText, Stroke, Ui, pos2};

use crate::theme::{CORNER_SIZE, FONT_SIZE_SMALL, SPACE_2, SPACE_3, Tokens};

/// Width of the line around a panel.
const FRAME_STROKE: f32 = 1.0;
/// Extra space between the letters of a panel title.
const TITLE_LETTER_SPACING: f32 = 1.5;

/// A panel with a title, an optional note on the right, and corner brackets.
pub(crate) struct HudPanel<'a> {
    title: &'a str,
    meta: Option<&'a str>,
    min_height: Option<f32>,
}

impl<'a> HudPanel<'a> {
    /// Creates a panel with the given title.
    pub(crate) const fn new(title: &'a str) -> Self {
        Self {
            title,
            meta: None,
            min_height: None,
        }
    }

    /// Makes the panel at least this tall, frame included, so the panels of a
    /// row end on the same line.
    pub(crate) const fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    /// Adds a short note at the right end of the title row.
    pub(crate) const fn meta(mut self, meta: &'a str) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Shows the panel and its content, and returns what the content returned.
    pub(crate) fn show<R>(
        self,
        ui: &mut Ui,
        tokens: &Tokens,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        let frame = Frame::NONE
            .fill(tokens.color_surface_panel)
            .stroke(Stroke::new(FRAME_STROKE, tokens.color_line))
            .inner_margin(SPACE_3);
        let shown = frame.show(ui, |ui| {
            if let Some(height) = self.min_height {
                ui.set_min_height((height - 2.0 * (SPACE_3 + FRAME_STROKE)).max(0.0));
            }
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.title.to_uppercase())
                        .size(FONT_SIZE_SMALL)
                        .extra_letter_spacing(TITLE_LETTER_SPACING)
                        .color(tokens.color_text_secondary),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(meta) = self.meta {
                        // A long note is cut short instead of pushing the panel wider.
                        ui.add(
                            Label::new(
                                RichText::new(meta)
                                    .size(FONT_SIZE_SMALL)
                                    .color(tokens.color_text_muted),
                            )
                            .truncate(),
                        );
                    }
                });
            });
            ui.add_space(SPACE_2);
            add_contents(ui)
        });
        paint_corners(ui.painter(), shown.response.rect, tokens);
        shown.inner
    }
}

/// Paints the top-left and bottom-right corner brackets.
fn paint_corners(painter: &Painter, rect: Rect, tokens: &Tokens) {
    let stroke = Stroke::new(1.0, tokens.color_accent);
    let (min, max) = (rect.min, rect.max);
    painter.line(
        vec![
            pos2(min.x, min.y + CORNER_SIZE),
            min,
            pos2(min.x + CORNER_SIZE, min.y),
        ],
        stroke,
    );
    painter.line(
        vec![
            pos2(max.x - CORNER_SIZE, max.y),
            max,
            pos2(max.x, max.y - CORNER_SIZE),
        ],
        stroke,
    );
}
