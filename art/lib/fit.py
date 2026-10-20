"""MakeHuman's body, moved onto the game's skeleton.

The game poses a character by solving sixteen joints (``view::skeleton``),
every frame, from the simulation. A skinned mesh rides those joints if it was
made around them: each vertex is told which joints move it and by how much,
and where every joint was when the mesh was made (its *bind pose*). So this
does three things:

1. **Chooses a bind pose** on the game's own skeleton. The game's rest pose
   hangs the arms straight down; MakeHuman's body is modelled with them out
   at about forty-five degrees. Rather than bend the mesh, the bind pose bends
   the skeleton -- each limb turned to point where MakeHuman's does -- so the
   body is changed as little as possible. Nothing in the game needs to know:
   it hands over where the joints are, and the bind pose is folded into the
   inverse bind matrices.
2. **Fits the body to that pose.** The game's bones have their own lengths
   (the Reaver is long in the leg), so each part of the body is carried from
   its MakeHuman bone to the game's: turned, stretched along the bone, and
   blended across a joint by the same weights that will move it.
3. **Folds MakeHuman's 160 bones' weights into the game's 16.**

All positions here are metres, in the game's character frame: +y up, +z
forward, and -- because the arena is right-handed -- the body's own *left* at
+x. That is the skeleton slot the game names ``arm.r``: a pose is authored in
a left-handed frame and reflected when it is drawn, so the slot named for the
author's left arm is drawn as the body's right (``view::hand_joint``).
MakeHuman's body is right-handed too, with its left at +x, so it comes in as
it is, and its ``.L`` bones go to the game's ``.r`` slots.
"""

import json

import numpy as np

# Joint order, exactly `view::skeleton::JOINTS`.
JOINTS = [
    "root", "spine", "chest", "head",
    "arm.l", "forearm.l", "hand.l",
    "arm.r", "forearm.r", "hand.r",
    "thigh.l", "shin.l", "foot.l",
    "thigh.r", "shin.r", "foot.r",
]
J = {n: i for i, n in enumerate(JOINTS)}


def our_bone(mh_bone):
    """Which game joint moves what a MakeHuman bone moved.

    Returns ``[(joint, share)]``. The neck is split between the chest and the
    head, because the game's head turns at the base of the neck.
    """
    b = mh_bone
    side = None
    # The body's left is the slot named right; see the module's comment.
    if b.endswith(".L"):
        side, b = "r", b[:-2]
    elif b.endswith(".R"):
        side, b = "l", b[:-2]

    def s(name):
        return f"{name}.{side}"

    if b in ("root", "spine05") or b.startswith("pelvis"):
        return [("root", 1.0)]
    if b in ("spine04", "spine03"):
        return [("spine", 1.0)]
    if b in ("spine02", "spine01") or b.startswith("clavicle") or b.startswith("breast"):
        return [("chest", 1.0)]
    if b == "neck01":
        return [("chest", 0.5), ("head", 0.5)]
    if side and (b.startswith("shoulder") or b.startswith("upperarm")):
        return [(s("arm"), 1.0)]
    if side and b.startswith("lowerarm"):
        return [(s("forearm"), 1.0)]
    if side and (b.startswith("wrist") or b.startswith("finger") or b.startswith("metacarpal")):
        return [(s("hand"), 1.0)]
    if side and b.startswith("upperleg"):
        return [(s("thigh"), 1.0)]
    if side and b.startswith("lowerleg"):
        return [(s("shin"), 1.0)]
    if side and (b.startswith("foot") or b.startswith("toe")):
        return [(s("foot"), 1.0)]
    # Everything else is in the head: neck02-03, the jaw, the eyes, the face.
    return [("head", 1.0)]


# --- Quaternions, [x, y, z, w], composed as the game composes them ---------

def q_axis(axis, angle):
    a = np.asarray(axis, float)
    a = a / np.linalg.norm(a)
    s, c = np.sin(angle / 2), np.cos(angle / 2)
    return np.array([a[0] * s, a[1] * s, a[2] * s, c])


def q_mul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return np.array([
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ])


def q_rot(q, v):
    u = q[:3]
    w = q[3]
    t = 2 * np.cross(u, v)
    return v + w * t + np.cross(u, t)


