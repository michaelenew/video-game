"""Who the Blood mage is, as numbers.

"Ordinary proportions carrying more weight than they should ... does not
look athletic" (``view::skeleton::build_for``): soft through the middle,
heavy-hipped, strong enough to swing a war scythe, nobody's idea of a
fighter -- which is the point of her.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.45, weight=0.66, proportions=0.6,
                # Black.
                races={"caucasian": 0.10, "african": 0.85, "asian": 0.05}) + [
    ("head/head-round.target", 0.25),
]

# The chest, pressed: flat -- no breasts (the owner, 2026-10-21).
BUST = 1.0
FLAT = mh.macro(gender="male", age="young", muscle=0.45, weight=0.66, proportions=0.6,
                races={"caucasian": 0.10, "african": 0.85, "asian": 0.05})

GIRTH = {}
HEAD_SCALE = 1.0
