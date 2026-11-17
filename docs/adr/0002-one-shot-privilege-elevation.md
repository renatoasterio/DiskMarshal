# 0002 — One-shot privilege elevation

Status: accepted (2026-11-17), pending a real test of the Touch ID prompt on macOS 27

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

Elevate once per operation with `NSAppleScript`
(`do shell script ... with administrator privileges`). macOS shows its own
authentication dialog and then runs `maccleaner-privileged-helper` as root for
that single request. Apple's guidance accepts this for limited-audience tools.

The helper accepts rule identifiers only, never paths or commands, and
re-applies the domain deletion policy.

## Consequences

- No root service stays installed or running.
- Every administrator cleanup asks for authentication again.
- The command line passed to the system is fixed; it is the single exception
  to the "no shell" rule.
- Residual risk and the migration path are described in
  [../security.md](../security.md).

## Open point

Confirm on macOS 27.2 that the dialog offers Touch ID. Locally verified so far:
the `system.privilege.admin` right uses the standard system authentication and
Touch ID is enabled on the target machine. If the dialog turns out to be
password-only, revisit the `SMAppService` option with a custom authorization right.
