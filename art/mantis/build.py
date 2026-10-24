"""The Mantis, sculpted and baked: the target its grown body is measured
against (``docs/design/exploration/0030_two_more_targets.md``).

    cargo run -p anim --bin export_beast -- mantis     # its skeleton, once
    art/blender.sh art/mantis/build.py [--look out.png] [--size 2048] [--no-export] [--fast]

**What it is** (``docs/design/creatures/mantis.md``): a three-metre duellist
with two scythes, upright on four walking legs, the thorax standing from
0.7 m to 2.4 m and the abdomen carried level behind. Its rest pose holds the
arms and blades straight out in front.

**What it is drawn as**: pale green chitin, the blades bone-white -- the
two things the fight is read from are the arms, so the arms are the
brightest thing on it (``game::species::mantis``). An orchid mantis pushed
toward a monster: plates of chitin parted by dark sutures and glossy where
they curve, the pronotum a long stalk rising to a flared shield, a
triangular head with two great compound eyes at its corners and antennae
sweeping back, forearms spined along their undersides, scythes serrated
along their cutting edge and hooked at the end, a leaf-like lobe on every
walking leg's femur, an abdomen in ringed segments, and veined wings.

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

NAME = "mantis"
SCALE = 0.3
TRIS = 60000

# The skeleton's joints (target/art/mantis.json).
ROOT = np.array([0.0, 1.0, 0.0])
SHOULDER = np.array([0.35, 2.2, -0.28])
ELBOW = np.array([2.05, 2.2, -0.28])
BLADE_END = 4.05
WING = np.array([-0.07, 2.15, -0.12])
# The walking legs: where each hangs from, front pair then hind.
HIPS = ((0.25, -0.42), (-0.35, -0.42))
KNEE_Y, ANKLE_Y = 0.45, 0.03

CHITIN = np.array([0.50, 0.62, 0.32])
CHITIN_DARK = np.array([0.24, 0.33, 0.15])
CHITIN_PALE = np.array([0.78, 0.84, 0.56])
SUTURE = np.array([0.08, 0.10, 0.05])
BELLY = np.array([0.70, 0.74, 0.46])
BONE = np.array([0.93, 0.91, 0.80])
BONE_DARK = np.array([0.42, 0.40, 0.34])
EYE = np.array([0.80, 0.90, 0.55])
EYE_PUPIL = np.array([0.05, 0.06, 0.04])
WING_COL = np.array([0.58, 0.68, 0.38])
WING_VEIN = np.array([0.28, 0.36, 0.16])


# ---------------------------------------------------------------------------
# 1. Forms
# ---------------------------------------------------------------------------


def thorax():
    """The lower thorax a bulb where the legs and the abdomen meet; above
    it the pronotum, a long stalk rising to a flared shield under the arms;
    the short neck."""
    lower = b.tube([(0.05, 0.72, 0), (0.12, 1.0, 0), (0.16, 1.32, 0)], [
        st(0.0, 0.14, 0.12, 0.12),
        st(0.45, 0.26, 0.24, 0.24),
        st(1.0, 0.16, 0.14, 0.14),
    ], ref=(1, 0, 0), sides=32)
    stalk = b.tube([(0.16, 1.25, 0), (0.20, 1.7, 0), (0.25, 2.1, 0), (0.28, 2.32, 0), (0.30, 2.45, 0)], [
        st(0.0, 0.13, 0.13, 0.12),
        st(0.35, 0.10, 0.10, 0.10),
        st(0.72, 0.16, 0.14, 0.12, nt=2.6),
        st(0.86, 0.24, 0.13, 0.12, nt=2.8),
        st(1.0, 0.08, 0.08, 0.08),
    ], ref=(1, 0, 0), sides=32)
    return [lower, stalk]


def head():
    """A triangle seen from the front: wide across the eyes at the top,
    narrowing to the mouthparts below."""
    face = b.tube([(0.36, 2.48, 0), (0.42, 2.66, 0), (0.46, 2.82, 0), (0.44, 2.95, 0)], [
        st(0.0, 0.07, 0.08, 0.08),
        st(0.3, 0.13, 0.12, 0.10),
        st(0.75, 0.24, 0.13, 0.11),
        st(1.0, 0.20, 0.10, 0.08),
    ], ref=(1, 0, 0), sides=32)
    eyes = [b.blob((0.43, 2.92, s * 0.17), (0.10, 0.14, 0.075),
                   axes=np.array([(1.0, 0.0, 0.0), (0.0, 1.0, s * 0.35), (0.0, -s * 0.35, 1.0)])) for s in (-1, 1)]
    return [face] + eyes


def abdomen():
    """Carried level behind, broad in the middle, tapering, a little
    raised at its end, flattened as a leaf is."""
    return b.tube([(0.0, 1.05, 0), (-0.4, 1.02, 0), (-1.1, 1.02, 0), (-1.75, 1.10, 0), (-2.08, 1.22, 0)], [
        st(0.0, 0.14, 0.12, 0.12),
        st(0.12, 0.24, 0.20, 0.20),
        st(0.45, 0.36, 0.26, 0.28, nb=1.7),
        st(0.78, 0.28, 0.20, 0.22),
        st(1.0, 0.05, 0.05, 0.05),
    ], sides=40)


def arm():
    """The left raptorial forelimb: the coxa and femur as one long segment
    straight out in front from the shoulder, deeper than it is wide; the
    blade's own core (the scythe is a hard piece on it)."""
    femur = b.tube([SHOULDER + (-0.05, 0, 0.03), SHOULDER + (0.6, 0.02, 0), SHOULDER + (1.3, 0.0, 0), ELBOW], [
        st(0.0, 0.10, 0.12, 0.12),
        st(0.2, 0.09, 0.14, 0.16),
        st(0.7, 0.085, 0.13, 0.15),
        st(1.0, 0.07, 0.09, 0.09),
    ], sides=28)
    core = b.tube([ELBOW, ELBOW + (0.8, -0.01, 0), (BLADE_END - 0.25, 2.18, ELBOW[2])], [
        st(0.0, 0.06, 0.07, 0.07),
        st(0.5, 0.045, 0.06, 0.05),
        st(1.0, 0.025, 0.03, 0.03),
    ], sides=20)
    return [femur, core]


