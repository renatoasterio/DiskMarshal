# Architecture

MacCleaner follows Clean Architecture (ports and adapters). Each layer is its
own crate, so the compiler rejects a dependency that points the wrong way.

## Layers

| Crate | Responsibility | May depend on |
|---|---|---|
| `maccleaner-domain` | Entities, value objects, pure rules. No I/O. | nothing internal |
| `maccleaner-application` | Use cases and ports (traits). | `domain` |
| `maccleaner-infrastructure` | macOS adapters that implement the ports. | `application`, `domain` |
| `maccleaner-ui` | Screens, components, theme, texts (egui). | `application`, `domain` |
| `apps/*` | Composition roots: wire adapters into use cases. | everything |
| `maccleaner-test-support` | Fixtures for tests (`TempTree`, `bitmap_bytes`). | nothing internal |
| `xtask` | Project tasks: `check`, `bundle`, `install`. | nothing internal |

Dependencies always point inward:

```
apps ──► ui ──────────────► application ──► domain
  └────► infrastructure ──► application ──► domain
```

The interface never talks to the operating system. It calls a use case, and
the use case reaches macOS through a port whose adapter was injected by the
binary.

## Composition roots

There are three binaries. Each one builds the adapters it needs and hands
them to the use cases; none of them holds rules of its own.

