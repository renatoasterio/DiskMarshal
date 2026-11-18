# Questions and troubleshooting

English · [Português](faq.pt-BR.md)

## Questions

### Does DiskMarshal delete anything?

Yes, but only what you select, and only after it shows a preview and you confirm. Opening the app, searching and analyzing never remove anything. Duplicate files, similar photos and Xcode archives go to the Trash; caches, temporary files, logs and build folders are removed for good. The command line tool, `diskmarshal-cli`, only reads.

### Is it safe to run?

It follows the rules in the [safety guide](safety.md): a preview first, personal files to the Trash, a list of places that can never be deleted, a second check of every item right before removal, and a written history. Those rules are covered by automated tests.

Be aware that the project is young and has no stable release. Some things have not been verified on the real thing yet; the next question lists them. Start with a small cleanup and look at the result.

### What has not been tested yet?

- **System cleanup, end to end.** The macOS authentication dialog and a real cleanup of `/Library/Caches` and `/Library/Logs` were not exercised by the project's tests. The helper that does it is covered by tests on temporary folders and by its simulation mode.
- **A file that is really cloud-only in iCloud.** The protection against downloads is in place, but was not tried on a real cloud-only file.
- **Starting with reduced motion,** with the "Reduce motion" switch of System Settings turned on.
- **Deleting real personal data from the interface.** It was exercised on test folders only.
- **Other macOS versions and Intel Macs.**

### Does it send my data anywhere?

No. DiskMarshal never connects to the internet. There is no telemetry, no account and no update check.

### Will scanning download my iCloud files?

No. DiskMarshal tells macOS not to download cloud-only files while it scans. They are measured and labeled, not fetched. The searches for duplicates and for similar photos skip them entirely.

### Why is there no app to download?

A downloadable app needs to be signed with an Apple developer certificate and checked by Apple. The project does not offer that today; it is an idea, not a promise. For now you build it from the source, once, with `cargo xtask install`. That gives you a regular app in the Applications folder, which opens from Launchpad or Spotlight. The [getting started guide](getting-started.md) walks through it.

### Do I need the Terminal every time?

No. You need it once, to build and install. After that DiskMarshal opens like any other app. You need the Terminal again only to update to a newer version.

### Does DiskMarshal need Full Disk Access?