def walking_leg(x, z):
    """A walking leg straight down from the hip: a femur, a thin tibia,
    and the tarsus lying flat on the floor from the ankle, beaded."""
    femur = b.tube([(x, 1.02, z), (x, 0.75, z), (x, KNEE_Y, z)], [
        st(0.0, 0.065, 0.065, 0.065),
        st(0.5, 0.055, 0.06, 0.06),
        st(1.0, 0.045, 0.045, 0.045),
    ], ref=(1, 0, 0), sides=20)
    tibia = b.tube([(x, KNEE_Y, z), (x, 0.25, z), (x, ANKLE_Y + 0.02, z)], [
        st(0.0, 0.042, 0.042, 0.042),
        st(1.0, 0.03, 0.03, 0.03),
    ], ref=(1, 0, 0), sides=16)
    coxa = b.tube([(x + 0.02, 1.02, z * 0.3), (x, 1.04, z * 0.75), (x, 1.0, z)], [
        st(0.0, 0.08, 0.08, 0.08),
        st(1.0, 0.07, 0.07, 0.07),
    ], sides=20)
    knee = b.blob((x, KNEE_Y, z), (0.055, 0.06, 0.055), n=16)
    out = [coxa, femur, tibia, knee]
    # The tarsus: five beads out along the floor, forward and out.
    d = np.array([0.55, 0.0, np.sign(z) * 0.84])
    d /= np.linalg.norm(d)
    for k in range(5):
        p = np.array([x, ANKLE_Y + 0.02, z]) + d * (0.04 + 0.055 * k)
        out.append(b.blob(p, (0.03 - 0.003 * k,) * 3, n=12))
    return out


def forms():
    left = arm()
    for x, z in HIPS:
        left += walking_leg(x, z)
    return thorax() + head() + [abdomen()] + both(left)


def shells(v, t):
    return None


# ---------------------------------------------------------------------------
# 2. Hard pieces
# ---------------------------------------------------------------------------


