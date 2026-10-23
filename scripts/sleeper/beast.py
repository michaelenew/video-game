"""The Sleeper, built: a giant armoured lizard, as a skeleton and the flesh
on it.

The owner (2026-10-23), after a cartoon dragon's mesh had been bent piece
by piece into something "goofy": "figure out a better way to make this
look like a big threatening terrestrial lizard boss the size of a
mountain". So this is built the way a creature is designed rather than
the way a mesh is patched -- from what the giant reptiles of reference art
share (the tarrasque, Godzilla, a tyrannosaur's stance and an
ankylosaur's armour and club):

- **A skeleton with fixed proportions**: chains of bones -- spine, neck,
  head, tail, two legs, two arms -- each a length. A pose is only which way
  each bone points, so the beast standing and the beast asleep are the
  same animal.
- **Flesh as volumes on the bones**: each bone a tapering capsule, plus the
  masses that make an animal heavy (a barrel chest, a belly, thighs like
  drums, a skull with a jaw) -- all blended smoothly where they meet, the
  way muscle meets muscle.
- **Armour that stays crisp**: blade spikes down the spine, plates either
  side of it, horns, claws and a club of bone, blended in only a little.

The volumes are a signed distance field -- for every point, how far it is
from the skin, negative inside -- and the skin is where that is zero,
found by marching cubes (scikit-image). Units are model units: the beast is
about eighteen long; `terrain.py` sizes it for the map.

x is the beast's left, y up, z the way it faces.
"""
import numpy as np

# ---------------------------------------------------------------------------
# The skeleton: chains of (length, radius at its start, radius at its end).
# ---------------------------------------------------------------------------

SPINE = [(2.15, 2.1, 2.3), (2.0, 2.3, 2.35), (1.7, 2.35, 2.0)]
NECK = [(1.0, 1.8, 1.65), (0.88, 1.65, 1.5), (0.77, 1.5, 1.4)]
TAIL = [(1.8, 1.95, 1.7), (1.75, 1.7, 1.45), (1.7, 1.45, 1.2), (1.6, 1.2, 0.98),
        (1.55, 0.98, 0.8), (1.45, 0.8, 0.64), (1.3, 0.64, 0.52), (1.2, 0.52, 0.46)]
LEG = [(3.26, 1.55, 1.05), (2.6, 1.0, 0.7), (1.2, 0.72, 0.62), (0.9, 0.62, 0.5)]
ARM = [(2.3, 0.62, 0.5), (2.2, 0.54, 0.4), (0.8, 0.42, 0.36)]
HEAD = 2.5
# How much bigger the whole head is drawn than the lengths above, about
# where it meets the neck (the owner: "head bigger and heavier").
HEAD_SCALE = 1.35

# Where the legs and arms leave the body: (side, up, along), from the hip
# and from the shoulders, in the body's own frame.
HIP = (1.45, -0.45, 0.15)
SHOULDER = (1.6, -0.9, -0.3)

# How far the parts blend: flesh into flesh, and armour into flesh.
FLESH = 0.35
ARMOUR = 0.1

# The skin's roughness: a scaled, rocky hide over every volume -- how deep,
# and how fine, in model units. Two scales of it: plates and pebbles.
HIDE = ((0.12, 1.1), (0.05, 3.2))

SKIN = 0
BONE = 1


def unit(v):
    v = np.asarray(v, float)
    return v / np.linalg.norm(v)


def chain(start, bones, directions):
    """The points of a chain from `start`, each bone along its direction."""
    pts = [np.asarray(start, float)]
    for (length, _, _), d in zip(bones, directions):
        pts.append(pts[-1] + unit(d) * length)
    return pts


def frame(forward, up):
    """Right-handed axes (left, up, forward) from a forward and an up hint."""
    f = unit(forward)
    left = unit(np.cross(up, f))
    return left, unit(np.cross(f, left)), f


