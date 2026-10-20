"""What the Reaver wears.

An assassin who fights by being somewhere else: nothing on her is heavy or
loud. A deep hood and a cowl over the shoulders, the lower face masked; a
fitted leather jerkin over a dark knit; one layered pauldron, on the left,
the side she leads a throw with; vambraces, fingerless gloves; long split coat
tails that trail her when she moves; soft boots wrapped to the knee; a sash in
her team's colour. Read from across the arena she is a dark, narrow shape with
a pale band of face and two points of steel.

Every piece is a function of the body and the skeleton's bind pose, in the
game's frame (``Rig``), so a change to her build changes her clothes too.
"""

import bpy
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

import forge
import garment as g
import stage

J = {n: i for i, n in enumerate(g.JOINTS)}

# Her own left, by skeleton slot: the arena is right-handed and a pose is
# reflected when drawn, so the body's left side hangs from the ``.r`` slots
# (``view::hand_joint``).
LEFT = "r"

# What the export needs besides the skeleton: the coat's joints, each with
# its parent and where it is in the bind pose; and points carried by a
# joint that the game reads but nothing draws (the blades' tips).
EXTRAS = {"tails": [], "markers": []}

# Colours are placeholders for the painted textures: what each material is.
PALETTE = {
    "knit": (0.07, 0.065, 0.085),
    "leather": (0.16, 0.10, 0.12),
    "leather_dark": (0.07, 0.055, 0.06),
    "hood": (0.10, 0.10, 0.17),
    "mask": (0.05, 0.05, 0.07),
    "sash": (0.45, 0.06, 0.08),
    "metal": (0.55, 0.55, 0.6),
    "metal_dark": (0.2, 0.19, 0.2),
    "wrap": (0.30, 0.27, 0.25),
    "eye": (0.80, 0.78, 0.76),
    "trousers": (0.06, 0.055, 0.07),
}


class Rig:
    def __init__(self, sk, origin, rot, marks):
        self.sk = sk
        self.o = {n: np.asarray(origin[i]) for n, i in J.items()}
        self.tip = {n: sk.tip(origin, rot, i) for n, i in J.items()}
        self.marks = marks

    def along(self, a, b, t):
        """A point ``t`` of the way from joint ``a`` to joint ``b``."""
        return self.o[a] + (self.o[b] - self.o[a]) * t

    def dir(self, a, b):
        d = self.o[b] - self.o[a]
        return d / np.linalg.norm(d)


_mats = {}


def mat(name):
    if name not in _mats:
        rough = 0.35 if name == "metal" else 0.75
        _mats[name] = stage.flat_material(name, PALETTE[name], rough)
    return _mats[name]


def w(fw, *names):
    return sum(fw[:, J[n]] for n in names)


def piece(body, name, gap, keep, trims=(), keeps=(), thick=0.004, smooth=2, faces=1200,
          material="leather", voxel=0.004, close=0.0, trim=0.03, folds=None):
    """A garment: the body inflated by ``gap``, the faces ``keep`` names, cut
    clean by planes (``trims`` thrown away on their normal's side, ``keeps``
    kept), and made twice, as a game character is:

    - the **high** piece, at the shell's full density, with its folds pushed
      into it (``folds(ob)``) and a little unevenness everywhere;
    - the **game** piece, thinned to a budget of ``faces`` triangles.

    The high piece is never exported. Its folds are baked onto the game
    piece's texture as a normal map and as the curvature the painter reads
    (``paint.passes``), so they catch the game's light and the painted
    light alike for no triangles. Both are given a thickness."""
    sh = g.shell(body, gap, voxel=voxel, smooth=smooth, close=close)
    hi = g.cut(sh, keep, name + "~hi")
    for point, normal in trims:
        g.trim(hi, point, normal)
    for point, normal in keeps:
        g.keep_side(hi, point, normal)
    g.smooth_border(hi)
    g.smooth_shape(hi, 0.4, 2)
    if trim:
        g.hem(hi, trim)
    ob = hi.copy()
    ob.data = hi.data.copy()
    ob.name = ob.data.name = name
    g.link(ob)
    # ``faces`` counts triangles; the shell is quads.
    g.decimate(ob, min(1.0, faces / max(1, 2 * len(ob.data.polygons))))
    g.fabric(hi)
    if folds:
        folds(hi)
    for o in (hi, ob):
        if thick:
            g.thicken(o, thick)
        o.data.shade_smooth()
        g.material(o, mat(material))
    hi["hi"] = True
    hi.hide_render = True
    return ob


