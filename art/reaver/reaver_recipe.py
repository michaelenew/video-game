"""Who the Reaver is, as numbers: the body under the costume.

A young woman, lean and strong rather than heavy -- the most mobile thing in
the arena reads as the lightest (`view::skeleton::build_for`) -- with
MakeHuman's "ideal" proportions, which are the long-legged, small-headed
heroic ones a painted fantasy game draws people with.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.72, weight=0.32, proportions=1.0)

# Thickness across each bone, against MakeHuman's: 1 keeps it.
GIRTH = {}

# The head is carried whole and scaled, never stretched.
HEAD_SCALE = 1.0
