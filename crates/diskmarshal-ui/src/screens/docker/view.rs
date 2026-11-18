//! Drawing of the Docker screen.

use diskmarshal_domain::cleanup::RiskLevel;
use diskmarshal_domain::docker::{DockerInventory, DockerResource, KindUsage, ResourceKind};
use egui::{Align, Layout, RichText, ScrollArea, Ui};

use super::view_model::{DockerState, DockerViewModel, Selection};
use crate::components::{
    ButtonKind, CellWidth, ConfirmText, HudPanel, RemovalEvent, SelectionRow, action_footer,
    activity_bar, cell_row, chip, hud_button, readout, segmented_bar, selection_row,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, risk_badge, shorten_middle};
use crate::theme::{FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

/// Space kept under the list for the footer.
const FOOTER_HEIGHT: f32 = 52.0;
/// The kinds in the order of the panels at the top: what takes the eye
/// first is what people know best.
const PANEL_ORDER: [ResourceKind; 4] = [
    ResourceKind::Image,
    ResourceKind::Container,
    ResourceKind::Volume,
    ResourceKind::BuildCache,
];
/// Height of the panels at the top, so the four end on the same line.
const USAGE_PANEL_HEIGHT: f32 = 124.0;
const USAGE_BAR_CELLS: usize = 18;
/// Longest name shown in a row; a longer one loses its middle.
const NAME_CHARACTERS: usize = 52;

/// What a row needs, copied out of the inventory so the model can change
/// while drawing.
struct RowData {
    resource: DockerResource,
    title: String,
    detail: String,
    risk: RiskLevel,
    ratio: f32,
    size: String,
    is_checked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut DockerViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.ensure_started(ui.ctx(), context.services);
    model.poll();
    let tokens = context.tokens;

    if model.is_inspecting() {
        HudPanel::new(pt_br::DOCKER_INSPECTING).show(ui, tokens, |ui| {
            ui.label(
                RichText::new(pt_br::DOCKER_INSPECTING_BODY).color(tokens.color_text_secondary),
            );
            ui.add_space(SPACE_2);
            let is_moving = context.appearance.motion.allows_ambient();
            activity_bar(ui, tokens, context.now, is_moving);
        });
        return;
    }
    let notice = match model.state() {
        Some(DockerState::Ready(inventory)) => {
            let inventory = inventory.clone();
            ready(model, ui, context, &inventory);
            return;
        }
        Some(DockerState::NotInstalled) => Notice {
            title: pt_br::DOCKER_MISSING_TITLE,
            meta: None,
            body: pt_br::DOCKER_MISSING_BODY.to_owned(),
            note: pt_br::DOCKER_MISSING_NOTE,
        },
        Some(DockerState::NotRunning) => Notice {
            title: pt_br::DOCKER_STOPPED_TITLE,
            meta: Some(pt_br::DOCKER_STOPPED_META),
            body: pt_br::DOCKER_STOPPED_BODY.to_owned(),
            note: pt_br::DOCKER_STOPPED_NOTE,
        },
        Some(DockerState::Failed(reason)) => Notice {
            title: pt_br::DOCKER_FAILED_TITLE,
            meta: None,
            body: reason.clone(),
            note: pt_br::DOCKER_STOPPED_NOTE,
        },
        None => Notice {
            title: pt_br::DOCKER_FAILED_TITLE,
            meta: None,
            body: model.failure().unwrap_or(pt_br::SCAN_LOST).to_owned(),
            note: pt_br::DOCKER_STOPPED_NOTE,
        },
    };
    if notice.show(ui, tokens) {
        model.inspect(ui.ctx(), context.services);
    }
}

/// What the screen says when Docker has nothing to list.
struct Notice {
    title: &'static str,
    meta: Option<&'static str>,
    body: String,
    note: &'static str,
}

impl Notice {
    /// Shows the notice and returns `true` when the user asks to check again.
    fn show(&self, ui: &mut Ui, tokens: &Tokens) -> bool {
        let panel = HudPanel::new(self.title);
        let panel = match self.meta {
            Some(meta) => panel.meta(meta),
            None => panel,
        };
        panel.show(ui, tokens, |ui| {
            ui.label(RichText::new(&self.body).color(tokens.color_text_secondary));
            ui.add_space(SPACE_2);
            ui.label(
                RichText::new(self.note)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_text_muted),
            );
            ui.add_space(SPACE_3);
            hud_button(ui, tokens, pt_br::DOCKER_CHECK_AGAIN, ButtonKind::Outline).clicked()
        })
    }
}

