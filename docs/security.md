# Security policy

MacCleaner deletes files, so a bug can destroy data. These rules exist to make
that hard. The plain-language version for users is the
[safety guide](guide/safety.md).

## Least privilege

- The app always runs as the logged-in user. It opens no network connection
  and sends no telemetry.
- The app never runs as root and keeps no background service.
- Administrator rights exist only inside `maccleaner-privileged-helper`, for
  one operation, after macOS itself authenticates the user. The app never sees
  or stores the password. See [Privileged helper](#privileged-helper) and
  [adr/0002-one-shot-privilege-elevation.md](adr/0002-one-shot-privilege-elevation.md).

## Deletion

- One code path deletes: the `DeleteItems` use case. Every screen reaches it
  through the same component (`components/removal_flow.rs`); the privileged
  helper reaches it through `CleanRules`.
- A target must be an absolute path with no `..`, inside one of the allowed
  roots (the folder that was searched, or the folders of the cleanup rules)
  and not a root itself.
- A target must be outside the protected list
  (`deletion/protected_paths.rs`):
  - anything inside `/System`, `/usr`, `/bin` or `/sbin`;
  - these locations themselves: `/`, `/Applications`, `/Library`,
    `/Library/Caches`, `/Library/Logs`, `/Users`, `/Volumes`, `/cores`,
    `/etc`, `/opt`, `/private`, `/private/tmp`, `/private/var`, `/tmp`;
  - these locations themselves, inside the home folder: the home folder,
    `.Trash`, `Applications`, `Desktop`, `Documents`, `Downloads`, `Library`,
    `Library/Application Support`, `Library/Caches`, `Library/Containers`,
    `Library/Developer`, `Library/Group Containers`, `Library/Logs`,
    `Library/Mobile Documents`, `Library/Preferences`, `Movies`, `Music`,
    `Pictures`, `Public`, `Sites`;
  - a `.git` folder and anything inside one;
  - anything inside a package (`.app`, `.photoslibrary`, `.framework`,
    `.sparsebundle` and the other extensions in `ScanPolicy`).
- Paths are compared without regard to letter case.
- A path is resolved on disk before the policy sees it, so a symbolic link in
  a parent folder cannot disguise where the target really is.
- A target is revalidated (device, inode, size, modification time) right
  before removal. If it changed since the preview, it is skipped.
- Removal never follows symbolic links and never crosses to another device: a
  tree that contains a folder on another device is refused as a whole.
- Personal files go to the Trash: duplicates, similar photos and Xcode
  archives. Caches, temporary files, logs, build folders and the content of
  the Trash are removed permanently. Both happen only after a preview and a
  confirmation.
- A request that fails any check is skipped and reported, never forced. One
  failure does not stop the others.
- Every removal is appended to
  `~/Library/Application Support/MacCleaner/history.jsonl` (file mode 0600,
  folder mode 0700).

Each screen adds its own guard on top:

- Duplicates and similar photos refuse to mark the last file of a group.
- A build folder is offered only when its marker file is beside it and git
  has no tracked files in it. When git cannot answer, the probe answers
  "tracked", so the folder is not offered.
- Cleanup rules remove the items directly inside their folders, never the
  folders. Rules with risk `review` and rules with scope `administrator` are
  never selected by default.

## iCloud

Reading a cloud-only file makes macOS download it. Each of the three binaries
calls `disable_dataless_materialization` before touching any file, so such a
read fails with `EDEADLK` instead. If the call fails, the binary stops without
scanning.

The searches that read content (duplicates, similar photos) take their files
from `local_files`, which leaves cloud-only files out.

Not verified: this was tested with files stored on the Mac, not with a file
that really is cloud-only.

## Code

- `unsafe` is allowed only in `maccleaner-infrastructure`, in small modules.
  Every block has a `// SAFETY:` comment; the lint
  `clippy::undocumented_unsafe_blocks` enforces it. The other library crates
  and all binaries declare `#![forbid(unsafe_code)]`.
- No shell. External programs (`/usr/bin/git`, `/usr/bin/open`,
  `/usr/bin/defaults`, `/usr/bin/codesign`, `/usr/bin/osascript`) are started
  with an absolute path, an argument list, a cleared environment and no
  standard input. The call to `git` also has a timeout.
- The only exception is the elevation: AppleScript's `do shell script` hands
  the helper's command line to a shell. The script is a constant; the helper
  path and the rule ids arrive as arguments and each one is quoted by
  AppleScript (`quoted form of`) before the shell sees it; and the rule ids
  are checked first to be lowercase words joined by hyphens.
- Configuration files are parsed with unknown fields rejected. So is the
  answer the helper sends back.

## Supply chain

- `Cargo.lock` is committed and the toolchain is pinned in `rust-toolchain.toml`.
- `cargo deny check` runs in the quality gate: known vulnerabilities, yanked
  crates, license allow-list, and crates.io as the only source.
- Dependencies enable only the features they need.
- CI uses first-party GitHub Actions only, with read-only permissions.

## Privileged helper

Two cleanup rules have the scope `administrator`: `system-caches`
(`/Library/Caches`) and `system-logs` (`/Library/Logs`). Cleaning them works
like this.

In the app (`privilege/osascript_privileged_cleaner.rs`):

1. The helper is looked for beside the running executable. In the installed
   app that is `MacCleaner.app/Contents/MacOS/maccleaner-privileged-helper`.
   Without it, the "Sistema" group is disabled.
2. Before elevating, the app checks the helper again: it must be a regular
   file, not a link, executable, and writable only by its owner. When the
   helper is inside an app bundle, `/usr/bin/codesign --verify --strict` must
   accept the bundle. Outside a bundle, as in a development build, there is no
   signature to check.
3. The rule ids are validated. Then the app runs `/usr/bin/osascript` with a
   fixed script that asks macOS to authenticate the user and to run the helper
   with those ids. macOS shows its own dialog; the app never receives the
   password or the fingerprint.
4. If the user closes the dialog, nothing is cleaned and the app reports the
   authentication as cancelled.

In the helper (`apps/privileged-helper/src/main.rs`):

1. It turns off iCloud downloads, like every binary.
2. It accepts only rule ids on its command line, plus `--dry-run`. An id is
   lowercase words joined by hyphens, at most 64 characters; at most 16 ids;
   none repeated. Anything shaped like a path or an option rejects the whole
   request, and the error names nothing the caller sent.
3. The folders come from the catalog compiled into the helper, never from the
   caller.
4. `CleanRules` accepts only ids that name a rule of that catalog whose scope
   is `administrator`. One id that does not stops the cleanup before anything
   is touched.
5. It analyzes the rules again and deletes through `DeleteItems`, with the
   same `ProtectedPaths` policy and the same revalidation as the app. The
   allowed roots are the folders of the requested rules.
6. It has no Trash and writes no history. It prints one line of JSON with what
   happened to each item (deleted, missing, changed, protected, failed) and
   exits.

Back in the app, `CleanSystemAreas` records every removed item in the history
of the user.

Things that follow from this design:

- No root service is installed or left running. Every administrator cleanup
  authenticates again.
- The helper works from its own analysis, not from the preview the user saw.
  It removes what the rule's folders hold at that moment.
- Paths protected by System Integrity Protection cannot be removed even as
  root; they come back as failed items.
- `--dry-run` runs every check and removes nothing. It is how the helper can
  be exercised without changing the system.

Not verified: the macOS authentication dialog and a real cleanup of
`/Library/Caches` and `/Library/Logs` were not exercised end to end by the
project's tests. The helper's logic is covered by tests on temporary folders
(`tests/system_cleanup_flow.rs`, `privilege/helper_protocol.rs`) and by the
simulation mode. Whether the dialog offers Touch ID or only the password is
decided by macOS.

Known, accepted risk: with one-shot elevation, malware already running as the
user could try to replace the helper between the check and its execution. The
signature check confirms that the bundle is intact, not who signed it: with
the default ad hoc signature, a program able to rewrite the bundle could sign
it again. This is acceptable for personal use; distributing the app to others
requires an Apple-issued signing identity and moving to a daemon registered
with `SMAppService`.
