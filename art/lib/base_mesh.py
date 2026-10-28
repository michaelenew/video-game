"""A body from a sculpted base mesh, used as it was made.

MakeHuman's bodies never read as a person in the game; Blender Studio's
realistic male does (``art/sources``, CC0). And it is used **as Blender made
it** (the owner, 2026-10-28: "The blender model is definitely modified from
what comes directly from blender, put it back"): fitted onto a skeleton
that was not his, his shoulders were pulled in five centimetres a side and
his trunk stretched. So nothing here moves a vertex. The skeleton is built
to him instead (``view::being::skeleton``, exported as ``DarkBeing``):

1. his joints are found on the mesh (``joints``) -- the middles of his
   limbs' cross-sections, where an anatomy's proportions put them;
2. the skeleton's bind pose turns each limb to lie along his;
3. his weights are Blender's own automatic ones, from an armature at that
   bind pose.

Everything is in the game's frame (``+y`` up, ``+z`` forward), metres.
"""

import os

import bpy
import numpy as np

import fit
import garment as g
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


def joints(p):
    """His joints, keyed by game joint: each limb's joints are the middles
    of its cross-sections, at the lengths of an adult's segments as shares
    of his height (Drillis and Contini): wrist 0.108 and elbow 0.254 back
    up the arm from the fingertip, the upper arm 0.186 long; knee 0.285 and
    ankle 0.045 of the height up the leg, the hip joint 0.53."""
    H = p[:, 1].max()
    out = {}
    for side, sx in (("l", -1.0), ("r", 1.0)):
        arm = (p[:, 0] * sx > 0.2) & (p[:, 1] > 0.6) & (p[:, 1] < 1.5)
        tip = p[arm][np.argmin(p[arm, 1])]
        rings = []
        for y in np.arange(tip[1] + 0.02, 1.45, 0.02):
            m = arm & (np.abs(p[:, 1] - y) < 0.01)
            if m.sum() > 5:
                rings.append(p[m].mean(0))
        pts = np.vstack([tip, rings])
        run = np.r_[0, np.cumsum(np.linalg.norm(np.diff(pts, axis=0), axis=1))]

        def along(length):
            return np.array([np.interp(length, run, pts[:, k]) for k in range(3)])
        wrist, elbow = along(0.108 * H), along(0.254 * H)
        up = along(0.33 * H) - elbow
        up /= np.linalg.norm(up)
        out[f"arm.{side}"] = elbow + up * 0.186 * H
        out[f"forearm.{side}"] = elbow
        out[f"hand.{side}"] = wrist
        out[f"handtip.{side}"] = tip
        leg = (p[:, 0] * sx > 0.0) & (p[:, 1] < 0.9) & (np.abs(p[:, 0]) < 0.3)

        def ring(y):
            return p[leg & (np.abs(p[:, 1] - y) < 0.008)].mean(0)
        knee, ankle, mid = ring(0.285 * H), ring(0.045 * H), ring(0.45 * H)
        d = (mid - knee) / np.linalg.norm(mid - knee)
        out[f"thigh.{side}"] = knee + d * (0.53 * H - knee[1]) / d[1]
        out[f"shin.{side}"] = knee
        out[f"foot.{side}"] = ankle
        foot = leg & (p[:, 1] < 0.05)
        out[f"toe.{side}"] = p[foot][np.argmax(p[foot, 2])]
    return out


def _bind(sk, lm):
    """The skeleton's bind pose with every limb turned to lie along his,
    as ``fit.fit`` turns them along MakeHuman's."""
    J = fit.J
    rots = [np.array([0, 0, 0, 1.0])] * len(fit.JOINTS)
    origin, rot = sk.solve(rots)
    for side in ("l", "r"):
        for a, b in ((f"arm.{side}", f"forearm.{side}"), (f"forearm.{side}", f"hand.{side}"),
                     (f"hand.{side}", f"handtip.{side}"), (f"thigh.{side}", f"shin.{side}"),
                     (f"shin.{side}", f"foot.{side}"), (f"foot.{side}", f"toe.{side}")):
            i = J[a]
            want = lm[b] - lm[a]
            if a.startswith("foot"):
                want = np.array([want[0], 0.0, want[2]])  # a foot lies flat
            parent = J["chest"] if a.startswith("arm.") else sk.parents[i]
            world = fit.q_between(sk.axis[i], want)
            rots[i] = fit.q_mul(rot[parent] * np.array([-1, -1, -1, 1]), world)
            origin, rot = sk.solve(rots)
    return origin, rot