/// The screen when Docker answered: totals, filter, list and footer.
fn ready(
    model: &mut DockerViewModel,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
    inventory: &DockerInventory,
) {
    let tokens = context.tokens;
    cell_row(ui, &[CellWidth::Share(1.0); 4], |index, ui| {
        let kind = PANEL_ORDER[index];
        usage_panel(ui, tokens, kind, inventory.usage_of(kind));
    });
    ui.add_space(SPACE_3);
    toolbar(model, ui, context, inventory);
    ui.add_space(SPACE_2);

    let rows = rows(model, inventory, context.services.wall_clock.unix_seconds());
    let list_height = (ui.available_height() - FOOTER_HEIGHT).max(120.0);
    ScrollArea::vertical()
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            HudPanel::new(pt_br::DOCKER_LIST_TITLE)
                .meta(pt_br::DOCKER_LIST_META)
                .show(ui, tokens, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if rows.is_empty() {
                        let nothing = if inventory.unused.is_empty() {
                            pt_br::DOCKER_NOTHING_UNUSED
                        } else {
                            pt_br::DOCKER_NOTHING_OF_KIND
                        };
                        ui.label(RichText::new(nothing).color(tokens.color_text_secondary));
                    }
                    for row in &rows {
                        let clicked = selection_row(
                            ui,
                            tokens,
                            &SelectionRow {
                                is_checked: row.is_checked,
                                is_enabled: true,
                                title: &row.title,
                                detail: &row.detail,
                                badge: Some(risk_badge(tokens, row.risk)),
                                ratio: Some(row.ratio),
                                trailing: &row.size,
                            },
                        )
                        .clicked();
                        if clicked {
                            model.toggle(&row.resource);
                        }
                    }
                });
        });

    let selection = model.selection();
    let summary = format!(
        "{} · {} {}",
        pt_br::SELECTED.of(selection.items),
        pt_br::DOCKER_UP_TO,
        selection.size
    );
    if action_footer(ui, tokens, &summary, model.notice(), pt_br::CLEANUP_ACTION) {
        model.request_removal();
    }

    let lines = confirm_lines(&selection);
    let text = ConfirmText {
        body: pt_br::DOCKER_CONFIRM_BODY,
        lines: &lines,
        action: pt_br::CLEANUP_AUTHORIZE,
    };
    match model.removal.show(ui.ctx(), context, &text) {
        RemovalEvent::Confirmed => model.confirm(ui.ctx(), context.services),
        RemovalEvent::Finished => model.inspect(ui.ctx(), context.services),
        RemovalEvent::Idle => {}
    }
}

/// How much Docker holds of one kind, and how much of that nothing uses.
fn usage_panel(ui: &mut Ui, tokens: &Tokens, kind: ResourceKind, usage: Option<&KindUsage>) {
    let (title, noun) = pt_br::docker_kind_plural(kind);
    HudPanel::new(title)
        .min_height(USAGE_PANEL_HEIGHT)
        .show(ui, tokens, |ui| {
            let Some(usage) = usage else {
                readout(ui, tokens, "—", "");
                return;
            };
            readout(ui, tokens, &usage.size.to_string(), "");
            let unused = usage.unused_ratio() as f32;
            segmented_bar(ui, tokens, unused, USAGE_BAR_CELLS, tokens.color_warn);
            ui.add_space(SPACE_2);
            let counted = if kind == ResourceKind::BuildCache {
                noun.of(usage.total_count)
            } else {
                format!(
                    "{} · {} {}",
                    noun.of(usage.total_count),
                    pt_br::group_digits(usage.active_count),
                    pt_br::DOCKER_IN_USE
                )
            };
            let share = format!("{:.0}% {}", unused * 100.0, pt_br::DOCKER_UNUSED);
            for (line, color) in [
                (counted, tokens.color_text_secondary),
                (share, tokens.color_text_muted),
            ] {
                ui.label(RichText::new(line).size(FONT_SIZE_SMALL).color(color));
            }
        });
}