# ---------------------------------------------------------------------------
# A pose: which way each bone points, and which way is up for the body.
# ---------------------------------------------------------------------------

STANDING = {
    "hip": (0.0, 5.4, 0.0),
    "up": (0, 1, 0),
    # Reared and hunched, as a giant theropod is drawn when it means harm:
    # the back rising steeply to great shoulders, the neck thrust forward
    # and down, the head low and level, the jaws open.
    "spine": [(0, 0.75, 1), (0, 0.65, 1), (0, 0.4, 1)],
    "neck": [(0, 0.05, 1), (0, -0.3, 1), (0, -0.45, 1)],
    "head": (0, -0.12, 1),
    "jaw": 14.0,
    # The tail thick and heavy, held out behind it nearly level, its club
    # clear of the ground.
    "tail": [(0, -0.4, -1), (0, -0.25, -1), (0, -0.15, -1), (0, -0.08, -1),
             (0, -0.03, -1), (0, 0.0, -1), (0.08, 0.03, -1), (0.15, 0.05, -1)],
    # The thigh long, forward and a little down, the knee well ahead of the
    # hip, the shin long and back to the ankle, the foot up on its toes, the
    # toes flat: long legs set wide under its weight, a prowler's, not a
    # waddler's (the owner: "the presumptive movement would be more of a
    # prowl").
    "leg": [(0.154, -0.354, 0.923), (0.036, -0.665, -0.746), (0.0, -1, 0.3), (0.12, -0.12, 1)],
    # The arms hanging still: the upper arm down, the elbow bent forward a
    # little -- relaxed, not reaching -- the wrist dropping the hand back
    # down, the claws loose.
    "arm": [(0.2, -1, -0.12), (0.09, -0.76, 0.64), (0.0, -1, 0.1)],
}

ASLEEP = {
    "hip": (0.0, 2.3, 0.0),
    "up": (0, 1, 0),
    "jaw": 0.0,
    # Curled up the whole of its length (the owner: "asymmetrically
    # positioned, curled up -- and not just at the tail"): the back itself
    # bent in a C to its left; the neck turned all the way round so the head
    # rests against its left haunch, chin down; the tail wrapped round the
    # other way to meet it, the club by the snout. Nothing mirrored: the
    # upper hind leg drawn up along the flank, the lower tucked under; one
    # forearm folded back along the chest, the other back under the neck;
    # the whole beast settled onto its right side.
    "spine": [(0.05, 0.0, 1), (0.4, -0.02, 0.92), (0.75, -0.08, 0.65)],
    "neck": [(0.95, -0.25, 0.15), (0.7, -0.35, -0.6), (0.25, -0.35, -0.9)],
    "head": (-0.05, -0.1, -1),
    "tail": [(0.05, -0.7, -1), (0.4, -0.4, -0.9), (0.8, -0.1, -0.6), (1, -0.03, 0.0),
             (0.8, 0.0, 0.6), (0.35, 0.0, 0.95), (-0.05, 0.0, 1), (-0.35, 0.0, 0.95)],
    "leg_L": [(0.55, -0.25, 0.8), (0.35, -0.35, -0.85), (0.3, -0.3, 0.9), (0.2, -0.1, 1)],
    "leg_R": [(-0.25, -0.65, 0.7), (-0.15, -0.45, -0.9), (-0.1, -0.4, 0.9), (-0.05, -0.1, 1)],
    "arm_L": [(0.45, -0.75, -0.35), (0.25, -0.35, -0.95), (0.15, -0.7, -0.6)],
    "arm_R": [(-0.25, -0.9, -0.25), (0.45, -0.2, -0.85), (0.4, -0.6, -0.65)],
    "roll": 18.0,
}


