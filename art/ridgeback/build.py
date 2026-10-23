"""The Ridgeback, formed by hand over its skeleton: the target a grown body
is measured against (``docs/design/exploration/0029_a_target_body.md``).

    cargo run -p anim --bin export_beast -- ridgeback     # its skeleton, once
    art/blender.sh art/ridgeback/build.py [--look out.png] [--clay] [--no-export]

What it is, from ``docs/design/monsters.md``: thirteen metres nose to tail,
five and a half at the back on long legs, armoured on the head, flanks,
haunch and tail, soft only on top -- the ridge it is named for, standing
proud of the spine, and the nape where the collar plates part. Drawn as a
heavy graviportal grazer that bites: a rhino's legs and hide, an
ankylosaur's back, a wedge of a head carried low with horns over the eyes.

Every number is metres in the game's frame (``+x`` to the head, ``+y`` up,
``+z`` its right) at rest; the left side is written and mirrored.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..")
sys.path[:0] = [os.path.join(HERE, "..", "lib")]

import numpy as np  # noqa: E402

import beast as b  # noqa: E402

SKELETON = os.path.join(ROOT, "target/art/ridgeback.json")
OUT = os.path.join(ROOT, "crates/game/assets/ridgeback.glb")


def st(t, w, top, bot, nt=2.0, nb=2.0, bumps=(), dy=0.0, dz=0.0):
    return {"t": t, "w": w, "top": top, "bot": bot, "nt": nt, "nb": nb, "bumps": bumps, "dy": dy, "dz": dz}


def sym(*pairs):
    """Bumps either side of the section: ``(angle from the top, width,
    height)`` written once."""
    out = []
    for a, w, h in pairs:
        out.append((a, w, h))
        if abs(a) > 1e-6 and abs(abs(a) - np.pi) > 1e-6:
            out.append((-a, w, h))
    return tuple(out)


# ---------------------------------------------------------------------------
# The flesh: every soft form, unioned into one skin
# ---------------------------------------------------------------------------


def muscled(rows, muscles):
    """Stations whose bumps are the same muscles all the way down, each one
    swelling and fading with the height given per station: a muscle as a
    form under the skin rather than a lump on it. ``rows``: ``(t, w, top,
    bot)``; ``muscles``: ``(angle from the front, width, [height per
    row])``."""
    out = []
    for i, (t, w, top, bot) in enumerate(rows):
        bumps = tuple((a, wd, hs[i]) for a, wd, hs in muscles)
        out.append(st(t, w, top, bot, bumps=bumps))
    return out


def torso():
    """Hips to the base of the neck along a line at 4.9 m; each station is
    written as where its back and belly are, in metres off the floor. The
    flanks come down round the tops of the legs, as an elephant's do, so a
    leg leaves the body at the elbow rather than the hip."""
    y0 = 4.85
    x0, x1 = -3.4, 2.5
    lo = 2.25  # the lower flank, either side of the belly
    rows = [
        # x,    back, belly, half-width, how far the flank drops round a leg
        (-3.4, 5.25, 4.55, 0.52, 0.0),
        (-2.8, 5.52, 4.05, 1.08, 0.10),
        (-2.1, 5.66, 3.72, 1.30, 0.26),     # the hip points, the thigh inside the skin
        (-1.5, 5.62, 3.70, 1.32, 0.24),
        (-0.8, 5.56, 3.82, 1.30, 0.04),     # a waist before the barrel
        (-0.1, 5.64, 3.66, 1.42, 0.0),      # the belly at its deepest
        (0.6, 5.74, 3.66, 1.44, 0.06),
        (1.1, 5.90, 3.70, 1.36, 0.24),      # the withers, the upper arm inside the skin
        (1.7, 5.96, 3.96, 1.10, 0.18),
        (2.5, 5.92, 4.55, 0.76, 0.0),
    ]
    stations = [st((x - x0) / (x1 - x0), w, back - y0, y0 - belly, nt=2.6, nb=2.0,
                   bumps=sym((1.05, 0.4, 0.05), (lo, 0.45, drop)))
                for x, back, belly, w, drop in rows]
    path = [(x0, y0, 0), (-1.5, y0, 0), (0.5, y0, 0), (x1, y0, 0)]
    return b.tube(path, stations)


def neck():
    path = [(1.5, 5.15, 0), (2.3, 5.42, 0), (3.35, 5.58, 0), (4.25, 5.52, 0)]
    return b.tube(path, [
        st(0.0, 1.00, 0.80, 1.05),
        st(0.35, 0.70, 0.58, 0.74, nt=2.3, bumps=sym((2.0, 0.5, 0.06))),
        st(0.75, 0.56, 0.45, 0.58, nt=2.3, bumps=sym((2.0, 0.5, 0.04))),
        st(1.0, 0.52, 0.42, 0.50, bumps=sym((2.0, 0.5, 0.0))),
    ])


def head():
    """The skull, carried with its snout down; the jaw its own form under it."""
    skull = b.tube([(3.95, 5.66, 0), (4.6, 5.66, 0), (5.25, 5.48, 0), (5.85, 5.24, 0)], [
        st(0.0, 0.50, 0.46, 0.42, bumps=sym((2.0, 0.35, 0.0), (1.25, 0.3, 0.0))),
        st(0.25, 0.58, 0.50, 0.44, nt=2.4, bumps=sym((2.0, 0.35, 0.16), (1.25, 0.3, 0.04))),
        st(0.5, 0.50, 0.42, 0.38, nt=2.4, bumps=sym((2.0, 0.35, 0.04), (1.25, 0.3, 0.10))),
        st(0.8, 0.38, 0.33, 0.30, nt=2.6, bumps=sym((2.0, 0.35, 0.0), (1.25, 0.3, 0.02))),
        st(1.0, 0.22, 0.19, 0.22, bumps=sym((2.0, 0.35, 0.0), (1.25, 0.3, 0.0))),
    ])
    jaw = b.tube([(4.1, 5.30, 0), (4.85, 5.16, 0), (5.7, 5.02, 0)], [
        st(0.0, 0.50, 0.18, 0.26, nb=2.6),
        st(0.5, 0.38, 0.15, 0.22, nb=2.6),
        st(1.0, 0.20, 0.11, 0.13),
    ])
    jowl = [b.blob((4.30, 5.40, s * 0.46), (0.34, 0.27, 0.18)) for s in (-1, 1)]   # the bite's muscle
    brows = [b.blob((4.82, 5.96, s * 0.34), (0.34, 0.13, 0.18)) for s in (-1, 1)]
    boss = b.blob((5.48, 5.62, 0), (0.24, 0.15, 0.16))                              # under the nose horn
    return [skull, jaw, boss] + jowl + brows


def tail():
    path = [(-2.9, 5.02, 0), (-4.15, 4.95, 0), (-5.4, 4.5, 0), (-6.54, 3.9, 0), (-7.65, 3.42, 0)]
    return b.tube(path, [
        st(0.0, 0.66, 0.48, 0.56, nt=2.4),
        st(0.25, 0.50, 0.38, 0.40, nt=2.4),
        st(0.5, 0.37, 0.28, 0.29, nt=2.4),
        st(0.75, 0.24, 0.19, 0.19),
        st(1.0, 0.05, 0.05, 0.05),
    ])


# Angles round a leg, from its front toward its inside (+z on the left):
# the outside is -pi/2, the back pi.
OUTSIDE, BACK, FRONT = -np.pi / 2, np.pi, 0.0


def foreleg():
    """The left one: a column from inside the shoulder to the floor, thick
    the whole way, the triceps a mass behind the upper arm, the shoulder
    capped, the elbow's point behind, the forearm's muscle down its outside
    front; the foot a pad that spreads under the weight."""
    z = -0.98
    path = [(1.12, 5.0, -0.85), (1.10, 4.5, z), (1.06, 3.4, -1.0), (1.05, 2.35, z),
            (1.10, 1.2, z), (1.14, 0.45, z), (1.16, 0.0, z)]
    rows = [
        (0.0, 0.62, 0.62, 0.70),
        (0.10, 0.60, 0.60, 0.74),
        (0.30, 0.50, 0.47, 0.60),
        (0.50, 0.43, 0.40, 0.46),   # the elbow
        (0.60, 0.42, 0.41, 0.40),
        (0.80, 0.38, 0.37, 0.37),
        (0.91, 0.37, 0.36, 0.36),   # the wrist
        (1.0, 0.56, 0.58, 0.50),    # the pad
    ]
    muscles = [
        (OUTSIDE + 0.6, 0.55, [0.10, 0.16, 0.06, 0.0, 0.0, 0.0, 0.0, 0.0]),     # deltoid
        (BACK, 0.65, [0.04, 0.10, 0.16, 0.04, 0.0, 0.0, 0.0, 0.0]),          # triceps
        (BACK, 0.30, [0.0, 0.0, 0.0, 0.20, 0.04, 0.0, 0.0, 0.0]),            # the elbow's point
        (OUTSIDE + 0.7, 0.55, [0.0, 0.0, 0.0, 0.06, 0.14, 0.05, 0.0, 0.0]),      # forearm
        (FRONT, 0.5, [0.0, 0.0, 0.06, 0.0, 0.0, 0.0, 0.0, 0.0]),             # biceps
    ]
    return [b.tube(path, muscled(rows, muscles), ref=(1, 0, 0))]


def hindleg():
    z = -0.95
    path = [(-1.75, 5.0, -0.82), (-1.85, 4.4, z), (-1.72, 3.3, -1.0), (-1.56, 2.3, -0.98),
            (-1.76, 1.3, z), (-1.88, 0.6, z), (-1.74, 0.0, z)]
    rows = [
        (0.0, 0.66, 0.70, 0.80),
        (0.12, 0.62, 0.64, 0.80),
        (0.32, 0.52, 0.54, 0.62),
        (0.50, 0.45, 0.46, 0.44),   # the knee
        (0.70, 0.40, 0.36, 0.46),
        (0.87, 0.38, 0.35, 0.38),   # the hock
        (1.0, 0.58, 0.60, 0.50),    # the pad
    ]
    muscles = [
        (BACK + 0.5, 0.6, [0.10, 0.16, 0.08, 0.0, 0.0, 0.0, 0.0]),          # glutes
        (BACK - 0.3, 0.55, [0.0, 0.08, 0.16, 0.04, 0.0, 0.0, 0.0]),          # hamstrings
        (FRONT - 0.3, 0.6, [0.0, 0.06, 0.14, 0.04, 0.0, 0.0, 0.0]),          # quadriceps
        (FRONT, 0.35, [0.0, 0.0, 0.0, 0.14, 0.0, 0.0, 0.0]),                 # the kneecap
        (BACK, 0.55, [0.0, 0.0, 0.0, 0.0, 0.18, 0.04, 0.0]),                 # the calf
        (BACK, 0.3, [0.0, 0.0, 0.0, 0.0, 0.0, 0.14, 0.0]),                   # the heel
    ]
    return [b.tube(path, muscled(rows, muscles), ref=(1, 0, 0))]


def folds():
    """Skin folds at the joints, as a rhino's: arcs half sunk into the skin
    round the front and outside of each leg at the wrist and below the
    elbow and knee, and a deep fold behind the shoulder and before the
    thigh, where the hide gathers as the leg swings."""
    out = []
    legs = (
        # x at the height, z, the heights, the leg's radius there
        (lambda y: 1.10 + 0.02 * (y < 1.2), -0.98, (0.56, 0.74, 2.10), (0.37, 0.37, 0.44)),
        (lambda y: -1.82 if y < 1.0 else -1.70, -0.95, (0.70, 0.90, 2.02), (0.39, 0.38, 0.45)),
    )
    for xf, z, ys, rs in legs:
        for k, (y, r) in enumerate(zip(ys, rs)):
            # Round the front and the outside, each its own length and sag,
            # so they read as gathered skin rather than rings.
            a = np.linspace(-2.2 + 0.35 * k, 0.7 - 0.25 * k, 14)
            sag = (0.04 + 0.03 * k) * np.sin(np.linspace(0, np.pi, 14)) + 0.02 * np.sin(np.linspace(0, 9, 14) + k)
            pts = [(xf(y) + r * np.cos(ai), y - s_, z + r * np.sin(ai)) for ai, s_ in zip(a, sag)]
            out.append(b.horn(pts, [0.035, 0.055, 0.06, 0.055, 0.035, 0.0], sides=10, tip=0.02))
    # The great folds behind the shoulder and in front of the thigh.
    for x, top, bot, lean in ((0.48, 5.55, 3.75, 0.12), (-1.15, 5.45, 3.80, -0.10)):
        pts = [(x + lean * f, top + (bot - top) * f, -1.30 - 0.12 * np.sin(np.pi * f)) for f in np.linspace(0, 1, 8)]
        out.append(b.horn(pts, [0.03, 0.09, 0.11, 0.11, 0.09, 0.03], sides=12, tip=0.02))
    return out


def toe_bulges():
    out = []
    for fx, fz, r in ((1.16, -0.98, 0.56), (-1.74, -0.95, 0.58)):
        for a in np.radians([-55, -18, 18, 55]):
            d = np.array([np.cos(a), 0.0, np.sin(a)])
            out.append(b.blob(np.array([fx, 0.22, fz]) + d * (r - 0.14), (0.17, 0.16, 0.15)))
    return out


def soft_forms():
    left = foreleg() + hindleg() + folds() + toe_bulges()
    return [torso(), neck(), tail()] + head() + left + [b.mirror(f) for f in left]


# ---------------------------------------------------------------------------
# The hard parts: plates, horns, nails, quills
# ---------------------------------------------------------------------------


def lerp_profile(x, xs, vs):
    return float(np.interp(x, xs, vs))


def back_plates(surf):
    """Two rows of keeled plates either side of the ridge, from the withers
    to the root of the tail, tallest over the barrel and leaning out, so the
    ridge runs in the trench between them."""
    out = []
    xs = np.arange(1.75, -3.05, -0.40)
    for i, x in enumerate(xs):
        height = lerp_profile(x, [-3.0, -2.0, -0.4, 0.8, 1.8], [0.34, 0.52, 0.66, 0.58, 0.36])
        length = lerp_profile(x, [-3.0, -0.4, 1.8], [0.42, 0.62, 0.44])
        stagger = 0.08 if i % 2 else -0.08
        for side in (-1, 1):
            z = side * (0.50 + 0.03 * np.cos(i))
            hit = surf.down(x + stagger * side, z)
            if hit is None:
                continue
            p, n = hit
            lean = np.array([0.0, 1.0, 0.0]) + np.array([0, 0, side * 0.42]) + n * 0.4
            out.append(b.osteoderm(p, lean, (1, 0, 0), length, height, 0.10, sweep=0.35))
    return out


def tail_spikes(surf, path):
    """Down the tail, the plates' rows carry on as spikes that shrink to the
    tip; on its middle and end, the quills it sprays, out of both sides."""
    curve = b.spline(path, 200)
    tang = np.gradient(curve, axis=0)
    tang /= np.linalg.norm(tang, axis=1, keepdims=True)
    out = []
    for k, s in enumerate(np.linspace(0.04, 0.86, 11)):
        i = int(s * 199)
        c, tg = curve[i], tang[i]
        up = np.array([0.0, 1.0, 0.0]) - tg * tg[1]
        up /= np.linalg.norm(up)
        size = 1.0 - 0.75 * s
        for side in (-1, 1):
            hit = surf.toward(c + up * 0.6 + np.array([0, 0, side * 0.32 * size]), c)
            if hit is None:
                continue
            p, n = hit
            lean = up + np.array([0, 0, side * 0.5])
            out.append(b.osteoderm(p, lean, tg, 0.36 * size + 0.08, 0.42 * size + 0.08, 0.07 * size + 0.02,
                                   sweep=0.5))
    # The quills: out of the flanks of the tail's last half, raked back.
    for s in np.linspace(0.5, 0.92, 8):
        i = int(s * 199)
        c, tg = curve[i], tang[i]
        for side in (-1, 1):
            hit = surf.toward(c + np.array([0, 0.05, side * 0.5]), c)
            if hit is None:
                continue
            p, n = hit
            d = n * 0.8 - tg * 0.9 + np.array([0, 0.25, 0])
            d /= np.linalg.norm(d)
            length = 0.55 - 0.25 * (s - 0.5)
            out.append(b.horn([p - d * 0.05, p + d * length * 0.5, p + d * length], [0.06, 0.035, 0.0], sides=10))
    # The end: three spikes off the tip.
    tip, tg = curve[-1], tang[-1]
    for a in (-0.5, 0.0, 0.5):
        d = tg + np.array([0, 0.35 + 0.2 * abs(a), a])
        d /= np.linalg.norm(d)
        out.append(b.horn([tip - tg * 0.25, tip + d * 0.3, tip + d * 0.62], [0.07, 0.04, 0.0], sides=10))
    return out


def head_hard():
    """Two horns over the eyes, sweeping forward and out; a horn on the
    nose; a crown of spikes off the back of the skull over the neck; tusks
    out of the jaw."""
    out = []
    for s in (-1, 1):
        out.append(b.horn([(4.62, 5.98, s * 0.36), (4.85, 6.38, s * 0.52), (5.15, 6.70, s * 0.60),
                           (5.45, 6.82, s * 0.56)], [0.17, 0.13, 0.08, 0.0], sides=16))
        out.append(b.horn([(5.30, 5.12, s * 0.20), (5.42, 5.36, s * 0.27), (5.50, 5.55, s * 0.30)],
                          [0.065, 0.045, 0.0], sides=10))
    out.append(b.horn([(5.40, 5.66, 0), (5.62, 5.92, 0), (5.76, 6.12, 0)], [0.14, 0.08, 0.0], sides=14))
    for k, z in enumerate((-0.42, -0.21, 0.0, 0.21, 0.42)):
        h = 0.62 - 0.18 * abs(z) / 0.42
        out.append(b.horn([(4.15, 5.92, z), (3.85, 6.12 + h * 0.4, z * 1.3), (3.55, 6.12 + h, z * 1.6)],
                          [0.10, 0.06, 0.0], sides=12))
    return out


def eyes(surf):
    """An eye either side under the brow, set into the face where a ray
    from the side meets it, half sunk; they glow, as the soft places do."""
    out = []
    for s in (-1, 1):
        hit = surf.toward((4.92, 5.78, s * 0.6), (4.92, 5.70, 0.0))
        if hit is None:
            continue
        p, n = hit
        out.append(b.blob(p - n * 0.02, (0.085, 0.065, 0.06)))
    return out


def toes():
    """Four hoof-nails round the front of every foot."""
    out = []
    feet = [(1.16, -0.98, 0.56), (-1.74, -0.95, 0.58)]
    for fx, fz, r in feet:
        for side in (-1, 1):
            z = fz * -side if side == 1 else fz
            for a in np.radians([-55, -18, 18, 55]):
                d = np.array([np.cos(a), 0.0, np.sin(a)])
                c = np.array([fx, 0.14, z]) + d * (r - 0.02)
                out.append(b.nail(c, d + np.array([0, -0.25, 0]), (0.16, 0.15, 0.15)))
    return out


def ridge(surf):
    """The ridge: a row of soft folds standing up off the spine between the
    plates, the strip a rider is climbing for."""
    out = []
    for x in np.arange(-0.75, 1.10, 0.13):
        hit = surf.down(x, 0.0)
        if hit is None:
            continue
        p, n = hit
        h = 0.30 + 0.08 * np.cos((x - 0.15) * 1.7)
        out.append(b.blob(p + np.array([0, h * 0.35, 0]), (0.05, h, 0.30), n=14))
    out.append(b.tube([(-0.85, 5.62, 0), (0.15, 5.68, 0), (1.15, 5.86, 0)],
                      [{"t": 0, "w": 0.2, "top": 0.12, "bot": 0.12}, {"t": 1, "w": 0.2, "top": 0.12, "bot": 0.12}]))
    return out


def nape(surf):
    out = []
    for x in np.arange(2.30, 3.25, 0.12):
        hit = surf.down(x, 0.0)
        if hit is None:
            continue
        p, n = hit
        out.append(b.blob(p + np.array([0, 0.06, 0]), (0.05, 0.20, 0.24), n=14))
    return out


def along_axis(path, ref=(0, 1, 0), n=300):
    """A curve and its frames, for laying plates round it."""
    c = b.spline(path, n)
    tg, side, up = b.frames(c, ref)
    return c, tg, side, up


def ring_ray(axis, s, phi, reach=4.0):
    """The ray from outside the body toward its axis at ``s`` (0 to 1 along
    it) and ``phi`` round it (0 on top, toward ``+side``)."""
    c, tg, side, up = axis
    i = int(np.clip(s, 0, 1) * (len(c) - 1))
    d = up[i] * np.cos(phi) + side[i] * np.sin(phi)
    return c[i] + d * reach, -d


def banded(surf, axis, s_ranges, phi_ranges, thick, lip, bevel=0.07, crown=0.02, overlap=0.15, seed=0,
           round_=0.55, keel=0.0, stagger=()):
    """Plates round an axis: one per ``s`` range and ``phi`` range, on the
    left side (``phi`` negative); the right is the mirror. ``overlap`` of a
    plate's length runs on behind it, raised, over the next."""
    out = []
    k = seed
    for s0, s1 in s_ranges:
        for row, (p0, p1) in enumerate(phi_ranges):
            k += 1
            shift = (s1 - s0) * (stagger[row] if row < len(stagger) else 0.0)
            s0, s1 = s0 + shift, s1 + shift
            length = s1 - s0
            sa = s0 - overlap * length

            def ray(u, v, sa=sa, s1=s1, p0=p0, p1=p1):
                # u = 0 is the plate's back edge, the one that overlaps.
                return ring_ray(axis, sa + (s1 - sa) * u, p0 + (p1 - p0) * v)

            nu = max(7, int(length * 55))
            nv = max(7, int(abs(p1 - p0) * 11))
            # No two alike: a little thicker or thinner, a little more lip.
            rng = np.random.default_rng(k)
            plate = b.shell(surf, ray, nu, nv, thick=thick * rng.uniform(0.85, 1.2), lift=0.015, bevel=bevel,
                            lip=lip * rng.uniform(0.7, 1.3), crown=crown, rough=0.015, seed=k,
                            round_=round_, keel=keel if row == 0 else keel * 0.4)
            if plate is not None:
                out.append(plate)
            s0, s1 = s0 - shift, s1 - shift
    return out


