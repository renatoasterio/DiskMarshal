//! Time-based motion. No screen computes time or easing on its own.

mod easing;
mod motion_preference;
mod timeline;

pub use motion_preference::MotionPreference;

pub(crate) use easing::ease_out_cubic;
pub(crate) use timeline::Timeline;
