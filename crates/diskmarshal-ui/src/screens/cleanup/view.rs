//! Drawing of the cleanup screen.

use diskmarshal_application::ports::AuthenticationKind;
use diskmarshal_domain::cleanup::{CleanupFinding, PrivilegeScope, RiskLevel};
use egui::{RichText, ScrollArea, Ui};

use super::view_model::{CleanupViewModel, Selection};
use crate::components::{
    ButtonKind, ConfirmText, HudPanel, RemovalEvent, SelectionRow, action_footer, activity_bar,
    hud_button, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, risk_badge};
use crate::theme::{FONT_SIZE_DATA, SPACE_2, SPACE_3, Tokens};

/// Space kept under the lists for the footer.
const FOOTER_HEIGHT: f32 = 52.0;

/// What a row needs, copied out of the plan so the model can change while drawing.
struct RowData {
    id: String,
    title: String,
    detail: String,
    risk: RiskLevel,
    ratio: f32,
    size: String,
    is_enabled: bool,
    is_checked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut CleanupViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.ensure_started(ui.ctx(), context.services);
    model.poll();
    let tokens = context.tokens;

    if let Some(progress) = model.progress() {
        HudPanel::new(pt_br::CLEANUP_ANALYZING).show(ui, tokens, |ui| {
            let counted = format!(
                "{} {} · {}",
                pt_br::group_digits(progress.items_seen),
                pt_br::ITEMS,
                progress.bytes_seen
            );
            ui.label(RichText::new(counted).size(FONT_SIZE_DATA + 5.0));
            ui.add_space(SPACE_2);
            activity_bar(
                ui,
                tokens,
                context.now,
                context.appearance.motion.allows_ambient(),
            );
        });
        return;
    }
    if model.plan().is_none() {
        HudPanel::new(pt_br::CLEANUP_FAILED).show(ui, tokens, |ui| {
            let reason = model.failure().unwrap_or(pt_br::SCAN_LOST).to_owned();
            ui.label(RichText::new(reason).color(tokens.color_alert));
            ui.add_space(SPACE_3);
            if hud_button(ui, tokens, pt_br::TRY_AGAIN, ButtonKind::Outline).clicked() {
                model.analyze(ui.ctx(), context.services);
            }
        });
        return;
    }

    let largest = model.largest();
    let (user_rows, admin_rows): (Vec<RowData>, Vec<RowData>) = {
        let rows = |scope: PrivilegeScope| {
            model
                .plan()
                .iter()
                .flat_map(|plan| &plan.findings)
                .filter(|finding| finding.rule.scope == scope)
                .map(|finding| row_data(model, finding, largest.as_bytes()))
                .collect()
        };
        (
            rows(PrivilegeScope::User),
            rows(PrivilegeScope::Administrator),
        )
    };

    let admin_meta = if model.cleans_system() {
        pt_br::cleanup_admin_meta(model.authentication())
    } else {
        pt_br::CLEANUP_ADMIN_UNAVAILABLE
    };
    let list_height = (ui.available_height() - FOOTER_HEIGHT).max(120.0);
    ScrollArea::vertical()
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            group(
                ui,
                tokens,
                model,
                pt_br::CLEANUP_USER_GROUP,
                pt_br::CLEANUP_USER_META,
                &user_rows,
            );
            ui.add_space(SPACE_3);
            group(
                ui,
                tokens,
                model,
                pt_br::CLEANUP_ADMIN_GROUP,
                admin_meta,
                &admin_rows,
            );
            if let Some(failure) = model.failure() {
                ui.add_space(SPACE_2);
                ui.label(RichText::new(failure).color(tokens.color_alert));
            }
        });

    let selection = model.selection();
    let summary = format!(
        "{} {} · {}",
        selection.rules,
        pt_br::SELECTED,
        selection.reclaimable
    );
    if action_footer(ui, tokens, &summary, model.notice(), pt_br::CLEANUP_ACTION) {
        model.request_removal();
    }

    let lines = confirm_lines(&selection, model.authentication());
    let text = ConfirmText {
        body: pt_br::CLEANUP_CONFIRM_BODY,
        lines: &lines,
        action: pt_br::CLEANUP_AUTHORIZE,
    };
    match model.removal.show(ui.ctx(), context, &text) {
        RemovalEvent::Confirmed => model.confirm(ui.ctx(), context.services),
        RemovalEvent::Finished => model.analyze(ui.ctx(), context.services),
        RemovalEvent::Idle => {}
    }
}

fn row_data(model: &CleanupViewModel, finding: &CleanupFinding, largest: u64) -> RowData {
    let size = finding.reclaimable();
    RowData {
        id: finding.rule.id.clone(),
        title: finding.rule.name.clone(),
        detail: finding.rule.description.clone(),
        risk: finding.rule.risk,
        ratio: if largest == 0 {
            0.0
        } else {
            size.as_bytes() as f32 / largest as f32
        },
        size: if finding.requests.is_empty() {
            pt_br::EMPTY.to_owned()
        } else {
            size.to_string()
        },
        is_enabled: model.is_selectable(finding),
        is_checked: model.is_selected(&finding.rule.id),
    }
}

fn group(
    ui: &mut Ui,
    tokens: &Tokens,
    model: &mut CleanupViewModel,
    title: &str,
    meta: &str,
    rows: &[RowData],
) {
    if rows.is_empty() {
        return;
    }
    HudPanel::new(title).meta(meta).show(ui, tokens, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in rows {
            let (badge, color) = risk_badge(tokens, row.risk);
            let clicked = selection_row(
                ui,
                tokens,
                &SelectionRow {
                    is_checked: row.is_checked,
                    is_enabled: row.is_enabled,
                    title: &row.title,
                    detail: &row.detail,
                    badge: Some((badge, color)),
                    ratio: Some(row.ratio),
                    trailing: &row.size,
                },
            )
            .clicked();
            if clicked {
                model.toggle(&row.id);
            }
        }
    });
}

fn confirm_lines(selection: &Selection, authentication: AuthenticationKind) -> Vec<String> {
    let mut lines = vec![format!(
        "{} {} · {} {} · {}",
        selection.rules,
        pt_br::RULES,
        pt_br::group_digits(selection.items as u64),
        pt_br::ITEMS,
        selection.reclaimable
    )];
    if selection.uses_trash {
        lines.push(pt_br::CLEANUP_CONFIRM_TRASH.to_owned());
    }
    if selection.needs_administrator {
        lines.push(pt_br::cleanup_confirm_admin(authentication).to_owned());
    }
    lines.push(pt_br::CLEANUP_CONFIRM_RECHECK.to_owned());
    lines
}