def mirror3(plate):
    v, f, w = plate
    mv, mf = b.mirror((v, f))
    return mv, mf, w


TORSO_AXIS = [(-3.4, 4.85, 0), (-1.5, 4.85, 0), (0.5, 4.85, 0), (2.5, 4.85, 0)]
NECK = [(1.5, 5.15, 0), (2.3, 5.42, 0), (3.35, 5.58, 0), (4.25, 5.52, 0)]
SKULL = [(3.95, 5.66, 0), (4.6, 5.66, 0), (5.25, 5.48, 0), (5.85, 5.24, 0)]
FORELEG = [(1.10, 4.5, -0.98), (1.06, 3.4, -1.0), (1.05, 2.35, -0.98)]
HINDLEG = [(-1.85, 4.4, -0.95), (-1.72, 3.3, -1.0), (-1.56, 2.3, -0.98)]


TAIL = [(-2.9, 5.02, 0), (-4.15, 4.95, 0), (-5.4, 4.5, 0), (-6.54, 3.9, 0), (-7.65, 3.42, 0)]


def armour(surf):
    """The hide's plates, left side: banded across the flanks in two rows,
    collars round the neck that part on top for the nape, a helm on the
    skull, bands down the tail, a shield on the outside of every upper leg.
    Mirrored for the right."""
    left = []
    # The flanks: the torso axis runs back to front, so s is x.
    torso = along_axis(TORSO_AXIS)
    x0, x1 = -3.4, 2.5
    xs = np.arange(-3.0, 1.95, 0.46)
    rows = [((x - x0) / (x1 - x0), (x + 0.42 - x0) / (x1 - x0)) for x in xs]
    left += banded(surf, torso, rows, [(-0.64, -1.10), (-1.08, -1.50)], thick=0.09, lip=0.06, seed=10,
                   overlap=0.22, keel=0.07, stagger=(0.0, 0.5), crown=0.03)
    # The neck: collars either side of the nape, which stays bare on top.
    neck = along_axis(NECK)
    left += banded(surf, neck, [(0.12, 0.32), (0.33, 0.53), (0.54, 0.74), (0.75, 0.95)],
                   [(-0.42, -1.45)], thick=0.07, lip=0.04, seed=40)
    # The tail: bands over its top and sides, shrinking with it.
    tail = along_axis(TAIL)
    ss = np.linspace(0.0, 0.88, 12)
    left += banded(surf, tail, [(a, b_ - 0.008) for a, b_ in zip(ss[:-1], ss[1:])], [(-0.18, -1.4)],
                   thick=0.06, lip=0.035, bevel=0.05, seed=60)
    # The legs: a shield on the outside of each upper leg, from the elbow or
    # knee to where the flank comes down over it.
    for path, seed in ((FORELEG, 80), (HINDLEG, 90)):
        leg = along_axis(path, ref=(1, 0, 0))
        # Two plates over the upper leg like a pauldron: the upper one
        # larger, overlapping the lower; s runs down the leg, so its back
        # edge -- the one that overlaps -- is its lower.
        left += banded(surf, leg, [(0.10, 0.52)], [(-np.pi / 2 - 0.95, -np.pi / 2 + 0.95)], thick=0.09,
                       lip=0.0, bevel=0.08, crown=0.05, overlap=0.0, seed=seed, round_=0.7, keel=0.05)
        left += banded(surf, leg, [(0.50, 0.86)], [(-np.pi / 2 - 0.8, -np.pi / 2 + 0.8)], thick=0.08,
                       lip=0.0, bevel=0.08, crown=0.04, overlap=0.0, seed=seed + 1, round_=0.7)
    right = [mirror3(p) for p in left]
    # The helm on the skull, one plate across the top.
    skull = along_axis(SKULL)
    helm = banded(surf, skull, [(0.18, 0.62)], [(-1.05, 1.05)], thick=0.06, lip=0.05, crown=0.05,
                  overlap=0.1, seed=99)
    return left + right + helm