def boots(body, r):
    out = []
    for s in "lr":
        knee, ankle = f"shin.{s}", f"foot.{s}"
        up = r.dir(ankle, knee)
        def folds(o, ankle=ankle, knee=knee):
            g.wrinkle(o, r.o[ankle] + np.array([0, 0.03, 0.03]), r.dir(knee, ankle), 0.04, 0.0035,
                      0.025, skew=0.8, seed=23)

        out.append(piece(body, f"boot.{s}", 0.008, lambda c, fw: w(fw, knee, ankle) > 0.3,
                         trims=[(r.along(knee, ankle, 0.12), up)], material="leather_dark",
                         smooth=7, faces=1000, folds=folds))
        out.append(piece(body, f"cuff.{s}", 0.016, lambda c, fw: w(fw, knee) > 0.3,
                         trims=[(r.along(knee, ankle, 0.07), up)],
                         keeps=[(r.along(knee, ankle, 0.20), up)], material="leather", faces=400))
        for i, at in enumerate((0.42, 0.66)):
            out.append(straps(body, r, knee, ankle, at, 0.022, 0.0135, f"strap{i}.{s}", "leather",
                              tilt=0.2 if i else -0.2))
    return out


def straps(body, r, a, b, at, width, gap, name, material, tilt=0.15):
    """A strap round a limb: a band of the body inflated past what it is
    worn over, between two planes across the bone, a little askew."""
    up = r.dir(b, a)
    side = np.cross(up, [0.0, 0.0, 1.0])
    n = up + side * tilt
    n = n / np.linalg.norm(n)
    mid = r.along(a, b, at)
    return piece(body, name, gap, lambda c, fw: w(fw, a, b) > 0.3,
                 trims=[(mid + up * width / 2, n)], keeps=[(mid - up * width / 2, n)],
                 material=material, thick=0.003, faces=260, trim=0.006)


def vambraces(body, r):
    out = []
    for s in "lr":
        elbow, wrist = f"forearm.{s}", f"hand.{s}"
        out.append(piece(body, f"vambrace.{s}", 0.008, lambda c, fw: w(fw, elbow, wrist) > 0.3,
                         trims=[(r.along(elbow, wrist, 0.30), r.dir(wrist, elbow)),
                                (r.along(elbow, wrist, 1.07), r.dir(elbow, wrist))], faces=700))
        for i, at in enumerate((0.52, 0.82)):
            out.append(straps(body, r, elbow, wrist, at, 0.016, 0.0125, f"bracer_strap{i}.{s}",
                              "leather_dark", tilt=0.1))
    return out


def armholes(r, inset):
    out = []
    for s, sign in (("l", -1), ("r", 1)):
        sh = r.o[f"arm.{s}"]
        out.append(([sh[0] - sign * inset, sh[1] - 0.05, 0.0], [sign * 0.8, 0.6, 0.0]))
    return out


def jerkin(body, r):
    """A fitted leather jerkin, collar to the top of the thigh: bridged
    over the bust and the small of the back as stiff leather is, its skirt
    falling past the belt, creased at the waist where it bends."""
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1]
    waist = r.o["spine"]

    def folds(o):
        g.wrinkle(o, waist + np.array([0, 0.03, 0]), [0, 1, 0], 0.06, 0.004, 0.035, seed=1)
        g.drape_folds(o, hip - 0.02, hip - 0.13, 0.004, 9, seed=2)

    return [piece(body, "jerkin", 0.012, smooth=8, close=0.04,
                  keep=lambda c, fw: w(fw, "root", "spine", "chest", "arm.l", "arm.r", "thigh.l", "thigh.r") > 0.4,
                  trims=[([0, hip - 0.12, 0], [0, -1, 0]), ([0, neck - 0.03, 0], [0, 1, 0])]
                  + armholes(r, 0.03), faces=2000, folds=folds, trim=0.025)]


