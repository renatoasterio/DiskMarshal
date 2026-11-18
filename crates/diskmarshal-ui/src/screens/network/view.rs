//! Drawing of the network screen: the tabs, what the three of them share,
//! and the dialog of a change to the settings.

use egui::{
    Align, Align2, Color32, Context, FontFamily, FontId, Layout, RichText, Sense, Stroke,
    StrokeKind, Ui, vec2,
};

use super::view_model::{ChangeFlow, ChangeKind, NetworkTab, NetworkViewModel};
use super::{analysis_tab, traffic_tab, tuning_tab};
use crate::components::{ButtonKind, activity_bar, chip, dialog, hud_button, segmented_bar};
use crate::i18n::pt_br;
use crate::screens::ScreenContext;
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

const BADGE_WIDTH: f32 = 84.0;
const BADGE_HEIGHT: f32 = 18.0;
/// Width kept for the label of a meter row.
const METER_LABEL_WIDTH: f32 = 58.0;
const METER_CELLS: usize = 14;

/// What the user asked of the dialog in this frame.
enum DialogAction {
    Confirm,
    Close,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut NetworkViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.ensure_started(ui.ctx(), context.services);
    model.poll(context.services);
    tabs(model, ui, context.tokens);
    ui.add_space(SPACE_3);
    match model.tab {
        NetworkTab::Traffic => traffic_tab::show(model, ui, context),
        NetworkTab::Analysis => analysis_tab::show(model, ui, context),
        NetworkTab::Tuning => tuning_tab::show(model, ui, context),
    }
    change_dialog(model, ui.ctx(), context);
}

fn tabs(model: &mut NetworkViewModel, ui: &mut Ui, tokens: &Tokens) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        for tab in NetworkTab::ALL {
            let offers = model.optimizations().len();
            let label = if tab == NetworkTab::Tuning && offers > 0 {
                format!("{} · {offers}", tab.label())
            } else {
                tab.label().to_owned()
            };
            if chip(ui, tokens, &label, model.tab == tab, None).clicked() {
                model.tab = tab;
            }
        }
        if let Some(report) = model.report() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let way_out = pt_br::network_interface(&report.profile);
                ui.label(small(&way_out, tokens.color_text_muted));
            });
        }
    });
}

/// Text in the small size.
pub(super) fn small(text: &str, color: Color32) -> RichText {
    RichText::new(text).size(FONT_SIZE_SMALL).color(color)
}

/// A short label in a colored outline.
pub(super) fn badge(ui: &mut Ui, label: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(BADGE_WIDTH, BADGE_HEIGHT), Sense::hover());
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, color), StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label.to_uppercase(),
        FontId::new(FONT_SIZE_SMALL - 1.0, FontFamily::Monospace),
        color,
    );
}

/// A label, a bar and a value on one line.
pub(super) fn meter_row(
    ui: &mut Ui,
    tokens: &Tokens,
    label: &str,
    ratio: f32,
    value: &str,
    color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        ui.allocate_ui(vec2(METER_LABEL_WIDTH, FONT_SIZE_DATA + 4.0), |ui| {
            ui.label(RichText::new(label).size(FONT_SIZE_DATA - 1.0));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).family(FontFamily::Monospace));
            segmented_bar(ui, tokens, ratio, METER_CELLS, color);
        });
    });
}

/// One item of a list of sentences.
pub(super) fn bullet(ui: &mut Ui, tokens: &Tokens, text: &str) {
    ui.label(
        RichText::new(format!("· {text}"))
            .size(FONT_SIZE_DATA - 1.0)
            .color(tokens.color_text_secondary),
    );
}

/// Shows, over the whole window, where a change to the settings stands.
fn change_dialog(model: &mut NetworkViewModel, ctx: &Context, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    let changes_of = |ui: &mut Ui, changes: &[diskmarshal_domain::network::NetworkChange]| {
        ui.add_space(SPACE_2);
        for change in changes {
            bullet(ui, tokens, &pt_br::network_change(change));
        }
        ui.add_space(SPACE_3);
    };
    let buttons = |ui: &mut Ui, confirm: Option<&str>| {
        let mut action = None;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
            if let Some(confirm) = confirm {
                if hud_button(ui, tokens, confirm, ButtonKind::Primary).clicked() {
                    action = Some(DialogAction::Confirm);
                }
                if hud_button(ui, tokens, pt_br::CANCEL, ButtonKind::Quiet).clicked() {
                    action = Some(DialogAction::Close);
                }
            } else if hud_button(ui, tokens, pt_br::CLOSE, ButtonKind::Primary).clicked() {
                action = Some(DialogAction::Close);
            }
        });
        action
    };

    let action = match &model.flow {
        ChangeFlow::Closed => None,
        ChangeFlow::Confirming(kind) => {
            let (changes, _) = model.pending(*kind);
            let (body, confirm) = match kind {
                ChangeKind::Apply => (pt_br::NETWORK_CONFIRM_BODY, pt_br::NETWORK_CONFIRM_APPLY),
                ChangeKind::Undo => (pt_br::NETWORK_UNDO_BODY, pt_br::NETWORK_UNDO_APPLY),
            };
            dialog(ctx, tokens, pt_br::NETWORK_CONFIRM_TITLE, |ui| {
                ui.label(body);
                changes_of(ui, &changes);
                let authentication = context.services.network_tuner.authentication();
                ui.label(small(
                    pt_br::network_authentication(authentication),
                    tokens.color_text_muted,
                ));
                ui.add_space(SPACE_3);
                buttons(ui, Some(confirm))
            })
        }
        ChangeFlow::Running { changes, .. } => {
            dialog(ctx, tokens, pt_br::NETWORK_APPLYING_TITLE, |ui| {
                changes_of(ui, changes);
                activity_bar(ui, tokens, context.now, true);
                ui.add_space(SPACE_1);
            });
            None
        }
        ChangeFlow::Done(kind, changes) => {
            let (title, body) = match kind {
                ChangeKind::Apply => (pt_br::NETWORK_APPLIED_TITLE, pt_br::NETWORK_APPLIED_BODY),
                ChangeKind::Undo => (pt_br::NETWORK_UNDONE_TITLE, pt_br::NETWORK_UNDONE_BODY),
            };
            dialog(ctx, tokens, title, |ui| {
                ui.label(body);
                changes_of(ui, changes);
                if *kind == ChangeKind::Apply && model.has_undo() {
                    ui.label(small(pt_br::NETWORK_APPLIED_NOTE, tokens.color_text_muted));
                    ui.add_space(SPACE_3);
                }
                buttons(ui, None)
            })
        }
        ChangeFlow::Failed(reason) => dialog(ctx, tokens, pt_br::NETWORK_FAILED_TITLE, |ui| {
            ui.label(RichText::new(reason).color(tokens.color_alert));
            ui.add_space(SPACE_3);
            buttons(ui, None)
        }),
    };
    match action {
        Some(DialogAction::Confirm) => model.confirm(ctx, context.services),
        Some(DialogAction::Close) => model.close_flow(),
        None => {}
    }
}
