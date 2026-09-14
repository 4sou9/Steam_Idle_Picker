"""Draws the app icon source (app-icon.png, 1024x1024).

A solid Steam blue circle with a white infinity sign in the center. The sign
is a lemniscate of Bernoulli drawn with a uniform stroke; at the crossing one
strand passes over the other and the lower strand stops short of it.

Run `python scripts/make-icon.py` and then `npx tauri icon app-icon.png` to
regenerate src-tauri/icons. Requires Pillow.
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
SCALE = 4  # supersampling for smooth edges
S = SIZE * SCALE

STEAM_BLUE = (27, 40, 56)  # #1B2838
WHITE = (255, 255, 255)

# Infinity geometry, as fractions of the icon size.
HALF_WIDTH = 0.29     # distance from the center to the tip of each loop
LOOP_HEIGHT = 1.25    # vertical stretch; 1.0 is the pure lemniscate
STROKE = 0.062
GAP = 0.028           # clearance between the strands at the crossing
SAMPLES = 4000


def point(t: float) -> tuple[float, float]:
    """Lemniscate of Bernoulli, centered in the icon (screen coordinates)."""
    denom = 1 + math.sin(t) ** 2
    x = math.cos(t) / denom
    y = math.sin(t) * math.cos(t) / denom
    return (
        S * (0.5 + HALF_WIDTH * x),
        S * (0.5 - HALF_WIDTH * LOOP_HEIGHT * y),
    )


def main() -> None:
    icon = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(icon)

    margin = S * 0.02
    d.ellipse((margin, margin, S - margin, S - margin), fill=STEAM_BLUE)

    r = S * STROKE / 2
    # The curve passes the center at t = pi/2 (drawn on top) and t = 3*pi/2.
    over = [point(math.pi / 2 + (k / 200 - 0.5) * 1.2) for k in range(201)]
    clearance = r * 2 + S * GAP

    for i in range(SAMPLES):
        t = 2 * math.pi * i / SAMPLES
        x, y = point(t)
        under = abs(t - 3 * math.pi / 2) < 0.6
        if under and min(math.hypot(x - ox, y - oy) for ox, oy in over) < clearance:
            continue
        d.ellipse((x - r, y - r, x + r, y + r), fill=WHITE)

    out = Path(__file__).resolve().parent.parent / "app-icon.png"
    icon.resize((SIZE, SIZE), Image.LANCZOS).save(out)
    print(f"saved {out}")


if __name__ == "__main__":
    main()
