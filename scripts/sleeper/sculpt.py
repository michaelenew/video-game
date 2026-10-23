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


def sculpt(model):
    """Reshape `model` in place: no wings, spikes down, a tarrasque's head."""
    names = dominant(model)
    # No wings.
    wing = np.char.startswith(names, "wing")
    model.triangles = model.triangles[~wing[model.triangles].any(1)]
    # The head first, so its horns are smoothed in their new place.
    p = tarrasque_head(model, model.positions)
    # Spikes down, but the back's.
    unique, which = welded(p)
    keep = np.zeros(len(unique), bool)
    np.logical_or.at(keep, which, back_spikes(model, names))
    tris = which[model.triangles]
    unique = smooth(unique, tris, keep)
    head = np.zeros(len(unique), bool)
    np.logical_or.at(head, which, weight_of(model, HEAD) > 0.5)
    unique = smooth(unique, tris, ~head, HEAD_ROUNDS)
    model.positions = unique[which]
    return model
