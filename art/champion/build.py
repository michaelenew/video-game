"""The Champion, built from nothing but these scripts and CC0 data.

    art/blender.sh art/champion/build.py [--look out.png] [--paint] [--size 2048]

The stages are ``art/lib/character.py``; what he wears is ``outfit.py``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "champion"
CLASS = "Champion"

# The painter's colours, linear albedo, judged lit in the game (the
# Reaver's lesson: Blender's studio flatters by about three times).
# Deep blue against his weapon's gold; pale breeches; warm leather.
COLOURS = {
    "knit": (0.04, 0.035, 0.03),
    "knit/2": (0.36, 0.21, 0.12),      # skin, warm brown
    "knit/3": (0.07, 0.04, 0.022),     # gloves
    "knit/4": (0.025, 0.015, 0.01),    # brows
    "doublet": (0.018, 0.03, 0.085),
    "breeches": (0.26, 0.22, 0.17),
    "boot": (0.035, 0.022, 0.015),
    "leather": (0.10, 0.05, 0.022),
    "leather_dark": (0.04, 0.025, 0.015),
    "hair": (0.016, 0.012, 0.010),
    "scalp": (0.0072, 0.0054, 0.0045),   # under the cards: the hair's own colour in shadow
    "team": (0.80, 0.80, 0.80),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.12, 0.16, 0.22),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (1.0, 0.75, 0.3)
METAL = {}
# Shade warmer than the Reaver's violet: a duellist in the sun.
ART = {"shade_tint": (0.6, 0.6, 0.9), "trim": (1.6, 1.4, 1.0)}

character.build(sys.modules[__name__])
