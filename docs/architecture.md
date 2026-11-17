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
| `maccleaner-test-support` | Fixtures and test doubles. | `application`, `domain` |

Dependencies always point inward:

```
apps ──► ui ──────────────► application ──► domain
  └────► infrastructure ──► application ──► domain
```

The interface never talks to the operating system. It calls a use case, and
the use case reaches macOS through a port whose adapter was injected by the
binary.

## How a feature is laid out

A feature keeps the same name in every layer, so its code is easy to find.
The disk scan, for example:

| Layer | Path |
|---|---|
| Domain | `crates/maccleaner-domain/src/scan/` (`FileTree`, `CloudState`, `ScanPolicy`) |
| Port | `crates/maccleaner-application/src/ports/file_system_walker.rs` |
| Use case | `crates/maccleaner-application/src/use_cases/scan_disk.rs` |
| Adapter | `crates/maccleaner-infrastructure/src/filesystem/readdir_walker.rs` |
| Entry point | `apps/cli/src/commands/scan.rs` |

## Rules that hold everywhere

- **One deletion path.** Every removal goes through the `DeleteItems` use case.
  No other code deletes files.
- **Cloud-only content is never read.** The first thing each binary does is
  call `disable_dataless_materialization`, so a scan cannot start an iCloud
  download. See [security.md](security.md).
- **Catalogs are data.** Cleanup rules and build-artifact rules live in TOML
  under `crates/maccleaner-infrastructure/resources/`, not in code.
- **`unsafe` stays in `infrastructure`.** The other library crates declare
  `#![forbid(unsafe_code)]`.

Decisions and their reasons are recorded in [adr/](adr/).
