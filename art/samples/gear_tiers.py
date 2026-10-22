"""A look sample, not the game's: the Champion in four tiers of gear.

    art/blender.sh art/samples/gear_tiers.py -- --tier 1..4 [--size 1024]

Opens the Champion's dressed scene (``target/art/champion_dressed.blend``,
which ``art/champion/build.py`` leaves), puts on one tier's armour and sword,
and exports it as ``crates/game/assets/champion.glb`` -- so the game draws
it, painted, with ``--painted-cast``. Put the real file back after
(``git checkout crates/game/assets/champion.glb``).

The tiers (``docs/design/exploration/0026_gear_tiers.md``), told apart by four
things at once -- outline, materials, ornament, magic -- and made of what the
valley's creatures leave, as a hunter's gear is:

1. **Basic**: boiled leather and iron, riveted. The outline is the body's.
2. **Upgraded**: steel plates edged in bronze, a Hornback's horn on the
   shoulder. Two materials and a trophy.
3. **Epic**: blue steel cut in the Galewing's feathers, gold filigree that
   glows faintly, a gem. The outline changes.
4. **Legendary**: pale Siegeshell plates that float apart, held by gold
   light. The outline stops being physical.
"""

import json
import math
import os
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path[:0] = [os.path.join(ROOT, "art", "lib"), os.path.join(ROOT, "art", "champion")]

import bmesh  # noqa: E402
import bpy  # noqa: E402
import numpy as np  # noqa: E402

import character  # noqa: E402
import garment as g  # noqa: E402
import stage  # noqa: E402
import wardrobe  # noqa: E402
from wardrobe import J, piece, rigid, w  # noqa: E402


def champion_spec():
    """The Champion's spec, without running his build."""
    src = open(os.path.join(ROOT, "art", "champion", "build.py")).read()
    src = src.replace("character.build(sys.modules[__name__])", "")
    spec = types.ModuleType("champion_spec")
    spec.__file__ = os.path.join(ROOT, "art", "champion", "build.py")
    exec(compile(src, spec.__file__, "exec"), spec.__dict__)
    return spec


# The gear's colours, linear albedo, and which of them are metal.
GEAR = {
    "hide": (0.11, 0.055, 0.025),       # boiled leather
    "iron": (0.075, 0.075, 0.08),
    "grip": (0.035, 0.02, 0.012),
    "steel": (0.20, 0.21, 0.23),
    "bronze": (0.30, 0.16, 0.06),
    "horn": (0.36, 0.30, 0.21),
    "galesteel": (0.03, 0.05, 0.11),     # blued steel, his doublet's blue
    "gold": (0.60, 0.40, 0.12),
    "gem": (0.05, 0.25, 0.60),
    "shell": (0.55, 0.53, 0.50),         # the Siegeshell's plate, pale
    "core": (0.95, 0.58, 0.14),          # what holds a legendary together
}
GEAR_METAL = {"iron": (0.85, 0.55), "steel": (1.0, 0.32), "bronze": (1.0, 0.38),
              "galesteel": (1.0, 0.28), "gold": (1.0, 0.25), "shell": (0.25, 0.3), "core": (1.0, 0.2)}


class Rig:
    """Joint places from the dressed scene's saved bind (the build's
    ``wardrobe.Rig`` without the skeleton it was made from)."""

    def __init__(self, origin):
        self.o = {n: np.asarray(origin[i]) for n, i in J.items()}

    def along(self, a, b, t):
        return self.o[a] + (self.o[b] - self.o[a]) * t

    def dir(self, a, b):
        d = self.o[b] - self.o[a]
        return d / np.linalg.norm(d)


def unit(v):
    v = np.asarray(v, dtype=float)
    return v / np.linalg.norm(v)


# --- Shapes, in the game's coordinates (y up, +z the way he faces) ----------

