"""Muscle, sculpted by script.

MakeHuman's body has the shape of a person and none of the anatomy a game
character is sculpted with: at full muscle its chest, arms and back are smooth
masses, and painted dark they read as a skin suit (the owner, 2026-10-26).
What a character sculptor adds is the muscles themselves -- each a mass with a
rounded top, lying where anatomy puts it, and **the grooves between them**,
which are what the eye reads as "muscular": the line under the pectorals, the
deltoid's edges, the abdominal grid, the serratus' fingers, the lats' sweep
(the Belvedere Torso, the Farnese Hercules).

So a physique is carved the way a sculptor carves one, in two passes on a
map of the body (the trunk by angle round its middle and height from the hips
to the shoulders, each limb by angle round its bone and distance along it):

- **masses**: broad, soft swellings where the big muscles are -- the chest,
  the lats, the traps, the deltoid, the biceps -- each a rounded rectangle
  rather than a ball, the highest under each point taken, never their sum;
- **separations**: grooves cut along the lines where muscles meet --
  the lower border of the chest, the line down the abdomen and its three
  bands, the abdomen's outer edge, the serratus' slits, the lats' edge, the
  shoulder blades, the spine, between the deltoid's heads and between
  biceps and triceps. These are what the eye reads as definition; masses
  alone read as lumps stuck on (the first sculpt). ``sculpt`` returns the
push, in metres, for every vertex; ``character.sculpt`` puts a share of it
into the game's mesh and all of it into the high body the normal map is
baked from.

Everything is in the game's frame (``+y`` up, ``+z`` forward) and mirrored:
a muscle is written for the side at ``+x`` and drawn on both.
"""

import numpy as np

import garment as g

J = {n: i for i, n in enumerate(g.JOINTS)}

# How many metres a degree round the trunk, and a unit of its height, are:
# the trunk's own scale, so a groove's width is written in centimetres.
TRUNK_R = 0.15
LIMB_R = {"upper": 0.055, "fore": 0.045, "thigh": 0.085, "shin": 0.055}

# --- The trunk ---------------------------------------------------------------
# Angles are degrees round the trunk's middle from the front, toward +x
# (mirrored); heights are 0 at the hips and 1 at the shoulders. A point 4 cm
# off the middle of the abdomen is about 20 degrees round, a pectoral's
# middle about 40.
#
# Masses: (name, angle, half-width deg, height, half-height, metres out,
# squareness -- 2 is an ellipse, 4 a rounded rectangle).
TRUNK = [
    # Fading up into the collarbone rather than stopping: squarer, its
    # top edge stood as a ledge across the chest.
    ("pectoral", 36.0, 42.0, 0.78, 0.25, 0.028, 2.0),
    ("pectoral_outer", 58.0, 14.0, 0.84, 0.10, 0.006, 2.2),
    ("abs_1", 20.0, 19.0, 0.655, 0.06, 0.010, 4.0),
    ("abs_2", 20.0, 20.0, 0.53, 0.06, 0.011, 4.0),
    ("abs_3", 20.0, 20.0, 0.405, 0.06, 0.011, 4.0),
    ("abs_low", 18.0, 18.0, 0.25, 0.09, 0.007, 3.0),
    ("oblique", 64.0, 26.0, 0.26, 0.15, 0.016, 2.6),
    ("serratus", 72.0, 12.0, 0.65, 0.12, 0.006, 3.0),
    ("lat", 112.0, 34.0, 0.66, 0.26, 0.022, 3.0),
    # Modest: big traps, but a man's -- at more they rose into a hump that
    # read as nothing human (the owner, 2026-10-26).
    ("trap_upper", 140.0, 46.0, 1.04, 0.1, 0.015, 2.2),
    ("trap_front", 85.0, 30.0, 1.04, 0.08, 0.008, 2.2),
    ("trap_mid", 168.0, 14.0, 0.84, 0.16, 0.009, 3.0),
    ("scapula", 138.0, 20.0, 0.84, 0.11, 0.010, 3.0),
    ("erector", 166.0, 10.0, 0.36, 0.26, 0.011, 3.0),
    ("glute", 150.0, 32.0, -0.10, 0.20, 0.016, 2.5),
]

