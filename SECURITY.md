# Reporting a vulnerability

DiskMarshal deletes files and runs a small part of itself, the privileged helper, with administrator rights when you clean system areas. A flaw here can cost someone their data, so security reports are taken seriously.

## How to report

**Please do not open a public issue.** Report privately instead:

1. Open the repository's [Security tab](https://github.com/renatoasterio/DiskMarshal/security).
2. Click **Report a vulnerability**.
3. Describe the problem and how to reproduce it.

Useful details: your macOS version, the DiskMarshal commit you tested, the exact steps, and what happened compared with what should have happened.

This is a volunteer project. You will get a reply as soon as a maintainer can read the report, and credit in the release notes if you want it.

## What is most valuable

- A way to make DiskMarshal delete something its [policy](docs/security.md) forbids: a protected location, a path outside the searched folders, the inside of a package or a `.git` folder.
- A way to use a symbolic link, a renamed folder or a race between the preview and the removal to redirect a deletion.
- A way to make a scan download iCloud files.
- Any way to make DiskMarshal open a network connection.
- Any way to make the privileged helper act on a path, a rule or a command that is not in the catalog compiled into it, or to make the app run something other than its own helper with administrator rights.
- Any way for the app to read the password typed in the macOS authentication dialog.

## Supported versions

DiskMarshal has no stable release yet. Fixes go to the `main` branch.

## Known and accepted

The administrator cleanup uses one-shot elevation: macOS authenticates the user and runs the helper once. Its residual risk is that malware already running as the user could try to replace the helper between the app's check and its execution. This is described in [docs/security.md](docs/security.md#privileged-helper) and in the [decision record](docs/adr/0002-one-shot-privilege-elevation.md), and is accepted for personal use.

## Not verified yet

A system cleanup of `/Library/Logs` with the fingerprint, and the uninstall of two real apps behind the macOS password dialog, were confirmed by hand on one Mac (macOS 27.2 on Apple Silicon). A real cleanup of `/Library/Caches`, the password dialog for the system cleanup, the password dialog after a cancelled fingerprint and the removal of an app's items from `/Library` were not exercised on the real thing. The automated tests cover the helper on temporary folders and its simulation mode (`--dry-run`); they cannot authenticate. A report from someone who tried this path on a real Mac is welcome, whether it worked or not.
