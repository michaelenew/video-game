"""Who the light being is, as numbers: an angel-knight.

The Dual mage's light half, drawn as a body of its own (the owner,
2026-10-25): "between a classic angel and a knight in shining armor ...
strong arms". On the Champion's skeleton (``build.CLASS``), the reference
middleweight and broader than hers: a man's frame, powerful but not
hulking -- the demon is the brute -- with the arms built up, because they
are bare from the shoulder plate to the vambrace.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=0.85, weight=0.5, proportions=1.0,
                races={"caucasian": 0.3, "african": 0.35, "asian": 0.35}) + [
    ("torso/torso-vshape-incr.target", 0.6),
    ("torso/torso-muscle-pectoral-incr.target", 0.6),
    ("measure/measure-upperarm-circ-incr.target", 0.8),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 1.0), ("upperarm-shoulder-muscle", 0.9), ("lowerarm-muscle", 0.8))]

GIRTH = {}
HEAD_SCALE = 0.95
