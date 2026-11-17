# Reporting a vulnerability

MacCleaner deletes files and, in the future, will run a small part of itself with administrator rights. A flaw here can cost someone their data, so security reports are taken seriously.

## How to report

**Please do not open a public issue.** Report privately instead:

1. Open the repository's [Security tab](https://github.com/renatoasterio/MacCleaner/security).
2. Click **Report a vulnerability**.
3. Describe the problem and how to reproduce it.

Useful details: your macOS version, the MacCleaner commit you tested, the exact steps, and what happened compared with what should have happened.

This is a volunteer project. You will get a reply as soon as a maintainer can read the report, and credit in the release notes if you want it.

## What is most valuable

- A way to make MacCleaner delete something its [policy](docs/security.md) forbids: a protected location, a path outside the scanned folders, the inside of a package or a `.git` folder.
- A way to use a symbolic link, a renamed folder or a race between the preview and the removal to redirect a deletion.
- A way to make a scan download iCloud files.
- Any way to make MacCleaner open a network connection.
- When the privileged helper exists: any way to make it act on a path or a command that is not in its fixed list.

## Supported versions

MacCleaner has no stable release yet. Fixes go to the `main` branch.

## Known and accepted

The planned administrator cleanup uses one-shot elevation. Its residual risk is described in [docs/security.md](docs/security.md#privileged-helper) and in the [decision record](docs/adr/0002-one-shot-privilege-elevation.md).
