//! Drawing of the disk map screen.

use std::path::PathBuf;

use diskmarshal_domain::scan::{CloudState, FileCategory, FileNode};
use diskmarshal_domain::treemap::{LayoutRect, squarify};
use egui::{
    Align, Align2, Color32, FontFamily, FontId, Layout, Painter, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};

use super::view_model::{ColorMode, DiskMapViewModel, MapBlock};
use crate::animation::ease_out_cubic;
use crate::components::{ButtonKind, HudPanel, activity_bar, chip, hud_button};
use crate::effects::glow::{self, TextStyle};
use crate::i18n::pt_br;
use crate::screens::{ScreenContext, display_path, shorten_middle};
use crate::theme::{
    FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_1, SPACE_2, SPACE_3, SPACE_4, Tokens, display,
};

/// Most folders of the trail shown at once: the scanned one and the last ones.
const MAX_CRUMBS: usize = 4;
/// Longest text of the scanned folder in the trail.
const ROOT_CRUMB_CHARS: usize = 40;
const ZOOM_SECONDS: f64 = 0.45;
const LEGEND_HEIGHT: f32 = 22.0;
const BLOCK_FILL_OPACITY: f32 = 0.14;
const BLOCK_HOVER_FILL_OPACITY: f32 = 0.3;
const BLOCK_LABEL_FONT_SIZE: f32 = 13.0;
/// A block narrower or shorter than this gets no label.
const MIN_LABEL_SIZE: (f32, f32) = (56.0, 22.0);
/// A block shorter than this shows only its name, without the size.
const MIN_HEIGHT_FOR_SIZE: f32 = 44.0;
const CARD_SIZE: (f32, f32) = (230.0, 104.0);
const CARD_LINE_HEIGHT: f32 = 16.0;

/// Draws the screen for this frame.
pub(crate) fn show(model: &mut DiskMapViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    model.poll();
    if let Some(progress) = model.progress() {
        scanning(
            model,
            ui,
            context,
            progress.items_seen,
            &progress.bytes_seen.to_string(),
        );
    } else if model.report().is_some() {
        toolbar(model, ui, context);
        ui.add_space(SPACE_2);
        map(model, ui, context);
        footer(model, ui, context);
    } else {
        introduction(model, ui, context);
    }
}

fn start(model: &mut DiskMapViewModel, ui: &Ui, context: &ScreenContext<'_>, root: PathBuf) {
    model.start_scan(ui.ctx(), context.services.walker.clone(), root);
}

/// What the user sees before the first scan: a choice of what to measure.
fn introduction(model: &mut DiskMapViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::MAP_INTRO_TITLE).show(ui, tokens, |ui| {
        ui.label(RichText::new(pt_br::MAP_INTRO_BODY).color(tokens.color_text_secondary));
        if let Some(failure) = model.failure() {
            ui.add_space(SPACE_2);
            ui.label(RichText::new(failure).color(tokens.color_alert));
        }
        ui.add_space(SPACE_3);
        ui.horizontal(|ui| {
            if hud_button(ui, tokens, pt_br::MAP_SCAN_HOME, ButtonKind::Primary).clicked() {
                start(model, ui, context, context.services.home.clone());
            }
            ui.add_space(SPACE_2);
            if hud_button(ui, tokens, pt_br::MAP_SCAN_DISK, ButtonKind::Outline).clicked() {
                start(model, ui, context, PathBuf::from("/"));
            }
        });
        ui.add_space(SPACE_2);
        ui.label(
            RichText::new(pt_br::MAP_INTRO_NOTE)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
    });
}

fn scanning(
    model: &mut DiskMapViewModel,
    ui: &mut Ui,
    context: &ScreenContext<'_>,
    items: u64,
    bytes: &str,
) {
    let tokens = context.tokens;
    HudPanel::new(pt_br::MAP_SCANNING_TITLE)
        .meta(pt_br::MAP_SCANNING_META)
        .show(ui, tokens, |ui| {
            let counted = format!("{} · {bytes}", pt_br::ITEMS.of(items));
            ui.label(
                RichText::new(counted)
                    .size(FONT_SIZE_DATA + 5.0)
                    .color(tokens.color_text_primary),
            );
            ui.add_space(SPACE_2);
            // The total is unknown while scanning, so the bar shows activity, not progress.
            activity_bar(
                ui,
                tokens,
                context.now,
                context.appearance.motion.allows_ambient(),
            );
            ui.add_space(SPACE_3);
            if hud_button(ui, tokens, pt_br::CANCEL, ButtonKind::Quiet).clicked() {
                model.cancel_scan();
            }
        });
}