def q_mat(q):
    x, y, z, w = q
    return np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ])


def q_between(a, b):
    """The shortest turn taking direction ``a`` onto ``b``."""
    a = a / np.linalg.norm(a)
    b = b / np.linalg.norm(b)
    d = float(np.dot(a, b))
    if d > 0.999999:
        return np.array([0.0, 0.0, 0.0, 1.0])
    if d < -0.999999:
        ax = np.cross(a, [1, 0, 0])
        if np.linalg.norm(ax) < 1e-6:
            ax = np.cross(a, [0, 1, 0])
        return q_axis(ax, np.pi)
    ax = np.cross(a, b)
    return q_axis(ax, np.arccos(d))


class Skeleton:
    """One class's skeleton, from ``cargo run -p anim --bin export``."""

    def __init__(self, export_json, class_name):
        with open(export_json) as f:
            d = json.load(f)
        self.parents = [j["parent"] for j in d["joints"]]
        names = [j["name"] for j in d["joints"]]
        assert names == JOINTS, names
        c = next(c for c in d["classes"] if c["name"] == class_name)
        self.scale = c["scale"]
        self.hip = c["hip"]
        self.height = c["height"]
        self.offset = [np.array(b["offset"]) for b in c["bones"]]
        self.axis = [np.array(b["axis"]) for b in c["bones"]]
        self.length = [b["length"] for b in c["bones"]]
        self.clips = {cl["name"]: cl for cl in d["clips"]}
        self.limits = [b["limits"] for b in c["bones"]]
        self.swing_sign = [b["swingSign"] for b in c["bones"]]

    def solve(self, rots, root_shift=(0, 0, 0)):
        """Forward kinematics with each joint's *local* rotation given."""
        origin = [None] * 16
        rot = [None] * 16
        for i in range(16):
            p = self.parents[i]
            if p < 0:
                origin[i] = self.offset[i] + np.asarray(root_shift) * self.scale
                rot[i] = rots[i]
            else:
                origin[i] = origin[p] + q_rot(rot[p], self.offset[i])
                rot[i] = q_mul(rot[p], rots[i])
                rot[i] /= np.linalg.norm(rot[i])
        return origin, rot

    def local(self, i, swing, spread, twist):
        """`view::skeleton::local_rotation`, in degrees, with its clamp."""
        (s0, s1), (p0, p1), (t0, t1) = self.limits[i]
        swing = min(max(swing, s0), s1)
        spread = min(max(spread, p0), p1)
        twist = min(max(twist, t0), t1)
        mirror = -1.0 if JOINTS[i].endswith(".l") else 1.0
        r = np.radians
        return q_mul(
            q_mul(q_axis([0, 0, 1], r(spread * mirror)),
                  q_axis([1, 0, 0], r(swing * self.swing_sign[i]))),
            q_axis([0, 1, 0], r(twist * mirror)),
        )

    def pose(self, frame):
        """A baked frame (hips in mm, then three angles a joint) to joints."""
        shift = np.array(frame[0:3]) / 1000.0
        rots = [self.local(i, *frame[3 + 3 * i: 6 + 3 * i]) for i in range(16)]
        return self.solve(rots, shift)

    def tip(self, origin, rot, i):
        return origin[i] + q_rot(rot[i], self.axis[i] * self.length[i])


def mirror_mh(verts):
    """MakeHuman's decimetres to metres. (Named for what it used to do: the
    body is not mirrored, see the module's comment.)"""
    return verts * 0.1


def landmarks(base, rig, v):
    """MakeHuman's joints, in metres, mirrored. Keyed by game joint."""

    def h(bone):
        return mirror_mh(rig.head(v, bone)[None])[0]
    lm = {}
    for side, S in (("l", "R"), ("r", "L")):
        lm[f"arm.{side}"] = h(f"upperarm01.{S}")
        lm[f"forearm.{side}"] = h(f"lowerarm01.{S}")
        lm[f"hand.{side}"] = h(f"wrist.{S}")
        lm[f"handtip.{side}"] = mirror_mh(rig.tail(v, f"finger3-3.{S}")[None])[0]
        lm[f"thigh.{side}"] = h(f"upperleg01.{S}")
        lm[f"shin.{side}"] = h(f"lowerleg01.{S}")
        lm[f"foot.{side}"] = h(f"foot.{S}")
        lm[f"toe.{side}"] = mirror_mh(rig.tail(v, f"toe3-1.{S}")[None])[0]
    lm["spine"] = h("spine04")
    lm["chest"] = h("spine02")
    lm["head"] = h("neck02")
    return lm


