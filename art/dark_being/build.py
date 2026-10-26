"""The dark being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/dark_being/build.py [--look out.png] [--paint] [--size 2048]

On the Champion's skeleton (``CLASS``), as the knight is (``recipe``): the being is posed with the
Dual mage's own pose when she casts, which is angles and so fits any skeleton
(``view::being::BODIES``). Everything else is its own (``recipe.py``,
``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]


import character  # noqa: E402
import outfit  # noqa: E402,F401
from recipe import BODY, FIT, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "dark_being"
CLASS = "Champion"

# Linear albedo, judged lit. **Obsidian**: near-black glass, glossy, so the
# muscle's shapes are drawn by its highlights; and a little violet on its
# ridges (``ART``) so they still read in shadow -- a little: the painter adds
# a ridge's light rather than multiplying it, so on black even a fifth of it
# turned the whole body lavender (the first two builds). Iron that is iron:
# grey, metal, its edges worn bright. A kilt of black wool.
COLOURS = {
    "skin": (0.009, 0.007, 0.013),
    "iron": (0.30, 0.29, 0.28),
    "kilt": (0.028, 0.02, 0.022),
    "hide": (0.035, 0.024, 0.02),
    "eye": (0.05, 0.03, 0.06),
    "eye/2": (0.45, 0.25, 0.75),
    "eye/3": (0.01, 0.01, 0.015),
}
# Its rims are plate, not honed blades: they do not shine.
EDGE_GLOW = False
GLOW = (0.72, 0.42, 1.0)
# Muscle carved into the body (``anatomy``): the masses into the game's mesh,
# the grooves into its normal map.
SCULPT = 1.0
SCULPT_SHARE = 1.0
METAL = {"iron": (1.0, 0.38), "skin": (0.0, 0.48)}
# Violet light painted on: in the shade, and on every ridge of the muscle.
ART = {
    "shade_tint": (1.2, 0.7, 2.2),
    "ao_strength": 0.5,
    "warm": (1.0, 0.6, 1.7),
    "edge_light": 0.07,
    "curv_gain": 8.0,
    "top_warm": 0.05,
    "crease_dark": 0.3,
    "trim": (1.9, 1.85, 1.8),
    "stroke_value": 0.04,
}


character.build(sys.modules[__name__])
