# Getting started

English · [Português](getting-started.pt-BR.md)

This guide takes you from nothing to DiskMarshal installed on your Mac and open, one step at a time. No programming experience is needed.

- **Time:** about 15 minutes, most of it waiting for downloads and for the build.
- **Disk space:** keep about 6 GB free for the tools and the build. The installed app itself takes about 10 MB.
- **Tested on:** macOS 27 on Apple Silicon. Other versions and Intel Macs should work but have not been tested yet. If something fails, [tell us](https://github.com/renatoasterio/DiskMarshal/issues).

> Installing and opening DiskMarshal deletes nothing. The app only removes what you select, after showing you a preview and asking for confirmation. Read the [safety guide](safety.md) before your first cleanup: it also says plainly what has not been tested yet.

## How to follow this guide

Every gray box is one command. For each one:

1. Copy the command.
2. Paste it into the Terminal.
3. Press Return and wait until the Terminal accepts typing again.

To open the Terminal, press `Command + Space`, type `Terminal` and press Return.

You only need the Terminal to install. After that, DiskMarshal opens like any other app.

## Step 1. Install the Xcode command line tools

These are Apple's basic tools for building software. Git comes with them.

```bash
xcode-select --install
```

A window asks you to confirm. Click **Install** and wait for it to finish.

**Check that it worked:**

```bash
xcode-select -p
```

You should see a path, such as `/Library/Developer/CommandLineTools`.

If the first command answered `command line tools are already installed`, you already have them. Go to the next step.

## Step 2. Install Homebrew

Homebrew installs developer tools on a Mac. First find out whether you already have it:

```bash
brew --version
```

- If you see a version number, such as `Homebrew 5.0.0`, go to step 3.
- If you see `command not found`, open [brew.sh](https://brew.sh), copy the installation command shown at the top of that page, paste it into the Terminal and press Return. When it finishes, it prints a section called **Next steps** with two or three commands. Run those too, then run `brew --version` again.

## Step 3. Install Rust

DiskMarshal is written in Rust, so you need the Rust tools to build it.

> Already have Rust (`cargo --version` prints a version)? Skip to step 4.

Install the Rust installer:

```bash
brew install rustup
```

Homebrew keeps the Rust tools in a folder of its own (`/opt/homebrew/opt/rustup/bin` on Apple Silicon), which the Terminal does not look into by itself. Tell the Terminal where to find them:

```bash
echo "export PATH=\"$(brew --prefix rustup)/bin:\$PATH\"" >> ~/.zshrc
```

Reload the Terminal settings:

```bash
source ~/.zshrc
```

Install Rust itself:

```bash
rustup default stable
```

**Check that it worked:**

```bash
cargo --version
```

You should see something like `cargo 1.100.0`. The project pins the exact Rust version it needs (in the file `rust-toolchain.toml`), and downloads it by itself the first time you build.

## Step 4. Download DiskMarshal

Choose where the project folder will live. This example uses your home folder.

```bash
cd ~
```

```bash
git clone https://github.com/renatoasterio/DiskMarshal.git
```

Enter the folder. Every command from here on must run inside it.

```bash
cd DiskMarshal
```

## Step 5. Build and install the app

```bash
cargo xtask install
```

This one command does four things:

1. Builds DiskMarshal. The first time, it also downloads the libraries DiskMarshal uses. Expect a few minutes. Lines starting with `Compiling` are normal.
2. Assembles the app in `dist/DiskMarshal.app`, inside the project folder: the app itself, a small helper used only for the system cleanup, and the icon.
3. Signs the app, so macOS agrees to run it.
4. Copies it to `/Applications`, the Applications folder of your Mac.

**Check that it worked:** the last lines should be these.

```text
xtask: installed /Applications/DiskMarshal.app
xtask: open it from Launchpad, Spotlight or the Applications folder
```

If the copy fails because your account cannot write to the Applications folder, install the app in the Applications folder of your own user instead:

```bash
cargo xtask install ~/Applications
```

## Step 6. Open the app

Press `Command + Space`, type `DiskMarshal` and press Return. You can also open it from Launchpad or from the Applications folder in Finder.

**What you should see:** a short start-up sequence (click to skip it), then the "Painel" (dashboard).

![The DiskMarshal dashboard: status bar on top, the nine screens on the left, gauges and the recoverable space on the right](../assets/screens/dashboard.png)

*Screenshot of the app with demonstration data. Your numbers will be different, and "Espaço recuperável" (recoverable space) starts with "não analisado" (not analyzed) on every row.*

Press `1` to `7` to visit each screen. To quit, press `Command + Q`.

## Step 7. Give Full Disk Access (optional)

macOS keeps some folders, such as the data of Mail and Safari and the Trash, hidden from every app until you allow it. DiskMarshal works without that permission; it just cannot see or measure those folders.

When the permission is missing, the screens that read your folders show this line at the top:

> Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises. **Abrir ajustes de Acesso Total ao Disco**

It means "Without Full Disk Access, macOS hides part of your home folder from the analyses", followed by the link "open the Full Disk Access settings". To give the permission:

1. Click the link **"Abrir ajustes de Acesso Total ao Disco"**. System Settings opens on the right page.
2. Turn on the switch next to **DiskMarshal**. If DiskMarshal is not in the list, click the **+** button and choose it in the Applications folder.
3. If macOS asks for your password or Touch ID to confirm the change, provide it.
4. Quit DiskMarshal (`Command + Q`) and open it again.

You can turn the switch off the same way at any time.

> **macOS forgets this permission when the app is rebuilt.** The app you just built carries a local signature (called ad hoc), which changes with every build. Each time you run `cargo xtask install` again, macOS treats the result as a new app, and you have to repeat this step. Signing with an identity of your own avoids that: set the environment variable `DISKMARSHAL_SIGNING_IDENTITY` to the name of the identity before running `cargo xtask install`.

## Step 8. Measure a folder from the Terminal (optional)

DiskMarshal also has a small command line tool. It measures a folder and lists its largest items, without changing anything:

```bash
cargo run --release -p diskmarshal-cli -- scan ~/Downloads
```

macOS may ask whether the Terminal can access your Downloads folder. Click **Allow**.

**What you should see:** a summary like this, with your own numbers.

```text
Pasta              /Users/you/Downloads
Itens              2.418
Ocupado em disco   27,1 GB
Tamanho lógico     27,0 GB
Somente na nuvem   0 B
Pastas sem acesso  0
Tempo              0,1 s

Maiores itens
     12,0 GB  Installers/
      9,2 GB  backup-2023.zip
```

The [usage guide](usage.md#the-command-line-tool) explains every line.

## You are done

| You want to | Do this |
|---|---|
| Open the app | Launchpad or Spotlight, like any other app. The Terminal is not needed. |
| Learn what each screen does | Read the [usage guide](usage.md). |
| Get the newest version | `cd ~/DiskMarshal`, then `git pull`, then `cargo xtask install` |
| Install it somewhere else | `cargo xtask install ~/Applications` (or any other folder) |
| Only build the app, without installing | `cargo xtask bundle`. The result is `dist/DiskMarshal.app` inside the project folder. |
| Open the app without installing it | `cargo run --release -p diskmarshal`. The "Sistema" (system) group of the cleanup screen may be unavailable this way: see the [usage guide](usage.md#the-sistema-group). |
| Free the disk space used by the build | `cd ~/DiskMarshal`, then `cargo clean`. The installed app keeps working. |
| Remove DiskMarshal | Move `/Applications/DiskMarshal.app` to the Trash. Then, if you want, delete the removal history in `~/Library/Application Support/DiskMarshal/` and the `DiskMarshal` project folder. |

## If something goes wrong

The [questions and troubleshooting](faq.md) page lists the common error messages and how to fix each one.

## For contributors

To run the same checks as the continuous integration, install the dependency auditor once and run the quality gate:

```bash
brew install cargo-deny
```

```bash
cargo xtask check
```

Then read [CONTRIBUTING.md](../../CONTRIBUTING.md).
