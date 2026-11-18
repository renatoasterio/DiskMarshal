//! A round gauge: an arc that fills up to a value, with the value in the middle.

use std::f32::consts::PI;

use egui::{Align2, FontFamily, FontId, Id, Pos2, Sense, Stroke, Ui, Vec2, pos2, vec2};

use crate::animation::ease_out_cubic;
use crate::components::AppearanceSettings;
use crate::effects::glow;
use crate::theme::{FONT_SIZE_SMALL, MOTION_ENTER_SECONDS, Tokens, display_bold};

const GAUGE_SIZE: f32 = 132.0;
const ARC_RADIUS: f32 = 48.0;
const ARC_WIDTH: f32 = 6.0;
const TICK_INNER: f32 = 57.0;
const TICK_OUTER: f32 = 61.0;
/// The arc starts at the lower left and sweeps three quarters of a turn.
const START_ANGLE: f32 = 0.75 * PI;
const SWEEP: f32 = 1.5 * PI;
const ARC_STEPS: usize = 60;
const TICKS: usize = 30;
const VALUE_FONT_SIZE: f32 = 24.0;
/// How long the arc takes to fill up the first time the gauge is shown.
const FILL_SECONDS: f32 = MOTION_ENTER_SECONDS * 2.5;

/// What a gauge shows.
pub(crate) struct RadialGauge<'a> {
    /// Tells this gauge apart from the others, for its animation.
    pub name: &'a str,
    /// How full the arc is, from 0.0 to 1.0.
    pub ratio: f32,
    /// The figure in the middle.
    pub value: &'a str,
    /// What the figure measures, under it.
    pub unit: &'a str,
    /// Draws the arc in the warning color.
    pub is_warning: bool,
}

/// Shows the gauge, centered in the space available.
pub(crate) fn radial_gauge(
    ui: &mut Ui,
    tokens: &Tokens,
    appearance: &AppearanceSettings,
    gauge: &RadialGauge<'_>,
) {
    let size = vec2(ui.available_width().max(GAUGE_SIZE), GAUGE_SIZE);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center() + vec2(0.0, 6.0);
    let point = |angle: f32, radius: f32| center + Vec2::angled(angle) * radius;

    for tick in 0..=TICKS {
        let angle = START_ANGLE + SWEEP * tick as f32 / TICKS as f32;
        painter.line_segment(
            [point(angle, TICK_INNER), point(angle, TICK_OUTER)],
            Stroke::new(1.0, tokens.color_line_strong),
        );
    }
    let arc = |share: f32| -> Vec<Pos2> {
        let steps = ((ARC_STEPS as f32 * share).ceil() as usize).max(1);
        (0..=steps)
            .map(|step| {
                point(
                    START_ANGLE + SWEEP * share * step as f32 / steps as f32,
                    ARC_RADIUS,
                )
            })
            .collect()
    };
    painter.line(arc(1.0), Stroke::new(ARC_WIDTH, tokens.color_line));

    // The arc fills up once, the first time the gauge is shown. After that
    // it follows the readings directly: animating toward every new reading
    // would keep the whole window repainting for as long as the app is open.
    let target = gauge.ratio.clamp(0.0, 1.0);
    let shown = if appearance.motion.allows_transitions() {
        let now = ui.input(|input| input.time);
        let id = Id::unique(("radial_gauge", gauge.name));
        let first_shown_at = ui.data_mut(|data| *data.get_temp_mut_or_insert_with(id, || now));
        let progress = ((now - first_shown_at) / f64::from(FILL_SECONDS)).clamp(0.0, 1.0) as f32;
        if progress < 1.0 {
            ui.ctx().request_repaint();
        }
        target * ease_out_cubic(progress)
    } else {
        target
    };
    if shown > 0.0 {
        let color = if gauge.is_warning {
            tokens.color_warn
        } else {
            tokens.color_accent
        };
        glow::path(
            &painter,
            &arc(shown),
            color,
            ARC_WIDTH,
            appearance.glow_strength,
        );
    }

    painter.text(
        pos2(center.x, center.y - 2.0),
        Align2::CENTER_CENTER,
        gauge.value,
        FontId::new(VALUE_FONT_SIZE, display_bold()),
        tokens.color_text_primary,
    );
    painter.text(
        pos2(center.x, center.y + 18.0),
        Align2::CENTER_CENTER,
        gauge.unit.to_uppercase(),
        FontId::new(FONT_SIZE_SMALL - 2.0, FontFamily::Monospace),
        tokens.color_text_secondary,
    );
}