# The traps' rise at the base of the neck, metres, and how far out from the
# middle the neck is: from there the slope falls to the point of the
# shoulder (``sculpt``).
TRAP_RISE = 0.034
TRAP_NECK = 0.06

# Separations: (polyline of (angle, height), half-width in metres, depth).
TRUNK_GROOVES = [
    # Down the middle of the abdomen, and its three bands.
    ([(0, 0.16), (0, 0.76)], 0.006, 0.006),
    ([(0, 0.592), (19, 0.586), (37, 0.60)], 0.005, 0.005),
    ([(0, 0.468), (20, 0.462), (38, 0.475)], 0.005, 0.005),
    ([(0, 0.342), (19, 0.338), (37, 0.35)], 0.005, 0.0045),
    # The abdomen's outer edge.
    ([(37, 0.72), (41, 0.50), (36, 0.30), (24, 0.14)], 0.006, 0.0045),
    # The lower border of the chest, sweeping up into the armpit.
    # Soft, as the classical chest is: deeper, it read as a shelf.
    ([(3, 0.645), (22, 0.62), (42, 0.63), (58, 0.69), (68, 0.86)], 0.014, 0.008),
    # The collarbone's line, and the groove between chest and shoulder.
    ([(10, 1.02), (30, 1.03), (48, 1.02)], 0.008, 0.002),
    ([(50, 1.02), (60, 0.92), (66, 0.84)], 0.006, 0.006),
    # The serratus' slits, and the obliques' upper border.
    ([(64, 0.70), (78, 0.73)], 0.004, 0.004),
    ([(63, 0.62), (77, 0.65)], 0.004, 0.004),
    ([(62, 0.54), (75, 0.57)], 0.004, 0.0035),
    ([(41, 0.46), (62, 0.50), (80, 0.48), (98, 0.42)], 0.006, 0.004),
    # The groove from the hip bone down to the groin, under the obliques'
    # swell, and down the sternum between the pectorals.
    ([(66, 0.16), (48, 0.06), (22, -0.04)], 0.008, 0.006),
    ([(0, 0.66), (0, 0.98)], 0.008, 0.004),
    # The back: the spine, the lats' edge, the shoulder blade, the traps'.
    ([(180, 0.10), (180, 1.02)], 0.011, 0.005),
    ([(96, 0.86), (106, 0.62), (124, 0.40), (150, 0.26), (176, 0.20)], 0.007, 0.0055),
    ([(124, 0.92), (120, 0.80), (132, 0.70), (152, 0.72), (162, 0.84), (156, 0.96)], 0.005, 0.0045),
    ([(180, 0.62), (160, 0.80), (140, 0.97)], 0.006, 0.004),
    ([(172, 0.12), (166, 0.62)], 0.005, 0.003),
]

# --- The limbs ---------------------------------------------------------------
# Angles are degrees round the bone from the front, toward the outside;
# distances along it are 0 at its root joint and 1 at the next.
#
# Masses: (name, limb, angle, half-width deg, along, half-length, metres
# out, squareness). Limbs: upper (shoulder to elbow), fore (elbow to wrist),
# thigh, shin.
LIMB = [
    ("deltoid", "upper", 60.0, 125.0, 0.18, 0.3, 0.022, 2.2),
    ("biceps", "upper", -5.0, 52.0, 0.56, 0.34, 0.026, 2.3),
    ("triceps", "upper", 175.0, 60.0, 0.46, 0.36, 0.024, 2.3),
    ("brachioradialis", "fore", 55.0, 42.0, 0.20, 0.34, 0.018, 2.3),
    ("flexors", "fore", -45.0, 60.0, 0.28, 0.34, 0.015, 2.3),
    ("extensors", "fore", 150.0, 50.0, 0.28, 0.34, 0.013, 2.3),
    ("quads", "thigh", 15.0, 85.0, 0.50, 0.40, 0.014, 2.5),
    ("hamstrings", "thigh", 180.0, 55.0, 0.50, 0.40, 0.011, 2.5),
    ("calf", "shin", 180.0, 70.0, 0.28, 0.24, 0.014, 2.5),
]

