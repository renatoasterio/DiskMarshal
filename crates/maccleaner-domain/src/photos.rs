//! Similar photos: images that look the same even when their files differ.

mod image_format;
mod photo_signature;
mod similarity_group;
mod similarity_tolerance;
mod thumbnail;

pub use image_format::{MINIMUM_PHOTO_SIZE, is_image_file, photo_candidates};
pub use photo_signature::PhotoSignature;
pub use similarity_group::{SimilarPhoto, SimilarityGroup, group_similar};
pub use similarity_tolerance::SimilarityTolerance;
pub use thumbnail::Thumbnail;
