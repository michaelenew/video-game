"""The Sleeper's body, reshaped before it is posed: the model is a European
dragon, and the beast asleep in the mountains is a tarrasque.

What the owner asked for (2026-10-23): "tone down all the spikiness ...
keep the big spikes along the back, that's cool"; "remove the wings as
well, since this thing shouldn't fly"; "manipulate the face to be more
tarrasque-like".

All of it happens to the mesh in its bind pose, before skinning, so the
pose (`pose.py`) carries it wherever the bones go:

- **No wings**: every triangle carried mostly by a wing bone is dropped.
- **Spikes smoothed**: the mesh is smoothed (Taubin's two-step smoothing,
  which pulls thin protrusions in without shrinking the body) everywhere
  but the row of big spikes down the back -- whatever stands well above
  the line of the spine, from the root of the neck to the middle of the
  tail.
- **A tarrasque's head**: the skull pressed toward a short, broad egg --
  a blunt, heavy head rather than a long snout -- and its swept horns and
  crest drawn most of the way in.

And on the second pass ("make the neck thicker ... add a row of prominent
spikes on its back, make the spikes on its back legs smaller"):

- **A thick neck**: pushed out from the line of its bones, thickest at the
  shoulders, at the back and sides only -- its throat is drawn in, so it
  hangs no bib down the chest.
- **The hind legs' spikes smaller**: smoothed a few rounds, not held.
- **A row of spikes down the back**: new cones, standing out of the middle
  of the back from the neck to the tail, tallest over the shoulders and
  hips, each carried by the spine bone under it.
"""
import numpy as np

# How many rounds of smoothing, and its two steps: in by LAMBDA, out by MU.
ROUNDS = 14
LAMBDA = 0.55
MU = -0.58

# The back's spikes: anything standing this far above the line of the
# spine, within this far of the middle line, between the root of the neck
# and the middle of the tail -- metres of the model. Whatever bone carries
# it: the tallest stand off the hips.
SPINE = ("tail_03", "tail_02", "tail_01", "pelvis", "chest", "neck_01")
BACK_ABOVE = 0.22
BACK_MIDDLE = 0.55

# How many rounds the hind legs' spikes are smoothed: fewer than the rest,
# so they shrink rather than go.
LEG_ROUNDS = 4
LEGS = ("thigh", "shin", "foot", "toes")

# The neck: how much thicker, at its root and at the head -- at the back
# and the sides; its throat is drawn in to this instead, so it hangs no bib
# down the chest.
NECK_ROOT = 1.7
NECK_TOP = 1.3
THROAT = 0.75
NECK = ("neck_01", "neck_02", "neck_03", "neck_04", "neck_05")

# The row of spikes: along these bones, from the tail to the head; how far
# apart, metres of the model; how tall and how wide at the foot, at their
# tallest; and the share of that each end of the row tapers to.
ROW = ("tail_04", "tail_03", "tail_02", "tail_01", "pelvis", "chest", "neck_01", "neck_03")
ROW_EVERY = 0.2
ROW_TALL = 0.55
ROW_WIDE = 0.1
ROW_ENDS = 0.35
ROW_SIDES = 5

# How many more rounds the head is smoothed, the rest held: its horns are
# long, and a tarrasque's brow is a ridge, not a crown of blades.
HEAD_ROUNDS = 30

# The head: an egg it is pressed toward, about a point this far along the
# snout from the head bone -- half-lengths along the snout, across and up
# and down, metres of the model -- and how much of anything outside it is
# kept (a horn a metre out keeps this share of the metre).
HEAD_ALONG = 0.45
HEAD_EGG = (0.62, 0.42, 0.36)
HEAD_KEEP = 0.25
HEAD = ("head", "jaw", "tongue", "fire_socket")


def bind_point(model, bone):
    """Where a bone is in the mesh's bind pose."""
    k = model.skin_joints.index(model.index[bone])
    return np.linalg.inv(model.inverse_bind[k])[:3, 3]


def dominant(model):
    """The name of the bone carrying most of each vertex."""
    heaviest = model.joints[np.arange(len(model.joints)), model.weights.argmax(1)]
    return np.array([model.names[model.skin_joints[j]] for j in heaviest])


