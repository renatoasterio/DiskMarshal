//! Thumbnails made by ImageIO, the image reader of macOS.
//!
//! ImageIO opens every format macOS knows, including HEIC and camera raw
//! files, and shrinks while decoding, so a thumbnail costs far less than
//! reading the whole image.

use std::ffi::c_void;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use maccleaner_application::ports::ImageThumbnailer;
use maccleaner_domain::photos::Thumbnail;

/// A reference to any Core Foundation object.
type CfRef = *const c_void;
/// `CFIndex`: a signed integer as wide as a pointer.
type CfIndex = isize;

/// A structure this module only ever takes the address of.
#[repr(C)]
struct Opaque {
    _private: [u8; 0],
}

#[repr(C)]
struct CgPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct CgSize {
    width: f64,
    height: f64,
}

#[repr(C)]
struct CgRect {
    origin: CgPoint,
    size: CgSize,
}

/// `kCFNumberSInt32Type` from `CFNumber.h`.
const CF_NUMBER_SINT32: CfIndex = 3;
/// `kCGImageAlphaPremultipliedLast` from `CGImage.h`: red, green, blue, alpha.
const ALPHA_PREMULTIPLIED_LAST: u32 = 1;
const BITS_PER_CHANNEL: usize = 8;
const BYTES_PER_PIXEL: usize = 4;
/// Largest side this adapter accepts to make. A caller asking for more is
/// asking for an image, not a thumbnail.
const MAX_SIDE: u32 = 2_048;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeDictionaryKeyCallBacks: Opaque;
    static kCFTypeDictionaryValueCallBacks: Opaque;
    static kCFBooleanTrue: CfRef;

    fn CFRelease(object: CfRef);
    fn CFURLCreateFromFileSystemRepresentation(
        allocator: CfRef,
        buffer: *const u8,
        length: CfIndex,
        is_directory: u8,
    ) -> CfRef;
    fn CFNumberCreate(allocator: CfRef, number_type: CfIndex, value: *const c_void) -> CfRef;
    fn CFDictionaryCreate(
        allocator: CfRef,
        keys: *const CfRef,
        values: *const CfRef,
        count: CfIndex,
        key_callbacks: *const Opaque,
        value_callbacks: *const Opaque,
    ) -> CfRef;
}

#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    static kCGImageSourceCreateThumbnailFromImageAlways: CfRef;
    static kCGImageSourceCreateThumbnailWithTransform: CfRef;
    static kCGImageSourceThumbnailMaxPixelSize: CfRef;

    fn CGImageSourceCreateWithURL(url: CfRef, options: CfRef) -> CfRef;
    fn CGImageSourceGetCount(source: CfRef) -> usize;
    fn CGImageSourceCreateThumbnailAtIndex(source: CfRef, index: usize, options: CfRef) -> CfRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGImageGetWidth(image: CfRef) -> usize;
    fn CGImageGetHeight(image: CfRef) -> usize;
    fn CGColorSpaceCreateDeviceRGB() -> CfRef;
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        color_space: CfRef,
        bitmap_info: u32,
    ) -> CfRef;
    fn CGContextDrawImage(context: CfRef, rect: CgRect, image: CfRef);
}

/// An object this module owns and releases when it goes out of scope.
struct Owned(CfRef);

impl Owned {
    /// Takes over an object returned by a `Create` function. Such functions
    /// return null when they fail, which becomes `None`.
    fn new(object: CfRef) -> Option<Self> {
        if object.is_null() {
            None
        } else {
            Some(Self(object))
        }
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the pointer is not null, came from a `Create` function, so
        // this module holds one reference to it, and it is released only here.
        unsafe { CFRelease(self.0) }
    }
}

fn unreadable(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("not a readable image: {}", path.display()),
    )
}

/// Makes thumbnails with ImageIO.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageIoThumbnailer;

impl ImageThumbnailer for ImageIoThumbnailer {
    fn thumbnail(&self, path: &Path, max_side: u32) -> io::Result<Thumbnail> {
        let failed = || unreadable(path);
        let side = i32::try_from(max_side.clamp(1, MAX_SIDE)).map_err(|_| failed())?;
        let image = thumbnail_image(path, side).ok_or_else(failed)?;
        let (width, height, pixels) = pixels_of(&image).ok_or_else(failed)?;
        Thumbnail::new(width, height, pixels).ok_or_else(failed)
    }
}

/// Asks ImageIO for a picture of the file, at most `side` pixels on its
/// longer side and turned the way the camera recorded it should be seen.
fn thumbnail_image(path: &Path, side: i32) -> Option<Owned> {
    let name = path.as_os_str().as_bytes();
    let length = CfIndex::try_from(name.len()).ok()?;
    // SAFETY: `name` points to `length` readable bytes for the whole call,
    // and a null allocator selects the default one.
    let url = Owned::new(unsafe {
        CFURLCreateFromFileSystemRepresentation(ptr::null(), name.as_ptr(), length, 0)
    })?;
    // SAFETY: `url` is a live CFURL, and null options are allowed.
    let source = Owned::new(unsafe { CGImageSourceCreateWithURL(url.0, ptr::null()) })?;
    // A file that holds no image can still give a source, with nothing in it.
    // SAFETY: `source` is a live CGImageSource.
    if unsafe { CGImageSourceGetCount(source.0) } == 0 {
        return None;
    }
    // SAFETY: the pointer refers to a live `i32`, the type the constant names.
    let max_pixels = Owned::new(unsafe {
        CFNumberCreate(ptr::null(), CF_NUMBER_SINT32, (&raw const side).cast())
    })?;

    // SAFETY: these are constants exported by ImageIO and Core Foundation,
    // initialized when the frameworks load and never written afterwards.
    let (keys, yes) = unsafe {
        (
            [
                kCGImageSourceCreateThumbnailFromImageAlways,
                kCGImageSourceCreateThumbnailWithTransform,
                kCGImageSourceThumbnailMaxPixelSize,
            ],
            kCFBooleanTrue,
        )
    };
    let values = [yes, yes, max_pixels.0];
    // SAFETY: `keys` and `values` each hold three live objects, matching the
    // count, and the callbacks are the standard ones for CF objects, which
    // make the dictionary retain what it stores.
    let options = Owned::new(unsafe {
        CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            3,
            &raw const kCFTypeDictionaryKeyCallBacks,
            &raw const kCFTypeDictionaryValueCallBacks,
        )
    })?;
    // SAFETY: `source` and `options` are live objects of the expected types.
    Owned::new(unsafe { CGImageSourceCreateThumbnailAtIndex(source.0, 0, options.0) })
}

