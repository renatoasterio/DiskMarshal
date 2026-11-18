# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- The project is now called DiskMarshal; it was MacCleaner, a name other
  products already use. Crates, binaries, the app bundle and its identifier,
  the `DISKMARSHAL_*` environment variables and the repository address all
  carry the new name. The removal history now lives in
  `~/Library/Application Support/DiskMarshal/`; a `history.jsonl` kept under
  the previous name is not moved automatically, so copy it there to keep it.
- The number keys now go from 1 to 9: the screens after the cleanup moved one
  or two places to make room for the apps and Docker screens.

### Added

- The fingerprint for administrator actions: on a Mac where the user turned
  on Touch ID for `sudo`, the system cleanup and the removal of an app's
  system items ask for the fingerprint, and fall back to the password dialog
  when it does not go through. The screens say which one will be asked. The
  usage guide shows the one line that turns it on.
- Apps screen: lists the installed apps with their size, version and whether
  they are open; for the chosen app, lists the bundle and what it left in the
  Library folders, each item with its kind, a "Seguro" or "Revisar" badge and
  its size. Items named exactly after the bundle identifier come selected;
  items that only start with it, or are named like the app, wait for the user.
  What belongs to the user goes to the Trash. An app that is open is refused.
- Uninstall with macOS authentication: a bundle owned by root and what the app
  left in `/Library` are removed by `diskmarshal-privileged-helper`, which is
  told the app by its identifier and removes only the requested items that its
  own analysis finds too. Not yet run as root or exercised against the real
  dialog.
- Docker screen: finds out whether Docker is installed and running; shows how
  much it holds of images, containers, volumes and build cache; lists what no
  container uses. The build cache and images without a name come selected.
  Docker itself removes what is approved, nothing is forced, and the report
  shows what Docker freed, measured before and after. The removal was exercised
  against a stand-in for Docker only.
- `diskmarshal-cli apps` and `diskmarshal-cli docker`: show what those two
  screens would find, without removing anything.
- Dashboard: a row for what Docker says it can free.
- `cargo xtask install`: builds the release app, assembles and signs
  `dist/DiskMarshal.app` (the app, the privileged helper and the icon) and copies
  it to `/Applications`, or to the folder given as an argument. `cargo xtask bundle`
  stops after assembling the app. The signature is ad hoc unless
  `DISKMARSHAL_SIGNING_IDENTITY` names a signing identity.
- App icon.
- Dashboard screen: gauges for disk, memory and the hottest temperature sensor
  (processor use when the Mac reports no sensor); the recoverable space found by
  the other screens, one row per source, each opening its screen; the latest
  entries of the removal history; a button that runs the cleanup analysis.
- Cleanup screen: analysis that starts by itself and removes nothing; rules for
  app caches, temporary files, logs, the Trash, Xcode DerivedData, simulator
  caches, Xcode device support, Xcode archives and package manager caches, each
  with a "Seguro" or "Revisar" badge; safe rules that found something come
  selected; removal after a confirmation dialog.
- Cleanup of system areas (`/Library/Caches` and `/Library/Logs`) with macOS
  authentication: on confirmation the app asks macOS to authenticate the user and
  to run `diskmarshal-privileged-helper` once. The helper accepts rule ids only,
  takes the folders from the catalog compiled into it, deletes through the same
  policy as the app, and has a simulation mode (`--dry-run`). The app never runs
  as root and keeps no background service. Not yet exercised end to end against
  the real dialog and folders.
- Duplicates screen: search of a chosen folder or the home folder, with a minimum
  size of 1, 10 or 100 MB; files compared by size, then beginning and end, then
  whole content (BLAKE3); hard links not counted as copies; cloud-only files
  never read; the oldest copy kept by default and at least one copy always kept;
  selected copies moved to the Trash.
- Similar photos screen: search of a chosen folder or the Pictures folder at three
  levels of similarity; compares loose image files of at least 50 KB (JPEG, PNG,
  HEIC/HEIF, TIFF, BMP, GIF, WebP and camera RAW) by an outline hash, a brightness
  map, the average color and the proportions; thumbnails with a reference photo
  and a match percentage; at least one photo per group always kept; marked photos
  moved to the Trash. The Photos app library is never read or changed.
- Developer screen: build and dependency folders of Node, .NET, Rust, Java/Kotlin,
  Python, Swift, Flutter, Go, PHP and Terraform projects, offered only when the
  project's marker file is beside the folder and git has no tracked files in it;
  safe folders of projects untouched for 30 days or more come selected; permanent
  removal after confirmation.
- Disk map screen: treemap of the home folder or the whole disk, colored by file
  type or by iCloud state, with zoom on click, a trail to go back, a card for the
  block under the pointer and "show in Finder" on right-click.
- Monitor screen: processor trace of the last two minutes, one bar per core,
  memory and swap, disk, temperatures grouped by component and the six busiest
  programs, read every second (programs every 2 s, temperatures every 5 s).
- Full Disk Access notice: when macOS denies access to guarded folders, the
  screens that read personal folders show one line with a link that opens the
  right page of System Settings.
- One removal flow shared by every screen: confirmation, removal in the
  background with each item revalidated, and a report of what was freed, sent to
  the Trash, skipped or not recorded.
- The app starts with reduced motion when "Reduce motion" is on in System
  Settings.
- Frame pacing that keeps idle processor use low: every frame right after input,
  twenty frames per second while background animations loop, one per second while
  the window has no focus.
- Public documentation in English and Portuguese: README with a tour GIF and
  screenshots taken from the app itself on demonstration data, and guides for
  getting started, usage, safety, and questions and troubleshooting. The README
  states what is built, what is not verified and the known limits.
- MIT license, contributing guide, code of conduct, vulnerability reporting policy,
  and issue and pull request templates.
- Each prototype screen has its own address (`index.html#cleanup`), and its counters
  animate after the start-up sequence instead of behind it.
- Navigable HTML prototype of the screens, with three palettes and motion
  controls.
- Layered Cargo workspace with shared lints, pinned toolchain, `cargo xtask check`
  and CI.
- Disk scan: file tree with aggregated sizes, iCloud state per item, packages
  measured as one item, hard links counted once, symbolic links never followed.
- Process-wide policy that keeps scans from downloading cloud-only iCloud files.
- Safe deletion: a path policy that protects system and personal locations, `.git`
  folders and the inside of packages; items are revalidated right before removal;
  personal files go to the Trash; every removal is written to a history file
  readable only by its owner.
- Interface frame in egui, matching the prototype: status bar with free space and
  clock, navigation rail with keyboard shortcuts (1 to 7), three palettes, motion
  and glow controls behind the appearance icon, start-up sequence, background grid
  and scanlines.
- `diskmarshal-cli scan <folder>`: read-only summary of a folder and its largest
  items.
