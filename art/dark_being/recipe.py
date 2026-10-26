"""Who the dark being is, as numbers: a warrior of obsidian.

The Dual mage's dark half, drawn as a body of its own. The owner,
2026-10-26: "a bodybuilder's physique, add some viking ... it's muscle
everywhere". So a clean classical build, not anatomy pushed until it bent
(the demon before it read misshapen): a man at the top of MakeHuman's
muscle and lean, so every muscle shows, a strong V, full chest and arms, the
stomach cut. On the Champion's skeleton (``build.CLASS``), the reference
proportions, so nothing is stretched or squat. Bald.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=1.0, weight=0.42, proportions=1.0,
                races={"caucasian": 0.6, "african": 0.2, "asian": 0.2}) + [
    ("torso/torso-vshape-incr.target", 0.6),
    ("torso/torso-muscle-pectoral-incr.target", 0.5),
    ("torso/torso-muscle-dorsi-incr.target", 0.5),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-waist-circ-decr.target", 0.3),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 0.6), ("upperarm-shoulder-muscle", 0.6), ("lowerarm-muscle", 0.5),
    ("upperleg-muscle", 0.5))]

GIRTH = {}
HEAD_SCALE = 0.96
