# Usage

English · [Português](usage.pt-BR.md)

How to use the MacCleaner window and the command line tool. If you have not installed it yet, start with [Getting started](getting-started.md).

> The interface is in Portuguese. This guide gives the on-screen word in quotes, followed by its meaning.

## The window

![The MacCleaner window](../assets/app-current.png)

| Area | What it shows |
|---|---|
| **Status bar** (top) | The app name, the free space on the startup disk, the time, and the appearance icon. |
| **Navigation rail** (left) | The seven screens. The lit one is where you are. |
| **Disk gauge** (bottom left) | How full the startup disk is. It turns amber and says "crítico" (critical) when less than 10% is free. |
| **Work area** (right) | The title and a one-line description of the current screen, then its content. |

### Keyboard

| Key | Action |
|---|---|
| `1` to `7` | Go to a screen, in the order of the rail |
| `Esc` | Close the appearance panel |
| `Command + Q` | Quit |

### Appearance

Click the sliders icon at the right end of the status bar to open the "Aparência" (appearance) panel.

![The appearance panel open over the dashboard, in the prototype](../assets/screens/appearance.png)

| Control | Options |
|---|---|
| "Paleta" (palette) | "Agência" (cyan), "Terminal" (green), "Tático" (amber) |
| "Animações" (animations) | "Completa": everything moves. "Reduzida": short transitions only, nothing loops. "Desligada": no animation at all. |
| "Brilho" (glow) | How strong the glow around titles and highlights is. All the way left turns it off. |
| "Rever abertura" | Plays the start-up sequence again. |

Close the panel with `Esc` or by clicking anywhere outside it. These choices are not saved yet: the app opens with the defaults each time.

To save battery, the animations slow to one frame per second whenever the window is not in front.

## The seven screens

Each screen is being built to match the prototype. Until then it shows "Módulo em construção" (module under construction). Click a name to see the design.

| Key | Screen | What it will do | State |
|---|---|---|---|
| `1` | ["Painel"](../assets/screens/dashboard.png) (dashboard) | Summary of the disk, memory, temperature and the space you can recover | In progress |
| `2` | ["Limpeza"](../assets/screens/cleanup.png) (cleanup) | Caches, temporary files and logs, with a risk label on each item | In progress |
| `3` | ["Duplicados"](../assets/screens/duplicates.png) (duplicates) | Files with identical content. One copy always stays. | In progress |
| `4` | ["Fotos"](../assets/screens/photos.png) (photos) | Identical and visually similar photos in your folders | In progress |
| `5` | ["Desenvolvedor"](../assets/screens/developer.png) (developer) | Build and dependency folders of code projects | In progress |
| `6` | ["Mapa do disco"](../assets/screens/disk-map.png) (disk map) | A map where each block is as large as the space it uses | In progress |
| `7` | ["Monitor"](../assets/screens/monitor.png) | Processor, memory, disk, fans and temperatures, live | In progress |

## The command line tool

`maccleaner-cli` runs the same engine as the app, from the Terminal. It is read-only: it never changes or deletes anything.

Run it from the project folder:

```bash
cargo run --release -p maccleaner-cli -- scan ~/Documents
```

### What `scan` accepts

```text
maccleaner-cli scan [--top <N>] <PATH>
```

| Part | Meaning | Default |
|---|---|---|
| `<PATH>` | The folder to measure | required |
| `--top <N>` | How many of the largest items to list | 15 |

Examples:

```bash
cargo run --release -p maccleaner-cli -- scan ~
```

```bash
cargo run --release -p maccleaner-cli -- scan ~/Projects --top 30
```

### Reading the result

```text
Pasta              /Users/you/Projects
Itens              1.386.543
Ocupado em disco   24,2 GB
Tamanho lógico     20,6 GB
Somente na nuvem   0 B
Pastas sem acesso  0
Tempo              9,3 s

Maiores itens
     12,5 GB  portal-web/
     11,7 GB  app-mobile/
    249,9 KB  notes.txt
```

| Line | Meaning |
|---|---|
| "Pasta" | The folder that was measured. |
| "Itens" | How many files and folders are inside it. |
| "Ocupado em disco" | The space those items really take on the disk. This is the number that matters when you want to free space. |
| "Tamanho lógico" | The size of the content itself. It differs from the line above because the disk stores files in blocks and can compress them. |
| "Somente na nuvem" | Content that exists only in iCloud. It takes no space on this Mac, and MacCleaner does not download it. |
| "Pastas sem acesso" | Folders macOS did not let the Terminal open. See the next section. |
| "Tempo" | How long the scan took. |
| "Maiores itens" | The largest items directly inside the folder. A `/` after the name marks a folder; "(pacote)" marks an app or a library measured as one item. |

Numbers use the Brazilian format: a comma for decimals and a dot for thousands.

### When "Pastas sem acesso" is not zero

macOS protects some folders, such as Mail, Safari and other apps' data. To let the scan read them, give the Terminal **Full Disk Access**:

1. Open **System Settings**.
2. Go to **Privacy & Security**, then **Full Disk Access**.
3. Turn on the switch next to **Terminal** (or the terminal app you use).
4. Quit the Terminal and open it again.

This only lets the Terminal read those folders. You can turn it off the same way.

### Measuring the whole disk

```bash
cargo run --release -p maccleaner-cli -- scan /
```

This takes longer and needs Full Disk Access to be complete. Two things are left out on purpose: other disks connected to the Mac, and the internal volume macOS shows twice, which would count every file two times.

## Next

- [Safety](safety.md): what MacCleaner can and cannot delete.
- [Questions and troubleshooting](faq.md).
