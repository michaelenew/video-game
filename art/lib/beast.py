"""A creature, formed by hand over its skeleton: the tools.

A creature in the game is a skeleton of bones and a box per part
(``sim::species``); ``cargo run -p anim --bin export_beast`` writes both, at
rest, to ``target/art/<species>.json``. A creature's ``build.py`` forms a body
over that skeleton with what is here, and writes it as a skinned glTF the game
poses with the bone transforms it already has.

Everything is written in the **game's frame**: metres, ``+x`` toward the
head, ``+y`` up, ``+z`` the creature's right. Blender sees it turned so its
``+z`` is up (``to_blender``); nothing a script writes needs to know.

The pieces, in the order a body is made:

- **Forms** (``tube``, ``blob``, ``horn``): closed meshes in numpy. A tube is
  a run of cross-sections along a curve, each a superellipse with bumps --
  how a sculptor blocks a limb in, and the same generalised cylinder
  ``game::contour`` grows a body from, so what is learned here goes back
  there (``docs/design/exploration/0028_a_target_body.md``).
- **Flesh** (``flesh``): every soft form unioned into one watertight skin by
  a voxel remesh, its seams rounded, so nothing can float or crease.
- **Plates** (``plate_on``): a patch of the skin lifted off it and given a
  thickness, a dome and a bevel, so armour fits what it covers.
- **Weights** (``weights``): every vertex to the bones nearest it, smoothed
  over the skin; a rigid piece takes the weights of the skin under it.
- **The file** (``write_glb``): one primitive a material, the joints in the
  species' bone order at their rest places.
"""

import json
import math
import os
import struct
import subprocess

import numpy as np

try:  # the forms need nothing but numpy; the rest needs Blender
    import bpy
    import bmesh  # noqa: F401  (after bpy, or it does not load)
    import mathutils
except ImportError:  # pragma: no cover
    bpy = None


# ---------------------------------------------------------------------------
# The skeleton
# ---------------------------------------------------------------------------


class Skeleton:
    """The species' bones and parts at rest, read from ``export_beast``."""

    def __init__(self, path):
        with open(path) as f:
            d = json.load(f)
        self.name = d["species"]
        self.bones = d["bones"]
        self.parts = d["parts"]
        self.index = {b["name"]: i for i, b in enumerate(self.bones)}

    def at(self, name):
        return np.array(self.bones[self.index[name]]["at"], float)

    def frame(self, i):
        """A bone's rest frame as a 4x4, columns its axes and origin."""
        b = self.bones[i]
        m = np.eye(4)
        m[:3, 0], m[:3, 1], m[:3, 2] = b["x"], b["y"], b["z"]
        for c in range(3):
            m[:3, c] /= np.linalg.norm(m[:3, c])
        m[:3, 3] = b["at"]
        return m

    def part_box(self, i):
        """A part's box in the world at rest: its corners' bounds."""
        p = self.parts[i]
        m = self.frame(p["bone"])
        lo, hi = np.array(p["min"]), np.array(p["max"])
        c = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])])
        w = c @ m[:3, :3].T + m[:3, 3]
        return w.min(0), w.max(0)


# ---------------------------------------------------------------------------
# Curves
# ---------------------------------------------------------------------------


def spline(points, n):
    """``n`` points along a centripetal Catmull-Rom through ``points``,
    spaced evenly by length."""
    p = np.asarray(points, float)
    p = np.vstack([2 * p[0] - p[1], p, 2 * p[-1] - p[-2]])
    dense = []
    for i in range(1, len(p) - 2):
        p0, p1, p2, p3 = p[i - 1], p[i], p[i + 1], p[i + 2]

        def tj(ti, a, b):
            return ti + max(np.linalg.norm(b - a), 1e-6) ** 0.5

        t0 = 0.0
        t1 = tj(t0, p0, p1)
        t2 = tj(t1, p1, p2)
        t3 = tj(t2, p2, p3)
        for t in np.linspace(t1, t2, 40, endpoint=False):
            a1 = (t1 - t) / (t1 - t0) * p0 + (t - t0) / (t1 - t0) * p1
            a2 = (t2 - t) / (t2 - t1) * p1 + (t - t1) / (t2 - t1) * p2
            a3 = (t3 - t) / (t3 - t2) * p2 + (t - t2) / (t3 - t2) * p3
            b1 = (t2 - t) / (t2 - t0) * a1 + (t - t0) / (t2 - t0) * a2
            b2 = (t3 - t) / (t3 - t1) * a2 + (t - t1) / (t3 - t1) * a3
            dense.append((t2 - t) / (t2 - t1) * b1 + (t - t1) / (t2 - t1) * b2)
    dense.append(p[-2])
    dense = np.array(dense)
    seg = np.linalg.norm(np.diff(dense, axis=0), axis=1)
    s = np.concatenate([[0], np.cumsum(seg)])
    want = np.linspace(0, s[-1], n)
    return np.stack([np.interp(want, s, dense[:, k]) for k in range(3)], 1)


def frames(curve, ref):
    """Tangent, side and up along a curve: ``up`` as near ``ref`` as the
    tangent allows (parallel transport would twist a body that is meant to
    keep its back on top)."""
    t = np.gradient(curve, axis=0)
    t /= np.linalg.norm(t, axis=1, keepdims=True)
    ref = np.asarray(ref, float)
    up = ref - t * (t @ ref)[:, None]
    up /= np.linalg.norm(up, axis=1, keepdims=True)
    side = np.cross(t, up)
    return t, side, up