| Binary | Path | What it wires |
|---|---|---|
| `maccleaner` | `apps/desktop` | Every adapter into `UiServices`, then opens the window. The only binary that depends on `maccleaner-ui`. |
| `maccleaner-cli` | `apps/cli` | The disk walker into `ScanDisk`. Read-only. |
| `maccleaner-privileged-helper` | `apps/privileged-helper` | The catalog, the walker, the inspector and the remover into `CleanRules`, for administrator rules only. Runs once, as root, started by macOS after it authenticates the user. See [security.md](security.md#privileged-helper). |

The helper has no Trash and keeps no history: it plugs in `NoTrashBin` and
`NoHistory` (`apps/privileged-helper/src/unused_ports.rs`). The app records
what the helper reports in the history of the user.

`cargo xtask bundle` puts `maccleaner` and `maccleaner-privileged-helper`
side by side in `dist/MacCleaner.app/Contents/MacOS/`, which is where the app
looks for the helper.

## What each crate holds

### `maccleaner-domain`

| Module | Holds |
|---|---|
| `scan` | `FileTree`, `FileNode`, `CloudState`, `FileCategory`, `ScanPolicy`, `LocalFile` |
| `deletion` | `ProtectedPaths` (the path policy), `DeletionRequest`, `ItemFingerprint`, `DeletionOutcome`, `HistoryEntry` |
| `cleanup` | `CleanupRule`, `RiskLevel`, `PrivilegeScope`, `CleanupPlan` |
| `dev_artifacts` | `ArtifactRule`, `Ecosystem`, `ProjectArtifact`, `find_artifacts` |
| `duplicates` | `DuplicateGroup`, `ContentHash`, `candidates_by_size` |
| `photos` | `PhotoSignature`, `SimilarityTolerance`, `SimilarityGroup`, `Thumbnail`, `photo_candidates` |
| `monitoring` | `SystemSnapshot`, `SensorReading`, `ThermalLevel`, `VolumeUsage`, `summarize_sensors` |
| `treemap` | `squarify`, `LayoutRect` |
| `shared` | `ByteSize`, number formatting |

### `maccleaner-application`

Use cases, in `src/use_cases/`:

| Use case | What it does |
|---|---|
| `ScanDisk` | Measures a folder into a `FileTree`. |
| `PlanCleanup` | Finds out what each cleanup rule would remove. Removes nothing. |
| `FindDevArtifacts` | Finds build and dependency folders of code projects. |
| `FindDuplicates` | Finds files with identical content. |
| `FindSimilarPhotos` | Finds image files that look the same. |
| `DeleteItems` | Removes approved items. The only code path that deletes. |
| `CleanRules` | Analyzes and cleans rules named by id, all of one privilege scope, with no preview. Chains `PlanCleanup` and `DeleteItems`. This is what the privileged helper runs. |
| `CleanSystemAreas` | Asks the `PrivilegedCleaner` port to clean administrator rules and records the result in the history. This is what the app runs. |

Ports, in `src/ports/`, with the adapter that implements each one:

| Port | Adapter | Infrastructure module |
|---|---|---|
| `FileSystemWalker` | `ReaddirWalker` | `filesystem` |
| `FileInspector` | `LstatFileInspector` | `filesystem` |
| `FileRemover` | `SafeFileRemover` | `filesystem` |
| `VolumeStats` | `StatfsVolumeStats` | `filesystem` |
| `TrashBin` | `NsFileManagerTrashBin` | `trash` |
| `HistoryRepository` | `JsonlHistoryRepository` | `persistence` |
| `RuleCatalog` | `TomlRuleCatalog` | `catalog` |
| `ContentHasher` | `Blake3ContentHasher` | `hashing` |
| `ImageThumbnailer` | `ImageIoThumbnailer` | `imaging` |
| `MetricsProvider` | `SysinfoMetricsProvider`, `UnavailableMetrics` | `metrics` |
| `PermissionProbe` | `FullDiskAccessProbe` | `permissions` |
| `PrivilegedCleaner` | `OsascriptPrivilegedCleaner` | `privilege` |
| `VersionControlProbe` | `GitVersionControlProbe` | `process` |
| `ExternalOpener` | `OpenCommandOpener` | `system` |
| `FolderPicker` | `NativeFolderPicker` | `system` |
| `WallClock` | `LocaltimeWallClock` | `system` |

### `maccleaner-infrastructure`

One module per technology: `catalog`, `filesystem`, `hashing`, `imaging`,
`metrics`, `permissions`, `persistence`, `privilege`, `process`, `system`,
`trash`. Besides the adapters above:

- `filesystem` exports `disable_dataless_materialization`, the process-wide
  policy against iCloud downloads, and `icloud_sync_roots`.
- `privilege` exports the protocol between the app and the helper
  (`HelperRequest`, `encode_outcomes`, `decode_outcomes`): rule ids on the
  command line one way, one line of JSON the other way.
- `system` exports `prefers_reduced_motion`, which reads the "Reduce motion"
  switch of System Settings.
- `resources/` holds the two rule catalogs, compiled into the binaries.

### `maccleaner-ui`

| Module | Holds |
|---|---|
| `app` | The window frame: status bar, navigation rail, work area, overlays. `UiServices` lists every port the interface uses. |
| `screens` | One folder per screen, each with a `view` (drawing) and a `view_model` (state and logic): `dashboard`, `cleanup`, `duplicates`, `photos`, `developer`, `disk_map`, `monitor`. `group_review` is shared by duplicates and photos. |
| `components` | Pieces reused by the screens: panels, buttons, selection rows, gauges, the removal flow, dialogs. |
| `tasks` | Work that runs off the interface thread: `Task`, `Search`, `Feed`. |
| `theme`, `effects`, `animation` | Tokens, palettes, fonts, glow, grid, motion levels. |
| `i18n` | Every text shown to the user (`pt_br.rs`). |
| `navigation` | The `Screen` enum, its labels and shortcuts. |

## How a feature is laid out

A feature keeps the same name in every layer, so its code is easy to find.
Similar photos, for example:

| Layer | Path |
|---|---|
| Domain | `crates/maccleaner-domain/src/photos/` (`PhotoSignature`, `SimilarityGroup`, `SimilarityTolerance`) |
| Port | `crates/maccleaner-application/src/ports/image_thumbnailer.rs` |
| Use case | `crates/maccleaner-application/src/use_cases/find_similar_photos.rs` |
| Adapter | `crates/maccleaner-infrastructure/src/imaging/image_io_thumbnailer.rs` |
| Screen | `crates/maccleaner-ui/src/screens/photos/` (`view.rs`, `view_model.rs`) |
| Flow test | `crates/maccleaner-infrastructure/tests/photos_flow.rs` |

## How a removal flows

Every screen that removes uses the same component,
`components/removal_flow.rs`:

1. The screen builds `DeletionRequest`s from what a use case found. Each one
   carries the fingerprint of the item as it was in the preview.
2. The user confirms in the dialog.
3. `DeleteItems` runs in a background task. For each request it resolves the
   path, applies `ProtectedPaths`, compares the fingerprint, and only then
   removes or moves to the Trash. Each removal is appended to the history.
4. If administrator rules were selected, `CleanSystemAreas` then calls the
   `PrivilegedCleaner` port. Its adapter asks macOS to authenticate the user
   and to run the helper, which executes `CleanRules`.
5. The flow shows the `DeletionReport`.

## Rules that hold everywhere

- **One deletion path.** Every removal goes through the `DeleteItems` use case.
  No other code deletes files. The helper reaches it through `CleanRules`.
- **Cloud-only content is never read.** The first thing each binary does is
  call `disable_dataless_materialization`, so a scan cannot start an iCloud
  download. See [security.md](security.md).
- **Catalogs are data.** Cleanup rules and build-artifact rules live in TOML
  under `crates/maccleaner-infrastructure/resources/`, not in code.
- **`unsafe` stays in `infrastructure`.** The other library crates and every
  binary declare `#![forbid(unsafe_code)]`.
- **Long work runs off the interface thread.** Scans, searches, removals and
  readings run in `tasks`; a screen only polls for results.

Decisions and their reasons are recorded in [adr/](adr/).
