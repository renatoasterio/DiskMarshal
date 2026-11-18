# Safety

English · [Português](safety.pt-BR.md)

A cleaner deletes files, so a mistake can cost you data. This page explains, in plain terms, the rules MacCleaner follows so that does not happen, and says honestly what has not been tested yet.

> **Where things stand today.** The app really deletes what you select. Every rule on this page is enforced in the code and covered by automated tests. Those tests run on temporary folders made for the purpose; removing real personal data from the interface was only tried on test folders. Start small: clean one thing, look at the result, then go on. The section [What has not been tested](#what-has-not-been-tested) lists the rest.

## The five rules

### 1. You always see a preview first

Before anything is removed, MacCleaner lists exactly what will go and how much space it frees. Searching and analyzing never remove anything. Then a dialog called "Autorização requerida" (authorization required) repeats how many items and how much space, and waits. Nothing happens until you click the button that confirms. "Cancelar" (cancel) closes it with nothing removed.

![The cleanup screen listing each rule, where it looks and how much it found, before anything is removed](../assets/screens/cleanup.png)

*Screenshot of the app with demonstration data.*

### 2. Personal files go to the Trash

| What is being removed | Screen | Where it goes | Can you get it back? |
|---|---|---|---|
| Duplicate files | "Duplicados" | The Trash | Yes. Open the Trash, right-click the item and choose **Put Back**. |
| Similar photos | "Fotos" | The Trash | Yes, the same way. |
| "Xcode Archives" | "Limpeza" | The Trash | Yes, the same way. |
| Caches, temporary files, logs, Xcode build output, package manager caches | "Limpeza" | Removed at once | No, but apps and tools recreate them when needed. |
| System caches and logs (`/Library/Caches`, `/Library/Logs`) | "Limpeza", group "Sistema" | Removed at once | No, but the system and apps recreate them when needed. |
| Build and dependency folders (`node_modules`, `target`, `bin`/`obj`) | "Desenvolvedor" | Removed at once | No, but your tools recreate them the next time you install or build. |
| What is already in the Trash | "Limpeza", rule "Lixeira" | Removed at once | **No.** This rule empties the Trash. |

With one exception, only data that can be recreated is ever removed for good. The exception is the rule "Lixeira" (Trash): it empties the Trash, and that cannot be undone. It carries the badge "Revisar" and is never selected for you: it runs only if you select it yourself.

Space sent to the Trash is only freed when you empty the Trash.

### 3. Some places can never be deleted

MacCleaner refuses to delete these, even if a bug or a bad rule asked it to:

- **Anything inside the system's own folders:** `/System`, `/usr`, `/bin` and `/sbin`.
- **The shared folders themselves:** `/Applications`, `/Library`, `/Library/Caches`, `/Library/Logs`, `/Users`, `/Volumes`, `/private`, `/tmp` and a few others. The content of `/Library/Caches` and `/Library/Logs` can be cleaned by the "Sistema" group; the folders stay.
- **Your personal folders themselves:** the home folder, Desktop, Documents, Downloads, Pictures, Movies, Music, Public, Sites, your own Applications folder and the Trash folder.
- **The Library folders themselves:** `~/Library`, `~/Library/Caches`, `~/Library/Logs`, `~/Library/Application Support`, `~/Library/Containers`, `~/Library/Group Containers`, `~/Library/Developer`, `~/Library/Preferences` and `~/Library/Mobile Documents` (where iCloud Drive lives). Their content can be cleaned; the folders stay.
- **Anything inside a `.git` folder,** which holds the history of a code project, and the `.git` folder itself.
- **Anything inside a package:** an app (`.app`), a Photos library (`.photoslibrary`), a Music or TV library, an iMovie or Final Cut library, a Logic project, a disk image bundle (`.sparsebundle`) and similar. Deleting pieces of these corrupts them, so MacCleaner treats each as one sealed item.
- **Anything outside the folders that were searched,** and the searched folder itself.

Letter case does not get around this: `/system` is treated as `/System`. A path written with `..` in it is refused outright.

### 4. Every item is checked again right before removal

Time passes between the preview and your click. So, immediately before removing an item, MacCleaner looks at it again. If it changed, moved or was replaced since you saw it in the preview, it is skipped and reported, not removed. The result dialog counts these as "itens deixados como estavam" (items left as they were).

It also never follows shortcuts (symbolic links): a link inside a cache folder is removed as a link, and whatever it points to is left alone. And it never reaches into another disk: a folder that contains another disk mounted inside it is refused whole.

### 5. Everything removed is written down

Each removal is recorded in a history file that only your user account can read:

```text
~/Library/Application Support/MacCleaner/history.jsonl
```

Each line records when it happened, what was removed, how much space it took, and whether it was deleted or sent to the Trash. The "Painel" (dashboard) shows the latest entries under "Últimas remoções" (latest removals). What the system cleanup removes is recorded there too.

If a removal happened but could not be written to the history, the result dialog says so.

## One deletion path for everything

Every screen that removes something uses the same piece of code to do it, and that code applies all five rules above. There is no second, quicker way to delete hidden somewhere. The helper that cleans system areas uses that same code.

## What each screen keeps for you

- **"Duplicados":** at least one copy of each file always stays. The oldest copy is kept by default.
- **"Fotos":** at least one photo of each group always stays. The largest file is kept by default. The library of the Photos app is never read or changed.
- **"Desenvolvedor":** a folder is offered only when the file that identifies the project is beside it and git has no tracked files inside it. No source code is removed.
- **"Limpeza":** a rule removes the content of its folders, never the folders. Rules marked "Revisar" (review) and the whole "Sistema" group are never selected for you.

## iCloud files are not downloaded

Files stored only in iCloud take no space on your Mac. Normally, any program that reads such a file makes macOS download it, which would fill the disk you are trying to free.

MacCleaner tells macOS, before reading anything, that it must not download files on its behalf. Cloud-only items are measured and labeled, never fetched. The searches for duplicates and similar photos leave them out, because comparing them would mean reading them. If macOS refuses that setting, MacCleaner stops instead of scanning.

This has been tested with files stored on the Mac. It has not been tested on a file that really is cloud-only.

## No network, no telemetry

MacCleaner does not connect to the internet. It sends no usage data, checks for no updates and has no account. What it learns about your disk stays on your Mac.

## Administrator areas

Two places need administrator rights: `/Library/Caches` ("Caches do sistema") and `/Library/Logs` ("Logs do sistema"). They are in the "Sistema" (system) group of the cleanup screen, and are never selected for you.

How it works when you select them and confirm:

1. **MacCleaner asks macOS to authenticate you.** macOS shows its own dialog, asking for Touch ID or the administrator password. The password goes to macOS. MacCleaner never sees it and never stores it.
2. **macOS runs a small helper once.** The helper is a separate little program that sits inside the app. It cleans the areas you selected, reports what it did, and exits.
3. **MacCleaner writes what was removed to your history,** like any other removal.

What keeps this narrow:

- **The app itself never runs as an administrator.** Only the helper does, and only for that one operation.
- **Nothing stays in the background.** No service is installed. Every system cleanup asks for authentication again.
- **The helper accepts only the names of known cleanup rules,** such as `system-caches`. It refuses anything shaped like a path or an option. Which folders a rule covers is written inside the helper itself, so nobody can point it at another folder.
- **The helper only accepts rules meant for an administrator,** looks at the folders again by itself, and removes through the same deletion rules as the app (rules 3 and 4 above).
- **The app checks the helper before using it:** it must be a regular file (not a shortcut) that only its owner can change, and, in the installed app, macOS must confirm that the app is still exactly what was signed.

What you should know:

- **This path has not been verified end to end.** The helper is covered by automated tests on temporary folders and by its simulation mode, which checks everything and removes nothing. The macOS authentication dialog and a real cleanup of `/Library/Caches` and `/Library/Logs` were not exercised by the project's tests.
- **The helper works from its own fresh look at the folders,** not from the list you saw on screen. It removes what those two folders hold at that moment, which can differ a little from the preview.
- **Touch ID or password is macOS's decision,** not MacCleaner's.
- **It needs the installed app** (`cargo xtask install`), because that is where the helper lives.
- **A known, accepted risk:** a malicious program already running in your user account could try to swap the helper in the instant between the check and its execution. This is considered acceptable for personal use. The details are in the [security policy](../security.md#privileged-helper).

## What has not been tested

Said plainly, so you can decide how careful to be:

- The macOS authentication dialog and a real system cleanup, from start to finish (see above).
- A file that is really only in iCloud.
- Starting the app with "Reduce motion" turned on in System Settings.
- Removing real personal data from the interface. It was exercised on test folders only.
- macOS versions other than 27, and Intel Macs.

## Known limits

- **Paths protected by macOS cannot be cleaned.** System Integrity Protection guards some files even from an administrator. They are reported as left as they were.
- **Some system areas are not cleaned at all:** Time Machine local snapshots, `/private/var/log` and `/private/var/tmp`.
- **Photos inside the library of the Photos app are out of scope.** Use the Photos app for those.
- **The space really freed can be smaller than the estimate.** The disk format of the Mac (APFS) can store two copies of a file as one, and can keep old versions in snapshots. Removing a file whose content is still used elsewhere frees less than its size.
- **Items in the Trash still take space** until you empty it.

## If something goes wrong

1. **Look in the Trash.** Personal files removed by MacCleaner are there. Right-click and choose **Put Back**.
2. **Read the history file** to see exactly what was removed and when. The dashboard shows the latest entries.
3. **Open an issue** at [github.com/renatoasterio/MacCleaner/issues](https://github.com/renatoasterio/MacCleaner/issues) describing what happened. If it looks like a security problem, follow [SECURITY.md](../../SECURITY.md) instead.

## For the technically curious

The rules above are enforced in code and covered by tests:

- Path rules: [`protected_paths.rs`](../../crates/maccleaner-domain/src/deletion/protected_paths.rs)
- The single code path that deletes: [`delete_items.rs`](../../crates/maccleaner-application/src/use_cases/delete_items.rs)
- The helper that cleans system areas: [`apps/privileged-helper/src/main.rs`](../../apps/privileged-helper/src/main.rs)
- How the app asks macOS to run it: [`osascript_privileged_cleaner.rs`](../../crates/maccleaner-infrastructure/src/privilege/osascript_privileged_cleaner.rs)
- The full policy for contributors: [docs/security.md](../security.md)