def mesh(name, verts, faces, material, joint, glow=0.0):
    ob = stage.mesh_object(name, g.g2b(np.asarray(verts, dtype=float)), faces)
    # Every shape here is closed; its faces turned to face out, whichever
    # way round they were written (the game draws one side only).
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(ob.data)
    bm.free()
    g.material(ob, wardrobe.mat(material))
    ob.data.shade_smooth()
    rigid(ob, joint)
    if glow > 0:
        grp = ob.vertex_groups.new(name="edge")
        grp.add(list(range(len(ob.data.vertices))), glow, "REPLACE")
    return ob


def frame(axis, hint=(0.0, 0.0, 1.0)):
    a = unit(axis)
    s = np.cross(a, hint)
    if np.linalg.norm(s) < 1e-4:
        s = np.cross(a, [1.0, 0.0, 0.0])
    s = unit(s)
    return a, s, np.cross(s, a)


def loft(rings, cap0=True, cap1=True):
    """Faces joining rings of equal size, capped at both ends."""
    n = len(rings[0])
    verts = [p for ring in rings for p in ring]
    faces = []
    for i in range(len(rings) - 1):
        for j in range(n):
            a, b = i * n + j, i * n + (j + 1) % n
            faces.append([a, b, b + n, a + n])
    if cap0:
        faces.append(list(range(n))[::-1])
    if cap1:
        faces.append([(len(rings) - 1) * n + j for j in range(n)])
    return verts, faces


def tube(a, b, radius, n=10, r1=None, hint=(0, 0, 1)):
    ax, s, u = frame(np.subtract(b, a), hint)
    r1 = radius if r1 is None else r1
    rings = []
    for p, r in ((np.asarray(a), radius), (np.asarray(b), r1)):
        rings.append([p + (s * math.cos(t) + u * math.sin(t)) * r for t in np.linspace(0, 2 * math.pi, n, endpoint=False)])
    return loft(rings)


def ball(c, radius, n=10, squash=(1.0, 1.0, 1.0)):
    rings = []
    for i in range(1, n):
        ph = math.pi * i / n
        rings.append([np.asarray(c) + np.array([math.sin(ph) * math.cos(t) * radius * squash[0],
                                                  math.cos(ph) * radius * squash[1],
                                                  math.sin(ph) * math.sin(t) * radius * squash[2]])
                      for t in np.linspace(0, 2 * math.pi, n, endpoint=False)])
    verts, faces = loft(rings, cap0=False, cap1=False)
    top, bot = len(verts), len(verts) + 1
    verts += [np.asarray(c) + [0, radius * squash[1], 0], np.asarray(c) - [0, radius * squash[1], 0]]
    m = n
    for j in range(m):
        faces.append([top, (j + 1) % m, j])
        last = (n - 2) * m
        faces.append([bot, last + j, last + (j + 1) % m])
    return verts, faces


def blade(base, axis, side, length, width, thick, shape, steps=14):
    """A blade: diamond sections along ``axis`` from ``base``, ``shape(t)``
    its width at ``t`` of the way to the point."""
    ax = unit(axis)
    sd = unit(side)
    nm = np.cross(ax, sd)
    rings = []
    for i in range(steps + 1):
        t = i / steps
        c = np.asarray(base) + ax * length * t
        wd = width * shape(t)
        th = thick * max(shape(t), 0.15)
        rings.append([c + sd * wd, c + nm * th, c - sd * wd, c - nm * th])
    return loft(rings)


def leaf(root, out, side, length, width, thick, curl=0.0):
    """A feather or a scale: a flat, pointed leaf from ``root`` along ``out``,
    curling toward ``side`` x ``out`` by ``curl``."""
    out = unit(out)
    sd = unit(side)
    nm = np.cross(out, sd)
    rings = []
    for i in range(9):
        t = i / 8
        c = np.asarray(root) + out * length * t + nm * curl * length * t * t
        wd = width * math.sin(math.pi * min(t * 1.15, 1.0)) ** 0.7 + 0.002
        rings.append([c + sd * wd, c + nm * thick, c - sd * wd, c - nm * thick])
    return loft(rings)


