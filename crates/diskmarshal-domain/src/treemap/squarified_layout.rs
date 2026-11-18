//! The squarified treemap algorithm (Bruls, Huizing and van Wijk).
//!
//! It places the largest items first, in rows along the shorter side of the
//! remaining space, and closes a row as soon as adding one more item would
//! make its rectangles less square.

use crate::treemap::LayoutRect;

/// Divides `bounds` into one rectangle per weight, each with an area
/// proportional to its weight.
///
/// The result has the same length and order as `weights`. Weights that are
/// zero, negative or not finite get an empty rectangle.
#[must_use]
pub fn squarify(weights: &[f64], bounds: LayoutRect) -> Vec<LayoutRect> {
    let mut rects = vec![LayoutRect::default(); weights.len()];
    let mut order: Vec<usize> = (0..weights.len())
        .filter(|index| weights.get(*index).is_some_and(|weight| is_usable(*weight)))
        .collect();
    order.sort_by(|a, b| weight_at(weights, *b).total_cmp(&weight_at(weights, *a)));

    let total: f64 = order.iter().map(|index| weight_at(weights, *index)).sum();
    let space = bounds.area();
    if total <= 0.0 || space <= 0.0 {
        return rects;
    }
    // Each item as (position in `weights`, area it must cover).
    let items: Vec<(usize, f64)> = order
        .into_iter()
        .map(|index| (index, weight_at(weights, index) / total * space))
        .collect();

    let mut free = bounds;
    let mut remaining = items.as_slice();
    while !remaining.is_empty() {
        let short_side = free.width.min(free.height);
        if short_side <= 0.0 {
            break;
        }
        let mut count = 1;
        while count < remaining.len()
            && worst_ratio(remaining.get(..=count).unwrap_or_default(), short_side)
                <= worst_ratio(remaining.get(..count).unwrap_or_default(), short_side)
        {
            count += 1;
        }
        let (row, rest) = remaining.split_at(count);
        free = place_row(row, free, &mut rects);
        remaining = rest;
    }
    rects
}

fn is_usable(weight: f64) -> bool {
    weight.is_finite() && weight > 0.0
}

fn weight_at(weights: &[f64], index: usize) -> f64 {
    weights.get(index).copied().unwrap_or(0.0)
}

/// The worst (largest) aspect ratio among the rectangles of a row laid along
/// a side of the given length. 1.0 is a perfect square.
fn worst_ratio(row: &[(usize, f64)], side: f64) -> f64 {
    let sum: f64 = row.iter().map(|(_, area)| area).sum();
    let largest = row.iter().map(|(_, area)| *area).fold(0.0, f64::max);
    let smallest = row
        .iter()
        .map(|(_, area)| *area)
        .fold(f64::INFINITY, f64::min);
    if sum <= 0.0 || smallest <= 0.0 {
        return f64::INFINITY;
    }
    let side_squared = side * side;
    (side_squared * largest / (sum * sum)).max(sum * sum / (side_squared * smallest))
}

/// Lays one row along the shorter side of `free` and returns what is left.
fn place_row(row: &[(usize, f64)], free: LayoutRect, rects: &mut [LayoutRect]) -> LayoutRect {
    let row_area: f64 = row.iter().map(|(_, area)| area).sum();
    let is_wide = free.width >= free.height;
    let short_side = if is_wide { free.height } else { free.width };
    let thickness = row_area / short_side;

    let mut offset = 0.0;
    for (index, area) in row {
        let length = area / thickness;
        let rect = if is_wide {
            LayoutRect::new(free.x, free.y + offset, thickness, length)
        } else {
            LayoutRect::new(free.x + offset, free.y, length, thickness)
        };
        if let Some(slot) = rects.get_mut(*index) {
            *slot = rect;
        }
        offset += length;
    }

    if is_wide {
        LayoutRect::new(
            free.x + thickness,
            free.y,
            (free.width - thickness).max(0.0),
            free.height,
        )
    } else {
        LayoutRect::new(
            free.x,
            free.y + thickness,
            free.width,
            (free.height - thickness).max(0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::squarify;
    use crate::treemap::LayoutRect;

    const BOUNDS: LayoutRect = LayoutRect::new(10.0, 20.0, 800.0, 400.0);
    const TOLERANCE: f64 = 1e-6;

    #[test]
    fn areas_are_proportional_to_the_weights() {
        let weights = [96.0, 71.0, 58.0, 44.0, 32.0, 27.0, 21.0, 12.0, 9.0];
        let total: f64 = weights.iter().sum();

        let rects = squarify(&weights, BOUNDS);

        for (weight, rect) in weights.iter().zip(&rects) {
            let expected = weight / total * BOUNDS.area();
            assert!(
                (rect.area() - expected).abs() < TOLERANCE,
                "{weight}: {rect:?}"
            );
        }
    }

    #[test]
    fn rectangles_fill_the_bounds_without_overlapping() {
        let weights = [50.0, 30.0, 10.0, 5.0, 3.0, 1.0, 1.0];

        let rects = squarify(&weights, BOUNDS);

        let covered: f64 = rects.iter().map(LayoutRect::area).sum();
        assert!((covered - BOUNDS.area()).abs() < TOLERANCE);
        for (index, rect) in rects.iter().enumerate() {
            assert!(rect.x >= BOUNDS.x - TOLERANCE && rect.right() <= BOUNDS.right() + TOLERANCE);
            assert!(rect.y >= BOUNDS.y - TOLERANCE && rect.bottom() <= BOUNDS.bottom() + TOLERANCE);
            for other in rects.iter().skip(index + 1) {
                assert!(
                    rect.overlap(other) < TOLERANCE,
                    "{rect:?} overlaps {other:?}"
                );
            }
        }
    }

    #[test]
    fn keeps_the_order_of_the_input_whatever_the_sizes() {
        let rects = squarify(&[1.0, 100.0, 10.0], BOUNDS);
        let areas: Vec<f64> = rects.iter().map(LayoutRect::area).collect();
        assert!(areas[1] > areas[2] && areas[2] > areas[0]);
    }

    #[test]
    fn a_single_item_takes_the_whole_space() {
        assert_eq!(squarify(&[42.0], BOUNDS), [BOUNDS]);
    }

    #[test]
    fn unusable_weights_get_an_empty_rectangle() {
        let rects = squarify(&[0.0, 5.0, -3.0, f64::NAN, f64::INFINITY], BOUNDS);
        assert_eq!(rects.len(), 5);
        assert_eq!(rects[1], BOUNDS);
        for index in [0, 2, 3, 4] {
            assert!(rects[index].area().abs() < TOLERANCE);
        }
    }

    #[test]
    fn nothing_to_lay_out_gives_nothing() {
        assert!(squarify(&[], BOUNDS).is_empty());
        let empty = LayoutRect::new(0.0, 0.0, 0.0, 100.0);
        assert!(
            squarify(&[1.0, 2.0], empty)
                .iter()
                .all(|rect| rect.area().abs() < TOLERANCE)
        );
    }

    #[test]
    fn prefers_squarish_rectangles() {
        let rects = squarify(
            &[6.0, 6.0, 4.0, 3.0, 2.0, 2.0, 1.0],
            LayoutRect::new(0.0, 0.0, 6.0, 4.0),
        );
        let worst = rects
            .iter()
            .map(|rect| (rect.width / rect.height).max(rect.height / rect.width))
            .fold(0.0, f64::max);
        assert!(worst < 3.0, "worst aspect ratio was {worst}");
    }
}
