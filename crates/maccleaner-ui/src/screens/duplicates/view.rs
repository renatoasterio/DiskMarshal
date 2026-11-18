//! Drawing of the duplicates screen.

use std::path::PathBuf;

use egui::{Align, Layout, RichText, ScrollArea, Ui};

use super::view_model::{DuplicatesViewModel, MAX_GROUPS_SHOWN, MinimumSize};
use crate::components::{
    ButtonKind, ConfirmText, HudPanel, ProgressPanel, RemovalEvent, SelectionRow, action_footer,
    chip, hud_button, progress_panel, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, display_path};
use crate::theme::{FONT_SIZE_SMALL, SPACE_2, SPACE_3};

/// Space kept under the list for the footer.
const FOOTER_HEIGHT: f32 = 52.0;

/// One group, copied out of the report so the model can change while drawing.
struct GroupData {
    title: String,
    meta: String,
    copies: Vec<CopyData>,
}

struct CopyData {
    path: PathBuf,
    shown_path: String,
    detail: String,
    size: String,
    is_checked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut DuplicatesViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.poll();
    let tokens = context.tokens;

    if let Some(progress) = model.progress() {
        let panel = ProgressPanel {
            title: pt_br::DUP_SEARCHING,
            meta: Some(pt_br::DUP_SEARCHING_META),
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
    let groups = groups(model, context);
    let hidden = model
        .report()
        .map_or(0, |report| report.groups.len())
        .saturating_sub(groups.len());
    let list_height = (ui.available_height() - FOOTER_HEIGHT).max(120.0);
    ScrollArea::vertical()
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if groups.is_empty() {
                HudPanel::new(pt_br::DUP_NONE_TITLE).show(ui, tokens, |ui| {
                    ui.label(
                        RichText::new(pt_br::DUP_NONE_BODY).color(tokens.color_text_secondary),
                    );
                });
            }
            for group in &groups {
                HudPanel::new(&group.title)
                    .meta(&group.meta)
                    .show(ui, tokens, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for copy in &group.copies {
                            let keep = (!copy.is_checked)
                                .then_some((pt_br::DUP_KEEP, tokens.color_accent));
                            let clicked = selection_row(
                                ui,
                                tokens,
                                &SelectionRow {
                                    is_checked: copy.is_checked,
                                    is_enabled: true,
                                    title: &copy.shown_path,
                                    detail: &copy.detail,
                                    badge: keep,
                                    ratio: None,
                                    trailing: &copy.size,
                                },
                            )
                            .clicked();
                            if clicked {
                                model.review.toggle(&copy.path, pt_br::DUP_KEEP_ONE);
                            }
                        }
                    });
                ui.add_space(SPACE_2);
            }
            if hidden > 0 {
                let note = format!(
                    "{} {hidden} {}",
                    pt_br::DUP_MORE_PREFIX,
                    pt_br::DUP_MORE_SUFFIX
                );
                ui.label(
                    RichText::new(note)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_text_muted),
                );
            }
        });

    let (count, size) = model.review.selection();
    let summary = format!("{count} {} · {size}", pt_br::SELECTED);
    if action_footer(
        ui,
        tokens,
        &summary,
        model.review.notice(),
        pt_br::MOVE_TO_TRASH,
    ) {
        model.review.request_removal();
    }

    let lines = [
        format!("{count} {} · {size}", pt_br::DUP_FILES),
        pt_br::DUP_CONFIRM_KEEP.to_owned(),
        pt_br::TRASH_FREES_LATER.to_owned(),
    ];
    let text = ConfirmText {
        body: pt_br::DUP_CONFIRM_BODY,
        lines: &lines,
        action: pt_br::MOVE_TO_TRASH,
    };
    match model.review.removal.show(ui.ctx(), context, &text) {
        RemovalEvent::Confirmed => model
            .review
            .confirm(ui.ctx(), context.services, root.clone()),
        RemovalEvent::Finished => model.start(ui.ctx(), context.services, root),
        RemovalEvent::Idle => {}
    }
}

/// What the user sees before the first search: where to look and how small
/// a file is still worth comparing.
fn introduction(model: &mut DuplicatesViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::DUP_INTRO_TITLE).show(ui, tokens, |ui| {
        ui.label(RichText::new(pt_br::DUP_INTRO_BODY).color(tokens.color_text_secondary));
        if let Some(failure) = model.failure() {
            ui.add_space(SPACE_2);
            ui.label(RichText::new(failure).color(tokens.color_alert));
        }
        ui.add_space(SPACE_3);
        minimum_size(model, ui, context);
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
            RichText::new(pt_br::DUP_INTRO_NOTE)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
    });
}

fn minimum_size(model: &mut DuplicatesViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        ui.label(
            RichText::new(pt_br::DUP_MINIMUM.to_uppercase())
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
        for minimum in MinimumSize::ALL {
            if chip(ui, tokens, minimum.label(), model.minimum == minimum, None).clicked() {
                model.minimum = minimum;
            }
        }
    });
}

/// The row above the list: the size limit and a new search.
fn toolbar(
    model: &mut DuplicatesViewModel,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
    root: &std::path::Path,
) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        minimum_size(model, ui, context);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
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

fn groups(model: &DuplicatesViewModel, context: &ScreenContext<'_>) -> Vec<GroupData> {
    let now = context.services.wall_clock.unix_seconds();
    let home = &context.services.home;
    let Some(report) = model.report() else {
        return Vec::new();
    };
    let root = &report.root;
    report
        .groups
        .iter()
        .take(MAX_GROUPS_SHOWN)
        .map(|found| {
            let group = &found.group;
            let name = group
                .copies
                .first()
                .and_then(|copy| copy.path.file_name())
                .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
            GroupData {
                title: name,
                meta: format!(
                    "{} {} · {} {}",
                    group.copies.len(),
                    pt_br::DUP_COPIES,
                    group.size,
                    pt_br::DUP_EACH
                ),
                copies: group
                    .copies
                    .iter()
                    .map(|copy| CopyData {
                        path: copy.path.clone(),
                        shown_path: shown_path(&copy.path, root, home),
                        detail: copy
                            .modified_unix_seconds
                            .map_or_else(String::new, |modified| {
                                format!("{} {}", pt_br::DEV_TOUCHED, pt_br::age(now - modified))
                            }),
                        size: copy.allocated.to_string(),
                        is_checked: model.review.is_marked(&copy.path),
                    })
                    .collect(),
            }
        })
        .collect()
}

/// A copy's location, written relative to the searched folder when it is
/// inside it. That keeps the part that differs between copies in view.
fn shown_path(path: &std::path::Path, root: &std::path::Path, home: &std::path::Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => display_path(path, home),
    }
}