def weight_of(model, bones):
    """How much of each vertex the named bones carry, 0 to 1."""
    ids = [model.skin_joints.index(model.index[b]) for b in bones if b in model.index]
    w = np.zeros(len(model.positions))
    for slot in range(4):
        w += np.where(np.isin(model.joints[:, slot], ids), model.weights[:, slot], 0)
    return w


def welded(points):
    """The mesh's points with duplicates (one a face, for its texture)
    merged: the unique points, and which one each vertex is."""
    return np.unique(np.round(points, 5), axis=0, return_inverse=True)


def back_spikes(model, names):
    """Which vertices are the back's big spikes, kept sharp."""
    p = model.positions
    line = np.array([bind_point(model, b) for b in SPINE])
    spine_y = np.interp(p[:, 2], line[:, 2], line[:, 1])
    along = (p[:, 2] > line[0, 2]) & (p[:, 2] < line[-1, 2])
    return (
        along
        & (np.abs(p[:, 0]) < BACK_MIDDLE)
        & (p[:, 1] > spine_y + BACK_ABOVE)
        & ~np.char.startswith(names, "wing")
    )


def smooth(points, tris, fixed, rounds=ROUNDS):
    """Taubin smoothing over the welded mesh, `fixed` points held."""
    n = len(points)
    edges = np.r_[tris[:, [0, 1]], tris[:, [1, 2]], tris[:, [2, 0]]]
    edges = np.unique(np.sort(edges, 1), axis=0)
    degree = np.bincount(edges.ravel(), minlength=n).astype(float)
    degree[degree == 0] = 1
    p = points.copy()
    for _ in range(rounds):
        for step in (LAMBDA, MU):
            acc = np.zeros_like(p)
            np.add.at(acc, edges[:, 0], p[edges[:, 1]])
            np.add.at(acc, edges[:, 1], p[edges[:, 0]])
            move = acc / degree[:, None] - p
            move[fixed] = 0
            p = p + step * move
    return p


def tarrasque_head(model, p):
    """The skull pressed toward an egg -- short, broad, heavy-jawed -- as
    far as each vertex is the head's: what is inside it swells out to it,
    and what stands out of it (the horns, the crest) is mostly drawn in."""
    w = weight_of(model, HEAD)
    at = bind_point(model, "head")
    snout = bind_point(model, "fire_socket") - at
    fwd = snout / np.linalg.norm(snout)
    side = np.array([1.0, 0, 0])
    up = np.cross(fwd, side)
    up *= np.sign(up[1])
    centre = at + fwd * HEAD_ALONG
    axes = np.c_[fwd, side, up]
    local = (p - centre) @ axes / np.array(HEAD_EGG)
    r = np.linalg.norm(local, axis=1)
    r_new = np.where(r > 1, 1 + (r - 1) * HEAD_KEEP, 0.6 + 0.4 * r)
    local = local * (r_new / np.maximum(r, 1e-9))[:, None]
    shaped = centre + (local * np.array(HEAD_EGG)) @ axes.T
    return p + w[:, None] * (shaped - p)


def thicker_neck(model, p):
    """The neck pushed out from the line of its bones, as far as each vertex
    is the neck's: thickest at the root, less toward the head."""
    line = np.array([bind_point(model, b) for b in ("chest",) + NECK + ("head",)])
    w = weight_of(model, NECK)
    out = p.copy()
    for k in np.nonzero(w > 0)[0]:
        best = None
        for i in range(len(line) - 1):
            a, b = line[i], line[i + 1]
            ab = b - a
            t = np.clip((p[k] - a) @ ab / (ab @ ab), 0, 1)
            c = a + t * ab
            d = np.linalg.norm(p[k] - c)
            if best is None or d < best[0]:
                best = (d, c, (i + t) / (len(line) - 1), ab)
        _, c, along, seg = best
        grow = NECK_ROOT + (NECK_TOP - NECK_ROOT) * along
        # How much this point is the throat: its side of the neck facing
        # forward and down, away from the back.
        d = seg / np.linalg.norm(seg)
        front = -np.cross(d, (1.0, 0, 0))
        r = p[k] - c
        throat = np.clip(r @ front / (np.linalg.norm(r) + 1e-9), 0, 1)
        grow = grow + (THROAT - grow) * throat
        out[k] = p[k] + w[k] * (c + r * grow - p[k])
    return out