def scythe(side):
    """The blade: a flat scythe in the vertical plane, deepest a third of
    the way out, its cutting edge below, serrated, and the end hooked
    down; bone-white. And the spines along it."""
    z = ELBOW[2] if side < 0 else -ELBOW[2]
    path = [(ELBOW[0] + 0.1, 2.22, z), (2.9, 2.24, z), (3.6, 2.21, z), (3.95, 2.12, z), (4.10, 1.94, z)]
    out = [(c.vane(path, [0.07, 0.11, 0.09, 0.05, 0.012], (0, 0, 1), thick=0.02, lean=0.55, notch=0.0,
                   rings=48), "blade")]
    # The serration: small teeth down the cutting edge.
    for k in range(14):
        x = ELBOW[0] + 0.2 + k * 0.13
        y = 2.20 - 0.16 * smooth(0.25, 0.0, abs((x - 2.9) / 1.6)) - 0.05
        base = np.array([x, y + 0.03, z])
        out.append((c.ringed_horn([base, base + (0.02, -0.06, 0), base + (0.05, -0.10, 0)], [0.014, 0.009, 0.0],
                                  ridges=1, depth=0.0, sides=8, rings=6), "spine"))
    return out


def arm_spines(side):
    """Two rows of spines down the underside of the femur, longer toward
    the elbow, raking forward: what it grips with."""
    out = []
    z0 = ELBOW[2] if side < 0 else -ELBOW[2]
    for row in (-1, 1):
        for k in range(9):
            x = SHOULDER[0] + 0.45 + k * 0.14
            L = 0.07 + 0.09 * k / 8
            base = np.array([x, 2.08, z0 + row * 0.05])
            d = np.array([0.45, -1.0, row * 0.25])
            d /= np.linalg.norm(d)
            out.append((c.ringed_horn([base + d * 0.02 * -1, base + d * L * 0.6, base + d * L], [0.018, 0.011, 0.0],
                                      ridges=2, depth=0.03, sides=8), "spine"))
    return out


def antennae():
    out = []
    for s in (-1, 1):
        root = np.array([0.52, 2.92, s * 0.07])
        path = [root, root + (0.2, 0.25, s * 0.08), root + (0.0, 0.62, s * 0.22), root + (-0.5, 0.86, s * 0.34),
                root + (-1.0, 0.9, s * 0.42)]
        out.append((c.ringed_horn(path, [0.012, 0.009, 0.007, 0.005, 0.002], ridges=20, depth=0.15, sides=6),
                    "antenna"))
    return out


def mouthparts():
    out = []
    for s in (-1, 1):
        base = np.array([0.48, 2.54, s * 0.05])
        out.append((c.ringed_horn([base, base + (0.06, -0.06, -s * 0.02), base + (0.04, -0.12, -s * 0.05)],
                                  [0.025, 0.016, 0.0], ridges=1, depth=0.0, sides=10), "claw"))
    return out


def wings():
    """The forewings: leathery, veined, lying back from the thorax on their
    bones."""
    out = []
    for s in (-1, 1):
        root = np.array([WING[0], WING[1], s * -WING[2]])
        path = [root, root + (-0.5, 0.02, s * 0.12), root + (-1.0, 0.0, s * 0.22), root + (-1.32, -0.02, s * 0.30)]
        out.append((c.vane(path, [0.05, 0.2, 0.2, 0.06], (0, 1, 0), thick=0.008, lean=0.25 * -s, rings=40),
                    "wing"))
    return out


def lobes():
    """A leaf-like lobe on the back of every walking leg's femur: the
    orchid mantis's petals, and the silhouette's flair."""
    out = []
    for x, z in HIPS:
        for s in (-1, 1):
            zz = -z if s > 0 else z
            root = np.array([x - 0.05, 0.98, zz])
            path = [root, root + (-0.06, -0.25, 0.0), root + (-0.04, -0.47, 0.0)]
            out.append((c.vane(path, [0.03, 0.10, 0.02], (0, 0, 1), thick=0.008, lean=-0.9, rings=20), "lobe"))
    return out


def tarsal_claws():
    out = []
    for x, z in HIPS:
        for s in (-1, 1):
            zz = -z if s > 0 else z
            d = np.array([0.55, 0.0, np.sign(zz) * 0.84])
            d /= np.linalg.norm(d)
            tip = np.array([x, ANKLE_Y + 0.02, zz]) + d * 0.27
            for side in (-1, 1):
                off = np.cross(d, (0, 1, 0)) * 0.012 * side
                out.append((c.ringed_horn([tip + off, tip + off + d * 0.04, tip + off + d * 0.06 - (0, 0.03, 0)],
                                          [0.008, 0.005, 0.0], ridges=1, depth=0.0, sides=6, rings=6), "claw"))
    return out


