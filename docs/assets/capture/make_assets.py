#!/usr/bin/env python3
"""Regenerates the screenshots and the tour GIF in docs/assets/ from the real app.

Run from anywhere:  python3 docs/assets/capture/make_assets.py

Needs cargo, ffmpeg and the Python package Pillow. This script is a tool for
the documentation; it is not part of MacCleaner.

How it works: the app is built in its development capture mode, which saves
one frame to a PNG and exits. Each shot runs it with HOME pointing at a
demonstration home folder created here, so the pictures show made-up caches,
apps, projects, duplicates and photos, a made-up Docker, and no path or
program of the machine that takes them. Nothing in the real home folder is read or changed. The window of
the app flashes on screen for a moment at each shot.

Pass --work-dir to choose where the demonstration folder is created; by
default it goes to a temporary folder that is removed at the end.
"""
import argparse
import json
import math
import os
import pathlib
import random
import shutil
import subprocess
import tempfile
import time

from PIL import Image, ImageDraw

REPOSITORY = pathlib.Path(__file__).resolve().parents[3]
ASSETS = REPOSITORY / "docs" / "assets"
BINARY = REPOSITORY / "target" / "debug" / "maccleaner"
SCREENS = ["dashboard", "cleanup", "apps", "duplicates", "photos", "developer", "docker", "disk_map", "monitor"]
PALETTES = ["agency", "terminal", "tactical"]
SHOT_WIDTH = 1280
TOUR_WIDTH = 900
TOUR_SECONDS_PER_SCREEN = 1.8
DAY = 86_400
MEGABYTE = 1_000_000


# ---------------------------------------------------------------- demo home

def write(home, relative, megabytes, age_days=0):
    """Creates a file of the given size whose content compresses to nothing less."""
    path = home / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(random.randbytes(int(megabytes * MEGABYTE)))
    age(path, age_days)
    return path


def age(path, days):
    """Makes a file or folder look last changed `days` ago."""
    moment = time.time() - days * DAY
    os.utime(path, (moment, moment))


def age_tree(folder, days):
    for directory, _, files in os.walk(folder):
        for name in files:
            age(pathlib.Path(directory) / name, days)
        age(pathlib.Path(directory), days)


def project(home, name, marker, folders, days):
    root = home / "Projetos" / name
    root.mkdir(parents=True, exist_ok=True)
    (root / marker).write_text("# demonstration project\n")
    (root / "src").mkdir(exist_ok=True)
    (root / "src" / "main.txt").write_text("demonstration source\n")
    for folder, megabytes in folders:
        for index in range(3):
            write(root, f"{folder}/part-{index}.bin", megabytes / 3)
    age_tree(root, days)


def grain(image, amount):
    """Adds film grain, so the file sizes look like those of real photos."""
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            shift = random.randint(-amount, amount)
            pixels[x, y] = tuple(max(0, min(255, channel + shift)) for channel in pixels[x, y])
    return image


def sunset(width=1200, height=900):
    image = Image.new("RGB", (width, height))
    draw = ImageDraw.Draw(image)
    for y in range(height):
        share = y / height
        draw.line([(0, y), (width, y)], fill=(int(250 - 120 * share), int(150 - 60 * share), int(90 + 80 * share)))
    draw.ellipse([width * 0.6, height * 0.25, width * 0.6 + height * 0.28, height * 0.53], fill=(255, 236, 170))
    ridge = [(x, height * 0.72 - height * 0.12 * math.sin(x / width * 7) - height * 0.05 * math.sin(x / width * 19))
             for x in range(0, width + 1, 8)]
    draw.polygon([(0, height)] + ridge + [(width, height)], fill=(38, 30, 60))
    draw.rectangle([0, height * 0.86, width, height], fill=(20, 40, 70))
    return grain(image, 10)