/// Draws the image into a buffer of RGBA pixels, top row first.
fn pixels_of(image: &Owned) -> Option<(u32, u32, Vec<u8>)> {
    // SAFETY: `image` is a live CGImage.
    let (width, height) = unsafe { (CGImageGetWidth(image.0), CGImageGetHeight(image.0)) };
    let fits = |side: usize| side > 0 && side <= MAX_SIDE as usize;
    if !fits(width) || !fits(height) {
        return None;
    }
    let row_bytes = width * BYTES_PER_PIXEL;
    let mut pixels = vec![0u8; row_bytes * height];
    // SAFETY: the function takes no arguments and returns a new object.
    let color_space = Owned::new(unsafe { CGColorSpaceCreateDeviceRGB() })?;
    {
        // SAFETY: `pixels` holds `height` rows of `row_bytes` bytes, exactly
        // what the context is told, and outlives the context, which is
        // released at the end of this block.
        let context = Owned::new(unsafe {
            CGBitmapContextCreate(
                pixels.as_mut_ptr().cast(),
                width,
                height,
                BITS_PER_CHANNEL,
                row_bytes,
                color_space.0,
                ALPHA_PREMULTIPLIED_LAST,
            )
        })?;
        let whole = CgRect {
            origin: CgPoint { x: 0.0, y: 0.0 },
            size: CgSize {
                width: width as f64,
                height: height as f64,
            },
        };
        // SAFETY: `context` and `image` are live, and the context writes only
        // inside the buffer it was created over.
        unsafe { CGContextDrawImage(context.0, whole, image.0) };
    }
    Some((
        u32::try_from(width).ok()?,
        u32::try_from(height).ok()?,
        pixels,
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use maccleaner_application::ports::ImageThumbnailer;
    use maccleaner_test_support::{TempTree, bitmap_bytes};

    use super::ImageIoThumbnailer;

    /// Red at the top left, green at the top right, blue across the bottom.
    fn quadrants(x: f32, y: f32) -> [u8; 3] {
        match (y < 0.5, x < 0.5) {
            (true, true) => [255, 0, 0],
            (true, false) => [0, 255, 0],
            (false, _) => [0, 0, 255],
        }
    }

    fn pixel(thumbnail: &maccleaner_domain::photos::Thumbnail, x: u32, y: u32) -> [u8; 3] {
        let start = ((y * thumbnail.width() + x) * 4) as usize;
        let rgba = thumbnail.rgba();
        [rgba[start], rgba[start + 1], rgba[start + 2]]
    }

    #[test]
    fn shrinks_an_image_keeping_its_proportions_and_its_top_row_first() {
        let fixture = TempTree::new().unwrap();
        let path = fixture.path("picture.bmp");
        fs::write(&path, bitmap_bytes(200, 100, quadrants)).unwrap();

        let thumbnail = ImageIoThumbnailer.thumbnail(&path, 40).unwrap();

        assert_eq!((thumbnail.width(), thumbnail.height()), (40, 20));
        let is_mostly = |[red, green, blue]: [u8; 3], channel: usize| {
            let channels = [red, green, blue];
            channels[channel] > 200
                && channels
                    .iter()
                    .enumerate()
                    .all(|(i, c)| i == channel || *c < 60)
        };
        assert!(is_mostly(pixel(&thumbnail, 5, 3), 0), "top left is red");
        assert!(is_mostly(pixel(&thumbnail, 34, 3), 1), "top right is green");
        assert!(is_mostly(pixel(&thumbnail, 20, 16), 2), "bottom is blue");
    }

    #[test]
    fn a_small_image_is_not_enlarged() {
        let fixture = TempTree::new().unwrap();
        let path = fixture.path("small.bmp");
        fs::write(&path, bitmap_bytes(12, 8, quadrants)).unwrap();

        let thumbnail = ImageIoThumbnailer.thumbnail(&path, 64).unwrap();

        assert_eq!((thumbnail.width(), thumbnail.height()), (12, 8));
    }

    #[test]
    fn a_file_that_is_not_an_image_is_an_error() {
        let fixture = TempTree::new().unwrap();
        let text = fixture.file("notes.jpg", 500).unwrap();

        assert!(ImageIoThumbnailer.thumbnail(&text, 64).is_err());
        assert!(
            ImageIoThumbnailer
                .thumbnail(&fixture.path("missing.jpg"), 64)
                .is_err()
        );
    }
}
