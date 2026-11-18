#!/usr/bin/env python3
"""Draws the app icon: AppIcon.icns next to this file and a preview in docs/assets/.

Run:  python3 apps/desktop/bundle/make_icon.py

Needs the Python package Pillow and the macOS tools `sips` and `iconutil`.
This is a build-asset tool; it is not part of DiskMarshal. The colors are the
Agency palette of the interface.
"""
import math
import pathlib
import subprocess
import tempfile

from PIL import Image, ImageDraw, ImageFilter

HERE = pathlib.Path(__file__).resolve().parent
SIZE = 1024
SURFACE = (7, 12, 16, 255)
PANEL = (10, 19, 25, 255)
LINE = (36, 80, 94, 255)
ACCENT = (47, 230, 255, 255)
WARN = (255, 176, 32, 255)


def arc_points(center, radius, start_degrees, end_degrees, step=1.0):
    cx, cy = center
    count = max(2, int(abs(end_degrees - start_degrees) / step))
    for index in range(count + 1):
        angle = math.radians(start_degrees + (end_degrees - start_degrees) * index / count)
        yield (cx + radius * math.cos(angle), cy + radius * math.sin(angle))


def draw_icon():
    icon = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(icon)
    margin, radius = 100, 185  # the macOS icon grid leaves a margin around the shape
    box = (margin, margin, SIZE - margin, SIZE - margin)
    draw.rounded_rectangle(box, radius, fill=SURFACE, outline=LINE, width=6)

    # Faint grid, clipped to the rounded shape.
    grid = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    grid_draw = ImageDraw.Draw(grid)
    for offset in range(margin, SIZE - margin, 64):
        grid_draw.line([(offset, margin), (offset, SIZE - margin)], fill=(23, 48, 57, 150), width=2)
        grid_draw.line([(margin, offset), (SIZE - margin, offset)], fill=(23, 48, 57, 150), width=2)
    mask = Image.new("L", (SIZE, SIZE), 0)
    ImageDraw.Draw(mask).rounded_rectangle((margin + 8, margin + 8, SIZE - margin - 8, SIZE - margin - 8), radius - 8, fill=255)
    icon.paste(grid, (0, 0), Image.composite(grid.split()[3], Image.new("L", (SIZE, SIZE), 0), mask))

    center = (SIZE / 2, SIZE / 2 + 10)
    gauge = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    gauge_draw = ImageDraw.Draw(gauge)
    # Track: 270 degrees, opening at the bottom.
    gauge_draw.line(list(arc_points(center, 250, 135, 405)), fill=LINE, width=44, joint="curve")
    # Value: most of the track in the accent color, the last stretch in amber.
    gauge_draw.line(list(arc_points(center, 250, 135, 330)), fill=ACCENT, width=44, joint="curve")
    gauge_draw.line(list(arc_points(center, 250, 338, 372)), fill=WARN, width=44, joint="curve")
    # Tick marks outside the track.
    for degrees in range(135, 406, 15):
        angle = math.radians(degrees)
        inner = (center[0] + 300 * math.cos(angle), center[1] + 300 * math.sin(angle))
        outer = (center[0] + 322 * math.cos(angle), center[1] + 322 * math.sin(angle))
        gauge_draw.line([inner, outer], fill=LINE, width=8)
    # Radar sweep and hub in the middle.
    gauge_draw.pieslice((center[0] - 170, center[1] - 170, center[0] + 170, center[1] + 170), 250, 320, fill=(47, 230, 255, 70))
    gauge_draw.ellipse((center[0] - 170, center[1] - 170, center[0] + 170, center[1] + 170), outline=LINE, width=6)
    gauge_draw.ellipse((center[0] - 26, center[1] - 26, center[0] + 26, center[1] + 26), fill=ACCENT)

    glow = gauge.filter(ImageFilter.GaussianBlur(18))
    icon.alpha_composite(glow)
    icon.alpha_composite(gauge)

    # Corner brackets, the signature of the interface's panels.
    draw = ImageDraw.Draw(icon)
    inset, arm = margin + 62, 92
    draw.line([(inset, inset + arm), (inset, inset), (inset + arm, inset)], fill=ACCENT, width=12, joint="curve")
    far = SIZE - inset
    draw.line([(far - arm, far), (far, far), (far, far - arm)], fill=ACCENT, width=12, joint="curve")
    return icon


def main():
    icon = draw_icon()
    with tempfile.TemporaryDirectory() as directory:
        iconset = pathlib.Path(directory) / "AppIcon.iconset"
        iconset.mkdir()
        for points in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                pixels = points * scale
                name = f"icon_{points}x{points}{'@2x' if scale == 2 else ''}.png"
                icon.resize((pixels, pixels), Image.LANCZOS).save(iconset / name)
        subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(HERE / "AppIcon.icns")], check=True)
    preview = HERE.parents[2] / "docs" / "assets" / "app-icon.png"
    icon.resize((256, 256), Image.LANCZOS).save(preview)
    print(f"wrote {HERE / 'AppIcon.icns'}")


if __name__ == "__main__":
    main()
