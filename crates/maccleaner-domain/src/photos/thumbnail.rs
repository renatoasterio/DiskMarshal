//! A small copy of an image, as plain pixels.

/// Bytes each pixel takes: red, green, blue and alpha.
const BYTES_PER_PIXEL: usize = 4;

/// A small copy of an image: rows of RGBA pixels, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thumbnail {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Thumbnail {
    /// Creates a thumbnail. Returns `None` when a side is zero or `rgba` does
    /// not hold exactly `width * height` pixels.
    #[must_use]
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let pixels = (width as usize).checked_mul(height as usize)?;
        let is_valid = pixels > 0 && pixels.checked_mul(BYTES_PER_PIXEL) == Some(rgba.len());
        is_valid.then_some(Self {
            width,
            height,
            rgba,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The pixels, four bytes each, top row first.
    #[must_use]
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// The red, green and blue of every pixel, top row first.
    pub(crate) fn colors(&self) -> impl Iterator<Item = [u8; 3]> + '_ {
        let (pixels, _) = self.rgba.as_chunks::<BYTES_PER_PIXEL>();
        pixels
            .iter()
            .map(|[red, green, blue, _]| [*red, *green, *blue])
    }
}

#[cfg(test)]
mod tests {
    use super::Thumbnail;

    #[test]
    fn refuses_pixels_that_do_not_fill_the_sides() {
        assert!(Thumbnail::new(2, 2, vec![0; 16]).is_some());
        assert!(Thumbnail::new(2, 2, vec![0; 15]).is_none());
        assert!(Thumbnail::new(0, 2, Vec::new()).is_none());
    }
}
