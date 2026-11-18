//! Drawing of the developer screen.

use std::path::PathBuf;

use egui::{Align, Layout, RichText, ScrollArea, Ui};
use maccleaner_domain::cleanup::RiskLevel;

use super::view_model::DeveloperViewModel;
use crate::components::{
    ButtonKind, ConfirmText, HudPanel, ProgressPanel, RemovalEvent, SelectionRow, action_footer,
    chip, hud_button, progress_panel, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, display_path};
use crate::theme::{FONT_SIZE_SMALL, SPACE_2, SPACE_3};

/// Space kept under the list for the footer.
const FOOTER_HEIGHT: f32 = 52.0;

/// What a row needs, copied out of the report so the model can change while drawing.
struct RowData {
    path: PathBuf,
    title: String,
    detail: String,
    risk: RiskLevel,
    ratio: f32,
    size: String,
    is_checked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut DeveloperViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.poll(context.services);
    let tokens = context.tokens;

    if let Some(progress) = model.progress() {
        let panel = ProgressPanel {
            title: pt_br::DEV_SEARCHING,
            meta: None,
            progress,
            now: context.now,
            is_moving: context.appearance.motion.allows_ambient(),
        };
        if progress_panel(ui, tokens, &panel) {
            model.cancel();
        }
        return;
    }
    let Some(root) = model.report().map(|report| report.root.clone()) else {
        introduction(model, ui, context);
        return;
    };

    toolbar(model, ui, context, &root);
    ui.add_space(SPACE_2);
    let rows = rows(model, context.services.wall_clock.unix_seconds());
    let list_height = (ui.available_height() - FOOTER_HEIGHT).max(120.0);
    ScrollArea::vertical()
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            HudPanel::new(pt_br::DEV_LIST_TITLE)
                .meta(&display_path(&root, &context.services.home))
                .show(ui, tokens, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if rows.is_empty() {
                        ui.label(
                            RichText::new(pt_br::DEV_NOTHING_FOUND)
                                .color(tokens.color_text_secondary),
                        );
                    }
                    for row in &rows {
                        let (badge, color) = match row.risk {
                            RiskLevel::Safe => (pt_br::RISK_SAFE, tokens.color_ok),
                            RiskLevel::Review => (pt_br::RISK_REVIEW, tokens.color_warn),
                        };
                        let clicked = selection_row(
                            ui,
                            tokens,
                            &SelectionRow {
                                is_checked: row.is_checked,
                                is_enabled: true,
                                title: &row.title,
                                detail: &row.detail,
                                badge: Some((badge, color)),
                                ratio: Some(row.ratio),
                                trailing: &row.size,
                            },
                        )
                        .clicked();
                        if clicked {
                            model.toggle(&row.path);
                        }
                    }
                });
        });

    let (count, size) = model.selection();
    let summary = format!("{count} {} · {size}", pt_br::SELECTED);
    if action_footer(ui, tokens, &summary, model.notice(), pt_br::DEV_ACTION) {
        model.request_removal();
    }

    let lines = [
        format!("{count} {} · {size}", pt_br::DEV_FOLDERS),
        pt_br::DEV_CONFIRM_SOURCE.to_owned(),
        pt_br::DEV_CONFIRM_RECREATE.to_owned(),
    ];
    let text = ConfirmText {
        body: pt_br::DEV_CONFIRM_BODY,
        lines: &lines,
        action: pt_br::DEV_AUTHORIZE,
    };
    match model.removal.show(ui.ctx(), context, &text) {
        RemovalEvent::Confirmed => model.confirm(ui.ctx(), context.services),
        RemovalEvent::Finished => model.start(ui.ctx(), context.services, root),
        RemovalEvent::Idle => {}
    }
}

/// What the user sees before the first search: a choice of where to look.
fn introduction(model: &mut DeveloperViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::DEV_INTRO_TITLE).show(ui, tokens, |ui| {
        ui.label(RichText::new(pt_br::DEV_INTRO_BODY).color(tokens.color_text_secondary));
        if let Some(failure) = model.failure() {
            ui.add_space(SPACE_2);
            ui.label(RichText::new(failure).color(tokens.color_alert));
        }
        ui.add_space(SPACE_3);
        ui.horizontal(|ui| {
            if hud_button(ui, tokens, pt_br::CHOOSE_FOLDER, ButtonKind::Primary).clicked()
                && let Some(folder) = context.services.folder_picker.pick_folder()
            {
                model.start(ui.ctx(), context.services, folder);
            }
            ui.add_space(SPACE_2);
            if hud_button(ui, tokens, pt_br::MAP_SCAN_HOME, ButtonKind::Outline).clicked() {
                model.start(ui.ctx(), context.services, context.services.home.clone());
            }
        });
        ui.add_space(SPACE_2);
        ui.label(
            RichText::new(pt_br::DEV_INTRO_NOTE)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
    });
}

/// The row above the list: the ecosystem filter and a new search.
fn toolbar(
    model: &mut DeveloperViewModel,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
    root: &std::path::Path,
) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        if chip(ui, tokens, pt_br::ALL, model.filter.is_none(), None).clicked() {
            model.filter = None;
        }
        for ecosystem in model.ecosystems() {
            let is_selected = model.filter == Some(ecosystem);
            if chip(ui, tokens, ecosystem.label(), is_selected, None).clicked() {
                model.filter = Some(ecosystem);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if hud_button(ui, tokens, pt_br::DEV_SEARCH_AGAIN, ButtonKind::Outline).clicked() {
                model.start(ui.ctx(), context.services, root.to_path_buf());
            }
            if hud_button(ui, tokens, pt_br::CHOOSE_FOLDER, ButtonKind::Quiet).clicked()
                && let Some(folder) = context.services.folder_picker.pick_folder()
            {
                model.start(ui.ctx(), context.services, folder);
            }
        });
    });
}

fn rows(model: &DeveloperViewModel, now_unix_seconds: i64) -> Vec<RowData> {
    let largest = model.largest().as_bytes();
    model
        .visible()
        .into_iter()
        .map(|item| {
            let artifact = &item.artifact;
            let age = artifact
                .project_modified_unix_seconds
                .map_or_else(String::new, |modified| {
                    format!(
                        " · {} {}",
                        pt_br::DEV_TOUCHED,
                        pt_br::age(now_unix_seconds - modified)
                    )
                });
            RowData {
                path: artifact.path.clone(),
                title: format!("{}  ›  {}", artifact.project, artifact.folder),
                detail: format!("{}{age}", artifact.ecosystem.label()),
                risk: artifact.risk,
                ratio: if largest == 0 {
                    0.0
                } else {
                    artifact.size.as_bytes() as f32 / largest as f32
                },
                size: artifact.size.to_string(),
                is_checked: model.is_selected(&artifact.path),
            }
        })
        .collect()
}
