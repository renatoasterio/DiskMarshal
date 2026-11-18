//! Drawing of the apps screen.

use diskmarshal_application::ports::AuthenticationKind;
use std::path::{Path, PathBuf};

use diskmarshal_domain::applications::InstalledApp;
use diskmarshal_domain::cleanup::{PrivilegeScope, RiskLevel};
use egui::{
    Align2, FontFamily, FontId, Rect, Response, RichText, Role, ScrollArea, Sense, Stroke,
    TextEdit, Ui, Vec2, WidgetInfo, pos2, vec2,
};

use super::view_model::{AppsViewModel, Selection};
use crate::components::{
    ButtonKind, CellWidth, ConfirmText, HudPanel, Icon, ProgressPanel, RemovalEvent, SelectionRow,
    action_footer, activity_bar, cell_row, hud_button, key_value, progress_panel, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, display_path, risk_badge, shorten_middle};
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

/// Space kept under the two columns for the footer.
const FOOTER_HEIGHT: f32 = 52.0;
/// Width of the column that lists the apps.
const LIST_WIDTH: f32 = 286.0;
/// What a panel puts between its frame and its content, frame included.
const PANEL_INSET: f32 = SPACE_3 + 1.0;
const APP_ROW_HEIGHT: f32 = 38.0;
const LOCK_SIZE: f32 = 11.0;
const SIZE_WIDTH: f32 = 70.0;
const CHOSEN_MARKER_WIDTH: f32 = 2.0;
const CHOSEN_FILL_OPACITY: f32 = 0.1;
const HOVER_FILL_OPACITY: f32 = 0.05;
/// Longest name of an item shown in a row; a longer one loses its middle.
const NAME_CHARACTERS: usize = 36;
/// The same for the quieter line, which is set smaller.
const DETAIL_CHARACTERS: usize = 44;

/// One app of the list, copied out so the model can change while drawing.
struct AppRow {
    bundle: PathBuf,
    name: String,
    detail: String,
    size: String,
    needs_administrator: bool,
    is_running: bool,
    is_chosen: bool,
}

/// One item of the chosen app, copied out so the model can change while drawing.
struct ItemRow {
    target: PathBuf,
    title: String,
    detail: String,
    risk: RiskLevel,
    size: String,
    is_enabled: bool,
    is_checked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut AppsViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.ensure_started(ui.ctx(), context.services);
    model.poll();
    let tokens = context.tokens;

