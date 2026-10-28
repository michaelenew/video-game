"""The Galewing, sculpted and baked: the target its grown body is measured
against (``docs/design/exploration/0031_two_more_targets.md``).

    cargo run -p anim --bin export_beast -- galewing     # its skeleton, once
    art/blender.sh art/galewing/build.py [--look out.png] [--size 2048] [--no-export] [--fast]

**What it is** (``docs/design/creatures/galewing.md``): a raptor eight metres
beak to tail with an eighteen-metre span, that lives where you cannot reach
it. Its rest pose is flight: wings spread level.

**What it is drawn as**: storm-grey, slate above and pale below, so against
the sky it reads dark and from under it light (``game::species::galewing``).
An eagle's build pushed toward a monster: a deep keeled breast, a heavy
brow over forward eyes, a hooked beak, a crest of blade feathers that
rises for the screech, a ruff of hackles, coverts shingled over the body and
the wings' tops, long barred flight feathers with worn nicks in them, a
fan of tail feathers, feathered thighs over scaled yellow legs, and black
talons as long as a forearm. The wing roots are a pale band across the
back -- where a hit counts double.

Built on ``art/lib/creature.py``; every number is metres in the game's frame
(``+x`` to the head, ``+y`` up, ``+z`` its right), at rest.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib")]

import numpy as np  # noqa: E402

import beast as b  # noqa: E402
import creature as c  # noqa: E402
from creature import both, mixc, smooth, st  # noqa: E402

NAME = "galewing"
SCALE = 1.0
TRIS = 70000

# The body's axis, the neck's and the head's.
BODY = [(-2.15, 2.70, 0), (-0.8, 2.66, 0), (0.6, 2.66, 0), (1.8, 2.80, 0), (2.45, 2.98, 0)]
NECK = [(2.05, 2.95, 0), (2.70, 3.22, 0), (3.20, 3.40, 0), (3.62, 3.42, 0)]
HEAD = [(3.40, 3.40, 0), (3.85, 3.46, 0), (4.22, 3.38, 0)]
# A left wing's leading edge, shoulder to tip: the bones run along it
# (shoulder 0.2,3.1,-0.85; arm -3.35; hand -6.84).
EDGE = [(0.30, 3.12, -0.55), (0.35, 3.14, -2.0), (0.45, 3.17, -3.35), (0.32, 3.17, -5.0),
        (0.05, 3.14, -6.84), (-0.30, 3.10, -8.0)]
# Where each section of the wing is, along the edge (0 to 1): the elbow and
# the wrist.
ELBOW, WRIST = 0.33, 0.70
HIP = (-0.2, 2.1, -0.6)
KNEE = (-0.1, 1.1, -0.6)
ANKLE = (-0.05, 0.25, -0.6)

SLATE = np.array([0.26, 0.28, 0.32])
SLATE_DARK = np.array([0.13, 0.14, 0.17])
SLATE_PALE = np.array([0.46, 0.48, 0.52])
UNDER = np.array([0.80, 0.77, 0.70])
UNDER_STREAK = np.array([0.42, 0.38, 0.34])
ROOT_BAND = np.array([0.82, 0.78, 0.66])
LEG_YELLOW = np.array([0.86, 0.66, 0.22])
LEG_SCALE = np.array([0.48, 0.34, 0.12])
BEAK = np.array([0.20, 0.20, 0.22])
BEAK_TIP = np.array([0.62, 0.62, 0.58])
CERE = np.array([0.88, 0.70, 0.22])
TALON = np.array([0.07, 0.07, 0.08])
TALON_TIP = np.array([0.40, 0.40, 0.42])
EYE = np.array([0.92, 0.50, 0.08])


# ---------------------------------------------------------------------------
# 1. Forms
# ---------------------------------------------------------------------------


def body():
    """A teardrop: the breast deep and keeled below the wings, the back
    flat between them, the rump narrowing into the tail."""
    return b.tube(BODY, [
        st(0.0, 0.42, 0.34, 0.30),
        st(0.22, 0.74, 0.52, 0.56, nt=2.3),
        st(0.48, 0.92, 0.60, 0.90, nt=2.4, nb=1.7),
        st(0.72, 0.86, 0.60, 0.98, nt=2.3, nb=1.6),
        st(0.90, 0.66, 0.52, 0.70),
        st(1.0, 0.40, 0.38, 0.40),
    ], sides=48)


def neck():
    """An S of neck, thick with hackles at its base."""
    return b.tube(NECK, [
        st(0.0, 0.62, 0.58, 0.66),
        st(0.35, 0.50, 0.50, 0.50),
        st(0.75, 0.38, 0.40, 0.36),
        st(1.0, 0.34, 0.36, 0.32),
    ], sides=40)


def head():
    """A raptor's skull: broad and flat over the brow, the brow a shelf over
    forward eyes, narrowing to where the beak's cere begins."""
    skull = b.tube(HEAD, [
        st(0.0, 0.34, 0.36, 0.32),
        st(0.40, 0.38, 0.36, 0.30, nt=2.6),
        st(0.80, 0.30, 0.28, 0.24),
        st(1.0, 0.20, 0.20, 0.17),
    ], sides=40)
    brows = [b.blob((3.86, 3.62, s * 0.24), (0.30, 0.09, 0.13)) for s in (-1, 1)]
    cheeks = [b.blob((3.78, 3.30, s * 0.27), (0.20, 0.15, 0.10)) for s in (-1, 1)]
    cere = b.blob((4.06, 3.40, 0), (0.12, 0.10, 0.17))
    return [skull, cere] + brows + cheeks


