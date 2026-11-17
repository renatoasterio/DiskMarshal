# Security policy

MacCleaner deletes files, so a bug can destroy data. These rules exist to make
that hard.

## Least privilege

- The app always runs as the logged-in user. It opens no network connection
  and sends no telemetry.
- Administrator rights exist only inside `maccleaner-privileged-helper`, for
  one operation, after macOS itself authenticates the user with Touch ID or
  password. The app never sees or stores the password. See
  [adr/0002-one-shot-privilege-elevation.md](adr/0002-one-shot-privilege-elevation.md).

## Deletion

- One code path deletes: the `DeleteItems` use case.
- A target must be inside a scanned root and outside the protected list
  (`/`, `/System`, `/usr`, the home folder itself, `~/Library`, `~/Documents`,
  volume roots, the inside of any package, `.git` folders, the app itself).
- A target is revalidated (device, inode, size, modification time) right
  before removal. If it changed since the preview, it is skipped.
- Removal never follows symbolic links and never crosses to another device.
- Personal files go to the Trash. Only caches, temporary files and build
  folders are removed permanently, after a preview and a confirmation.
- Every removal is appended to
  `~/Library/Application Support/MacCleaner/history.jsonl` (mode 0600).

## iCloud

Reading a cloud-only file makes macOS download it. Each binary calls
`disable_dataless_materialization` before touching any file, so such a read
fails with `EDEADLK` instead. If the call fails, the binary stops without
scanning.

## Code

- `unsafe` is allowed only in `maccleaner-infrastructure`, in small modules.
  Every block has a `// SAFETY:` comment; the lint
  `clippy::undocumented_unsafe_blocks` enforces it.
- No shell. External programs (`git`, `xcrun`) run through one adapter, with an
  absolute path, an argument list, a clean environment and a timeout. The only
  exception is the fixed elevation command, which carries no variable data.
- Configuration files are parsed with unknown fields rejected.

## Supply chain

- `Cargo.lock` is committed and the toolchain is pinned in `rust-toolchain.toml`.
- `cargo deny check` runs in the quality gate: known vulnerabilities, yanked
  crates, license allow-list, and crates.io as the only source.
- Dependencies enable only the features they need.
- CI uses first-party GitHub Actions only, with read-only permissions.

## Privileged helper

- Accepts rule identifiers only. Paths come from the catalog embedded in the
  helper; an unknown identifier or an extra field rejects the whole request.
- Re-applies the full deletion policy and refuses SIP-protected paths, user
  home folders and volume roots.
- The app verifies that the helper carries its own code signature before
  asking macOS to run it.

Known, accepted risk: with one-shot elevation, malware already running as the
user could try to replace the helper between the signature check and its
execution. This is acceptable for personal use; distributing the app to others
requires moving to a daemon registered with `SMAppService`.