def pronotum_spikes():
    """Short spikes along the shield's edge under the arms."""
    out = []
    for s in (-1, 1):
        for k, (y, L) in enumerate(((2.02, 0.05), (2.10, 0.07), (2.18, 0.08))):
            base = np.array([0.24 + 0.02 * k, y, s * 0.20])
            d = np.array([-0.2, 0.3, s * 1.0])
            d /= np.linalg.norm(d)
            out.append((c.ringed_horn([base, base + d * L], [0.014, 0.0], ridges=1, depth=0.0, sides=8, rings=6),
                        "spine"))
    return out


def hood():
    """Leaf-like flanges either side of the pronotum, flaring under the arms:
    the hood a leaf mantis wears, and what makes the neck a silhouette."""
    out = []
    for s in (-1, 1):
        for k, (y0, y1, w) in enumerate(((1.55, 2.0, 0.10), (1.9, 2.28, 0.14))):
            root = np.array([0.22, y0, s * 0.07])
            path = [root, root + (0.03, (y1 - y0) * 0.5, s * 0.02), np.array([0.27, y1, s * 0.09])]
            out.append((c.vane(path, [0.02, w, 0.02], (1, 0, 0), thick=0.006, lean=s * 0.9, rings=16), "lobe"))
    return out


def ocelli():
    return [(b.blob((0.50, 2.97, s * 0.04), (0.018, 0.018, 0.018), n=10), "ocellus") for s in (-1, 0, 1)]


def hard(surf):
    out = []
    for side in (-1, 1):
        out += scythe(side) + arm_spines(side)
    return out + antennae() + mouthparts() + wings() + lobes() + tarsal_claws() + pronotum_spikes() + hood() + ocelli()


def paint_hard(pieces):
    cols, glows, roughs = [], [], []
    for (v, f), kind in pieces:
        v = np.asarray(v)
        s = c.along(v)
        g = np.zeros((len(v), 3))
        if kind == "blade":
            # Bone-white, the edge below greyed and polished, the back a
            # little green where it grows from the arm.
            y = v[:, 1]
            col = mixc(BONE, BONE_DARK, smooth(2.12, 2.0, y) * 0.5)
            col = mixc(col, CHITIN_PALE, smooth(0.12, 0.0, s) * 0.6)
            col = col * (0.9 + 0.15 * b.fbm(v, 30.0, 2, seed=70))[:, None]
            rough = np.full(len(v), 0.45)
            g = np.tile(BONE * 0.04, (len(v), 1))
        elif kind == "spine":
            col = mixc(BONE * 0.9, SUTURE * 2.5, smooth(0.5, 1.0, s))
            rough = np.full(len(v), 0.35)
        elif kind == "wing":
            # Veins: a network of dark lines on the leathery green.
            f1, f2, _ = b.cellular(v * np.array([0.35, 1.0, 1.0]), 22.0, seed=71)
            vein = 1 - smooth(0.0, 0.07, f2 - f1)
            long_vein = np.exp(-((np.sin(v[:, 2] * 60 + v[:, 0] * 8) ** 2) / 0.02))
            col = mixc(WING_COL, WING_VEIN, np.maximum(vein, long_vein * 0.8) * 0.8)
            col = mixc(col, CHITIN_PALE, smooth(0.85, 1.0, s) * 0.4)
            rough = np.full(len(v), 0.6)
        elif kind == "lobe":
            col = mixc(CHITIN_PALE, CHITIN, smooth(0.0, 0.8, s) * 0.5)
            f1, f2, _ = b.cellular(v, 40.0, seed=72)
            col = mixc(col, CHITIN_DARK, (1 - smooth(0.0, 0.06, f2 - f1)) * 0.5)
            rough = np.full(len(v), 0.5)
        elif kind == "antenna":
            col = mixc(CHITIN_DARK, SUTURE, smooth(0.3, 1.0, s))
            rough = np.full(len(v), 0.5)
        elif kind == "ocellus":
            col = np.tile(np.array([0.30, 0.34, 0.16]), (len(v), 1))
            rough = np.full(len(v), 0.15)
        else:
            col = mixc(CHITIN_DARK, SUTURE, s)
            rough = np.full(len(v), 0.35)
        cols.append(col)
        glows.append(g)
        roughs.append(rough)
    return cols, glows, roughs


