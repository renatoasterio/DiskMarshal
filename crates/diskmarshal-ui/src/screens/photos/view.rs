//! Drawing of the photos screen.

use std::path::{Path, PathBuf};

use diskmarshal_domain::photos::SimilarityTolerance;
use egui::{
    Align, Align2, Color32, CursorIcon, FontFamily, FontId, Layout, Rect, Response, RichText, Role,
    ScrollArea, Sense, Stroke, StrokeKind, TextureHandle, Ui, WidgetInfo, pos2, vec2,
};

use super::view_model::{MAX_GROUPS_SHOWN, PhotosViewModel};
use crate::components::{
    ButtonKind, ConfirmText, HudPanel, ProgressPanel, RemovalEvent, action_footer, chip,
    hud_button, progress_panel,
};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, display_path};
use crate::theme::{FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, Tokens};

/// Space kept under the list for the footer.
const FOOTER_HEIGHT: f32 = 52.0;
/// Geometry of the card of one photo: the picture and two lines under it.
const CARD_WIDTH: f32 = 168.0;
const PICTURE_HEIGHT: f32 = 104.0;
const CARD_HEIGHT: f32 = PICTURE_HEIGHT + 44.0;
const BRACKET_SIZE: f32 = 12.0;
const BRACKET_INSET: f32 = 6.0;
const CHECK_SIZE: f32 = 12.0;
/// How much a photo marked for removal is dimmed.
const MARKED_OPACITY: f32 = 0.45;

/// One group, copied out of the report so the model can change while drawing.
struct GroupData {
    title: String,
    meta: String,
    photos: Vec<PhotoData>,
}

struct PhotoData {
    path: PathBuf,
    /// The tag over the picture: the reference, or how alike it is.
    tag: String,
    name: String,
    detail: String,
    is_marked: bool,
}

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut PhotosViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.poll(ui.ctx());
    let tokens = context.tokens;

    if let Some(progress) = model.progress() {
        let panel = ProgressPanel {
            title: pt_br::PHOTOS_SEARCHING,
            meta: Some(pt_br::PHOTOS_SEARCHING_META),
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
    let groups = groups(model, &root, &context.services.home);
    let (total, compared) = model.report().map_or((0, 0), |report| {
        (report.groups.len(), report.photos_compared)
    });
    let hidden = total.saturating_sub(groups.len());
    let list_height = (ui.available_height() - FOOTER_HEIGHT).max(120.0);
    ScrollArea::vertical()
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if groups.is_empty() {
                HudPanel::new(pt_br::PHOTOS_NONE_TITLE).show(ui, tokens, |ui| {
                    let body = format!(
                        "{} {}",
                        pt_br::PHOTOS_COMPARED.of(compared),
                        pt_br::PHOTOS_NONE_BODY
                    );
                    ui.label(RichText::new(body).color(tokens.color_text_secondary));
                });
            }
            for group in &groups {
                HudPanel::new(&group.title)
                    .meta(&group.meta)
                    .show(ui, tokens, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(SPACE_3, SPACE_3);
                            for photo in &group.photos {
                                let picture = model.picture(&photo.path).cloned();
                                if photo_card(ui, tokens, photo, picture.as_ref()).clicked() {
                                    model.review.toggle(&photo.path, pt_br::PHOTOS_KEEP_ONE);
                                }
                            }
                        });
                    });
                ui.add_space(SPACE_2);
            }
            if hidden > 0 {
                let note = format!(
                    "{} {}",
                    pt_br::DUP_MORE_PREFIX,
                    pt_br::PHOTOS_MORE_SUFFIX.of(hidden)
                );
                ui.label(
                    RichText::new(note)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_text_muted),
                );
            }
        });

    let (count, size) = model.review.selection();
    let summary = format!("{} · {size}", pt_br::PHOTOS_MARKED.of(count));
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
        format!("{} · {size}", pt_br::PHOTOS_COUNT.of(count)),
        pt_br::PHOTOS_CONFIRM_KEEP.to_owned(),
        pt_br::TRASH_FREES_LATER.to_owned(),
    ];
    let text = ConfirmText {
        body: pt_br::PHOTOS_CONFIRM_BODY,
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

/// What the user sees before the first search: where to look and how alike
/// two photos must be.
fn introduction(model: &mut PhotosViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::TITLE_PHOTOS).show(ui, tokens, |ui| {
        ui.label(RichText::new(pt_br::PHOTOS_INTRO_BODY).color(tokens.color_text_secondary));
        if let Some(failure) = model.failure() {
            ui.add_space(SPACE_2);
            ui.label(RichText::new(failure).color(tokens.color_alert));
        }
        ui.add_space(SPACE_3);
        tolerance(model, ui, tokens);
        ui.add_space(SPACE_3);
        ui.horizontal(|ui| {
            if hud_button(ui, tokens, pt_br::CHOOSE_FOLDER, ButtonKind::Primary).clicked()
                && let Some(folder) = context.services.folder_picker.pick_folder()
            {
                model.start(ui.ctx(), context.services, folder);
            }
            ui.add_space(SPACE_2);
            if hud_button(
                ui,
                tokens,
                pt_br::PHOTOS_SEARCH_PICTURES,
                ButtonKind::Outline,
            )
            .clicked()
            {
                let pictures = context.services.home.join("Pictures");
                model.start(ui.ctx(), context.services, pictures);
            }
        });
        ui.add_space(SPACE_2);
        ui.label(
            RichText::new(pt_br::PHOTOS_INTRO_NOTE)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
    });
}

fn tolerance(model: &mut PhotosViewModel, ui: &mut Ui, tokens: &Tokens) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_2;
        ui.label(
            RichText::new(pt_br::PHOTOS_TOLERANCE.to_uppercase())
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
        for option in SimilarityTolerance::ALL {
            let label = pt_br::similarity_tolerance(option);
            if chip(ui, tokens, label, model.tolerance == option, None).clicked() {
                model.tolerance = option;
            }
        }
    });
}

