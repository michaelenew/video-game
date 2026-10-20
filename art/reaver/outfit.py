"""What the Reaver wears.

An assassin who fights by being somewhere else: nothing on her is heavy or
loud. A deep hood and a cowl over the shoulders, the lower face masked; a
fitted leather jerkin over a dark knit; one layered pauldron, on the left,
the side she leads a throw with; vambraces, fingerless gloves; long split coat
tails that trail her when she moves; soft boots wrapped to the knee; a sash in
her team's colour. Read from across the arena she is a dark, narrow shape with
a pale band of face and two points of steel.

The tailor's tools and the garments every class can wear are
``art/lib/wardrobe.py``; what is here is hers alone.
"""

import bpy
import numpy as np

import forge
import garment as g
import stage
from wardrobe import (EXTRAS, J, LEFT, belt, boots, coat_tails, eyes, jerkin, mat, piece,  # noqa: F401
                      sleeves, trousers, vambraces, w)

def hood(body, r):
    """The hood and the cowl under it, one piece: everything from the upper
    chest up, with the face left open, standing well off the skull so it
    reads as cloth with air in it rather than a swimming cap."""
    eye = (r.marks["eye.l"] + r.marks["eye.r"]) / 2
    shoulder = r.o["arm.l"][1]

    def keep(c, fw):
        trunk = w(fw, "chest", "head", "spine", "arm.l", "arm.r") > 0.5
        dx = c[:, 0] / 0.072
        dy = (c[:, 1] - (eye[1] - 0.03)) / 0.088
        face = (dx * dx + dy * dy < 1.0) & (c[:, 2] > 0.0)
        return trunk & ~face

    # The cowl's hem comes to a point at the breastbone and between the
    # shoulder blades.
    trims = [([0, shoulder - 0.20, 0.0], [0.62, -1, 0.0]), ([0, shoulder - 0.20, 0.0], [-0.62, -1, 0.0])]
    for s, sign in (("l", -1), ("r", 1)):
        trims.append(([sign * (r.o[f"arm.{s}"][0] * sign + 0.06), 0, 0], [sign, -0.15, 0]))
    neck = r.o["head"]

    def folds(o):
        g.wrinkle(o, neck + np.array([0, 0.05, 0]), [0, 1, 0], 0.05, 0.006, 0.03, skew=1.5, seed=17)
        g.drape_folds(o, shoulder + 0.02, shoulder - 0.2, 0.005, 7, seed=19)

    ob = piece(body, "hood", 0.03, keep, trims=trims, smooth=6, material="hood", thick=0.005,
               voxel=0.005, faces=2200, close=0.05, trim=0.025, folds=folds)
    _peak(ob, r.o["head"], eye)
    _brim(ob, eye)
    return [ob]


def _peak(ob, head, eye):
    """Pull the back of the hood up and back into a soft point."""
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    crown = eye[1] + 0.04
    back = -p[:, 2]
    t = np.clip((p[:, 1] - crown) / 0.09, 0, 1) * np.clip(back / 0.09 + 0.2, 0, 1)
    p[:, 1] += 0.03 * t * t
    p[:, 2] -= 0.07 * t * t * t
    ob.data.vertices.foreach_set("co", g.g2b(p).ravel())
    ob.data.update()


def _brim(ob, eye):
    """Bring the hood's opening forward past the face, so the brow is in its
    shadow: what makes a hood read as deep."""
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    near = np.clip(1 - np.abs(p[:, 0]) / 0.13, 0, 1) * np.clip((p[:, 2] - 0.0) / 0.1, 0, 1)
    high = np.clip((p[:, 1] - (eye[1] - 0.12)) / 0.1, 0, 1)
    t = near * high
    p[:, 2] += 0.035 * t
    p[:, 1] += 0.01 * t
    ob.data.vertices.foreach_set("co", g.g2b(p).ravel())
    ob.data.update()


