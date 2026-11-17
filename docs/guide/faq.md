# Questions and troubleshooting

English · [Português](faq.pt-BR.md)

## Questions

### Does MacCleaner delete anything today?

No. The app shows its frame while the screens are being built, and `maccleaner-cli` only reads. See the [project status](../../README.md#project-status).

### Is it safe to run?

Yes. Today it only reads file names and sizes. When cleanup arrives, it will follow the rules in the [safety guide](safety.md): a preview first, personal files to the Trash, and a list of places that can never be deleted.

### Does it send my data anywhere?

No. MacCleaner never connects to the internet. There is no telemetry, no account and no update check.

### Will scanning download my iCloud files?

No. MacCleaner tells macOS not to download cloud-only files while it scans. They are measured and labeled, not fetched.

### Why is there no app to download?

The project is young, and a downloadable app needs to be signed with an Apple developer certificate. A signed `.app` is on the [roadmap](../../README.md#roadmap). For now you build it from the source, which the [getting started guide](getting-started.md) walks through.

### Which Macs and macOS versions work?

It is developed and tested on macOS 27 on Apple Silicon. Other versions and Intel Macs should work but are untested. If you try one, an issue telling us how it went is very welcome.

### Does it work on Windows or Linux?

No. MacCleaner is built on macOS features such as iCloud placeholders, the Trash and the system sensors.

### Why is the interface in Portuguese?

The project started in Brazil. Every text of the interface lives in one file, so adding English is a contained task. Contributions are welcome.

### Is it really safe to delete `node_modules`, `target` and similar folders?

Yes, as long as the project still has the file that describes it (`package.json`, `Cargo.toml`, and so on). Those folders are generated: your tools recreate them the next time you install or build. MacCleaner will only offer a folder when it finds that file next to it.

### What is the difference between "Ocupado em disco" and "Tamanho lógico"?

"Tamanho lógico" (logical size) is the size of the content. "Ocupado em disco" (used on disk) is the space it really takes, which is what you get back by deleting it. They differ because disks store files in blocks and can compress them.

### How is this different from other cleaners?

The code is open, so anyone can check what it does. It never deletes without a preview, never touches the inside of your Photos library, and never sends anything over the network.

## Troubleshooting

Find the message you see in the left column.

| What you see | Why | Fix |
|---|---|---|
| `command not found: cargo` | The Terminal does not know where Rust is. | Run the two commands from [step 3](getting-started.md#step-3-install-rust) that start with `echo` and `source`, then try again. |
| `command not found: brew` | Homebrew is not installed, or its "Next steps" were skipped. | Follow [step 2](getting-started.md#step-2-install-homebrew), including the commands Homebrew prints at the end. |
| `xcrun: error: invalid active developer path` | The Xcode command line tools are missing. This often happens after a macOS update. | Run `xcode-select --install`. |
| `error: could not find Cargo.toml` | The command ran outside the project folder. | Run `cd ~/MacCleaner` first. |
| `error: no such command: xtask` | Same cause: the command ran outside the project folder. | Run `cd ~/MacCleaner` first. |
| `error: no such command: deny` during `cargo xtask check` | The dependency auditor is not installed. | Run `brew install cargo-deny`. |
| The build stops with `No space left on device` | The disk is full. The build needs about 1 GB. | Free some space, then run the command again. |
| "Pastas sem acesso" is greater than zero | macOS did not let the Terminal read some folders. | Give the Terminal [Full Disk Access](usage.md#when-pastas-sem-acesso-is-not-zero). |
| macOS asks whether the Terminal can access Documents, Desktop or Downloads | Normal macOS protection for those folders. | Click **Allow** to include them in the scan, or **Don't Allow** to leave them out. |
| `erro: não foi possível impedir downloads do iCloud` | macOS refused the setting that prevents iCloud downloads, so MacCleaner stopped on purpose. | Please [open an issue](https://github.com/renatoasterio/MacCleaner/issues) with your macOS version. |
| The window is slow or the fans spin up | Animations redraw the window continuously. | Open the appearance panel and set "Animações" to "Reduzida" or "Desligada". |
| The project folder takes a lot of space | Build output is kept in the `target` folder. | Run `cargo clean` inside the project folder. The next build recreates it. |

## Still stuck?

Open an issue at [github.com/renatoasterio/MacCleaner/issues](https://github.com/renatoasterio/MacCleaner/issues). Include:

- your macOS version (Apple menu, **About This Mac**);
- the command you ran;
- the full message the Terminal printed.

Issues in English or Portuguese are both fine.