def trousers(body, r):
    """Close-cut trousers of soft dark cloth, tucked into the boots: the
    crotch bridged, folds pushed together at the knee, bunched where the boot
    top grips them, and hanging from the seat."""
    hip = r.o["thigh.l"][1]

    def folds(o):
        for i, s in enumerate("lr"):
            knee = r.o[f"shin.{s}"]
            g.wrinkle(o, knee, r.dir(f"thigh.{s}", f"shin.{s}"), 0.08, 0.005, 0.04, seed=3 + i)
            top = r.along(f"shin.{s}", f"foot.{s}", 0.17)
            g.wrinkle(o, top, r.dir(f"shin.{s}", f"foot.{s}"), 0.045, 0.006, 0.028, seed=5 + i,
                      bias=0.2)
            g.wrinkle(o, r.o[f"thigh.{s}"] + np.array([0, -0.08, 0.04]),
                      r.dir(f"thigh.{s}", f"shin.{s}"), 0.07, 0.0035, 0.05, skew=1.4, seed=7 + i)

    keep = lambda c, fw: w(fw, "root", "thigh.l", "thigh.r", "shin.l", "shin.r") > 0.4  # noqa: E731
    trims = [([0, hip + 0.07, 0], [0, 1, 0])]
    for s in "lr":
        trims.append((r.along(f"shin.{s}", f"foot.{s}", 0.30), r.dir(f"shin.{s}", f"foot.{s}")))
    return [piece(body, "trousers", 0.009, keep, trims=trims, smooth=4, close=0.025,
                  material="trousers", faces=1800, folds=folds, trim=0.0)]


def sleeves(body, r):
    """Sleeves of the same knit as the trousers, from under the jerkin to
    under the vambrace, gathered at the elbow."""
    out = []
    for i, s in enumerate("lr"):
        sh, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"

        def folds(o, s=s, el=el, sh=sh, i=i):
            g.wrinkle(o, r.o[el], r.dir(sh, el), 0.07, 0.0045, 0.03, seed=11 + i)
            g.wrinkle(o, r.along(sh, el, 0.15), r.dir(sh, el), 0.05, 0.003, 0.04, skew=1.2,
                      seed=13 + i)

        out.append(piece(body, f"sleeve.{s}", 0.006, lambda c, fw, sh=sh, el=el: w(fw, sh, el) > 0.45,
                         trims=[(r.along(el, wr, 0.55), r.dir(el, wr))],
                         keeps=[(r.o[sh] - r.dir(sh, el) * 0.01, r.dir(sh, el))],
                         smooth=3, close=0.01, material="trousers", faces=700, folds=folds, trim=0.0))
    return out


def belt(body, r):
    hip = r.o["thigh.l"][1]
    trunk = lambda c, fw: w(fw, "root", "spine", "thigh.l", "thigh.r") > 0.4  # noqa: E731
    b = piece(body, "belt", 0.015, trunk, close=0.03, trim=0.008,
              trims=[([0, hip + 0.035, 0], [0, 1, 0.12])],
              keeps=[([0, hip - 0.01, 0], [0, 1, 0.12])], material="leather_dark", thick=0.006, faces=700)
    sash = piece(body, "sash", 0.012, trunk, close=0.03, trim=0.01,
                 trims=[([0, hip + 0.10, 0], [0.25, 1, 0])],
                 keeps=[([0, hip + 0.03, 0], [-0.15, 1, 0])], material="sash", thick=0.003, faces=700)
    return [b, sash]


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