def skeleton(pose):
    """Every bone's ends for a pose, by part, and the frames the armour is
    placed in."""
    up = np.asarray(pose["up"], float)
    hip = np.asarray(pose["hip"], float)
    spine = chain(hip, SPINE, pose["spine"])
    neck = chain(spine[-1], NECK, pose["neck"])
    head_dir = unit(pose["head"])
    tail = chain(hip, TAIL, pose["tail"])
    body = frame(spine[1] - spine[0], up)
    shoulders = frame(spine[-1] - spine[-2], up)
    out = {"spine": spine, "neck": neck, "tail": tail, "head": (neck[-1], head_dir)}
    for side, s in (("L", 1), ("R", -1)):
        h = hip + s * HIP[0] * body[0] + HIP[1] * body[1] + HIP[2] * body[2]
        leg = pose.get(f"leg_{side}", pose.get("leg"))
        leg = [(s * d[0], d[1], d[2]) for d in leg] if f"leg_{side}" not in pose else leg
        out[f"leg_{side}"] = chain(h, LEG, leg)
        a = spine[-1] + s * SHOULDER[0] * shoulders[0] + SHOULDER[1] * shoulders[1] + SHOULDER[2] * shoulders[2]
        arm = pose.get(f"arm_{side}", pose.get("arm"))
        arm = [(s * d[0], d[1], d[2]) for d in arm] if f"arm_{side}" not in pose else arm
        out[f"arm_{side}"] = chain(a, ARM, arm)
    out["up"] = up
    return out


# ---------------------------------------------------------------------------
# The volumes.
# ---------------------------------------------------------------------------


class Part:
    """One volume: a distance function over points, its bounds, how far it
    blends, and what it is (skin or bone)."""

    def __init__(self, fn, lo, hi, blend, kind):
        self.fn, self.lo, self.hi, self.blend, self.kind = fn, np.asarray(lo), np.asarray(hi), blend, kind
        # What it belongs to ("horn", or nothing), and, for a capsule, its
        # ends and radii: so a horn can be left out of the skin and laid on
        # the map as stone instead (`terrain.py`).
        self.group = None
        self.capsule = None


def capsule(a, b, ra, rb, blend=FLESH, kind=SKIN):
    """A capsule from `a` to `b`, its radius tapering from `ra` to `rb`."""
    a, b = np.asarray(a, float), np.asarray(b, float)
    ab = b - a
    ll = ab @ ab

    def fn(p):
        h = np.clip(((p - a) @ ab) / ll, 0, 1)
        return np.linalg.norm(p - a - h[..., None] * ab, axis=-1) - (ra + (rb - ra) * h)

    r = max(ra, rb)
    part = Part(fn, np.minimum(a, b) - r, np.maximum(a, b) + r, blend, kind)
    part.capsule = (a, b, ra, rb)
    return part


def ellipsoid(c, axes, radii, blend=FLESH, kind=SKIN):
    """An ellipsoid about `c`, its axes the columns of `axes`."""
    c = np.asarray(c, float)
    m = np.asarray(axes, float)
    radii = np.asarray(radii, float)

    def fn(p):
        q = ((p - c) @ m) / radii
        return (np.linalg.norm(q, axis=-1) - 1) * radii.min()

    r = radii.max()
    return Part(fn, c - r, c + r, blend, kind)


def wedge(c, axes, half, taper, round_=0.15, blend=FLESH, kind=SKIN):
    """A rounded box about `c` (axes the columns of `axes`: across, up,
    along), `half` its half-sizes, narrowing toward its front by `taper` (a
    share): a skull's flat planes, a jaw's edge -- what makes a head read as
    bone and not as a ball."""
    c = np.asarray(c, float)
    m = np.asarray(axes, float)
    half = np.asarray(half, float)

    def fn(p):
        q = (p - c) @ m
        front = np.clip((q[..., 2] / half[2] + 1) / 2, 0, 1)
        k = 1 - taper * front
        q = np.stack([q[..., 0] / k, q[..., 1] / k, q[..., 2]], -1)
        e = np.abs(q) - (half - round_)
        out = np.linalg.norm(np.maximum(e, 0), axis=-1) + np.minimum(e.max(-1), 0) - round_
        return out * k

    r = half.max() * 1.5
    return Part(fn, c - r, c + r, blend, kind)