def interp(stations, key, s):
    """A station value at ``s`` (0 to 1 along a tube), eased between
    stations so a profile has no corners."""
    ts = np.array([st["t"] for st in stations], float)
    vs = np.array([st[key] for st in stations], float)
    i = np.clip(np.searchsorted(ts, s, side="right") - 1, 0, len(ts) - 2)
    u = np.clip((s - ts[i]) / np.maximum(ts[i + 1] - ts[i], 1e-9), 0, 1)
    u = u * u * (3 - 2 * u)
    return vs[i] * (1 - u) + vs[i + 1] * u


# ---------------------------------------------------------------------------
# Forms: closed meshes as (verts, faces), faces as quads or triangles
# ---------------------------------------------------------------------------


def _signed_pow(x, e):
    return np.sign(x) * np.abs(x) ** e


def section(theta, w, top, bot, n_top=2.0, n_bot=2.0, bumps=()):
    """A cross-section's points as (side, up) for angles ``theta`` (0 on
    top, increasing toward the side ``+side``): a superellipse ``w`` wide
    either way, reaching ``top`` above and ``bot`` below the centre, with
    each half its own squareness, and ``bumps`` -- ``(angle, width,
    height)`` -- pushed out along the radius, the muscle under a skin."""
    c, s = np.cos(theta), np.sin(theta)
    n = np.where(c >= 0, n_top, n_bot)
    up = _signed_pow(c, 2.0 / n) * np.where(c >= 0, top, bot)
    side = _signed_pow(s, 2.0 / n) * w
    r = np.hypot(side, up)
    k = np.ones_like(theta)
    for a, width, h in bumps:
        d = np.angle(np.exp(1j * (theta - a)))
        k = k + h * np.exp(-(d / width) ** 2)
    return side * k, up * k, r


def tube(path, stations, ref=(0, 1, 0), sides=48, rings=None, cap=True):
    """A closed tube along the curve through ``path``.

    ``stations``: dicts with ``t`` (0 to 1 along it) and ``w``, ``top``,
    ``bot``, and optionally ``nt``/``nb`` (squareness), ``dy`` (the section
    raised off the curve), ``bumps`` (the bumps of the nearest station on the
    near side, eased toward the next station's when both have the same
    count). ``ref`` is which way the sections' ``top`` faces."""
    curve0 = np.asarray(path, float)
    length = np.sum(np.linalg.norm(np.diff(curve0, axis=0), axis=1))
    rings = rings or max(8, int(length / 0.04))
    curve = spline(curve0, rings)
    t, side, up = frames(curve, ref)
    s = np.linspace(0, 1, rings)
    theta = np.linspace(0, 2 * np.pi, sides, endpoint=False)
    for st in stations:
        st.setdefault("nt", 2.0)
        st.setdefault("nb", 2.0)
        st.setdefault("dy", 0.0)
        st.setdefault("dz", 0.0)
        st.setdefault("bumps", ())
    w = interp(stations, "w", s)
    top = interp(stations, "top", s)
    bot = interp(stations, "bot", s)
    nt = interp(stations, "nt", s)
    nb = interp(stations, "nb", s)
    dy = interp(stations, "dy", s)
    dz = interp(stations, "dz", s)
    ts = np.array([st["t"] for st in stations])
    verts = []
    for i in range(rings):
        j = int(np.clip(np.searchsorted(ts, s[i], side="right") - 1, 0, len(ts) - 1))
        k = min(j + 1, len(ts) - 1)
        b0, b1 = stations[j]["bumps"], stations[k]["bumps"]
        if len(b0) == len(b1) and k != j:
            u = np.clip((s[i] - ts[j]) / max(ts[k] - ts[j], 1e-9), 0, 1)
            u = u * u * (3 - 2 * u)
            bumps = [tuple(a * (1 - u) + b * u for a, b in zip(p, q)) for p, q in zip(b0, b1)]
        else:
            bumps = b0
        sx, sy, _ = section(theta, w[i], top[i], bot[i], nt[i], nb[i], bumps)
        c = curve[i] + up[i] * dy[i] + side[i] * dz[i]
        verts.append(c + np.outer(sx, side[i]) + np.outer(sy, up[i]))
    verts = np.concatenate(verts)
    faces = []
    for i in range(rings - 1):
        a, b = i * sides, (i + 1) * sides
        for j in range(sides):
            k = (j + 1) % sides
            faces.append([a + j, b + j, b + k, a + k])
    if cap:
        c0 = len(verts)
        c1 = c0 + 1
        verts = np.vstack([verts, verts[:sides].mean(0), verts[-sides:].mean(0)])
        last = (rings - 1) * sides
        for j in range(sides):
            k = (j + 1) % sides
            faces.append([c0, j, k])
            faces.append([c1, last + k, last + j])
    return verts, faces


