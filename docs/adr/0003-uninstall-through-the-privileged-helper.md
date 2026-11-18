# 0003 — Uninstall through the privileged helper, by identifier

Status: accepted (2026-11-17) and implemented. Not run as root yet, and the
authentication dialog was not exercised for it; see
[security.md](../security.md#uninstalling-with-administrator-rights).

## Context

More than half of the apps on the development machine (36 of 80) are owned by
root: the App Store and installer packages put them there. The logged-in user
cannot move such a bundle to the Trash, nor remove what the app left in
`/Library`. An uninstaller that handles only the apps the user owns leaves
most of the work undone.

[0002](0002-one-shot-privilege-elevation.md) gave the app one way to act as
an administrator: a helper that macOS runs once, which accepted rule ids and
nothing shaped like a path.

Options considered:

| Option | Why not |
|---|---|
| Uninstall only what the user owns | Half of the apps could not be uninstalled. |
| Ask Finder to move the items to the Trash, with AppleScript | Finder would authenticate the user and the items would be recoverable, but the app would need the permission to control Finder, and a second way to delete would exist beside `DeleteItems`. |
| Let the helper remove the paths it is given | The helper would do as root whatever a caller wrote. |

## Decision

The helper accepts a second form of request: the bundle identifier of an app,
followed by the items to remove. It then repeats the whole analysis by
itself: it finds the app with that identifier, plans its uninstall with the
catalog compiled into it, and removes a requested item only when its own plan
holds that item, as the bundle or as something found in an administrator
folder. One item outside the plan stops the request before anything is
touched. An app that is open, or an identifier two apps share, stops it too.

The paths the app sends therefore choose among what the helper found by
itself. They cannot add anything to it.

The items are removed for good. The helper has no Trash: it runs as root and
does not know, and must not be told, whose Trash to fill.

The app runs this part first. When the user closes the authentication
dialog, nothing has been removed, not even the items that need no
administrator.

## Consequences

- The helper now reads `Info.plist` files as root, with a parser written in
  safe Rust, to learn the identifier of each installed app.
- The helper lists `/Applications` on every uninstall, which takes a few
  seconds.
- An app in the personal applications folder of the user that is owned by
  root cannot be uninstalled: the helper does not look there.
- An uninstall with administrator rights cannot be undone from the Trash.
  The confirmation dialog says so.
- The matching rule is shared by the app and the helper
  (`LeftoverMatcher` in the domain), so a change to it changes what root may
  remove. It needs the same care as a change to deletion.
