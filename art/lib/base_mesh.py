"""A body from a sculpted base mesh, fitted to the game's skeleton through
MakeHuman.

MakeHuman's bodies, however they were pushed, never read as a person in the
game (the owner, 2026-10-27: "Obviously everything is completely wrong ...
use Blender's model for the dark side as a starting point"). A base mesh an
artist sculpted -- Blender Studio's realistic male, CC0 (``art/sources``) --
does. What it has not got is anything the pipeline needs: no skeleton, no
weights, and its own pose and proportions.

So it is fitted *through* a MakeHuman body, which has all of those
(``fit.fit``):

1. A plain MakeHuman man is fitted to the game's skeleton as every class
   is; his body before the fit (``rest``) and after it are the map.
2. The base mesh is laid over him before the fit: scaled to his height, and
   each arm and leg turned and stretched, joint by joint, onto his.
3. Each of its vertices then moves as the MakeHuman vertices nearest it
   moved, and takes their weights.

Its shape is its own throughout; MakeHuman only says where the joints are
and which bone carries what. Everything is in the game's frame (``+y`` up,
``+z`` forward), metres.
"""

import os

import numpy as np
from mathutils import kdtree

import fit
import garment as g
import mh
import stage

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))


def load(name="human_base_mesh_male"):
    """The base mesh in the game's frame: vertices, faces, per-loop UVs (as
    ``uvs`` and each face's indices into them) and the eyes (centre, radius)."""
    d = np.load(os.path.join(ROOT, "art", "sources", f"{name}.npz"))
    verts = g.b2g(d["verts"].astype(np.float64))
    faces, face_uvs, at = [], [], 0
    for n in d["counts"]:
        faces.append([int(i) for i in d["loops"][at:at + n]])
        face_uvs.append(list(range(at, at + n)))
        at += n
    eyes = d["eyes"].astype(np.float64)
    eyes = np.c_[g.b2g(eyes[:, :3]), eyes[:, 3]]
    return verts, faces, _one_tile(d["uvs"].astype(np.float64), d["counts"]), face_uvs, eyes