# ---------------------------------------------------------------------------
# 3. Detail and paint on the sculpt
# ---------------------------------------------------------------------------


def sculpt(v, t, kind):
    n = b.normals(v, t)
    x, y, z = v[:, 0], v[:, 1], v[:, 2]
    up = n[:, 1]
    az = np.abs(z)
    on_abdomen = smooth(-0.1, -0.25, x) * smooth(1.6, 1.4, y)
    on_eye = np.zeros(len(v))
    for s in (-1, 1):
        d = ((x - 0.44) / 0.13) ** 2 + ((y - 2.90) / 0.14) ** 2 + ((z - s * 0.19) / 0.11) ** 2
        on_eye = np.maximum(on_eye, smooth(1.15, 0.85, d))
    on_head = smooth(2.45, 2.55, y) * smooth(0.25, 0.32, x)
    on_arm = smooth(2.05, 2.12, y) * smooth(2.32, 2.28, y) * smooth(0.45, 0.6, x) * smooth(0.18, 0.22, az)
    on_leg = smooth(0.95, 0.85, y) * smooth(0.3, 0.36, az)

    # -- Segments and sutures: the abdomen's rings, the legs' joints, a
    # suture down the middle of the back and round the shield's edge.
    ring = np.abs(((x + 0.02 * np.sin(z * 20)) / 0.19) % 1.0 - 0.5) * 2
    tergite = (1 - smooth(0.0, 0.12, ring)) * on_abdomen
    overlap = np.clip(((x / 0.19) % 1.0), 0, 1) * on_abdomen  # each segment rising toward its back edge
    midline = np.exp(-(z / 0.012) ** 2) * smooth(0.0, 0.6, up) * (1 - on_eye)
    # Plates: a few broad plates on the chitin, their seams sutures --
    # sparse, as an insect's sclerites are, not a crazing.
    p1, p2, cid = b.cellular(v * np.array([1.0, 0.35, 1.0]), 4.0, seed=21)
    plate_seam = (1 - smooth(0.0, 0.02, p2 - p1)) * (1 - on_abdomen)
    # Carinae: fine ridges running down each limb and the stalk.
    carina = np.abs(np.sin(np.arctan2(z - np.round(z / 0.42) * 0.42, x) * 3.0)) ** 8 * (on_leg + on_arm) * 0.6
    # Fine surface: pitting and a grain along each segment.
    pits = smooth(0.25, 0.0, b.cellular(v, 140.0, seed=22)[0]) * 0.6
    grain = b.fbm(v * np.array([0.3, 1.0, 1.0]), 60.0, 2, seed=23) - 0.5
    # The compound eyes: hexagonal facets.
    f1, f2, _ = b.cellular(v, 260.0, seed=24, jitter=0.3)
    facet = (1 - smooth(0.0, 0.12, f2 - f1)) * on_eye
    d = (-0.010 * tergite + 0.008 * overlap - 0.004 * midline - 0.003 * plate_seam * (1 - on_eye)
         - 0.0004 * pits + 0.0003 * grain - 0.0012 * facet + 0.002 * carina)
    v = v + n * d[:, None]

    # -- Paint: pale green chitin, darker down the sides and on the
    # underside's segments, pale on the rises, dark in every suture.
    shade = b.fbm(v, 1.2, 2, seed=25)
    col = mixc(CHITIN, CHITIN_DARK, smooth(0.3, -0.6, up) * 0.35 + (0.5 - shade).clip(0, 1) * 0.15)
    col = mixc(col, BELLY, smooth(-0.3, -0.8, up) * 0.6)
    col = mixc(col, CHITIN_PALE, smooth(0.5, 0.9, up) * 0.3 + overlap * 0.25)
    own = b.cell_rand(cid, 26)
    col = col * (0.95 + 0.08 * own)[:, None]
    col = mixc(col, SUTURE, np.clip(tergite * 0.75 + plate_seam * 0.5 * (1 - on_eye) + midline * 0.4, 0, 1))
    # A leaf's veining, faint, and a few dead-leaf flecks.
    vf1, vf2, _ = b.cellular(v * np.array([0.5, 1.0, 1.0]), 18.0, seed=27)
    veins = (1 - smooth(0.0, 0.015, vf2 - vf1)) * 0.18
    col = mixc(col, CHITIN_DARK, veins)
    flecks = smooth(0.78, 0.8, b.fbm(v, 10.0, 3, seed=28))
    col = mixc(col, np.array([0.58, 0.46, 0.26]), flecks * 0.25)
    # Every joint darker, where the chitin turns to membrane.
    for jy in (KNEE_Y, 1.0, 2.2, 2.42):
        col = mixc(col, CHITIN_DARK, np.exp(-((y - jy) / 0.03) ** 2) * 0.45 * (on_leg + on_arm + (az < 0.2)).clip(0, 1))
    # The arms the brightest thing on it: paling toward the elbow.
    col = mixc(col, CHITIN_PALE, on_arm * smooth(0.6, 1.9, x) * 0.6)
    # The eyes: glassy and dark, deep green going to black at their
    # backs, faceted, with a dull ember low in them -- a monster's, not a
    # cartoon's: no pupil looking out.
    eye_col = mixc(np.array([0.36, 0.44, 0.20]), np.array([0.06, 0.08, 0.04]), smooth(0.5, -0.4, n[:, 0]) * 0.8)
    eye_col = mixc(eye_col, np.array([0.02, 0.03, 0.01]), facet * 0.6)
    col = mixc(col, eye_col, on_eye)
    curv = b.curvature(v, t)
    # Only the broad bends: chitin's fine grain is gloss, not grime.
    col = col * (1 - 0.25 * smooth(-0.2, -0.8, curv))[:, None]
    col = mixc(col, np.minimum(col * 1.2 + 0.05, 1.0), smooth(0.3, 0.9, curv) * 0.25)
    glow = np.zeros((len(v), 3))
    # Chitin is glossy; the eyes glossier.
    rough = 0.38 - 0.18 * on_eye + 0.1 * pits + 0.0 * on_head
    return v, col, glow, rough


