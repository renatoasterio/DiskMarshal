# Security policy

DiskMarshal deletes files, so a bug can destroy data. These rules exist to make
that hard. The plain-language version for users is the
[safety guide](guide/safety.md).

## Least privilege

- The app always runs as the logged-in user and sends no telemetry. It links
  no network code: when the user asks for an analysis of the connection or a
  speed test, programs that ship with macOS send the packets. See
  [Network](#network). Docker is asked through its own program, which talks
  to the engine on this Mac.
- The app never runs as root and keeps no background service.
- Administrator rights exist only inside `diskmarshal-privileged-helper`, for
  one operation, after macOS itself authenticates the user. The app never sees
  or stores the password. See [Privileged helper](#privileged-helper) and
  [adr/0002-one-shot-privilege-elevation.md](adr/0002-one-shot-privilege-elevation.md).

## Deletion

- One code path deletes: the `DeleteItems` use case. Every screen reaches it
  through the same component (`components/removal_flow.rs`); the privileged
  helper reaches it through `CleanRules` and `RemoveAppItems`.
- What Docker holds is the one thing removed another way. It is not made of
  files this app can reach, so the `PruneDocker` use case asks Docker to remove
  each approved resource, through the `ContainerEngine` port. Nothing is
  forced: Docker refuses a resource that is in use, and it is reported as left.
- A target must be an absolute path with no `..`, inside one of the allowed
  roots (the folder that was searched, the folders of the cleanup rules, or,
  for an uninstall, the folder of the app and the folders it left items in)
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
- Personal files go to the Trash: duplicates, similar photos, Xcode
  archives, and what an uninstall removes from the account of the user.
  Caches, temporary files, logs, build folders and the content of the Trash
  are removed permanently. So is what an uninstall removes with
  administrator rights. Both happen only after a preview and a
  confirmation.
- A request that fails any check is skipped and reported, never forced. One
  failure does not stop the others.
- Every removal is appended to
  `~/Library/Application Support/DiskMarshal/history.jsonl` (file mode 0600,
  folder mode 0700).

Each screen adds its own guard on top:

- Duplicates and similar photos refuse to mark the last file of a group.
- A build folder is offered only when its marker file is beside it and git
  has no tracked files in it. When git cannot answer, the probe answers
  "tracked", so the folder is not offered.
- The uninstaller refuses an app that is running, when it is examined and
  again right before the removal. An item belongs to an app only when its name
  is the app's bundle identifier, alone or followed by a dot, or the name of
  the app in the folders the catalog allows that for. Only exact identifier
  matches are selected by default. An item that carries the longer identifier
  of another installed app is left to that app. When another copy with the
  same identifier is installed, only the bundle is offered. And nothing is
  selected by default unless the bundle itself can be removed.
- A Docker resource is listed only when no container uses it. The build cache
  and images without a name are the only ones selected by default.
- Cleanup rules remove the items directly inside their folders, never the
  folders. Rules with risk `review` and rules with scope `administrator` are
  never selected by default.

## Network

The network screen does three things, with three different reaches.

**Reading the traffic** (`NettopTrafficProbe`). It runs `/usr/bin/nettop`,
which ships with macOS, once per reading, as the logged-in user, and reads
the counters the system keeps for every open connection: addresses, ports,
bytes each way, bytes sent again and the average answer time.

- No packet is captured. The content of what travels is never read.
- `nettop` is asked for addresses as numbers (`-n`), so reading the traffic
  looks up no name on the network.
- Nothing is written to disk and nothing is sent anywhere. The readings live
  in memory, and are taken only while the network screen is open.

**Measuring the connection**, only when the user clicks a button.

- The analysis (`AnalyzeNetwork`) reads settings with `/sbin/route`,
  `/usr/sbin/networksetup`, `/usr/sbin/ipconfig`, `/usr/sbin/scutil` and
  `/usr/sbin/system_profiler`, then uses the network: ten pings to the router
  and ten to `1.1.1.1` (`/sbin/ping`, `/sbin/ping6`), and one question, asked
  twice, to each name server in use and to one address of each public service
  in `PUBLIC_RESOLVERS` (`/usr/bin/dig`, for `example.com`). When macOS is set
  to look for a proxy, `/usr/bin/dscacheutil` asks once whether the name
  `wpad` resolves, which is the question macOS itself is already asking.
- The speed test (`NetworkQualitySpeedTester`) runs `/usr/bin/networkQuality`,
  which talks to Apple's servers and moves real data: more than a gigabyte
  on a fast connection. The interface says so before the button is clicked.
- Neither carries anything about the user or the disk. The app links no
  network code of its own: every packet is sent by a program of macOS.

**Changing settings**, through the privileged helper, after a confirmation
and after macOS authenticates the user. See
[Changing network settings](#changing-network-settings) and
[adr/0004-network-changes-through-the-privileged-helper.md](adr/0004-network-changes-through-the-privileged-helper.md).

Verified by hand, on one Mac (macOS 27.2 on Apple Silicon): the traffic of
that Mac was listed as an ordinary user, in the app and in
`diskmarshal-cli network`; the analysis read the settings, the wireless link
and the answer times of that network; `networkQuality` ran with a cleared
environment and its answer was read. Not verified: the speed test button
itself; other macOS versions, where these programs may print in another
form (a report that cannot be read is refused or left empty, never guessed
at); a Mac with no way out to a network, a connected VPN, and a network that
answers the search for a proxy.

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

- `unsafe` is allowed only in `diskmarshal-infrastructure`, in small modules.
  Every block has a `// SAFETY:` comment; the lint
  `clippy::undocumented_unsafe_blocks` enforces it. The other library crates
  and all binaries declare `#![forbid(unsafe_code)]`.
- No shell. External programs (`/usr/bin/git`, `/usr/bin/open`,
  `/usr/bin/defaults`, `/usr/bin/codesign`, `/usr/bin/osascript`, the
  programs named under [Network](#network), and Docker's
  `docker`, looked for only in the fixed places its installers use) are
  started with an absolute path, an argument list, a cleared environment and
  no standard input. Docker receives `HOME`, where it keeps the address of its
  engine, and nothing else. A name passed to Docker comes after `--`, so it is
  never read as an option. The calls to `git` and `docker` have a timeout.
- The only exception is the elevation: AppleScript's `do shell script` hands
  the helper's command line to a shell. The script is a constant; the helper
  path and what it is asked arrive as arguments and each one is quoted by
  AppleScript (`quoted form of`) before the shell sees it; and every argument
  is checked first: a rule id is lowercase words joined by hyphens, a bundle
  identifier is letters, digits, dots, hyphens and underscores, and an item
  is an absolute path with no `.` or `..` and no control character.
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

In the app (`privilege/macos_privileged_cleaner.rs` and the two launchers beside it):

1. The helper is looked for beside the running executable. In the installed
   app that is `DiskMarshal.app/Contents/MacOS/diskmarshal-privileged-helper`.
   Without it, the "Sistema" group is disabled.
2. Before elevating, the app checks the helper again: it must be a regular
   file, not a link, executable, and writable only by its owner. When the
   helper is inside an app bundle, `/usr/bin/codesign --verify --strict` must
   accept the bundle. Outside a bundle, as in a development build, there is no
   signature to check.
3. The rule ids are validated. Then the app asks macOS to authenticate the
   user and to run the helper with those ids, in one of two ways. Either way
   macOS shows its own dialog and the app never receives the password or the
   fingerprint.
   - **Fingerprint, through `sudo`** (`privilege/sudo_launch.rs`). Used only
     when Touch ID for `sudo` is on: an active `auth … pam_tid.so` line in
     `/etc/pam.d/sudo_local` or `/etc/pam.d/sudo`. That is a setting of the
     Mac, off by default; the app reads it and never writes it. The app runs
     `/usr/bin/sudo -S -k -p "" -- helper ids…` with a cleared environment
     and a closed standard input. `-k` means an earlier authentication is
     never reused. `-S` with nothing to read means `sudo` cannot wait for a
     password at a terminal: without the fingerprint it fails at once.
   - **Password, through AppleScript** (`privilege/osascript_launch.rs`).
     Used when Touch ID for `sudo` is off, and whenever `sudo` did not
     authenticate the user (fingerprint refused, cancelled or unavailable).
     The app runs `/usr/bin/osascript` with a fixed script; the dialog text,
     the helper and the ids arrive as arguments and AppleScript quotes each
     word before the shell sees it.
4. If the user closes the password dialog, nothing is cleaned and the app
   reports the authentication as cancelled.

Turning on Touch ID for `sudo` is the user's decision and applies to every
use of `sudo` on that Mac, not only to DiskMarshal. It does not widen what
the helper accepts: the same checks run whichever way it is started.

In the helper (`apps/privileged-helper/src/main.rs`):

1. It turns off iCloud downloads, like every binary.
2. For a cleanup it accepts only rule ids on its command line, plus
   `--dry-run`. An id is lowercase words joined by hyphens, at most 64
   characters; at most 16 ids; none repeated. Anything shaped like a path or
   an option rejects the whole request, and the error names nothing the
   caller sent. The other two forms it accepts, `--uninstall` and
   `--network`, are described in
   [Uninstalling with administrator rights](#uninstalling-with-administrator-rights)
   and [Changing network settings](#changing-network-settings).
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

### Uninstalling with administrator rights

An app the App Store or an installer put on the Mac is owned by root, and so
is what it left in `/Library`. The user cannot move those to the Trash, so
the app asks the helper, with a second form of request:
`--uninstall <bundle identifier> <item>...`.

In the app, `UninstallSystemItems` sends the identifier of the app and the
paths the user approved. Both are validated before macOS is asked to
authenticate anyone. This part runs before anything else of the uninstall:
when the user closes the dialog, nothing has been removed.

In the helper, `RemoveAppItems` trusts none of it:

1. The identifier must have the shape of one, and each item must be an
   absolute path with no `.` or `..` and no control character; at most 64
   items. Anything else rejects the whole request.
2. The helper lists the apps in `/Applications` itself and looks for the one
   with that identifier. None, or more than one, stops the request.
3. An app that is running stops the request.
4. The helper plans the uninstall again, with the catalog compiled into it
   and the same matching rule as the app (`LeftoverMatcher`).
5. Every requested item must be in that plan, as the bundle or as something
   found in a folder the catalog marks for an administrator. The library of
   the account the helper runs as is never part of it. One item that is not
   in the plan stops the request before anything is touched. The paths the
   caller sent can only narrow what the helper found by itself.
6. The items are removed through `DeleteItems`, for good, with the
   fingerprints the helper just read and the same `ProtectedPaths` policy.

Verified by hand, on one Mac (macOS 27.2 on Apple Silicon): two apps in
`/Applications` were uninstalled behind the macOS password dialog, with the
helper running as root; what they had left in the user's Library went to the
Trash. Not verified: the same with the fingerprint, and the removal of items
in `/Library`, where nothing of those two apps was removed. The tests
exercise this path on temporary folders (`tests/uninstall_flow.rs`,
`privilege/helper_protocol.rs`) and with `--dry-run` against a real app, as
an ordinary user.

### Changing network settings

Three settings of macOS can make a connection faster and need an
administrator to change: the search for a proxy before each new connection,
the cache of names already looked up, and the name servers of a network
service. The app asks the helper with a third form of request, `--network`,
followed by one group of words per change, the groups separated by `--then`:

| Group | Change | What the helper runs |
|---|---|---|
| `flush-dns` | Empty the name cache | `/usr/bin/dscacheutil -flushcache`, then `/usr/bin/killall -HUP mDNSResponder` |
| `proxy-discovery on\|off <service>` | Turn the proxy search on or off | `/usr/sbin/networksetup -setproxyautodiscovery <service> on\|off` |
| `dns <service> [<server>...]` | Set the name servers; none means the ones the network hands out | `/usr/sbin/networksetup -setdnsservers <service> <server>...\|empty` |

In the app, `TuneNetwork` refuses an empty or malformed request before macOS
is asked to authenticate anyone, and `MacosNetworkTuner` starts the helper
the same way a cleanup does, behind the same checks of the helper file and
of the signature of the bundle.

In the helper, `ChangeNetworkSettings` trusts none of it:

1. A request carries what to change, never a command. Each group must be one
   of the three forms, word for word; anything else rejects the whole
   request, and the error names nothing the caller sent.
2. A service name is printable, at most 64 characters, with no space around
   it, and does not start with a hyphen, so it can never be read as an
   option. A server must parse as an IP address; at most four per service.
   At most eight changes per request.
3. The helper lists the network services itself
   (`networksetup -listallnetworkservices`). A change that names a service
   macOS does not list stops the request before anything is touched.
4. Each change becomes one fixed command, started with an absolute path, an
   argument list, a cleared environment, no standard input and a time limit.
   No shell is involved after the launch.
5. The changes are applied in order. One that macOS refuses stops the ones
   after it; the ones before it stay applied, and the app reports the
   failure.

What this cannot do: delete a file, name a program to run, or set anything
other than those three settings. Every one is an ordinary setting, visible
in System Settings, and each change the app offers comes with the change
that takes it back (flushing the cache has nothing to take back). The app
remembers what it applied only while it is open.

The app decides what to offer (`NetworkReport::optimizations`), and only two
changes start selected: flushing the cache, and turning the proxy search off
when the network was seen not to answer it. Changing the name servers is
never selected by default: the server in use may filter content or know
names that exist only inside the network.

Not verified: a real change, by the password or by the fingerprint. The
tests exercise the parsing, the checks and the order of the changes with
stand-ins (`privilege/network_request.rs`,
`use_cases/change_network_settings.rs`), and `--dry-run` was run against
this Mac as an ordinary user: it accepted a request for the three changes on
a real service and refused an unknown service, a server that is not an
address, an option in place of a service and an empty request.

Things that follow from this design:

- No root service is installed or left running. Every administrator cleanup
  authenticates again.
- The helper works from its own analysis, not from the preview the user saw.
  It removes what the rule's folders hold at that moment.
- Paths protected by System Integrity Protection cannot be removed even as
  root; they come back as failed items.
- `--dry-run` runs every check and removes nothing. It is how the helper can
  be exercised without changing the system.

Verified by hand, on one Mac (macOS 27.2 on Apple Silicon): with Touch ID turned
on for `sudo`, a cleanup of `/Library/Logs` asked for the fingerprint, removed
the items as root and recorded them in the history. Not verified: a real
cleanup of `/Library/Caches`, the password dialog for this cleanup, and the
password dialog that follows a fingerprint that did not go through. The tests
cover the helper's logic on temporary folders
(`tests/system_cleanup_flow.rs`, `privilege/helper_protocol.rs`), its
simulation mode, how the `sudo` configuration is read and how a run is
interpreted; they cannot authenticate.

Known, accepted risk: an app is believed when it declares its bundle
identifier. An app that declares the identifier of another app would have
that app's items listed as its own, as sure matches. The user sees every item
before confirming, what is theirs goes to the Trash, and the apps macOS owns
cannot be removed at all; but nothing checks that the identifier is true.

Known, accepted risk: with one-shot elevation, malware already running as the
user could try to replace the helper between the check and its execution. The
signature check confirms that the bundle is intact, not who signed it: with
the default ad hoc signature, a program able to rewrite the bundle could sign
it again. This is acceptable for personal use; distributing the app to others
requires an Apple-issued signing identity and moving to a daemon registered
with `SMAppService`.
