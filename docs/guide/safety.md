# Safety

English · [Português](safety.pt-BR.md)

A cleaner deletes files, so a mistake can cost you data. This page explains, in plain terms, the rules DiskMarshal follows so that does not happen, and says honestly what has not been tested yet.

> **Where things stand today.** The app really deletes what you select. Every rule on this page is enforced in the code and covered by automated tests. Those tests run on temporary folders made for the purpose; removing real personal data from the interface was only tried on test folders. Start small: clean one thing, look at the result, then go on. The section [What has not been tested](#what-has-not-been-tested) lists the rest.

## The five rules

### 1. You always see a preview first

Before anything is removed, DiskMarshal lists exactly what will go and how much space it frees. Searching and analyzing never remove anything. Then a dialog called "Autorização requerida" (authorization required) repeats how many items and how much space, and waits. Nothing happens until you click the button that confirms. "Cancelar" (cancel) closes it with nothing removed.

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
| An app you installed yourself, and what it left in your Library | "Aplicativos" | The Trash | Yes, the same way. |
| An app the App Store or an installer put there, and what it left in `/Library` | "Aplicativos", group "Sistema" | Removed at once | No. The app can be installed again. What it kept in `/Library` cannot be brought back. |
| Docker's build cache, images, stopped containers and volumes | "Docker" | Removed at once, by Docker itself | The build cache and images come back with a new build or download. **The content of a container or a volume does not.** |
| What is already in the Trash | "Limpeza", rule "Lixeira" | Removed at once | **No.** This rule empties the Trash. |

Only data that can be recreated is removed for good, with three exceptions, and each one needs a choice of yours:

- **The rule "Lixeira" (Trash)** empties the Trash, and that cannot be undone. It carries the badge "Revisar" and is never selected for you.
- **A Docker container or volume** holds content that exists nowhere else. Both carry the badge "Revisar" and are never selected for you.
- **The administrator part of an uninstall.** You chose to uninstall that app, and what it kept in `/Library` goes with it for good.

Space sent to the Trash is only freed when you empty the Trash.

### 3. Some places can never be deleted

DiskMarshal refuses to delete these, even if a bug or a bad rule asked it to:

- **Anything inside the system's own folders:** `/System`, `/usr`, `/bin` and `/sbin`.
- **The shared folders themselves:** `/Applications`, `/Library`, `/Library/Caches`, `/Library/Logs`, `/Users`, `/Volumes`, `/private`, `/tmp` and a few others. The content of `/Library/Caches` and `/Library/Logs` can be cleaned by the "Sistema" group; the folders stay.
- **Your personal folders themselves:** the home folder, Desktop, Documents, Downloads, Pictures, Movies, Music, Public, Sites, your own Applications folder and the Trash folder.
- **The Library folders themselves:** `~/Library`, `~/Library/Caches`, `~/Library/Logs`, `~/Library/Application Support`, `~/Library/Containers`, `~/Library/Group Containers`, `~/Library/Developer`, `~/Library/Preferences` and `~/Library/Mobile Documents` (where iCloud Drive lives). Their content can be cleaned; the folders stay.
- **Anything inside a `.git` folder,** which holds the history of a code project, and the `.git` folder itself.
- **Anything inside a package:** an app (`.app`), a Photos library (`.photoslibrary`), a Music or TV library, an iMovie or Final Cut library, a Logic project, a disk image bundle (`.sparsebundle`) and similar. Deleting pieces of these corrupts them, so DiskMarshal treats each as one sealed item.
- **Anything outside the folders that were searched,** and the searched folder itself.

Letter case does not get around this: `/system` is treated as `/System`. A path written with `..` in it is refused outright.

### 4. Every item is checked again right before removal

Time passes between the preview and your click. So, immediately before removing an item, DiskMarshal looks at it again. If it changed, moved or was replaced since you saw it in the preview, it is skipped and reported, not removed. The result dialog counts these as "itens deixados como estavam" (items left as they were).

It also never follows shortcuts (symbolic links): a link inside a cache folder is removed as a link, and whatever it points to is left alone. And it never reaches into another disk: a folder that contains another disk mounted inside it is refused whole.

### 5. Everything removed is written down

Each removal is recorded in a history file that only your user account can read:

```text
~/Library/Application Support/DiskMarshal/history.jsonl
```

Each line records when it happened, what was removed, how much space it took, and whether it was deleted or sent to the Trash. The "Painel" (dashboard) shows the latest entries under "Últimas remoções" (latest removals). What the system cleanup removes is recorded there too.

If a removal happened but could not be written to the history, the result dialog says so.

## One deletion path for everything

Every screen that removes something uses the same piece of code to do it, and that code applies all five rules above. There is no second, quicker way to delete hidden somewhere. The helper that cleans system areas and removes the system items of an uninstalled app uses that same code.

One thing cannot go through it: what Docker holds. Those are not files the app can reach, so Docker itself removes them, one at a time, and refuses anything that is in use. DiskMarshal never forces Docker to remove something.

## What each screen keeps for you

- **"Duplicados":** at least one copy of each file always stays. The oldest copy is kept by default.
- **"Fotos":** at least one photo of each group always stays. The largest file is kept by default. The library of the Photos app is never read or changed.
- **"Desenvolvedor":** a folder is offered only when the file that identifies the project is beside it and git has no tracked files inside it. No source code is removed.
- **"Limpeza":** a rule removes the content of its folders, never the folders. Rules marked "Revisar" (review) and the whole "Sistema" group are never selected for you.
- **"Aplicativos":** an app that is open is refused. Only items named after the app are found, and only exact matches come selected. When another copy of the app is installed, its data stays.
- **"Docker":** what a container uses is never listed, and Docker refuses what came into use in the meantime. Named images, containers and volumes are never selected for you.

## iCloud files are not downloaded

Files stored only in iCloud take no space on your Mac. Normally, any program that reads such a file makes macOS download it, which would fill the disk you are trying to free.

DiskMarshal tells macOS, before reading anything, that it must not download files on its behalf. Cloud-only items are measured and labeled, never fetched. The searches for duplicates and similar photos leave them out, because comparing them would mean reading them. If macOS refuses that setting, DiskMarshal stops instead of scanning.

This has been tested with files stored on the Mac. It has not been tested on a file that really is cloud-only.

## The network: only when you ask

DiskMarshal sends no usage data, checks for no updates and has no account. What it learns about your disk and your network stays on your Mac.

Reading the traffic uses nothing but counters macOS already keeps: no packet is captured, so the content of what travels is never seen. The app uses the network itself in two cases, each started by a button on the network screen:

- "Analisar conexão" (analyze connection) sends ten pings to your router and ten to 1.1.1.1, and asks your name servers and three public ones (Cloudflare, Google and Quad9) for the address of `example.com`. When macOS is set to look for a proxy, it also asks once whether your network answers that search. A few kilobytes in all.
- "Medir velocidade" (measure speed) runs the speed test that ships with macOS against Apple's servers. It moves real data: more than 1 GB on a fast connection.

Neither one carries anything about you or your files.

## Network settings

The network screen can change three settings of macOS, only after you confirm and after macOS authenticates you: turn off the automatic search for a proxy, flush the DNS cache, and set the name servers of your network service. Each is an ordinary setting you can see in System Settings › Network. "Desfazer alterações" (undo changes) puts it back while the app is open. No file is deleted.

Not tested yet: a real change made from the app. The helper that makes the changes was exercised only in its simulation mode, which checks the request against the Mac and changes nothing.

## Administrator areas

Two places need administrator rights: `/Library/Caches` ("Caches do sistema") and `/Library/Logs` ("Logs do sistema"). They are in the "Sistema" (system) group of the cleanup screen, and are never selected for you.

How it works when you select them and confirm:

1. **DiskMarshal asks macOS to authenticate you.** macOS shows its own dialog, asking for the administrator password, or for your fingerprint when you turned on Touch ID for administrator commands (the [usage guide](usage.md#using-the-fingerprint-instead-of-the-password) shows how). Either one goes to macOS. DiskMarshal never sees it and never stores it.
2. **macOS runs a small helper once.** The helper is a separate little program that sits inside the app. It cleans the areas you selected, reports what it did, and exits.
3. **DiskMarshal writes what was removed to your history,** like any other removal.

What keeps this narrow:

- **The app itself never runs as an administrator.** Only the helper does, and only for that one operation.
- **Nothing stays in the background.** No service is installed. Every system cleanup asks for authentication again.
- **The helper accepts only the names of known cleanup rules,** such as `system-caches`. It refuses anything shaped like a path or an option. Which folders a rule covers is written inside the helper itself, so nobody can point it at another folder.
- **The helper only accepts rules meant for an administrator,** looks at the folders again by itself, and removes through the same deletion rules as the app (rules 3 and 4 above).
- **The app checks the helper before using it:** it must be a regular file (not a shortcut) that only its owner can change, and, in the installed app, macOS must confirm that the app is still exactly what was signed.

What you should know:

- **This path was confirmed by hand on one Mac only.** With Touch ID turned on for `sudo`, a real cleanup of `/Library/Logs` asked for the fingerprint, removed the items and recorded them in the history; that was on macOS 27.2 on Apple Silicon. A real cleanup of `/Library/Caches` and the password dialog for this cleanup were not exercised. The automated tests cover the helper on temporary folders and its simulation mode, which checks everything and removes nothing; they cannot authenticate.
- **The helper works from its own fresh look at the folders,** not from the list you saw on screen. It removes what those two folders hold at that moment, which can differ a little from the preview.
- **The fingerprint is something you turn on.** By default macOS asks for the password. It takes the fingerprint after you turn on Touch ID for `sudo`, a setting of macOS that DiskMarshal never changes for you. When the fingerprint does not go through, the password dialog appears.
- **It needs the installed app** (`cargo xtask install`), because that is where the helper lives.
- **A known, accepted risk:** a malicious program already running in your user account could try to swap the helper in the instant between the check and its execution. This is considered acceptable for personal use. The details are in the [security policy](../security.md#privileged-helper).

### Uninstalling an app that needs an administrator

An app the App Store or an installer put on the Mac belongs to the system, and so does what it left in `/Library`. Removing those works the same way: macOS authenticates you, then runs the helper once.

- **The helper is told which app, by its identifier, and which items.** It then finds the app and what belongs to it by itself, and removes an item only when its own search found it too. One item that is not the app's stops everything before anything is touched.
- **It refuses an app that is open,** and an identifier that two installed apps share.
- **These items are removed for good.** The helper runs as an administrator and cannot reach your Trash.
- **If you close the authentication dialog, nothing at all is removed,** not even the part that needs no administrator. That part only starts after the administrator part worked.

## What has not been tested

Said plainly, so you can decide how careful to be:

- A real cleanup of `/Library/Caches`, and the password dialog for the system cleanup (see above for what was confirmed by hand).
- A file that is really only in iCloud.
- Starting the app with "Reduce motion" turned on in System Settings.
- Removing real personal data from the interface. It was exercised on test folders only.
- macOS versions other than 27, and Intel Macs.
- Uninstalling an app with the fingerprint, and removing what an app left in `/Library`. Two real apps were uninstalled by hand behind the password dialog, and nothing of theirs was removed from `/Library`; that part ran on test folders and in the helper's simulation mode.
- Removing anything from a real Docker. Reading what Docker holds was checked on a real one; the removal ran against a stand-in program that answers like Docker.
- Whether Docker Desktop gives the freed space back to macOS.

## Known limits

- **Paths protected by macOS cannot be cleaned.** System Integrity Protection guards some files even from an administrator. They are reported as left as they were.
- **Some system areas are not cleaned at all:** Time Machine local snapshots, `/private/var/log` and `/private/var/tmp`.
- **Photos inside the library of the Photos app are out of scope.** Use the Photos app for those.
- **The space really freed can be smaller than the estimate.** The disk format of the Mac (APFS) can store two copies of a file as one, and can keep old versions in snapshots. Removing a file whose content is still used elsewhere frees less than its size.
- **Items in the Trash still take space** until you empty it.
- **The uninstaller finds what is named after the app.** Data kept under another name stays. And an app that declares the identifier of another app would have that app's items listed as its own, so read the list before confirming.
- **Docker is asked through its own program only.** Other container engines are not looked for.

## If something goes wrong

1. **Look in the Trash.** Personal files removed by DiskMarshal are there. Right-click and choose **Put Back**.
2. **Read the history file** to see exactly what was removed and when. The dashboard shows the latest entries.
3. **Open an issue** at [github.com/renatoasterio/DiskMarshal/issues](https://github.com/renatoasterio/DiskMarshal/issues) describing what happened. If it looks like a security problem, follow [SECURITY.md](../../SECURITY.md) instead.

## For the technically curious

The rules above are enforced in code and covered by tests:

- Path rules: [`protected_paths.rs`](../../crates/diskmarshal-domain/src/deletion/protected_paths.rs)
- The single code path that deletes: [`delete_items.rs`](../../crates/diskmarshal-application/src/use_cases/delete_items.rs)
- The helper that cleans system areas: [`apps/privileged-helper/src/main.rs`](../../apps/privileged-helper/src/main.rs)
- How an item is matched to an app: [`leftover_matcher.rs`](../../crates/diskmarshal-domain/src/applications/leftover_matcher.rs)
- What Docker is asked to do: [`docker_cli_container_engine.rs`](../../crates/diskmarshal-infrastructure/src/process/docker_cli_container_engine.rs)
- How the app asks macOS to run it: [`macos_privileged_cleaner.rs`](../../crates/diskmarshal-infrastructure/src/privilege/macos_privileged_cleaner.rs)
- The full policy for contributors: [docs/security.md](../security.md)