def _weights(ob, sk, origin, rot):
    """Blender's automatic weights (bone heat) from an armature at the bind
    pose; any vertex the heat left unweighted takes its nearest bone."""
    arm_data = bpy.data.armatures.new("bind")
    rig = bpy.data.objects.new("bind", arm_data)
    bpy.context.collection.objects.link(rig)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    heads, tails = {}, {}
    for i, name in enumerate(fit.JOINTS):
        head = origin[i]
        tail = sk.tip(origin, rot, i)
        if np.linalg.norm(tail - head) < 1e-3:
            tail = head + np.array([0.0, 0.05, 0.0])
        bone = arm_data.edit_bones.new(name)
        bone.head = tuple(g.g2b(np.asarray(head)))
        bone.tail = tuple(g.g2b(np.asarray(tail)))
        heads[name], tails[name] = head, tail
    for i, name in enumerate(fit.JOINTS):
        if sk.parents[i] >= 0:
            arm_data.edit_bones[name].parent = arm_data.edit_bones[fit.JOINTS[sk.parents[i]]]
    bpy.ops.object.mode_set(mode="OBJECT")
    for o in bpy.context.selected_objects:
        o.select_set(False)
    ob.select_set(True)
    rig.select_set(True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    n = len(ob.data.vertices)
    W = np.zeros((n, len(fit.JOINTS)))
    index = {vg.index: vg.name for vg in ob.vertex_groups}
    for v in ob.data.vertices:
        for gr in v.groups:
            name = index[gr.group]
            if name in fit.J:
                W[v.index, fit.J[name]] = gr.weight
    co = np.zeros(n * 3)
    ob.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    empty = W.sum(axis=1) < 1e-6
    if empty.any():
        dist = np.full((empty.sum(), len(fit.JOINTS)), np.inf)
        for name, j in fit.J.items():
            a, b = heads[name], tails[name]
            ab = b - a
            t = np.clip(((p[empty] - a) @ ab) / max(ab @ ab, 1e-9), 0, 1)
            dist[:, j] = np.linalg.norm(p[empty] - (a + np.outer(t, ab)), axis=1)
        W[np.flatnonzero(empty), np.argmin(dist, axis=1)] = 1.0
    print(f"  weights: {int(empty.sum())} of {n} vertices left to their nearest bone", flush=True)
    # Off the armature again: the exporter writes the skeleton itself.
    ob.parent = None
    for m in list(ob.modifiers):
        ob.modifiers.remove(m)
    for vg in list(ob.vertex_groups):
        ob.vertex_groups.remove(vg)
    bpy.data.objects.remove(rig)
    bpy.data.armatures.remove(arm_data)
    return W / np.maximum(W.sum(axis=1, keepdims=True), 1e-9)


def body(spec):
    """``character.body`` for a spec with a ``BASE`` mesh: the base mesh,
    every vertex where its maker left it, on its own skeleton (``CLASS``,
    built to it); returns what ``character.body`` does."""
    sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), spec.CLASS)
    p, faces, uvs, face_uvs, eyes = load(spec.BASE)
    # Placed only: standing on the floor, its trunk's middle over the
    # skeleton's. Never scaled or bent.
    lift = -p[:, 1].min()
    chest = np.abs(p[:, 1] - 1.3) < 0.02
    shift = np.array([0.0, lift, -(p[chest, 2].max() + p[chest, 2].min()) / 2])
    p = p + shift
    eyes[:, :3] += shift
    lm = joints(p)
    origin, rot = _bind(sk, lm)
    ob = stage.mesh_object("body", g.g2b(p), faces, uvs, face_uvs)
    W = _weights(ob, sk, origin, rot)
    g.set_weights(ob, W)
    g.normalize(ob)
    marks = {"eye_radius": float(eyes[:, 3].mean())}
    for c in eyes[:, :3]:
        marks["eye.l" if c[0] < 0 else "eye.r"] = c
    for side in ("l", "r"):
        wrist, tip = lm[f"hand.{side}"], lm[f"handtip.{side}"]
        marks[f"wrist.{side}"] = wrist
        marks[f"grip.{side}"] = wrist + 0.45 * (tip - wrist)
        marks[f"grip_axis.{side}"] = np.array([0.0, 0.0, 1.0])
    return ob, sk, origin, rot, marks