def bones(sk):
    """Each bone's segment and how thick the body round it is, for the
    weights: the body's bones along its middle, the legs' down them."""
    i = sk.index
    out = [
        (i["root"], (-3.1, 4.95, 0), (-1.0, 5.0, 0), 0.85),
        (i["spine"], (-1.0, 5.0, 0), (0.6, 5.1, 0), 0.9),
        (i["chest"], (0.6, 5.1, 0), (1.9, 5.2, 0), 0.9),
        (i["neck"], (1.9, 5.3, 0), (3.0, 5.5, 0), 0.6),
        (i["neck2"], (3.0, 5.55, 0), (3.95, 5.6, 0), 0.5),
        (i["head"], (4.05, 5.55, 0), (5.9, 5.3, 0), 0.45),
        (i["tail1"], (-3.1, 5.0, 0), (-4.15, 4.95, 0), 0.45),
        (i["tail2"], (-4.15, 4.95, 0), (-5.4, 4.5, 0), 0.32),
        (i["tail3"], (-5.4, 4.5, 0), (-6.54, 3.9, 0), 0.22),
        (i["tail4"], (-6.54, 3.9, 0), (-7.9, 3.3, 0), 0.12),
    ]
    for side, z in (("l", -1.0), ("r", 1.0)):
        out += [
            (i[f"shoulder.{side}"], (1.10, 4.55, 0.98 * z), (1.05, 2.35, 0.98 * z), 0.48),
            (i[f"forearm.{side}"], (1.05, 2.35, 0.98 * z), (1.16, 0.0, 0.98 * z), 0.40),
            (i[f"thigh.{side}"], (-1.85, 4.45, 0.95 * z), (-1.56, 2.3, 0.95 * z), 0.52),
            (i[f"shin.{side}"], (-1.56, 2.3, 0.95 * z), (-1.74, 0.0, 0.95 * z), 0.42),
        ]
    return out


