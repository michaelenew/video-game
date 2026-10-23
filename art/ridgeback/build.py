"""The Ridgeback, sculpted and baked: the target a grown body is measured
against (``docs/design/exploration/0029_a_target_body.md``).

    cargo run -p anim --bin export_beast -- ridgeback     # its skeleton, once
    art/blender.sh art/ridgeback/build.py [--look out.png] [--size 2048] [--no-export] [--fast]

**What it is** (``docs/design/monsters.md``): thirteen metres nose to tail,
five and a half at the back on long legs, armoured on the head, flanks,
haunch and tail, soft only on top -- the ridge it is named for, standing
proud of the spine, and the nape where the collar plates part.

**What it is drawn as** -- the owner (2026-10-23): "something you'd find in
a triple A game". A living ruin, after Shadow of the Colossus and Monster
Hunter's elders: a front-heavy grazer that bites, its forequarters a hump
under a mantle of stone-thick carapace, a domed back of interlocking scutes
split down the middle by a seam of glowing flesh, a low armoured head with
horns that sweep back and curl, a hide wrinkled like an elephant's, and
legs like a rhino's, plated down the shin.

**How**, as a game creature is made, by script:

1. *Forms*: soft masses as tubes of superellipse sections with the muscles
   running down them, unioned into a coarse skin.
2. *Carapace*: shells grown off that skin wherever the plan says armour --
   a scute pattern from cellular noise, grooves between scutes, a dome on
   each, a chunky rim -- unioned with the skin into one sculpt.
3. *Detail*, millimetres to centimetres, on the two-million-vertex sculpt:
   wrinkles and pebbled scale on the hide, erosion, growth rings, pits and
   cracks on the stone, ribs on the soft places.
4. *Bake*: a light mesh for the game, its colour, normals, glow and
   roughness read off the sculpt into textures.

Every number is metres in the game's frame (``+x`` to the head, ``+y`` up,
``+z`` its right), at rest. The left is written and mirrored.
"""

import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..")
sys.path[:0] = [os.path.join(HERE, "..", "lib")]

import numpy as np  # noqa: E402

import beast as b  # noqa: E402

SKELETON = os.path.join(ROOT, "target/art/ridgeback.json")
OUT = os.path.join(ROOT, "crates/game/assets/ridgeback.glb")
WORK = os.path.join(ROOT, "target/art/ridgeback")

T0 = time.time()


def say(*a):
    print(f"[{time.time() - T0:6.1f}s]", *a, flush=True)


def st(t, w, top, bot, nt=2.0, nb=2.0, bumps=(), dy=0.0, dz=0.0):
    return {"t": t, "w": w, "top": top, "bot": bot, "nt": nt, "nb": nb, "bumps": bumps, "dy": dy, "dz": dz}


def sym(*pairs):
    out = []
    for a, w, h in pairs:
        out.append((a, w, h))
        if abs(a) > 1e-6 and abs(abs(a) - np.pi) > 1e-6:
            out.append((-a, w, h))
    return tuple(out)


def muscled(rows, muscles, nt=2.0, nb=2.0):
    out = []
    for i, (t, w, top, bot) in enumerate(rows):
        bumps = tuple((a, wd, hs[i]) for a, wd, hs in muscles)
        out.append(st(t, w, top, bot, nt=nt, nb=nb, bumps=bumps))
    return out


# ---------------------------------------------------------------------------
# 1. Forms
# ---------------------------------------------------------------------------

Y0 = 4.85  # the torso's axis
TORSO_X = (-3.45, 2.85)
NECK = [(1.9, 5.15, 0), (2.65, 5.32, 0), (3.45, 5.34, 0), (4.10, 5.18, 0)]
HEAD = [(3.95, 5.30, 0), (4.55, 5.30, 0), (5.15, 5.12, 0), (5.80, 4.90, 0)]
TAIL = [(-2.9, 5.02, 0), (-4.15, 4.95, 0), (-5.4, 4.5, 0), (-6.54, 3.9, 0), (-7.75, 3.38, 0)]
FORE = [(1.14, 5.0, -0.80), (1.12, 4.5, -1.0), (1.06, 3.4, -1.04), (1.04, 2.35, -1.0),
        (1.10, 1.2, -1.0), (1.15, 0.45, -1.0), (1.18, 0.0, -1.0)]
HIND = [(-1.75, 5.0, -0.82), (-1.86, 4.4, -0.97), (-1.72, 3.3, -1.02), (-1.55, 2.3, -1.0),
        (-1.77, 1.3, -0.97), (-1.90, 0.6, -0.97), (-1.76, 0.0, -0.97)]
# Angles round a leg, from its front toward its inside (+z on the left).
OUTSIDE, BACK, FRONT = -np.pi / 2, np.pi, 0.0


def torso():
    """Front-heavy: the barrel deepest under the shoulders, the withers a
    hump the mantle sits on, the haunch lower and narrower; the flanks come
    down round the tops of the legs, as an elephant's do."""
    x0, x1 = TORSO_X
    rows = [
        # x,     back, belly, half-width, flank over a leg
        (-3.45, 5.18, 4.58, 0.52, 0.0),
        (-2.85, 5.44, 4.02, 1.04, 0.10),
        (-2.15, 5.52, 3.58, 1.26, 0.30),
        (-1.45, 5.48, 3.50, 1.28, 0.26),
        (-0.75, 5.52, 3.38, 1.36, 0.06),
        (0.0, 5.62, 3.24, 1.48, 0.0),
        (0.6, 5.78, 3.18, 1.54, 0.10),
        (1.1, 5.98, 3.24, 1.54, 0.32),
        (1.65, 6.02, 3.46, 1.34, 0.28),
        (2.3, 5.88, 3.98, 1.02, 0.06),
        (2.85, 5.70, 4.52, 0.74, 0.0),
    ]
    stations = [st((x - x0) / (x1 - x0), w, back - Y0, Y0 - belly, nt=2.4, nb=2.0,
                   bumps=sym((1.05, 0.42, 0.05), (2.25, 0.45, drop)))
                for x, back, belly, w, drop in rows]
    return b.tube([(x0, Y0, 0), (-1.5, Y0, 0), (0.5, Y0, 0), (x1, Y0, 0)], stations, sides=64)


def neck():
    """Thick and short-looking, carried low; a dewlap of skin under it."""
    return b.tube(NECK, [
        st(0.0, 1.02, 0.92, 1.10),
        st(0.35, 0.80, 0.68, 0.92, nt=2.3, bumps=sym((2.2, 0.45, 0.10))),
        st(0.70, 0.66, 0.54, 0.74, nt=2.3, bumps=sym((2.2, 0.45, 0.08))),
        st(1.0, 0.60, 0.50, 0.62, bumps=sym((2.2, 0.45, 0.0))),
    ], sides=56)