def hoop(c, axis, radius, minor, n=24, m=6):
    ax, s, u = frame(axis)
    verts, faces = [], []
    for i in range(n):
        a = 2 * math.pi * i / n
        rc = np.asarray(c) + (s * math.cos(a) + u * math.sin(a)) * radius
        rd = unit(rc - c)
        for j in range(m):
            b = 2 * math.pi * j / m
            verts.append(rc + (rd * math.cos(b) + ax * math.sin(b)) * minor)
    for i in range(n):
        for j in range(m):
            a, b = i * m + j, i * m + (j + 1) % m
            c2, d = ((i + 1) % n) * m + (j + 1) % m, ((i + 1) % n) * m + j
            faces.append([a, b, c2, d])
    return verts, faces


def horn(root, out, up, length, radius):
    """A horn: a tube that tapers to a point and curls toward ``up``."""
    out, up = unit(out), unit(up)
    rings = []
    for i in range(12):
        t = i / 11
        c = np.asarray(root) + out * length * t + up * length * 0.45 * t * t
        tan = unit(out + up * 0.9 * t)
        _, s, u = frame(tan)
        r = radius * (1 - t) ** 0.9 + 0.0015
        rings.append([c + (s * math.cos(a) + u * math.sin(a)) * r for a in np.linspace(0, 2 * math.pi, 8, endpoint=False)])
    return loft(rings)


def merge(*parts):
    verts, faces = [], []
    for v, f in parts:
        k = len(verts)
        verts += list(v)
        faces += [[i + k for i in face] for face in f]
    return verts, faces


# --- The pieces --------------------------------------------------------------

def reach(body, centre, within=0.16, outward=None):
    """How far the body (and so, with a margin, his clothes) reaches from
    ``centre``: the furthest of its points within ``within`` of it, on the
    ``outward`` side if one is given."""
    p = verts_of(body)
    d = p - centre
    near = np.linalg.norm(d, axis=1) < within
    if outward is not None:
        near &= d @ unit(outward) > 0.0
    return float(np.linalg.norm(d[near], axis=1).max())


def lame(body, r, name, material, th0, th1, out=0.0, phi=80.0, thick=0.006, glow=0.0):
    """A shoulder plate: a band of a sphere round the shoulder joint, from
    ``th0`` to ``th1`` degrees down from the top, ``phi`` either way round
    from straight out, its lower edge rolled out a little. Hard-edged, as
    plate is; sized from the body so it stands clear of the clothes."""
    c = r.o["arm.l"] + np.array([0.0, 0.01, 0.0])
    R = reach(body, c, 0.13, outward=[-1.0, 0.6, 0.0]) + 0.035 + out
    nt, nphi = 8, 18
    rings_out, rings_in = [], []
    for i in range(nt + 1):
        t = i / nt
        th = math.radians(th0 + (th1 - th0) * t)
        roll = 1.0 + 0.06 * t ** 3
        ro, ri = [], []
        for k in range(nphi + 1):
            ph = math.radians(-phi + 2 * phi * k / nphi)
            d = np.array([-math.sin(th) * math.cos(ph), math.cos(th), math.sin(th) * math.sin(ph)])
            ro.append(c + d * R * roll)
            ri.append(c + d * (R * roll - thick))
        rings_out.append(ro)
        rings_in.append(ri)
    n = nphi + 1
    verts = [p for ring in rings_out for p in ring] + [p for ring in rings_in for p in ring]
    off = len(verts) // 2
    faces = []
    for i in range(nt):
        for k in range(nphi):
            a, b = i * n + k, i * n + k + 1
            faces.append([a, b, b + n, a + n])
            faces.append([off + a + n, off + b + n, off + b, off + a])
    # The rim: the four edges closed between the faces.
    for i in range(nt):
        for k in (0, nphi):
            a, b = i * n + k, (i + 1) * n + k
            faces.append([a, b, off + b, off + a] if k == 0 else [b, a, off + a, off + b])
    for i in (0, nt):
        for k in range(nphi):
            a, b = i * n + k, i * n + k + 1
            faces.append([b, a, off + a, off + b] if i == 0 else [a, b, off + b, off + a])
    return mesh(name, verts, faces, material, "arm.l", glow=glow)


