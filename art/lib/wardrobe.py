"""What every character's clothes are cut with.

A garment is a function of the body and the skeleton's bind pose, in the
game's frame (``Rig``), so a change to a build changes its clothes too. The
pieces here are the tailor's tools; what a class wears, and in what, is its
own ``outfit.py``.
"""

import bpy
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

import garment as g
import stage

J = {n: i for i, n in enumerate(g.JOINTS)}

# A character's own left, by skeleton slot: the arena is right-handed and a
# pose is reflected when drawn, so the body's left side hangs from the ``.r``
# slots (``view::hand_joint``).
LEFT = "r"
RIGHT = "l"

# What the export needs besides the skeleton: the coat's joints, each with
# its parent and where it is in the bind pose; and points carried by a joint
# that the game reads but nothing draws (a blade's tip).
EXTRAS = {"tails": [], "markers": []}

# Placeholder colours for the materials, for looking at a model in Blender:
# what the game shows is the painted texture (``character.COLOURS``).
PALETTE = {}


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

    @property
    def eye(self):
        """Between the eyes."""
        return (self.marks["eye.l"] + self.marks["eye.r"]) / 2


_mats = {}


def mat(name):
    if name not in _mats:
        rough = 0.35 if name.startswith("metal") else 0.75
        _mats[name] = stage.flat_material(name, PALETTE.get(name, (0.5, 0.5, 0.5)), rough)
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
        g.tame(o)
        o.data.shade_smooth()
        g.material(o, mat(material))
    hi["hi"] = True
    hi.hide_render = True
    return ob


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


def armholes(r, inset):
    out = []
    for s, sign in (("l", -1), ("r", 1)):
        sh = r.o[f"arm.{s}"]
        out.append(([sh[0] - sign * inset, sh[1] - 0.05, 0.0], [sign * 0.8, 0.6, 0.0]))
    return out


def rigid(ob, joint):
    """Carried whole by one joint: a held thing, an ornament."""
    W = np.zeros((len(ob.data.vertices), len(g.ALL)))
    W[:, J[joint]] = 1.0
    g.set_weights(ob, W)
    ob["rigid"] = True
    return ob


def eyes(body, r, iris="eye"):
    """Two eyes, set a little into the sockets so the lids overlap them:
    white, an iris (the material's second colour) and a pupil (its third)."""
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
        g.vertex_mask(ob, np.clip((front - 0.80) / 0.04, 0, 1), np.clip((front - 0.955) / 0.01, 0, 1))
        g.material(ob, mat(iris))
        out.append(rigid(ob, "head"))
    return out


def face_paint(p, r, brow=1.0, kohl=0.0, lips=0.0):
    """What a painter puts on a face to be read from a distance, as a mask
    over the body's vertices ``p``: brows, a smudge round the eyes, the
    lips. Returned as one channel (the body's fourth colour)."""
    out = np.zeros(len(p))
    for s in "lr":
        e = r.marks[f"eye.{s}"]
        d = p - e
        side = np.sign(e[0])
        smudge = np.exp(-((d[:, 0] - side * 0.004) / 0.019) ** 2 - (d[:, 1] / 0.010) ** 2)
        arch = 0.016 + 0.004 * np.cos(np.clip((d[:, 0] * side + 0.002) / 0.022, -1.5, 1.5) * 1.6)
        b = np.exp(-((d[:, 1] - arch) / 0.0035) ** 2) * (np.abs(d[:, 0] * side + 0.002) < 0.021)
        front = np.clip(d[:, 2] / 0.006 + 1.0, 0, 1)
        out = np.maximum(out, np.clip(smudge * kohl + b * brow, 0, 1) * front)
    if lips:
        e = r.eye
        mouth = e + np.array([0.0, -0.072, 0.008])
        d = p - mouth
        lip = np.exp(-(d[:, 0] / 0.022) ** 4 - (d[:, 1] / 0.007) ** 2) * (d[:, 2] > -0.012)
        out = np.maximum(out, lip * lips)
    return out


