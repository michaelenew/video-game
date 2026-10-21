"""MakeHuman's base body, read straight from its files.

MakeHuman (makehumancommunity.org) is a parametric human: one base mesh of
about thirteen thousand quads with good topology, and a library of *targets*
-- per-vertex offsets -- that move it toward a gender, an age, a build or a
proportion. Its mesh, targets, skeleton and weights are all CC0, so we may
take them, change them and ship them with no strings.

This module needs only numpy. It reads:

- ``3dobjs/base.obj``: the mesh, in decimetres, facing +z, its left at +x;
- ``targets/**/*.target``: lines of ``vertex dx dy dz``;
- ``rigs/default.mhskel``: bones, each end a joint named by a set of vertices
  whose middle it is (helper cubes inside the body);
- ``rigs/default_weights.mhw``: how much each bone moves each vertex.

Everything is returned in MakeHuman's own frame and units; ``fit.py`` moves it
onto the game's skeleton.
"""

import json
import os

import numpy as np


class Base:
    """The base mesh: every vertex (body and helpers), its faces by group."""

    def __init__(self, root):
        self.root = root
        verts, uvs, faces, face_uvs, groups = [], [], [], [], []
        group = None
        with open(os.path.join(root, "3dobjs", "base.obj")) as f:
            for line in f:
                if line.startswith("v "):
                    verts.append([float(x) for x in line.split()[1:4]])
                elif line.startswith("vt "):
                    uvs.append([float(x) for x in line.split()[1:3]])
                elif line.startswith("g "):
                    group = line.split()[1]
                elif line.startswith("f "):
                    idx = [p.split("/") for p in line.split()[1:]]
                    faces.append([int(p[0]) - 1 for p in idx])
                    face_uvs.append([int(p[1]) - 1 if len(p) > 1 and p[1] else -1 for p in idx])
                    groups.append(group)
        self.verts = np.array(verts, dtype=np.float64)
        self.uvs = np.array(uvs, dtype=np.float64)
        self.faces = faces
        self.face_uvs = face_uvs
        self.groups = groups

    def target(self, rel, weight=1.0, into=None):
        """Add ``weight`` of the target at ``rel`` (under ``targets/``)."""
        out = self.verts.copy() if into is None else into
        path = os.path.join(self.root, "targets", rel)
        data = np.loadtxt(path, comments="#", ndmin=2)
        if data.size == 0:
            # The average of every slider is the base mesh itself, and its
            # target file is empty.
            return out
        idx = data[:, 0].astype(int)
        out[idx] += weight * data[:, 1:4]
        return out

    def body_faces(self):
        """The faces of the skin itself, without the helper geometry."""
        return [i for i, g in enumerate(self.groups) if g == "body"]


class Rig:
    """MakeHuman's default skeleton: where each joint is, for a given mesh."""

    def __init__(self, root):
        with open(os.path.join(root, "rigs", "default.mhskel")) as f:
            d = json.load(f)
        self.bones = d["bones"]
        self.joints = {k: np.array(v, dtype=int) for k, v in d["joints"].items()}
        with open(os.path.join(root, "rigs", "default_weights.mhw")) as f:
            w = json.load(f)
        self.weights = {
            bone: (np.array([p[0] for p in pairs], dtype=int), np.array([p[1] for p in pairs]))
            for bone, pairs in w["weights"].items()
        }

    def joint(self, verts, name):
        return verts[self.joints[name]].mean(axis=0)

    def head(self, verts, bone):
        return self.joint(verts, self.bones[bone]["head"])

    def tail(self, verts, bone):
        return self.joint(verts, self.bones[bone]["tail"])


def shaped(base, recipe):
    """The base mesh with a recipe's targets added: ``[(path, weight)]``."""
    v = base.verts.copy()
    for rel, w in recipe:
        if w:
            base.target(rel, w, into=v)
    return v


def macro(gender="female", age="young", muscle=0.5, weight=0.5, races=None, proportions=0.0):
    """MakeHuman's macro sliders as a list of targets, the way it blends them.

    ``muscle`` and ``weight`` run 0 to 1 with 0.5 the average, as in the
    program: below the middle they blend toward ``min``, above toward ``max``.
    """

    def split(x):
        if x >= 0.5:
            t = (x - 0.5) * 2
            return [("average", 1 - t), ("max", t)]
        t = (0.5 - x) * 2
        return [("average", 1 - t), ("min", t)]

    out = []
    for m, wm in split(muscle):
        for ww_name, ww in split(weight):
            out.append(
                (
                    f"macrodetails/universal-{gender}-{age}-{m}muscle-{ww_name}weight.target",
                    wm * ww,
                )
            )
    races = races or {"caucasian": 1 / 3, "african": 1 / 3, "asian": 1 / 3}
    for race, w in races.items():
        out.append((f"macrodetails/{race}-{gender}-{age}.target", w))
    if proportions:
        for m, wm in split(muscle):
            for ww_name, ww in split(weight):
                out.append(
                    (
                        f"macrodetails/proportions/{gender}-{age}-{m}muscle-{ww_name}weight-idealproportions.target",
                        proportions * wm * ww,
                    )
                )
    return out