def head():
    """A heavy wedge carried low: a broad cranium, a deep muzzle that
    narrows to a beak, a jaw hung under it with the bite's muscle bulging
    behind the eye, a heavy brow over a sunken eye."""
    skull = b.tube(HEAD, [
        st(0.0, 0.58, 0.52, 0.42, bumps=sym((2.0, 0.35, 0.0), (1.2, 0.3, 0.0))),
        st(0.22, 0.66, 0.56, 0.44, nt=2.5, bumps=sym((2.0, 0.35, 0.14), (1.2, 0.3, 0.06))),
        st(0.48, 0.54, 0.46, 0.40, nt=2.6, bumps=sym((2.0, 0.35, 0.02), (1.2, 0.3, 0.12))),
        st(0.78, 0.40, 0.36, 0.33, nt=2.8, bumps=sym((2.0, 0.35, 0.0), (1.2, 0.3, 0.02))),
        st(1.0, 0.20, 0.22, 0.22, bumps=sym((2.0, 0.35, 0.0), (1.2, 0.3, 0.0))),
    ], sides=56)
    jaw = b.tube([(4.05, 4.98, 0), (4.85, 4.78, 0), (5.72, 4.62, 0)], [
        st(0.0, 0.58, 0.22, 0.36, nb=2.6),
        st(0.5, 0.46, 0.20, 0.32, nb=2.8),
        st(0.85, 0.32, 0.16, 0.24, nb=2.6),
        st(1.0, 0.22, 0.12, 0.14),
    ], sides=40)
    jowls = [b.blob((4.22, 5.08, s * 0.52), (0.40, 0.32, 0.20)) for s in (-1, 1)]
    brows = [b.blob((4.70, 5.62, s * 0.40), (0.42, 0.15, 0.22)) for s in (-1, 1)]
    boss = b.blob((5.42, 5.22, 0), (0.30, 0.18, 0.20))
    return [skull, jaw, boss] + jowls + brows


def tail():
    return b.tube(TAIL, [
        st(0.0, 0.72, 0.52, 0.60, nt=2.4),
        st(0.25, 0.54, 0.42, 0.44, nt=2.4),
        st(0.5, 0.40, 0.32, 0.32, nt=2.4),
        st(0.78, 0.26, 0.21, 0.21),
        st(0.94, 0.18, 0.15, 0.15),
        st(1.0, 0.08, 0.08, 0.08),
    ], sides=48)


