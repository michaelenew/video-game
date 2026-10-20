"""Who the Blood mage is, as numbers.

"Ordinary proportions carrying more weight than they should ... does not
look athletic" (``view::skeleton::build_for``): soft through the middle,
heavy-hipped, strong enough to swing a war scythe, nobody's idea of a
fighter -- which is the point of her.
"""

import mh

BODY = mh.macro(gender="female", age="young", muscle=0.45, weight=0.66, proportions=0.6) + [
    ("breast/female-young-averagemuscle-maxweight-averagecup-averagefirmness.target", 0.35),
    ("head/head-round.target", 0.25),
]

GIRTH = {}
HEAD_SCALE = 1.0
