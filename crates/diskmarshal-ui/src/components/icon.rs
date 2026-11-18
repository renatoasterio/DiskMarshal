//! Line icons, drawn from the same coordinates as the prototype's SVGs.

use egui::{Color32, Painter, Rect, Stroke, StrokeKind, vec2};

/// One stroke of an icon, in a 24 x 24 box.
enum IconShape {
    /// An open path through the given points.
    Line(&'static [[f32; 2]]),
    /// A rectangle: x, y, width, height.
    Rect([f32; 4]),
    /// A circle: center x, center y, radius.
    Circle([f32; 3]),
}

/// The icons of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Icon {
    Dashboard,
    Cleanup,
    Apps,
    Duplicates,
    Photos,
    Developer,
    Docker,
    DiskMap,
    Monitor,
    Network,
    Sliders,
    /// Marks what needs an administrator.
    Lock,
}

impl Icon {
    const VIEW_BOX: f32 = 24.0;
    const STROKE_WIDTH: f32 = 1.5;

    /// Paints the icon inside `rect`, which should be square.
    pub(crate) fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        let scale = rect.width() / Self::VIEW_BOX;
        let stroke = Stroke::new(Self::STROKE_WIDTH, color);
        let at = |point: [f32; 2]| rect.min + vec2(point[0], point[1]) * scale;
        for shape in self.shapes() {
            match shape {
                IconShape::Line(points) => {
                    painter.line(points.iter().map(|point| at(*point)).collect(), stroke);
                }
                IconShape::Rect([x, y, width, height]) => {
                    let area = Rect::from_min_size(at([*x, *y]), vec2(*width, *height) * scale);
                    painter.rect_stroke(area, 0.0, stroke, StrokeKind::Middle);
                }
                IconShape::Circle([x, y, radius]) => {
                    painter.circle_stroke(at([*x, *y]), radius * scale, stroke);
                }
            }
        }
    }

    const fn shapes(self) -> &'static [IconShape] {
        use IconShape::{Circle, Line, Rect};
        match self {
            Self::Dashboard => &[
                Rect([3.0, 3.0, 7.0, 7.0]),
                Rect([14.0, 3.0, 7.0, 4.0]),
                Rect([14.0, 11.0, 7.0, 10.0]),
                Rect([3.0, 14.0, 7.0, 7.0]),
            ],
            Self::Cleanup => &[
                Line(&[[12.0, 3.0], [12.0, 9.0]]),
                Line(&[[12.0, 15.0], [12.0, 21.0]]),
                Line(&[[3.0, 12.0], [9.0, 12.0]]),
                Line(&[[15.0, 12.0], [21.0, 12.0]]),
                Line(&[[6.0, 6.0], [9.0, 9.0]]),
                Line(&[[15.0, 15.0], [18.0, 18.0]]),
                Line(&[[18.0, 6.0], [15.0, 9.0]]),
                Line(&[[9.0, 15.0], [6.0, 18.0]]),
            ],
            Self::Apps => &[
                Rect([4.0, 4.0, 6.0, 6.0]),
                Rect([14.0, 4.0, 6.0, 6.0]),
                Rect([4.0, 14.0, 6.0, 6.0]),
                Line(&[[14.5, 14.5], [19.5, 19.5]]),
                Line(&[[19.5, 14.5], [14.5, 19.5]]),
            ],
            Self::Docker => &[
                Line(&[[3.0, 18.0], [21.0, 18.0]]),
                Rect([5.0, 12.0, 4.0, 5.0]),
                Rect([10.0, 12.0, 4.0, 5.0]),
                Rect([15.0, 12.0, 4.0, 5.0]),
                Rect([10.0, 6.0, 4.0, 5.0]),
            ],
            Self::Duplicates => &[
                Rect([8.0, 8.0, 12.0, 12.0]),
                Line(&[
                    [16.0, 8.0],
                    [16.0, 4.0],
                    [4.0, 4.0],
                    [4.0, 16.0],
                    [8.0, 16.0],
                ]),
            ],
            Self::Photos => &[
                Rect([3.0, 5.0, 18.0, 14.0]),
                Line(&[
                    [3.0, 16.0],
                    [8.0, 11.0],
                    [12.0, 15.0],
                    [15.0, 12.0],
                    [21.0, 18.0],
                ]),
                Circle([16.0, 9.0, 1.5]),
            ],
            Self::Developer => &[
                Line(&[[8.0, 7.0], [3.0, 12.0], [8.0, 17.0]]),
                Line(&[[16.0, 7.0], [21.0, 12.0], [16.0, 17.0]]),
                Line(&[[13.0, 4.0], [11.0, 20.0]]),
            ],
            Self::DiskMap => &[
                Rect([3.0, 3.0, 18.0, 18.0]),
                Line(&[[3.0, 12.0], [13.0, 12.0]]),
                Line(&[[13.0, 3.0], [13.0, 21.0]]),
                Line(&[[13.0, 8.0], [21.0, 8.0]]),
                Line(&[[13.0, 15.0], [21.0, 15.0]]),
            ],
            Self::Monitor => &[Line(&[
                [2.0, 12.0],
                [6.0, 12.0],
                [9.0, 4.0],
                [13.0, 20.0],
                [16.0, 12.0],
                [22.0, 12.0],
            ])],
            Self::Lock => &[
                Rect([5.0, 11.0, 14.0, 10.0]),
                Line(&[
                    [8.0, 11.0],
                    [8.0, 7.5],
                    [9.5, 4.5],
                    [12.0, 3.5],
                    [14.5, 4.5],
                    [16.0, 7.5],
                    [16.0, 11.0],
                ]),
            ],
            Self::Network => &[
                Circle([12.0, 12.0, 9.0]),
                Line(&[[3.0, 12.0], [21.0, 12.0]]),
                Line(&[[12.0, 3.0], [14.6, 7.5], [14.6, 16.5], [12.0, 21.0]]),
                Line(&[[12.0, 3.0], [9.4, 7.5], [9.4, 16.5], [12.0, 21.0]]),
            ],
            Self::Sliders => &[
                Line(&[[3.0, 6.0], [12.0, 6.0]]),
                Line(&[[17.0, 6.0], [21.0, 6.0]]),
                Line(&[[3.0, 12.0], [6.0, 12.0]]),
                Line(&[[11.0, 12.0], [21.0, 12.0]]),
                Line(&[[3.0, 18.0], [14.0, 18.0]]),
                Line(&[[19.0, 18.0], [21.0, 18.0]]),
                Circle([14.5, 6.0, 2.2]),
                Circle([8.5, 12.0, 2.2]),
                Circle([16.5, 18.0, 2.2]),
            ],
        }
    }
}