def skin_and_gloves(body, r, gloved_to=None, bare_head=True):
    """The body's two masks: what is skin (the face and anything bare) and
    what is glove. ``gloved_to``: how far down the hand a glove reaches, as a
    share of wrist to grip (1.05 stops at the knuckles; None, no gloves)."""
    W = g.weights_of(body)
    co = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    head = W[:, J["head"]] + W[:, J["neck"]]
    # Down to the collar: the neck is skin wherever the head and neck carry
    # a fifth of it, or a band of grey shows between the jaw and the collar.
    skin = np.clip((head - 0.2) * 3, 0, 1) if bare_head else np.zeros(len(p))
    glove = np.zeros(len(p))
    for s in "lr":
        hand = W[:, J[f"hand.{s}"]]
        if gloved_to is None:
            skin = np.maximum(skin, np.clip(hand * 3, 0, 1))
            continue
        wrist = r.marks[f"wrist.{s}"]
        grip = r.marks[f"grip.{s}"]
        along = (p - wrist) @ ((grip - wrist) / np.linalg.norm(grip - wrist))
        reach = np.linalg.norm(grip - wrist)
        fingers = np.clip((along - reach * gloved_to) / 0.01, 0, 1)
        skin = np.maximum(skin, hand * fingers)
        glove = np.maximum(glove, np.clip(hand * 3, 0, 1) * (1 - fingers))
    return p, W, skin, glove


def hair(body, r, name="hair", gap=0.012, front=0.035, nape=-0.06, side=0.0, part=0.0,
         grooves=9, depth=0.008, material="hair", faces=2600, close=0.02, lift=0.012, fall=0.0):
    """Hair as a painted figure's is: one sculpted mass over the skull, not
    strands. The head inflated by ``gap``, cut back from the face along a
    hairline ``front`` above the brows, down to ``nape`` at the back (metres
    from the eyes' height) and to ``side`` in front of the ears; the high
    piece is combed with grooves running back from the hairline, which the
    bake turns into the locks a painter would put in."""
    e = r.eye
    head_c = r.o["head"]

    def keep(c, fw):
        on_head = w(fw, "head", "neck") > 0.5
        rel = c - e
        above_brow = rel[:, 1] > front - 0.06 * np.clip(-rel[:, 2] / 0.10, 0, 1) ** 0.6
        behind = rel[:, 2] < side
        back = (rel[:, 2] < -0.03) & (rel[:, 1] > nape)
        return on_head & ((above_brow & (rel[:, 2] < 0.10)) | (behind & (rel[:, 1] > -0.02)) | back)

    def comb(o, grooves=grooves, depth=depth):
        co = np.zeros(len(o.data.vertices) * 3)
        o.data.vertices.foreach_get("co", co)
        pp = g.b2g(co.reshape(-1, 3))
        rel = pp - head_c
        nrm = rel / np.maximum(np.linalg.norm(rel, axis=1, keepdims=True), 1e-6)
        # Volume: the crown and the front lifted off the skull, as hair
        # stands rather than lies.
        crown = np.clip((pp[:, 1] - (e[1] + 0.02)) / 0.08, 0, 1)
        pp = pp + nrm * (lift * crown ** 1.5)[:, None]
        # A bob falls: below the brow the hair swings out from the head and
        # down, rather than following the skull in to the nape.
        if fall:
            flat = rel * np.array([1.0, 0.0, 1.0])
            out = flat / np.maximum(np.linalg.norm(flat, axis=1, keepdims=True), 1e-6)
            low = np.clip((e[1] + 0.03 - pp[:, 1]) / 0.10, 0, 1)
            pp = pp + out * (fall * low)[:, None] - np.array([0.0, 1.0, 0.0]) * (fall * 0.8 * low ** 2)[:, None]
        # Locks: grooves running back from a parting, each its own width and
        # depth and wandering a little, so they read as hair a painter laid
        # in rather than as rows.
        ang = np.arctan2(rel[:, 0] - part, rel[:, 1] + 0.02)
        wander = 0.35 * np.sin(rel[:, 2] * 41.0 + rel[:, 0] * 17.0) + 0.2 * np.sin(rel[:, 1] * 63.0)
        phase = ang * grooves + wander
        lock = np.floor(phase / (2 * np.pi))
        size = 0.55 + 0.45 * ((np.sin(lock * 12.9898) * 43758.5453) % 1.0)
        groove = (np.abs(np.sin(phase / 2)) ** 6) * size
        pp = pp - nrm * groove[:, None] * depth
        o.data.vertices.foreach_set("co", g.g2b(pp).ravel())
        o.data.update()

    ob = piece(body, name, gap, keep, smooth=6, close=close, material=material, thick=0.004,
               faces=faces, voxel=0.004, trim=0.0, folds=comb)
    # The game piece gets the volume too (not the locks: those are baked).
    comb(ob, depth=0.0)
    return ob