def fit(base, rig, v, sk, girth=None, head_scale=1.0, carried=None):
    """The body on the game's skeleton, in a bind pose chosen to suit it.

    Returns ``(verts, faces, face_uvs, uvs, weights, bind_origin, bind_rot)``
    with ``weights`` an ``(n, 16)`` array summing to one per vertex, and only
    the skin's own vertices kept (renumbered).

    The joints are found on ``v``; the vertices moved are ``carried`` if
    given -- the same body with its fingers closed, say, which would
    otherwise drag the hand's bone toward the curled fingertips.
    """
    girth = girth or {}
    body = base.body_faces()
    keep = sorted({i for f in body for i in base.faces[f]})
    remap = {old: new for new, old in enumerate(keep)}
    faces = [[remap[i] for i in base.faces[f]] for f in body]
    face_uvs = [list(base.face_uvs[f]) for f in body]

    mv = mirror_mh(v if carried is None else carried)
    ground = mv[keep][:, 1].min()
    lm = landmarks(base, rig, v)
    pts = mv[keep].copy()

    # One overall scale first, so the per-bone fit is only the residue: the
    # shoulders' height against the game's.
    rest_o, rest_r = sk.solve([np.array([0, 0, 0, 1.0])] * 16)
    k = rest_o[J["arm.r"]][1] / (lm["arm.r"][1] - ground)
    shift = np.array([0.0, -ground, 0.0])
    pts = (pts + shift) * k
    for key in lm:
        lm[key] = (lm[key] + shift) * k

    # The bind pose: every limb turned to point where MakeHuman's points.
    rots = [np.array([0, 0, 0, 1.0])] * 16
    origin, rot = sk.solve(rots)
    chains = []
    for side in ("l", "r"):
        chains += [
            (f"arm.{side}", f"forearm.{side}"),
            (f"forearm.{side}", f"hand.{side}"),
            (f"hand.{side}", f"handtip.{side}"),
            (f"thigh.{side}", f"shin.{side}"),
            (f"shin.{side}", f"foot.{side}"),
            (f"foot.{side}", f"toe.{side}"),
        ]
    for a, b in chains:
        i = J[a]
        want = lm[b] - lm[a]
        if a.startswith("foot"):
            want = np.array([want[0], 0.0, want[2]])  # a foot lies flat
        p = sk.parents[i]
        world = q_between(sk.axis[i], want)
        # local = parent^-1 * world
        pinv = rot[p] * np.array([-1, -1, -1, 1])
        rots[i] = q_mul(pinv, world)
        origin, rot = sk.solve(rots)

    # Each game bone's map: MakeHuman's segment onto the game's.
    seg_mh = {}
    seg_us = {}
    for a, b in chains:
        seg_mh[a] = (lm[a], lm[b])
        seg_us[a] = (origin[J[a]], sk.tip(origin, rot, J[a]) if not a.startswith(("hand", "foot")) else None)
    # Limbs: the far end is the next joint, except the hand and foot, which
    # keep their shape and are only carried.
    for side in ("l", "r"):
        for a, b in ((f"arm.{side}", f"forearm.{side}"), (f"forearm.{side}", f"hand.{side}"),
                     (f"thigh.{side}", f"shin.{side}"), (f"shin.{side}", f"foot.{side}")):
            seg_us[a] = (origin[J[a]], origin[J[b]])
    # The trunk is upright in both: a map in height alone, piecewise between
    # the hip joints, the waist, the chest and the neck.
    hipmid_mh = (lm["thigh.l"] + lm["thigh.r"]) / 2
    hipmid_us = (origin[J["thigh.l"]] + origin[J["thigh.r"]]) / 2
    trunk = {
        "root": (hipmid_mh, lm["spine"], hipmid_us, origin[J["spine"]]),
        "spine": (lm["spine"], lm["chest"], origin[J["spine"]], origin[J["chest"]]),
        "chest": (lm["chest"], lm["head"], origin[J["chest"]], origin[J["head"]]),
    }

    def bone_map(name, p):
        g = girth.get(name, 1.0)
        if name in trunk:
            a, b, A, B = trunk[name]
            s = (B[1] - A[1]) / (b[1] - a[1])
            d = p - a
            out = np.empty_like(p)
            out[:, 1] = A[1] + d[:, 1] * s
            out[:, 0] = A[0] + d[:, 0] * g
            out[:, 2] = A[2] + d[:, 2] * g
            return out
        if name == "head":
            a, A = lm["head"], origin[J["head"]]
            return A + (p - a) * head_scale
        a, b = seg_mh[name]
        A, B = seg_us[name]
        if B is None:
            # Carried whole: turned with its bone, not stretched.
            q = q_between(b - a, q_rot(rot[J[name]], sk.axis[J[name]]))
            return A + (p - a) @ q_mat(q).T
        u = (b - a) / np.linalg.norm(b - a)
        s = np.linalg.norm(B - A) / np.linalg.norm(b - a)
        q = q_between(b - a, B - A)
        d = p - a
        along = d @ u
        perp = d - np.outer(along, u)
        local = np.outer(along * s, u) + perp * g
        return A + local @ q_mat(q).T

    # Weights folded into the game's joints.
    W = np.zeros((len(keep), 16))
    for bone, (idx, w) in rig.weights.items():
        for joint, share in our_bone(bone):
            for vi, wi in zip(idx, w):
                if vi in remap:
                    W[remap[vi], J[joint]] += wi * share
    W /= np.maximum(W.sum(axis=1, keepdims=True), 1e-9)

    out = np.zeros_like(pts)
    for name in JOINTS:
        j = J[name]
        m = W[:, j] > 0
        if m.any():
            out[m] += W[m, j:j + 1] * bone_map(name, pts[m])

    # Points on the face the costume is cut round, carried as the head is.
    def head_point(mh_point):
        p = (mirror_mh(np.asarray(mh_point)[None])[0] + shift) * k
        return bone_map("head", p[None])[0]

    cv = v if carried is None else carried

    def hand_point(side, mh_point):
        p = (mirror_mh(np.asarray(mh_point)[None])[0] + shift) * k
        return bone_map(f"hand.{side}", p[None])[0]

    marks = {
        # Named by slot, like the joints: the body's left eye is ``eye.r``.
        "eye.l": head_point(v[_group_verts(base, "joint-r-eye")].mean(0)),
        "eye.r": head_point(v[_group_verts(base, "joint-l-eye")].mean(0)),
        "eye_radius": k * 0.1 * _radius(v[_group_verts(base, "joint-r-eye")]),
    }
    for side, S in (("l", "R"), ("r", "L")):
        # The grip: the hollow the closed fingers make, and which way a handle
        # held in it runs (from the little finger's side to the index's).
        knuckles = [rig.head(cv, f"finger{f}-1.{S}") for f in (2, 3, 4, 5)]
        tips = [rig.tail(cv, f"finger{f}-2.{S}") for f in (2, 3, 4, 5)]
        index = hand_point(side, knuckles[0])
        little = hand_point(side, knuckles[3])
        hollow = hand_point(side, (np.mean(knuckles, 0) + np.mean(tips, 0)) / 2)
        wrist = hand_point(side, rig.head(cv, f"wrist.{S}"))
        marks[f"grip.{side}"] = hollow
        marks[f"grip_axis.{side}"] = (index - little) / np.linalg.norm(index - little)
        marks[f"wrist.{side}"] = wrist
    uvs = base.uvs
    return out, faces, face_uvs, uvs, W, origin, rot, marks


def _group_verts(base, group):
    return sorted({i for f, g in zip(base.faces, base.groups) if g == group for i in f})


def _radius(pts):
    c = pts.mean(0)
    return float(np.linalg.norm(pts - c, axis=1).max())