def parts_of(sk, points, weak):
    """The nearest part to each point, of the weak points or of the rest."""
    best = np.full(len(points), np.inf)
    out = np.zeros(len(points), dtype=np.int64)
    for k, part in enumerate(sk.parts):
        if part["weak"] != weak:
            continue
        lo, hi = sk.part_box(k)
        d = b.box_distance(points, lo, hi)
        better = d < best
        out[better], best[better] = k, d[better]
    return out


def mix(a, c, f):
    f = np.clip(np.asarray(f, float), 0, 1)[:, None]
    return np.asarray(a, float) * (1 - f) + np.asarray(c, float) * f


def smooth(a, c, x):
    t = np.clip((x - a) / (c - a), 0, 1)
    return t * t * (3 - 2 * t)


# The palette, as the eye sees it (sRGB): a weathered grey-brown hide paler
# beneath, slate armour gone green with moss on what faces the sky, horn
# that darkens to its root, and the soft places hot.
HIDE_BACK = (0.33, 0.30, 0.26)
HIDE_FLANK = (0.50, 0.44, 0.36)
HIDE_BELLY = (0.70, 0.62, 0.48)
MUD = (0.30, 0.25, 0.19)
SLATE = (0.36, 0.40, 0.35)
WORN = (0.56, 0.55, 0.47)
MOSS = (0.36, 0.42, 0.17)
HORN_ROOT = (0.30, 0.24, 0.18)
HORN_TIP = (0.92, 0.87, 0.72)
EMBER = (1.0, 0.45, 0.20)
EMBER_DEEP = (0.62, 0.12, 0.06)
EYE = (1.0, 0.70, 0.18)