def hide_covered(body, gone):
    """Take away the skin nothing will ever see: fewer triangles, and none of
    it can poke through what covers it when a joint bends it further than
    the cloth over it."""
    g.drop_faces(body, gone)


# --- Garments every class may wear, at the Reaver's cut by default ----------

def boots(body, r, material="leather_dark", cuff="leather", strap="leather", top=0.12,
          straps_at=(0.42, 0.66), cuffed=True, gap=0.008):
    """Soft boots to under the knee: folded at the ankle, a turned-down cuff,
    straps round the shin."""
    out = []
    for s in "lr":
        knee, ankle = f"shin.{s}", f"foot.{s}"
        up = r.dir(ankle, knee)
        def folds(o, ankle=ankle, knee=knee):
            g.wrinkle(o, r.o[ankle] + np.array([0, 0.03, 0.03]), r.dir(knee, ankle), 0.04, 0.0035,
                      0.025, skew=0.8, seed=23)

        toe = f"toe.{s}"
        # The toe too: a boot taken off the shin and ankle's skin alone left
        # the toes out once they had a joint of their own.
        out.append(piece(body, f"boot.{s}", gap,
                         lambda c, fw, knee=knee, ankle=ankle, toe=toe: w(fw, knee, ankle, toe) > 0.3,
                         trims=[(r.along(knee, ankle, top), up)], material=material,
                         smooth=7, faces=1000, folds=folds))
        if cuffed:
            out.append(piece(body, f"cuff.{s}", gap + 0.008, lambda c, fw, knee=knee: w(fw, knee) > 0.3,
                             trims=[(r.along(knee, ankle, top - 0.05), up)],
                             keeps=[(r.along(knee, ankle, top + 0.08), up)], material=cuff, faces=400))
        for i, at in enumerate(straps_at):
            out.append(straps(body, r, knee, ankle, at, 0.022, gap + 0.0055, f"strap{i}.{s}", strap,
                              tilt=0.2 if i else -0.2))
    return out


def vambraces(body, r, material="leather", strap="leather_dark", reach=0.30, straps_at=(0.52, 0.82),
              sides="lr"):
    """Leather bracers from below the elbow over the back of the hand."""
    out = []
    for s in sides:
        elbow, wrist = f"forearm.{s}", f"hand.{s}"
        out.append(piece(body, f"vambrace.{s}", 0.008,
                         lambda c, fw, elbow=elbow, wrist=wrist: w(fw, elbow, wrist) > 0.3,
                         trims=[(r.along(elbow, wrist, reach), r.dir(wrist, elbow)),
                                (r.along(elbow, wrist, 1.07), r.dir(elbow, wrist))], faces=700,
                         material=material))
        for i, at in enumerate(straps_at):
            out.append(straps(body, r, elbow, wrist, at, 0.016, 0.0125, f"bracer_strap{i}.{s}",
                              strap, tilt=0.1))
    return out