def plate_gorget(body, r, name, material, low=0.06, high=0.03, out=0.0, phi=95.0, thick=0.006, glow=0.0):
    """A plate at the throat: round the front of the neck and down over the
    top of the chest, each height's radius measured from the body round
    it."""
    p = verts_of(body)
    y0, y1 = r.o["chest"][1] + low, r.o["neck"][1] + high
    nh, nphi = 5, 20
    rings_out, rings_in = [], []
    for i in range(nh + 1):
        t = i / nh
        y = y1 + (y0 - y1) * t
        band = p[np.abs(p[:, 1] - y) < 0.02]
        band = band[np.abs(band[:, 0]) < 0.17]
        ro, ri = [], []
        for k in range(nphi + 1):
            ph = math.radians(-phi + 2 * phi * k / nphi)
            d = np.array([math.sin(ph), 0.0, math.cos(ph)])
            ang = np.arctan2(band[:, 0], band[:, 2])
            sel = band[np.abs(ang - ph) < math.radians(12)]
            rad = float(np.max(sel[:, 0] * d[0] + sel[:, 2] * d[2])) if len(sel) else 0.12
            rr = rad + 0.04 + out + 0.025 * t * t
            ro.append(np.array([0.0, y, 0.0]) + d * rr)
            ri.append(np.array([0.0, y, 0.0]) + d * (rr - thick))
        rings_out.append(ro)
        rings_in.append(ri)
    n = nphi + 1
    verts = [q for ring in rings_out for q in ring] + [q for ring in rings_in for q in ring]
    off = len(verts) // 2
    faces = []
    for i in range(nh):
        for k in range(nphi):
            a, b = i * n + k, i * n + k + 1
            faces.append([a, a + n, b + n, b])
            faces.append([off + b, off + b + n, off + a + n, off + a])
    for i in range(nh):
        for k in (0, nphi):
            a, b = i * n + k, (i + 1) * n + k
            faces.append([b, a, off + a, off + b] if k == 0 else [a, b, off + b, off + a])
    for i in (0, nh):
        for k in range(nphi):
            a, b = i * n + k, i * n + k + 1
            faces.append([a, b, off + b, off + a] if i == 0 else [b, a, off + a, off + b])
    return mesh(name, verts, faces, material, "chest", glow=glow)


def bracer(r, body, name, material, gap, at0=0.25, at1=1.02, faces=500):
    e, h = "forearm.r", "hand.r"
    return piece(body, name, gap, lambda c, fw: w(fw, e, h) > 0.3,
                 trims=[(r.along(e, h, at0), r.dir(h, e)), (r.along(e, h, at1), r.dir(e, h))],
                 material=material, faces=faces, trim=0.008)


def verts_of(ob):
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    return g.b2g(co.reshape(-1, 3))


def rivets(ob, joint, material, count, radius=0.006, at="low"):
    """Rivets round the lower (or upper) edge of a piece."""
    p = verts_of(ob)
    y = p[:, 1]
    band = p[(y < y.min() + 0.012)] if at == "low" else p[(y > y.max() - 0.012)]
    if len(band) == 0:
        return []
    mid = band.mean(axis=0)
    ang = np.arctan2(band[:, 2] - mid[2], band[:, 0] - mid[0])
    out = []
    for k, a in enumerate(np.linspace(ang.min(), ang.max(), count)):
        i = int(np.argmin(np.abs(ang - a)))
        c = band[i] + unit(band[i] - mid) * 0.004 + np.array([0, 0.01 if at == "low" else -0.01, 0])
        out.append(mesh(f"rivet.{ob.name}.{k}", *ball(c, radius, 6), material, joint))
    return out


