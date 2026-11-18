# 0002 — One-shot privilege elevation

Status: accepted (2026-11-17) and implemented. The macOS authentication dialog
and a real system cleanup have not been exercised end to end yet; see
[Not verified](#not-verified).

## Context

Some cleanup targets (`/Library/Caches`, `/Library/Logs`, old logs in
`/private/var/log`, local Time Machine snapshots) need administrator rights.
The user must authenticate with the native macOS dialog, by Touch ID or
password, and the app must never handle the password.

Options considered:

| Option | Why not |
|---|---|
| Daemon registered with `SMAppService` + XPC | Needs an Apple-issued signing identity and approval in System Settings; leaves a root service installed. Right choice for wide distribution, heavy for a personal tool. |
| `AuthorizationExecuteWithPrivileges` | Deprecated since macOS 10.7. |
| `sudo` | Meant for interactive terminals, not for use as an API. |

## Decision

Elevate once per operation with AppleScript's
`do shell script ... with administrator privileges`. macOS shows its own
authentication dialog and then runs `diskmarshal-privileged-helper` as root for
that single request. Apple's guidance accepts this for limited-audience tools.

The helper accepts rule identifiers only, never paths or commands, and
re-applies the domain deletion policy.

[0003](0003-uninstall-through-the-privileged-helper.md) later added a second
form of request, for uninstalling an app. It keeps the same principle: the
helper works out by itself what may be removed.

## How it was implemented

The decision stands. Two details differ from what was first written here:

| First written | Implemented | What that means |
|---|---|---|
| Run the script in-process with `NSAppleScript`. | Run `/usr/bin/osascript` as a child process, with a fixed script passed as `-e` arguments. | No Objective-C bridge and no `unsafe` code are involved. The call looks like every other external program the app starts: absolute path, argument list, cleared environment. |
| Hand the request to the helper in a request file. | Pass the rule ids on the helper's command line. | No request file exists for another process to swap or leave behind. An id is lowercase words joined by hyphens, so nothing shaped like a path or an option gets through. |

The rest, as built:

- The helper is a third binary, `apps/privileged-helper`. `cargo xtask bundle`
  places it beside the main executable inside `DiskMarshal.app`, and signs it
  before sealing the bundle.
- The app looks for the helper beside its own executable. Before elevating it
  checks that the helper is a regular file (not a link), executable, and
  writable only by its owner; inside an app bundle it also requires
  `codesign --verify --strict` to accept the bundle.
- The script is a constant. The text for the dialog, the helper path and the
  rule ids arrive as arguments, and AppleScript quotes each word of the
  command before the shell sees it.
- The helper takes the folders from the catalog compiled into it, accepts
  only rules whose scope is `administrator`, analyzes them again, and deletes
  through `CleanRules`, which chains `PlanCleanup` and `DeleteItems`: the same
  policy and use cases as the app.
- The helper answers with one line of JSON listing what happened to each
  item, and exits. The app records the removed items in the history of the
  user (`CleanSystemAreas`).
- The helper has a simulation mode, `--dry-run`, that runs every check and
  removes nothing.
- Only `/Library/Caches` and `/Library/Logs` have rules today. Old logs in
  `/private/var/log` and local Time Machine snapshots, mentioned in the
  context above, are not cleaned.

## Consequences

- No root service stays installed or running.
- Every administrator cleanup asks for authentication again.
- The command line passed to the system is built by a fixed script; it is the
  single exception to the "no shell" rule.
- System cleanup works only where the helper sits beside the executable, which
  in practice means the app built with `cargo xtask bundle` or
  `cargo xtask install`.
- The helper acts on its own analysis of the rule's folders, not on the
  preview the user saw in the app.
- Residual risk and the migration path are described in
  [../security.md](../security.md#privileged-helper), and accepted: malware
  already running as the user could try to replace the helper between the
  check and its execution. Distributing the app to others requires moving to
  a daemon registered with `SMAppService`.

## Not verified

- The macOS authentication dialog and a real cleanup of `/Library/Caches` and
  `/Library/Logs` were not exercised end to end by the project's tests. The
  helper is covered by tests on temporary folders and by its simulation mode.
- Whether the dialog offers Touch ID or only the password is decided by
  macOS. Verified locally when the decision was made: the
  `system.privilege.admin` right uses the standard system authentication and
  Touch ID was enabled on the target machine. If the dialog turns out to be
  password-only, revisit the `SMAppService` option with a custom authorization
  right.