def sleeves(body, r, material="trousers", to=0.55, gap=0.006, close=0.01, sides="lr", name="sleeve",
            faces=700):
    """Sleeves of the same knit as the trousers, from under the jerkin to
    under the vambrace, gathered at the elbow."""
    out = []
    for i, s in enumerate("lr"):
        sh, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"

        def folds(o, s=s, el=el, sh=sh, i=i):
            g.wrinkle(o, r.o[el], r.dir(sh, el), 0.07, 0.0045, 0.03, seed=11 + i)
            g.wrinkle(o, r.along(sh, el, 0.15), r.dir(sh, el), 0.05, 0.003, 0.04, skew=1.2,
                      seed=13 + i)

        if s not in sides:
            continue
        out.append(piece(body, f"{name}.{s}", gap, lambda c, fw, sh=sh, el=el: w(fw, sh, el) > 0.45,
                         trims=[(r.along(el, wr, to), r.dir(el, wr))],
                         keeps=[(r.o[sh] - r.dir(sh, el) * 0.01, r.dir(sh, el))],
                         smooth=3, close=close, material=material, faces=faces, folds=folds, trim=0.0))
    return out


def trousers(body, r, material="trousers", hem=0.30, gap=0.009, close=0.025, name="trousers"):
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
        trims.append((r.along(f"shin.{s}", f"foot.{s}", hem), r.dir(f"shin.{s}", f"foot.{s}")))
    return [piece(body, name, gap, keep, trims=trims, smooth=4, close=close,
                  material=material, faces=1800, folds=folds, trim=0.0)]


def jerkin(body, r, material="leather", gap=0.012, low=0.12, collar=0.03, name="jerkin", faces=2000,
           inset=0.03, close=0.055, trims=(), sleeved=False, smooth=8, trim=0.025):
    """A fitted leather jerkin, collar to the top of the thigh: bridged
    over the bust and the small of the back as stiff leather is, its skirt
    falling past the belt, creased at the waist where it bends."""
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1]
    waist = r.o["spine"]

    def folds(o):
        g.wrinkle(o, waist + np.array([0, 0.03, 0]), [0, 1, 0], 0.06, 0.004, 0.035, seed=1)
        g.drape_folds(o, hip - 0.02, hip - 0.13, 0.004, 9, seed=2)

    holes = [] if sleeved else armholes(r, inset)
    return [piece(body, name, gap, smooth=smooth, close=close,
                  keep=lambda c, fw: w(fw, "root", "spine", "chest", "arm.l", "arm.r", "thigh.l", "thigh.r") > 0.4,
                  trims=[([0, hip - low, 0], [0, -1, 0]), ([0, neck - collar, 0], [0, 1, 0])]
                  + holes + list(trims), faces=faces, folds=folds, trim=trim, material=material)]


def belt(body, r, material="leather_dark", sash="sash", sashed=True, at=0.0):
    """A belt at the hips, and over it a sash (in the team's colour)."""
    hip = r.o["thigh.l"][1] + at
    trunk = lambda c, fw: w(fw, "root", "spine", "thigh.l", "thigh.r") > 0.4  # noqa: E731
    b = piece(body, "belt", 0.015, trunk, close=0.03, trim=0.008,
              trims=[([0, hip + 0.035, 0], [0, 1, 0.12])],
              keeps=[([0, hip - 0.01, 0], [0, 1, 0.12])], material=material, thick=0.006, faces=700)
    if not sashed:
        return [b]
    sash = piece(body, "sash", 0.012, trunk, close=0.03, trim=0.01,
                 trims=[([0, hip + 0.10, 0], [0.25, 1, 0])],
                 keeps=[([0, hip + 0.03, 0], [-0.15, 1, 0])], material=sash, thick=0.003, faces=700)
    return [b, sash]