def add_row(model):
    """A row of cones down the middle of the back, its foot on the body's
    surface, each carried by the bones that carry the body where it stands
    -- so it moves with the skin under it, not apart from it."""
    line = np.array([bind_point(model, b) for b in ROW])
    seg = np.linalg.norm(np.diff(line, axis=0), axis=1)
    total = seg.sum()
    used = np.unique(model.triangles)
    body = model.positions[used]
    verts, tris, joints, weights = [], [], [], []
    for d in np.arange(ROW_EVERY / 2, total, ROW_EVERY):
        i = min(np.searchsorted(np.cumsum(seg), d), len(seg) - 1)
        t = (d - np.cumsum(seg)[i] + seg[i]) / seg[i]
        at = line[i] + t * (line[i + 1] - line[i])
        # Headward along the spine; its back is that turned up (and back,
        # where the neck stands up).
        along = (line[i + 1] - line[i]) / seg[i]
        back = np.cross(along, (1.0, 0, 0))
        back /= np.linalg.norm(back)
        # The body's surface over the spine here: the furthest of it out
        # along the back, close to that line.
        # Only the body round this point of the spine: not the back behind
        # the neck, which a line out of the neck's back runs into.
        rel = body - at
        out = rel @ back
        off = rel - np.outer(out, back)
        near = (np.linalg.norm(off, axis=1) < 0.18) & (out < 0.7)
        top = out[near].max() if near.any() else 0.2
        # Tallest over the shoulders and hips, tapering at both ends.
        u = d / total
        size = ROW_ENDS + (1 - ROW_ENDS) * np.sin(np.pi * u) ** 0.7
        foot = at + back * (top - 0.04)
        tip = foot + back * ROW_TALL * size + along * -0.12 * size
        side = np.cross(back, along)
        base = len(model.positions) + len(verts)
        verts.append(tip)
        for k in range(ROW_SIDES):
            a = 2 * np.pi * k / ROW_SIDES
            # Flattened across the body: a blade more than a cone.
            verts.append(foot + ROW_WIDE * size * (np.cos(a) * along * 1.6 + np.sin(a) * side * 0.7))
        for k in range(ROW_SIDES):
            tris.append((base, base + 1 + k, base + 1 + (k + 1) % ROW_SIDES))
        under = used[np.argmin(np.linalg.norm(body - foot, axis=1))]
        joints += [model.joints[under]] * (ROW_SIDES + 1)
        weights += [model.weights[under]] * (ROW_SIDES + 1)
    model.positions = np.r_[model.positions, np.array(verts)]
    model.joints = np.r_[model.joints, np.array(joints)]
    model.weights = np.r_[model.weights, np.array(weights)]
    model.triangles = np.r_[model.triangles, np.array(tris)]


def sculpt(model):
    """Reshape `model` in place: no wings, spikes down but a row down the
    back, a thick neck, a tarrasque's head."""
    names = dominant(model)
    # No wings.
    wing = np.char.startswith(names, "wing")
    model.triangles = model.triangles[~wing[model.triangles].any(1)]
    # The head first, so its horns are smoothed in their new place.
    p = tarrasque_head(model, model.positions)
    # Spikes down, but the back's; the hind legs' only a little.
    unique, which = welded(p)
    spikes = back_spikes(model, names)
    on_legs = np.isin(names, [n for n in names if n.startswith(LEGS)])
    keep = np.zeros(len(unique), bool)
    np.logical_or.at(keep, which, spikes)
    leg_spike = np.zeros(len(unique), bool)
    np.logical_or.at(leg_spike, which, spikes & on_legs)
    tris = which[model.triangles]
    unique = smooth(unique, tris, keep)
    unique = smooth(unique, tris, ~leg_spike, LEG_ROUNDS)
    head = np.zeros(len(unique), bool)
    np.logical_or.at(head, which, weight_of(model, HEAD) > 0.5)
    unique = smooth(unique, tris, ~head, HEAD_ROUNDS)
    model.positions = thicker_neck(model, unique[which])
    add_row(model)
    return model