def blob(center, radii, axes=None, n=24, power=2.0):
    """An ellipsoid -- or, with ``power`` above two, a rounded box -- round
    ``center``, its radii along ``axes`` (rows; the world's by default)."""
    axes = np.eye(3) if axes is None else np.asarray(axes, float)
    axes = axes / np.linalg.norm(axes, axis=1, keepdims=True)
    lat = np.linspace(0, np.pi, n // 2 + 1)
    lon = np.linspace(0, 2 * np.pi, n, endpoint=False)
    verts = []
    e = 2.0 / power
    for a in lat:
        for b in lon:
            x = _signed_pow(np.sin(a), e) * _signed_pow(np.cos(b), e)
            y = _signed_pow(np.sin(a), e) * _signed_pow(np.sin(b), e)
            z = _signed_pow(np.cos(a), e)
            verts.append(np.asarray(center) + radii[0] * x * axes[0] + radii[1] * y * axes[1] + radii[2] * z * axes[2])
    verts = np.array(verts)
    faces = []
    rows = n // 2 + 1
    for i in range(rows - 1):
        for j in range(n):
            k = (j + 1) % n
            faces.append([i * n + j, i * n + k, (i + 1) * n + k, (i + 1) * n + j])
    return _weld(verts, faces)


def horn(path, radii, sides=16, flat=1.0, ref=(0, 1, 0), tip=0.0):
    """A horn, claw or spike: a tube along ``path`` whose radius runs
    through ``radii`` (eased), ``flat`` its section's depth over its width,
    closed to a point at the end when ``tip`` is zero."""
    rs = list(radii)
    st = [{"t": i / (len(rs) - 1), "w": r, "top": r * flat, "bot": r * flat} for i, r in enumerate(rs)]
    st[-1]["w"] = max(st[-1]["w"], tip)
    return tube(path, st, ref=ref, sides=sides, rings=max(10, 6 * len(path)))


def _weld(verts, faces, eps=1e-7):
    key = np.round(np.asarray(verts) / max(eps, 1e-9)).astype(np.int64) if eps > 1e-6 else None
    if key is None:
        q = np.round(np.asarray(verts) * 1e6).astype(np.int64)
    else:
        q = key
    _, first, inv = np.unique(q, axis=0, return_index=True, return_inverse=True)
    inv = inv.ravel()
    v = np.asarray(verts)[first]
    out = []
    for f in faces:
        g = [int(inv[i]) for i in f]
        dedup = [x for i, x in enumerate(g) if x != g[i - 1]]
        if len(set(dedup)) >= 3:
            out.append(dedup)
    return v, out


def mirror(form):
    """A form on the other side: ``z`` negated, faces turned so they still
    face out."""
    v, f = form
    v = np.array(v, float)
    v[:, 2] *= -1
    return v, [list(reversed(x)) for x in f]


def joined(forms):
    verts, faces, off = [], [], 0
    for v, f in forms:
        verts.append(np.asarray(v, float))
        faces.extend([[i + off for i in x] for x in f])
        off += len(v)
    return np.concatenate(verts), faces


# ---------------------------------------------------------------------------
# Blender round trip
# ---------------------------------------------------------------------------


def to_blender(v):
    v = np.asarray(v, float)
    return np.stack([v[:, 0], -v[:, 2], v[:, 1]], 1)


def from_blender(v):
    v = np.asarray(v, float)
    return np.stack([v[:, 0], v[:, 2], -v[:, 1]], 1)


def obj(name, form, smooth=True):
    v, f = form
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(map(float, p)) for p in to_blender(v)], [], [tuple(int(i) for i in x) for x in f])
    me.validate()
    if smooth:
        me.shade_smooth()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def read(ob):
    """An object's evaluated mesh, triangulated, as (verts, tris) in the
    game's frame."""
    dg = bpy.context.evaluated_depsgraph_get()
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    import bmesh as bm_mod

    bm = bm_mod.new()
    bm.from_mesh(me)
    bm_mod.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()
    v = np.zeros(len(me.vertices) * 3)
    me.vertices.foreach_get("co", v)
    t = np.zeros(len(me.polygons) * 3, dtype=np.int64)
    me.polygons.foreach_get("vertices", t)
    bpy.data.meshes.remove(me)
    return from_blender(v.reshape(-1, 3)), t.reshape(-1, 3)


def flesh(forms, voxel=0.03, smooth=6, name="flesh"):
    """Every soft form unioned into one watertight skin: a voxel remesh
    (which is what makes the union), then a Taubin smoothing that rounds the
    creases where two forms met without shrinking the whole."""
    ob = obj(name, joined(forms))
    m = ob.modifiers.new("remesh", "REMESH")
    m.mode = "VOXEL"
    m.voxel_size = voxel
    m.adaptivity = 0.0
    v, t = read(ob)
    bpy.data.objects.remove(ob)
    v = taubin(v, t, smooth)
    return v, t


def decimate(form, ratio, name="dec"):
    ob = obj(name, form)
    m = ob.modifiers.new("dec", "DECIMATE")
    m.ratio = ratio
    m.use_collapse_triangulate = True
    v, t = read(ob)
    bpy.data.objects.remove(ob)
    return v, t


# ---------------------------------------------------------------------------
# Mesh arithmetic
# ---------------------------------------------------------------------------


def edges_of(t):
    e = np.concatenate([t[:, [0, 1]], t[:, [1, 2]], t[:, [2, 0]]])
    e = np.sort(e, axis=1)
    return np.unique(e, axis=0)


def neighbour_mean(v, e, values=None):
    values = v if values is None else values
    n = len(v)
    acc = np.zeros((n,) + values.shape[1:])
    cnt = np.zeros(n)
    np.add.at(acc, e[:, 0], values[e[:, 1]])
    np.add.at(acc, e[:, 1], values[e[:, 0]])
    np.add.at(cnt, e[:, 0], 1)
    np.add.at(cnt, e[:, 1], 1)
    cnt = np.maximum(cnt, 1)
    return acc / cnt.reshape((-1,) + (1,) * (values.ndim - 1))


def taubin(v, t, iters, lam=0.5, mu=-0.53, keep=None):
    """Taubin's smoothing: a step in and a step back out, so it rounds
    without shrinking. ``keep``, a 0-to-1 weight per vertex, holds some in
    place."""
    e = edges_of(t)
    v = v.copy()
    k = 1.0 if keep is None else (1.0 - keep)[:, None]
    for _ in range(iters):
        v += lam * k * (neighbour_mean(v, e) - v)
        v += mu * k * (neighbour_mean(v, e) - v)
    return v