/// The row above the map: where the user is, the color mode and a new scan.
fn toolbar(model: &mut DiskMapViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_1 + 2.0;
        // The scanned folder is written the way people read it, and a deep
        // trail shows its two ends, so the row never outgrows the window.
        let mut crumbs = model.crumbs();
        let scanned = model
            .report()
            .map(|report| display_path(&report.root, &context.services.home));
        if let (Some(first), Some(scanned)) = (crumbs.first_mut(), scanned) {
            *first = shorten_middle(&scanned, ROOT_CRUMB_CHARS);
        }
        let last = crumbs.len().saturating_sub(1);
        let hidden = crumbs.len().saturating_sub(MAX_CRUMBS);
        for (depth, name) in crumbs.iter().enumerate() {
            if (1..=hidden).contains(&depth) {
                if depth == 1 {
                    ui.label(RichText::new("… ›").color(tokens.color_text_muted));
                }
                continue;
            }
            if depth == last {
                ui.label(RichText::new(name).color(tokens.color_text_primary));
            } else {
                let link = ui.add(
                    egui::Label::new(RichText::new(name).color(tokens.color_accent))
                        .sense(Sense::click()),
                );
                if link.clicked() {
                    model.go_to_depth(depth);
                }
                ui.label(RichText::new("›").color(tokens.color_text_muted));
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = SPACE_2;
            let root = model.report().map(|report| report.root.clone());
            if hud_button(ui, tokens, pt_br::MAP_RESCAN, ButtonKind::Outline).clicked()
                && let Some(root) = root
            {
                start(model, ui, context, root);
            }
            for (mode, label) in [
                (ColorMode::Cloud, pt_br::MAP_COLOR_CLOUD),
                (ColorMode::Category, pt_br::MAP_COLOR_TYPE),
            ] {
                if chip(ui, tokens, label, model.color_mode == mode, None).clicked() {
                    model.color_mode = mode;
                }
            }
            ui.label(
                RichText::new(pt_br::MAP_COLOR_BY.to_uppercase())
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_text_muted),
            );
        });
    });
}

fn map(model: &mut DiskMapViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    let footer_height = LEGEND_HEIGHT * 2.0 + SPACE_2;
    let size = vec2(
        ui.available_width(),
        (ui.available_height() - footer_height).max(120.0),
    );
    let (area, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 0.0, tokens.color_surface_window);
    painter.rect_stroke(
        area,
        0.0,
        Stroke::new(1.0, tokens.color_line_strong),
        StrokeKind::Inside,
    );

    let blocks = model.blocks();
    if blocks.is_empty() {
        painter.text(
            area.center(),
            Align2::CENTER_CENTER,
            pt_br::MAP_EMPTY_FOLDER,
            FontId::new(FONT_SIZE_DATA, FontFamily::Proportional),
            tokens.color_text_muted,
        );
        return;
    }

    let weights: Vec<f64> = blocks.iter().map(|block| block.weight).collect();
    let unit = LayoutRect::new(0.0, 0.0, 1.0, 1.0);
    let layout = squarify(&weights, unit);
    let growth = model.zoom().map(|zoom| {
        let progress = if context.appearance.motion.allows_transitions() {
            ease_out_cubic(zoom.timeline.progress(context.now, ZOOM_SECONDS))
        } else {
            1.0
        };
        (zoom.origin, progress)
    });
    let rects: Vec<Rect> = layout
        .iter()
        .map(|rect| to_screen(grown(*rect, growth), area))
        .collect();

    let pointer = response.hover_pos();
    let hovered = pointer.and_then(|pointer| rects.iter().position(|rect| rect.contains(pointer)));
    for (index, (block, rect)) in blocks.iter().zip(&rects).enumerate() {
        paint_block(
            &painter,
            tokens,
            model.color_mode,
            block,
            *rect,
            hovered == Some(index),
        );
    }

    if let (Some(pointer), Some(index)) = (pointer, hovered)
        && let Some(block) = blocks.get(index)
    {
        let crosshair = Stroke::new(1.0, tokens.color_accent.gamma_multiply(0.55));
        painter.hline(area.x_range(), pointer.y, crosshair);
        painter.vline(pointer.x, area.y_range(), crosshair);
        let share = block.weight / weights.iter().sum::<f64>();
        paint_card(&painter, tokens, block, share, pointer, area);

        if response.clicked()
            && block.is_enterable
            && let (Some(node), Some(origin)) = (block.node, layout.get(index))
        {
            model.enter(node, *origin, context.now);
        }
        if response.secondary_clicked()
            && let Some(path) = block.node.and_then(|node| model.path_of(node))
        {
            // Nothing useful can be shown if Finder refuses; the map stays as it is.
            let _ = context.services.opener.reveal_in_finder(&path);
        }
    }
}

