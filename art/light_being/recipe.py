"""Who the light being is, as numbers: a celestial knight.

The Dual mage's light half, drawn as a body of its own (the owner,
2026-10-26): "a celestial knight. Not holy ... but light". Plate covers it,
so the body under it only has to carry the plate broadly: a strong man's
frame on the Champion's skeleton (``build.CLASS``), the same as the dark
being's, so the halves meet.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=0.85, weight=0.5, proportions=1.0,
                races={"caucasian": 0.3, "african": 0.35, "asian": 0.35}) + [
    ("torso/torso-vshape-incr.target", 0.5),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 0.8), ("upperarm-shoulder-muscle", 0.8), ("lowerarm-muscle", 0.6))]

GIRTH = {}
HEAD_SCALE = 0.95
