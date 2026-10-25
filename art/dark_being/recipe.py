"""Who the dark being is, as numbers: a titan.

The Dual mage's dark half, drawn as a body of its own (the owner,
2026-10-25: "three different beings"). Not a man with the muscle slider up
-- that read as "a guy in a bad skin suit" (the owner, 2026-10-25) -- but
anatomy pushed past a person's: a deep V from shoulders to a narrow waist,
slabs of pectoral and lat, round shoulders, arms thick at the biceps and the
forearm, a neck as wide as the head, the stomach cut, the hands big. Lean,
so every mass shows, and a head a little small, which makes a figure read
as huge.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=1.0, weight=0.4, proportions=1.0,
                races={"caucasian": 0.34, "african": 0.33, "asian": 0.33}) + [
    ("torso/torso-vshape-incr.target", 1.0),
    ("torso/torso-muscle-pectoral-incr.target", 1.0),
    ("torso/torso-muscle-dorsi-incr.target", 1.0),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-waist-circ-decr.target", 0.6),
    ("measure/measure-neck-circ-incr.target", 1.0),
    ("measure/measure-upperarm-circ-incr.target", 1.0),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 1.0), ("upperarm-shoulder-muscle", 1.0), ("lowerarm-muscle", 1.0),
    ("upperleg-muscle", 0.8), ("lowerleg-muscle", 0.6), ("hand-scale", 0.7))]

# No girth added per bone: it steps at every joint (a shelf across the
# chest, the first build). The bulk is the targets'.
GIRTH = {}
HEAD_SCALE = 0.84