# Separations on the limbs: (limb, polyline of (angle, along), half-width
# in metres, depth).
LIMB_GROOVES = [
    # Between the deltoid's three heads, and its lower edge.
    ("upper", [(15, -0.05), (20, 0.30)], 0.005, 0.005),
    ("upper", [(115, -0.05), (110, 0.30)], 0.005, 0.0045),
    # Soft: a hard edge here read as a ball stuck on the arm (2026-10-26).
    ("upper", [(-30, 0.30), (60, 0.42), (150, 0.30)], 0.011, 0.0035),
    # Between biceps and triceps, outside and in.
    ("upper", [(85, 0.40), (90, 0.90)], 0.006, 0.006),
    ("upper", [(-95, 0.35), (-90, 0.90)], 0.006, 0.005),
    # The forearm's top line.
    ("fore", [(15, 0.05), (25, 0.55)], 0.005, 0.004),
    # The thigh's sweep and teardrop.
    ("thigh", [(75, 0.15), (80, 0.85)], 0.006, 0.005),
    ("thigh", [(-55, 0.55), (-30, 0.92)], 0.006, 0.005),
    # The calf's two heads.
    ("shin", [(180, 0.05), (180, 0.40)], 0.005, 0.004),
]

LIMBS = {"upper": ("arm", "forearm"), "fore": ("forearm", "hand"),
         "thigh": ("thigh", "shin"), "shin": ("shin", "foot")}


def _angle(a, b):
    """The difference of two angles in degrees, wrapped to -180..180."""
    return (a - b + 180.0) % 360.0 - 180.0


def _mass(dx, dy, square):
    """A muscle's swelling at normalised offsets: one at its middle, nought
    at its edge, a rounded rectangle when ``square`` is high, with soft
    shoulders."""
    d = (np.abs(dx) ** square + np.abs(dy) ** square) ** (1.0 / square)
    x = np.clip(1.0 - d, 0.0, 1.0)
    return x * x * (3.0 - 2.0 * x)


def _groove(u, v, line, half):
    """How deep in a groove a point is, 0 to 1: a gaussian across the
    distance from ``(u, v)`` (metres) to a polyline (metres)."""
    best = np.full(len(u), 1e9)
    for (a0, b0), (a1, b1) in zip(line[:-1], line[1:]):
        du, dv = a1 - a0, b1 - b0
        ll = du * du + dv * dv
        t = np.clip(((u - a0) * du + (v - b0) * dv) / max(ll, 1e-12), 0.0, 1.0)
        d2 = (u - a0 - t * du) ** 2 + (v - b0 - t * dv) ** 2
        best = np.minimum(best, d2)
    return np.exp(-best / (half * half))


