"""Who the light being is, as numbers: a seraph.

The Dual mage's light half, drawn as a body of its own (the owner,
2026-10-25: "three different beings"). Tall and athletic where she is
slight: a woman's frame well above average muscle and lean, broad enough
through the shoulders to carry armour, long in the neck, a head held small
and high.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.78, weight=0.3, proportions=1.0,
                races={"caucasian": 0.2, "african": 0.4, "asian": 0.4}) + [
    ("breast/female-young-averagemuscle-minweight-mincup-averagefirmness.target", 0.4),
]

# The chest, pressed under the cuirass: one shape.
BUST = 0.8
FLAT = mh.macro(gender="male", age="young", muscle=0.78, weight=0.3, proportions=1.0,
                races={"caucasian": 0.2, "african": 0.4, "asian": 0.4})

GIRTH = {"chest": 1.12, "arm.l": 1.1, "arm.r": 1.1, "forearm.l": 1.05, "forearm.r": 1.05}
HEAD_SCALE = 0.92
