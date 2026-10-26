"""The light being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/light_being/build.py [--look out.png] [--paint] [--size 2048]

On the Champion's skeleton (``CLASS``), as the dark being is: posed with the
Dual mage's own pose when she casts, which is angles and so fits any
skeleton (``view::being::BODIES``). Everything else is its own
(``recipe.py``, ``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import numpy as np  # noqa: E402

import character  # noqa: E402
import garment as g  # noqa: E402
import outfit  # noqa: E402,F401
import wardrobe  # noqa: E402
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "light_being"
CLASS = "Champion"

# Linear albedo, judged lit. Pale silver plate, half metal and glossy: the
# game has no reflections for metal to show, so fully metal reads as
# gunmetal, and matte white as plastic; gold
# only at the belt; a pale skirt and cloak with a cool grey in them.
COLOURS = {
    # What shows between the plates, and the hands: steel, a shade under the
    # plate, never the cloth-brown the cast's knit defaults to.
    "knit": (0.60, 0.62, 0.68),
    "knit/2": (0.60, 0.62, 0.68),
    "knit/3": (0.78, 0.80, 0.84),      # gauntlets: plate
    "knit/4": (0.60, 0.62, 0.68),
    "plate": (0.9, 0.91, 0.94),
    "trim": (0.92, 0.66, 0.26),
    "cloak": (0.84, 0.86, 0.90),
    "skirt": (0.70, 0.72, 0.78),
}
# Its rims are plate, not honed blades: they do not shine.
EDGE_GLOW = False
GLOW = (1.0, 0.92, 0.72)
METAL = {"plate": (0.15, 0.22), "trim": (0.8, 0.25), "knit": (0.6, 0.22)}
# No painted trim: the plate's rims are real (``hard``), in gold.
ART = {"shade_tint": (0.75, 0.8, 1.0), "warm": (1.1, 1.05, 0.98), "trim": (1.0, 1.0, 1.0),
       "stitch_dark": 0.0, "edge_light": 0.14}
# A being of light: its plate gives a little light of its own, its gold
# frame a little more, so under any sky it reads white and gold rather than
# as steel taking the sky's colour (2026-10-26).
LUMINOUS = {"cloak": 0.015, "plate": 0.04, "trim": 0.09}


def veins(pos):
    """Light let into the plate: the helm's slit across its eyes, and a
    small sun at the breast -- a disc and eight thin rays."""
    out = np.zeros(pos.shape[:-1])
    eye = wardrobe.EXTRAS.get("visor")
    if eye is not None:
        c = g.g2b(np.array(eye))
        rel = pos - c
        across = np.abs(rel[..., 0])
        front = -rel[..., 1]          # Blender's -y is forward
        slit = np.exp(-(rel[..., 2] / 0.006) ** 2) * (across < 0.055) * (front > -0.01)
        out = np.maximum(out, np.clip(slit * np.clip(1.2 - across / 0.055, 0, 1) * 1.6, 0, 1))
    sun = wardrobe.EXTRAS.get("sun")
    if sun is not None:
        c = g.g2b(np.array(sun))
        rel = pos - c
        flat = np.sqrt(rel[..., 0] ** 2 + rel[..., 2] ** 2)
        front = -rel[..., 1] > -0.06
        ang = np.arctan2(rel[..., 2], rel[..., 0])
        disc = np.exp(-(flat / 0.014) ** 4)
        ring = np.exp(-((flat - 0.024) / 0.0025) ** 2)
        rays = np.exp(-(np.sin(ang * 4.0) * flat / 0.0022) ** 2) * (flat > 0.03) * (flat < 0.07)
        out = np.maximum(out, np.clip((disc + ring + rays * (1 - flat / 0.07)) * front, 0, 1))
    return out


character.build(sys.modules[__name__])