def wing():
    """The left wing's arm: an airfoil slab along the leading edge, deep
    at the root and thinning to the hand, carrying the coverts. The flight
    feathers past its trailing edge are hard pieces (``hard``)."""
    # Chord and the share of it ahead of the edge.
    stations = []
    for t, chord, thick in ((0.0, 1.6, 0.34), (0.15, 1.5, 0.28), (ELBOW, 1.25, 0.22), (0.5, 1.1, 0.17),
                            (WRIST, 0.85, 0.13), (0.86, 0.55, 0.09), (1.0, 0.30, 0.06)):
        half = chord / 2
        stations.append(st(t, half, thick, thick * 0.75, nt=1.6, nb=2.0, dz=-(half - 0.18)))
    return b.tube(EDGE, stations, ref=(0, 1, 0), sides=40)


def legs():
    """A thigh in a thick feathered trouser, a scaled tarsus, and a foot of
    three toes forward and one back."""
    thigh = b.tube([(-0.25, 2.35, -0.52), HIP, (-0.14, 1.55, -0.60), (-0.10, 1.05, -0.60)], [
        st(0.0, 0.40, 0.42, 0.42),
        st(0.4, 0.40, 0.42, 0.42),
        st(0.85, 0.30, 0.32, 0.32),
        st(1.0, 0.20, 0.20, 0.20),
    ], ref=(1, 0, 0), sides=32)
    tarsus = b.tube([KNEE, (-0.08, 0.7, -0.6), ANKLE, (-0.02, 0.10, -0.6)], [
        st(0.0, 0.16, 0.16, 0.16),
        st(0.5, 0.12, 0.13, 0.13),
        st(0.85, 0.13, 0.14, 0.14),
        st(1.0, 0.16, 0.17, 0.17),
    ], ref=(1, 0, 0), sides=24)
    out = [thigh, tarsus]
    for deg, L in TOES:
        a = np.radians(deg)
        d = np.array([np.cos(a), 0.0, np.sin(a)])
        root = np.array([0.0, 0.12, -0.6])
        out.append(b.tube([root, root + d * L * 0.5 + np.array([0, 0.03, 0]), root + d * L - np.array([0, 0.04, 0])], [
            st(0.0, 0.10, 0.09, 0.08),
            st(0.5, 0.075, 0.08, 0.06),
            st(1.0, 0.06, 0.06, 0.05),
        ], sides=16))
    return out


# Toes: (angle round from forward, toward the right; length). Three forward
# and the hallux back.
TOES = ((-32, 0.62), (0, 0.70), (32, 0.60), (180, 0.48))


def forms():
    left = [wing()] + legs()
    return [body(), neck()] + head() + both(left) + [b.blob((-2.0, 2.72, 0), (0.45, 0.22, 0.55))]


# ---------------------------------------------------------------------------
# 2. No shells: a bird's armour is its feathers, which are sculpted into the
# skin (coverts) or stand off it (flights, tail, crest).
# ---------------------------------------------------------------------------


def shells(v, t):
    return None


# ---------------------------------------------------------------------------
# 3. Hard pieces: flight feathers, tail, crest, beak, talons
# ---------------------------------------------------------------------------


