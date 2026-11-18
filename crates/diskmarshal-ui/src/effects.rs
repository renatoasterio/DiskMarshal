//! Painted effects shared by the whole interface. No screen computes glow or
//! scanlines on its own.

pub(crate) mod glow;
mod grid_background;
mod scanline_overlay;

pub(crate) use grid_background::paint as paint_grid;
pub(crate) use scanline_overlay::paint as paint_scanlines;