def blade(base, direction, height, width, flat_along, blend=ARMOUR, kind=BONE, flare=1.2, tip=0.0):
    """A spike: a cone from `base` along `direction`, `width` at its foot,
    flattened to a blade lying along `flat_along` (thin across it). `flare`
    is how fast it narrows from its foot (more is a broader foot and a
    slimmer point); `tip` how round its point is."""
    base = np.asarray(base, float)
    d = unit(direction)
    across = unit(np.cross(d, flat_along))
    end = base + d * height

    def fn(p):
        rel = p - base
        # Squash across the blade: a blade is a cone pressed thin.
        squeezed = rel + (rel @ across)[..., None] * across * 1.6
        h = np.clip((squeezed @ d) / height, 0, 1)
        return np.linalg.norm(squeezed - h[..., None] * d * height, axis=-1) - (width - tip) * (1 - h) ** flare - tip

    lo = np.minimum(base, end) - width * 1.2
    hi = np.maximum(base, end) + width * 1.2
    return Part(fn, lo, hi, blend, kind)


def horn(base, directions, length, width, blend=ARMOUR, kind=BONE):
    """A curving horn: tapering capsules end to end, heavy at the root and
    thinning fastest toward the tip."""
    parts, at = [], np.asarray(base, float)
    n = len(directions)
    girth = lambda k: width * (1 - k / n) ** 0.85 + 0.03
    for i, d in enumerate(directions):
        nxt = at + unit(d) * length / n
        parts.append(capsule(at, nxt, girth(i), girth(i + 1), blend, kind))
        parts[-1].group = "horn"
        at = nxt
    return parts


def trimmed(part, at, normal):
    """A part with everything beyond a plane cut away: the plane through
    `at`, `normal` pointing to the side removed."""
    at, n = np.asarray(at, float), unit(normal)
    fn = part.fn
    return Part(lambda p: np.maximum(fn(p), (p - at) @ n), part.lo, part.hi, part.blend, part.kind)


def scaled(part, centre, k):
    """A part drawn `k` times its size about `centre`."""
    c = np.asarray(centre, float)
    fn = part.fn
    out = Part(lambda p: fn(c + (p - c) / k) * k, c + (part.lo - c) * k, c + (part.hi - c) * k, part.blend * k, part.kind)
    out.group = part.group
    if part.capsule is not None:
        a, b, ra, rb = part.capsule
        out.capsule = (c + (a - c) * k, c + (b - c) * k, ra * k, rb * k)
    return out


