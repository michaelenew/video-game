"""Who the dark being is, as numbers: a warrior of obsidian.

The Dual mage's dark half, drawn as a body of its own. The owner,
2026-10-26: "a bodybuilder's physique, add some viking ... it's muscle
everywhere". So a clean classical build, not anatomy pushed until it bent
(the demon before it read misshapen): a man at the top of MakeHuman's
muscle and lean, so every muscle shows, a strong V, full chest and arms, the
stomach cut. On the Champion's skeleton (``build.CLASS``), as the knight is:
on the Bulwark's, whose shoulders are far wider than any man's, the trunk
was stretched to reach them into a flat shelf with the deltoids hanging
under it (the owner, 2026-10-26: "just make a muscular dude"). The bulk is
in the muscle, not the frame. Bald.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=1.0, weight=0.45, proportions=1.0,
                races={"caucasian": 0.6, "african": 0.2, "asian": 0.2}) + [
    ("torso/torso-vshape-incr.target", 0.6),
    ("torso/torso-muscle-pectoral-incr.target", 0.5),
    ("torso/torso-muscle-dorsi-incr.target", 0.5),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-waist-circ-decr.target", 0.0),
    ("measure/measure-hips-circ-decr.target", 0.9),
    # A statue's chest: no nipples to speak of.
    ("breast/nipple-size-decr.target", 1.0),
    ("breast/nipple-point-decr.target", 1.0),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 1.0), ("upperarm-shoulder-muscle", 1.0), ("lowerarm-muscle", 0.8),
    ("upperleg-muscle", 0.5), ("lowerleg-muscle", 0.6))]

# Built up evenly, one value down the whole trunk and one down each limb:
# girths that differ step at the joint between them.
GIRTH = {"root": 1.06, "spine": 1.08, "chest": 1.1, "arm.l": 1.22, "arm.r": 1.22,
         "forearm.l": 1.15, "forearm.r": 1.15, "thigh.l": 1.03, "thigh.r": 1.03,
         "shin.l": 1.04, "shin.r": 1.04}

# The trunk fitted plainly, as the knight's is: smoothing, widening and
# relaxing it (``fit.fit``'s ``trunk``) were for the Bulwark's shoulders, and
# on the Champion's they pressed a crease across the top of the chest.
FIT = None
HEAD_SCALE = 0.94