def edge_at(u):
    """A point on the left leading edge, ``u`` from shoulder (0) to tip (1)."""
    curve = b.spline(EDGE, 200)
    return curve[int(np.clip(u, 0, 1) * 199)]


def chord_at(u):
    return np.interp(u, (0.0, 0.15, ELBOW, 0.5, WRIST, 0.86, 1.0), (1.6, 1.5, 1.25, 1.1, 0.85, 0.55, 0.30))


def feather(root, d, L, width, seed=0, tilt=0.0, finger=0.0, lean=0.35):
    """One feather from ``root`` along ``d``, ``L`` long and ``width``
    across at its widest: a rounded tip, a slight droop, its vane tilted
    ``tilt`` about its shaft so a row of them shingles, and ``finger`` of
    its outer part narrowed (an eagle's fingered primaries)."""
    d = np.asarray(d, float)
    side = np.cross(d, (0.0, 1.0, 0.0))
    side /= max(np.linalg.norm(side), 1e-9)
    normal = np.array([0.0, 1.0, 0.0]) + side * tilt
    path = [root, root + d * L * 0.3 + (0, 0.035, 0), root + d * L * 0.65 + (0, 0.03, 0),
            root + d * L * 0.9 - (0, 0.01, 0), root + d * L - (0, 0.03, 0)]
    w = width / 2
    widths = [w * 0.55, w, w * (1 - finger), w * 0.8 * (1 - finger), w * 0.12]
    return c.vane(path, widths, normal, thick=0.012, lean=lean, notch=0.12, seed=seed, rings=14)


def flight_feathers():
    """The left wing's: tertials and secondaries along its trailing edge,
    pointing back; primaries off the hand, fanning from back to out past
    the tip, longest at the leading edge; each slightly under the one
    before so they shingle."""
    out = []
    # Secondaries and tertials: from the root to the wrist.
    n = 22
    for i in range(n):
        u = 0.04 + (WRIST - 0.06) * i / (n - 1)
        e = edge_at(u)
        root = e + np.array([-(chord_at(u) - 0.35), -0.02 - 0.004 * i, 0.0])
        L = 1.15 - 0.15 * u / WRIST
        d = np.array([-1.0, -0.05, -0.12 * u])
        d /= np.linalg.norm(d)
        out.append((feather(root, d, L, 0.46, seed=i, tilt=0.12), "flight"))
        # A greater covert over its base, shorter, lying on the wing.
        cov = root + np.array([0.35, 0.10, 0.0])
        out.append((feather(cov, d, 0.62, 0.40, seed=200 + i, tilt=0.1, lean=0.25), "covert"))
    # Primaries: off the hand, fanning from back-and-out to straight out.
    n = 10
    for i in range(n):
        f = i / (n - 1)
        u = WRIST + (1.0 - WRIST) * f * 0.92
        e = edge_at(u)
        root = e + np.array([-chord_at(u) * 0.55, -0.03 - 0.005 * i, 0.0])
        # From pointing back at the wrist to pointing out at the tip.
        a = np.radians(10 + 72 * f)
        d = np.array([-np.cos(a), -0.03, -np.sin(a)])
        d /= np.linalg.norm(d)
        L = 1.2 + 0.5 * np.sin(np.pi * f)
        # Emarginated: narrowing in its outer third, as an eagle's
        # fingered primaries do.
        out.append((feather(root, d, L, 0.40, seed=40 + i, tilt=0.18, finger=0.4 * f), "flight"))
        cov = root + d * -0.05 + np.array([0.12, 0.09, 0.0])
        out.append((feather(cov, d, 0.55, 0.32, seed=240 + i, tilt=0.12, lean=0.25), "covert"))
    return out


def tail_feathers():
    out = []
    n = 12
    for i in range(n):
        f = i / (n - 1) * 2 - 1
        root = np.array([-2.05, 2.72 - 0.004 * abs(f * 6), f * 0.32])
        d = np.array([-1.0, 0.0, f * 0.36])
        d /= np.linalg.norm(d)
        L = 2.15 - 0.15 * abs(f)
        out.append((feather(root, d, L, 0.40, seed=80 + i, tilt=0.1 * f, lean=0.1), "tail"))
    return out


