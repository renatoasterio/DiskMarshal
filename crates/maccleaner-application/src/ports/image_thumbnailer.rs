//! Port for reading images as small pictures.

use std::io;
use std::path::Path;

use maccleaner_domain::photos::Thumbnail;

/// Makes small copies of image files.
pub trait ImageThumbnailer: Send + Sync {
    /// A copy of the image whose longer side has at most `max_side` pixels,
    /// turned the way the image is meant to be seen.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read or is not an image.
    fn thumbnail(&self, path: &Path, max_side: u32) -> io::Result<Thumbnail>;
}
