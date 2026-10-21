"""The Blood mage, built from nothing but these scripts and CC0 data.

    art/blender.sh art/blood_mage/build.py [--look out.png] [--paint] [--size 2048]

The stages are ``art/lib/character.py``; what she wears is ``outfit.py``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "blood_mage"
CLASS = "Blood mage"

# Black, oxblood and bone, linear albedo, judged lit: the coat a black
# with red in it, the leather the colour of old blood, ash-pale hair.
COLOURS = {
    "knit": (0.03, 0.02, 0.02),
    "knit/2": (0.13, 0.065, 0.04),     # skin, deep brown
    "knit/3": (0.025, 0.012, 0.015),   # gloves
    "knit/4": (0.04, 0.006, 0.012),    # lips, brows, the shadow round the eyes
    "coat": (0.022, 0.008, 0.012),
    "leather": (0.12, 0.012, 0.022),
    "leather_dark": (0.035, 0.012, 0.012),
    "glove": (0.025, 0.012, 0.015),
    "trousers": (0.025, 0.015, 0.018),
    "boot": (0.025, 0.015, 0.015),
    "hair": (0.36, 0.34, 0.36),
    "team": (0.80, 0.80, 0.80),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.30, 0.03, 0.05),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (0.8, 0.05, 0.12)
METAL = {}
ART = {"shade_tint": (0.7, 0.45, 0.75), "trim": (1.5, 1.2, 1.2)}

character.build(sys.modules[__name__])