def crest():
    """Blade feathers swept back off the crown, longest in the middle: what
    rises for the screech."""
    out = []
    for i, (z, L) in enumerate(((-0.16, 0.75), (-0.06, 0.95), (0.06, 0.95), (0.16, 0.75), (0.0, 1.10))):
        root = np.array([3.72 - 0.08 * abs(z) * 4, 3.68, z])
        d = np.array([-1.0, 0.42, z * 0.8])
        d /= np.linalg.norm(d)
        n = np.cross(d, np.array([0.0, 0.0, 1.0]) if abs(z) < 0.1 else np.array([0.0, 0.3, 1.0]))
        n = np.cross(n, d)
        out.append((c.vane([root, root + d * L * 0.5 + np.array([0, 0.05, 0]), root + d * L + np.array([0, -0.05, 0])],
                           [0.07, 0.09, 0.03], np.array([0.0, 1.0, 0.0]), thick=0.012, lean=0.2, seed=120 + i),
                    "crest"))
    return out


def beak():
    upper = c.ringed_horn([(4.05, 3.42, 0), (4.34, 3.44, 0), (4.56, 3.33, 0), (4.66, 3.14, 0), (4.60, 3.00, 0)],
                          [0.21, 0.19, 0.14, 0.07, 0.0], ridges=2, depth=0.02, flat=1.4, sides=20,
                          ref=np.array([0.0, 1.0, 0.0]))
    lower = c.ringed_horn([(4.08, 3.22, 0), (4.36, 3.17, 0), (4.52, 3.14, 0)], [0.14, 0.09, 0.0], ridges=1,
                          depth=0.0, flat=0.85, sides=16, ref=np.array([0.0, 1.0, 0.0]))
    return [(upper, "beak"), (lower, "beak")]


def talons():
    out = []
    for s in (-1, 1):
        flip = np.array([1.0, 1.0, -s * 1.0])
        for deg, L in TOES:
            a = np.radians(deg)
            d = np.array([np.cos(a), 0.0, np.sin(a)])
            tip = np.array([0.0, 0.12, -0.6]) + d * L - np.array([0, 0.04, 0])
            big = 1.25 if deg == 180 else 1.0
            path = [tip - d * 0.08 + np.array([0, 0.05, 0]), tip + d * 0.14 * big + np.array([0, 0.07, 0]),
                    tip + d * 0.30 * big + np.array([0, 0.0, 0]), tip + d * 0.36 * big - np.array([0, 0.12, 0]),
                    tip + d * 0.32 * big - np.array([0, 0.22, 0])]
            out.append((c.ringed_horn([p * flip for p in path], [0.075 * big, 0.065 * big, 0.045, 0.022, 0.0],
                                      ridges=3, depth=0.02, flat=1.35, sides=14), "talon"))
    return out


def eyes(surf):
    out = []
    for s in (-1, 1):
        hit = surf.toward((4.02, 3.48, s * 0.6), (3.95, 3.48, 0.0))
        if hit is None:
            continue
        p, n = hit
        out.append((b.blob(p - n * 0.03, (0.10, 0.09, 0.08), n=20), "eye"))
    return out


def hard(surf):
    left = flight_feathers()
    mirrored = [((b.mirror(f)[0], b.mirror(f)[1]), k) for f, k in left]
    return left + mirrored + tail_feathers() + crest() + beak() + talons() + eyes(surf)


def paint_hard(pieces):
    cols, glows, roughs = [], [], []
    for (v, f), kind in pieces:
        v = np.asarray(v)
        s = c.along(v)
        if kind in ("flight", "tail", "covert"):
            # Barred across, dark toward the tip, a pale fringe round its
            # edge and a pale shaft down its middle.
            across = np.abs(c.across_of(len(v)))
            bars = smooth(0.35, 0.65, np.sin(s * np.pi * (7 if kind == "flight" else 6)) * 0.5 + 0.5)
            col = mixc(SLATE, SLATE_DARK, bars * 0.55 + smooth(0.6, 1.0, s) * 0.35)
            col = mixc(col, SLATE_PALE, np.maximum(smooth(0.94, 1.0, s), smooth(0.7, 1.0, across)) * 0.55)
            col = mixc(col, UNDER, smooth(0.25, 0.0, across) * 0.6)
            col = col * (0.88 + 0.24 * b.fbm(v, 6.0, 2, seed=61))[:, None]
            rough = np.full(len(v), 0.75)
        elif kind == "crest":
            col = mixc(SLATE_DARK, UNDER, smooth(0.5, 1.0, s) * 0.7)
            rough = np.full(len(v), 0.7)
        elif kind == "beak":
            col = mixc(CERE, BEAK, smooth(0.0, 0.18, s))
            col = mixc(col, BEAK_TIP, smooth(0.7, 1.0, s))
            rough = np.full(len(v), 0.35)
        elif kind == "talon":
            col = mixc(TALON, TALON_TIP, smooth(0.6, 1.0, s) * 0.5)
            rough = np.full(len(v), 0.3)
        else:
            # An iris round a black pupil, looking out and a little forward.
            centre = v.mean(0)
            out = np.array([0.45, 0.0, np.sign(centre[2])])
            out /= np.linalg.norm(out)
            dirn = (v - centre) / np.maximum(np.linalg.norm(v - centre, axis=1, keepdims=True), 1e-9)
            facing = dirn @ out
            col = mixc(np.tile(EYE, (len(v), 1)), np.array([0.02, 0.02, 0.02]), smooth(0.86, 0.92, facing))
            col = mixc(col, np.array([0.35, 0.18, 0.05]), smooth(0.5, 0.2, facing))
            rough = np.full(len(v), 0.1)
        g = np.zeros((len(v), 3))
        if kind == "eye":
            g = col * 0.35
        cols.append(col)
        glows.append(g)
        roughs.append(rough)
    return cols, glows, roughs