/// The row above the list: how alike photos must be, and a new search.
fn toolbar(model: &mut PhotosViewModel, ui: &mut Ui, context: &ScreenContext<'_>, root: &Path) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        tolerance(model, ui, tokens);
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

fn groups(model: &PhotosViewModel, root: &Path, home: &Path) -> Vec<GroupData> {
    let Some(report) = model.report() else {
        return Vec::new();
    };
    report
        .groups
        .iter()
        .take(MAX_GROUPS_SHOWN)
        .map(|found| GroupData {
            title: pt_br::PHOTOS_GROUP.of(found.group.photos.len()),
            meta: format!("{} {}", pt_br::PHOTOS_FREES, found.group.reclaimable()),
            photos: found
                .group
                .photos
                .iter()
                .zip(0..)
                .map(|(photo, position)| PhotoData {
                    path: photo.path.clone(),
                    tag: if position == 0 {
                        pt_br::PHOTOS_REFERENCE.to_owned()
                    } else {
                        format!("{} {:.0}%", pt_br::PHOTOS_MATCH, photo.similarity * 100.0)
                    },
                    name: photo
                        .path
                        .file_name()
                        .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                    detail: format!(
                        "{} · {}",
                        photo.logical_size,
                        folder_of(&photo.path, root, home)
                    ),
                    is_marked: model.review.is_marked(&photo.path),
                })
                .collect(),
        })
        .collect()
}

/// The folder a photo is in, written relative to the searched folder.
fn folder_of(path: &Path, root: &Path, home: &Path) -> String {
    let Some(folder) = path.parent() else {
        return String::new();
    };
    match folder.strip_prefix(root) {
        Ok(relative) if relative.as_os_str().is_empty() => "./".to_owned(),
        Ok(relative) => relative.display().to_string(),
        Err(_) => display_path(folder, home),
    }
}

/// One photo: its picture inside target brackets, how alike it is, where it
/// is, and whether it is marked for removal.
fn photo_card(
    ui: &mut Ui,
    tokens: &Tokens,
    photo: &PhotoData,
    picture: Option<&TextureHandle>,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD_WIDTH, CARD_HEIGHT), Sense::click());
    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(&photo.name);
    response
        .widget_info(|| WidgetInfo::selected(Role::CheckBox, true, photo.is_marked, &photo.name));
    let painter = ui.painter_at(rect);
    let frame = Rect::from_min_size(rect.min, vec2(CARD_WIDTH, PICTURE_HEIGHT));
    let is_lit = response.hovered() || response.has_focus();
    let accent = if photo.is_marked {
        tokens.color_alert
    } else {
        tokens.color_accent
    };

    painter.rect_filled(frame, 0.0, tokens.color_surface_void);
    match picture {
        Some(picture) => {
            let space = frame.shrink(1.0);
            let scale =
                (space.width() / picture.size_vec2().x).min(space.height() / picture.size_vec2().y);
            let fitted = Rect::from_center_size(space.center(), picture.size_vec2() * scale);
            let whole = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            let opacity = if photo.is_marked { MARKED_OPACITY } else { 1.0 };
            painter.image(
                picture.id(),
                fitted,
                whole,
                Color32::WHITE.gamma_multiply(opacity),
            );
        }
        None => {
            painter.text(
                frame.center(),
                Align2::CENTER_CENTER,
                pt_br::PHOTOS_NO_PICTURE,
                FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace),
                tokens.color_text_muted,
            );
        }
    }
    let border = if is_lit || photo.is_marked {
        accent
    } else {
        tokens.color_line
    };
    painter.rect_stroke(frame, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    brackets(
        &painter,
        frame.shrink(BRACKET_INSET),
        Stroke::new(2.0, accent),
    );

    let small = FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace);
    let tag = painter.layout_no_wrap(photo.tag.clone(), small.clone(), tokens.color_text_primary);
    let tag_rect = Rect::from_min_size(
        frame.left_bottom() + vec2(BRACKET_INSET, -BRACKET_INSET - tag.size().y - 2.0),
        tag.size() + vec2(SPACE_2, 2.0),
    );
    painter.rect_filled(
        tag_rect,
        0.0,
        tokens.color_surface_void.gamma_multiply(0.85),
    );
    painter.galley(
        tag_rect.min + vec2(SPACE_1, 1.0),
        tag,
        tokens.color_text_primary,
    );

    let detail_top = frame.bottom() + SPACE_1 + 1.0;
    let detail = painter.layout_no_wrap(
        photo.detail.clone(),
        small.clone(),
        tokens.color_text_secondary,
    );
    painter.with_clip_rect(rect).galley(
        pos2(rect.left(), detail_top),
        detail,
        tokens.color_text_secondary,
    );

    let check_top = detail_top + FONT_SIZE_SMALL + SPACE_2;
    let check = Rect::from_min_size(pos2(rect.left(), check_top), vec2(CHECK_SIZE, CHECK_SIZE));
    painter.rect_stroke(check, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    if photo.is_marked {
        painter.rect_filled(check.shrink(3.0), 0.0, accent);
    }
    let (state, state_color) = if photo.is_marked {
        (pt_br::PHOTOS_REMOVE, tokens.color_alert)
    } else {
        (pt_br::DUP_KEEP, tokens.color_ok)
    };
    painter.text(
        pos2(check.right() + SPACE_2, check.center().y),
        Align2::LEFT_CENTER,
        state,
        small,
        state_color,
    );
    response
}

/// The corner brackets that frame a picture like a target.
fn brackets(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let (min, max) = (rect.min, rect.max);
    painter.line(
        vec![
            pos2(min.x, min.y + BRACKET_SIZE),
            min,
            pos2(min.x + BRACKET_SIZE, min.y),
        ],
        stroke,
    );
    painter.line(
        vec![
            pos2(max.x - BRACKET_SIZE, max.y),
            max,
            pos2(max.x, max.y - BRACKET_SIZE),
        ],
        stroke,
    );
}