def _one_tile(uvs, counts):
    """The base mesh's UVs are laid over four tiles side by side (``u`` from
    0 to 4); the painter has one square. Each tile is packed into a quarter
    of it, two by two -- each face by the tile its middle is in, so no face
    is split. Left on four, three tiles were painted over the first, and a
    band of the leg came out across the back."""
    out = uvs.copy()
    at = 0
    for n in counts:
        f = slice(at, at + n)
        tile = int(np.clip(np.floor(uvs[f, 0].mean()), 0, 3))
        out[f, 0] = ((uvs[f, 0] - tile) + tile % 2) / 2
        out[f, 1] = (uvs[f, 1] + tile // 2) / 2
        at += n
    return out


def _smooth(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def _q_mat(a, b):
    return fit.q_mat(fit.q_between(a, b))


def _segment(p, a, b, A, B):
    """``p`` carried from the segment ``a -> b`` onto ``A -> B``: turned with
    it, stretched along it, its girth kept."""
    u = (b - a) / np.linalg.norm(b - a)
    s = np.linalg.norm(B - A) / np.linalg.norm(b - a)
    d = p - a
    along = d @ u
    perp = d - np.outer(along, u)
    return A + (np.outer(along * s, u) + perp) @ _q_mat(b - a, B - A).T


def _chain(p, mask, ours, theirs, root_blend=(0.02, 0.1), joint_blend=0.04):
    """Carry the limb's vertices (``mask``) from our joints ``ours`` (root,
    middle, end) onto ``theirs``: the upper segment, the lower, and what is
    past the end carried rigidly with the lower. Blended across the middle
    joint, and from nothing at the root into the trunk."""
    a, b, c = ours
    A, B, C = theirs
    # ``mask`` may be a share, not a yes or no: the limb fading out.
    amount = np.asarray(mask, float)
    mask = amount > 0
    amount = amount[mask]
    q = p[mask]
    upper = _segment(q, a, b, A, B)
    lower = _segment(q, b, c, B, C)
    end = C + (q - c) @ _q_mat(c - b, C - B).T
    u1 = (b - a) / np.linalg.norm(b - a)
    u2 = (c - b) / np.linalg.norm(c - b)
    t1 = (q - b) @ u1          # past the middle joint along the upper bone
    t2 = (q - c) @ u2          # past the end joint along the lower bone
    w_low = _smooth(-joint_blend, joint_blend, t1)
    # The thumb starts beside the wrist, not past it: the hand is carried
    # whole from a little before it.
    w_end = _smooth(-0.04, 0.0, t2)
    moved = upper * (1 - w_low)[:, None] + lower * (w_low * (1 - w_end))[:, None] + end * (w_low * w_end)[:, None]
    from_root = (q - a) @ u1
    w = _smooth(*root_blend, from_root) * amount
    out = p.copy()
    out[mask] = q + (moved - q) * w[:, None]
    return out


def _centroid(p, mask):
    return p[mask].mean(axis=0) if mask.any() else None


def _joined(mask, start, nbrs):
    """The part of ``mask`` joined to vertex ``start`` through the mesh."""
    keep = np.zeros(len(mask), bool)
    stack = [start]
    keep[start] = True
    while stack:
        i = stack.pop()
        for j in nbrs[i]:
            if mask[j] and not keep[j]:
                keep[j] = True
                stack.append(j)
    return keep


def _neighbours(n, faces):
    nbrs = [[] for _ in range(n)]
    for f in faces:
        for a, b in zip(f, f[1:] + f[:1]):
            nbrs[a].append(b)
            nbrs[b].append(a)
    return nbrs


def _limbs(p, lm, nbrs):
    """Our base mesh's own arm and leg joints, found on it after it is
    scaled to MakeHuman's: each root at MakeHuman's (the shoulder, the hip),
    and the joints down the limb as the middles of the limb's own cross-
    sections where MakeHuman's are along it."""
    out = {}
    for side in ("l", "r"):
        sx = np.sign(lm[f"arm.{side}"][0])
        sh, el, wr, tip = (lm[f"{n}.{side}"] for n in ("arm", "forearm", "hand", "handtip"))
        # The arm: everything well out to its side below the armpit, and the
        # hand's lowest point.
        arm = (p[:, 0] * sx > abs(sh[0]) * 1.05) & (p[:, 1] < sh[1] - 0.06)
        ftip_i = np.flatnonzero(arm)[np.argmin(p[arm, 1])]
        ftip = p[ftip_i]
        d = (ftip - sh) / np.linalg.norm(ftip - sh)
        length = np.linalg.norm(ftip - sh)
        theirs = np.linalg.norm(tip - sh)
        along = (p - sh) @ d
        perp = np.linalg.norm((p - sh) - np.outer(along, d), axis=1)
        near = perp < 0.09
        mid = {}
        for name, at in (("forearm", el), ("hand", wr)):
            f = np.linalg.norm(at - sh) / theirs
            mid[name] = _centroid(p, near & (np.abs(along - f * length) < 0.012) & (p[:, 0] * sx > 0.05))
        # The whole arm, hand and all: everything further out than the
        # trunk ever is. (Within a few centimetres of a line down it, the
        # thumb's side of the hand was left by the thigh.)
        # Nothing below the fingertips: the feet splay as far out. By the
        # armpit the arm fades into the chest across the pectoral and the
        # deltoid, or the turn creases the chest where it stops; lower down,
        # beside the thigh, it stops sharp.
        lateral = p[:, 0] * sx
        high = _smooth(sh[1] - 0.3, sh[1] - 0.15, p[:, 1])
        lo = abs(sh[0]) * (0.92 - 0.27 * high)
        hi = abs(sh[0]) * (0.96 + 0.08 * high)
        whole = _smooth(lo, hi, lateral) * ((p[:, 1] < sh[1] + 0.15) & (p[:, 1] > ftip[1] - 0.02))
        # And only what is joined to the hand: a thigh's outside as far out
        # is not.
        whole = whole * _joined(whole > 0, ftip_i, nbrs)
        out[f"arm.{side}"] = (sh, mid["forearm"], mid["hand"], d, whole)
        # The leg: its cross-sections at MakeHuman's knee and ankle heights.
        hip, kn, an = (lm[f"{n}.{side}"] for n in ("thigh", "shin", "foot"))
        leg = (p[:, 0] * sx > 0.01) & (p[:, 1] < hip[1])
        knee = _centroid(p, leg & (np.abs(p[:, 1] - kn[1]) < 0.01))
        ankle = _centroid(p, leg & (np.abs(p[:, 1] - an[1]) < 0.01) & (p[:, 2] < an[2] + 0.04))
        out[f"thigh.{side}"] = (hip, knee, ankle, None, leg)
    return out


def body(spec):
    """``character.body`` for a spec with a ``BASE`` mesh: the base mesh on
    the game's skeleton, with MakeHuman's weights; returns what
    ``character.body`` does."""
    base = mh.Base(os.path.join(ROOT, "target/art/mh"))
    rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
    v = mh.shaped(base, spec.PROXY)
    sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), spec.CLASS)
    extras = {}
    # His hands open, as the base mesh's are: fitted closed, his fingers'
    # curl dragged its open ones into spikes.
    fitted, _, _, _, W_mh, bo, br, marks = fit.fit(base, rig, v, sk, None, 1.0, extras=extras)
    rest, lm = extras["rest"], extras["lm"]

    p, faces, uvs, face_uvs, eyes = load(spec.BASE)
    n = len(p)
    # The eyes' centres ride along as two more points through every step.
    p = np.vstack([p, eyes[:, :3]])
    # His height, standing where he stands, the trunk's depth on his.
    k = (rest[:, 1].max() - rest[:, 1].min()) / (p[:n, 1].max() - p[:n, 1].min())
    p = p * k
    p[:, 1] += rest[:, 1].min() - p[:n, 1].min()
    band = lambda q: np.abs(q[:, 1] - lm["chest"][1]) < 0.03
    p[:, 2] += rest[band(rest), 2].mean() - p[:n][band(p[:n]) & (np.abs(p[:n, 0]) < 0.12), 2].mean()

    # Each limb onto his.
    joints = _limbs(p[:n], lm, _neighbours(n, faces))
    before = p[:n].copy()
    if os.environ.get("BASE_DEBUG"):
        for key, val in joints.items():
            print("JOINTS", key, [None if x is None else np.round(x, 3).tolist() for x in val[:3]])
        for key in ("arm.l", "forearm.l", "hand.l", "handtip.l", "thigh.l", "shin.l", "foot.l"):
            print("LM", key, np.round(lm[key], 3).tolist())
    for side in ("l", "r"):
        sh, el, wr, _, mask = joints[f"arm.{side}"]
        mask = np.r_[mask, 0.0, 0.0]
        p = _chain(p, mask, (sh, el, wr), (lm[f"arm.{side}"], lm[f"forearm.{side}"], lm[f"hand.{side}"]),
                   root_blend=(0.0, 0.09))
        hip, kn, an, _, mask = joints[f"thigh.{side}"]
        mask = np.r_[mask, False, False]
        p = _chain(p, mask, (hip, kn, an), (lm[f"thigh.{side}"], lm[f"shin.{side}"], lm[f"foot.{side}"]),
                   root_blend=(-0.02, 0.08), joint_blend=0.05)

    # Each vertex takes the weights of his nearest.
    tree = kdtree.KDTree(len(rest))
    for i, q in enumerate(rest):
        tree.insert(q, i)
    tree.balance()
    W = np.zeros((len(p), W_mh.shape[1]))
    for i, q in enumerate(p):
        found = tree.find_n(q, 6)
        idx = np.array([f[1] for f in found])
        dist = np.array([f[2] for f in found])
        w = 1.0 / np.maximum(dist, 1e-4) ** 2
        W[i] = (w / w.sum()) @ W_mh[idx]
    W /= np.maximum(W.sum(axis=1, keepdims=True), 1e-9)
    # The hand is the hand joint's, all of it, from a little before our own
    # wrist: taken from his nearest, the thumb -- beside the wrist, against
    # his bent hand -- was shared with the forearm and smeared between them.
    for side in ("l", "r"):
        sh, el, wr, _, mask = joints[f"arm.{side}"]
        u = (wr - el) / np.linalg.norm(wr - el)
        t = (before - wr) @ u
        hand = _smooth(-0.04, 0.0, t) * (mask > 0.5)
        one = np.zeros(W.shape[1])
        one[fit.J[f"hand.{side}"]] = 1.0
        W[:n] = W[:n] * (1 - hand)[:, None] + one * hand[:, None]
    # And moves as his body moved, bone by bone: each of his vertices is
    # carried by a blend of its bones' maps (``fit``'s ``bone_map``), and
    # ours by the same maps and the same blend. A vertex is not dragged
    # after his nearest: a hand is carried whole, however its fingers lie.
    his_map, origin = extras["bone_map"], extras["origin"]
    # Except the trunk: his is mapped in height piece by piece between his
    # spine's joints and the game's, which stretched our belly to twice its
    # height and squashed the chest (a body that is not his has its own
    # proportions between them). Ours is one even map from the hips to the
    # base of the neck, meeting the legs' and the head's where they start.
    hip_his = (lm["thigh.l"] + lm["thigh.r"]) / 2
    hip_ours = (origin[fit.J["thigh.l"]] + origin[fit.J["thigh.r"]]) / 2
    neck_his, neck_ours = lm["head"], origin[fit.J["neck"]]
    stretch = (neck_ours[1] - hip_ours[1]) / (neck_his[1] - hip_his[1])

    def bone_map(name, q):
        if name in ("root", "spine", "chest", "clavicle.l", "clavicle.r"):
            out = hip_ours + (q - hip_his)
            out[:, 1] = hip_ours[1] + (q[:, 1] - hip_his[1]) * stretch
            return out
        return his_map(name, q)

    out = np.zeros_like(p)
    for j, name in enumerate(fit.JOINTS):
        m = W[:, j] > 0
        if m.any():
            out[m] += W[m, j:j + 1] * bone_map(name, p[m])

    # Its own eyes, where its sockets are: named as MakeHuman's marks are,
    # by which side of the face they are on.
    for eye in out[n:]:
        name = "eye.l" if np.sign(eye[0]) == np.sign(marks["eye.l"][0]) else "eye.r"
        marks[name] = eye
    marks["eye_radius"] = float(eyes[:, 3].mean() * k)
    if os.environ.get("BASE_DEBUG"):
        print("EYES in", np.round(p[n:], 3).tolist(), "out", np.round(out[n:], 3).tolist(),
              "W", [[(fit.JOINTS[j], round(float(x), 2)) for j, x in enumerate(row) if x > 0.05] for row in W[n:]])

    ob = stage.mesh_object("body", g.g2b(out[:n]), faces, uvs, face_uvs)
    g.set_weights(ob, W[:n])
    g.normalize(ob)
    return ob, sk, bo, br, marks