def city(width=1200, height=900):
    image = Image.new("RGB", (width, height), (12, 16, 40))
    draw = ImageDraw.Draw(image)
    x = 0
    while x < width:
        building_width = random.randint(60, 150)
        building_height = random.randint(int(height * 0.25), int(height * 0.75))
        draw.rectangle([x, height - building_height, x + building_width - 8, height], fill=(26, 32, 66))
        for window_y in range(height - building_height + 14, height - 10, 26):
            for window_x in range(x + 10, x + building_width - 20, 22):
                if random.random() < 0.55:
                    draw.rectangle([window_x, window_y, window_x + 9, window_y + 12], fill=(255, 214, 120))
        x += building_width
    draw.ellipse([width * 0.12, height * 0.08, width * 0.12 + 90, height * 0.08 + 90], fill=(235, 238, 250))
    return grain(image, 6)


def forest(width=1200, height=900):
    image = Image.new("RGB", (width, height), (150, 200, 235))
    draw = ImageDraw.Draw(image)
    for _ in range(46):
        x = random.randint(-40, width)
        tree_height = random.randint(int(height * 0.3), int(height * 0.8))
        draw.polygon([(x, height), (x + 70, height - tree_height), (x + 140, height)], fill=(20, random.randint(60, 140), 50))
    return grain(image, 10)


def desk(width=1200, height=900):
    image = Image.new("RGB", (width, height), (232, 226, 214))
    draw = ImageDraw.Draw(image)
    draw.rectangle([width * 0.1, height * 0.15, width * 0.55, height * 0.8], fill=(250, 250, 248))
    for line in range(14):
        draw.line([(width * 0.14, height * 0.22 + line * 36), (width * 0.5, height * 0.22 + line * 36)], fill=(90, 90, 100), width=6)
    draw.ellipse([width * 0.65, height * 0.3, width * 0.9, height * 0.3 + width * 0.25], fill=(120, 70, 40))
    return grain(image, 5)


def save_photo(home, image, relative, **options):
    path = home / "Imagens" / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, **options)


def app(home, name, identifier, version, megabytes):
    """Creates an app bundle with the description every app carries."""
    bundle = f"Applications/{name}.app/Contents"
    description = home / bundle / "Info.plist"
    description.parent.mkdir(parents=True, exist_ok=True)
    description.write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n<plist version="1.0"><dict>\n'
        f"<key>CFBundleIdentifier</key><string>{identifier}</string>\n"
        f"<key>CFBundleShortVersionString</key><string>{version}</string>\n"
        "</dict></plist>\n"
    )
    write(home, f"{bundle}/MacOS/programa", megabytes)


def build_demo_apps(home):
    """Apps to uninstall. The largest is the one the apps screen examines."""
    app(home, "Editor Exemplo", "com.exemplo.editor", "4.2", 62)
    app(home, "Navegador Exemplo", "com.exemplo.navegador", "128.0", 48)
    app(home, "Planilhas Exemplo", "com.exemplo.planilhas", "11.3", 37)
    app(home, "Música Exemplo", "com.exemplo.musica", "2.9", 21)
    app(home, "Conversa Exemplo", "com.exemplo.conversa", "7.1", 16)
    app(home, "Notas Exemplo", "com.exemplo.notas", "1.8", 9)
    # What the editor left behind, besides its cache.
    write(home, "Library/Application Support/Editor Exemplo/projetos.db", 9.4)
    write(home, "Library/Containers/com.exemplo.editor/Data/documentos.db", 5.2)
    write(home, "Library/HTTPStorages/com.exemplo.editor/httpstorages.sqlite", 0.4)
    write(home, "Library/Preferences/com.exemplo.editor.plist", 0.004)
    write(home, "Library/Preferences/com.exemplo.editor.ajudante.plist", 0.002)
    write(home, "Library/Saved Application State/com.exemplo.editor.savedState/janelas.data", 0.06)


