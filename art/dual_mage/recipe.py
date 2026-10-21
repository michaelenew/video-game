"""Who the Dual mage is, as numbers.

"The floatiest and the frailest" (``view::skeleton::build_for``): slight,
long-legged, narrow-shouldered -- a vessel under load (``dual-mage.md``),
and her body should look as if it could not hold much more.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.40, weight=0.22, proportions=1.0,
                # North African.
                races={"caucasian": 0.60, "african": 0.30, "asian": 0.10}) + [
    ("breast/female-young-averagemuscle-minweight-mincup-averagefirmness.target", 0.4),
    ("head/head-invertedtriangular.target", 0.3),
]

GIRTH = {}
HEAD_SCALE = 1.0