def coat_tails(body, r, spans=None, material="hood", flare=0.42, bottom=0.30, at=0.005, ripple=0.18):
    """Up to four long panels from the belt down, dropped under cloth
    simulation onto the legs: by default the Reaver's, open at the front,
    split at the back and sides, the sides to the knee and the back pair to
    the calf. ``spans``: (from, to, bottom) round the waist, in degrees from
    the front, each panel's lowest point in metres above the floor."""
    hip = r.o["thigh.l"][1]
    top = hip + at
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
    if spans is None:
        spans = [(52, 104, 0.50), (108, 177, bottom), (183, 252, bottom), (256, 308, 0.50)]
    spans = [(deg(a0), deg(a1), end) for a0, a1, end in spans]
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
            out_by = (top - y) * flare
            for th, rt in zip(thetas, r0):
                # A ripple to seed the folds: cloth falling from a perfect
                # cone stays a cone.
                rr = rt + out_by * (1 + ripple * np.sin(th * 23.0 + k))
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
        g.material(ob, mat(material))
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




def covered(r, c, fw, boots_to=0.22, sleeves=True, trunk=True, trousers=True, thighs_only=False):
    """The usual skin nothing sees, under boots to ``boots_to`` down the
    shin, trousers, sleeves and a top -- for ``hide_covered``."""
    out = np.zeros(len(c), dtype=bool)
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1]
    for s in "lr":
        knee, ankle = f"shin.{s}", f"foot.{s}"
        if boots_to is not None:
            top = r.along(knee, ankle, boots_to)
            out |= (w(fw, knee, ankle, f"toe.{s}") > 0.6) & (((c - top) @ r.dir(ankle, knee)) < 0)
        if trousers:
            out |= w(fw, f"thigh.{s}") > 0.85
            if not thighs_only:
                out |= (w(fw, knee) > 0.85) & (((c - r.along(knee, ankle, 0.2)) @ r.dir(ankle, knee)) > 0)
        if sleeves:
            out |= w(fw, f"arm.{s}") > 0.9
            out |= (w(fw, f"forearm.{s}") > 0.9) & (
                ((c - r.along(f"forearm.{s}", f"hand.{s}", 0.75)) @ r.dir(f"hand.{s}", f"forearm.{s}")) > 0)
    if trunk:
        t = w(fw, "root", "spine", "chest") > 0.85
        out |= t & (c[:, 1] > hip + 0.02) & (c[:, 1] < neck - 0.08) & (np.abs(c[:, 0]) < 0.11)
    return out


def side_mask(ob, r, right=1.0, soft=0.012):
    """A mask on a piece: one on the character's *right* side of the
    middle line (for a material's second colour), nought on the left."""
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    # Her right hangs from the ``.l`` slots (``LEFT``).
    sign = np.sign(r.o[f"arm.{RIGHT}"][0])
    m = np.clip(0.5 + p[:, 0] * sign / soft, 0, 1)
    if right < 1.0:
        m = 1 - m
    g.vertex_mask(ob, m)
    return ob


def cape(body, r, side=None, name="team_cape", material="team", low=0.32, gap=0.03, faces=1600, back_only=False):
    """A short cape over one shoulder (``side``: a slot, ``LEFT`` is her
    left) or both (None): the trunk and the upper arm inflated well clear,
    from the collar to ``low`` below the shoulder, hanging down the back.
    ``back_only``: none of it over the top of the shoulder, where a
    pauldron is."""
    sh = r.o["arm.l"][1]
    neck = r.o["head"][1]
    sign = None if side is None else np.sign(r.o[f"arm.{side}"][0])

    def keep(c, fw):
        k = w(fw, "chest", "spine", "arm.l", "arm.r") > 0.35
        if sign is not None:
            k &= c[:, 0] * sign > -0.02
        # Down the back; in front, only over the top of the shoulder.
        k &= (c[:, 2] < -0.03) if back_only else ((c[:, 2] < 0.02) | (c[:, 1] > sh - 0.02))
        return k

    trims = [([0, sh - low, 0], [0, -1, -0.25]), ([0, neck - 0.06, 0], [0, 1, 0])]

    def folds(o):
        g.drape_folds(o, sh, sh - low, 0.006, 7, seed=31)

    return [piece(body, name, gap, keep, trims=trims, smooth=6, close=0.05, material=material,
                  thick=0.005, voxel=0.005, faces=faces, trim=0.02, folds=folds)]


