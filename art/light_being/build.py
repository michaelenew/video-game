"""The light being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/light_being/build.py [--look out.png] [--paint] [--size 2048]

Posed on the Dual mage's skeleton (``CLASS``), because it throws what she
throws; everything else about it is its own (``recipe.py``, ``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, BUST, FLAT, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "light_being"
CLASS = "Dual mage"

# Linear albedo, judged lit: a white robe, a bright warm gold, a paler gold
# mask. The gold is only half metal: fully metal it mirrors the dark of
# the world round it and reads bronze (the first build).
COLOURS = {
    "knit": (0.36, 0.24, 0.16),        # skin, warm brown
    "robe": (0.92, 0.88, 0.78),
    "gold": (1.0, 0.72, 0.28),
    "gold_dark": (0.62, 0.40, 0.12),
    "mask": (1.0, 0.86, 0.52),
    "halo": (1.0, 0.9, 0.55),
    "hair": (0.80, 0.62, 0.30),
    "scalp": (0.36, 0.28, 0.13),
    "hair/2": (0.98, 0.90, 0.62),
}
GLOW = (1.0, 0.82, 0.42)
METAL = {"gold": (0.55, 0.3), "gold_dark": (0.55, 0.38), "mask": (0.5, 0.22), "halo": (0.3, 0.3)}
# It is light, not lit: everything it wears gives off some of its colour,
# the gold more than the robe, the sun behind its head the most.
LUMINOUS = {"robe": 0.03, "gold": 0.06, "gold_dark": 0.04, "mask": 0.08, "hair": 0.04, "halo": 1.0}
ART = {"shade_tint": (0.95, 0.75, 0.55)}

character.build(sys.modules[__name__])