def build_demo_home(home):
    random.seed(7)
    build_demo_apps(home)
    for app, megabytes in [("com.exemplo.navegador", 26), ("com.exemplo.editor", 14), ("com.exemplo.musica", 8)]:
        for index in range(4):
            write(home, f"Library/Caches/{app}/cache-{index}.bin", megabytes / 4)
    write(home, "Library/Logs/DiagnosticReports/editor-2026-10-02.crash", 1.4)
    write(home, "Library/Logs/sincronizacao.log", 1.8)
    for index in range(3):
        write(home, f"tmp/exportacao-{index}.tmp", 2.1, age_days=20)
    write(home, "tmp/recente.tmp", 3.0)
    write(home, ".Trash/apresentacao-antiga.key", 4.2)
    for index in range(4):
        write(home, f"Library/Developer/Xcode/DerivedData/AppExemplo-abc/Build/objeto-{index}.o", 5)
    write(home, "Library/Developer/CoreSimulator/Caches/dyld/cache.bin", 6)
    for folder, megabytes in [(".npm/_cacache", 9), (".cargo/registry/cache", 7), (".gradle/caches", 5)]:
        write(home, f"{folder}/pacotes.bin", megabytes)

    project(home, "loja-web", "package.json", [("node_modules", 18), ("dist", 3)], days=75)
    project(home, "api-pedidos", "Api.csproj", [("bin", 6), ("obj", 4)], days=40)
    project(home, "conversor-cli", "Cargo.toml", [("target", 22)], days=3)
    project(home, "app-mobile", "pubspec.yaml", [(".dart_tool", 5), ("build", 9)], days=120)
    project(home, "analise-dados", "pyproject.toml", [(".venv", 14), ("__pycache__", 0.3)], days=10)

    video = write(home, "Vídeos/ferias-2025.mov", 8, age_days=200)
    for copy_path, days in [("Downloads/ferias-2025 (1).mov", 90), ("Mesa/ferias-2025 cópia.mov", 30)]:
        (home / copy_path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(video, home / copy_path)
        age(home / copy_path, days)
    contract = write(home, "Documentos/contrato-assinado.pdf", 2.5, age_days=150)
    (home / "Downloads").mkdir(exist_ok=True)
    shutil.copyfile(contract, home / "Downloads/contrato-assinado.pdf")
    audio = write(home, "Música/set-ao-vivo.wav", 5, age_days=300)
    (home / "Backup").mkdir(exist_ok=True)
    shutil.copyfile(audio, home / "Backup/set-ao-vivo.wav")

    picture = sunset()
    save_photo(home, picture, "Viagem/por-do-sol.jpg", quality=95)
    save_photo(home, picture.resize((900, 675), Image.LANCZOS), "Exportadas/por-do-sol-web.jpg", quality=82)
    picture = city()
    save_photo(home, picture, "Viagem/cidade-noite.png")
    save_photo(home, picture.resize((1000, 750), Image.LANCZOS), "Exportadas/cidade-noite.jpg", quality=90)
    picture = forest()
    save_photo(home, picture, "Viagem/floresta.jpg", quality=92)
    save_photo(home, picture.resize((1100, 825), Image.LANCZOS), "Backup antigo/floresta (1).jpg", quality=85)
    save_photo(home, desk(), "Documentos/mesa.jpg", quality=92)

    history = home / "Library/Application Support/MacCleaner/history.jsonl"
    history.parent.mkdir(parents=True, exist_ok=True)
    now = int(time.time())
    entries = [
        (2 * 3600, "deleted", "Library/Caches/com.exemplo.video", 1_840_000_000),
        (2 * 3600, "deleted", "Library/Developer/Xcode/DerivedData/AppAntigo-xyz", 3_210_000_000),
        (DAY, "trashed", "Downloads/instalador-antigo.dmg", 612_000_000),
        (3 * DAY, "deleted", "Projetos/site-2023/node_modules", 742_000_000),
        (3 * DAY, "trashed", "Imagens/Exportadas/retrato (2).jpg", 6_400_000),
        (9 * DAY, "deleted", ".npm/_cacache/content-v2", 1_120_000_000),
    ]
    # The newest entry goes last, the way the app appends them.
    with history.open("w") as file:
        for seconds_ago, action, relative, size in reversed(entries):
            record = {
                "timestamp_unix_seconds": now - seconds_ago,
                "action": action,
                "path": str(home / relative),
                "reclaimable_bytes": size,
            }
            file.write(json.dumps(record, ensure_ascii=False) + "\n")


# ------------------------------------------------------------------ capture

def capture(home, screen, palette, target):
    environment = {
        "HOME": str(home),
        "TMPDIR": str(home / "tmp"),
        "PATH": "/usr/bin:/bin",
        "EFRAME_SCREENSHOT_TO": str(target),
        "MACCLEANER_SCREEN": screen,
        "MACCLEANER_PALETTE": palette,
        "MACCLEANER_ANALYZE": "1",
        "MACCLEANER_DEMO": "1",
        "MACCLEANER_SCAN": str(home),
        "MACCLEANER_DEV": str(home / "Projetos"),
        "MACCLEANER_DUP": str(home),
        "MACCLEANER_PHOTOS": str(home / "Imagens"),
        "MACCLEANER_APPS": str(home / "Applications"),
    }
    # The made-up Docker is asked only for its own screen: next to a small
    # demonstration home, its gigabytes would be all the dashboard shows.
    if screen == "docker":
        environment["MACCLEANER_DOCKER"] = "1"
    subprocess.run([str(BINARY)], env=environment, check=True, capture_output=True, timeout=180)
    return Image.open(target).convert("RGB")


def scaled(image, width):
    return image.resize((width, round(image.height * width / image.width)), Image.LANCZOS)


def save_small(image, path):
    """Saves with a palette of 256 colors: a fraction of the size, same look."""
    path.parent.mkdir(parents=True, exist_ok=True)
    image.quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(path, optimize=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--work-dir", type=pathlib.Path, help="where to create the demonstration folder")
    arguments = parser.parse_args()

    subprocess.run(
        [os.environ.get("CARGO", "cargo"), "build", "--package", "maccleaner", "--features", "capture"],
        cwd=REPOSITORY, check=True,
    )
    work = arguments.work_dir or pathlib.Path(tempfile.mkdtemp(prefix="maccleaner-assets-"))
    home = work / "inicio"
    shots = work / "shots"
    try:
        shutil.rmtree(home, ignore_errors=True)
        shots.mkdir(parents=True, exist_ok=True)
        build_demo_home(home)

        frames = []
        for screen in SCREENS:
            image = capture(home, screen, PALETTES[0], shots / f"{screen}.png")
            save_small(scaled(image, SHOT_WIDTH), ASSETS / "screens" / f"{screen.replace('_', '-')}.png")
            frames.append(scaled(image, TOUR_WIDTH))
            print("captured", screen)

        gap, tile_width = 16, 620
        tiles = [scaled(capture(home, "dashboard", palette, shots / f"palette-{palette}.png"), tile_width)
                 for palette in PALETTES]
        sheet = Image.new("RGB", (tile_width * len(tiles) + gap * (len(tiles) + 1), tiles[0].height + 2 * gap), (5, 8, 11))
        for index, tile in enumerate(tiles):
            sheet.paste(tile, (gap + index * (tile_width + gap), gap))
        save_small(sheet, ASSETS / "palettes.png")
        print("captured palettes")

        for index, frame in enumerate(frames):
            frame.save(shots / f"tour-{index:02d}.png")
        subprocess.run(
            ["ffmpeg", "-y", "-loglevel", "error", "-framerate", f"1/{TOUR_SECONDS_PER_SCREEN}",
             "-i", str(shots / "tour-%02d.png"),
             "-vf", "fps=5,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=none",
             str(ASSETS / "tour.gif")],
            check=True,
        )
        print("made tour.gif")
    finally:
        shutil.rmtree(home, ignore_errors=True)
        shutil.rmtree(shots, ignore_errors=True)
        if not arguments.work_dir:
            shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