    if let Some(progress) = model.progress() {
        let panel = ProgressPanel {
            title: pt_br::APPS_LISTING,
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
    if model.apps().is_none() {
        HudPanel::new(pt_br::APPS_FAILED).show(ui, tokens, |ui| {
            let reason = model.failure().unwrap_or(pt_br::APPS_CANCELLED).to_owned();
            ui.label(RichText::new(reason).color(tokens.color_alert));
            ui.add_space(SPACE_3);
            if hud_button(ui, tokens, pt_br::TRY_AGAIN, ButtonKind::Outline).clicked() {
                model.list(ui.ctx(), context.services);
            }
        });
        return;
    }

    let height = (ui.available_height() - FOOTER_HEIGHT).max(160.0);
    let bottom = ui.cursor().top() + height;
    let mut clicked = None;
    cell_row(
        ui,
        &[CellWidth::Fixed(LIST_WIDTH), CellWidth::Share(1.0)],
        |index, ui| {
            if index == 0 {
                clicked = app_list(model, ui, tokens, bottom);
            } else {
                detail(model, ui, context, height);
            }
        },
    );
    if let Some(bundle) = clicked {
        model.choose(ui.ctx(), context.services, &bundle);
    }

    let selection = model.selection();
    let summary = format!(
        "{} {} · {}",
        selection.items,
        pt_br::SELECTED,
        selection.reclaimable
    );
    let chosen = model.chosen_app().cloned();
    // An app that is open says so all the time, not only when asked to go.
    let open = chosen
        .as_ref()
        .filter(|app| app.is_running)
        .map(|app| pt_br::apps_running(&app.name));
    let notice = model.notice().map(str::to_owned).or(open);
    if action_footer(ui, tokens, &summary, notice.as_deref(), pt_br::APPS_ACTION) {
        model.request_removal();
    }

    let name = chosen.map_or_else(String::new, |app| app.name);
    let body = pt_br::apps_confirm_body(&name);
    let lines = confirm_lines(&selection, model.authentication());
    let text = ConfirmText {
        body: &body,
        lines: &lines,
        action: pt_br::APPS_ACTION,
    };
    match model.removal.show(ui.ctx(), context, &text) {
        RemovalEvent::Confirmed => model.confirm(ui.ctx(), context.services),
        RemovalEvent::Finished => model.list(ui.ctx(), context.services),
        RemovalEvent::Idle => {}
    }
}

/// The column of installed apps, with the search above it. Returns the
/// bundle of the app that was clicked.
fn app_list(
    model: &mut AppsViewModel,
    ui: &mut Ui,
    tokens: &Tokens,
    bottom: f32,
) -> Option<PathBuf> {
    let (count, size) = model.totals();
    let meta = format!("{count} {} · {size}", pt_br::APPS_COUNT);
    let mut clicked = None;
    HudPanel::new(pt_br::APPS_LIST_TITLE)
        .meta(&meta)
        .min_height(bottom - ui.cursor().top())
        .show(ui, tokens, |ui| {
            ui.add(
                TextEdit::singleline(&mut model.query)
                    .hint_text(pt_br::APPS_SEARCH_HINT)
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(SPACE_2);
            let rows = app_rows(model);
            let room = (bottom - PANEL_INSET - ui.cursor().top()).max(APP_ROW_HEIGHT);
            ScrollArea::vertical()
                .id_salt("apps_list")
                .max_height(room)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if rows.is_empty() {
                        let nothing = if model.query.trim().is_empty() {
                            pt_br::APPS_NONE_INSTALLED
                        } else {
                            pt_br::APPS_NONE_MATCH
                        };
                        ui.label(RichText::new(nothing).color(tokens.color_text_secondary));
                    }
                    for row in &rows {
                        if app_row(ui, tokens, row).clicked() {
                            clicked = Some(row.bundle.clone());
                        }
                    }
                });
        });
    clicked
}

fn app_rows(model: &AppsViewModel) -> Vec<AppRow> {
    model
        .visible()
        .into_iter()
        .map(|app| AppRow {
            bundle: app.path.clone(),
            name: app.name.clone(),
            detail: app.version.as_ref().map_or_else(
                || app.bundle_id.clone(),
                |version| format!("{} {version}", pt_br::APPS_VERSION),
            ),
            size: app.size.to_string(),
            needs_administrator: app.scope == PrivilegeScope::Administrator,
            is_running: app.is_running,
            is_chosen: model.is_chosen(&app.path),
        })
        .collect()
}

/// One app of the list. The response is clicked when the user chooses it.
fn app_row(ui: &mut Ui, tokens: &Tokens, row: &AppRow) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), APP_ROW_HEIGHT), Sense::click());
    let painter = ui.painter_at(rect);
    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, tokens.color_line),
    );
    if row.is_chosen {
        painter.rect_filled(
            rect,
            0.0,
            tokens.color_accent.gamma_multiply(CHOSEN_FILL_OPACITY),
        );
        let marker = Rect::from_min_size(rect.min, vec2(CHOSEN_MARKER_WIDTH, rect.height()));
        painter.rect_filled(marker, 0.0, tokens.color_accent);
    } else if response.hovered() || response.has_focus() {
        painter.rect_filled(
            rect,
            0.0,
            tokens.color_accent.gamma_multiply(HOVER_FILL_OPACITY),
        );
    }

    let right = rect.right() - SPACE_2;
    painter.text(
        pos2(right, rect.center().y),
        Align2::RIGHT_CENTER,
        &row.size,
        FontId::new(FONT_SIZE_DATA, FontFamily::Monospace),
        tokens.color_text_primary,
    );

    let left = rect.left() + SPACE_2 + CHOSEN_MARKER_WIDTH;
    let names = Rect::from_min_max(rect.min, pos2(right - SIZE_WIDTH, rect.bottom()));
    let text = painter.with_clip_rect(names);
    let mut name_left = left;
    if row.needs_administrator {
        let lock = Rect::from_center_size(
            pos2(left + LOCK_SIZE / 2.0, rect.center().y - 8.0),
            Vec2::splat(LOCK_SIZE),
        );
        Icon::Lock.paint(&text, lock, tokens.color_warn);
        name_left += LOCK_SIZE + SPACE_1 + 2.0;
    }
    text.text(
        pos2(name_left, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        &row.name,
        FontId::new(FONT_SIZE_DATA, FontFamily::Proportional),
        tokens.color_text_primary,
    );
    let small = FontId::new(FONT_SIZE_SMALL, FontFamily::Proportional);
    let detail = text.text(
        pos2(left, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        &row.detail,
        small.clone(),
        tokens.color_text_muted,
    );
    if row.is_running {
        text.text(
            pos2(detail.right() + SPACE_1 + 2.0, rect.center().y + 9.0),
            Align2::LEFT_CENTER,
            format!("· {}", pt_br::APPS_OPEN.to_lowercase()),
            small,
            tokens.color_alert,
        );
    }

    response.widget_info(|| WidgetInfo::selected(Role::Button, true, row.is_chosen, &row.name));
    response
}

/// The column about the chosen app: what it is and what belongs to it.
fn detail(model: &mut AppsViewModel, ui: &mut Ui, context: &ScreenContext<'_>, height: f32) {
    let tokens = context.tokens;
    ScrollArea::vertical()
        .id_salt("apps_detail")
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let Some(app) = model.chosen_app().cloned() else {
                HudPanel::new(pt_br::APPS_CHOOSE_TITLE).show(ui, tokens, |ui| {
                    ui.label(
                        RichText::new(pt_br::APPS_CHOOSE_BODY).color(tokens.color_text_secondary),
                    );
                });
                return;
            };
            app_facts(model, ui, tokens, &app);
            ui.add_space(SPACE_3);

            if model.is_planning() {
                HudPanel::new(pt_br::APPS_PLANNING).show(ui, tokens, |ui| {
                    let is_moving = context.appearance.motion.allows_ambient();
                    activity_bar(ui, tokens, context.now, is_moving);
                });
                return;
            }
            if let Some(failure) = model.plan_failure().map(str::to_owned) {
                HudPanel::new(pt_br::APPS_PLAN_FAILED).show(ui, tokens, |ui| {
                    ui.label(RichText::new(failure).color(tokens.color_alert));
                    ui.add_space(SPACE_3);
                    if hud_button(ui, tokens, pt_br::TRY_AGAIN, ButtonKind::Outline).clicked() {
                        model.choose(ui.ctx(), context.services, &app.path);
                    }
                });
                return;
            }

            let home = &context.services.home;
            let user_rows = item_rows(model, &app, PrivilegeScope::User, home);
            let system_rows = item_rows(model, &app, PrivilegeScope::Administrator, home);
            group(
                ui,
                tokens,
                model,
                pt_br::CLEANUP_USER_GROUP,
                pt_br::APPS_USER_META,
                &user_rows,
            );
            let system_meta = if model.removes_system() {
                pt_br::apps_admin_meta(model.authentication())
            } else {
                pt_br::CLEANUP_ADMIN_UNAVAILABLE
            };
            if !user_rows.is_empty() && !system_rows.is_empty() {
                ui.add_space(SPACE_3);
            }
            group(
                ui,
                tokens,
                model,
                pt_br::CLEANUP_ADMIN_GROUP,
                system_meta,
                &system_rows,
            );
        });
}

