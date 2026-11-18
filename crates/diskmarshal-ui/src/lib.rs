//! Interface layer of DiskMarshal: screens, components, theme and texts.
//!
//! This crate draws with egui and reaches the system only through the ports of
//! `diskmarshal-application`. It never depends on `diskmarshal-infrastructure`.
//! The visual reference is `design/prototype/index.html`.

#![forbid(unsafe_code)]

mod animation;
mod app;
mod components;
mod effects;
mod i18n;
mod navigation;
mod screens;
mod tasks;
mod theme;

pub use animation::MotionPreference;
pub use app::{DiskMarshalUi, UiOptions, UiServices};
pub use navigation::Screen;
pub use theme::{Palette, WINDOW_SIZE};