def collar(body, r, name="collar", material="leather", height=0.07, gap=0.012):
    """A standing collar round the neck."""
    neck = r.o["neck"]
    return [piece(body, name, gap, lambda c, fw: w(fw, "neck", "chest") > 0.4,
                  # Higher at the back than the front, as a standing collar
                  # is cut, so it clears the jaw.
                  trims=[(neck + np.array([0, height - 0.02, 0]), [0, 1, 0.45])],
                  keeps=[(neck + np.array([0, -0.03, 0]), [0, 1, 0])],
                  material=material, thick=0.004, faces=500, trim=0.006, smooth=3)]


def body_masks(body, r, gloved_to=None, brow=1.0, kohl=0.0, lips=0.0):
    """Skin, glove and face paint as the body's three masks."""
    p, W, skin, glove = skin_and_gloves(body, r, gloved_to)
    fp = face_paint(p, r, brow=brow, kohl=kohl, lips=lips) * skin
    g.vertex_mask(body, skin, glove, fp)


def stone(name, at, radii, material, joint, seed=0, facets=1, rough=0.25):
    """A hard rounded thing hung on a joint -- a charm on a belt, a knot of
    hair: an icosphere, squashed to ``radii`` and knocked out of round."""
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2 + facets, radius=1.0,
                                          location=tuple(g.g2b(np.asarray(at, float))))
    ob = bpy.context.active_object
    ob.name = name
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    p = co.reshape(-1, 3)
    rnd = np.random.default_rng(seed)
    bumps = rnd.normal(size=(6, 3))
    k = 1 + rough * 0.15 * np.sin(p @ bumps.T * 2.0).sum(1)
    radii_b = g.g2b(np.asarray(radii, float)[None])[0]
    p = p * k[:, None] * np.abs(radii_b)
    ob.data.vertices.foreach_set("co", p.ravel())
    ob.data.update()
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.shade_smooth()
    g.material(ob, mat(material))
    return rigid(ob, joint)


def band(body, r, a, b, at, width, gap, name, material):
    """A hard band round a limb -- a bracelet, an armlet: a strap, stiffer."""
    return straps(body, r, a, b, at, width, gap, name, material, tilt=0.0)


# --- Hair cards ---------------------------------------------------------------
#
# What real-time hair is in every game that has it: strips of painted strands
# ("cards") with gaps between the strands, laid in layers over a scalp along
# the way the hair is combed. The cut-out edges and the layers over one
# another are what read as hair; a sculpted mass, however it is combed, reads
# as a helmet (the first three tries, 2026-10-21). The scalp (``hair``) stays,
# slimmer, so nothing bald shows between cards.

CARD_VARIANTS = 4