def foreleg():
    """A column from inside the shoulder to the floor, thick the whole way:
    the deltoid capping it, the triceps a mass behind, the elbow's point,
    the forearm's muscle down its outside; a pad that spreads at the floor."""
    rows = [
        (0.0, 0.88, 0.88, 0.96),
        (0.10, 0.84, 0.84, 1.00),
        (0.30, 0.66, 0.62, 0.80),
        (0.50, 0.50, 0.48, 0.56),
        (0.60, 0.50, 0.50, 0.48),
        (0.80, 0.46, 0.45, 0.45),
        (0.91, 0.45, 0.44, 0.44),
        (1.0, 0.64, 0.68, 0.58),
    ]
    muscles = [
        (OUTSIDE + 0.6, 0.55, [0.12, 0.20, 0.08, 0.0, 0.0, 0.0, 0.0, 0.0]),
        (BACK, 0.65, [0.04, 0.12, 0.22, 0.05, 0.0, 0.0, 0.0, 0.0]),
        (BACK, 0.30, [0.0, 0.0, 0.0, 0.24, 0.05, 0.0, 0.0, 0.0]),
        (OUTSIDE + 0.7, 0.55, [0.0, 0.0, 0.0, 0.08, 0.18, 0.06, 0.0, 0.0]),
        (FRONT, 0.5, [0.0, 0.0, 0.08, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ]
    return [b.tube(FORE, muscled(rows, muscles), ref=(1, 0, 0), sides=48)]


def hindleg():
    rows = [
        (0.0, 0.84, 0.88, 1.00),
        (0.12, 0.80, 0.82, 1.00),
        (0.32, 0.64, 0.66, 0.78),
        (0.50, 0.48, 0.50, 0.48),
        (0.70, 0.44, 0.40, 0.52),
        (0.87, 0.42, 0.40, 0.42),
        (1.0, 0.60, 0.64, 0.54),
    ]
    muscles = [
        (BACK + 0.5, 0.6, [0.12, 0.20, 0.10, 0.0, 0.0, 0.0, 0.0]),
        (BACK - 0.3, 0.55, [0.0, 0.10, 0.20, 0.05, 0.0, 0.0, 0.0]),
        (FRONT - 0.3, 0.6, [0.0, 0.08, 0.18, 0.05, 0.0, 0.0, 0.0]),
        (FRONT, 0.35, [0.0, 0.0, 0.0, 0.16, 0.0, 0.0, 0.0]),
        (BACK, 0.55, [0.0, 0.0, 0.0, 0.0, 0.22, 0.05, 0.0]),
        (BACK, 0.3, [0.0, 0.0, 0.0, 0.0, 0.0, 0.18, 0.0]),
    ]
    return [b.tube(HIND, muscled(rows, muscles), ref=(1, 0, 0), sides=48)]


def toes():
    """Three great toes round the front of each pad, each a mass the claw
    grows from."""
    out = []
    for (fx, fz, r) in ((1.18, -1.0, 0.64), (-1.76, -0.97, 0.60)):
        for a in np.radians([-48, 0, 48]):
            d = np.array([np.cos(a), 0.0, np.sin(a)])
            out.append(b.blob(np.array([fx, 0.26, fz]) + d * (r - 0.12), (0.24, 0.24, 0.21)))
    return out


def crests():
    """The soft places, as flesh standing out of the skin: the ridge along
    the spine between the carapace's halves, and the nape between the
    collar plates. Their ribs are sculpted in the detail pass."""
    ridge = b.tube([(-0.95, 5.50, 0), (0.15, 5.66, 0), (1.35, 5.92, 0)], [
        st(0.0, 0.10, 0.10, 0.10),
        st(0.12, 0.34, 0.34, 0.30, nt=2.6),
        st(0.5, 0.38, 0.42, 0.30, nt=2.6),
        st(0.88, 0.34, 0.36, 0.30, nt=2.6),
        st(1.0, 0.10, 0.12, 0.10),
    ], sides=40)
    nape = b.tube([(2.30, 5.80, 0), (2.75, 5.92, 0), (3.30, 5.86, 0)], [
        st(0.0, 0.08, 0.08, 0.08),
        st(0.2, 0.26, 0.24, 0.24),
        st(0.8, 0.24, 0.22, 0.24),
        st(1.0, 0.08, 0.08, 0.08),
    ], sides=32)
    return [ridge, nape]


def club():
    """The end of the tail swells into the armoured knob the quills fan
    from: what the spray is thrown by."""
    return [b.blob((-7.25, 3.60, 0), (0.62, 0.40, 0.44)), b.blob((-6.85, 3.78, 0), (0.45, 0.34, 0.36))]


def soft_forms():
    left = foreleg() + hindleg()
    return [torso(), neck(), tail()] + head() + crests() + toes() + club() + left + [b.mirror(f) for f in left]


# ---------------------------------------------------------------------------
# 2. Where the armour is, and what it is made of
# ---------------------------------------------------------------------------


class Axis:
    """A curve, sampled densely, with frames: for laying things round it."""

    def __init__(self, path, ref=(0, 1, 0), n=400):
        self.c = b.spline(path, n)
        self.tg, self.side, self.up = b.frames(self.c, ref)
        self.n = n

    def coords(self, p, chunk=200000):
        """For each point: how far along (0 to 1), the angle round from
        ``up`` toward ``side``, and the distance off the curve."""
        s = np.zeros(len(p))
        phi = np.zeros(len(p))
        r = np.zeros(len(p))
        for a in range(0, len(p), chunk):
            q = p[a:a + chunk]
            d2 = ((q[:, None, :] - self.c[None, ::4, :]) ** 2).sum(-1)
            i = np.argmin(d2, axis=1) * 4
            # refine among the neighbours
            best = i.copy()
            bd = np.full(len(q), np.inf)
            for k in range(-4, 5):
                j = np.clip(i + k, 0, self.n - 1)
                dd = ((q - self.c[j]) ** 2).sum(-1)
                better = dd < bd
                best[better], bd[better] = j[better], dd[better]
            d = q - self.c[best]
            s[a:a + chunk] = best / (self.n - 1)
            phi[a:a + chunk] = np.arctan2((d * self.side[best]).sum(-1), (d * self.up[best]).sum(-1))
            r[a:a + chunk] = np.sqrt(bd)
        return s, phi, r


def smooth(a, c, x):
    t = np.clip((np.asarray(x, float) - a) / (c - a), 0, 1)
    return t * t * (3 - 2 * t)


class Plan:
    """Which part of the body a point is, and how far inside the armour
    there: one region per kind of plate, each with its scute size, its
    thickness and its own edges. A point's ``armour`` is metres inside its
    region's edge (negative outside); its ``region`` names it."""

    REGIONS = ["back", "neck", "helm", "tail", "fore_shin", "hind_shin", "cheek"]
    # scute frequency (per metre), thickness, dome
    STYLE = {
        "back": (0.95, 0.20, 0.02),
        "neck": (1.5, 0.12, 0.015),
        "helm": (1.5, 0.10, 0.015),
        "tail": (1.8, 0.11, 0.015),
        "fore_shin": (2.6, 0.09, 0.01),
        "hind_shin": (2.6, 0.09, 0.01),
        "cheek": (2.4, 0.06, 0.01),
    }

    def __init__(self):
        self.torso = Axis([(TORSO_X[0], Y0, 0), (TORSO_X[1], Y0, 0)])
        self.neck = Axis(NECK)
        self.head = Axis(HEAD)
        self.tail = Axis(TAIL)
        self.fore = Axis(FORE[1:], ref=(1, 0, 0))
        self.hind = Axis(HIND[1:], ref=(1, 0, 0))

    def classify(self, p):
        """``region`` (index into REGIONS, -1 for bare hide) and
        ``armour`` (metres inside that region's edge) for every point."""
        n = len(p)
        region = np.full(n, -1)
        inside = np.full(n, -1.0)
        x, y, z = p[:, 0], p[:, 1], p[:, 2]

        def claim(mask, name, depth):
            k = self.REGIONS.index(name)
            take = mask & (depth > inside)
            region[take] = k
            inside[take] = depth[take]

        # The back: a carapace from the withers to the root of the tail,
        # reaching down over the shoulders as a mantle and over the haunch,
        # parted down the middle where the ridge stands.
        s, phi, r = self.torso.coords(p)
        a = np.abs(phi)
        reach = 1.20 + 0.62 * smooth(0.3, 1.3, x) * smooth(2.4, 1.6, x) + 0.28 * smooth(-1.2, -2.0, x) \
            * smooth(-3.2, -2.5, x)
        trench = 0.30 * smooth(-1.25, -0.85, x) * smooth(1.75, 1.35, x)
        lo, hi = -3.15, 2.30
        d_phi = (reach - a) * np.maximum(r, 0.5)
        d_x = np.minimum(x - lo, hi - x)
        d_trench = np.abs(z) - trench
        depth = np.minimum.reduce([d_phi, d_x, np.where(trench > 0.01, d_trench, 9.0)])
        claim((x > lo - 0.3) & (x < hi + 0.3) & (y > 3.2) & (a < reach + 0.5), "back", depth)

        # The neck: collars either side of the nape, open on top.
        s, phi, r = self.neck.coords(p)
        a = np.abs(phi)
        near = (r < 1.4) & (x > 1.95) & (x < 4.15)
        depth = np.minimum.reduce([(a - 0.42) * r, (1.75 - a) * r, (s - 0.12) * 2.4, (0.96 - s) * 2.4])
        claim(near, "neck", depth)

        # The helm: over the skull from the brow back, and down its cheeks.
        s, phi, r = self.head.coords(p)
        a = np.abs(phi)
        near = (r < 1.0) & (x > 3.85)
        depth = np.minimum.reduce([(1.15 - a) * r, (s - 0.04) * 1.9, (0.70 - s) * 1.9])
        claim(near, "helm", depth)
        cheek = np.minimum.reduce([(a - 1.35) * r, (2.05 - a) * r, (s - 0.10) * 1.9, (0.42 - s) * 1.9])
        claim(near & (y > 4.95), "cheek", cheek)

        # The tail: plates over its top and flanks, to near its tip.
        s, phi, r = self.tail.coords(p)
        a = np.abs(phi)
        near = (r < 1.1) & (x < -2.7)
        depth = np.minimum.reduce([(1.55 - a) * np.maximum(r, 0.2), (s - 0.02) * 5.0, (1.02 - s) * 5.0])
        claim(near, "tail", depth)

        # The shins: plates down the front and outside of each lower leg.
        for name, axis, lo_y, hi_y in (("fore_shin", self.fore, 0.55, 2.15), ("hind_shin", self.hind, 0.62, 2.05)):
            for side in (-1, 1):
                q = p * np.array([1, 1, side * -1.0])  # the right leg seen as the left
                s, phi, r = axis.coords(q)
                near = (r < 0.9) & (q[:, 2] < -0.3)
                ph = phi  # 0 at the front, negative toward the outside
                depth = np.minimum.reduce([(1.25 - np.abs(ph + 0.35)) * r, y - lo_y, hi_y - y])
                claim(near & (np.sign(z) == side), name, depth)
        return region, inside

    AXES = {"back": "torso", "neck": "neck", "helm": "head", "cheek": "head", "tail": "tail",
            "fore_shin": "fore", "hind_shin": "hind"}

    def scutes(self, p, region):
        """For each point: its scute's id, the distance to the nearest groove
        between scutes (metres) and how near its scute's middle it is (0 at
        the middle, about 1 at the edge). Scutes are laid out on the body's
        own coordinates -- along it and round it -- with little jitter, so
        they fall in rows the way a tortoise's and an ankylosaur's do,
        largest along the spine, and the two sides mirror."""
        cid = np.zeros(len(p), np.int64)
        groove = np.full(len(p), 9.0)
        middle = np.ones(len(p))
        lean = np.zeros(len(p))
        across = np.ones(len(p))
        for k, name in enumerate(self.REGIONS):
            m = region == k
            if not m.any():
                continue
            freq = self.STYLE[name][0]
            axis = getattr(self, self.AXES[name])
            q0 = p[m].copy()
            if name in ("fore_shin", "hind_shin"):
                q0[:, 2] = -np.abs(q0[:, 2])
            sv, phi, r = axis.coords(q0)
            length = np.linalg.norm(np.diff(axis.c, axis=0), axis=1).sum()
            # Round the body in metres, measured at a typical radius, and
            # growing from the spine out so the edge rows are smaller.
            rad = {"back": 1.35, "neck": 0.8, "helm": 0.55, "cheek": 0.55, "tail": 0.45,
                   "fore_shin": 0.5, "hind_shin": 0.48}[name]
            a = np.abs(phi) * rad
            a = a * (1.0 + 0.35 * np.clip(a / rad - 0.4, 0, 2))
            if name in ("fore_shin", "hind_shin"):
                # Bands round the leg, as an armadillo's: one scute wraps.
                a = a * 0.18
            q = np.stack([sv * length * 0.62, a, np.zeros(len(a))], 1)
            f1, f2, c, ctr = b.cellular(q, freq, seed=31 + k * 7, jitter=0.45, centres=True)
            cid[m] = c + k * 1000003
            groove[m] = (f2 - f1) / freq * 0.5
            middle[m] = np.clip(f1 / 0.62, 0, 1.4)
            # Behind its scute's middle (positive) or ahead of it, in metres
            # along the body: the tail's and the legs' run the other way.
            sign = -1.0 if name in ("tail", "fore_shin", "hind_shin") else 1.0
            lean[m] = sign * (ctr[:, 0] - q[:, 0]) / 0.62
            across[m] = np.abs(ctr[:, 1] - q[:, 1])
        return cid, groove, middle, lean, across


def subdivide(v, t):
    """Every triangle into four, at its edges' midpoints."""
    e = np.sort(np.concatenate([t[:, [0, 1]], t[:, [1, 2]], t[:, [2, 0]]]), axis=1)
    uniq, inv = np.unique(e, axis=0, return_inverse=True)
    inv = inv.ravel()
    mid = (v[uniq[:, 0]] + v[uniq[:, 1]]) / 2
    nv = len(v)
    m = inv.reshape(3, -1).T + nv  # (tri, [01, 12, 20])
    a, b_, c = t[:, 0], t[:, 1], t[:, 2]
    m01, m12, m20 = m[:, 0], m[:, 1], m[:, 2]
    nt = np.concatenate([
        np.stack([a, m01, m20], 1), np.stack([m01, b_, m12], 1),
        np.stack([m20, m12, c], 1), np.stack([m01, m12, m20], 1)])
    return np.concatenate([v, mid]), nt


def carapace(v, t, plan):
    """The armour as closed shells grown off the coarse skin: wherever the
    plan puts armour, the skin is lifted by its region's thickness -- full
    over a scute, sunk to a quarter in the grooves between them, domed at
    each scute's middle, its rim a chunky step -- and closed underneath, so
    the voxel union reads it as one solid with the skin."""
    n = b.normals(v, t)
    region, inside = plan.classify(v)
    keep = (region >= 0) & (inside > -0.06)
    faces = t[keep[t].all(1)]
    used = np.unique(faces)
    remap = -np.ones(len(v), np.int64)
    remap[used] = np.arange(len(used))
    pv, pn, pt = v[used], n[used], remap[faces]
    say(f"carapace patch {len(pv)} verts; subdividing")
    for _ in range(2):
        k = len(pv)
        pv, pt = subdivide(pv, pt)
        e = np.sort(np.concatenate([pt[:, [0, 1]], pt[:, [1, 2]], pt[:, [2, 0]]]), axis=1)
        e = np.unique(e, axis=0)
        # new normals: mean of the edge's ends (they were made in order)
        pn = np.concatenate([pn, np.zeros((len(pv) - k, 3))])
        acc = np.zeros_like(pn)
        cnt = np.zeros(len(pv))
        old = e[(e[:, 0] < k) & (e[:, 1] >= k)]
        np.add.at(acc, old[:, 1], pn[old[:, 0]])
        np.add.at(cnt, old[:, 1], 1)
        pn[k:] = acc[k:] / np.maximum(cnt[k:, None], 1)
        pn /= np.maximum(np.linalg.norm(pn, axis=1, keepdims=True), 1e-9)
    region, inside = plan.classify(pv)
    ok = (region >= 0) & (inside > 0)
    faces = pt[ok[pt].all(1)]
    used = np.unique(faces)
    remap = -np.ones(len(pv), np.int64)
    remap[used] = np.arange(len(used))
    pv, pn, pt, region, inside = pv[used], pn[used], remap[faces], region[used], inside[used]
    say(f"carapace {len(pv)} verts")
    cid, groove, middle, lean, across = plan.scutes(pv, region)
    thick = np.array([Plan.STYLE[r][1] for r in Plan.REGIONS])[region]
    dome = np.array([Plan.STYLE[r][2] for r in Plan.REGIONS])[region]
    # Each scute a little its own: thicker or thinner.
    own = b.cell_rand(cid, 3)
    h = thick * (0.85 + 0.3 * own)
    # Shingled: each scute rises toward its back edge, which lies over the
    # front of the one behind -- a pangolin's, a pine cone's.
    h = h + thick * 0.55 * np.clip(lean / 0.45, -0.45, 1.0)
    # A keel down its middle, along the body.
    h = h + thick * 0.22 * np.exp(-(across / 0.10) ** 2) * smooth(0.02, 0.08, groove)
    # The groove: a crisp cut channel between scutes, chamfered either side.
    h = h * (0.15 + 0.85 * smooth(0.006, 0.028, groove))
    h = h + dome * (1 - np.clip(middle, 0, 1) ** 2) * smooth(0.01, 0.06, groove)
    # The rim: chunky, a slight chamfer over its last few centimetres.
    h = h * (0.72 + 0.28 * smooth(0.0, 0.05, inside))
    top = pv + pn * h[:, None]
    bottom = pv - pn * 0.08
    k = len(pv)
    verts = np.concatenate([top, bottom])
    tris = [pt, pt[:, ::-1] + k]
    for a, c in b._boundary_edges(pt):
        tris.append(np.array([[a, c, c + k], [a, c + k, a + k]]))
    tris = np.concatenate(tris)
    return verts, tris


# ---------------------------------------------------------------------------
# 3. Hard parts: horns, spikes, claws, teeth
# ---------------------------------------------------------------------------


def ringed_horn(path, radii, sides=None, rings=None, ridges=14, depth=0.05, flat=1.0, twist=0.0, ref=None):
    """A horn with growth ridges: a tube whose radius runs through ``radii``
    with ``ridges`` bands round it, each a little proud of the next.
    ``flat`` squashes its section along ``ref`` (by default the world axis
    least along it, so a horn that points straight up still has a frame)."""
    path = np.asarray(path, float)
    length = np.linalg.norm(np.diff(path, axis=0), axis=1).sum()
    rings = rings or int(np.clip(length / 0.045, 10, 48))
    sides = sides or int(np.clip(max(radii) / 0.012, 8, 18))
    curve = b.spline(path, rings)
    if ref is None:
        mean = curve[-1] - curve[0]
        mean = mean / max(np.linalg.norm(mean), 1e-9)
        axes = np.eye(3)
        ref = axes[np.argmin(np.abs(axes @ mean))]
        if abs(mean[1]) < 0.7:
            ref = np.array([0.0, 1.0, 0.0])
    tg, side, up = b.frames(curve, ref)
    m = len(curve)
    s = np.linspace(0, 1, m)
    rr = np.interp(s, np.linspace(0, 1, len(radii)), radii)
    band = 1 + depth * (np.abs(np.sin(s * np.pi * ridges)) ** 0.5 - 0.5) * smooth(0.0, 0.1, s) * smooth(1.0, 0.75, s)
    theta = np.linspace(0, 2 * np.pi, sides, endpoint=False)
    verts = []
    for i in range(m):
        th = theta + twist * s[i]
        ring = curve[i] + rr[i] * band[i] * (np.outer(np.cos(th), up[i]) * flat + np.outer(np.sin(th), side[i]))
        verts.append(ring)
    verts = np.concatenate(verts)
    faces = []
    for i in range(m - 1):
        a, c = i * sides, (i + 1) * sides
        for j in range(sides):
            k = (j + 1) % sides
            faces.append([a + j, c + j, c + k, a + k])
    c0 = len(verts)
    verts = np.vstack([verts, curve[0], curve[-1] + tg[-1] * 0.001])
    last = (m - 1) * sides
    for j in range(sides):
        k = (j + 1) % sides
        faces.append([c0, j, k])
        faces.append([c0 + 1, last + k, last + j])
    return verts, faces


def hard_parts(surf):
    """Every hard piece, with what it is (for its paint): ``(form, kind)``."""
    out = []
    # The horns: from behind the brow, sweeping back and out, then curling
    # forward and up -- the head's frame.
    for s in (-1, 1):
        out.append((ringed_horn([(4.35, 5.55, s * 0.40), (4.05, 5.95, s * 0.66), (3.80, 6.40, s * 0.88),
                                 (3.95, 6.85, s * 1.02), (4.35, 7.08, s * 1.00), (4.72, 7.02, s * 0.86)],
                                [0.25, 0.22, 0.18, 0.13, 0.08, 0.0], ridges=18, depth=0.07, flat=0.85), "horn"))
        # A second, smaller pair below, swept straight back over the neck.
        out.append((ringed_horn([(4.15, 5.32, s * 0.52), (3.75, 5.48, s * 0.70), (3.35, 5.70, s * 0.80)],
                                [0.12, 0.08, 0.0], ridges=8, depth=0.05), "horn"))
        # Tusks out of the jaw, up past the beak.
        out.append((ringed_horn([(5.30, 4.85, s * 0.26), (5.45, 5.05, s * 0.32), (5.52, 5.28, s * 0.30)],
                                [0.075, 0.05, 0.0], ridges=4, depth=0.03, sides=14), "tooth"))
        # A row of blunt teeth along each side of the mouth.
        for k, x in enumerate(np.linspace(4.55, 5.35, 6)):
            zz = s * (0.36 - 0.12 * (x - 4.55) / 0.8)
            out.append((ringed_horn([(x, 4.98, zz), (x + 0.02, 5.08, zz * 1.02)], [0.045, 0.0], ridges=1,
                                    depth=0.0, sides=10, rings=8), "tooth"))
    # A row of horn nubs along each brow, over the eye: what makes the face
    # read from across the arena.
    for s in (-1, 1):
        for k, (x, y, zz, L) in enumerate(((4.98, 5.62, 0.40, 0.18), (4.80, 5.70, 0.46, 0.24),
                                           (4.60, 5.74, 0.50, 0.28))):
            base = np.array([x, y, s * zz])
            d = np.array([-0.35, 0.75, s * 0.55])
            d /= np.linalg.norm(d)
            out.append((ringed_horn([base - d * 0.06, base + d * L * 0.6, base + d * L], [0.07, 0.04, 0.0],
                                    ridges=3, depth=0.04), "horn"))
    # The nose horn: a thick boss, one short blunt horn.
    out.append((ringed_horn([(5.42, 5.25, 0), (5.62, 5.55, 0), (5.70, 5.80, 0)], [0.17, 0.11, 0.0],
                            ridges=6, depth=0.06), "horn"))
    # The spines either side of the ridge: few, thick, curving back, tallest
    # over the shoulders -- the silhouette's crown.
    for k, (x, h) in enumerate(((1.55, 1.05), (1.0, 1.30), (0.35, 1.18), (-0.35, 0.95), (-1.05, 0.78),
                                (-1.75, 0.62), (-2.45, 0.48))):
        for s in (-1, 1):
            hit = surf.down(x, s * 0.60)
            if hit is None:
                continue
            p, n = hit
            u = np.array([-0.45, 1.0, s * 0.28])
            u /= np.linalg.norm(u)
            tip = p + u * h + np.array([-0.30 * h, -0.05 * h, 0])
            out.append((ringed_horn([p - n * 0.25, p + u * h * 0.45, tip],
                                    [0.20 * h + 0.06, 0.12 * h + 0.03, 0.0], ridges=6, depth=0.05,
                                    flat=1.5, ref=np.array([1.0, 0.0, 0.0])), "spine"))
    # Over the ridge itself, the bones of the spine break the surface:
    # blades standing out of the glowing flesh, raked back.
    for x in np.arange(-0.72, 1.32, 0.34):
        hit = surf.down(x, 0.0)
        if hit is None:
            continue
        p, n = hit
        h = 0.42 + 0.12 * np.cos((x - 0.3) * 1.6)
        u = np.array([-0.35, 1.0, 0.0])
        u /= np.linalg.norm(u)
        out.append((ringed_horn([p - n * 0.2, p + u * h * 0.5, p + u * h - np.array([0.12, 0, 0])],
                                [0.075, 0.05, 0.0], ridges=2, depth=0.03, flat=2.6,
                                ref=np.array([1.0, 0.0, 0.0])), "bone"))
    # The mantle's edge: spikes flaring out over each shoulder, a collar.
    torso = Axis([(TORSO_X[0], Y0, 0), (TORSO_X[1], Y0, 0)])
    for x, ang, L in ((0.55, 1.62, 0.55), (0.95, 1.78, 0.78), (1.35, 1.80, 0.86), (1.75, 1.66, 0.66)):
        for s in (-1, 1):
            i = int((x - TORSO_X[0]) / (TORSO_X[1] - TORSO_X[0]) * (torso.n - 1))
            c = torso.c[i]
            d = torso.up[i] * np.cos(ang) + torso.side[i] * np.sin(ang) * s
            hit = surf.toward(c + d * 1.6, c)
            if hit is None:
                continue
            p, n = hit
            u = n * 1.0 + np.array([-0.55, 0.25, 0])
            u /= np.linalg.norm(u)
            out.append((ringed_horn([p - n * 0.15, p + u * L * 0.5, p + u * L + np.array([-0.12, -0.06, 0])],
                                    [0.15, 0.09, 0.0], ridges=5, depth=0.05), "spine"))
    # Down the tail: the rows carry on, shrinking to the club.
    tail_axis = Axis(TAIL)
    for k, sv in enumerate(np.linspace(0.05, 0.78, 7)):
        i = int(sv * (tail_axis.n - 1))
        c, tg, up = tail_axis.c[i], tail_axis.tg[i], tail_axis.up[i]
        size = 1.0 - 0.6 * sv
        for s in (-1, 1):
            hit = surf.toward(c + up * 0.8 + np.array([0, 0, s * 0.30 * size]), c)
            if hit is None:
                continue
            p, n = hit
            u = up + np.array([0, 0, s * 0.45]) - tg * 0.5
            u /= np.linalg.norm(u)
            h = 0.62 * size + 0.1
            out.append((ringed_horn([p - n * 0.12, p + u * h * 0.5, p + u * h - tg * 0.15 * h],
                                    [0.13 * size + 0.04, 0.07 * size + 0.02, 0.0], ridges=4, depth=0.05), "spine"))
    # The quills it sprays: fanned out of the club, roots glowing.
    club_c = np.array([-7.25, 3.60, 0.0])
    rng = np.random.default_rng(7)
    for k in range(22):
        az = rng.uniform(-np.pi, np.pi)
        el = rng.uniform(-0.25, 1.1)
        d = np.array([-np.cos(el) * 0.6 + 0.0, np.sin(el), 0.0]) + np.array([0, 0, np.sin(az) * 0.9])
        d[0] -= 0.4
        d /= np.linalg.norm(d)
        hit = surf.toward(club_c + d * 1.2, club_c)
        if hit is None:
            continue
        p, n = hit
        L = rng.uniform(0.45, 0.85)
        dd = n * 0.8 + d * 0.5 + np.array([-0.3, 0.1, 0])
        dd /= np.linalg.norm(dd)
        out.append((ringed_horn([p - dd * 0.08, p + dd * L * 0.5, p + dd * L], [0.06, 0.035, 0.0], ridges=3,
                                depth=0.04), "quill"))
    # A spur off the back of each elbow.
    for s in (-1, 1):
        out.append((ringed_horn([(0.80, 2.45, s * 1.0), (0.45, 2.65, s * 1.08), (0.18, 2.95, s * 1.10)],
                                [0.11, 0.07, 0.0], ridges=5, depth=0.05, sides=14), "spine"))
    # Claws: one over each toe, thick and hooked down.
    for (fx, fz, r) in ((1.18, -1.0, 0.64), (-1.76, -0.97, 0.60)):
        for s in (-1, 1):
            zc = fz if s == -1 else -fz
            for a in np.radians([-48, 0, 48]):
                d = np.array([np.cos(a), 0.0, np.sin(a) * (1 if s == -1 else -1)])
                base = np.array([fx, 0.30, zc]) + d * (r + 0.02)
                out.append((ringed_horn([base - d * 0.12, base + d * 0.12 + np.array([0, -0.08, 0]),
                                         base + d * 0.24 + np.array([0, -0.26, 0])],
                                        [0.15, 0.11, 0.0], ridges=4, depth=0.04, sides=16, flat=0.75), "claw"))
    return out


# ---------------------------------------------------------------------------
# 4. Detail, colour and glow on the sculpt
# ---------------------------------------------------------------------------

# The palette, as the eye sees it (sRGB).
# Warmer than it looks right in a studio: the game's sky light is blue, and
# a grey hide under it reads as cold stone (judged lit, in the game).
HIDE_DARK = np.array([0.24, 0.18, 0.14])
HIDE = np.array([0.56, 0.44, 0.33])
HIDE_STRIPE = np.array([0.33, 0.24, 0.18])
HIDE_DUST = np.array([0.76, 0.64, 0.50])
BELLY = np.array([0.84, 0.70, 0.52])
MUD = np.array([0.62, 0.48, 0.34])
STONE = np.array([0.93, 0.86, 0.72])
STONE_DARK = np.array([0.70, 0.62, 0.50])
STONE_WARM = np.array([0.92, 0.78, 0.58])
GROOVE = np.array([0.08, 0.07, 0.06])
MOSS = np.array([0.33, 0.40, 0.16])
LICHEN = np.array([0.78, 0.62, 0.30])
CHAR = np.array([0.13, 0.08, 0.06])
EMBER = np.array([1.0, 0.42, 0.12])
EMBER_DEEP = np.array([0.55, 0.08, 0.03])
HORN_ROOT = np.array([0.24, 0.20, 0.16])
HORN_TIP = np.array([0.96, 0.93, 0.84])
EYE = np.array([1.0, 0.80, 0.30])


def mixc(a, c, f):
    f = np.clip(np.asarray(f, float), 0, 1)[:, None]
    return np.asarray(a, float) * (1 - f) + np.asarray(c, float) * f


def soft_regions(p):
    """How much each point is the ridge's flesh or the nape's, 0 to 1."""
    x, y, z = p[:, 0], p[:, 1], p[:, 2]
    ridge = smooth(0.32, 0.18, np.abs(z)) * smooth(-1.05, -0.8, x) * smooth(1.45, 1.2, x) * smooth(5.45, 5.75, y + 0.18 * x)
    nape = smooth(0.26, 0.14, np.abs(z)) * smooth(2.25, 2.4, x) * smooth(3.4, 3.25, x) * smooth(5.62, 5.82, y)
    return ridge, nape


def sculpt(v, t, kind, plan):
    """The detail pass on the dense union, and its paint: returns the moved
    vertices and per-vertex albedo, glow and roughness."""
    n = b.normals(v, t)
    x, y, z = v[:, 0], v[:, 1], v[:, 2]
    stone = smooth(0.35, 0.65, kind)
    ridge, nape = soft_regions(v)
    soft = np.maximum(ridge, nape)
    skin = (1 - stone) * (1 - soft)
    region, inside = plan.classify(v)
    region = np.where(stone > 0.5, np.maximum(region, 0), region)
    cid, groove, middle, lean, across = plan.scutes(v, np.where(stone > 0.5, region, -1))
    say("sculpt: noise")

    # -- The hide: a crazed network of creases, folds at the joints, and
    # pebbled scale down the legs and over the feet.
    f1, f2, _ = b.cellular(v, 9.0, seed=5)
    crease = 1 - smooth(0.0, 0.10, f2 - f1)
    g1, g2, _ = b.cellular(v * np.array([1.0, 2.2, 1.0]), 22.0, seed=9)
    fine = 1 - smooth(0.0, 0.12, g2 - g1)
    joints = np.zeros(len(v))
    folds = np.zeros(len(v))
    for (yj, w) in ((0.55, 0.22), (2.25, 0.28), (0.72, 0.24), (2.15, 0.26)):
        m = np.exp(-((y - yj) / w) ** 2) * smooth(3.2, 2.6, y)
        joints = np.maximum(joints, m)
    wobble = b.fbm(v, 1.5, 2, seed=4)
    folds = (np.maximum(0, np.sin(y * 2 * np.pi / 0.085 + wobble * 6)) ** 3) * joints
    # The neck's folds run round it.
    neck_s, neck_phi, neck_r = plan.neck.coords(v)
    neck_m = (neck_r < 1.3) & (x > 2.0) & (x < 4.1)
    folds += np.where(neck_m, np.maximum(0, np.sin(neck_s * 2 * np.pi * 9 + wobble * 5)) ** 3 * 0.8, 0)
    scale_m = smooth(2.4, 1.6, y) * (1 - joints * 0.6)
    s1, s2, _ = b.cellular(v, 30.0, seed=13)
    pebble = (1 - np.clip(s1 / 0.55, 0, 1) ** 2) * scale_m
    lump = b.fbm(v, 3.0, 3, seed=6) - 0.5
    # The belly: broad transverse scales from the chest to between the
    # hind legs, a crocodile's underside.
    belly_m = smooth(-0.35, -0.75, n[:, 1]) * smooth(-2.4, -1.9, x) * smooth(2.0, 1.5, x) * smooth(4.2, 3.7, y)
    band = np.abs(((x + 0.03 * np.sin(z * 4)) / 0.16) % 1.0 - 0.5) * 2
    seam_b = 1 - smooth(0.0, 0.18, band)
    d_skin = -0.008 * crease * (1 - belly_m) - 0.0035 * fine - 0.016 * folds + 0.0045 * pebble + 0.010 * lump \
        - 0.012 * seam_b * belly_m + 0.006 * belly_m * (1 - band)

    # -- The stone: lumpy, eroded, ringed with growth, pitted, cracked, and
    # chipped where it is thin.
    erode = b.fbm(v, 2.6, 5, seed=21) - 0.5
    grit = b.fbm(v, 14.0, 3, seed=22) - 0.5
    rings = np.sin(np.clip(middle, 0, 1.2) * 2 * np.pi * 5.5) * smooth(0.25, 0.85, middle)
    p1, p2, _ = b.cellular(v, 26.0, seed=23)
    pits = smooth(0.30, 0.0, p1) * smooth(0.45, 0.6, b.fbm(v, 4.0, 2, seed=24))
    c1, c2, _ = b.cellular(v, 3.2, seed=25)
    crack = (1 - smooth(0.0, 0.025, c2 - c1)) * smooth(0.52, 0.62, b.fbm(v, 1.8, 2, seed=26))
    chip = smooth(0.035, 0.01, groove) * smooth(0.55, 0.7, b.fbm(v, 9.0, 2, seed=27))
    d_stone = 0.016 * erode + 0.006 * grit - 0.007 * pits - 0.020 * crack - 0.018 * chip

    # -- The soft places: a crust over heat, split along a network of
    # fissures, as cooling lava is -- the light is in the cracks.
    r1, r2, _ = b.cellular(v, 5.5, seed=31)
    lava = 1 - smooth(0.0, 0.06, r2 - r1)
    l1, l2, _ = b.cellular(v, 16.0, seed=32)
    vein = 1 - smooth(0.0, 0.08, l2 - l1)
    d_soft = -0.028 * lava - 0.007 * vein + 0.014 * (b.fbm(v, 4.0, 3, seed=33) - 0.5)

    # -- The face: nostrils, eyes sunk under the brow, the line of the mouth.
    face = np.zeros(len(v))
    for zs in (-1, 1):
        face -= 0.055 * np.exp(-(((x - 5.62) / 0.06) ** 2 + ((y - 5.12) / 0.045) ** 2 + ((z - zs * 0.15) / 0.05) ** 2))
        face -= 0.045 * np.exp(-(((x - 4.86) / 0.13) ** 2 + ((y - 5.40) / 0.10) ** 2 + ((np.abs(z) - 0.50) / 0.12) ** 2))
    mouth_y = np.interp(x, [4.15, 4.6, 5.7], [5.02, 4.98, 4.82])
    mouth = np.exp(-((y - mouth_y) / 0.022) ** 2) * smooth(4.25, 4.45, x) * smooth(0.12, 0.25, np.abs(z)) \
        * (x < 5.85)
    face -= 0.03 * mouth
    beak = smooth(5.25, 5.45, x) * smooth(4.86, 4.96, y) * (x < 6.2)

    d = skin * (d_skin * (1 - beak) + 0.002 * beak * np.sin(z * 120)) + stone * d_stone + soft * d_soft + face
    v = v + n * d[:, None]
    say("sculpt: paint")

    # -- Paint.
    up = n[:, 1]
    # Hide: dark, dusty on what faces the sky and on the tops of its
    # creases, paler beneath, mud up the legs.
    c = HIDE[None, :] * np.ones((len(v), 1))
    # Stripes down the flanks and legs, broken by noise: a pattern, as
    # every large animal's hide has, so the dark is not one dark.
    stripe = smooth(0.15, 0.55, np.sin(x * 2 * np.pi / 0.9 + 2.2 * b.fbm(v, 0.9, 3, seed=45)) * 0.5 + 0.5) \
        * smooth(0.6, 0.1, np.abs(up))
    c = mixc(c, HIDE_STRIPE, stripe * 0.6)
    c = mixc(c, BELLY, smooth(-0.2, -0.8, up) * smooth(4.6, 3.8, y))
    c = mixc(c, BELLY * 1.12, belly_m * (1 - band) * 0.6)
    c = mixc(c, HIDE_DARK, belly_m * seam_b * 0.8)
    c = mixc(c, HIDE_DUST, smooth(0.3, 0.9, up) * 0.45 + (1 - crease) * 0.10)
    c = mixc(c, HIDE_DARK, crease * 0.75 + folds * 0.5 + fine * 0.25)
    c = mixc(c, MUD, smooth(1.4, 0.15, y) * 0.8)
    c = c * (0.85 + 0.3 * b.fbm(v, 1.2, 3, seed=40))[:, None]
    # The beak: dark, smooth horn, paler at its cutting edge.
    c = mixc(c, mixc(np.array([0.20, 0.17, 0.14]), np.array([0.55, 0.50, 0.42]), smooth(5.0, 4.86, y) * 0.8),
             beak)
    c = mixc(c, HIDE_DARK * 0.4, mouth * 0.9)
    hide_c = c
    # Stone: each scute its own tint of weathered bone-grey, dark in its
    # grooves, cracks and pits, pale and worn on its edges and its rises,
    # moss and lichen on what faces the sky, charred near the ridge.
    own = b.cell_rand(cid, 41)
    c = mixc(STONE, STONE_WARM, own * 0.6)
    c = mixc(c, STONE_DARK, (0.5 - erode).clip(0, 1) * 0.35 + smooth(0.5, -0.3, up) * 0.45)
    c = c * (0.93 + 0.12 * b.fbm(v, 6.0, 2, seed=42))[:, None]
    c = mixc(c, np.array([0.94, 0.91, 0.84]), np.clip(erode * 2.2, 0, 1) * 0.35 + chip * 0.6)
    mossy = smooth(0.3, 0.8, up) * smooth(0.48, 0.62, b.fbm(v, 1.6, 4, seed=43)) * smooth(0.02, 0.06, groove)
    c = mixc(c, MOSS, mossy * 0.85)
    c = mixc(c, LICHEN, smooth(0.72, 0.8, b.fbm(v, 7.0, 2, seed=44)) * smooth(0.0, 0.5, up) * 0.7)
    c = mixc(c, GROOVE, (1 - smooth(0.006, 0.03, groove)) * 0.9 + crack * 0.9 + pits * 0.6)
    near_ridge = smooth(0.95, 0.35, np.abs(z)) * smooth(-1.3, -0.9, x) * smooth(1.8, 1.4, x) * stone
    c = mixc(c, CHAR, near_ridge * 0.7 * (1 - smooth(0.02, 0.06, groove) * 0.4))
    stone_c = c
    # Wear and grime: a convex edge rubbed pale, a hollow filled dark --
    # read off how the sculpt itself bends, on every material.
    curv = b.curvature(v, t)
    edge = smooth(0.08, 0.5, curv)
    hollow = smooth(-0.06, -0.45, curv)
    # Soft: a charred crust, hot in its fissures.
    soft_c = mixc(CHAR, EMBER_DEEP, lava * 0.85 + vein * 0.35)
    soft_c = mixc(soft_c, EMBER, lava * 0.5)
    albedo = hide_c * skin[:, None] + stone_c * stone[:, None] + soft_c * soft[:, None]
    albedo = mixc(albedo, np.minimum(albedo * 1.45 + 0.08, 1.0), edge * (0.55 * stone + 0.35 * skin))
    albedo = albedo * (1 - 0.40 * hollow * (1 - soft))[:, None]
    # Glow: the soft places, and the seams of the scutes beside the ridge,
    # as if the heat in it showed through the joins.
    seam = (1 - smooth(0.004, 0.022, groove)) * near_ridge * smooth(0.0, 0.5, up)
    glow = soft * (0.06 + lava * 1.0 + vein * 0.35) + seam * 0.9 + crack * near_ridge * 0.8
    glow_c = mixc(EMBER_DEEP, EMBER, glow)[:, :] * glow[:, None]
    rough = 0.82 * skin * (1 - beak) + 0.55 * skin * beak + 0.9 * stone + 0.5 * soft
    return v, albedo, glow_c, rough


def paint_hard(pieces):
    """Horn, spine, quill, claw and tooth: dark at the root, pale toward the
    tip, banded where its growth ridges are, a little glossy."""
    cols, glows, roughs = [], [], []
    for (v, f), kind in pieces:
        v = np.asarray(v)
        # along: from the first ring to the last vertex, 0 to 1
        root = v[0]
        dist = np.linalg.norm(v - root, axis=1)
        s = dist / max(dist.max(), 1e-6)
        if kind == "horn":
            c = mixc(HORN_ROOT, HORN_TIP, smooth(0.05, 0.75, s))
        elif kind in ("tooth", "claw"):
            c = mixc(np.array([0.30, 0.25, 0.18]), np.array([0.88, 0.84, 0.72]), smooth(0.0, 0.8, s))
        elif kind == "quill":
            c = mixc(np.array([0.45, 0.12, 0.05]), np.array([0.92, 0.86, 0.70]), smooth(0.05, 0.6, s))
        else:
            c = mixc(np.array([0.34, 0.29, 0.24]), np.array([0.90, 0.85, 0.74]), smooth(0.1, 0.9, s))
        c = c * (0.85 + 0.3 * b.fbm(v, 9.0, 2, seed=50))[:, None]
        g = np.zeros((len(v), 3))
        if kind == "quill":
            g = mixc(EMBER_DEEP, EMBER, np.ones(len(v))) * smooth(0.35, 0.0, s)[:, None] * 0.8
        cols.append(c)
        glows.append(g)
        roughs.append(np.full(len(v), 0.45 if kind in ("horn", "tooth", "claw") else 0.6))
    return cols, glows, roughs


def eyes(surf):
    out = []
    for s in (-1, 1):
        hit = surf.toward((4.86, 5.44, s * 0.7), (4.86, 5.36, 0.0))
        if hit is None:
            continue
        p, n = hit
        out.append((b.blob(p - n * 0.05, (0.095, 0.07, 0.065), n=16), "eye"))
    return out


# ---------------------------------------------------------------------------
# 5. Weights and parts
# ---------------------------------------------------------------------------


def bones(sk):
    i = sk.index
    out = [
        (i["root"], (-3.1, 4.95, 0), (-1.0, 5.0, 0), 0.85),
        (i["spine"], (-1.0, 5.0, 0), (0.6, 5.1, 0), 0.9),
        (i["chest"], (0.6, 5.1, 0), (2.0, 5.2, 0), 0.95),
        (i["neck"], (2.0, 5.25, 0), (3.1, 5.35, 0), 0.65),
        (i["neck2"], (3.1, 5.35, 0), (3.95, 5.3, 0), 0.55),
        (i["head"], (4.05, 5.25, 0), (5.9, 4.95, 0), 0.5),
        (i["tail1"], (-3.1, 5.0, 0), (-4.15, 4.95, 0), 0.5),
        (i["tail2"], (-4.15, 4.95, 0), (-5.4, 4.5, 0), 0.36),
        (i["tail3"], (-5.4, 4.5, 0), (-6.54, 3.9, 0), 0.25),
        (i["tail4"], (-6.54, 3.9, 0), (-8.0, 3.3, 0), 0.15),
    ]
    for side, z in (("l", -1.0), ("r", 1.0)):
        out += [
            (i[f"shoulder.{side}"], (1.10, 4.55, 1.0 * z), (1.04, 2.35, 1.0 * z), 0.55),
            (i[f"forearm.{side}"], (1.04, 2.35, 1.0 * z), (1.18, 0.0, 1.0 * z), 0.48),
            (i[f"thigh.{side}"], (-1.86, 4.45, 0.97 * z), (-1.55, 2.3, 0.97 * z), 0.55),
            (i[f"shin.{side}"], (-1.55, 2.3, 0.97 * z), (-1.76, 0.0, 0.97 * z), 0.45),
        ]
    return out


def parts_of(sk, points):
    """The nearest of the creature's parts to each point (weak or not)."""
    best = np.full(len(points), np.inf)
    out = np.zeros(len(points), dtype=np.int64)
    for k in range(len(sk.parts)):
        lo, hi = sk.part_box(k)
        d = b.box_distance(points, lo, hi)
        better = d < best
        out[better], best[better] = k, d[better]
    return out


# ---------------------------------------------------------------------------
# The build
# ---------------------------------------------------------------------------


def main():
    args = sys.argv[1:]
    fast = "--fast" in args
    size = int(args[args.index("--size") + 1]) if "--size" in args else (1024 if fast else 2048)
    look = args[args.index("--look") + 1] if "--look" in args else os.path.join(ROOT, "target/art/ridgeback.png")
    os.makedirs(WORK, exist_ok=True)
    import bpy

    bpy.ops.wm.read_factory_settings(use_empty=True)
    sk = b.Skeleton(SKELETON)
    plan = Plan()

    say("forms")
    cv, ct = b.flesh(soft_forms(), voxel=0.03, smooth=14)
    cn = b.normals(cv, ct)
    surf = b.Surface(cv, ct)
    say(f"coarse skin {len(cv)} verts")
    sv, stt = carapace(cv, ct, plan)

    # The union: skin and shells into one sculpt, carrying which is which.
    say("union")
    hi_voxel = 0.02 if fast else 0.012
    ob = b.obj("sculpt", b.joined([(cv, ct), (sv, stt)]), smooth=False)
    me = ob.data
    kind = np.concatenate([np.zeros(len(cv)), np.ones(len(sv))])
    a = me.attributes.new("kind", "FLOAT", "POINT")
    a.data.foreach_set("value", kind)
    me.remesh_voxel_size = hi_voxel
    me.use_remesh_preserve_attributes = True
    with bpy.context.temp_override(active_object=ob, object=ob, selected_objects=[ob]):
        bpy.ops.object.voxel_remesh()
    hv = np.zeros(len(me.vertices) * 3)
    me.vertices.foreach_get("co", hv)
    hv = b.from_blender(hv.reshape(-1, 3))
    kind = np.zeros(len(me.vertices))
    me.attributes["kind"].data.foreach_get("value", kind)
    v_, t_ = b.read(ob)
    bpy.data.objects.remove(ob)
    ht = t_
    say(f"sculpt {len(hv)} verts, {len(ht)} tris")
    hv = b.taubin(hv, ht, 2)

    # The game's mesh: the union before its detail, made light.
    lv, lt = b.decimate((hv, ht), 60000 / len(ht))
    say(f"game mesh {len(lv)} verts, {len(lt)} tris")

    # The detail and the paint, on the sculpt.
    hv, albedo, glow, rough = sculpt(hv, ht, kind, plan)
    pieces = hard_parts(surf) + eyes(surf)
    hcols, hglows, hroughs = paint_hard(pieces)
    for k, ((v, f), kd) in enumerate(pieces):
        if kd == "eye":
            hcols[k] = np.tile(EYE, (len(v), 1))
            hglows[k] = np.tile(EYE * 1.2, (len(v), 1))
            hroughs[k] = np.full(len(v), 0.2)
    hard_v, hard_f = b.joined([p[0] for p in pieces])
    hard_t = b.triangles(hard_f)
    say(f"{len(pieces)} hard pieces, {len(hard_t)} tris")

    # High: the sculpt and the hard pieces, carrying their paint.
    high_v = np.concatenate([hv, hard_v])
    high_t = np.concatenate([ht, hard_t + len(hv)])
    high_c = np.concatenate([albedo] + hcols)
    high_g = np.concatenate([glow] + hglows)
    high_r = np.concatenate([rough] + hroughs)
    # Low: the light mesh and the same hard pieces.
    low_v = np.concatenate([lv, hard_v])
    low_t = np.concatenate([lt, hard_t + len(lv)])

    def srgb_lin(c):
        return b.srgb_to_linear(np.clip(c, 0, 1))

    high = b.obj("high", (high_v, high_t))
    for name, vals in (("albedo", srgb_lin(high_c)), ("glow", np.clip(high_g, 0, 4)),
                       ("rough", np.stack([high_r] * 3, 1))):
        ca = high.data.color_attributes.new(name, "FLOAT_COLOR", "POINT")
        ca.data.foreach_set("color", np.concatenate([vals, np.ones((len(vals), 1))], 1).astype(np.float32).ravel())
    low = b.obj("low", (low_v, low_t))
    say("unwrap")
    extrude = np.concatenate([np.full(len(lv), 0.06), np.full(len(hard_v), 0.006)])
    tex = b.bake_textures(low, high, size, WORK, fast=fast, extrude=extrude)
    say("baked")

    # Weights, parts, and the file.
    gv, gn, guv, gtan, gt, src = b.gather_low(low)
    segs = bones(sk)
    w_skin = b.weights(lv, lt, segs, falloff=0.15, smooth=10)
    # Hard pieces move as one with the skin at their root.
    w_low = np.zeros((len(low_v), w_skin.shape[1]))
    w_low[:len(lv)] = w_skin
    near = b.nearest(lv, hard_v)
    off = len(lv)
    for (v, f), kd in pieces:
        k = len(v)
        rootv = int(np.argmin(surf.distance(np.asarray(v[: min(k, 64)]))))
        w_low[off:off + k] = w_skin[near[off - len(lv) + rootv]]
        off += k
    weights = w_low[src]
    part = parts_of(sk, gv)
    say(f"export: {len(gv)} verts, {len(gt)} tris")
    if "--no-export" not in args:
        b.write_beast_glb(OUT, sk, gv, gn, guv, gtan, gt, weights, part, tex, "ridgeback")
        say(f"wrote {OUT}: {os.path.getsize(OUT) / 1e6:.1f} MB")
    b.studio_textured(low, tex, look)
    say(f"wrote {look}")


if __name__ == "__main__":
    main()
