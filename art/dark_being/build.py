"""The dark being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/dark_being/build.py [--look out.png] [--paint] [--size 2048]

Posed on the Dual mage's skeleton (``CLASS``), because it throws what she
throws; everything else about it is its own (``recipe.py``, ``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import numpy as np  # noqa: E402

import character  # noqa: E402
import outfit  # noqa: E402,F401
import paint  # noqa: E402
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "dark_being"
CLASS = "Dual mage"

# Linear albedo, judged lit: glass black with violet in it, glossy enough
# that each facet throws a clean highlight.
COLOURS = {
    "stone": (0.012, 0.008, 0.022),
    "heart": (0.75, 0.5, 1.0),
}
GLOW = (0.7, 0.35, 1.0)
METAL = {"stone": (0.0, 0.12)}
ART = {"shade_tint": (0.5, 0.4, 1.0)}

# The heart, in the object space ``veins`` is given (Blender's frame:
# +z up, -y forward), metres.
HEART = np.array([0.0, -0.17, 1.42])


def veins(pos):
    """Cracks of light, running out from the heart: two noises crossing
    their middle make a network of lines; near the heart they are wide and
    bright and many, and they thin to hairlines and die out down the limbs."""
    a = paint._hash_noise(pos, 4.0, 1.0)
    b = paint._hash_noise(pos, 9.0, 4.0)
    d = np.linalg.norm(pos - HEART, axis=-1)
    near = np.exp(-d / 0.35)
    width = 0.004 + 0.016 * near
    line = np.exp(-((a - 0.5) / width) ** 2) + np.exp(-((b - 0.5) / (width * 0.8)) ** 2) * near
    return np.clip(line * (0.25 + 0.75 * near) * 1.4, 0, 1)


character.build(sys.modules[__name__])