/// The panel that says which app this is.
fn app_facts(model: &AppsViewModel, ui: &mut Ui, tokens: &Tokens, app: &InstalledApp) {
    let origin = if app.is_running {
        pt_br::APPS_OPEN_NOW
    } else {
        match app.scope {
            PrivilegeScope::User => pt_br::APPS_OWNER_USER,
            PrivilegeScope::Administrator => pt_br::APPS_OWNER_SYSTEM,
        }
    };
    HudPanel::new(&app.name)
        .meta(origin)
        .show(ui, tokens, |ui| {
            key_value(ui, tokens, pt_br::APPS_IDENTIFIER, &app.bundle_id);
            let version = app.version.as_deref().unwrap_or("—");
            key_value(ui, tokens, pt_br::APPS_VERSION_LABEL, version);
            let Some(plan) = model.plan() else { return };
            let found = format!(
                "{} {} · {}",
                plan.items.len(),
                pt_br::ITEMS,
                plan.reclaimable()
            );
            key_value(ui, tokens, pt_br::APPS_FOUND, &found);
            if plan.has_other_copy {
                ui.add_space(SPACE_1);
                ui.label(
                    RichText::new(pt_br::APPS_OTHER_COPY)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_warn),
                );
            }
        });
}

fn item_rows(
    model: &AppsViewModel,
    app: &InstalledApp,
    scope: PrivilegeScope,
    home: &Path,
) -> Vec<ItemRow> {
    model
        .plan()
        .iter()
        .flat_map(|plan| &plan.items)
        .filter(|item| item.scope == scope)
        .map(|item| {
            let target = &item.request.target;
            let name = target.file_name().map_or_else(
                || target.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            let folder = target
                .parent()
                .map_or_else(String::new, |parent| display_path(parent, home));
            let detail = format!("{} · {folder}", pt_br::leftover_kind(item.kind));
            ItemRow {
                target: target.clone(),
                title: shorten_middle(&name, NAME_CHARACTERS),
                detail: shorten_middle(&detail, DETAIL_CHARACTERS),
                risk: item.risk,
                size: item.request.reclaimable.to_string(),
                is_enabled: model.is_selectable(item, app),
                is_checked: model.is_selected(target),
            }
        })
        .collect()
}

fn group(
    ui: &mut Ui,
    tokens: &Tokens,
    model: &mut AppsViewModel,
    title: &str,
    meta: &str,
    rows: &[ItemRow],
) {
    if rows.is_empty() {
        return;
    }
    HudPanel::new(title).meta(meta).show(ui, tokens, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in rows {
            let clicked = selection_row(
                ui,
                tokens,
                &SelectionRow {
                    is_checked: row.is_checked,
                    is_enabled: row.is_enabled,
                    title: &row.title,
                    detail: &row.detail,
                    badge: Some(risk_badge(tokens, row.risk)),
                    ratio: None,
                    trailing: &row.size,
                },
            )
            .clicked();
            if clicked {
                model.toggle(&row.target);
            }
        }
    });
}

fn confirm_lines(selection: &Selection, authentication: AuthenticationKind) -> Vec<String> {
    let mut lines = vec![format!(
        "{} {} · {}",
        selection.items,
        pt_br::ITEMS,
        selection.reclaimable
    )];
    for (applies, line) in [
        (selection.uses_trash, pt_br::APPS_CONFIRM_TRASH),
        (
            selection.needs_administrator,
            pt_br::apps_confirm_admin(authentication),
        ),
        (selection.has_review, pt_br::APPS_CONFIRM_REVIEW),
    ] {
        if applies {
            lines.push(line.to_owned());
        }
    }
    lines
}
