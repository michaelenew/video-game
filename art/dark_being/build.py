"""The dark being, built from nothing but these scripts and CC0 data.

    art/blender.sh art/dark_being/build.py [--look out.png] [--paint] [--size 2048]

On the Champion's skeleton (``CLASS``): the being is posed with the Dual
mage's own pose when she casts, which is angles and so fits any skeleton
(``view::being::BODIES``). Everything else is its own (``recipe.py``,
``outfit.py``).
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import numpy as np  # noqa: E402

import character  # noqa: E402
import garment as g  # noqa: E402
import outfit  # noqa: E402,F401
import wardrobe  # noqa: E402
from recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402,F401

NAME = "dark_being"
CLASS = "Champion"

# Linear albedo, judged lit. **Obsidian**: near-black glass, glossy, so the
# muscle's shapes are drawn by its highlights; and violet in its shade and on
# its ridges (``ART``) so they still read in shadow. Iron that is iron:
# grey, metal, its edges worn bright. A kilt of black wool.
COLOURS = {
    "skin": (0.022, 0.016, 0.032),
    "iron": (0.30, 0.29, 0.28),
    "kilt": (0.03, 0.022, 0.03),
    "eye": (0.05, 0.03, 0.06),
    "eye/2": (0.45, 0.25, 0.75),
    "eye/3": (0.01, 0.01, 0.015),
}
GLOW = (0.72, 0.42, 1.0)
METAL = {"iron": (1.0, 0.38), "skin": (0.0, 0.22)}
# Violet light painted on: in the shade, and on every ridge of the muscle.
ART = {
    "shade_tint": (1.4, 0.7, 2.6),
    "ao_strength": 0.6,
    "warm": (1.25, 0.9, 1.9),
    "edge_light": 0.9,
    "curv_gain": 9.0,
    "top_warm": 0.3,
    "crease_dark": 0.25,
    "trim": (1.9, 1.85, 1.8),
    "stroke_value": 0.06,
}


def _cells(p, scale, seed):
    """The gap between the nearest and second-nearest of a scatter of points
    in 3D: small along the boundaries between their cells, which is where
    glass breaks."""
    q = p * scale
    base = np.floor(q)
    f1 = np.full(len(p), 9.0)
    f2 = np.full(len(p), 9.0)
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            for dz in (-1, 0, 1):
                cell = base + np.array([dx, dy, dz])
                h = np.sin(cell @ np.array([127.1, 311.7, 74.7]) + seed) * 43758.5453
                jit = np.stack([h, h * 1.37, h * 1.71], -1) % 1.0
                d = np.linalg.norm(cell + jit - q, axis=-1)
                f2 = np.where(d < f1, f1, np.minimum(f2, d))
                f1 = np.minimum(f1, d)
    return (f2 - f1) / scale


def veins(pos):
    """**Shatter**: round each rift the obsidian broken into shards -- small
    near the impact and larger further out, the cracks between them thin and
    blazing at the centre and fading to hairlines at the edge -- with the
    impact itself burning. Everywhere else, nothing."""
    out = np.zeros(pos.shape[:-1])
    flat = pos.reshape(-1, 3)
    o = out.reshape(-1)
    for centre, radius in wardrobe.EXTRAS.get("rifts", []):
        c = g.g2b(np.array(centre))
        d = np.linalg.norm(flat - c, axis=1)
        near = d < radius
        if not near.any():
            continue
        q, dn = flat[near], d[near] / radius
        # Shards: finer toward the impact.
        gap = np.minimum(_cells(q, 70.0, 1.0), _cells(q, 34.0, 5.0) * 1.3)
        width = 0.0022 * (1.0 - dn) + 0.0006
        crack = np.exp(-(gap / width) ** 2)
        # A few long cracks running out from the impact: lines in angle.
        rel = q - c
        ang = np.arctan2(rel[:, 2], rel[:, 0]) + np.arctan2(rel[:, 1], np.linalg.norm(rel[:, [0, 2]], axis=1))
        rays = np.exp(-((np.sin(ang * 4.5) * dn * radius) / 0.0018) ** 2) * (dn < 0.95)
        core = np.exp(-(dn / 0.12) ** 2)
        fade = (1.0 - dn) ** 1.2
        o[near] = np.maximum(o[near], np.clip(np.maximum(crack, rays * 0.9) * fade + core, 0, 1))
    return out


character.build(sys.modules[__name__])