def mask(body, r):
    eye = (r.marks["eye.l"] + r.marks["eye.r"]) / 2
    shoulder = r.o["arm.l"][1]
    return [piece(body, "mask", 0.004, lambda c, fw: w(fw, "head", "chest") > 0.5,
                  # Over the bridge of the nose, just under the eyes.
                  trims=[([0, eye[1] - 0.016, 0.0], [0, 1, 0.12])],
                  keeps=[([0, shoulder + 0.02, 0.0], [0, 1, 0])], material="mask", thick=0.003,
                  faces=1000, voxel=0.003, close=0.02, trim=0.012)]


def daggers(body, r):
    """Two fighting knives, held forward, edge out."""
    out = []
    for s in "lr":
        grip = r.marks[f"grip.{s}"]
        u = r.marks[f"grip_axis.{s}"]
        edge = grip - r.marks[f"wrist.{s}"]
        origin = grip + u * 0.052
        parts = []
        bv, bf, bedge = forge.blade()
        parts.append(("blade", bv, bf, "metal"))
        # The tip and the root of the blade, for the trail its cut leaves.
        span = forge.frame(np.array([[0.30, 0.012, 0.0], [0.06, 0.0, 0.0]]), origin, u, edge)
        EXTRAS["markers"].append((f"blade.{s}.tip", f"hand.{s}", span[0].tolist()))
        EXTRAS["markers"].append((f"blade.{s}.base", f"hand.{s}", span[1].tolist()))
        gv, gf = forge.guard()
        parts.append(("guard", gv, gf, "metal_dark"))
        hv, hf = forge.handle()
        parts.append(("grip", hv, hf, "wrap"))
        pv, pf = forge.pommel()
        parts.append(("pommel", pv, pf, "metal_dark"))
        for name, v, f, m in parts:
            ob = stage.mesh_object(f"dagger_{name}.{s}", g.g2b(forge.frame(v, origin, u, edge)), f,
                                   smooth=name != "blade")
            g.material(ob, mat(m))
            # Held: the hand moves it and nothing else.
            W = np.zeros((len(v), len(g.ALL)))
            W[:, J[f"hand.{s}"]] = 1.0
            g.set_weights(ob, W)
            if name == "blade":
                # The honed edge, which carries her colour of light.
                honed = ob.vertex_groups.new(name="edge")
                for i, e in enumerate(bedge):
                    if e > 0:
                        honed.add([i], float(e), "REPLACE")
            ob["rigid"] = True
            out.append(ob)
    return out


def pauldron(body, r):
    """Three overlapping plates of boiled leather on the left shoulder, the
    side she leads with: the one asymmetric thing in her silhouette."""
    sh = r.o[f"arm.{LEFT}"]
    down = r.dir(f"arm.{LEFT}", f"forearm.{LEFT}")
    out = []
    for i, (gap, reach, at) in enumerate(((0.034, 0.105, 0.0), (0.028, 0.085, 0.07), (0.022, 0.07, 0.125))):
        centre = sh + down * at + np.array([0.01 * np.sign(sh[0]), 0.035 - at * 0.15, 0.0])

        def keep(c, fw, centre=centre, reach=reach):
            d = np.linalg.norm(c - centre, axis=1)
            side = c[:, 0] * np.sign(sh[0]) > np.abs(sh[0]) - 0.06
            return (d < reach) & side

        plate = piece(body, f"pauldron{i}", gap, keep, trims=[(sh + down * (at + 0.075), down)],
                      smooth=3, material="leather" if i else "leather_dark", thick=0.006,
                      faces=(600, 450, 350)[i])
        out.append(plate)
    return out



WARDROBE = ["boots", "vambraces", "sleeves", "trousers", "jerkin", "belt", "mask", "hood", "pauldron", "coat_tails", "daggers", "eyes"]