/// While a zoom runs, shrinks a rectangle toward the block the map grows from.
fn grown(rect: LayoutRect, growth: Option<(LayoutRect, f32)>) -> LayoutRect {
    let Some((origin, progress)) = growth else {
        return rect;
    };
    let progress = f64::from(progress);
    let start = LayoutRect::new(
        origin.x + rect.x * origin.width,
        origin.y + rect.y * origin.height,
        rect.width * origin.width,
        rect.height * origin.height,
    );
    let mix = |from: f64, to: f64| from + (to - from) * progress;
    LayoutRect::new(
        mix(start.x, rect.x),
        mix(start.y, rect.y),
        mix(start.width, rect.width),
        mix(start.height, rect.height),
    )
}

/// Maps a rectangle in 0.0..=1.0 coordinates onto the map area, with a
/// one-point gap between neighbors.
fn to_screen(rect: LayoutRect, area: Rect) -> Rect {
    let min = pos2(
        area.left() + rect.x as f32 * area.width(),
        area.top() + rect.y as f32 * area.height(),
    );
    let size = vec2(
        rect.width as f32 * area.width(),
        rect.height as f32 * area.height(),
    );
    Rect::from_min_size(min, size).shrink(1.0)
}

fn block_color(tokens: &Tokens, mode: ColorMode, block: &MapBlock) -> Color32 {
    match mode {
        ColorMode::Category => category_color(tokens, block.category),
        ColorMode::Cloud => cloud_color(tokens, block.cloud),
    }
}

fn category_color(tokens: &Tokens, category: FileCategory) -> Color32 {
    match category {
        FileCategory::System => tokens.color_category_system,
        FileCategory::Code => tokens.color_category_code,
        FileCategory::Photo => tokens.color_category_photo,
        FileCategory::Media => tokens.color_category_media,
        FileCategory::Document => tokens.color_category_document,
        FileCategory::Archive => tokens.color_category_archive,
        FileCategory::Other => tokens.color_text_muted,
    }
}

fn cloud_color(tokens: &Tokens, state: CloudState) -> Color32 {
    match state {
        CloudState::Local => tokens.color_category_system,
        CloudState::Synced => tokens.color_category_document,
        CloudState::CloudOnly => tokens.color_category_archive,
    }
}

fn paint_block(
    painter: &Painter,
    tokens: &Tokens,
    mode: ColorMode,
    block: &MapBlock,
    rect: Rect,
    is_hovered: bool,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let color = block_color(tokens, mode, block);
    let opacity = if is_hovered {
        BLOCK_HOVER_FILL_OPACITY
    } else {
        BLOCK_FILL_OPACITY
    };
    painter.rect_filled(rect, 0.0, color.gamma_multiply(opacity));
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, color), StrokeKind::Inside);
    if rect.width() < MIN_LABEL_SIZE.0 || rect.height() < MIN_LABEL_SIZE.1 {
        return;
    }

    let text_color = color.lerp_to_gamma(Color32::WHITE, 0.3);
    let clipped = painter.with_clip_rect(rect.shrink(SPACE_1));
    let name = glow::text(
        &clipped,
        rect.min + vec2(SPACE_2, SPACE_1 + 2.0),
        Align2::LEFT_TOP,
        &block.name.to_uppercase(),
        &TextStyle {
            font: FontId::new(BLOCK_LABEL_FONT_SIZE, display()),
            color: text_color,
            letter_spacing: 1.0,
            glow: 0.0,
        },
    );
    if rect.height() >= MIN_HEIGHT_FOR_SIZE {
        let size = match mode {
            ColorMode::Category => block.allocated,
            ColorMode::Cloud => block.logical,
        };
        clipped.text(
            pos2(name.left(), name.bottom() + 2.0),
            Align2::LEFT_TOP,
            size.to_string(),
            FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace),
            text_color,
        );
    }
}

