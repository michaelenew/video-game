"""Who the Reaver is, as numbers: the body under the costume.

A young woman, lean and strong rather than heavy -- the most mobile thing in
the arena reads as the lightest (`view::skeleton::build_for`) -- with
MakeHuman's "ideal" proportions, which are the long-legged, small-headed
heroic ones a painted fantasy game draws people with.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.72, weight=0.32, proportions=1.0,
                # Northern European.
                races={"caucasian": 0.90, "african": 0.0, "asian": 0.10}) + [
    # Athletic, in a sports top (the owner, 2026-10-20): a small, firm,
    # close-set bust, carried high -- one compressed shape under the clothes
    # rather than two outlines out toward the arms.
    ("breast/female-young-maxmuscle-minweight-mincup-maxfirmness.target", 0.55),
    ("breast/breast-dist-decr.target", 0.85),
    ("breast/breast-point-decr.target", 0.6),
    ("breast/breast-volume-vert-up.target", 0.35),
]

# Thickness across each bone, against MakeHuman's: 1 keeps it.
# The chest, pressed: one shape under her fitted top.
BUST = 0.55
FLAT = mh.macro(gender="male", age="young", muscle=0.72, weight=0.32, proportions=1.0,
                races={"caucasian": 0.90, "african": 0.0, "asian": 0.10})

GIRTH = {}

# The head is carried whole and scaled, never stretched.
HEAD_SCALE = 1.0
