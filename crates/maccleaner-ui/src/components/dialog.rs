//! A panel shown over the whole window, which must be answered before
//! anything else can be clicked.

use egui::{Area, Color32, Context, Id, Order, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use crate::components::HudPanel;
use crate::theme::Tokens;

const DIALOG_WIDTH: f32 = 480.0;
/// How far above the middle of the window the dialog starts.
const DIALOG_RISE: f32 = 150.0;
/// Opacity of the shade over the rest of the window, out of 255.
const SHADE_ALPHA: u8 = 170;

/// Shows a dialog titled `title` and returns what its content returned.
pub(crate) fn dialog<R>(
    ctx: &Context,
    tokens: &Tokens,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let window = ctx.content_rect();
    Area::new(Id::unique("dialog"))
        .order(Order::Foreground)
        .fixed_pos(window.min)
        .show(ctx, |ui| {
            // Taking the whole window keeps clicks from reaching what is underneath.
            ui.allocate_rect(window, Sense::click());
            ui.painter()
                .rect_filled(window, 0.0, Color32::from_black_alpha(SHADE_ALPHA));
            let top_left = pos2(
                window.center().x - DIALOG_WIDTH / 2.0,
                window.center().y - DIALOG_RISE,
            );
            let space = Rect::from_min_size(top_left, vec2(DIALOG_WIDTH, window.height()));
            ui.scope_builder(UiBuilder::new().max_rect(space), |ui| {
                HudPanel::new(title).show(ui, tokens, add_contents)
            })
            .inner
        })
        .inner
}