# ---------------------------------------------------------------------------
# 4. Detail and paint on the sculpt
# ---------------------------------------------------------------------------


def sculpt(v, t, kind):
    n = b.normals(v, t)
    x, y, z = v[:, 0], v[:, 1], v[:, 2]
    up = n[:, 1]
    az = np.abs(z)
    # Where on it: the wing (out past the shoulder), the legs, the head.
    on_wing = smooth(0.95, 1.35, az) * smooth(2.4, 2.7, y)
    on_leg = smooth(2.0, 1.6, y) * smooth(1.1, 0.8, np.abs(az - 0.6) + 0.0) * (x < 1.0)
    tarsus = on_leg * smooth(1.15, 1.0, y)
    on_head = smooth(3.35, 3.6, x)

    # -- Feathers: shingled cells, longer than wide, each rising toward its
    # tip, which lies over the base of the one behind; smaller on the head
    # and neck, larger over the back and the wing's coverts.
    size = np.where(on_wing > 0.5, 0.30, 0.22) * (1 - 0.45 * on_head)
    q = np.stack([x / 0.55, y, z], 1) / size[:, None]
    f1, f2, cid, ctr = b.cellular(q, 1.0, seed=7, jitter=0.55, centres=True)
    edge = (f2 - f1) * size
    lean = (q[:, 0] - ctr[:, 0]) * -1.0  # behind its middle: toward the tip
    feather = 0.022 * np.clip(lean / 0.5, -0.5, 1.0) * smooth(0.0, 0.03, edge)
    feather -= 0.012 * (1 - smooth(0.0, 0.025, edge))
    # Barbs: fine lines down each feather.
    barbs = np.sin((y * 0.7 + z) * 2 * np.pi / 0.025 + b.fbm(v, 2.0, 2, seed=8) * 4) * 0.0015
    plumage = (1 - tarsus)
    # The hackles: longer, ragged feathers round the base of the neck.
    hackle = smooth(1.7, 2.1, x) * smooth(3.1, 2.7, x) * smooth(2.5, 2.9, y)
    rag = (b.fbm(v * np.array([0.4, 1.0, 1.0]), 6.0, 3, seed=9) - 0.5) * 0.04 * hackle
    # The thigh's trousers: shaggy, in long strands downward.
    strands = (b.fbm(v * np.array([3.0, 0.4, 3.0]), 9.0, 2, seed=10) - 0.5) * 0.035 * on_leg * (1 - tarsus)
    # The tarsus: scutes in rows, a reptile's.
    s1, s2, _ = b.cellular(v * np.array([1.0, 2.2, 1.0]), 22.0, seed=11)
    scutes = (1 - smooth(0.0, 0.10, s2 - s1)) * tarsus
    d = plumage * (feather + barbs + rag + strands) - 0.006 * scutes + 0.004 * tarsus * smooth(0.3, 0.0, s1)
    v = v + n * d[:, None]

    # -- Paint: slate above, pale below, the line between them wavering,
    # the underside streaked; the wing roots a pale band across the back.
    lit = smooth(-0.35, 0.25, up + 0.15 * (b.fbm(v, 0.8, 3, seed=12) - 0.5))
    col = mixc(UNDER, SLATE, lit)
    streak = smooth(0.55, 0.8, b.fbm(v * np.array([0.25, 1.0, 1.0]), 6.0, 2, seed=13)) * (1 - lit)
    col = mixc(col, UNDER_STREAK, streak * 0.55)
    own = b.cell_rand(cid, 14)
    col = col * (0.88 + 0.2 * own)[:, None]
    col = mixc(col, SLATE_DARK, (1 - smooth(0.0, 0.02, edge)) * 0.6 * lit)
    col = mixc(col, SLATE_PALE, smooth(0.25, 0.6, lean) * 0.25 * lit)
    # The roots: a pale band across the back from shoulder to shoulder.
    band = smooth(1.0, 1.3, az) * smooth(3.4, 2.9, az) * smooth(0.2, 0.6, up) * smooth(-1.4, -0.9, x) \
        * smooth(1.4, 0.9, x)
    col = mixc(col, ROOT_BAND, band * 0.85)
    # The head: a dark hood, a pale stripe over each eye, the throat pale.
    hood = on_head * smooth(0.0, 0.4, up)
    col = mixc(col, SLATE_DARK, hood * 0.7)
    brow = on_head * np.exp(-(((y - 3.66) / 0.05) ** 2)) * smooth(0.12, 0.2, az) * smooth(4.15, 3.85, x)
    col = mixc(col, UNDER, brow * 0.9)
    col = mixc(col, UNDER, on_head * smooth(0.1, -0.5, up) * 0.8)
    col = mixc(col, CERE, smooth(4.0, 4.06, x) * smooth(0.2, 0.14, az))
    # The legs: the tarsus raptor-yellow, dark between its scutes.
    leg = mixc(LEG_YELLOW, LEG_SCALE, scutes * 0.8)
    col = mixc(col, leg, tarsus)
    # Grime in the folds, a little rubbed light on the edges.
    curv = b.curvature(v, t)
    col = col * (1 - 0.35 * smooth(-0.05, -0.4, curv))[:, None]
    col = mixc(col, np.minimum(col * 1.3 + 0.05, 1.0), smooth(0.08, 0.5, curv) * 0.4)
    glow = np.zeros((len(v), 3))
    rough = 0.82 * plumage + 0.5 * tarsus
    return v, col, glow, rough


