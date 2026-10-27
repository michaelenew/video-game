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
    ("torso/torso-vshape-incr.target", 1.0),
    ("torso/torso-muscle-pectoral-incr.target", 0.5),
    ("torso/torso-muscle-dorsi-incr.target", 0.5),
    ("stomach/stomach-tone-incr.target", 1.0),
    ("measure/measure-waist-circ-decr.target", 0.7),
    ("measure/measure-hips-circ-decr.target", 0.9),
    # A statue's chest: no nipples to speak of.
    ("breast/nipple-size-decr.target", 1.0),
    ("breast/nipple-point-decr.target", 1.0),
    # A man's neck: thick and short, the traps rising into it. A thin
    # column on a big body read as a monster's (2026-10-26).
    ("neck/neck-scale-horiz-incr.target", 0.8),
    ("neck/neck-scale-depth-incr.target", 0.6),
    ("neck/neck-scale-vert-decr.target", 0.5),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 0.7), ("upperarm-shoulder-muscle", 0.4), ("lowerarm-muscle", 0.6),
    ("upperleg-muscle", 0.5), ("lowerleg-muscle", 0.6))]

# Built up evenly, one value down the whole trunk and one down each limb:
# girths that differ step at the joint between them.
# The trunk's taper is MakeHuman's (``BODY``: the V, the waist), not the
# girth's: built up evenly it read as a box (2026-10-26, beside a scan of
# the Lincoln Mars).
GIRTH = {"root": 1.0, "spine": 1.0, "chest": 1.02, "arm.l": 1.06, "arm.r": 1.06,
         "forearm.l": 1.06, "forearm.r": 1.06, "thigh.l": 1.03, "thigh.r": 1.03,
         "shin.l": 1.04, "shin.r": 1.04}

# The trunk fitted plainly, as the knight's is: smoothing, widening and
# relaxing it (``fit.fit``'s ``trunk``) were for the Bulwark's shoulders, and
# on the Champion's they pressed a crease across the top of the chest.
FIT = None
# Full size: a small head on a big body is a monster's, not a man's.
HEAD_SCALE = 1.0

# **The body is Blender Studio's realistic male** (``art/sources``, CC0; the
# owner, 2026-10-27: "Just use blender's model for the dark side as a
# starting point"), fitted to the skeleton through a plain MakeHuman man of
# about his build (``base_mesh``). ``BODY`` and the girths above are the
# MakeHuman body it replaced, kept for the clay check.
BASE = "human_base_mesh_male"
PROXY = mh.macro(gender="male", age="young", muscle=0.6, weight=0.5, proportions=1.0)
