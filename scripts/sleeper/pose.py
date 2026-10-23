"""The Sleeper's pose, and the sheet that shows it.

    python3 scripts/sleeper/pose.py [out.png]

Starts from the last frame of the model's own "Die" clip -- the body down
on the ground, legs folded -- and points bones from there. Every aim is
a **world** direction (x is the dragon's left, y up, z the way it faces),
because "curl the tail round to the right" is a statement about the world,
not about a bone's local frame. Then the whole body is rolled onto its
right side.

What the owner asked for (2026-10-23): tail curled up; head tucked under a
forelimb, looking natural; forelimbs attached at the shoulder and half the
size; lying slightly on one side, so it is not symmetrical; one hind limb a
ridge along the flank; a tarrasque more than a European dragon. The body
itself -- wings off, spikes down, the head reshaped -- is `sculpt.py`'s.
"""
import sys

import numpy as np

from model import Model, axis_angle, quat_matrix, quat_mul
from sculpt import sculpt

# How far over onto its right side the whole body lies, degrees.
ROLL = 22.0

# Where each bone points, in order from the body out: (bone, the bone it
# points at, a world direction). Each turns that bone, and everything
# hanging from it, so its child lies along that direction from it. The
# body itself stays as the clip left it: down, legs folded.
AIMS = [
    # The neck: round to the left and back, so the head lies along the left
    # shoulder, as an animal asleep turns its head in to its side.
    ("neck_01", "neck_02", (0.55, 0.05, 0.85)),
    ("neck_02", "neck_03", (0.95, -0.2, 0.25)),
    ("neck_03", "neck_04", (0.75, -0.45, -0.45)),
    ("neck_04", "neck_05", (0.35, -0.55, -0.75)),
    ("neck_05", "head", (0.15, -0.5, -0.85)),
    # The head: on the ground along the flank, muzzle back toward the
    # haunch, under the left forearm.
    ("head", "fire_socket", (0.05, -0.2, -1.0)),
    # The left forelimb, uppermost: out of the shoulder and laid over the
    # head.
    ("upperarm.L", "forearm.L", (0.85, 0.35, 0.35)),
    ("forearm.L", "hand.L", (0.55, -0.45, -0.7)),
    ("hand.L", "fingers.L", (0.2, -0.6, -0.75)),
    # The right forelimb, under: folded back along the chest.
    ("upperarm.R", "forearm.R", (-0.5, -0.4, 0.75)),
    ("forearm.R", "hand.R", (0.4, -0.3, 0.85)),
    # The left hind leg, uppermost, drawn up along the flank: a ridge.
    ("thigh.L", "shin.L", (0.55, -0.25, 0.8)),
    ("shin.L", "foot.L", (0.35, -0.35, -0.85)),
    # The right hind leg, under the body.
    ("thigh.R", "shin.R", (-0.3, -0.6, 0.7)),
    ("shin.R", "foot.R", (-0.2, -0.4, -0.9)),
    # The tail: back, round the right side and forward along it, the tip
    # turned in.
    ("tail_01", "tail_02", (-0.3, -0.4, -0.9)),
    ("tail_02", "tail_03", (-0.8, -0.3, -0.5)),
    ("tail_03", "tail_04", (-1.0, -0.1, 0.2)),
    ("tail_04", "tail_05", (-0.45, 0.0, 0.9)),
    ("tail_05", "tail_06", (0.15, 0.0, 1.0)),
]

# Bones scaled: (bone, factor). Scaling a bone scales what hangs from it.
SCALES = [
    ("upperarm.L", 0.65),
    ("upperarm.R", 0.65),
    # A T. rex's legs: longer than the dragon's.
    ("thigh.L", 1.35),
    ("thigh.R", 1.35),
]

# How far forward the forelimbs leave the body, metres of the model: the
# shoulders moved up the chest, so the arms reach from in front.
SHOULDERS_FORWARD = 0.2