# ---------------------------------------------------------------------------
# 4. Weights
# ---------------------------------------------------------------------------


def bones(sk):
    i = sk.index
    out = [
        (i["root"], (0.05, 0.75, 0), (0.15, 1.30, 0), 0.20),
        (i["abdomen"], (-0.15, 1.05, 0), (-2.1, 1.2, 0), 0.24),
        (i["thorax"], (0.18, 1.35, 0), (0.28, 2.35, 0), 0.12),
        (i["neck"], (0.29, 2.38, 0), (0.33, 2.52, 0), 0.06),
        (i["head"], (0.36, 2.52, 0), (0.46, 3.0, 0), 0.16),
    ]
    for side, zs in (("l", -1.0), ("r", 1.0)):
        out += [
            (i[f"arm.{side}"], (0.35, 2.2, 0.28 * zs), (2.0, 2.2, 0.28 * zs), 0.12),
            (i[f"blade.{side}"], (2.1, 2.2, 0.28 * zs), (4.05, 2.1, 0.28 * zs), 0.08),
            (i[f"wing.{side}"], (-0.07, 2.15, 0.12 * zs), (-1.37, 2.15, 0.4 * zs), 0.05),
        ]
        for leg, (x, z) in zip(("m", "h"), HIPS):
            zz = abs(z) * zs
            out += [
                (i[f"femur.{leg}{side}"], (x, 1.0, zz), (x, KNEE_Y + 0.02, zz), 0.07),
                (i[f"tibia.{leg}{side}"], (x, KNEE_Y - 0.02, zz), (x, ANKLE_Y + 0.06, zz), 0.05),
                (i[f"tarsus.{leg}{side}"], (x, ANKLE_Y + 0.02, zz),
                 (x + 0.15, ANKLE_Y + 0.02, zz + 0.22 * zs), 0.05),
            ]
    return out


VIEWS = dict(
    views=((70, 1.2), (30, 2.2), (-20, 1.0), (150, 2.5)),
    distance=6.5,
    look=(0.6, 1.5, 0.0),
    near=((40, 0.2, (0.45, 2.8, 0)), (90, 0.3, (3.0, 2.15, -0.28)), (60, 0.4, (0.25, 0.3, -0.42)),
          (150, 0.6, (-1.0, 1.2, 0))),
    near_distance=1.6,
)


if __name__ == "__main__":
    c.make(sys.modules[__name__])
