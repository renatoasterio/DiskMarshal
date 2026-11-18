# Contributing to DiskMarshal

Thank you for wanting to help. Issues and pull requests are welcome in English or Portuguese.

Contribuições em português são bem-vindas: pode abrir issues e pull requests em português.

## Ways to help

- **Try it on your Mac and tell us how it went.** It is only tested on macOS 27 with Apple Silicon. A report from another macOS version or an Intel Mac is valuable, even when everything works.
- **Verify what is not verified.** The README lists [what was not confirmed on the real thing](README.md#not-verified-yet): the system cleanup from the macOS dialog to the result, a file that is really cloud-only, a start with "Reduce motion" on. A careful report on any of these helps a lot.
- **Add a cleanup rule or a kind of project.** Both are entries in a TOML file. See [Adding a cleanup rule or an ecosystem](#adding-a-cleanup-rule-or-an-ecosystem).
- **Translate the interface.** Every text lives in `crates/diskmarshal-ui/src/i18n/`.
- **Improve the documentation.** If a step in a guide confused you, it will confuse someone else.

For anything larger than a small fix, open an issue first so we can agree on the approach before you write the code. The README also lists a few [ideas](README.md#what-is-left) nobody has started.

## Setting up

Follow [Getting started](docs/guide/getting-started.md), then install the dependency auditor:

```bash
brew install cargo-deny
```

## The project tasks

Project automation runs as `cargo xtask <task>`, from the project folder.

| Task | What it does |
|---|---|
| `cargo xtask check` | The quality gate: formatting, lints, tests and the dependency audit. |
| `cargo xtask bundle` | Builds the release binaries and assembles and signs `dist/DiskMarshal.app`: the app, the privileged helper and the icon. |
| `cargo xtask install` | Does what `bundle` does and copies the app to `/Applications`. `cargo xtask install ~/Applications` installs somewhere else. |

A pull request must pass the quality gate:

```bash
cargo xtask check
```

While developing, `cargo run -p diskmarshal` opens the app straight from the source. The cleanup of system areas needs the helper beside the executable, so try that part with the app made by `cargo xtask bundle` or `cargo xtask install`.

The app is signed ad hoc unless the environment variable `DISKMARSHAL_SIGNING_IDENTITY` names a signing identity. With an ad hoc signature, macOS forgets the Full Disk Access permission each time the app is rebuilt and reinstalled.

## Rules that matter most

Read [architecture](docs/architecture.md), [conventions](docs/conventions.md) and the [security policy](docs/security.md) before a larger change. In short:

1. **Dependencies point inward.** `diskmarshal-ui` never depends on `diskmarshal-infrastructure`. The interface reaches macOS only through a port.
2. **One code path deletes.** Every removal goes through the `DeleteItems` use case. A change that touches deletion needs a test proving what it refuses to delete.
3. **Nothing reads cloud-only content.** A scan must never start an iCloud download.
4. **The privileged helper takes rule ids, nothing else.** No path, no command and no option ever reaches it from the app. Its folders come from the catalog compiled into it.
5. **No `unwrap`, `expect` or `panic!`** outside tests. `unsafe` only in `diskmarshal-infrastructure`, with a `SAFETY:` comment.
6. **No stray strings or colors.** Interface texts go in `i18n/`; colors, sizes and durations go in `theme/tokens.rs`.
7. **Say what is true.** The interface and the documentation never present a planned feature as finished, or something untested as tested.

## Adding a cleanup rule or an ecosystem

What DiskMarshal knows how to clean is data, not code. The three catalogs are in `crates/diskmarshal-infrastructure/resources/` and are compiled into the binaries, so rebuild after changing them.

### A cleanup rule

Add an entry to `cleanup_rules.toml`:

```toml
[[rule]]
id = "example-cache"
name = "Cache do Exemplo"
description = "~/Library/Application Support/Exemplo/Cache · recriado ao abrir o app"
locations = ["~/Library/Application Support/Exemplo/Cache"]
risk = "safe"
scope = "user"
method = "permanent"
```

| Field | Meaning |
|---|---|
| `id` | Unique name: lowercase words joined by hyphens. |
| `name`, `description` | What the cleanup screen shows, in Brazilian Portuguese. |
| `locations` | The folders to clean. Each one is absolute, or starts with `~/` (the home folder), or is `$TMPDIR` (the user's temporary folder). No folder may be inside the folder of another rule. |
| `risk` | `safe`: recreated automatically, selected by default. `review`: may be worth keeping, never selected by default. |
| `scope` | `user`: the logged-in user can remove it. `administrator`: macOS must authenticate an administrator first. |
| `method` | `permanent`: removed at once. `trash`: moved to the Trash. |
| `minimum_age_days` | Optional. Only items not modified for at least this many days are removed. |

A rule removes the content of its folders, never the folders themselves. Unknown fields are rejected. The tests in `catalog/toml_rule_catalog.rs` check the whole catalog: every folder absolute, none inside another rule's folder, and everything a rule removes allowed by the deletion policy.

Think twice before adding a rule with `scope = "administrator"`. It is compiled into the privileged helper and runs as root, the helper cannot use the Trash, and the change needs the same care as a change to deletion. Open an issue first.

### A kind of project (developer screen)

Add an entry to `artifact_rules.toml`:

```toml
[[project]]
ecosystem = "node"
markers = ["package.json"]
safe = ["node_modules"]
review = ["dist"]
```

| Field | Meaning |
|---|---|
| `ecosystem` | One of `node`, `dotnet`, `rust`, `jvm`, `python`, `swift`, `flutter`, `go`, `php`, `terraform`. |
| `markers` | Files that identify the project. A folder is offered only when one of them sits next to it. A marker starting with `*.` matches any file with that extension. |
| `safe` | Folder names nobody uses for anything else. Selected by default in projects untouched for 30 days or more. |
| `review` | Common names a project might use for real content. Never selected by default. |

An entry needs at least one marker and one folder. An ecosystem that is not in the list also needs a variant in `Ecosystem` (`crates/diskmarshal-domain/src/dev_artifacts/ecosystem.rs`) and in the catalog reader (`crates/diskmarshal-infrastructure/src/catalog/toml_rule_catalog.rs`).

When you add either kind of entry, update the tables in the usage guide, in both languages.

### A folder where apps leave items (apps screen)

Add an entry to `leftover_locations.toml`:

```toml
[[location]]
folder = "~/Library/Application Support"
kind = "support"
scope = "user"
by_name = true
```

| Field | Meaning |
|---|---|
| `folder` | The folder whose direct items are compared with the app being uninstalled. Absolute, or starting with `~/`. Each folder appears once. |
| `kind` | What the items found there are, for the label in the list: `support`, `caches`, `preferences`, `container`, `saved-state`, `web-data`, `logs`, `launch-item` or `helper`. |
| `scope` | `user`: goes to the Trash. `administrator`: removed for good by the privileged helper. |
| `by_name` | Optional. `true` also accepts an item named exactly like the app. Such a match is never selected by default. Leave it out where items are named only after identifiers. |

An item belongs to an app when its name is the bundle identifier of the app, alone or followed by a dot. That rule is code, not data: `LeftoverMatcher` in the domain. The privileged helper uses it to decide what root may remove, so a change to it, like an `administrator` folder, needs the same care as a change to deletion. Open an issue first.

## Commits and pull requests

- Create a branch from `main`: `feat/disk-map`, `fix/scan-hard-links`, `docs/usage-guide`.
- Write commit messages as [Conventional Commits](https://www.conventionalcommits.org/): `feat(scan): add bulk walker`.
- Keep a pull request to one subject, and describe what you changed and how you checked it.
- Update `CHANGELOG.md` when the change is visible to users.
- The README and the guides exist in English and Portuguese. Change both in the same pull request.

## Checking the interface without watching the window

A development-only build saves one still frame of the app and exits. Use it to check a screen, or to compare it with the design reference in `design/prototype/index.html`:

```bash
cargo build -p diskmarshal --features capture
```

```bash
EFRAME_SCREENSHOT_TO=/tmp/diskmarshal.png DISKMARSHAL_SCREEN=cleanup DISKMARSHAL_PALETTE=terminal DISKMARSHAL_ANALYZE=1 ./target/debug/diskmarshal
```

| Variable | Effect |
|---|---|
| `EFRAME_SCREENSHOT_TO` | The PNG file to save. |
| `DISKMARSHAL_SCREEN` | The screen to show: `dashboard`, `cleanup`, `apps`, `duplicates`, `photos`, `developer`, `docker`, `disk_map` or `monitor`. |
| `DISKMARSHAL_PALETTE` | `agency`, `terminal` or `tactical`. |
| `DISKMARSHAL_ANALYZE=1` | Analyzes the cleanup rules first, so the cleanup screen opens filled in. |
| `DISKMARSHAL_SCAN=<folder>` | Measures a folder first, so the disk map opens filled in. |
| `DISKMARSHAL_DEV=<folder>` | Searches a folder for build folders first. |
| `DISKMARSHAL_DUP=<folder>` | Searches a folder for duplicates first. |
| `DISKMARSHAL_PHOTOS=<folder>` | Searches a folder for similar photos first. |
| `DISKMARSHAL_APPS=<folder>` | Lists the apps of that folder instead of the installed ones, and examines the first one, so the apps screen opens filled in. |
| `DISKMARSHAL_DOCKER=1` | Asks Docker first, so the Docker screen opens filled in. |
| `DISKMARSHAL_DEMO=1` | Replaces the readings of the monitor with made-up ones, and what Docker holds too. |

This build skips the start-up sequence and all motion, so the frame is still. It only reads: nothing is removed. Run it with `HOME` pointing at a demonstration folder to keep every personal path out of the picture.

## Updating the screenshots

The screenshots and the tour GIF in `docs/assets/` are produced by a script. It builds the app in the capture mode above and runs it once per screen, with `HOME` pointing at a demonstration home folder that the script creates. So the pictures show made-up caches, projects, duplicates and photos, and no path or program of the machine that takes them. Nothing in the real home folder is read or changed.

It needs cargo, [ffmpeg](https://ffmpeg.org) and the Python package [Pillow](https://python-pillow.org):

```bash
python3 docs/assets/capture/make_assets.py
```

It writes `docs/assets/screens/*.png` (one per screen), `docs/assets/palettes.png` and `docs/assets/tour.gif`. Run it after changing how a screen looks. Pass `--work-dir <folder>` to choose where the demonstration folder is created.

The app icon has its own script, which draws `apps/desktop/bundle/AppIcon.icns` and the preview `docs/assets/app-icon.png`:

```bash
python3 apps/desktop/bundle/make_icon.py
```

Captions in the documentation say that these are screenshots of the app with demonstration data.

## Reporting a vulnerability

Do not open a public issue. Follow [SECURITY.md](SECURITY.md).

## Conduct

Everyone taking part is expected to follow the [code of conduct](CODE_OF_CONDUCT.md).