def normals(v, t):
    fn = np.cross(v[t[:, 1]] - v[t[:, 0]], v[t[:, 2]] - v[t[:, 0]])
    n = np.zeros_like(v)
    for k in range(3):
        np.add.at(n, t[:, k], fn)
    return n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-12)


def curvature(v, t, n=None):
    """Mean curvature's sign and size per vertex, roughly: how far a vertex
    stands out of its neighbours' mean, along its normal (positive on a
    ridge, negative in a crease), over the mean edge length."""
    n = normals(v, t) if n is None else n
    e = edges_of(t)
    m = neighbour_mean(v, e)
    el = np.mean(np.linalg.norm(v[e[:, 0]] - v[e[:, 1]], axis=1))
    return np.sum((v - m) * n, axis=1) / el


# ---------------------------------------------------------------------------
# Plates: armour cut from the skin it covers
# ---------------------------------------------------------------------------


def plate_on(v, t, n, inside, thick=0.08, dome=0.06, lift=0.015, keel=None, edge=0.35,
             rim=0.02, lip=None):
    """A plate over the faces of the skin ``(v, t)`` whose three corners
    are all ``inside`` (a bool per vertex): the patch lifted ``lift`` off
    the skin, its top a further ``thick`` plus a ``dome`` that rises from
    its edge to its middle, and ``keel(points)`` -- metres more along the
    normal per point, for a ridge along it -- where given; its edge bevelled
    over the outer ``edge`` of its width and ``lip(points)`` raising it
    where an overlapping plate's trailing edge stands proud."""
    faces = t[inside[t].all(1)]
    if len(faces) == 0:
        return None
    used = np.unique(faces)
    remap = -np.ones(len(v), dtype=np.int64)
    remap[used] = np.arange(len(used))
    f = remap[faces]
    p = v[used]
    nn = n[used]
    # Distance from the edge, in rings of the patch's own vertices.
    e = edges_of(f)
    boundary = _boundary_vertices(f, len(p))
    depth = np.full(len(p), np.inf)
    depth[boundary] = 0
    for _ in range(200):
        a, b = e[:, 0], e[:, 1]
        nd = np.minimum(depth[a] + 1, depth[b] + 1)
        changed = False
        for x, y in ((a, b), (b, a)):
            better = depth[x] + 1 < depth[y]
            if better.any():
                np.minimum.at(depth, y, depth[x] + 1)
                changed = True
        del nd
        if not changed:
            break
    depth[np.isinf(depth)] = 0
    s = depth / max(depth.max(), 1)
    bevel = np.clip(s / max(edge, 1e-6), 0, 1)
    bevel = np.sin(bevel * np.pi / 2)
    h = thick * (0.35 + 0.65 * bevel) + dome * np.sin(np.clip(s, 0, 1) * np.pi / 2)
    if keel is not None:
        h = h + keel(p) * bevel
    if lip is not None:
        h = h + lip(p)
    bottom = p + nn * lift
    top = p + nn * (lift + rim + h)[:, None]
    k = len(p)
    verts = np.vstack([top, bottom])
    tris = [tuple(x) for x in f] + [(c + k, b + k, a + k) for a, b, c in f]
    for a, b in _boundary_edges(f):
        tris.append((a, b + k, a + k))
        tris.append((a, b, b + k))
    return verts, np.array(tris)


def _boundary_edges(f):
    """The patch's edges used by one face only, in that face's winding."""
    count = {}
    order = {}
    for tri in f:
        for i in range(3):
            a, b = int(tri[i]), int(tri[(i + 1) % 3])
            key = (min(a, b), max(a, b))
            count[key] = count.get(key, 0) + 1
            order[key] = (a, b)
    return [order[k] for k, c in count.items() if c == 1]


def _boundary_vertices(f, n):
    out = np.zeros(n, bool)
    for a, b in _boundary_edges(f):
        out[a] = out[b] = True
    return out


# ---------------------------------------------------------------------------
# Weights
# ---------------------------------------------------------------------------


def segment_distance(p, a, b):
    ab = b - a
    u = np.clip(((p - a) @ ab) / max(ab @ ab, 1e-12), 0, 1)
    q = a + u[:, None] * ab
    return np.linalg.norm(p - q, axis=1)


def weights(v, t, bones, falloff=0.12, smooth=8):
    """A weight per bone per vertex: ``bones`` is ``(index, a, b, radius)``,
    the segment a bone carries and how thick the body round it is; nearer
    the segment's surface weighs more, then the weights are smoothed over
    the skin so a joint bends rather than tears."""
    count = max(b[0] for b in bones) + 1
    w = np.zeros((len(v), count))
    for i, a, b, r in bones:
        d = segment_distance(v, np.asarray(a, float), np.asarray(b, float)) - r
        w[:, i] = np.maximum(w[:, i], np.exp(-np.maximum(d, 0) / falloff - np.maximum(-d, 0) * 0.0))
    w /= np.maximum(w.sum(1, keepdims=True), 1e-12)
    if t is not None and smooth:
        e = edges_of(t)
        for _ in range(smooth):
            w = 0.5 * w + 0.5 * neighbour_mean(v, e, w)
    return top4(w)


def top4(w):
    idx = np.argsort(-w, axis=1)[:, :4]
    tw = np.take_along_axis(w, idx, axis=1)
    tw[tw < 0.02] = 0
    tw /= np.maximum(tw.sum(1, keepdims=True), 1e-12)
    out = np.zeros_like(w)
    np.put_along_axis(out, idx, tw, axis=1)
    return out


# ---------------------------------------------------------------------------
# Occlusion, by rays against the whole body
# ---------------------------------------------------------------------------


