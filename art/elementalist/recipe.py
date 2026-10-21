"""Who the Elementalist is, as numbers.

Tall and narrow with a heavy head, a caster's silhouette
(``view::skeleton::build_for``): long-limbed and spare, strong in the
forearms and hands that she drives into the ground.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.55, weight=0.30, proportions=1.0,
                # East Asian.
                races={"caucasian": 0.10, "african": 0.05, "asian": 0.85}) + [
    ("breast/female-young-averagemuscle-minweight-mincup-averagefirmness.target", 0.4),
    ("head/head-oval.target", 0.3),
]

# The chest, pressed: one shape under her tunic.
BUST = 0.55
FLAT = mh.macro(gender="male", age="young", muscle=0.55, weight=0.30, proportions=1.0,
                races={"caucasian": 0.10, "african": 0.05, "asian": 0.85})

GIRTH = {}
HEAD_SCALE = 1.0