def body_parts(pose):
    """Every volume of the beast in a pose."""
    sk = skeleton(pose)
    up = sk["up"]
    parts = []

    def along(points, bones, blend=FLESH):
        for (a, b), (_, ra, rb) in zip(zip(points, points[1:]), bones):
            parts.append(capsule(a, b, ra, rb, blend))

    spine, neck, tail = sk["spine"], sk["neck"], sk["tail"]
    along(spine, SPINE)
    along(neck, NECK)
    along(tail, TAIL)

    # The mass of it: a belly hanging under the middle of the back, a barrel
    # chest under the shoulders, haunches over the hips.
    left, bup, fwd = frame(spine[2] - spine[0], up)
    parts.append(ellipsoid(spine[1] - bup * 0.55, np.c_[left, bup, fwd], (2.2, 1.55, 3.0)))
    left2, bup2, fwd2 = frame(spine[3] - spine[2], up)
    parts.append(ellipsoid(spine[2] - bup2 * 0.6 + fwd2 * 0.4, np.c_[left2, bup2, fwd2], (2.5, 2.3, 1.9)))
    # Great shoulders: a hump of muscle where the neck meets the back.
    parts.append(ellipsoid(spine[3] + bup2 * 0.4 - fwd2 * 0.2, np.c_[left2, bup2, fwd2], (2.2, 1.6, 1.4)))

    # The legs: thighs like drums, a heel, a broad foot with three toes.
    for side, s in (("L", 1), ("R", -1)):
        leg = sk[f"leg_{side}"]
        along(leg, LEG)
        thigh = leg[1] - leg[0]
        l, u, f = frame(thigh, up)
        # The thigh's muscle along the whole of its length, so the long
        # thigh shows as long.
        parts.append(ellipsoid(leg[0] + thigh * 0.45, np.c_[l, u, f], (1.45, 1.5, 2.25)))
        shin = leg[2] - leg[1]
        sl, su, sf = frame(shin, up if abs(unit(shin) @ up) < 0.95 else f)
        parts.append(ellipsoid(leg[1] + shin * 0.35, np.c_[sl, su, sf], (0.9, 0.95, 1.35), 0.35))
        toes = leg[4] - leg[3]
        tl, tu, tf = frame(toes, up)
        parts.append(ellipsoid(leg[3] + toes * 0.3, np.c_[tl, tu, tf], (1.2, 0.5, 1.15)))
        for spread in (-0.55, 0.0, 0.55):
            toe = leg[3] + tl * spread * 1.0 + toes * 0.45
            tip = toe + unit(tf + tl * spread * 0.7) * 1.1
            parts.append(capsule(toe, tip, 0.42, 0.3, 0.3))
            parts.append(blade(tip - tu * 0.05, tf + tl * spread * 0.7 - tu * 0.7, 0.8, 0.26, tu, 0.1))
        # A spur off the back of the heel.
        parts.append(blade(leg[2] - sf * 0.0, -unit(toes) + tu * 0.3, 0.8, 0.25, tu))
        # The arms: shoulder, forearm, hand, three claws.
        arm = sk[f"arm_{side}"]
        along(arm, ARM, 0.4)
        hand = arm[3] - arm[2]
        hl, hu, hf = frame(hand, up if abs(unit(hand) @ up) < 0.95 else fwd)
        forearm = arm[2] - arm[1]
        fl, fu, ff = frame(forearm, up if abs(unit(forearm) @ up) < 0.95 else fwd)
        parts.append(ellipsoid(arm[1] + forearm * 0.35, np.c_[fl, fu, ff], (0.66, 0.66, 1.0), 0.3))
        parts.append(ellipsoid(arm[0] + (arm[1] - arm[0]) * 0.3, np.c_[fl, fu, ff], (0.8, 0.8, 1.05), 0.4))
        for spread in (-0.35, 0.0, 0.35):
            parts.append(blade(arm[3] + hl * spread * 0.8, unit(hand) + hl * spread * 0.35 - hu * 0.3, 1.2, 0.24, hl, 0.08))

    # The head: wedges, not balls. An underbite (the owner: "top and
    # bottom jaws need to be different sizes ... the chin / jaw more
    # prominent and heavy and the profile receded toward the back"): the
    # lower jaw the biggest thing in the face -- long, wide, deep at a
    # squared chin that juts past the snout, tusks rising from its corners
    # outside the upper lip -- and the upper skull short and set back over
    # it, so the profile falls away from the chin to the muzzle to the
    # brow. A heavy brow shelf over each eye, cheek bosses, ram's horns, a
    # fringe of blades behind, teeth along both jaws.
    at, d = sk["head"]
    hl, hu, hf = frame(d, up)
    head_from = len(parts)
    parts.append(wedge(at + hf * 0.95 + hu * 0.25, np.c_[hl, hu, hf], (1.05, 0.8, 1.1), 0.5, 0.45, 0.3))
    gape = np.radians(pose.get("jaw", 0.0))
    hinge = at + hf * 0.35 - hu * 0.35
    jf = unit(hf * np.cos(gape) - hu * np.sin(gape))
    ju = unit(np.cross(jf, hl))
    # The jaw runs on straight past the teeth to its chin, and the chin is
    # an angle, not a bulb: the jaw's front cut back from a hard corner at
    # its foot up to the tusks (the owner: "continue the jaw line straight
    # along its main path, then make the chin more of an angle").
    # One block, even in depth, so its underside runs dead straight to
    # the front; the chin's angle cut into its upper front by a slanted
    # plane through the foot of its front face.
    jaw_mid = hinge + jf * 1.35 - ju * 0.45
    jaw = wedge(jaw_mid, np.c_[hl, ju, jf], (1.05, 0.62, 1.55), 0.0, 0.25, 0.3)
    foot = jaw_mid + jf * 1.55 - ju * 0.62
    cut = unit(jf * np.cos(np.radians(35)) + ju * np.sin(np.radians(35)))
    parts.append(trimmed(jaw, foot, cut))
    for s_ in (1, -1):
        parts.append(wedge(at + hf * 0.95 + hu * 0.75 + hl * s_ * 0.62, np.c_[hl, hu, hf], (0.42, 0.22, 0.5), 0.2, 0.1, 0.15))
        parts.append(ellipsoid(at + hf * 0.45 - hu * 0.15 + hl * s_ * 0.95, np.c_[hl, hu, hf], (0.4, 0.55, 0.6), 0.25))
        # The horns (the owner: "bigger horns, very heavy on top then
        # wrapping down around the face"): rooted thick on the crown, up
        # and back, out and over, then curling down past the cheek and
        # forward along the jaw -- a ram's, on a giant.
        base = at + hf * 0.45 + hu * 0.85 + hl * s_ * 0.55
        parts += horn(base, [hl * s_ * 0.4 + hu * 0.9 - hf * 0.35, hl * s_ * 0.7 + hu * 0.3 - hf * 0.8,
                             hl * s_ * 0.95 - hu * 0.55 - hf * 0.35, hl * s_ * 0.75 - hu * 0.8 + hf * 0.45,
                             hl * s_ * 0.35 - hu * 0.35 + hf * 1.0, hl * s_ * 0.3 + hu * 0.15 + hf, hl * s_ * 0.2 + hu * 0.35 + hf],
                      5.2, 0.72, 0.2)
        for k in range(4):
            parts.append(blade(at + hf * (0.15 - 0.28 * k) + hu * (0.55 - 0.18 * k) + hl * s_ * (0.9 + 0.12 * k),
                               -hf + hl * s_ * 0.9 + hu * 0.35, 0.6 - 0.1 * k, 0.18, hu))
        # Teeth: the upper row short, under the set-back muzzle; the lower
        # longer, along the jaw; and a tusk at each front corner of the
        # chin, rising outside the upper lip.
        for k in range(5):
            x = 0.7 + 0.24 * k
            parts.append(blade(at + hf * x - hu * 0.45 + hl * s_ * (0.7 - 0.06 * k), -hu + hf * 0.1, 0.42, 0.11, hf, 0.04))
        for k in range(5):
            x = 0.62 + 0.23 * k
            parts.append(blade(hinge + jf * x + ju * 0.15 + hl * s_ * (0.85 - 0.03 * k), ju + jf * 0.1, 0.42, 0.11, jf, 0.04))
        parts.append(blade(hinge + jf * 1.7 + ju * 0.1 + hl * s_ * 0.72, ju + hl * s_ * 0.25 - jf * 0.15, 1.05, 0.25, jf, 0.08))
    parts.append(blade(at + hf * 1.75 + hu * 0.6, hu + hf * 0.5, 0.6, 0.24, hf))
    parts[head_from:] = [scaled(q, at, HEAD_SCALE) for q in parts[head_from:]]

    # The back: a row of tall blades down the spine from the hips to the
    # nape, tallest over the shoulders, and armour knobs on the flanks.
    # Not on the tail (the owner: "remove the spikes from the tail").
    line = spine + neck[1:]
    lens = np.r_[0, np.cumsum(np.linalg.norm(np.diff(line, axis=0), axis=1))]
    total = lens[-1]
    peak = lens[len(spine) - 1] / total
    rng = np.random.default_rng(7)
    for n, x in enumerate(np.arange(0.3, total, 0.55)):
        i = min(np.searchsorted(lens, x) - 1, len(line) - 2)
        t = (x - lens[i]) / (lens[i + 1] - lens[i])
        p = line[i] + t * (line[i + 1] - line[i])
        fwd_here = unit(line[i + 1] - line[i])
        l, u, f = frame(fwd_here, up)
        u_ = x / total
        size = 0.3 + 0.7 * np.exp(-((u_ - peak) / 0.3) ** 2)
        size *= 0.8 + 0.4 * rng.random()
        radius = np.interp(x, lens, _radii(line, sk))
        # Two rows, staggered either side of the spine, leaning back.
        s_ = 1 if n % 2 else -1
        base = p + u * (radius - 0.4) + l * s_ * 0.35
        # A broad, flared foot growing out of the back, not a slab stood on
        # it, and a point a little dull (the owner: "thicker at the base so
        # they don't read like a stone overhanging ... the tips just a
        # little dull").
        parts.append(blade(base - u * 0.25, u + f * -0.25 + l * s_ * 0.12, 0.75 + 3.4 * size, 1.5 * size + 0.4, f,
                           ARMOUR * 2, BONE, 1.5, 0.08 + 0.04 * size))
        if size > 0.55:
            # Over the shoulders and hips, a lower plate outside each.
            parts.append(blade(p + u * (radius - 0.65) + l * -s_ * 0.95, u * 0.8 + l * -s_ * 0.6 - f * 0.2, 0.55 + 1.6 * size, 1.0 * size + 0.2, f,
                               ARMOUR * 2, BONE, 1.5, 0.07))
        # Armour knobs on the flanks, scattered.
        for s2 in (1, -1):
            a_ = 0.55 + 0.25 * rng.random()
            c = p + (u * np.cos(a_) + l * s2 * np.sin(a_)) * radius * 0.92
            parts.append(blade(c, u * np.cos(a_) + l * s2 * np.sin(a_), 0.35 + 0.4 * size, 0.28, f, 0.12))

    # The club: a heavy knob of bone at the tail's end, smooth.
    end = tail[-1]
    tl, tu, tf = frame(tail[-1] - tail[-2], up)
    parts.append(ellipsoid(end + tf * 0.4, np.c_[tl, tu, tf], (1.5, 1.15, 1.55), 0.35, BONE))
    return parts