def sculpt(p, W, origin, strength=1.0):
    """How far to push each vertex out along its normal, in metres.

    ``p``: vertices, ``(n, 3)``, in the game's frame; ``W``: their joint
    weights, ``(n, len(garment.JOINTS))``; ``origin``: the bind pose's joint
    origins, in ``garment.JOINTS`` order. The mesh has to be dense -- a
    face a centimetre or so across -- for a groove to have a shape."""
    o = {n: np.asarray(origin[i], float) for n, i in J.items()}

    def wsum(*names):
        return sum(W[:, J[n]] for n in names)

    # Every vertex is sculpted as trunk and as each limb near it, and the
    # results mixed by how much of it each part carries: a hard choice of
    # one left a seam where the shoulder meets the chest.
    parts = []
    trunk_w = wsum("root", "spine", "chest", "clavicle.l", "clavicle.r")
    t = trunk_w > 0.05
    if t.any():
        hip = (o["thigh.l"][1] + o["thigh.r"][1]) / 2
        sh = (o["arm.l"][1] + o["arm.r"][1]) / 2
        span = sh - hip
        q = p[t]
        h = (q[:, 1] - hip) / span
        core = trunk_w[t] > 0.6
        bins = np.round(h * 20).astype(int)
        centre = np.zeros(len(q))
        for b in np.unique(bins):
            m = bins == b
            ref = m & core if (m & core).any() else m
            centre[m] = (q[ref, 2].max() + q[ref, 2].min()) / 2
        ang = np.degrees(np.arctan2(np.abs(q[:, 0]), q[:, 2] - centre))
        u, v = np.radians(ang) * TRUNK_R, h * span
        best = np.zeros(len(q))
        for _, a, wa, hh, wh, amp, sq in TRUNK:
            best = np.maximum(best, amp * _mass(_angle(ang, a) / wa, (h - hh) / wh, sq))
        # The traps' slope: highest at the base of the neck and falling to
        # nothing at the point of the shoulder, so the line from neck to
        # shoulder runs down as a man's does rather than level, a yoke (the
        # owner, 2026-10-26: "the shoulders and traps are super weird").
        # It is across the trunk, not round it, which the masses above
        # cannot say.
        reach = abs(o["arm.l"][0]) - 0.03
        lateral = np.abs(q[:, 0])
        k = (lateral - TRAP_NECK) / max(reach - TRAP_NECK, 1e-6)
        k = np.clip(k, 0.0, 1.0)
        slope = TRAP_RISE * (1 - k * k * (3 - 2 * k))
        hk = np.clip((h - 0.82) / 0.16, 0.0, 1.0) * np.clip((1.3 - h) / 0.1, 0.0, 1.0)
        best = np.maximum(best, slope * hk * hk * (3 - 2 * hk))
        for line, half, depth in TRUNK_GROOVES:
            pts = [(np.radians(a) * TRUNK_R, b * span) for a, b in line]
            best -= depth * _groove(u, v, pts, half)
        full = np.zeros(len(p))
        full[t] = best
        parts.append((trunk_w, full))

    for s in "lr":
        outward = np.sign(o[f"arm.{s}"][0]) or 1.0
        for limb, (a, b) in LIMBS.items():
            lw = W[:, J[f"{a}.{s}"]]
            m = lw > 0.05
            if not m.any():
                continue
            root, tip = o[f"{a}.{s}"], o[f"{b}.{s}"]
            axis = tip - root
            length = np.linalg.norm(axis)
            axis = axis / length
            front = np.array([0.0, 0.0, 1.0]) - axis * axis[2]
            front /= np.linalg.norm(front)
            side = np.cross(axis, front)
            if np.dot(side, [outward, 0.0, 0.0]) < 0:
                side = -side
            q = p[m] - root
            along = q @ axis / length
            ang = np.degrees(np.arctan2(q @ side, q @ front))
            r = LIMB_R[limb]
            u, v = np.radians(ang) * r, along * length
            best = np.zeros(m.sum())
            for _, lb, a0, wa, t0, wt, amp, sq in LIMB:
                if lb == limb:
                    best = np.maximum(best, amp * _mass(_angle(ang, a0) / wa, (along - t0) / wt, sq))
            for lb, line, half, depth in LIMB_GROOVES:
                if lb == limb:
                    pts = [(np.radians(x) * r, y * length) for x, y in line]
                    best -= depth * _groove(u, v, pts, half)
            full = np.zeros(len(p))
            full[m] = best
            parts.append((lw, full))

    total = sum(wt for wt, _ in parts)
    push = sum(wt * f for wt, f in parts) / np.maximum(total, 1e-6)
    # Nothing on the head, the hands or the feet.
    # Faded out smoothly toward them: a push cut off as steeply as the
    # weights change at the neck spiked there.
    fade = np.clip(total, 0.0, 1.0)
    push *= fade * fade * (3.0 - 2.0 * fade)
    return push * strength


def relax(push, edges, rounds=3):
    """Even a push out over a mesh's edges (``edges``: an ``(m, 2)`` array):
    where the trunk's masses meet the neck the push changes faster than
    the mesh can follow, and it spikes."""
    n = len(push)
    a, b = edges[:, 0], edges[:, 1]
    deg = np.bincount(a, minlength=n) + np.bincount(b, minlength=n)
    for _ in range(rounds):
        acc = np.bincount(a, weights=push[b], minlength=n) + np.bincount(b, weights=push[a], minlength=n)
        push = 0.5 * push + 0.5 * acc / np.maximum(deg, 1)
    return push


def mesh_edges(me):
    e = np.zeros(len(me.edges) * 2, dtype=np.int64)
    me.edges.foreach_get("vertices", e)
    return e.reshape(-1, 2)