# ---------------------------------------------------------------------------
# 5. Weights
# ---------------------------------------------------------------------------


def bones(sk):
    i = sk.index
    out = [
        (i["root"], (-2.0, 2.68, 0), (1.4, 2.70, 0), 0.75),
        (i["chest"], (1.4, 2.75, 0), (2.45, 2.98, 0), 0.75),
        (i["neck"], (2.45, 3.05, 0), (3.55, 3.40, 0), 0.45),
        (i["head"], (3.60, 3.42, 0), (4.85, 3.20, 0), 0.40),
        (i["tail"], (-2.05, 2.70, 0), (-4.3, 2.72, 0), 0.30),
    ]
    for side, zs in (("l", -1.0), ("r", 1.0)):
        out += [
            (i[f"shoulder.{side}"], (0.1, 3.12, 0.85 * zs), (0.1, 3.14, 3.35 * zs), 0.55),
            (i[f"arm.{side}"], (0.1, 3.14, 3.35 * zs), (-0.1, 3.14, 6.84 * zs), 0.45),
            (i[f"hand.{side}"], (-0.1, 3.12, 6.84 * zs), (-0.6, 3.1, 9.3 * zs), 0.40),
            (i[f"thigh.{side}"], (-0.2, 2.2, 0.6 * zs), (-0.1, 1.15, 0.6 * zs), 0.32),
            (i[f"shin.{side}"], (-0.1, 1.1, 0.6 * zs), (-0.05, 0.28, 0.6 * zs), 0.16),
            (i[f"foot.{side}"], (-0.05, 0.22, 0.6 * zs), (0.5, 0.05, 0.6 * zs), 0.20),
        ]
    return out


VIEWS = dict(
    views=((90, 6.0), (35, 9.0), (-10, 4.0), (150, 10.0)),
    distance=19.0,
    look=(0.0, 2.6, 0.0),
    near=((50, 0.6, (4.0, 3.4, 0)), (110, 0.4, (0.0, 0.6, -0.6)), (60, 3.0, (0.0, 3.1, -5.0)),
          (160, 1.4, (-3.0, 2.8, 0))),
    near_distance=4.5,
)


if __name__ == "__main__":
    c.make(sys.modules[__name__])
