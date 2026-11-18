//! An axis-aligned rectangle used by layouts.

/// A rectangle given by its top-left corner and its size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutRect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width. Never negative in a rectangle produced by a layout.
    pub width: f64,
    /// Height. Never negative in a rectangle produced by a layout.
    pub height: f64,
}

impl LayoutRect {
    /// Creates a rectangle from its top-left corner and size.
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Area of the rectangle.
    #[must_use]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Right edge.
    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Bottom edge.
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Area shared with `other`. Zero when they only touch or are apart.
    #[must_use]
    pub fn overlap(&self, other: &Self) -> f64 {
        let width = self.right().min(other.right()) - self.x.max(other.x);
        let height = self.bottom().min(other.bottom()) - self.y.max(other.y);
        if width > 0.0 && height > 0.0 {
            width * height
        } else {
            0.0
        }
    }
}
