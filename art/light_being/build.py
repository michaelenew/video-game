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

# Linear albedo, judged lit. Pale silver plate, bright and mostly metal (white
# and half metal read as plastic, dark and fully metal as gunmetal); gold
# only at the belt; a pale skirt and cloak with a cool grey in them.
COLOURS = {
    "knit": (0.36, 0.24, 0.16),
    "knit/2": (0.36, 0.24, 0.16),
    "knit/3": (0.80, 0.82, 0.88),      # gauntlets: plate
    "knit/4": (0.36, 0.24, 0.16),
    "plate": (0.80, 0.82, 0.88),
    "gold": (0.90, 0.70, 0.36),
    "cloak": (0.82, 0.84, 0.88),
    "skirt": (0.70, 0.72, 0.76),
}
GLOW = (1.0, 0.93, 0.76)
METAL = {"plate": (0.72, 0.2), "gold": (0.8, 0.3), "knit": (0.6, 0.25)}
ART = {"shade_tint": (0.75, 0.8, 1.0), "warm": (1.12, 1.06, 0.98)}
LUMINOUS = {"plate": 0.008, "cloak": 0.02}


def veins(pos):
    """The helm's slit of light: a thin band across its face at the eyes."""
    eye = wardrobe.EXTRAS.get("visor")
    if eye is None:
        return np.zeros(pos.shape[:-1])
    c = g.g2b(np.array(eye))
    rel = pos - c
    across = np.abs(rel[..., 0])
    front = -rel[..., 1]          # Blender's -y is forward
    height = rel[..., 2]
    slit = np.exp(-(height / 0.006) ** 2) * (across < 0.055) * (front > -0.01)
    return np.clip(slit * np.clip(1.2 - across / 0.055, 0, 1) * 1.6, 0, 1)


character.build(sys.modules[__name__])
