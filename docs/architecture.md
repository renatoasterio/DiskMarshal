# Architecture

DiskMarshal follows Clean Architecture (ports and adapters). Each layer is its
own crate, so the compiler rejects a dependency that points the wrong way.

## Layers

| Crate | Responsibility | May depend on |
|---|---|---|
| `diskmarshal-domain` | Entities, value objects, pure rules. No I/O. | nothing internal |
| `diskmarshal-application` | Use cases and ports (traits). | `domain` |
| `diskmarshal-infrastructure` | macOS adapters that implement the ports. | `application`, `domain` |
| `diskmarshal-ui` | Screens, components, theme, texts (egui). | `application`, `domain` |
| `apps/*` | Composition roots: wire adapters into use cases. | everything |
| `diskmarshal-test-support` | Fixtures for tests (`TempTree`, `bitmap_bytes`). | nothing internal |
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
| `diskmarshal` | `apps/desktop` | Every adapter into `UiServices`, then opens the window. The only binary that depends on `diskmarshal-ui`. |
| `diskmarshal-cli` | `apps/cli` | The adapters that only read into `ScanDisk`, `ListInstalledApps`, `PlanUninstall` and `InspectDocker`. It never removes anything. |
| `diskmarshal-privileged-helper` | `apps/privileged-helper` | The catalog, the walker, the inspector and the remover into `CleanRules`, for administrator rules only, and into `RemoveAppItems`, for the system items of an app being uninstalled. Runs once, as root, started by macOS after it authenticates the user. See [security.md](security.md#privileged-helper). |

The helper has no Trash and keeps no history: it plugs in `NoTrashBin` and
`NoHistory` (`apps/privileged-helper/src/unused_ports.rs`). The app records
what the helper reports in the history of the user.

`cargo xtask bundle` puts `diskmarshal` and `diskmarshal-privileged-helper`
side by side in `dist/DiskMarshal.app/Contents/MacOS/`, which is where the app
looks for the helper.

## What each crate holds

### `diskmarshal-domain`

| Module | Holds |
|---|---|
| `scan` | `FileTree`, `FileNode`, `CloudState`, `FileCategory`, `ScanPolicy`, `LocalFile` |
| `deletion` | `ProtectedPaths` (the path policy), `DeletionRequest`, `ItemFingerprint`, `DeletionOutcome`, `HistoryEntry` |
| `cleanup` | `CleanupRule`, `RiskLevel`, `PrivilegeScope`, `CleanupPlan` |
| `dev_artifacts` | `ArtifactRule`, `Ecosystem`, `ProjectArtifact`, `find_artifacts` |
| `applications` | `InstalledApp`, `LeftoverLocation`, `LeftoverKind`, `LeftoverMatcher` (the rule that says an item belongs to an app), `UninstallPlan` |
| `docker` | `DockerInventory`, `KindUsage`, `DockerResource`, `ResourceKind`, `EngineStatus` |
| `duplicates` | `DuplicateGroup`, `ContentHash`, `candidates_by_size` |
| `photos` | `PhotoSignature`, `SimilarityTolerance`, `SimilarityGroup`, `Thumbnail`, `photo_candidates` |
| `monitoring` | `SystemSnapshot`, `SensorReading`, `ThermalLevel`, `VolumeUsage`, `summarize_sensors` |
| `treemap` | `squarify`, `LayoutRect` |
| `shared` | `ByteSize`, number formatting |

### `diskmarshal-application`

Use cases, in `src/use_cases/`:

| Use case | What it does |
|---|---|
| `ScanDisk` | Measures a folder into a `FileTree`. |
| `PlanCleanup` | Finds out what each cleanup rule would remove. Removes nothing. |
| `FindDevArtifacts` | Finds build and dependency folders of code projects. |
| `FindDuplicates` | Finds files with identical content. |
| `FindSimilarPhotos` | Finds image files that look the same. |
| `DeleteItems` | Removes approved items. The only code path that deletes files. |
| `CleanRules` | Analyzes and cleans rules named by id, all of one privilege scope, with no preview. Chains `PlanCleanup` and `DeleteItems`. This is what the privileged helper runs. |
| `CleanSystemAreas` | Asks the `PrivilegedCleaner` port to clean administrator rules and records the result in the history. This is what the app runs. |
| `ListInstalledApps` | Lists the app bundles in the applications folders, with their size and whether they are open. |
| `PlanUninstall` | Finds what belongs to one app: the bundle and what it left in the library folders. Removes nothing. |
| `RemoveAppItems` | Finds an app by its identifier, plans its uninstall again and removes the requested items that its own plan also holds, for good. This is what the privileged helper runs for an uninstall. |
| `UninstallSystemItems` | Asks the `PrivilegedCleaner` port to remove the system items of an app and records the result in the history. This is what the app runs. |
| `InspectDocker` | Asks Docker what it holds and what of it nothing uses. Removes nothing. |
| `PruneDocker` | Asks Docker to remove approved resources, records them in the history and measures what Docker freed. |

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
| `AppBundleReader` | `PlistAppBundleReader` | `bundles` |
| `RunningAppProbe` | `SysinfoRunningAppProbe` | `system` |
| `FolderLister` | `ReaddirFolderLister` | `filesystem` |
| `ContainerEngine` | `DockerCliContainerEngine` | `process` |

### `diskmarshal-infrastructure`

One module per technology: `bundles`, `catalog`, `filesystem`, `hashing`, `imaging`,
`metrics`, `permissions`, `persistence`, `privilege`, `process`, `system`,
`trash`. Besides the adapters above:

- `bundles` exports `application_folders`, the folders where apps are
  installed.
- `filesystem` exports `disable_dataless_materialization`, the process-wide
  policy against iCloud downloads, and `icloud_sync_roots`.
- `privilege` exports the protocol between the app and the helper
  (`HelperRequest`, `HelperTask`, `encode_outcomes`, `decode_outcomes`): on the
  command line one way, either rule ids or an app identifier followed by its
  items; one line of JSON the other way.
- `system` exports `prefers_reduced_motion`, which reads the "Reduce motion"
  switch of System Settings.
- `process` holds `bounded_command`, which every adapter that runs another
  program with a time limit goes through.
- `resources/` holds the three catalogs, compiled into the binaries: cleanup
  rules, build-artifact rules and the folders where apps leave their items.

### `diskmarshal-ui`

| Module | Holds |
|---|---|
| `app` | The window frame: status bar, navigation rail, work area, overlays. `UiServices` lists every port the interface uses. |
| `screens` | One folder per screen, each with a `view` (drawing) and a `view_model` (state and logic): `dashboard`, `cleanup`, `apps`, `duplicates`, `photos`, `developer`, `docker`, `disk_map`, `monitor`. `group_review` is shared by duplicates and photos. |
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
| Domain | `crates/diskmarshal-domain/src/photos/` (`PhotoSignature`, `SimilarityGroup`, `SimilarityTolerance`) |
| Port | `crates/diskmarshal-application/src/ports/image_thumbnailer.rs` |
| Use case | `crates/diskmarshal-application/src/use_cases/find_similar_photos.rs` |
| Adapter | `crates/diskmarshal-infrastructure/src/imaging/image_io_thumbnailer.rs` |
| Screen | `crates/diskmarshal-ui/src/screens/photos/` (`view.rs`, `view_model.rs`) |
| Flow test | `crates/diskmarshal-infrastructure/tests/photos_flow.rs` |

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

Two screens run something else in steps 3 and 4, through the same component:

- The apps screen runs the administrator part first, with
  `UninstallSystemItems`, and only then `DeleteItems`. When the user closes
  the authentication dialog, nothing at all has been removed.
- The Docker screen has no files to delete. It runs `PruneDocker`, which asks
  Docker to remove each approved resource.

## Rules that hold everywhere

- **One deletion path.** Every removal goes through the `DeleteItems` use case.
  No other code deletes files. The helper reaches it through `CleanRules` and
  `RemoveAppItems`. What Docker holds is not files: `PruneDocker` asks Docker
  to remove it.
- **Cloud-only content is never read.** The first thing each binary does is
  call `disable_dataless_materialization`, so a scan cannot start an iCloud
  download. See [security.md](security.md).
- **Catalogs are data.** Cleanup rules, build-artifact rules and the folders
  where apps leave their items live in TOML
  under `crates/diskmarshal-infrastructure/resources/`, not in code.
- **`unsafe` stays in `infrastructure`.** The other library crates and every
  binary declare `#![forbid(unsafe_code)]`.
- **Long work runs off the interface thread.** Scans, searches, removals and
  readings run in `tasks`; a screen only polls for results.

Decisions and their reasons are recorded in [adr/](adr/).