def turn(model, pose, bone, axis, degrees):
    """Turn `bone` by `degrees` about a world `axis`, in place."""
    i = model.index[bone]
    world = model.world(pose)
    p = model.parent[i]
    parent = world[p][:3, :3] if p >= 0 else np.eye(3)
    # The parent's pure rotation (its scale taken out).
    parent = parent / np.linalg.norm(parent, axis=0, keepdims=True)
    local_axis = parent.T @ np.asarray(axis, float)
    t, r, s = pose[i]
    pose[i] = (t, quat_mul(axis_angle(local_axis, degrees), r), s)


def aim(model, pose, bone, child, direction):
    """Turn `bone` so `child` lies along a world `direction` from it."""
    at = model.bone_points(pose)
    now = at[child] - at[bone]
    now /= np.linalg.norm(now)
    want = np.asarray(direction, float)
    want /= np.linalg.norm(want)
    axis = np.cross(now, want)
    if np.linalg.norm(axis) < 1e-9:
        return
    degrees = np.degrees(np.arccos(np.clip(now @ want, -1, 1)))
    turn(model, pose, bone, axis, degrees)


def shift(model, pose, bone, offset):
    """Move `bone` (and what hangs from it) by a world `offset`, in place."""
    i = model.index[bone]
    world = model.world(pose)
    parent = world[model.parent[i]][:3, :3]
    t, r, s = pose[i]
    pose[i] = (t + np.linalg.solve(parent, np.asarray(offset, float)), r, s)


def proportioned(model, pose):
    """The limbs' sizes and where the forelimbs leave the body, in place."""
    for bone, k in SCALES:
        i = model.index[bone]
        t, r, s = pose[i]
        pose[i] = (t, r, s * k)
    world = model.world(pose)
    chest = world[model.index["chest"]][:3, :3]
    forward = chest[:, 1] / np.linalg.norm(chest[:, 1])
    for side in ("clavicle.L", "clavicle.R"):
        shift(model, pose, side, forward * SHOULDERS_FORWARD)


def posed(model):
    """The Sleeper's pose: the clip's end, the proportions, the aims."""
    pose = model.clip_end("Die")
    proportioned(model, pose)
    for bone, child, direction in AIMS:
        aim(model, pose, bone, child, direction)
    # Its mouth shut: the jaw back to where the skull's own line is.
    i = model.index["jaw"]
    t, _, s = pose[i]
    pose[i] = (t, np.array([0.0, 0.0, 0.0, 1.0]), s)
    return pose


# **Standing**, as it is when it wakes (the owner: "make it stand on two
# legs"; then "the pose and proportions should be like a T rex with bigger
# arms", "hunch it a little more"): the body pitched up only this many
# degrees about the hips, near level and balanced over them; the legs bent
# under the hips; the tail held straight out behind; the neck in a low S
# and the head forward; the arms folded up at the chest.
REAR = 22.0
STAND_AIMS = [
    # Long, bent legs: the knee forward, the shin back, and up on its toes.
    ("thigh.L", "shin.L", (0.1, -0.6, 0.8)),
    ("thigh.R", "shin.R", (-0.1, -0.6, 0.8)),
    ("shin.L", "foot.L", (0.0, -0.75, -0.65)),
    ("shin.R", "foot.R", (0.0, -0.75, -0.65)),
    ("foot.L", "toes.L", (0.05, -0.85, 0.5)),
    ("foot.R", "toes.R", (-0.05, -0.85, 0.5)),
    ("tail_01", "tail_02", (0.0, -0.2, -1.0)),
    ("tail_02", "tail_03", (0.0, -0.08, -1.0)),
    ("tail_03", "tail_04", (0.0, 0.0, -1.0)),
    ("tail_04", "tail_05", (0.0, 0.03, -1.0)),
    ("tail_05", "tail_06", (0.0, 0.06, -1.0)),
    ("neck_01", "neck_02", (0.0, 0.5, 0.85)),
    ("neck_02", "neck_03", (0.0, 0.3, 0.95)),
    ("neck_03", "neck_04", (0.0, 0.1, 1.0)),
    ("neck_04", "neck_05", (0.0, -0.05, 1.0)),
    ("neck_05", "head", (0.0, -0.15, 1.0)),
    ("head", "fire_socket", (0.0, -0.25, 1.0)),
    # The arms folded up at the chest: the elbow down and back, the forearm
    # forward and up.
    ("upperarm.L", "forearm.L", (0.45, -0.85, 0.0)),
    ("upperarm.R", "forearm.R", (-0.45, -0.85, 0.0)),
    ("forearm.L", "hand.L", (0.35, 0.0, 1.0)),
    ("forearm.R", "hand.R", (-0.35, 0.0, 1.0)),
]


