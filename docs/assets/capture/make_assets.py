#!/usr/bin/env python3
"""Regenerates the prototype screenshots and the tour GIF in docs/assets/.

Run from anywhere:  python3 docs/assets/capture/make_assets.py

Needs Google Chrome, ffmpeg and the Python package Pillow. Set CHROME to the
browser binary if it is not in the default place. This script is a tool for
the documentation; it is not part of MacCleaner.

How it works: headless Chrome opens a copy of the prototype for each shot and
saves one frame. Under --virtual-time-budget Chrome advances timers but not
CSS animations or requestAnimationFrame, so:

- "still" shots switch the prototype's own motion setting off, which shows
  every screen in its final state;
- "live" shots keep motion on, drive requestAnimationFrame from a timer and
  drop CSS animations. That keeps the timer-driven motion: the start-up
  sequence, the counters and the live graph.
"""
import os
import pathlib
import subprocess
import tempfile

from PIL import Image

REPOSITORY = pathlib.Path(__file__).resolve().parents[3]
PROTOTYPE = REPOSITORY / "design" / "prototype" / "index.html"
ASSETS = REPOSITORY / "docs" / "assets"
CHROME = os.environ.get("CHROME", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")

SCALE = 2                         # device pixels per CSS pixel
VIEWPORT = (1132, 900)            # CSS pixels; larger than the app window
WINDOW_BOX = (16, 16, 1116, 736)  # the app window inside the page, CSS pixels
STILL_WIDTH = 1600
GIF_SIZE = (1000, 654)
BACKDROP = (4, 7, 10)             # --color-surface-void

STILL = "<script>document.documentElement.dataset.motion = 'off';</script>\n"
LIVE = (
    "<script>window.requestAnimationFrame = (cb) => setTimeout(() => cb(performance.now()), 16);</script>\n"
    "<style>*, *::before, *::after { animation: none !important; transition: none !important; }</style>\n"
)


def click(selector, at):
    return f"setTimeout(() => document.querySelector({selector!r}).click(), {at});"


def block(name):
    return f"[...document.querySelectorAll('.tm-block')].find(e => e.textContent.includes({name!r}))"


def hover(name, at):
    return (
        f"setTimeout(() => {{ const b = {block(name)}; const r = b.getBoundingClientRect();"
        f" b.dispatchEvent(new MouseEvent('mousemove', {{bubbles: true, clientX: r.left + r.width * .45,"
        f" clientY: r.top + r.height * .4}})); }}, {at});"
    )


def zoom(name, at):
    return f"setTimeout(() => {block(name)}.click(), {at});"


def palette(name):
    return f"document.documentElement.dataset.palette = {name!r};"


ADMIN_CLEANUP = click("#rule-sys-caches", 300) + click("#act", 600)

# name: (mode, screen address, virtual time in ms, script run after the page's own)
SHOTS = {
    "dashboard": (STILL, "dashboard", 2500, ""),
    "cleanup": (STILL, "cleanup", 2500, ""),
    "cleanup-confirm": (STILL, "cleanup", 2500, ADMIN_CLEANUP),
    "cleanup-touch-id": (STILL, "cleanup", 2500, ADMIN_CLEANUP + click("[data-act=go]", 900)),
    "duplicates": (STILL, "duplicates", 2500, ""),
    "photos": (STILL, "photos", 2500, ""),
    "developer": (STILL, "developer", 2500, ""),
    "disk-map": (STILL, "disk-map", 2500, ""),
    "disk-map-inspect": (STILL, "disk-map", 2500, hover("Projetos", 400)),
    "disk-map-zoom": (STILL, "disk-map", 2500, zoom("Projetos", 400) + hover("portal-web", 900)),
    "monitor": (STILL, "monitor", 3200, ""),
    "appearance": (STILL, "dashboard", 2500, click("#settings-toggle", 400)),
    "palette-terminal": (STILL, "dashboard", 2500, palette("terminal")),
    "palette-tactical": (STILL, "dashboard", 2500, palette("tactical")),
    "boot-1": (LIVE, "dashboard", 450, ""),
    "boot-2": (LIVE, "dashboard", 900, ""),
    "boot-3": (LIVE, "dashboard", 1350, ""),
    "boot-4": (LIVE, "dashboard", 1800, ""),
    "dashboard-1": (LIVE, "dashboard", 2150, ""),
    "dashboard-2": (LIVE, "dashboard", 2350, ""),
    "dashboard-3": (LIVE, "dashboard", 2550, ""),
    "dashboard-4": (LIVE, "dashboard", 2850, ""),
    "monitor-1": (LIVE, "monitor", 2600, ""),
    "monitor-2": (LIVE, "monitor", 3350, ""),
    "monitor-3": (LIVE, "monitor", 4100, ""),
}

# Screenshots published in docs/assets/screens/.
STILLS = [
    "dashboard", "cleanup", "cleanup-confirm", "cleanup-touch-id", "duplicates", "photos",
    "developer", "disk-map", "disk-map-inspect", "monitor", "appearance",
]

# The tour: shot name and how long it stays on screen, in seconds.
TOUR = [
    ("boot-1", .32), ("boot-2", .32), ("boot-3", .32), ("boot-4", .38),
    ("dashboard-1", .16), ("dashboard-2", .16), ("dashboard-3", .16), ("dashboard-4", .3),
    ("dashboard", 1.7), ("cleanup", 1.6), ("cleanup-confirm", 1.4), ("cleanup-touch-id", 1.6),
    ("duplicates", 1.5), ("photos", 1.5), ("developer", 1.5),
    ("disk-map", 1.1), ("disk-map-inspect", 1.4), ("disk-map-zoom", 1.6),
    ("monitor-1", .55), ("monitor-2", .55), ("monitor-3", 1.3),
    ("appearance", 1.4), ("palette-terminal", 1.4), ("palette-tactical", 1.8),
]


def capture(work):
    """Saves one cropped frame per entry of SHOTS and returns them by name."""
    source = PROTOTYPE.read_text()
    shots = {}
    for name, (mode, screen, budget, script) in SHOTS.items():
        page = work / f"{name}.html"
        page.write_text(mode + source + (f"\n<script>{script}</script>\n" if script else ""))
        raw = work / f"{name}.png"
        subprocess.run(
            [
                CHROME, "--headless", "--disable-gpu", "--hide-scrollbars", "--no-first-run",
                f"--user-data-dir={work / 'chrome-profile'}",
                f"--force-device-scale-factor={SCALE}",
                f"--window-size={VIEWPORT[0]},{VIEWPORT[1]}",
                f"--virtual-time-budget={budget}",
                f"--screenshot={raw}",
                f"file://{page}#{screen}",
            ],
            check=True, capture_output=True, timeout=120,
        )
        box = tuple(value * SCALE for value in WINDOW_BOX)
        shots[name] = Image.open(raw).convert("RGB").crop(box)
        print(f"captured {name}", flush=True)
    return shots


def save_png(image, path):
    """Saves with 256 adaptive colors: the flat interface stays sharp at a fraction of the size."""
    path.parent.mkdir(parents=True, exist_ok=True)
    reduced = image.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    reduced.save(path, optimize=True)


def fit_width(image, width):
    return image.resize((width, round(image.height * width / image.width)), Image.LANCZOS)


def write_stills(shots):
    for name in STILLS:
        save_png(fit_width(shots[name], STILL_WIDTH), ASSETS / "screens" / f"{name}.png")

    tile_width, gap = 800, 16
    tiles = [fit_width(shots[name], tile_width) for name in ("dashboard", "palette-terminal", "palette-tactical")]
    strip = Image.new("RGB", (tile_width * 3 + gap * 2, tiles[0].height), BACKDROP)
    for index, tile in enumerate(tiles):
        strip.paste(tile, (index * (tile_width + gap), 0))
    save_png(strip, ASSETS / "palettes.png")


def write_tour(shots, work):
    lines = []
    for index, (name, seconds) in enumerate(TOUR):
        frame = work / f"tour-{index:02d}.png"
        shots[name].resize(GIF_SIZE, Image.LANCZOS).save(frame)
        lines += [f"file '{frame}'", f"duration {seconds}"]
    lines.append(f"file '{frame}'")  # the concat format needs the last file repeated
    playlist = work / "tour.txt"
    playlist.write_text("\n".join(lines) + "\n")
    subprocess.run(
        [
            "ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(playlist),
            "-vf", "split[a][b];[a]palettegen=max_colors=200:stats_mode=full[p];"
                   "[b][p]paletteuse=dither=none:diff_mode=rectangle",
            "-fps_mode", "vfr", "-loop", "0", str(ASSETS / "tour.gif"),
        ],
        check=True,
    )


def main():
    with tempfile.TemporaryDirectory(prefix="maccleaner-assets-") as directory:
        work = pathlib.Path(directory)
        shots = capture(work)
        write_stills(shots)
        write_tour(shots, work)
    print(f"assets written to {ASSETS}")


if __name__ == "__main__":
    main()
