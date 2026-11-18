//! A radar dial: rings, cross hairs and a sweep that goes around.

use std::f32::consts::TAU;

use egui::{Mesh, Sense, Shape, Stroke, Ui, Vec2};

use crate::theme::Tokens;

const RINGS: usize = 3;
/// How much of the turn the fading sweep covers.
const SWEEP_SPAN: f32 = TAU * 0.28;
const SWEEP_STEPS: u32 = 24;
const SWEEP_OPACITY: f32 = 0.5;
const RING_OPACITY: f32 = 0.25;
/// Where the fixed blips sit, as (angle in turns, distance from the center).
const BLIPS: [(f32, f32); 3] = [(0.92, 0.55), (0.42, 0.45), (0.2, 0.62)];

/// Shows the dial with the given diameter. `sweep_turns` places the leading
/// edge of the sweep, in turns; `None` draws the dial without a sweep.
pub(crate) fn radar(ui: &mut Ui, tokens: &Tokens, diameter: f32, sweep_turns: Option<f32>) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = diameter / 2.0 - 1.0;
    let faint = Stroke::new(1.0, tokens.color_accent.gamma_multiply(RING_OPACITY));

    for ring in 1..=RINGS {
        painter.circle_stroke(center, radius * ring as f32 / RINGS as f32, faint);
    }
    painter.hline(rect.x_range(), center.y, faint);
    painter.vline(center.x, rect.y_range(), faint);
    painter.circle_stroke(center, radius, Stroke::new(1.0, tokens.color_line_strong));

    if let Some(turns) = sweep_turns {
        let lead = turns * TAU;
        let mut mesh = Mesh::default();
        mesh.colored_vertex(
            center,
            tokens.color_accent.gamma_multiply(SWEEP_OPACITY * 0.5),
        );
        for step in 0..=SWEEP_STEPS {
            let share = step as f32 / SWEEP_STEPS as f32;
            let angle = lead - SWEEP_SPAN * share;
            let color = tokens
                .color_accent
                .gamma_multiply(SWEEP_OPACITY * (1.0 - share));
            mesh.colored_vertex(center + Vec2::angled(angle) * radius, color);
        }
        for step in 1..=SWEEP_STEPS {
            mesh.add_triangle(0, step, step + 1);
        }
        painter.add(Shape::mesh(mesh));
    }
    for (turn, distance) in BLIPS {
        let position = center + Vec2::angled(turn * TAU) * radius * distance;
        painter.circle_filled(position, 2.5, tokens.color_accent);
    }
}
