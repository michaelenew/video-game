"""The light being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/light_being/build.py [--look out.png] [--paint] [--size 2048]

On the Champion's skeleton (``CLASS``): the being is posed with the Dual
mage's own pose, which is angles and so fits any skeleton
(``view::being::BODIES``). Everything else is its own (``recipe.py``,
``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "light_being"
CLASS = "Champion"

# Linear albedo, judged lit. Polished steel, cool and fully metal so it
# shines (white and half metal, the first build read as white plastic);
# warm gold trim; a white cloak and skirt.
COLOURS = {
    "knit": (0.30, 0.20, 0.14),
    "knit/2": (0.36, 0.24, 0.16),      # skin, warm brown
    "knit/3": (0.80, 0.80, 0.84),      # gauntlets: steel
    "knit/4": (0.10, 0.06, 0.04),      # brows
    "steel": (0.56, 0.59, 0.66),
    "gold": (1.0, 0.72, 0.28),
    "cloak": (0.95, 0.93, 0.88),
    "skirt": (0.90, 0.88, 0.82),
    "halo": (1.0, 0.9, 0.6),
    "hair": (0.55, 0.40, 0.22),
    "scalp": (0.25, 0.18, 0.10),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.45, 0.62, 0.85),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (1.0, 0.86, 0.5)
METAL = {"steel": (0.9, 0.16), "gold": (0.85, 0.25), "knit": (0.0, 0.6)}
ART = {"shade_tint": (0.7, 0.75, 1.0)}
# It is light: the plate and cloth give off a little of their own colour,
# the ring behind its head all of it.
LUMINOUS = {"cloak": 0.03, "halo": 1.0}

character.build(sys.modules[__name__])