def hair_cards(scalp, r, style="back", length=0.07, layers=(0.004, 0.009, 0.014),
               counts=(40, 55, 65), width=0.024, part=0.0, fringe=False, knot=None,
               jaw=-0.085, seed=1, split=False):
    """Cards over ``scalp`` (a hair piece), combed by ``style``:

    - ``back``: swept back off the forehead and down at the nape, short;
    - ``bob``: falling from a parting at ``part`` to ``jaw`` below the eyes,
      with a ``fringe`` forward over the brow if asked;
    - ``knot``: drawn back to ``knot`` (a point), where a bun sits.

    ``split``: cards on her right use the texture's second colour.
    Returned as one object, its material ``hair_cards``, carried by the
    head."""
    import bmesh
    from mathutils import Vector
    from mathutils.bvhtree import BVHTree

    rnd = np.random.default_rng(seed)
    e = r.eye
    c = r.o["head"] + np.array([0.0, 0.10, -0.01])
    co = np.zeros(len(scalp.data.vertices) * 3)
    scalp.data.vertices.foreach_get("co", co)
    pts = g.b2g(co.reshape(-1, 3))
    tris = []
    for poly in scalp.data.polygons:
        v = list(poly.vertices)
        for k in range(1, len(v) - 1):
            tris.append((v[0], v[k], v[k + 1]))
    tris = np.array(tris)
    a, b, cc = pts[tris[:, 0]], pts[tris[:, 1]], pts[tris[:, 2]]
    area = 0.5 * np.linalg.norm(np.cross(b - a, cc - a), axis=1)
    # Rooted high, as hair grows from the crown and the parting and falls
    # over the rest: cards rooted evenly left the top of the head bare.
    high = (a[:, 1] + b[:, 1] + cc[:, 1]) / 3 - e[1]
    prob = area * (0.25 + np.clip((high + 0.01) / 0.09, 0, 1) ** 1.5)
    prob = prob / prob.sum()
    tree = BVHTree.FromPolygons([Vector(p) for p in pts], tris.tolist())
    right = np.sign(r.o[f"arm.{RIGHT}"][0])

    def surface(p):
        loc, nrm, _, dist = tree.find_nearest(Vector(p))
        return np.array(loc), np.array(nrm)

    def flow(p):
        rel = p - c
        if style == "knot" and knot is not None:
            d = np.asarray(knot) - p
        elif style == "bob":
            if fringe and rel[2] > 0.035 and p[1] > e[1] + 0.01:
                d = np.array([0.0, -0.6, 1.0])
            else:
                # Away from the parting over the crown, then down; at the
                # front, back from the face rather than out over the temple.
                away = np.array([p[0] - part, 0.0, 0.0])
                front = np.clip(rel[2] / 0.06, 0, 1)
                d = (away / max(np.linalg.norm(away), 0.02) * 0.6 * (1 - 0.6 * front)
                     + np.array([0.0, -1.0, -0.15 - 0.6 * front]))
        else:
            d = np.array([0.0, -0.25, -1.0]) if p[1] > e[1] else np.array([0.0, -1.0, -0.3])
        return d / np.linalg.norm(d)

    verts, faces, uvs, sides = [], [], [], []
    for li, (lift, count) in enumerate(zip(layers, counts)):
        pick = rnd.choice(len(tris), size=count, p=prob)
        for t in pick:
            w1, w2 = rnd.random(2)
            if w1 + w2 > 1:
                w1, w2 = 1 - w1, 1 - w2
            p = a[t] + (b[t] - a[t]) * w1 + (cc[t] - a[t]) * w2
            loc, n = surface(p)
            p = loc + n * lift
            reach = length * (0.6 + 0.7 * rnd.random()) * (1.0 - 0.12 * li)
            if style == "knot" and knot is not None:
                reach = min(reach, np.linalg.norm(np.asarray(knot) - p) + 0.01)
            step = 0.008
            line = [p]
            travelled = 0.0
            while travelled < reach:
                d = flow(line[-1])
                below = line[-1][1] < e[1] - 0.025
                if below:
                    # Off the scalp's edge the hair simply falls, a little
                    # in toward the neck, rather than being pushed out from a
                    # surface it has left -- which fanned the ends.
                    flat = (line[-1] - c) * np.array([1.0, 0.0, 1.0])
                    d = np.array([0.0, -1.0, 0.0]) - flat * 1.5 + d * 0.25
                    d /= np.linalg.norm(d)
                q = line[-1] + d * step
                if not below:
                    loc, n2 = surface(q)
                    off = (q - loc) @ n2
                    if off < lift:
                        q = q + n2 * (lift - off)
                if style == "bob" and q[1] < e[1] + jaw:
                    break
                line.append(q)
                travelled += step
            if len(line) < 3:
                continue
            line = np.array(line)
            wid = width * (0.7 + 0.6 * rnd.random())
            on_right = (line[0][0] - c[0]) * right > 0
            variant = int(rnd.integers(0, CARD_VARIANTS // 2)) + (CARD_VARIANTS // 2 if (split and on_right) else 0)
            if not split:
                variant = int(rnd.integers(0, CARD_VARIANTS))
            u0 = variant / CARD_VARIANTS
            twist = rnd.normal() * 0.10
            base = len(verts)
            m = len(line)
            for i, q in enumerate(line):
                tng = line[min(i + 1, m - 1)] - line[max(i - 1, 0)]
                tng /= np.linalg.norm(tng)
                _, nn = surface(q)
                across = np.cross(tng, nn)
                across /= max(np.linalg.norm(across), 1e-6)
                across = across * np.cos(twist) + nn * np.sin(twist)
                s = i / (m - 1)
                half = wid * (1.0 - 0.45 * s) / 2
                verts += [q - across * half, q + across * half]
                uvs += [(u0 + 0.02 / CARD_VARIANTS, 1 - s), (u0 + 0.98 / CARD_VARIANTS, 1 - s)]
            for i in range(m - 1):
                k = base + 2 * i
                faces.append((k, k + 1, k + 3, k + 2))
    ob = stage.mesh_object("cards", g.g2b(np.array(verts)), faces)
    uvl = ob.data.uv_layers.new(name="UVMap")
    lv = np.zeros(len(ob.data.loops), int)
    ob.data.loops.foreach_get("vertex_index", lv)
    uvl.data.foreach_set("uv", np.array(uvs)[lv].ravel())
    for p in ob.data.polygons:
        p.use_smooth = True
    ob.data.materials.append(bpy.data.materials.new("hair_cards"))
    print(f"    {len(faces)} card quads")
    return rigid(ob, "head")


def strand_texture(colour, second=None, size=1024, seed=5):
    """The cards' texture: ``CARD_VARIANTS`` columns, each a lock of strands
    from root (top) to tip (bottom) -- thin, swaying a little, each its own
    shade, darker at the root, a band of sheen a third of the way down, and
    tapering to nothing at uneven tips. Alpha is the strands; between them,
    the colour is kept so a cut edge never fringes dark. Linear RGBA."""
    rnd = np.random.default_rng(seed)
    h = w = size
    rgb = np.zeros((h, w, 3))
    alpha = np.zeros((h, w))
    shade = np.ones((h, w))
    col_w = w // CARD_VARIANTS
    yy = np.arange(h)[:, None] / h
    for v in range(CARD_VARIANTS):
        x0 = v * col_w
        base = np.asarray(second if (second is not None and v >= CARD_VARIANTS // 2) else colour, float)
        rgb[:, x0:x0 + col_w] = base
        for _ in range(190):
            # Thicker in the lock's middle, thinning to its edges, so cards
            # side by side blend into one fall of hair instead of showing
            # each strip's straight sides.
            cx = x0 + col_w * np.clip(0.5 + rnd.normal() * 0.2, 0.03, 0.97)
            sway = rnd.normal() * 2.0
            phase = rnd.random() * 6.28
            thick = 1.0 + 1.8 * rnd.random()
            end = 0.72 + 0.28 * rnd.random()
            tone = 0.7 + 0.6 * rnd.random()
            xs = cx + sway * np.sin(yy[:, 0] * 5.0 + phase) + (yy[:, 0] ** 2) * rnd.normal() * 6.0
            xi = np.arange(x0, x0 + col_w)[None, :]
            d = np.abs(xi - xs[:, None])
            strand = np.clip(1.0 - (d - thick * (1 - 0.6 * yy)) / 1.0, 0, 1)
            strand *= (yy < end) * np.clip((end - yy) / 0.22, 0, 1) ** 0.5
            region = alpha[:, x0:x0 + col_w]
            alpha[:, x0:x0 + col_w] = np.maximum(region, strand)
            shade[:, x0:x0 + col_w] = np.where(strand > 0.3, tone, shade[:, x0:x0 + col_w])
    u = (np.arange(w) % col_w) / col_w
    alpha *= np.clip(1.0 - np.abs(2 * u - 1) ** 2.5, 0, 1)[None, :] ** 0.7
    root = 0.55 + 0.45 * np.clip(yy / 0.25, 0, 1)
    sheen = 1.0 + 0.9 * np.exp(-((yy - 0.32) / 0.06) ** 2)
    out = np.concatenate([rgb * (shade * root * sheen)[..., None], alpha[..., None]], 2)
    return out
