# MacCleaner

Disk cleaner and system monitor for macOS, written in Rust.

It scans the disk and maps what uses space (local vs. iCloud), finds duplicate
files and similar photos, cleans caches and temporary data, removes build and
dependency folders from code projects, and monitors CPU, memory, disk,
temperature and fans.

## Status

| Step | State |
|---|---|
| HTML prototype of the interface (`design/prototype/index.html`) | Approved |
| Workspace, quality gate, CI | Done |
| Disk scan (domain, use case, walker, `maccleaner-cli scan`) | Done |
| Safe deletion (policy, `DeleteItems`, remover, Trash, history) | Done |
| Interface frame (theme, status bar, navigation rail, appearance panel, start-up sequence) | Done |
| Screen content: disk map, cleanup, developer, duplicates, photos, monitor, dashboard | Planned |

## Requirements

- macOS on Apple Silicon or Intel, Xcode command line tools.
- Rust, installed with Homebrew's `rustup`. Its tools are not on `PATH` by default:

  ```bash
  export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
  ```

- `cargo-deny` for the dependency audit: `brew install cargo-deny`.

## Everyday commands

```bash
cargo xtask check                          # format, lint, tests, dependency audit
cargo run -p maccleaner-cli -- scan ~      # measure a folder, read-only
cargo run -p maccleaner                    # open the app
```

To compare the interface with the prototype without watching the window, build
the development-only `capture` variant. It saves one still frame and exits:

```bash
cargo build -p maccleaner --features capture
```

```bash
EFRAME_SCREENSHOT_TO=/tmp/maccleaner.png MACCLEANER_SCREEN=cleanup ./target/debug/maccleaner
```

## Layout

```
apps/      binaries (composition roots): cli, desktop, and later privileged-helper
assets/    embedded fonts and their licenses
crates/    libraries, one per layer: domain, application, infrastructure, ui, test-support
design/    HTML prototype, the visual reference for the interface
docs/      architecture, conventions, security policy, decision records
xtask/     project automation
```

Read [docs/architecture.md](docs/architecture.md) before adding code, and
[docs/conventions.md](docs/conventions.md) before naming anything.