No, it works without it. But macOS then hides some folders from the app, such as the data of Mail and Safari and the Trash, so they do not appear in the results. The screens that read your folders show a one-line notice with the link "Abrir ajustes de Acesso Total ao Disco" (open the Full Disk Access settings). The steps are in the [getting started guide](getting-started.md#step-7-give-full-disk-access-optional).

### Why did macOS forget the Full Disk Access I gave?

Because the app was rebuilt. The app you build carries a local signature (ad hoc) that changes with every build, so after each `cargo xtask install` macOS sees a new app. Give the permission again, or sign with an identity of your own by setting the environment variable `DISKMARSHAL_SIGNING_IDENTITY` before running `cargo xtask install`.

### How does the cleanup of system areas ask for my password?

It does not ask; macOS does. When you confirm a cleanup that includes the "Sistema" (system) group, macOS shows its own authentication dialog and collects your administrator password, or your fingerprint when you turned that on. DiskMarshal never sees either. macOS then runs a small helper that sits inside the app, once, and the helper exits when it is done. The [safety guide](safety.md#administrator-areas) explains the details.

### Will the dialog offer Touch ID?

Only after you turn it on. The password dialog of macOS does not take the fingerprint. Once you turn on Touch ID for `sudo`, with one line in the Terminal shown in the [usage guide](usage.md#using-the-fingerprint-instead-of-the-password), DiskMarshal asks for the fingerprint first and shows the password dialog when it does not go through. The fingerprint path was not verified by the project's automated tests: it needs a finger on the sensor.

### Does DiskMarshal run as an administrator or leave something running in the background?

No to both. The app always runs as your own user. Administrator rights exist only inside the small helper, for one cleanup, after macOS authenticates you. No service is installed and nothing keeps running afterwards.

### Why did the disk free less space than the app estimated?

A few possible reasons:

- What went to the Trash still takes space until you empty the Trash.
- The disk format of the Mac (APFS) can store two copies of a file as one, and can keep old versions in snapshots. Removing a file whose content is still used elsewhere frees less than its size.
- Some items could not be removed (macOS protected them, or they changed after the preview). The result dialog counts them as "itens deixados como estavam" (items left as they were).

### Why are the photos in my Photos app not found?

The "Fotos" screen compares loose image files in folders only. The library of the Photos app is one sealed package, and DiskMarshal never reads or changes it. To remove duplicates inside that library, use the Photos app itself.

### Why are there no fan speeds in the monitor?

The monitor does not read fan speeds; this is a known limit. It shows processor, memory, swap, disk, temperatures and the busiest programs.

### How much disk and processor does the app use?

Measured on an Apple M2 Pro: the installed app takes about 10 MB. Idle, it uses roughly 3% of one processor core while its window is not in front, and roughly 10 to 15% while it is in front with full animations. Setting "Animações" (animations) to "Reduzida" (reduced) in the appearance panel lowers that.

### Which Macs and macOS versions work?

It is developed and tested on macOS 27 on Apple Silicon. Other versions and Intel Macs should work but are untested. If you try one, an issue telling us how it went is very welcome.

### Does it work on Windows or Linux?

No. DiskMarshal is built on macOS features such as iCloud placeholders, the Trash and the system sensors.

### Why is the interface in Portuguese?

The project started in Brazil. Every text of the interface lives in one file, so adding English is a contained task. Contributions are welcome. Until then, the [usage guide](usage.md) gives the meaning of each label.

### Is it really safe to delete `node_modules`, `target` and similar folders?

Yes, as long as the project still has the file that describes it (`package.json`, `Cargo.toml`, and so on). Those folders are recreated by your tools the next time you install or build. DiskMarshal only offers a folder when it finds that file next to it and git has no tracked files inside the folder. The removal is permanent, so the folder does not go to the Trash.

### What is the difference between "Ocupado em disco" and "Tamanho lógico"?

"Tamanho lógico" (logical size) is the size of the content. "Ocupado em disco" (used on disk) is the space it really takes, which is what you get back by deleting it. They differ because disks store files in blocks and can compress them.

### How is this different from other cleaners?

The code is open, so anyone can check what it does. It never deletes without a preview, never touches the inside of your Photos library, and never sends anything over the network.

### Why does the uninstaller leave some files of an app behind?

It removes what carries the identifier of the app or its name, in the Library folders where apps keep their things. A document the app saved in your folders is yours and stays. So does a folder named after the maker of the app, which other apps of that maker may share. Items marked "Revisar" are found too, but wait for you to select them.

### Why can I not select an app, or some of its items?

There are two reasons. The app is open: it is marked "aberto" in the list, so quit it and click it again. Or the item needs an administrator and you are running from the source code: the "Sistema" group then shows "só no app instalado · cargo xtask install".

### Why did cleaning Docker not free space on my Mac right away?

The space is freed inside Docker. Docker Desktop keeps everything in one large file on the Mac, and that file may not shrink at once. The result dialog shows what Docker freed inside itself. Whether and when Docker Desktop gives that space back to macOS was not checked by this project.

### How do I uninstall it?

Move `/Applications/DiskMarshal.app` to the Trash. The removal history is in `~/Library/Application Support/DiskMarshal/`; delete that folder if you do not want to keep it. If you no longer need the source, move the `DiskMarshal` project folder to the Trash too.

## Troubleshooting

Find the message you see in the left column.

### While installing

| What you see | Why | Fix |
|---|---|---|
| `command not found: cargo` | The Terminal does not know where Rust is. | Run the two commands from [step 3](getting-started.md#step-3-install-rust) that start with `echo` and `source`, then try again. |
| `command not found: brew` | Homebrew is not installed, or its "Next steps" were skipped. | Follow [step 2](getting-started.md#step-2-install-homebrew), including the commands Homebrew prints at the end. |
| `xcrun: error: invalid active developer path` | The Xcode command line tools are missing. This often happens after a macOS update. | Run `xcode-select --install`. |
| `error: could not find Cargo.toml` | The command ran outside the project folder. | Run `cd ~/DiskMarshal` first. |
| `error: no such command: xtask` | Same cause: the command ran outside the project folder. | Run `cd ~/DiskMarshal` first. |
| `error: no such command: deny` during `cargo xtask check` | The dependency auditor is not installed. | Run `brew install cargo-deny`. |
| The build stops with `No space left on device` | The disk is full. The build needs about 1 GB. | Free some space, then run the command again. |
| `xtask: could not replace the installed app` or `xtask: could not copy the app to the destination folder` | Most likely your account cannot write to `/Applications`. | Install in your own Applications folder instead: `cargo xtask install ~/Applications`. |
| `xtask: could not sign the app` | Most likely the name in `DISKMARSHAL_SIGNING_IDENTITY` does not match a signing identity on this Mac. | Correct the name, or remove the variable to use the local signature. |
| The project folder takes a lot of space | Build output is kept in the `target` folder. | Run `cargo clean` inside the project folder. The installed app keeps working, and the next build recreates the folder. |

### In the app

| What you see | Why | Fix |
|---|---|---|
| "Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises." | The app does not have Full Disk Access, so macOS hides some folders from it. | Optional. Click "Abrir ajustes de Acesso Total ao Disco" and follow [step 7](getting-started.md#step-7-give-full-disk-access-optional). |
| That notice came back after an update | macOS forgets the permission when the app is rebuilt with the local signature. | Give the permission again. See [the question above](#why-did-macos-forget-the-full-disk-access-i-gave). |
| The row "Lixeira" says "vazio" although the Trash has items | macOS only shows the Trash to apps with Full Disk Access. | Give Full Disk Access, or empty the Trash in Finder. |
| The group "Sistema" says "só no app instalado · cargo xtask install" and cannot be selected | The app is running from the source, without the helper that cleans system areas. | Install the app with `cargo xtask install` and open the installed one. |
| "As áreas de sistema não foram limpas: a autenticação foi cancelada." | The macOS authentication dialog was closed. | Nothing was removed from the system areas. Run the cleanup again and authenticate if you want them cleaned. |
| "As áreas de sistema não foram limpas: esse recurso só existe no app instalado." | The helper is missing, was changed, or the app's signature is no longer valid. | Run `cargo xtask install` again and open the installed app. |
| "As áreas de sistema não foram limpas:" followed by another reason | The cleanup started and could not finish. | Nothing is forced. Please [open an issue](https://github.com/renatoasterio/DiskMarshal/issues) with the message. |
| "itens deixados como estavam: mudaram, sumiram ou o macOS não permitiu" | Some items changed after the preview, were already gone, or are protected by macOS. They were skipped on purpose. | Nothing to fix. The screen looks again after you close the dialog. |
| "Selecione ao menos um item." | The action button was clicked with nothing selected. | Select something first. |
| "Ao menos uma cópia de cada arquivo precisa ficar." or "Ao menos uma foto de cada grupo precisa ficar." | You tried to mark every file of a group for removal. | Keep one. To remove all of them, use Finder. |
| "Análise interrompida" | The cleanup analysis could not finish. | Click "Tentar de novo" (try again). |
| "Aguardando leituras" stays on the monitor | macOS did not report the system readings. | Wait a few seconds. If it stays, please [open an issue](https://github.com/renatoasterio/DiskMarshal/issues) with your macOS version. |
| macOS asks whether DiskMarshal can access Documents, Desktop or Downloads | Normal macOS protection for those folders. | Click **Allow** to include them, or **Don't Allow** to leave them out. |
| The window is slow, or the Mac gets warm | Animations redraw the window continuously while it is in front. | Open the appearance panel and set "Animações" to "Reduzida" or "Desligada". |

### In the command line tool

| What you see | Why | Fix |
|---|---|---|
| "Pastas sem acesso" is greater than zero | macOS did not let the Terminal read some folders. | Give the Terminal [Full Disk Access](usage.md#when-pastas-sem-acesso-is-not-zero). |
| macOS asks whether the Terminal can access Documents, Desktop or Downloads | Normal macOS protection for those folders. | Click **Allow** to include them in the scan, or **Don't Allow** to leave them out. |
| `erro: não foi possível impedir downloads do iCloud` | macOS refused the setting that prevents iCloud downloads, so DiskMarshal stopped on purpose. | Please [open an issue](https://github.com/renatoasterio/DiskMarshal/issues) with your macOS version. |

## Still stuck?

Open an issue at [github.com/renatoasterio/DiskMarshal/issues](https://github.com/renatoasterio/DiskMarshal/issues). Include:

- your macOS version (Apple menu, **About This Mac**);
- what you were doing: the screen and the button, or the command you ran;
- the full message the app or the Terminal showed.

Issues in English or Portuguese are both fine.