def _radii(line, sk):
    """The body's radius at each point of the back's line, for seating the
    spikes on the skin: the spine's, then the neck's."""
    spine_r = [SPINE[0][1]] + [b[2] for b in SPINE]
    neck_r = [b[2] for b in NECK]
    return spine_r + neck_r


def smin(a, b, k):
    """A smooth minimum: the union of two volumes, blended over `k`."""
    h = np.clip((k - np.abs(a - b)) / k, 0, None)
    return np.minimum(a, b) - h * h * k * 0.25


def value_noise(points, scale, seed):
    """Smooth value noise in [-1, 1] at points, cells `1/scale` across."""
    rng = np.random.default_rng(seed)
    table = rng.uniform(-1, 1, (64, 64, 64)).astype(np.float32)
    q = points * scale
    i = np.floor(q).astype(int)
    f = q - i
    f = f * f * (3 - 2 * f)
    out = 0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (f[..., 0] if dx else 1 - f[..., 0]) * (f[..., 1] if dy else 1 - f[..., 1]) * (f[..., 2] if dz else 1 - f[..., 2])
                out = out + w * table[(i[..., 0] + dx) % 64, (i[..., 1] + dy) % 64, (i[..., 2] + dz) % 64]
    return out


def field(parts, cell):
    """The distance field over a grid round the parts, and which kind of
    part is nearest at each point."""
    lo = np.min([p.lo for p in parts], axis=0) - 0.5
    hi = np.max([p.hi for p in parts], axis=0) + 0.5
    n = np.ceil((hi - lo) / cell).astype(int) + 1
    d = np.full(n, 10.0, np.float32)
    kind = np.zeros(n, np.uint8)
    for part in parts:
        i0 = np.clip(np.floor((part.lo - part.blend - lo) / cell).astype(int), 0, n - 1)
        i1 = np.clip(np.ceil((part.hi + part.blend - lo) / cell).astype(int) + 1, 1, n)
        axes = [lo[k] + np.arange(i0[k], i1[k]) * cell for k in range(3)]
        g = np.stack(np.meshgrid(*axes, indexing="ij"), -1)
        v = part.fn(g).astype(np.float32)
        sub = d[i0[0]:i1[0], i0[1]:i1[1], i0[2]:i1[2]]
        ksub = kind[i0[0]:i1[0], i0[1]:i1[1], i0[2]:i1[2]]
        ksub[v < sub] = part.kind
        d[i0[0]:i1[0], i0[1]:i1[1], i0[2]:i1[2]] = smin(sub, v, part.blend)
    # The hide: roughness laid over the skin only near it, where it shows.
    near = np.abs(d) < 0.6
    pts = lo + np.argwhere(near) * cell
    rough = sum(amp * value_noise(pts, freq, 11 + k) for k, (amp, freq) in enumerate(HIDE))
    rough = rough * np.where(kind[near] == BONE, 0.35, 1.0)
    d[near] += rough.astype(np.float32)
    return d, kind, lo