def sword(r, tier):
    """His sword, in the left hand, held low and forward."""
    hand = r.o["hand.r"]
    arm = r.dir("forearm.r", "hand.r")
    fwd = np.array([0.0, 0.0, 1.0])
    ax = unit(arm * 0.55 + fwd * 0.85)
    side = unit(np.cross(ax, [1.0, 0.0, 0.0]))
    grip_c = hand + arm * 0.07 + np.array([-0.012, 0.0, 0.0])
    j = "hand.r"
    parts = []
    if tier == 1:
        # An arming sword: straight, a bar guard, a leather grip, a disc.
        parts.append(mesh("grip", *tube(grip_c - ax * 0.07, grip_c + ax * 0.05, 0.016), "grip", j))
        parts.append(mesh("guard", *tube(grip_c + ax * 0.06 - side * 0.09, grip_c + ax * 0.06 + side * 0.09, 0.009), "iron", j))
        parts.append(mesh("pommel", *ball(grip_c - ax * 0.085, 0.022, 8, (1, 0.6, 1)), "iron", j))
        parts.append(mesh("blade", *blade(grip_c + ax * 0.07, ax, side, 0.72, 0.024, 0.006,
                                           lambda t: 1.0 - 0.15 * t if t < 0.88 else (1 - t) / 0.12 * 0.85), "iron", j))
    elif tier == 2:
        # Steel with a fuller's spine, a guard of bronze curling like horns,
        # the grip bound in hide, a bronze pommel.
        parts.append(mesh("grip", *tube(grip_c - ax * 0.08, grip_c + ax * 0.05, 0.017), "hide", j))
        g1 = grip_c + ax * 0.065
        parts.append(mesh("guard.l", *horn(g1, side, ax, 0.11, 0.014), "bronze", j))
        parts.append(mesh("guard.r", *horn(g1, -side, ax, 0.11, 0.014), "bronze", j))
        parts.append(mesh("pommel", *ball(grip_c - ax * 0.1, 0.026, 8, (1, 0.8, 1)), "bronze", j))
        parts.append(mesh("blade", *blade(grip_c + ax * 0.07, ax, side, 0.8, 0.03, 0.007,
                                           lambda t: 1.05 - 0.2 * t if t < 0.85 else (1 - t) / 0.15 * 0.88), "steel", j))
        parts.append(mesh("spine", *blade(grip_c + ax * 0.08, ax, side, 0.62, 0.006, 0.009, lambda t: 1.0 - 0.5 * t), "bronze", j))
    elif tier == 3:
        # Blued steel, leaf-shaped toward the point, a guard of swept
        # feathers, gold inlay down the middle that glows faintly, a gem.
        parts.append(mesh("grip", *tube(grip_c - ax * 0.08, grip_c + ax * 0.05, 0.016), "grip", j))
        g1 = grip_c + ax * 0.065
        for k, sgn in enumerate((1, -1)):
            for f in range(3):
                out = unit(side * sgn * (1.0 - 0.2 * f) + ax * (0.35 + 0.35 * f) - fwd * 0.1)
                parts.append(mesh(f"feather.{k}.{f}", *leaf(g1, out, np.cross(out, fwd), 0.10 - 0.02 * f, 0.016, 0.004, curl=0.15),
                                  "galesteel", j))
        parts.append(mesh("gem", *ball(grip_c - ax * 0.105, 0.022, 10), "gem", j, glow=0.5))
        parts.append(mesh("cap", *hoop(grip_c - ax * 0.105, ax, 0.024, 0.004), "gold", j))
        parts.append(mesh("blade", *blade(grip_c + ax * 0.07, ax, side, 0.86, 0.03, 0.007,
                                           lambda t: 0.85 + 0.35 * math.sin(math.pi * min(t / 0.8, 1.0)) if t < 0.8 else (1 - t) / 0.2 * 0.85),
                          "galesteel", j))
        parts.append(mesh("inlay", *blade(grip_c + ax * 0.08, ax, side, 0.7, 0.005, 0.0085, lambda t: 1.0 - 0.6 * t),
                          "gold", j, glow=0.35))
    else:
        # A blade in two halves held a finger apart by gold light, its point
        # in pieces that float ahead of it, a ring of light at the guard.
        parts.append(mesh("grip", *tube(grip_c - ax * 0.08, grip_c + ax * 0.05, 0.016), "shell", j))
        parts.append(mesh("pommel", *ball(grip_c - ax * 0.11, 0.024, 10, (1, 1.4, 1)), "core", j, glow=1.0))
        parts.append(mesh("halo", *hoop(grip_c + ax * 0.07, ax, 0.075, 0.004, 32), "core", j, glow=1.0))
        nm = np.cross(ax, side)
        for k, sgn in enumerate((1, -1)):
            base = grip_c + ax * 0.08 + side * sgn * 0.017
            parts.append(mesh(f"half.{k}", *blade(base, ax, side, 0.74, 0.016, 0.006,
                                                  lambda t: 1.0 - 0.3 * t), "shell", j))
        parts.append(mesh("core", *blade(grip_c + ax * 0.07, ax, side, 0.8, 0.007, 0.004, lambda t: 1.0 - 0.7 * t),
                          "core", j, glow=1.0))
        for f in range(4):
            c = grip_c + ax * (0.87 + 0.07 * f)
            parts.append(mesh(f"shard.{f}", *blade(c, ax, side + nm * 0.3 * (f % 2), 0.05 - 0.008 * f, 0.022 - 0.004 * f, 0.005,
                                                   lambda t: 1.0 - t), "shell", j))
    return parts


