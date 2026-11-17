# Getting started

English · [Português](getting-started.pt-BR.md)

This guide takes you from nothing to the MacCleaner window open on your Mac, one step at a time. No programming experience is needed.

- **Time:** about 15 minutes, most of it waiting for downloads.
- **Disk space:** keep about 6 GB free for the tools and the build.
- **Tested on:** macOS 27 on Apple Silicon. Other versions and Intel Macs should work but have not been tested yet. If something fails, [tell us](https://github.com/renatoasterio/MacCleaner/issues).

> MacCleaner is in early development. Following this guide is safe: today the app and the command line tool only read your disk. Nothing is deleted.

## How to follow this guide

Every gray box is one command. For each one:

1. Copy the command.
2. Paste it into the Terminal.
3. Press Return and wait until the Terminal accepts typing again.

To open the Terminal, press `Command + Space`, type `Terminal` and press Return.

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

MacCleaner is written in Rust, so you need the Rust tools to build it.

> Already have Rust (`cargo --version` prints a version)? Skip to step 4.

Install the Rust installer:

```bash
brew install rustup
```

Tell the Terminal where to find the Rust tools:

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

You should see something like `cargo 1.100.0`. The project pins the exact Rust version it needs, and downloads it by itself the first time you build.

## Step 4. Download MacCleaner

Choose where the project folder will live. This example uses your home folder.

```bash
cd ~
```

```bash
git clone https://github.com/renatoasterio/MacCleaner.git
```

Enter the folder. Every command from here on must run inside it.

```bash
cd MacCleaner
```

## Step 5. Open the app

```bash
cargo run --release -p maccleaner
```

The first time, this downloads the libraries MacCleaner uses and builds everything. Expect about a minute on a recent Mac and longer on older ones. Lines starting with `Compiling` are normal.

**What you should see:** a short start-up sequence, then this window.

![The MacCleaner window: status bar on top, navigation rail on the left, work area on the right](../assets/app-current.png)

Press `1` to `7` to visit each screen. To quit, press `Command + Q`.

The screens still show "Módulo em construção" (module under construction). That is expected: see the [project status](../../README.md#project-status).

## Step 6. Measure a folder

The command line tool already does real work. It measures a folder and lists its largest items, without changing anything:

```bash
cargo run --release -p maccleaner-cli -- scan ~/Downloads
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

The [usage guide](usage.md) explains every line.

## You are done

| You want to | Do this |
|---|---|
| Open the app again | `cd ~/MacCleaner`, then `cargo run --release -p maccleaner` |
| Get the newest version | `cd ~/MacCleaner`, then `git pull`, then open the app again |
| Free the disk space used by the build | `cd ~/MacCleaner`, then `cargo clean` |
| Remove MacCleaner | Move the `MacCleaner` folder to the Trash |

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