def _rotation(axis, angle):
    a = axis / np.linalg.norm(axis)
    x, y, z = a
    c, s = np.cos(angle), np.sin(angle)
    C = 1 - c
    return np.array([
        [c + x * x * C, x * y * C - z * s, x * z * C + y * s],
        [y * x * C + z * s, c + y * y * C, y * z * C - x * s],
        [z * x * C - y * s, z * y * C + x * s, c + z * z * C],
    ])


def grip(base, rig, verts, curl=(62.0, 75.0, 45.0), thumb=(25.0, 30.0, 20.0), sides=("L", "R")):
    """Close the hands round a grip: each finger bone turned about its own
    knuckle by MakeHuman's weights, the way a pose is applied in MakeHuman.

    ``curl`` is the bend at each of a finger's three joints, in degrees, and
    ``thumb`` the same for the thumb, which closes across the palm rather than
    into it. The game's hand is one bone, so the grip is modelled in.
    """
    v = verts.copy()
    for S in sides:
        wrist_h = rig.head(verts, f"wrist.{S}")
        mid = rig.head(verts, f"finger3-1.{S}")
        index = rig.head(verts, f"finger2-1.{S}")
        little = rig.head(verts, f"finger5-1.{S}")
        across = index - little
        along = mid - wrist_h
        # The palm faces where the fingers fold toward: for MakeHuman's
        # A-pose hands, down and a little back.
        palm = np.cross(along, across)
        if palm[1] > 0:
            palm = -palm
        palm /= np.linalg.norm(palm)
        moved = {}

        def bone_tf(bone):
            if bone in moved:
                return moved[bone]
            parent = rig.bones[bone]["parent"]
            if not (parent.startswith("finger") or parent.startswith("metacarpal")):
                pt = (np.eye(3), np.zeros(3))
            else:
                pt = bone_tf(parent)
            name = bone.split(".")[0]
            if name.startswith("finger"):
                f, seg = name[6:].split("-")
                seg = int(seg) - 1
                head = rig.head(verts, bone)
                tail = rig.tail(verts, bone)
                d = tail - head
                if f == "1":
                    axis = np.cross(d, palm) * 0.6 + np.cross(across, palm) * 0.0 + along * 0.0
                    axis = np.cross(palm, d) if np.linalg.norm(axis) < 1e-9 else axis
                    ang = np.radians(thumb[seg])
                    axis = np.cross(d, palm + across / np.linalg.norm(across) * 0.6)
                else:
                    axis = np.cross(d, palm)
                    ang = np.radians(curl[seg])
                R = _rotation(axis, ang)
                # x' = R (x - head) + head, after the parent's motion.
                pR, pt_ = pt
                R2 = pR @ R
                t2 = pR @ (head - R @ head) + pt_
                moved[bone] = (R2, t2)
            else:
                moved[bone] = pt
            return moved[bone]

        total = np.zeros(len(v))
        acc = np.zeros_like(v)
        for bone, (idx, w) in rig.weights.items():
            if not bone.endswith(f".{S}"):
                continue
            if not (bone.startswith("finger") or bone.startswith("metacarpal")):
                continue
            R, t = bone_tf(bone)
            acc[idx] += w[:, None] * (verts[idx] @ R.T + t)
            total[idx] += w
        m = total > 0
        v[m] = acc[m] + (1 - total[m])[:, None] * verts[m]
    return v


def flatten_bust(base, rig, verts, flat, amount):
    """The bust pressed toward a flat chest: each vertex the breasts' own
    bones move is blended toward ``flat`` (the same body built male) by
    ``amount`` times their weight -- a half for one compressed shape under a
    fitted top, all of it for none. The bones' weights fade out over the
    ribs, so the blend has no edge."""
    mask = np.zeros(len(verts))
    for bone in ("breast.L", "breast.R"):
        idx, w = rig.weights[bone]
        mask[idx] = np.maximum(mask[idx], w)
    mask = np.clip(mask * 3.0, 0, 1) * amount
    out = verts + mask[:, None] * (flat - verts)
    # Then smoothed over the same region, a few rounds, so nothing the flat
    # chest still carries -- a nipple, a pectoral's edge -- stands out of a
    # fitted top.
    edges = []
    for f in base.faces:
        for i in range(len(f)):
            edges.append((f[i], f[(i + 1) % len(f)]))
    e = np.array(edges)
    n = len(verts)
    deg = np.zeros(n)
    np.add.at(deg, e[:, 0], 1)
    np.add.at(deg, e[:, 1], 1)
    for _ in range(6):
        acc = np.zeros_like(out)
        np.add.at(acc, e[:, 0], out[e[:, 1]])
        np.add.at(acc, e[:, 1], out[e[:, 0]])
        avg = acc / np.maximum(deg, 1)[:, None]
        out = out + (mask * 0.6)[:, None] * (avg - out)
    return out