def occlusion(v, n, scene_v, scene_t, rays=24, reach=1.5, seed=3):
    """How open each vertex is to the sky round it, 0 to 1: rays over the
    hemisphere its normal faces, against every triangle of the body."""
    bvh = mathutils.bvhtree.BVHTree.FromPolygons(
        [mathutils.Vector(p) for p in scene_v], [tuple(int(i) for i in x) for x in scene_t])
    rng = np.random.default_rng(seed)
    d = rng.normal(size=(rays, 3))
    d /= np.linalg.norm(d, axis=1, keepdims=True)
    out = np.zeros(len(v))
    for i in range(len(v)):
        p, nn = v[i], n[i]
        open_ = 0.0
        total = 0.0
        for r in d:
            c = r @ nn
            if c < 0:
                r = -r
                c = -c
            o = mathutils.Vector(p + nn * 0.01)
            hit = bvh.ray_cast(o, mathutils.Vector(r), reach)
            total += c
            if hit[0] is None:
                open_ += c
        out[i] = open_ / max(total, 1e-9)
    return out


# ---------------------------------------------------------------------------
# The file
# ---------------------------------------------------------------------------


def write_glb(path, skeleton, prims, name):
    """``prims``: a list of dicts ``name``, ``pos`` (n,3), ``nrm`` (n,3),
    ``col`` (n,4 linear), ``part`` (n,), ``w`` (n,bones), ``idx`` (m,3).
    The joints are the species' bones, in its order, at their rest frames;
    a vertex's part is ``TEXCOORD_1.x``."""
    FLOAT, U16, U32, U8 = 5126, 5123, 5125, 5121
    blob = bytearray()
    views, accessors = [], []

    def pad(b):
        return b + b"\x00" * ((4 - len(b) % 4) % 4)

    def acc(arr, ctype, kind, target=None, minmax=False, normalized=False):
        a = np.ascontiguousarray(arr)
        off = len(blob)
        blob.extend(pad(a.tobytes()))
        view = {"buffer": 0, "byteOffset": off, "byteLength": a.nbytes}
        if target:
            view["target"] = target
        views.append(view)
        e = {"bufferView": len(views) - 1, "componentType": ctype, "count": len(a), "type": kind}
        if normalized:
            e["normalized"] = True
        if minmax:
            e["min"] = a.min(axis=0).tolist()
            e["max"] = a.max(axis=0).tolist()
        accessors.append(e)
        return len(accessors) - 1

    materials, primitives = [], []
    for mi, d in enumerate(prims):
        materials.append({"name": d["name"], "pbrMetallicRoughness": {
            "baseColorFactor": [1, 1, 1, 1], "metallicFactor": 0.0, "roughnessFactor": 0.85}})
        W = d["w"]
        top = np.argsort(-W, axis=1)[:, :4]
        tw = np.take_along_axis(W, top, axis=1)
        tw = tw / np.maximum(tw.sum(1, keepdims=True), 1e-9)
        joints = top.astype(np.uint8)
        joints[tw == 0] = 0
        # Weights in a byte each, summing to exactly 255 (the leftover on the
        # heaviest), as the cast's are; colours in two bytes a channel, which
        # a dark hide needs and a byte would band.
        qw = np.round(tw * 255).astype(np.int32)
        qw[:, 0] += 255 - qw.sum(axis=1)
        col = np.round(np.clip(d["col"], 0, 1) * 65535).astype(np.uint16)
        nrm = d["nrm"] / np.maximum(np.linalg.norm(d["nrm"], axis=1, keepdims=True), 1e-9)
        part = np.stack([d["part"], np.zeros(len(d["part"]))], 1).astype(np.float32)
        primitives.append({
            "attributes": {
                "POSITION": acc(d["pos"].astype(np.float32), FLOAT, "VEC3", 34962, True),
                "NORMAL": acc(nrm.astype(np.float32), FLOAT, "VEC3", 34962),
                "COLOR_0": acc(col, U16, "VEC4", 34962, normalized=True),
                "TEXCOORD_1": acc(part, FLOAT, "VEC2", 34962),
                "JOINTS_0": acc(joints, U8, "VEC4", 34962),
                "WEIGHTS_0": acc(qw.astype(np.uint8), U8, "VEC4", 34962, normalized=True),
            },
            "indices": acc(d["idx"].astype(np.uint32).ravel(), U32, "SCALAR", 34963),
            "material": mi,
        })

    nb = len(skeleton.bones)
    ibm = np.stack([np.linalg.inv(skeleton.frame(i)).T.astype(np.float32) for i in range(nb)])
    ibm_acc = acc(ibm.reshape(nb, 16), FLOAT, "MAT4")
    nodes = []
    for i, b in enumerate(skeleton.bones):
        m = skeleton.frame(i)
        q = _quat(m[:3, :3])
        nodes.append({"name": b["name"], "translation": list(map(float, b["at"])),
                      "rotation": list(map(float, q))})
    mesh_node = len(nodes)
    nodes.append({"name": name, "mesh": 0, "skin": 0})
    gltf = {
        "asset": {"version": "2.0", "generator": "art/lib/beast.py"},
        "scene": 0,
        "scenes": [{"nodes": list(range(nb)) + [mesh_node]}],
        "nodes": nodes,
        "meshes": [{"name": name, "primitives": primitives}],
        "materials": materials,
        "skins": [{"joints": list(range(nb)), "inverseBindMatrices": ibm_acc}],
        "buffers": [{"byteLength": len(blob)}],
        "bufferViews": views,
        "accessors": accessors,
    }
    js = pad(json.dumps(gltf, separators=(",", ":")).encode()).replace(b"\x00", b" ")
    js = js + b" " * ((4 - len(js) % 4) % 4)
    bin_ = pad(bytes(blob))
    total = 12 + 8 + len(js) + 8 + len(bin_)
    with open(path, "wb") as f:
        f.write(struct.pack("<III", 0x46546C67, 2, total))
        f.write(struct.pack("<II", len(js), 0x4E4F534A))
        f.write(js)
        f.write(struct.pack("<II", len(bin_), 0x004E4942))
        f.write(bin_)
    return path