def _hide_covered(body, r, only):
    """Take away the skin nothing will ever see: under the boots, under the
    jerkin, the back of the head inside the hood. Fewer triangles, and none
    of it can poke through what covers it when a joint bends it further than
    the cloth over it."""
    if only:
        return
    eye = (r.marks["eye.l"] + r.marks["eye.r"]) / 2
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1]
    W = g.weights_of(body)

    def gone(c, fw):
        out = np.zeros(len(c), dtype=bool)
        for s in "lr":
            knee, ankle = f"shin.{s}", f"foot.{s}"
            boot_top = r.along(knee, ankle, 0.22)
            up = r.dir(ankle, knee)
            inside = w(fw, knee, ankle) > 0.6
            out |= inside & (((c - boot_top) @ up) < 0)
        # Inside the trousers and the sleeves, away from where they end.
        for s in "lr":
            out |= w(fw, f"thigh.{s}") > 0.85
            out |= (w(fw, f"shin.{s}") > 0.85) & (((c - r.along(f"shin.{s}", f"foot.{s}", 0.2)) @ r.dir(f"foot.{s}", f"shin.{s}")) > 0)
            out |= w(fw, f"arm.{s}") > 0.9
            out |= (w(fw, f"forearm.{s}") > 0.9) & (((c - r.along(f"forearm.{s}", f"hand.{s}", 0.75)) @ r.dir(f"hand.{s}", f"forearm.{s}")) > 0)
        trunk = w(fw, "root", "spine", "chest") > 0.85
        out |= trunk & (c[:, 1] > hip + 0.02) & (c[:, 1] < neck - 0.08) & (np.abs(c[:, 0]) < 0.11)
        # The head, but for the band of face between the brow and the mask.
        head = w(fw, "head") > 0.9
        # Down well inside the mask, whose top edge dips toward the nose.
        face = (c[:, 2] > 0.0) & (c[:, 1] > eye[1] - 0.10) & (c[:, 1] < eye[1] + 0.06) & (np.abs(c[:, 0]) < 0.08)
        out |= head & ~face & (c[:, 1] > eye[1] - 0.2)
        return out

    g.drop_faces(body, gone)


def dress(body, r, only=None):
    import time
    pieces = []
    for name in WARDROBE:
        if only and name not in only:
            continue
        t = time.time()
        pieces += globals()[name](body, r)
        print(f"  {name}: {time.time() - t:.1f}s", flush=True)
    for p in pieces:
        if not (p.get("rigid") or p.get("weighted")):
            g.bind_to(p, body)
    body.data.materials.append(mat("knit"))
    _body_masks(body, r)
    _hide_covered(body, r, only)
    return pieces


def _body_masks(body, r):
    """What of the body is skin and what is glove, painted onto it: the face
    (what the hood and mask leave of it), and the fingers past the glove."""
    W = g.weights_of(body)
    co = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    head = W[:, J["head"]]
    skin = np.clip((head - 0.5) * 4, 0, 1)
    glove = np.zeros(len(p))
    for s in "lr":
        hand = W[:, J[f"hand.{s}"]]
        wrist = r.marks[f"wrist.{s}"]
        grip = r.marks[f"grip.{s}"]
        # Fingerless: the glove stops at the middle knuckles.
        along = (p - wrist) @ ((grip - wrist) / np.linalg.norm(grip - wrist))
        reach = np.linalg.norm(grip - wrist)
        fingers = np.clip((along - reach * 1.05) / 0.01, 0, 1)
        skin = np.maximum(skin, hand * fingers)
        glove = np.maximum(glove, np.clip(hand * 3, 0, 1) * (1 - fingers))
    # What a painter puts on a face to be read from a distance: brows, and
    # the eyes smudged dark -- she is an assassin, and her eyes are all of
    # her face the mask leaves.
    paint = np.zeros(len(p))
    for s in "lr":
        e = r.marks[f"eye.{s}"]
        d = p - e
        out = np.sign(e[0])
        kohl = np.exp(-((d[:, 0] - out * 0.004) / 0.019) ** 2 - (d[:, 1] / 0.010) ** 2)
        arch = 0.016 + 0.004 * np.cos(np.clip((d[:, 0] * out + 0.002) / 0.022, -1.5, 1.5) * 1.6)
        brow = np.exp(-((d[:, 1] - arch) / 0.0035) ** 2) * (np.abs(d[:, 0] * out + 0.002) < 0.021)
        front = np.clip(d[:, 2] / 0.006 + 1.0, 0, 1)
        paint = np.maximum(paint, np.clip(kohl * 0.75 + brow, 0, 1) * front)
    paint *= skin
    g.vertex_mask(body, skin, glove, paint)