/// The row above the list: the filter by kind, which Docker answered, and
/// a way to ask again.
fn toolbar(
    model: &mut DockerViewModel,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
    inventory: &DockerInventory,
) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        if chip(ui, tokens, pt_br::ALL, model.filter.is_none(), None).clicked() {
            model.filter = None;
        }
        for kind in model.kinds() {
            let is_selected = model.filter == Some(kind);
            if chip(ui, tokens, pt_br::docker_kind(kind), is_selected, None).clicked() {
                model.filter = Some(kind);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if hud_button(ui, tokens, pt_br::DOCKER_MEASURE_AGAIN, ButtonKind::Outline).clicked() {
                model.inspect(ui.ctx(), context.services);
            }
            ui.add_space(SPACE_1);
            let engine = format!(
                "Docker {} · {}",
                inventory.engine_version,
                pt_br::DOCKER_RUNNING
            );
            ui.label(
                RichText::new(engine)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_text_muted),
            );
        });
    });
}

fn rows(
    model: &DockerViewModel,
    inventory: &DockerInventory,
    now_unix_seconds: i64,
) -> Vec<RowData> {
    let largest = model.largest().as_bytes();
    model
        .visible()
        .into_iter()
        .map(|resource| RowData {
            resource: resource.clone(),
            title: title(resource, inventory),
            detail: detail(resource, now_unix_seconds),
            risk: resource.risk(),
            ratio: if largest == 0 {
                0.0
            } else {
                resource.size.as_bytes() as f32 / largest as f32
            },
            size: resource.size.to_string(),
            is_checked: model.is_selected(resource),
        })
        .collect()
}

/// What a row is called: the name Docker gives it, or what stands for one.
fn title(resource: &DockerResource, inventory: &DockerInventory) -> String {
    if resource.kind == ResourceKind::BuildCache {
        let records = inventory
            .usage_of(ResourceKind::BuildCache)
            .map_or(0, |usage| usage.total_count);
        return pt_br::DOCKER_RECORDS.of(records);
    }
    if resource.name.is_empty() {
        return format!("({})", pt_br::DOCKER_UNNAMED);
    }
    shorten_middle(&resource.name, NAME_CHARACTERS)
}

/// The quieter line of a row: what the resource is and why it is listed.
fn detail(resource: &DockerResource, now_unix_seconds: i64) -> String {
    let reason = match resource.kind {
        ResourceKind::BuildCache => pt_br::DOCKER_DETAIL_CACHE,
        ResourceKind::Image if resource.name.is_empty() => pt_br::DOCKER_DETAIL_LEFTOVER,
        ResourceKind::Image => pt_br::DOCKER_DETAIL_UNUSED,
        ResourceKind::Container => pt_br::DOCKER_DETAIL_STOPPED,
        ResourceKind::Volume => pt_br::DOCKER_DETAIL_VOLUME,
    };
    let mut parts = vec![
        pt_br::docker_kind(resource.kind).to_owned(),
        reason.to_owned(),
    ];
    if let Some(created) = resource.created_unix_seconds {
        parts.push(pt_br::age(now_unix_seconds.saturating_sub(created)));
    }
    parts.join(" · ")
}

fn confirm_lines(selection: &Selection) -> Vec<String> {
    let mut lines = vec![
        format!(
            "{} · {} {}",
            pt_br::ITEMS.of(selection.items),
            pt_br::DOCKER_UP_TO,
            selection.size
        ),
        pt_br::DOCKER_CONFIRM_REBUILD.to_owned(),
    ];
    if selection.has_data {
        lines.push(pt_br::DOCKER_CONFIRM_DATA.to_owned());
    }
    lines.push(pt_br::DOCKER_CONFIRM_IN_USE.to_owned());
    lines
}