def _quat(R):
    t = np.trace(R)
    if t > 0:
        s = np.sqrt(t + 1.0) * 2
        return np.array([(R[2, 1] - R[1, 2]) / s, (R[0, 2] - R[2, 0]) / s, (R[1, 0] - R[0, 1]) / s, 0.25 * s])
    i = int(np.argmax(np.diag(R)))
    j, k = (i + 1) % 3, (i + 2) % 3
    s = np.sqrt(1.0 + R[i, i] - R[j, j] - R[k, k]) * 2
    q = np.zeros(4)
    q[i] = 0.25 * s
    q[j] = (R[j, i] + R[i, j]) / s
    q[k] = (R[k, i] + R[i, k]) / s
    q[3] = (R[k, j] - R[j, k]) / s
    return q


# ---------------------------------------------------------------------------
# A studio for a creature
# ---------------------------------------------------------------------------


def studio(pieces, out, views=((90, 8), (35, 12), (0, 10), (145, 14)), size=(560, 400),
           distance=19.0, look=(0.0, 3.6, 0.0), samples=16, lens=50, clay=False):
    """Render ``pieces`` -- ``(verts, tris, colours)`` -- from each view,
    ``(yaw from the creature's left, degrees; camera height off the look
    point, metres)``, and lay them side by side into ``out``."""
    s = bpy.context.scene
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = samples
    s.cycles.use_denoising = True
    s.view_settings.view_transform = "AgX"
    s.render.resolution_x, s.render.resolution_y = size
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs[0].default_value = (0.55, 0.6, 0.68, 1)
    bg.inputs[1].default_value = 0.7
    s.world = world
    mat = bpy.data.materials.new("vc")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 0.75
    if not clay:
        a = nt.nodes.new("ShaderNodeVertexColor")
        a.layer_name = "Col"
        nt.links.new(a.outputs["Color"], bsdf.inputs["Base Color"])
    else:
        bsdf.inputs["Base Color"].default_value = (0.5, 0.48, 0.45, 1)
    for i, (v, t, c) in enumerate(pieces):
        ob = obj(f"piece{i}", (v, t))
        me = ob.data
        if c is not None:
            ca = me.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
            ca.data.foreach_set("color", np.asarray(c, np.float32).ravel())
        me.materials.append(mat)
    ground = obj("ground", (np.array([[-30, 0, -30], [30, 0, -30], [30, 0, 30], [-30, 0, 30]], float),
                            [[0, 3, 2, 1]]))
    gm = bpy.data.materials.new("g")
    gm.use_nodes = True
    gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.32, 0.30, 0.26, 1)
    ground.data.materials.append(gm)
    for name, energy, d, col in (("key", 3.5, (-0.5, -0.6, -1.0), (1.0, 0.93, 0.82)),
                                 ("rim", 2.0, (0.6, 0.8, -0.5), (0.65, 0.72, 1.0))):
        l = bpy.data.lights.new(name, "SUN")
        l.energy = energy
        l.color = col
        l.angle = 0.1
        o = bpy.data.objects.new(name, l)
        s.collection.objects.link(o)
        o.rotation_euler = mathutils.Vector(d).to_track_quat("-Z", "Y").to_euler()
    tiles = []
    base, _ = os.path.splitext(out)
    look_b = mathutils.Vector(to_blender(np.array([look]))[0])
    for k, view in enumerate(views):
        yaw, h = view[0], view[1]
        if len(view) > 2:
            look = view[2]
            look_b = mathutils.Vector(to_blender(np.array([look]))[0])
        cam = bpy.data.cameras.new("cam")
        cam.lens = lens
        o = bpy.data.objects.new("cam", cam)
        s.collection.objects.link(o)
        a = math.radians(yaw)
        # yaw 90 sees its left side; 0 its nose.
        eye = np.array([look[0] + distance * math.cos(a), look[1] + h, look[2] - distance * math.sin(a)])
        o.location = mathutils.Vector(to_blender(np.array([eye]))[0])
        o.rotation_euler = (look_b - o.location).to_track_quat("-Z", "Y").to_euler()
        s.camera = o
        path = f"{base}_{k}.png"
        s.render.filepath = path
        bpy.ops.render.render(write_still=True)
        bpy.data.objects.remove(o)
        tiles.append(path)
    rows = [tiles[i:i + 2] for i in range(0, len(tiles), 2)]
    row_files = []
    for r, row in enumerate(rows):
        rf = f"{base}_row{r}.png"
        subprocess.run(["convert", *row, "+append", rf], check=True)
        row_files.append(rf)
    subprocess.run(["convert", *row_files, "-append", out], check=True)
    for f in tiles + row_files:
        os.remove(f)
    return out


# ---------------------------------------------------------------------------
# Hard parts on a body
# ---------------------------------------------------------------------------