def placing(pose, verts):
    """How a posed beast is set down: its roll onto its side, and how far
    it is lifted so it lies on the ground -- worked out once from its skin
    and kept, so anything else of it (its horns, as stone) goes where the
    skin does."""
    a = np.radians(pose.get("roll", 0.0))
    # Rolled onto its right side about the line it faces along.
    r = np.array([[np.cos(a), -np.sin(a), 0], [np.sin(a), np.cos(a), 0], [0, 0, 1]])
    lift = -np.percentile((verts @ r.T)[:, 1], 1)
    return lambda p: np.asarray(p, float) @ r.T + np.array([0.0, lift, 0.0])


def build(pose, cell=0.07, without=()):
    """The beast's skin in a pose: vertices, triangles, a colour a triangle
    (flesh or bone), and how it was set down (`placing`). `without` leaves
    out the parts of those groups ("horn")."""
    from skimage.measure import marching_cubes

    parts = [p for p in body_parts(pose) if p.group not in without]
    d, kind, lo = field(parts, cell)
    verts, faces, _, _ = marching_cubes(d, 0.0, spacing=(cell, cell, cell))
    verts = verts + lo
    idx = np.clip(np.round((verts - lo) / cell).astype(int), 0, np.array(d.shape) - 1)
    vk = kind[idx[:, 0], idx[:, 1], idx[:, 2]]
    tk = vk[faces].max(1)
    flesh = np.array([0.50, 0.47, 0.42])
    bone = np.array([0.80, 0.76, 0.66])
    colours = np.where(tk[:, None] == BONE, bone, flesh)
    place = placing(pose, verts)
    # Marching cubes winds faces one way; the renderer is two-sided anyway.
    return place(verts), faces[:, [0, 2, 1]], colours, place


def horns(pose, place):
    """The horns as chains of capsules -- (from, to, radius there, radius
    here) -- set down as `place` sets the skin down."""
    out = []
    for p in body_parts(pose):
        if p.group == "horn" and p.capsule is not None:
            a, b, ra, rb = p.capsule
            out.append((place(a), place(b), ra, rb))
    return out
