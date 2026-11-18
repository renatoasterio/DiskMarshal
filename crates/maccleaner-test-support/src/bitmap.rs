//! Image files made from a formula, so tests need no binary fixtures.

/// Size of the two headers of an uncompressed BMP file.
const HEADER_BYTES: u32 = 54;
const BYTES_PER_PIXEL: u32 = 3;

/// Builds an uncompressed 24-bit BMP file `width` by `height` pixels.
///
/// `color_at` receives the position of a pixel, each coordinate from 0.0 to
/// 1.0 with the origin at the top left, and returns its red, green and blue.
#[must_use]
pub fn bitmap_bytes(width: u32, height: u32, color_at: impl Fn(f32, f32) -> [u8; 3]) -> Vec<u8> {
    let row_bytes = (width * BYTES_PER_PIXEL).next_multiple_of(4);
    let data_bytes = row_bytes * height;
    let mut file = Vec::with_capacity((HEADER_BYTES + data_bytes) as usize);

    file.extend(b"BM");
    file.extend((HEADER_BYTES + data_bytes).to_le_bytes());
    file.extend(0u32.to_le_bytes());
    file.extend(HEADER_BYTES.to_le_bytes());

    file.extend(40u32.to_le_bytes());
    file.extend(width.to_le_bytes());
    file.extend(height.to_le_bytes());
    file.extend(1u16.to_le_bytes());
    file.extend(24u16.to_le_bytes());
    file.extend(0u32.to_le_bytes());
    file.extend(data_bytes.to_le_bytes());
    file.extend(2_835u32.to_le_bytes());
    file.extend(2_835u32.to_le_bytes());
    file.extend(0u32.to_le_bytes());
    file.extend(0u32.to_le_bytes());

    // Rows are stored bottom first, each pixel as blue, green, red.
    for y in (0..height).rev() {
        for x in 0..width {
            let [red, green, blue] = color_at(x as f32 / width as f32, y as f32 / height as f32);
            file.extend([blue, green, red]);
        }
        file.extend(std::iter::repeat_n(
            0u8,
            (row_bytes - width * BYTES_PER_PIXEL) as usize,
        ));
    }
    file
}

#[cfg(test)]
mod tests {
    use super::bitmap_bytes;

    #[test]
    fn the_file_has_the_size_its_header_declares() {
        let file = bitmap_bytes(5, 3, |_, _| [1, 2, 3]);
        // Rows of 5 pixels take 15 bytes, padded to 16.
        assert_eq!(file.len(), 54 + 16 * 3);
        assert_eq!(&file[2..6], &(file.len() as u32).to_le_bytes());
        // The first stored pixel is blue, green, red.
        assert_eq!(&file[54..57], &[3, 2, 1]);
    }
}