class Surface:
    """Rays against a body, in the game's frame."""

    def __init__(self, v, t):
        self.bvh = mathutils.bvhtree.BVHTree.FromPolygons(
            [mathutils.Vector(p) for p in v], [tuple(int(i) for i in x) for x in t])

    def cast(self, origin, direction, reach=50.0):
        """The first point a ray meets and the surface's normal there, or
        ``None``."""
        d = np.asarray(direction, float)
        hit = self.bvh.ray_cast(mathutils.Vector(origin), mathutils.Vector(d / np.linalg.norm(d)), reach)
        if hit[0] is None:
            return None
        n = np.array(hit[1])
        if n @ d > 0:
            n = -n
        return np.array(hit[0]), n

    def distance(self, points):
        """How far each point is from the body."""
        out = np.zeros(len(points))
        for i, p in enumerate(points):
            hit = self.bvh.find_nearest(mathutils.Vector(p))
            out[i] = hit[3] if hit[0] is not None else 0.0
        return out

    def down(self, x, z, top=12.0):
        """Where a plumb line at (x, z) meets the body from above."""
        return self.cast((x, top, z), (0, -1, 0))

    def toward(self, point, centre):
        """Where the line from outside ``point`` back toward ``centre`` meets
        the body: the surface over a point, seen from an axis."""
        p, c = np.asarray(point, float), np.asarray(centre, float)
        d = p - c
        d /= np.linalg.norm(d)
        return self.cast(c + d * 6.0, -d)


def osteoderm(base, up, along, length, height, thick, sweep=0.3, sink=0.18, sides=10, keel=0.35):
    """A plate standing out of a back: a lens in section, a leaf in side
    view, its tip swept back along ``-along`` by ``sweep`` of its height and
    a keel down its middle; its foot sunk ``sink`` into the skin so it grows
    out of it."""
    up = np.asarray(up, float) / np.linalg.norm(up)
    along = np.asarray(along, float)
    along = along - up * (along @ up)
    along /= np.linalg.norm(along)
    base = np.asarray(base, float)
    path = [base - up * sink,
            base + up * height * 0.45 - along * sweep * height * 0.2,
            base + up * height - along * sweep * height]
    h = length / 2
    stations = [
        {"t": 0.0, "w": thick, "top": h * 1.05, "bot": h * 1.05, "bumps": ((np.pi / 2, 0.35, keel), (-np.pi / 2, 0.35, keel))},
        {"t": 0.25, "w": thick * 0.95, "top": h, "bot": h, "bumps": ((np.pi / 2, 0.35, keel), (-np.pi / 2, 0.35, keel))},
        {"t": 0.65, "w": thick * 0.7, "top": h * 0.62, "bot": h * 0.7, "bumps": ((np.pi / 2, 0.35, keel), (-np.pi / 2, 0.35, keel))},
        {"t": 1.0, "w": thick * 0.15, "top": h * 0.06, "bot": h * 0.08, "bumps": ((np.pi / 2, 0.35, 0.0), (-np.pi / 2, 0.35, 0.0))},
    ]
    for s in stations:
        s["nt"] = s["nb"] = 1.6
    return tube(path, stations, ref=along, sides=sides, rings=14)


def nail(centre, out, size, sides=14):
    """A hoof-nail: a blunt rounded wedge pointing ``out`` and down."""
    out = np.asarray(out, float)
    out /= np.linalg.norm(out)
    up = np.array([0.0, 1.0, 0.0])
    across = np.cross(up, out)
    axes = np.stack([out, across, up])
    return blob(np.asarray(centre, float), (size[0], size[1], size[2]), axes=axes, n=sides, power=2.6)


def rounded(u, v, r):
    """The unit square drawn toward a disc by ``r`` (0 a square, 1 a
    circle), so a plate's outline is a rounded shape rather than a tile."""
    x, y = u * 2 - 1, v * 2 - 1
    cx = x * np.sqrt(max(1 - y * y / 2, 0))
    cy = y * np.sqrt(max(1 - x * x / 2, 0))
    x, y = x + (cx - x) * r, y + (cy - y) * r
    return (x + 1) / 2, (y + 1) / 2


