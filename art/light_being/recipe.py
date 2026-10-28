"""Who the light being is, as numbers: a celestial knight.

The Dual mage's light half, drawn as a body of its own (the owner,
2026-10-26): "a celestial knight. Not holy ... but light". Plate covers it,
so the body under it only has to carry the plate -- lean, so the plate's
own designed shape is what shows (``outfit.cuirass``): a strong man's
frame on the Champion's skeleton (``build.CLASS``), the same as the dark
being's, so the halves meet.
"""

import mh

BODY = mh.macro(gender="male", age="young", muscle=0.7, weight=0.3, proportions=1.0,
                races={"caucasian": 0.3, "african": 0.35, "asian": 0.35}) + [
    ("torso/torso-vshape-incr.target", 1.0),
    # In hard at the waist and hips, so the plate can be: it stands clear
    # of the body, and a waist no narrower than his could not narrow (the
    # owner, 2026-10-28: "narrower waist, wider shoulders, less clunky").
    ("measure/measure-waist-circ-decr.target", 1.0),
    ("measure/measure-hips-circ-decr.target", 1.0),
] + [(f"armslegs/{s}-{part}-incr.target", w) for s in "lr" for part, w in (
    ("upperarm-muscle", 0.8), ("upperarm-shoulder-muscle", 0.8), ("lowerarm-muscle", 0.6))]

# Slim legs, so the plate on them is: plate is the limb's own size, and on
# MakeHuman's thighs the cuisses stood as columns (2026-10-28).
GIRTH = {"thigh.l": 0.86, "thigh.r": 0.86, "shin.l": 0.9, "shin.r": 0.9}
HEAD_SCALE = 0.95
