"""The Dual mage, built from nothing but these scripts and CC0 data.

    art/blender.sh art/dual_mage/build.py [--look out.png] [--paint] [--size 2048]

The stages are ``art/lib/character.py``; what she wears is ``outfit.py``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "dual_mage"
CLASS = "Dual mage"

# Halves, linear albedo, judged lit: every split material's first colour is
# her left (obsidian, violet in its black) and its second her right (ivory).
COLOURS = {
    "knit": (0.04, 0.035, 0.04),
    "knit/2": (0.34, 0.20, 0.12),      # skin, olive
    "knit/4": (0.08, 0.03, 0.06),      # brows, lips, kohl
    "vessel": (0.022, 0.012, 0.05),
    "vessel/2": (0.52, 0.47, 0.38),
    "vessel_l": (0.022, 0.012, 0.05),
    "vessel_r": (0.52, 0.47, 0.38),
    "boot": (0.02, 0.012, 0.03),
    "boot/2": (0.40, 0.34, 0.25),
    "gold": (0.75, 0.52, 0.20),
    "obsidian": (0.02, 0.015, 0.035),
    "hair": (0.018, 0.012, 0.03),
    "hair/2": (0.62, 0.60, 0.62),
    "team": (0.80, 0.80, 0.80),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.25, 0.20, 0.45),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (0.9, 0.8, 1.0)
METAL = {"gold": (1.0, 0.3), "obsidian": (0.4, 0.12)}
ART = {"shade_tint": (0.6, 0.5, 0.95)}

character.build(sys.modules[__name__])
