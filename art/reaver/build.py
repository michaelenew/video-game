"""The Shadow Reaver, built from nothing but this script and CC0 data.

    art/blender.sh art/reaver/build.py [--look out.png] [--paint] [--size 2048]

Body (MakeHuman, fitted to her skeleton) -> costume (cut from the body, cloth
draped) -> blades -> painted textures -> ``crates/game/assets/reaver.glb``.
The stages are ``art/lib/character.py``; the design is
``docs/design/exploration/0024_a_character_done_properly.md``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import character  # noqa: E402
import outfit  # noqa: E402,F401
from reaver_recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "reaver"
CLASS = "Shadow Reaver"

# The painter's colours: what each material is, before light. Brighter and
# more saturated than the thing itself, as a painted figure's are, because
# the painting and then the game's light both take value out.
# Pushed toward purple on the owner's word (2026-10-20): plum leather, a
# violet hood, cloth with violet in its black.
# Linear albedo, judged lit in the game (CLAUDE.md, "Judge a palette lit"):
# the first set, three times this, came out pale lavender and tan under the
# arena's sun.
COLOURS = {
    "knit": (0.035, 0.024, 0.06),
    "knit/2": (0.56, 0.38, 0.31),      # skin, fair
    "knit/4": (0.06, 0.03, 0.05),      # brows and the smudge round her eyes
    "knit/3": (0.07, 0.035, 0.05),     # glove
    "leather": (0.09, 0.035, 0.055),
    "leather_dark": (0.045, 0.024, 0.05),
    "hood": (0.05, 0.02, 0.11),
    "mask": (0.03, 0.015, 0.045),
    "metal": (0.62, 0.64, 0.70),
    "metal_dark": (0.08, 0.075, 0.09),
    "wrap": (0.14, 0.09, 0.06),
    "trousers": (0.032, 0.018, 0.045),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.22, 0.14, 0.42),
    "eye/3": (0.01, 0.01, 0.015),
    "sash": (0.80, 0.80, 0.80),
}
# Her colour of light: the shadow's violet, on the blades' edges.
GLOW = (0.55, 0.25, 1.0)
METAL = {"metal": (1.0, 0.32), "metal_dark": (0.9, 0.45)}


character.build(sys.modules[__name__])
