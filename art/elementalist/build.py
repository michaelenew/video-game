"""The Elementalist, built from nothing but these scripts and CC0 data.

    art/blender.sh art/elementalist/build.py [--look out.png] [--paint] [--size 2048]

The stages are ``art/lib/character.py``; what she wears is ``outfit.py``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "elementalist"
CLASS = "Elementalist"

# Earth colours, linear albedo, judged lit: slate, rust, linen, river stone;
# copper hair.
COLOURS = {
    "knit": (0.05, 0.04, 0.035),
    "knit/2": (0.50, 0.34, 0.23),      # skin
    "knit/4": (0.16, 0.06, 0.05),      # brows, lips
    "tunic": (0.035, 0.05, 0.055),
    "robe": (0.16, 0.055, 0.02),
    "linen": (0.36, 0.31, 0.24),
    "trousers": (0.05, 0.04, 0.03),
    "boot": (0.06, 0.035, 0.02),
    "leather": (0.09, 0.05, 0.025),
    "stone": (0.18, 0.17, 0.155),
    "hair": (0.018, 0.014, 0.013),
    "team": (0.80, 0.80, 0.80),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.20, 0.16, 0.06),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (1.0, 0.5, 0.15)
METAL = {}
ART = {"shade_tint": (0.6, 0.55, 0.85), "trim": (1.5, 1.35, 1.15)}

character.build(sys.modules[__name__])
