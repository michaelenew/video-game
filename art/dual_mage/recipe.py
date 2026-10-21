"""Who the Dual mage is, as numbers.

"The floatiest and the frailest" (``view::skeleton::build_for``): slight,
long-legged, narrow-shouldered -- a vessel under load (``dual-mage.md``),
and her body should look as if it could not hold much more.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.40, weight=0.22, proportions=1.0,
                # White: Northern European.
                races={"caucasian": 0.92, "african": 0.0, "asian": 0.08}) + [
    ("breast/female-young-averagemuscle-minweight-mincup-averagefirmness.target", 0.4),
    ("head/head-invertedtriangular.target", 0.3),
]

# The chest, pressed: one shape under her tunic.
BUST = 0.55
FLAT = mh.macro(gender="male", age="young", muscle=0.40, weight=0.22, proportions=1.0,
                races={"caucasian": 0.92, "african": 0.0, "asian": 0.08})

GIRTH = {}
HEAD_SCALE = 1.0
