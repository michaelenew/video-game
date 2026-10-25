"""The dark being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/dark_being/build.py [--look out.png] [--paint] [--size 2048]

On the Bulwark's skeleton (``CLASS``), the broadest in the game: the being is
posed with the Dual mage's own pose, which is angles and so fits any
skeleton (``view::being::BODIES``). Everything else is its own
(``recipe.py``, ``outfit.py``).
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
CLASS = "Bulwark"

# Linear albedo, judged lit. Skin of a dark charcoal with violet in its
# black, so the muscle's light and shade read on it; dark iron; black horn,
# paler toward the tip; violet eyes.
COLOURS = {
    "skin": (0.05, 0.035, 0.06),
    "iron": (0.12, 0.11, 0.11),
    "horn": (0.04, 0.03, 0.035),
    "hide": (0.035, 0.022, 0.02),
    "eye": (0.7, 0.4, 1.0),
    "eye/2": (0.85, 0.6, 1.0),
    "eye/3": (1.0, 0.9, 1.0),
}
GLOW = (0.7, 0.35, 1.0)
METAL = {"iron": (0.9, 0.55), "horn": (0.0, 0.3), "skin": (0.0, 0.55)}
ART = {"shade_tint": (0.5, 0.4, 0.9)}


def veins(pos):
    """Faint violet seams in the skin, as if the dark showed through where it
    is stretched over the muscle: thin, and only on the trunk and arms."""
    a = paint._hash_noise(pos, 6.0, 1.0)
    line = np.exp(-((a - 0.5) / 0.006) ** 2)
    height = pos[..., 2]
    upper = np.clip((height - 1.05) / 0.25, 0, 1)
    return np.clip(line * upper * 0.55, 0, 1)


character.build(sys.modules[__name__])