def coat_tails(body, r):
    """Four long panels from the belt to the calf, open at the front, split
    at the back and sides, dropped under cloth simulation onto her legs."""
    hip = r.o["thigh.l"][1]
    top = hip + 0.005
    bottom = 0.30
    collider = g.shell(body, 0.012, voxel=0.006, smooth=2)
    g.collide(collider, 0.006)
    # How far the belt line is from the middle, all the way round.
    sh = g.shell(body, 0.022, voxel=0.006, smooth=3)
    dg = bpy.context.evaluated_depsgraph_get()
    tree = BVHTree.FromObject(sh, dg)

    def radius(theta):
        d = np.array([np.sin(theta), 0.0, np.cos(theta)])
        o = np.array([0.0, top, 0.0])
        hit = tree.ray_cast(Vector(g.g2b(o + d * 0.6).tolist()), Vector(g.g2b(-d).tolist()))
        if hit[0] is None:
            return 0.17
        return float(np.linalg.norm(g.b2g(np.array(hit[0])) - o))

    panels = []
    deg = np.radians
    # Sides to the knee, the back pair to the calf.
    spans = [(deg(52), deg(104), 0.50), (deg(108), deg(177), bottom),
             (deg(183), deg(252), bottom), (deg(256), deg(308), 0.50)]
    for k, (a0, a1, end) in enumerate(spans):
        mid = (a0 + a1) / 2
        r_mid = radius(mid)
        out = np.array([np.sin(mid), 0.0, np.cos(mid)])
        pivot = np.array([0.0, top, 0.0]) + out * r_mid
        knee = top - (top - end) * 0.45
        EXTRAS["tails"].append((f"tail.{k}.a", "root", pivot.tolist()))
        EXTRAS["tails"].append((f"tail.{k}.b", f"tail.{k}.a",
                                (pivot + out * (top - knee) * 0.42 + np.array([0, knee - top, 0])).tolist()))
        rows = int((top - end) / 0.02)
        cols = max(6, int((a1 - a0) / deg(5.5)))
        thetas = np.linspace(a0, a1, cols + 1)
        r0 = np.array([radius(t) for t in thetas])
        verts = []
        for i in range(rows + 1):
            t = i / rows
            y = top + (end - top) * t
            flare = (top - y) * 0.42
            for th, rt in zip(thetas, r0):
                # A ripple to seed the folds: cloth falling from a perfect
                # cone stays a cone.
                rr = rt + flare * (1 + 0.18 * np.sin(th * 23.0 + k))
                verts.append([np.sin(th) * rr, y, np.cos(th) * rr])
        verts = np.array(verts)
        faces = []
        n = cols + 1
        for i in range(rows):
            for j in range(cols):
                a = i * n + j
                faces.append([a, a + n, a + n + 1, a + 1])
        ob = stage.mesh_object(f"tail{k}", g.g2b(verts), faces)
        pin = ob.vertex_groups.new(name="pin")
        for i in range(2):
            for j in range(n):
                pin.add([i * n + j], 1.0 if i == 0 else 0.5, "REPLACE")
        panels.append(ob)
    for k, ob in enumerate(panels):
        end = spans[k][2]
        g.drape(ob, "pin", frames=90, mass=0.3, stiff=20.0, bend=0.4)
        ob.vertex_groups.remove(ob.vertex_groups["pin"])
        g.thicken(ob, 0.004)
        ob.data.shade_smooth()
        g.material(ob, mat("hood"))
        # Weights by hand: the belt carries the top, the panel's own two
        # joints the rest -- swung by springs in the game, pushed clear of
        # the legs (`crates/game/src/reaver.rs`, `coats`).
        co = np.zeros(len(ob.data.vertices) * 3)
        ob.data.vertices.foreach_get("co", co)
        p = g.b2g(co.reshape(-1, 3))
        down = np.clip((top - p[:, 1]) / (top - end), 0, 1)

        def ramp(x, a, b):
            t = np.clip((x - a) / (b - a), 0, 1)
            return t * t * (3 - 2 * t)

        low = ramp(down, 0.35, 0.85)
        belt = 1 - ramp(down, 0.0, 0.3)
        W = np.zeros((len(p), len(g.ALL)))
        W[:, J["root"]] = belt
        W[:, g.ALL.index(f"tail.{k}.b")] = low
        W[:, g.ALL.index(f"tail.{k}.a")] = np.clip(1 - belt - low, 0, 1)
        g.set_weights(ob, W)
        ob["weighted"] = True
    collider.modifiers.clear()
    return panels


def eyes(body, r):
    """Two eyes, the one part of her face the hood and mask leave: grey-violet
    irises, set a little into the sockets so the lids overlap them."""
    out = []
    for s in "lr":
        c = r.marks[f"eye.{s}"] + np.array([0.0, 0.0, -0.004])
        rad = 0.0122
        bpy.ops.mesh.primitive_uv_sphere_add(segments=24, ring_count=16, radius=rad,
                                             location=tuple(g.g2b(c)))
        ob = bpy.context.active_object
        ob.name = f"eye.{s}"
        # Into the mesh: the export reads vertices, not objects' places.
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        bpy.ops.object.shade_smooth()
        co = np.zeros(len(ob.data.vertices) * 3)
        ob.data.vertices.foreach_get("co", co)
        p = g.b2g(co.reshape(-1, 3))
        d = (p - c) / np.linalg.norm(p - c, axis=1, keepdims=True)
        front = d[:, 2]
        iris = np.clip((front - 0.80) / 0.04, 0, 1)
        pupil = np.clip((front - 0.955) / 0.01, 0, 1)
        g.vertex_mask(ob, iris, pupil)
        g.material(ob, mat("eye"))
        W = np.zeros((len(p), len(g.ALL)))
        W[:, J["head"]] = 1.0
        g.set_weights(ob, W)
        ob["rigid"] = True
        out.append(ob)
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