/// The data card that follows the pointer.
fn paint_card(
    painter: &Painter,
    tokens: &Tokens,
    block: &MapBlock,
    share: f64,
    pointer: Pos2,
    area: Rect,
) {
    let size = vec2(CARD_SIZE.0, CARD_SIZE.1);
    let min = pos2(
        (pointer.x + SPACE_4)
            .min(area.right() - size.x - SPACE_1)
            .max(area.left() + SPACE_1),
        (pointer.y + SPACE_4)
            .min(area.bottom() - size.y - SPACE_1)
            .max(area.top() + SPACE_1),
    );
    let card = Rect::from_min_size(min, size);
    painter.rect_filled(card, 0.0, tokens.color_surface_window);
    painter.rect_stroke(
        card,
        0.0,
        Stroke::new(1.0, tokens.color_accent),
        StrokeKind::Inside,
    );

    let left = card.left() + SPACE_2 + 2.0;
    let clipped = painter.with_clip_rect(card.shrink(SPACE_1));
    let title = glow::text(
        &clipped,
        pos2(left, card.top() + SPACE_2),
        Align2::LEFT_TOP,
        &block.name.to_uppercase(),
        &TextStyle {
            font: FontId::new(FONT_SIZE_DATA + 1.0, display()),
            color: tokens.color_accent,
            letter_spacing: 1.2,
            glow: 0.0,
        },
    );

    let on_disk = format!("{} · {:.0}%", block.allocated, share * 100.0);
    let hint = match (block.node, block.is_enterable) {
        (None, _) => "",
        (Some(_), true) => pt_br::MAP_CARD_ENTER,
        (Some(_), false) => pt_br::MAP_CARD_REVEAL,
    };
    let rows = [
        (pt_br::MAP_CARD_SIZE, on_disk),
        (pt_br::MAP_CARD_ITEMS, pt_br::group_digits(block.item_count)),
        (
            pt_br::MAP_CARD_CLOUD,
            pt_br::cloud_state(block.cloud).to_owned(),
        ),
        (
            pt_br::MAP_CARD_TYPE,
            pt_br::category(block.category).to_owned(),
        ),
    ];
    let font = FontId::new(FONT_SIZE_SMALL, FontFamily::Monospace);
    let mut top = title.bottom() + SPACE_1;
    for (label, value) in rows {
        clipped.text(
            pos2(left, top),
            Align2::LEFT_TOP,
            label,
            font.clone(),
            tokens.color_text_muted,
        );
        clipped.text(
            pos2(left + 64.0, top),
            Align2::LEFT_TOP,
            value,
            font.clone(),
            tokens.color_text_primary,
        );
        top += CARD_LINE_HEIGHT;
    }
    clipped.text(
        pos2(left, top),
        Align2::LEFT_TOP,
        hint,
        font,
        tokens.color_text_muted,
    );
}

/// The legend and, when macOS blocked some folders, how to fix that.
fn footer(model: &DiskMapViewModel, ui: &mut Ui, context: &ScreenContext<'_>) {
    let tokens = context.tokens;
    ui.add_space(SPACE_2);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_3;
        let entries: Vec<(Color32, &str)> = match model.color_mode {
            ColorMode::Category => FileCategory::ALL
                .into_iter()
                .map(|category| (category_color(tokens, category), pt_br::category(category)))
                .collect(),
            ColorMode::Cloud => [CloudState::Local, CloudState::Synced, CloudState::CloudOnly]
                .into_iter()
                .map(|state| (cloud_color(tokens, state), pt_br::cloud_state(state)))
                .collect(),
        };
        for (color, label) in entries {
            let (swatch, _) = ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
            ui.painter()
                .rect_filled(swatch, 0.0, color.gamma_multiply(0.3));
            ui.painter()
                .rect_stroke(swatch, 0.0, Stroke::new(1.0, color), StrokeKind::Inside);
            ui.label(
                RichText::new(label)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_text_secondary),
            );
        }
    });

    let Some(report) = model.report() else { return };
    let blocked = report.inaccessible_paths.len();
    ui.horizontal(|ui| {
        let root = report.tree.root();
        let seconds = format!("{:.1}", report.elapsed.as_secs_f64()).replace('.', ",");
        let summary = format!(
            "{} · {} · {seconds} s",
            pt_br::ITEMS.of(root.map_or(0, FileNode::item_count)),
            root.map(|node| node.allocated_size().to_string())
                .unwrap_or_default(),
        );
        ui.label(
            RichText::new(summary)
                .size(FONT_SIZE_SMALL)
                .color(tokens.color_text_muted),
        );
        if blocked > 0 {
            let warning = format!("· {}", pt_br::MAP_BLOCKED_FOLDERS.of(blocked));
            ui.label(
                RichText::new(warning)
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_warn),
            );
            let link = ui.add(
                egui::Label::new(
                    RichText::new(pt_br::OPEN_FULL_DISK_ACCESS)
                        .size(FONT_SIZE_SMALL)
                        .color(tokens.color_accent),
                )
                .sense(Sense::click()),
            );
            if link.clicked() {
                // If System Settings does not open, the text above still says what is missing.
                let _ = context.services.opener.open_full_disk_access_settings();
            }
        }
    });
}
