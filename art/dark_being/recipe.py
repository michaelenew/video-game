"""Who the dark being is, as numbers: a hulking demon.

The Dual mage's dark half, drawn as a body of its own (the owner,
2026-10-25): "a heavily muscled torso and arms ... a general classic
hulking demon type figure". On the Bulwark's skeleton (``build.CLASS``),
the broadest in the game, and anatomy pushed past a person's: a deep V,
slabs of pectoral and lat, round heavy shoulders and traps, arms thick at
the biceps and the forearm, a neck as wide as the head, huge hands. Heavy as
well as hard -- a brute, not a bodybuilder -- and a small head, which is
what makes a figure read as huge.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=1.0, weight=0.62, proportions=1.0,
                races={"caucasian": 0.34, "african": 0.33, "asian": 0.33}) + [
    ("torso/torso-vshape-incr.target", 1.0),
    ("torso/torso-muscle-pectoral-incr.target", 1.0),
    ("torso/torso-muscle-dorsi-incr.target", 1.0),
    ("torso/torso-scale-depth-incr.target", 0.6),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-neck-circ-incr.target", 1.0),
    ("measure/measure-upperarm-circ-incr.target", 1.0),
    ("measure/measure-shoulder-dist-incr.target", 1.0),
    ("neck/neck-scale-horiz-incr.target", 1.0),
    ("head/head-square.target", 0.6),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 1.0), ("upperarm-shoulder-muscle", 1.0), ("lowerarm-muscle", 1.0),
    ("upperleg-muscle", 1.0), ("lowerleg-muscle", 0.8), ("hand-scale", 1.0))]

# The trunk and arms a little thicker still, evenly: one value down the
# trunk and one down each arm, so nothing steps at a joint (the shelf across
# the chest of the first titan came from girths that differed).
GIRTH = {"root": 1.08, "spine": 1.08, "chest": 1.08, "arm.l": 1.12, "arm.r": 1.12,
         "forearm.l": 1.12, "forearm.r": 1.12}
HEAD_SCALE = 0.8
