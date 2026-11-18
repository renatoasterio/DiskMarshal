//! The optimization tab: the changes offered for this connection, what only
//! the user can do, and the one button that applies what is selected.

use egui::{Align, Layout, RichText, Ui};

use super::view::{bullet, small};
use super::view_model::{NetworkTab, NetworkViewModel};
use crate::components::{
    ButtonKind, HudPanel, SelectionRow, action_footer, hud_button, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, risk_badge};
use crate::theme::{SPACE_2, SPACE_3, Tokens};

/// Space kept at the bottom for the footer.
const FOOTER_HEIGHT: f32 = 52.0;

/// What the user asked for in this frame.
enum Request {
    Toggle(usize),
    Apply,
    Undo,
}

/// Draws the tab for this frame.
pub(super) fn show(model: &mut NetworkViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    let Some(report) = model.report().cloned() else {
        let mut is_asked = false;
        HudPanel::new(pt_br::NETWORK_TAB_TUNING).show(ui, tokens, |ui| {
            ui.label(
                RichText::new(pt_br::NETWORK_TUNING_NEEDS_ANALYSIS)
                    .color(tokens.color_text_secondary),
            );
            ui.add_space(SPACE_3);
            is_asked =
                hud_button(ui, tokens, pt_br::NETWORK_ANALYZE, ButtonKind::Primary).clicked();
        });
        if is_asked {
            model.tab = NetworkTab::Analysis;
            model.analyze(ui.ctx(), context.services);
        }
        return;
    };

    let can_change = context.services.network_tuner.is_available();
    let mut request = None;
    ui.label(small(
        pt_br::NETWORK_TUNING_INTRO,
        tokens.color_text_secondary,
    ));
    ui.add_space(SPACE_2);
    let authentication = context.services.network_tuner.authentication();
    HudPanel::new(pt_br::NETWORK_ACTIONS)
        .meta(pt_br::admin_meta(authentication))
        .show(ui, tokens, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (index, offer) in model.optimizations().iter().enumerate() {
                let is_applied = model.is_applied(index);
                let detail = pt_br::network_optimization_detail(offer);
                let row = SelectionRow {
                    is_checked: model.is_selected(index),
                    is_enabled: can_change && !is_applied,
                    title: pt_br::network_optimization_title(&offer.kind),
                    detail: &detail,
                    badge: Some(risk_badge(tokens, offer.risk)),
                    ratio: None,
                    trailing: if is_applied {
                        pt_br::NETWORK_APPLIED
                    } else {
                        ""
                    },
                };
                if selection_row(ui, tokens, &row).clicked() {
                    request = Some(Request::Toggle(index));
                }
            }
            if model.has_undo() {
                ui.add_space(SPACE_2);
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if hud_button(ui, tokens, pt_br::NETWORK_UNDO, ButtonKind::Outline).clicked() {
                        request = Some(Request::Undo);
                    }
                });
            }
        });
    ui.add_space(SPACE_3);

    HudPanel::new(pt_br::NETWORK_ADVICE)
        .meta(pt_br::NETWORK_ADVICE_META)
        .show(ui, tokens, |ui| {
            let advice = report.advice();
            if advice.is_empty() {
                bullet(ui, tokens, pt_br::NETWORK_NO_ADVICE);
            }
            for suggestion in &advice {
                bullet(ui, tokens, &pt_br::network_advice(suggestion));
            }
            ui.add_space(SPACE_2);
            ui.label(small(pt_br::NETWORK_NO_TCP_TWEAKS, tokens.color_text_muted));
        });

    // The footer stays at the bottom of the screen whatever is above it.
    let gap = ui.available_height() - FOOTER_HEIGHT;
    if gap > 0.0 {
        ui.add_space(gap);
    }
    if footer(model, ui, tokens, can_change) {
        request = Some(Request::Apply);
    }

    match request {
        Some(Request::Toggle(index)) => model.toggle(index),
        Some(Request::Apply) => model.request_apply(),
        Some(Request::Undo) => model.request_undo(),
        None => {}
    }
}

/// Shows what is selected and the button that applies it. Returns `true`
/// when the button was clicked.
fn footer(model: &NetworkViewModel, ui: &mut Ui, tokens: &Tokens, can_change: bool) -> bool {
    let summary = pt_br::NETWORK_SELECTED_ACTIONS.of(model.selected_count());
    let notice = if can_change {
        model.notice()
    } else {
        Some(pt_br::NETWORK_TUNER_UNAVAILABLE)
    };
    action_footer(ui, tokens, &summary, notice, pt_br::NETWORK_APPLY) && can_change
}