def gear(body, r, tier):
    """One tier's armour and sword."""
    out = []
    if tier == 1:
        cap = lame(body, r, "pauldron", "hide", 0, 72, thick=0.008)
        out += [cap] + rivets(cap, "arm.l", "iron", 6)
        vb = bracer(r, body, "bracer", "hide", 0.02)
        out += [vb] + rivets(vb, "forearm.r", "iron", 4, at="high")
        out.append(plate_gorget(body, r, "gorget", "hide", low=0.1, thick=0.008))
    elif tier == 2:
        for k, (a, b) in enumerate([(0, 44), (36, 66), (58, 92)]):
            out.append(lame(body, r, f"lame{k}", "steel", a, b, out=0.008 * k))
            out.append(lame(body, r, f"lametrim{k}", "bronze", b - 5, b, out=0.008 * k + 0.003, thick=0.004))
        top = r.o["arm.l"] + np.array([-0.04, 0.09, -0.02])
        out.append(mesh("hornback", *horn(top, [-0.3, 0.2, -1.0], [-0.4, 1.0, 0.0], 0.2, 0.026), "horn", "arm.l"))
        out.append(bracer(r, body, "bracer", "steel", 0.022))
        out.append(bracer(r, body, "bracertrim", "bronze", 0.026, at0=0.25, at1=0.33, faces=200))
        out.append(plate_gorget(body, r, "gorget", "steel"))
        out.append(plate_gorget(body, r, "gorgettrim", "bronze", low=0.21, out=0.003, thick=0.004))
        out += rivets(out[-1], "chest", "bronze", 5)
    elif tier == 3:
        for k, (a, b) in enumerate([(0, 50), (42, 80)]):
            out.append(lame(body, r, f"lame{k}", "galesteel", a, b, out=0.01 * k))
            out.append(lame(body, r, f"filigree{k}", "gold", b - 6, b, out=0.01 * k + 0.003, thick=0.004, glow=0.4))
        # Galewing feathers fanned back and down off the shoulder.
        sh = r.o["arm.l"] + np.array([-0.05, 0.06, -0.03])
        for f in range(5):
            a = -0.5 + 0.28 * f
            outd = unit([-0.55 - 0.1 * f, 0.15 - 0.22 * f, -0.75])
            out.append(mesh(f"feather{f}", *leaf(sh + np.array([0, -0.025 * f, 0.02 * f]), outd,
                                                  np.cross(outd, [0, 1, 0]) + [0, a * 0.2, 0], 0.22 - 0.02 * f, 0.03, 0.004, curl=-0.25),
                            "galesteel", "arm.l"))
        out.append(mesh("shouldergem", *ball(r.o["arm.l"] + np.array([-0.075, 0.05, 0.05]), 0.018, 10), "gem", "arm.l", glow=0.5))
        out.append(bracer(r, body, "bracer", "galesteel", 0.022))
        out.append(bracer(r, body, "bracertrim", "gold", 0.026, at0=0.25, at1=0.31, faces=200))
        out.append(plate_gorget(body, r, "gorget", "galesteel"))
        out.append(plate_gorget(body, r, "gorgettrim", "gold", low=0.2, out=0.003, thick=0.004, glow=0.4))
    else:
        # Three plates hung apart round the shoulder, gold light between.
        # Three plates hung apart, each further out, gold light in the gaps.
        for k, (a, b) in enumerate([(0, 32), (42, 64), (74, 98)]):
            out.append(lame(body, r, f"plate{k}", "shell", a, b, out=0.022 * k, phi=70 + 6 * k))
        for k, (a, b) in enumerate([(35, 39), (67, 71)]):
            out.append(lame(body, r, f"seam{k}", "core", a, b, out=0.011 + 0.022 * k, phi=60, thick=0.003, glow=1.0))
        out.append(mesh("crown", *hoop(r.o["arm.l"] + np.array([-0.05, 0.16, 0.0]), [-0.3, 1.0, 0.0], 0.06, 0.005, 28),
                        "core", "arm.l", glow=1.0))
        out.append(bracer(r, body, "bracer", "shell", 0.03, at0=0.3, at1=0.62))
        out.append(bracer(r, body, "bracer2", "shell", 0.04, at0=0.7, at1=1.0))
        c = r.along("forearm.r", "hand.r", 0.66)
        out.append(mesh("wristlight", *hoop(c, r.dir("forearm.r", "hand.r"), 0.06, 0.004, 28), "core", "forearm.r", glow=1.0))
        out.append(plate_gorget(body, r, "gorget", "shell", low=0.1, out=0.01))
        out.append(plate_gorget(body, r, "gorgetlight", "core", low=0.24, high=0.0, out=0.02, thick=0.003, glow=1.0))
        out.append(mesh("gorgetcore", *ball(r.o["chest"] + np.array([0.0, 0.17, 0.16]), 0.02, 10), "core", "chest", glow=1.0))
    return out + sword(r, tier)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    tier = int(args[args.index("--tier") + 1])
    size = int(args[args.index("--size") + 1]) if "--size" in args else 1024
    spec = champion_spec()
    spec.COLOURS = dict(spec.COLOURS, **GEAR)
    spec.METAL = dict(spec.METAL, **GEAR_METAL)
    wardrobe.PALETTE.update({k: v for k, v in spec.COLOURS.items() if "/" not in k})
    bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "target/art/champion_dressed.blend"))
    meta = json.loads(bpy.context.scene["bind"])
    bo, br = np.array(meta["o"]), np.array(meta["r"])
    wardrobe.EXTRAS.update(meta.get("extras", {}))
    r = Rig(bo)
    body = bpy.data.objects["body"]
    pieces = gear(body, r, tier)
    for p in pieces:
        if not p.get("rigid"):
            g.bind_to(p, body)
    character.finish(spec, bo, br, size)


main()
