# 0004 — Network changes through the privileged helper, as fixed forms

Status: accepted (2026-11-18) and implemented. Not confirmed on the real
thing: see [security.md](../security.md#changing-network-settings).

## Context

The network screen finds things that slow a connection down on the Mac. Three
of them can be fixed by changing a setting of macOS, and each needs an
administrator: the search for a proxy before every new connection, stale
entries in the name cache, and a slow name server.

The app already has one way to do something as an administrator: macOS
authenticates the user and runs `diskmarshal-privileged-helper` once
([0002](0002-one-shot-privilege-elevation.md)). Until now the helper only
removed files, and everything it accepts chooses among what it finds by
itself.

## Decision

The helper takes a third form of request, `--network`, that names what to
change and never how:

- A request is a list of changes, each one of three fixed forms: flush the
  name cache, turn the proxy search of a service on or off, set the name
  servers of a service. There is no form that carries a command, a path or
  an option.
- The helper turns each change into the one command for it
  (`networksetup`, `dscacheutil`, `killall -HUP mDNSResponder`), started
  with an absolute path and an argument list.
- A service is accepted only when macOS lists it; a server only when it
  parses as an IP address.
- Every change the app offers carries the change that takes it back, built
  from what the analysis read before anything was touched.
- The app offers changes; it applies none without a confirmation, and
  selects by default only the ones that cannot take anything away from the
  user.

The rule lives in the application layer (`ChangeNetworkSettings`, behind the
`NetworkSettings` port), not in the helper binary, so it is tested without
an administrator.

## Alternatives considered

- **Running `networksetup` from the app, as the user.** An administrator
  account can change some of these settings without a password, but a
  standard account cannot, and flushing the name cache needs root either
  way. Two paths for the same thing would have to be kept in step.
- **A general "run this command as root" request.** One request form for
  everything is the smallest code and the largest hole: whatever can make
  the app ask would run anything.
- **Tuning TCP parameters with `sysctl`.** Not done. On current macOS these
  settings bring no gain that could be measured here, do not survive a
  restart, and can make a connection worse.
- **Asking a program to quit when it takes most of the connection.** Left
  out for now. The screen points the program out; closing it stays with the
  user, who knows whether it has unsaved work.

## Consequences

- The helper can now change settings as well as remove files. What it can
  set is three settings, each visible in System Settings.
- A change that breaks a connection (a proxy that was needed, a name server
  that knew internal names) can be taken back in the app while it is open,
  and in System Settings at any time.
- What was applied is not kept between runs of the app.
- Each request authenticates again, like every other use of the helper.