def standing(model):
    """The beast up on its hind legs."""
    pose = [tuple(np.copy(v) for v in r) for r in model.rest]
    proportioned(model, pose)
    turn(model, pose, "pelvis", (1, 0, 0), -REAR)
    for bone, child, direction in STAND_AIMS:
        aim(model, pose, bone, child, direction)
    i = model.index["jaw"]
    t, _, s = pose[i]
    pose[i] = (t, np.array([0.0, 0.0, 0.0, 1.0]), s)
    return pose


def stood():
    """The standing mesh, its feet on the ground."""
    m = sculpt(Model())
    v = m.skin(standing(m))
    v[:, 1] -= v[:, 1].min()
    return v, m.triangles


def lie(verts):
    """Roll the body onto its right side and set it on the ground."""
    roll = quat_matrix(axis_angle((0, 0, 1), ROLL))
    v = verts @ roll.T
    v[:, 1] -= np.percentile(v[:, 1], 2)
    return v


def mesh():
    """The posed, lying mesh: vertices (metres of the model) and triangles."""
    m = sculpt(Model())
    return lie(m.skin(posed(m))), m.triangles


# What each part is drawn in on the sheet, by the bone that carries most of
# a triangle: so a pose can be read without knowing the model.
PARTS = [
    ("head", (0.80, 0.35, 0.25), ("head", "jaw", "tongue", "fire_socket")),
    ("neck", (0.85, 0.65, 0.30), ("neck_",)),
    ("tail", (0.30, 0.50, 0.80), ("tail_",)),
    ("forelimbs", (0.35, 0.65, 0.35), ("clavicle", "upperarm", "forearm", "hand", "fingers")),
    ("hind limbs", (0.55, 0.40, 0.70), ("thigh", "shin", "foot", "toes")),
]
BODY = (0.62, 0.58, 0.52)


def colours(model):
    """One colour a triangle, by the part its heaviest bone belongs to."""
    heaviest = model.joints[np.arange(len(model.joints)), model.weights.argmax(1)]
    names = [model.names[model.skin_joints[j]] for j in heaviest]
    per_vertex = []
    for n in names:
        c = BODY
        for _, colour, prefixes in PARTS:
            if any(n.startswith(p) for p in prefixes):
                c = colour
        per_vertex.append(c)
    per_vertex = np.array(per_vertex)
    return per_vertex[model.triangles].mean(1)


def main():
    from render import sheet, view

    out = sys.argv[1] if len(sys.argv) > 1 else "target/sleeper-pose.png"
    c = colours(sculpt(Model()))
    up, ut = stood()
    v, tris = mesh()
    size = (440, 400)
    panels = [
        ("Standing: from its left", view(up, ut, (-1, -0.1, 0), colours=c, size=size)),
        ("Standing: from in front, to its left", view(up, ut, (-0.6, -0.15, -0.8), colours=c, size=size)),
        ("Standing: from behind, to its right", view(up, ut, (0.6, -0.25, 0.75), colours=c, size=size)),
        ("Asleep: from above (it faces up the page)", view(v, tris, (0.0001, -1, 0), colours=c, up=(0, 0, 1), size=size)),
        ("Asleep: from its left side", view(v, tris, (-1, -0.2, 0), colours=c, size=size)),
        ("Asleep: from behind and to its right, above", view(v, tris, (0.6, -0.6, 0.6), colours=c, size=size)),
    ]
    title = "The Sleeper. Red head, orange neck, blue tail, green forelimbs, purple hind limbs, grey body and back spikes."
    sheet(panels, columns=3, title=title).save(out)
    print(out)


if __name__ == "__main__":
    main()