def shell(surf, ray, nu, nv, thick=0.08, lift=0.02, bevel=0.06, lip=0.0, crown=0.0, rough=0.0, seed=0,
          round_=0.0, keel=0.0):
    """A hard plate laid on a body: ``ray(u, v)``, for ``u`` and ``v`` 0 to
    1 across the plate, gives a ray's origin and direction toward the skin;
    the grid of hits, lifted ``lift`` off it, is the plate's underside and
    the same pushed out ``thick`` its top -- flat, chamfered to a quarter of
    that over ``bevel`` metres from its edge, raised ``lip`` along its back
    edge (``u`` = 0) so a row of them overlaps, and ``crown`` along its
    middle. ``rough`` nicks its edge, so a row of them is not a row of
    copies. Returns ``None`` if a ray misses."""
    rng = np.random.default_rng(seed)
    us = np.linspace(0, 1, nu)
    vs = np.linspace(0, 1, nv)
    pts = np.zeros((nu, nv, 3))
    nrm = np.zeros((nu, nv, 3))
    for i, u in enumerate(us):
        for j, v in enumerate(vs):
            o, d = ray(*rounded(u, v, round_))
            hit = surf.cast(o, d)
            if hit is None:
                return None
            pts[i, j], nrm[i, j] = hit
    # Smooth the normals over the grid, so the plate is one curved face
    # rather than a print of every voxel under it.
    for _ in range(3):
        m = nrm.copy()
        m[1:-1] = (nrm[:-2] + nrm[1:-1] * 2 + nrm[2:]) / 4
        m[:, 1:-1] = (m[:, :-2] + m[:, 1:-1] * 2 + m[:, 2:]) / 4
        nrm = m / np.linalg.norm(m, axis=2, keepdims=True)
    lu = np.linalg.norm(np.diff(pts, axis=0), axis=2).sum(0).mean()
    lv = np.linalg.norm(np.diff(pts, axis=1), axis=2).sum(1).mean()
    U, V = np.meshgrid(us, vs, indexing="ij")
    edge = np.minimum.reduce([U * lu, (1 - U) * lu, V * lv, (1 - V) * lv])
    if rough:
        edge = edge - rough * rng.random(edge.shape) * (edge < bevel * 1.5)
    # A cut chamfer, not a roll: straight from 45% at the edge to the full
    # height at ``bevel`` in, where it meets the flat top in a crease.
    h = thick * np.clip(0.45 + 0.55 * edge / max(bevel, 1e-6), 0.45, 1.0)
    h = h + lip * np.clip(1 - U / 0.35, 0, 1) ** 1.5 * np.clip(edge / max(bevel, 1e-6), 0, 1)
    h = h + crown * np.sin(np.pi * U) * np.sin(np.pi * V)
    # A keel down its length, as on an osteoderm.
    h = h + keel * np.exp(-((V - 0.5) / 0.13) ** 2) * np.sin(np.pi * np.clip(U * 0.85 + 0.1, 0, 1)) * \
        np.clip(edge / max(bevel, 1e-6), 0, 1)
    bottom = pts + nrm * lift
    top = pts + nrm * (lift + h)[..., None]
    k = nu * nv
    verts = np.concatenate([top.reshape(-1, 3), bottom.reshape(-1, 3)])

    def at(i, j):
        return i * nv + j

    faces = []
    for i in range(nu - 1):
        for j in range(nv - 1):
            a, b_, c, d = at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)
            faces.append([a, b_, c, d])
            faces.append([d + k, c + k, b_ + k, a + k])
    ring = [at(i, 0) for i in range(nu)] + [at(nu - 1, j) for j in range(1, nv)] + \
        [at(i, nv - 1) for i in range(nu - 2, -1, -1)] + [at(0, j) for j in range(nv - 2, 0, -1)]
    for a, c in zip(ring, ring[1:] + ring[:1]):
        faces.append([c, a, a + k, c + k])
    # Face the top outward: if the first quad's normal points into the
    # body, turn every face over.
    f0 = faces[0]
    fn = np.cross(verts[f0[1]] - verts[f0[0]], verts[f0[2]] - verts[f0[0]])
    if fn @ nrm[0, 0] < 0:
        faces = [list(reversed(f)) for f in faces]
    # How worn each vertex is: the chamfer and the rim, where a plate is
    # knocked; the underside counts as its rim.
    wear = np.concatenate([np.clip(1 - edge / max(bevel, 1e-6), 0, 1).ravel(), np.ones(k)])
    return verts, faces, wear


def triangles(faces):
    """Quads and fans to triangles."""
    return np.array([[f[0], f[i], f[i + 1]] for f in faces for i in range(1, len(f) - 1)], dtype=np.int64)


def hard_normals(v, t, angle=35.0, attrs=()):
    """Split the mesh where its faces meet at more than ``angle`` degrees,
    so a plate's chamfer and sides read as cut edges under smooth shading:
    every corner takes the mean normal of the faces round its vertex that
    are within the angle of its own face, and corners that differ become
    vertices of their own. ``attrs``: per-vertex arrays carried along.
    Returns (v, t, normals, attrs)."""
    v = np.asarray(v, float)
    t = np.asarray(t, np.int64)
    fn = np.cross(v[t[:, 1]] - v[t[:, 0]], v[t[:, 2]] - v[t[:, 0]])
    area = np.linalg.norm(fn, axis=1, keepdims=True)
    fu = fn / np.maximum(area, 1e-12)
    cos = np.cos(np.radians(angle))
    faces_of = [[] for _ in range(len(v))]
    for f, tri in enumerate(t):
        for k in tri:
            faces_of[k].append(f)
    out_v, out_n, out_a, out_t = [], [], [[] for _ in attrs], np.zeros_like(t)
    key = {}
    for f, tri in enumerate(t):
        for c, k in enumerate(tri):
            near = [g for g in faces_of[k] if fu[g] @ fu[f] >= cos]
            n = (fn[near]).sum(0)
            n /= max(np.linalg.norm(n), 1e-12)
            q = (int(k), tuple(np.round(n * 64).astype(int)))
            if q not in key:
                key[q] = len(out_v)
                out_v.append(v[k])
                out_n.append(n)
                for a, arr in zip(out_a, attrs):
                    a.append(arr[k])
            out_t[f, c] = key[q]
    return (np.array(out_v), out_t, np.array(out_n), [np.array(a) for a in out_a])


def noise3(p, scale, seed=0.0):
    """A cheap smooth value noise, 0 to 1, from summed sines."""
    p = np.asarray(p, float) * scale
    a = np.sin(p[:, 0] * 1.7 + seed) * np.sin(p[:, 1] * 2.3 + 1.3 * seed) * np.sin(p[:, 2] * 1.9 + 0.7)
    b_ = np.sin(p[:, 0] * 3.1 + p[:, 2] * 1.3 + seed * 2) * np.sin(p[:, 1] * 3.7 - p[:, 0] * 0.9)
    c = np.sin((p[:, 0] + p[:, 1] * 0.6 - p[:, 2] * 0.8) * 5.3 + seed * 3)
    return np.clip(0.5 + 0.35 * a + 0.2 * b_ + 0.1 * c, 0, 1)


def nearest(src, dst):
    """For each point of ``dst``, the index of the nearest point of ``src``."""
    kd = mathutils.kdtree.KDTree(len(src))
    for i, p in enumerate(src):
        kd.insert(mathutils.Vector(p), i)
    kd.balance()
    return np.array([kd.find(mathutils.Vector(p))[1] for p in dst])


def box_distance(p, lo, hi):
    d = np.maximum(np.maximum(lo - p, p - hi), 0)
    return np.linalg.norm(d, axis=1)


def srgb_to_linear(c):
    c = np.asarray(c, float)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
