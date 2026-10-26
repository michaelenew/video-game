"""Who the dark being is, as numbers: a warrior of obsidian.

The Dual mage's dark half, drawn as a body of its own. The owner,
2026-10-26: "a bodybuilder's physique, add some viking ... it's muscle
everywhere". So a clean classical build, not anatomy pushed until it bent
(the demon before it read misshapen): a man at the top of MakeHuman's
muscle and lean, so every muscle shows, a strong V, full chest and arms, the
stomach cut. On the Bulwark's skeleton (``build.CLASS``), whose shoulders are
broad enough to carry it. Bald.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=1.0, weight=0.6, proportions=1.0,
                races={"caucasian": 0.6, "african": 0.2, "asian": 0.2}) + [
    ("torso/torso-vshape-incr.target", 0.6),
    ("torso/torso-muscle-pectoral-incr.target", 0.5),
    ("torso/torso-muscle-dorsi-incr.target", 0.5),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-waist-circ-decr.target", 0.3),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 1.0), ("upperarm-shoulder-muscle", 1.0), ("lowerarm-muscle", 0.8),
    ("upperleg-muscle", 0.7))]

# Built up evenly, one value down the whole trunk and one down each limb,
# so nothing steps at a joint: on her frame MakeHuman's strongest man still
# came out slim (the first obsidian build).
GIRTH = {"root": 1.08, "spine": 1.08, "chest": 1.08, "arm.l": 1.15, "arm.r": 1.15,
         "forearm.l": 1.12, "forearm.r": 1.12, "thigh.l": 1.08, "thigh.r": 1.08}
HEAD_SCALE = 0.94
