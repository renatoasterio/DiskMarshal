# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Navigable HTML prototype of the seven screens, with three palettes and motion controls.
- Layered Cargo workspace with shared lints, pinned toolchain, `cargo xtask check` and CI.
- Disk scan: file tree with aggregated sizes, iCloud state per item, packages measured as
  one item, hard links counted once, symbolic links never followed.
- Process-wide policy that keeps scans from downloading cloud-only iCloud files.
- Safe deletion: a path policy that protects system and personal locations, `.git` folders
  and the inside of packages; items are revalidated right before removal; personal files go
  to the Trash; every removal is written to a history file readable only by its owner.
- Interface frame in egui, matching the prototype: status bar with free space and clock,
  navigation rail with keyboard shortcuts (1 to 7), three palettes, motion and glow
  controls behind the appearance icon, start-up sequence, background grid and scanlines.
  Screens still show a placeholder.
- `maccleaner-cli scan <folder>`: read-only summary of a folder and its largest items.