def main():
    args = sys.argv[1:]
    look = args[args.index("--look") + 1] if "--look" in args else os.path.join(ROOT, "target/art/ridgeback.png")
    import bpy

    bpy.ops.wm.read_factory_settings(use_empty=True)
    sk = b.Skeleton(SKELETON)
    v, t = b.flesh(soft_forms(), voxel=0.035, smooth=16)
    n = b.normals(v, t)
    print(f"flesh: {len(v)} verts, {len(t)} tris")
    surf = b.Surface(v, t)
    plates = armour(surf)
    hard = back_plates(surf) + tail_spikes(surf, TAIL) + head_hard() + toes()
    glow = ridge(surf) + nape(surf) + eyes(surf)
    print(f"{len(plates)} plates, {len(hard)} hard parts, {len(glow)} soft folds")

    fv, ft = b.decimate((v, t), 0.17)
    fn = b.normals(fv, ft)
    pv, pf = b.joined([(p[0], p[1]) for p in plates])
    pw = np.concatenate([p[2] for p in plates])
    pid = np.concatenate([np.full(len(p[0]), k) for k, p in enumerate(plates)])
    pv, pt, pn, (pw, pid) = b.hard_normals(pv, b.triangles(pf), 35.0, attrs=(pw, pid))
    hv, hf = b.joined(hard)
    ht = b.triangles(hf)
    hn = b.normals(hv, ht)
    gv, gf = b.joined(glow)
    gt = b.triangles(gf)
    gn = b.normals(gv, gt)
    print(f"flesh {len(ft)}, plates {len(pt)}, hard {len(ht)}, glow {len(gt)} tris")

    # How shut in each vertex is, against everything solid.
    allv, allt = b.joined([(fv, ft), (pv, pt), (hv, ht)])
    print("occlusion...")
    f_ao = b.occlusion(fv, fn, allv, allt, rays=14, reach=1.2)
    p_ao = b.occlusion(pv, pn, allv, allt, rays=10, reach=0.9)
    h_ao = b.occlusion(hv, hn, allv, allt, rays=8, reach=0.6)

    # The hide.
    up = fn[:, 1]
    c = mix(HIDE_FLANK, HIDE_BELLY, smooth(-0.15, -0.75, up))
    c = mix(c, HIDE_BACK, smooth(0.25, 0.85, up))
    c = mix(c, MUD, smooth(1.6, 0.2, fv[:, 1]) * 0.85)
    curv = b.curvature(fv, ft, fn)
    c = c * (1 + 0.5 * np.clip(curv, -0.6, 0.6))[:, None]
    c = c * (0.82 + 0.36 * b.noise3(fv, 1.3, 1.0))[:, None]
    c = c * (0.30 + 0.70 * f_ao ** 1.3)[:, None]
    f_col = c

    # The armour: each plate its own shade of slate, worn pale at its
    # edges, mossed where it faces the sky, dark where it is shut in.
    rng = np.random.default_rng(5)
    shade = rng.uniform(0.82, 1.15, pid.max() + 1)[pid]
    c = np.asarray(SLATE)[None, :] * shade[:, None]
    c = mix(c, WORN, pw * 0.8)
    moss = smooth(0.35, 0.8, pn[:, 1]) * smooth(0.45, 0.7, b.noise3(pv, 2.2, 3.0)) * (1 - pw)
    c = mix(c, MOSS, moss * 0.85)
    c = c * (0.85 + 0.3 * b.noise3(pv, 4.0, 7.0))[:, None]
    c = c * (0.30 + 0.70 * p_ao ** 1.2)[:, None]
    p_col = c

    # Horn, nail and quill: dark at the root, pale toward the tip.
    root = smooth(0.0, 0.45, surf.distance(hv))
    c = mix(HORN_ROOT, HORN_TIP, root ** 0.55)
    c = c * (0.88 + 0.24 * b.noise3(hv, 6.0, 2.0))[:, None]
    c = c * (0.40 + 0.60 * h_ao)[:, None]
    h_col = c

    g_col = mix(EMBER_DEEP, EMBER, smooth(0.0, 0.25, surf.distance(gv)))
    eye = gv[:, 0] > 4.6
    g_col[eye] = EYE

    def rgba(srgb):
        return np.concatenate([b.srgb_to_linear(np.clip(srgb, 0, 1)), np.ones((len(srgb), 1))], 1)

    # Weights: the skin's from its bones; everything on it takes the
    # weights of the skin under it.
    segs = bones(sk)
    fw = b.weights(fv, ft, segs, falloff=0.14, smooth=10)
    pw_ = fw[b.nearest(fv, pv)]
    gw = fw[b.nearest(fv, gv)]
    # A horn or spike moves as one: every vertex the weights at its root.
    hw = np.zeros((len(hv), fw.shape[1]))
    off = 0
    near_h = b.nearest(fv, hv)
    for v_, f_ in hard:
        k = len(v_)
        d = surf.distance(v_)
        rootv = off + int(np.argmin(d))
        hw[off:off + k] = fw[near_h[rootv]]
        off += k

    hide = {
        "name": "hide",
        "pos": np.concatenate([fv, pv, hv]),
        "nrm": np.concatenate([fn, pn, hn]),
        "col": np.concatenate([rgba(f_col), rgba(p_col), rgba(h_col)]),
        "w": np.concatenate([fw, pw_, hw]),
        "idx": np.concatenate([ft, pt + len(fv), ht + len(fv) + len(pv)]),
    }
    hide["part"] = parts_of(sk, hide["pos"], weak=False)
    soft = {"name": "glow", "pos": gv, "nrm": gn, "col": rgba(g_col), "w": gw, "idx": gt,
            "part": parts_of(sk, gv, weak=True)}

    clay = "--clay" in args
    pieces = [(fv, ft, None if clay else rgba(f_col)),
              (pv, pt, None if clay else rgba(p_col)),
              (hv, ht, None if clay else rgba(h_col)),
              (gv, gt, None if clay else rgba(g_col))]
    b.studio(pieces, look, clay=clay)
    b.studio(pieces, look.replace(".png", "_near.png"), clay=clay, distance=7.5,
             views=((60, 1.5, (4.6, 5.6, 0)), (100, 0.0, (1.0, 2.0, -1.0)), (150, 1.0, (-5.0, 4.4, 0)),
                    (90, 4.0, (-0.6, 5.5, 0))))
    print(f"wrote {look}")
    if "--no-export" not in args:
        b.write_glb(OUT, sk, [hide, soft], "ridgeback")
        print(f"wrote {OUT}: {os.path.getsize(OUT) / 1e6:.1f} MB, "
              f"{len(hide['idx']) + len(soft['idx'])} triangles")


def colour_each(rgb):
    return np.concatenate([np.asarray(rgb, float) ** 2.2, np.ones((len(rgb), 1))], 1)


def colour(v, rgb):
    return np.concatenate([np.tile(np.asarray(rgb, float) ** 2.2, (len(v), 1)), np.ones((len(v), 1))], 1)


if __name__ == "__main__":
    main()
