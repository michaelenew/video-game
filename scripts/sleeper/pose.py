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
    # The left forelimb, uppermost: out of the shoulder, forward and down
    # to the elbow, and the forearm folded back along over the head.
    ("upperarm.L", "forearm.L", (0.6, -0.45, 0.65)),
    ("forearm.L", "hand.L", (0.2, 0.05, -1.0)),
    ("hand.L", "fingers.L", (-0.2, -0.6, -0.75)),
    # The right forelimb, under: the elbow down and forward, the forearm
    # folded in under the chin.
    ("upperarm.R", "forearm.R", (-0.35, -0.6, 0.7)),
    ("forearm.R", "hand.R", (0.85, -0.2, 0.45)),
    ("hand.R", "fingers.R", (0.6, -0.4, -0.6)),
    # The left hind leg, uppermost, drawn up along the flank: a ridge.
    ("thigh.L", "shin.L", (0.55, -0.25, 0.8)),
    ("shin.L", "foot.L", (0.2, -0.2, -1.0)),
    ("foot.L", "toes.L", (0.25, -0.35, 0.9)),
    # The right hind leg, under the body.
    ("thigh.R", "shin.R", (-0.3, -0.6, 0.7)),
    ("shin.R", "foot.R", (-0.2, -0.4, -0.9)),
    ("foot.R", "toes.R", (-0.2, -0.3, 0.9)),
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
    # Forelimbs long enough to reach the ground stretched (`sculpt.py`
    # slims them against it): `ARM_REACH` checks it.
    ("upperarm.L", 1.25),
    ("upperarm.R", 1.25),
    # A T. rex's legs: longer than the dragon's.
    ("thigh.L", 1.5),
    ("thigh.R", 1.5),
    # Real feet to stand on: bigger, toes and all.
    ("foot.L", 1.5),
    ("foot.R", 1.5),
    # A heavier head, and a longer tail.
    ("head", 1.3),
    ("tail_01", 1.25),
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


def point(model, pose, bone, direction):
    """Turn `bone` so its own length lies along a world `direction`: for a
    bone with no child to aim at (the toes)."""
    world = model.world(pose)[model.index[bone]][:3, :3]
    now = world[:, 1] / np.linalg.norm(world[:, 1])
    want = np.asarray(direction, float)
    want /= np.linalg.norm(want)
    axis = np.cross(now, want)
    if np.linalg.norm(axis) > 1e-9:
        turn(model, pose, bone, axis, np.degrees(np.arccos(np.clip(now @ want, -1, 1))))


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
# and the head forward; the arms hanging. The shins are swung back until
# the feet are under its centre of mass (`balanced`).
REAR = 22.0
STAND_AIMS = [
    # The thighs straight forward and a little down (the owner: "they
    # should extend forward and a little down when it's standing"); the
    # shin back down to the ankle (how far is `balanced`'s to find); up on
    # its toes.
    ("thigh.L", "shin.L", (0.1, -0.3, 1.0)),
    ("thigh.R", "shin.R", (-0.1, -0.3, 1.0)),
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
    # The arms hanging, relaxed: down and a little forward, the elbows
    # a little bent.
    ("upperarm.L", "forearm.L", (0.25, -1.0, 0.1)),
    ("upperarm.R", "forearm.R", (-0.25, -1.0, 0.1)),
    ("forearm.L", "hand.L", (0.1, -1.0, 0.35)),
    ("forearm.R", "hand.R", (-0.1, -1.0, 0.35)),
    ("hand.L", "fingers.L", (0.05, -0.8, 0.6)),
    ("hand.R", "fingers.R", (-0.05, -0.8, 0.6)),
]


def centre_of_mass(verts, tris):
    """Where the body's weight is, taking its surface as its skin: the
    middle of its triangles, each by its area."""
    p = verts[tris]
    area = np.linalg.norm(np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0]), axis=1)
    return (p.mean(1) * area[:, None]).sum(0) / area.sum()


def flat_feet(model, pose):
    """The toes laid flat on the ground, spread a little: a foot planted."""
    point(model, pose, "toes.L", (0.15, -0.05, 1.0))
    point(model, pose, "toes.R", (-0.15, -0.05, 1.0))


def balanced(model, pose):
    """Swing the shins back until the feet stand under the centre of mass,
    front to back: the angle behind straight down, degrees, that puts them
    nearest, tried every two degrees."""
    best = None
    for back in range(0, 80, 2):
        trial = [tuple(np.copy(v) for v in r) for r in pose]
        a = np.radians(back)
        for side in ("L", "R"):
            aim(model, trial, f"shin.{side}", f"foot.{side}", (0.0, -np.cos(a), -np.sin(a)))
        for bone, child, direction in STAND_AIMS:
            if bone.startswith("foot"):
                aim(model, trial, bone, child, direction)
        flat_feet(model, trial)
        at = model.bone_points(trial)
        feet = (at["toes.L"][2] + at["toes.R"][2]) / 2
        off = abs(feet - centre_of_mass(model.skin(trial), model.triangles)[2])
        if best is None or off < best[0]:
            best = (off, trial)
    pose[:] = best[1]


def standing(model):
    """The beast up on its hind legs."""
    pose = [tuple(np.copy(v) for v in r) for r in model.rest]
    proportioned(model, pose)
    turn(model, pose, "pelvis", (1, 0, 0), -REAR)
    for bone, child, direction in STAND_AIMS:
        aim(model, pose, bone, child, direction)
    balanced(model, pose)
    i = model.index["jaw"]
    t, _, s = pose[i]
    pose[i] = (t, np.array([0.0, 0.0, 0.0, 1.0]), s)
    return pose


def arm_reach(model, pose):
    """How far the arm reaches stretched against how high the shoulder is
    over the feet: at least one, it can touch the ground."""
    at = model.bone_points(pose)
    chain = [at[b] for b in ("upperarm.L", "forearm.L", "hand.L", "fingers.L")]
    length = sum(np.linalg.norm(b - a) for a, b in zip(chain, chain[1:]))
    floor = min(at["toes.L"][1], at["toes.R"][1])
    return length / (at["upperarm.L"][1] - floor)


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
    m = sculpt(Model())
    print(f"arm reach / shoulder height: {arm_reach(m, standing(m)):.2f}")
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
