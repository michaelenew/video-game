"""Who the Champion is, as numbers: the body under the doublet.

A duellist from the line of the aristocrats' champions (``champion.md``):
the reference middleweight (``view::skeleton::Build::REFERENCE``), a fencer's
build rather than a wrestler's -- long in the arm, strong through the
shoulders, lean at the waist -- and a squared jaw a painter would give a
hero.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=0.70, weight=0.40, proportions=1.0) + [
    ("head/head-square.target", 0.35),
    ("chin/chin-bones-incr.target", 0.3),
]

GIRTH = {}
HEAD_SCALE = 1.0
