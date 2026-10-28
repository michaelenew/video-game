"""A creature, sculpted and baked: the pipeline the Ridgeback was made by
(``art/ridgeback/build.py``), as one builder a species is a spec on.

    art/blender.sh art/<species>/build.py [--look out.png] [--size 2048] [--no-export] [--fast]

A spec is a module with:

- ``NAME``, and ``SCALE`` -- about how big a thing it is, in metres, against
  the Ridgeback's 1.0: the voxel sizes and the bake's cage follow it;
- ``forms()``: the soft masses, as ``beast`` forms, unioned into the skin;
- ``shells(v, t)``: armour grown off the coarse skin, closed, or ``None``;
- ``sculpt(v, t, kind)``: the detail and the paint on the dense union
  (``kind`` 0 on the skin, 1 on a shell), returning the moved vertices and
  per-vertex albedo (sRGB), glow (linear) and roughness;
- ``hard(surf)``: the hard pieces -- horns, claws, feathers, blades -- as
  ``((verts, faces), kind)``, each bound wholly to the skin at its root;
- ``paint_hard(pieces)``: their colour, glow and roughness;
- ``bones(sk)``: the segments the skin is weighted to, ``(bone, a, b, radius)``;
- ``VIEWS``: ``dict(views=..., distance=..., look=..., near=..., near_distance=...)``
  for the studio sheets.

Every number is metres in the game's frame (``+x`` to the head, ``+y`` up,
``+z`` its right), at rest.
"""

import os
import sys
import time

import numpy as np

import beast as b

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
T0 = time.time()


def say(*a):
    print(f"[{time.time() - T0:6.1f}s]", *a, flush=True)


# ---------------------------------------------------------------------------
# Small things every spec writes with
# ---------------------------------------------------------------------------


def st(t, w, top, bot, nt=2.0, nb=2.0, bumps=(), dy=0.0, dz=0.0):
    """A tube's section at ``t`` along it: half-width, half-height above and
    below, how square above and below, and bumps round it."""
    return {"t": t, "w": w, "top": top, "bot": bot, "nt": nt, "nb": nb, "bumps": bumps, "dy": dy, "dz": dz}


def smooth(a, c, x):
    t = np.clip((np.asarray(x, float) - a) / (c - a), 0, 1)
    return t * t * (3 - 2 * t)


def mixc(a, c, f):
    f = np.clip(np.asarray(f, float), 0, 1)[:, None]
    return np.asarray(a, float) * (1 - f) + np.asarray(c, float) * f


def muscled(rows, muscles, nt=2.0, nb=2.0):
    """Sections from ``(t, w, top, bot)`` rows, with each muscle -- ``(angle,
    width, heights per row)`` -- a bump round them."""
    out = []
    for i, (t, w, top, bot) in enumerate(rows):
        bumps = tuple((a, wd, hs[i]) for a, wd, hs in muscles)
        out.append(st(t, w, top, bot, nt=nt, nb=nb, bumps=bumps))
    return out


def both(forms):
    """Left forms and their mirrors."""
    return list(forms) + [b.mirror(f) for f in forms]


class Axis:
    """A curve, sampled densely, with frames: for laying things round it."""

    def __init__(self, path, ref=(0, 1, 0), n=400):
        self.c = b.spline(path, n)
        self.tg, self.side, self.up = b.frames(self.c, ref)
        self.n = n
        self.length = np.linalg.norm(np.diff(self.c, axis=0), axis=1).sum()

    def coords(self, p, chunk=200000):
        """For each point: how far along (0 to 1), the angle round from
        ``up`` toward ``side``, and the distance off the curve."""
        s = np.zeros(len(p))
        phi = np.zeros(len(p))
        r = np.zeros(len(p))
        for a in range(0, len(p), chunk):
            q = p[a:a + chunk]
            d2 = ((q[:, None, :] - self.c[None, ::4, :]) ** 2).sum(-1)
            i = np.argmin(d2, axis=1) * 4
            best = i.copy()
            bd = np.full(len(q), np.inf)
            for k in range(-4, 5):
                j = np.clip(i + k, 0, self.n - 1)
                dd = ((q - self.c[j]) ** 2).sum(-1)
                better = dd < bd
                best[better], bd[better] = j[better], dd[better]
            d = q - self.c[best]
            s[a:a + chunk] = best / (self.n - 1)
            phi[a:a + chunk] = np.arctan2((d * self.side[best]).sum(-1), (d * self.up[best]).sum(-1))
            r[a:a + chunk] = np.sqrt(bd)
        return s, phi, r


def subdivide(v, t):
    """Every triangle into four, at its edges' midpoints."""
    e = np.sort(np.concatenate([t[:, [0, 1]], t[:, [1, 2]], t[:, [2, 0]]]), axis=1)
    uniq, inv = np.unique(e, axis=0, return_inverse=True)
    inv = inv.ravel()
    mid = (v[uniq[:, 0]] + v[uniq[:, 1]]) / 2
    nv = len(v)
    m = inv.reshape(3, -1).T + nv
    a, b_, c = t[:, 0], t[:, 1], t[:, 2]
    m01, m12, m20 = m[:, 0], m[:, 1], m[:, 2]
    nt = np.concatenate([
        np.stack([a, m01, m20], 1), np.stack([m01, b_, m12], 1),
        np.stack([m20, m12, c], 1), np.stack([m01, m12, m20], 1)])
    return np.concatenate([v, mid]), nt


def shell(v, t, inside_of, height, splits=2, under=0.08):
    """Armour as a closed shell grown off the skin ``(v, t)``: wherever
    ``inside_of(points)`` (metres inside the armour's edge) is positive the
    skin is lifted by ``height(points, normals, inside)``, and closed
    ``under`` metres beneath, so a voxel union reads it as one solid with the
    skin. The patch is split ``splits`` times first, so the shell's own
    detail -- grooves, rims -- has vertices to stand on."""
    n = b.normals(v, t)
    inside = inside_of(v)
    keep = inside > -0.06
    faces = t[keep[t].all(1)]
    if len(faces) == 0:
        return None
    used = np.unique(faces)
    remap = -np.ones(len(v), np.int64)
    remap[used] = np.arange(len(used))
    pv, pn, pt = v[used], n[used], remap[faces]
    for _ in range(splits):
        k = len(pv)
        pv, pt = subdivide(pv, pt)
        e = np.sort(np.concatenate([pt[:, [0, 1]], pt[:, [1, 2]], pt[:, [2, 0]]]), axis=1)
        e = np.unique(e, axis=0)
        pn = np.concatenate([pn, np.zeros((len(pv) - k, 3))])
        acc = np.zeros_like(pn)
        cnt = np.zeros(len(pv))
        old = e[(e[:, 0] < k) & (e[:, 1] >= k)]
        np.add.at(acc, old[:, 1], pn[old[:, 0]])
        np.add.at(cnt, old[:, 1], 1)
        pn[k:] = acc[k:] / np.maximum(cnt[k:, None], 1)
        pn /= np.maximum(np.linalg.norm(pn, axis=1, keepdims=True), 1e-9)
    inside = inside_of(pv)
    ok = inside > 0
    faces = pt[ok[pt].all(1)]
    used = np.unique(faces)
    remap = -np.ones(len(pv), np.int64)
    remap[used] = np.arange(len(used))
    pv, pn, pt, inside = pv[used], pn[used], remap[faces], inside[used]
    h = height(pv, pn, inside)
    top = pv + pn * h[:, None]
    bottom = pv - pn * under
    k = len(pv)
    verts = np.concatenate([top, bottom])
    tris = [pt, pt[:, ::-1] + k]
    for a, c in b._boundary_edges(pt):
        tris.append(np.array([[a, c, c + k], [a, c + k, a + k]]))
    return verts, np.concatenate(tris)


def ringed_horn(path, radii, sides=None, rings=None, ridges=14, depth=0.05, flat=1.0, twist=0.0, ref=None):
    """A horn, claw or spine with growth ridges: a tube whose radius runs
    through ``radii`` with ``ridges`` bands round it, each a little proud of
    the next. ``flat`` squashes its section along ``ref``."""
    path = np.asarray(path, float)
    length = np.linalg.norm(np.diff(path, axis=0), axis=1).sum()
    rings = rings or int(np.clip(length / 0.045, 10, 48))
    sides = sides or int(np.clip(max(radii) / 0.012, 8, 18))
    curve = b.spline(path, rings)
    if ref is None:
        mean = curve[-1] - curve[0]
        mean = mean / max(np.linalg.norm(mean), 1e-9)
        axes = np.eye(3)
        ref = axes[np.argmin(np.abs(axes @ mean))]
        if abs(mean[1]) < 0.7:
            ref = np.array([0.0, 1.0, 0.0])
    tg, side, up = b.frames(curve, ref)
    m = len(curve)
    s = np.linspace(0, 1, m)
    rr = np.interp(s, np.linspace(0, 1, len(radii)), radii)
    band = 1 + depth * (np.abs(np.sin(s * np.pi * ridges)) ** 0.5 - 0.5) * smooth(0.0, 0.1, s) * smooth(1.0, 0.75, s)
    theta = np.linspace(0, 2 * np.pi, sides, endpoint=False)
    verts = []
    for i in range(m):
        th = theta + twist * s[i]
        ring = curve[i] + rr[i] * band[i] * (np.outer(np.cos(th), up[i]) * flat + np.outer(np.sin(th), side[i]))
        verts.append(ring)
    verts = np.concatenate(verts)
    faces = []
    for i in range(m - 1):
        a, c = i * sides, (i + 1) * sides
        for j in range(sides):
            k = (j + 1) % sides
            faces.append([a + j, c + j, c + k, a + k])
    c0 = len(verts)
    verts = np.vstack([verts, curve[0], curve[-1] + tg[-1] * 0.001])
    last = (m - 1) * sides
    for j in range(sides):
        k = (j + 1) % sides
        faces.append([c0, j, k])
        faces.append([c0 + 1, last + k, last + j])
    return verts, faces


def vane(path, widths, normal, thick=0.012, lean=0.35, rings=None, notch=0.0, seed=0):
    """A flat blade along ``path`` -- a feather, a scythe, a wing's vein
    panel -- ``widths[i]`` across at each station, lying in the plane whose
    normal is ``normal`` (a vector, or one per station), ``thick`` through,
    the shaft ``lean`` of the way from its leading edge (0 in the middle, 1
    at the edge). ``notch`` cuts shallow nicks into its trailing edge, as a
    worn flight feather has. A closed slab, so it bakes from both sides."""
    path = np.asarray(path, float)
    length = np.linalg.norm(np.diff(path, axis=0), axis=1).sum()
    rings = rings or int(np.clip(length / 0.05, 8, 60))
    c = b.spline(path, rings)
    tg = np.gradient(c, axis=0)
    tg /= np.maximum(np.linalg.norm(tg, axis=1, keepdims=True), 1e-9)
    nrm = np.asarray(normal, float)
    if nrm.ndim == 1:
        nrm = np.tile(nrm, (rings, 1))
    else:
        idx = np.linspace(0, len(nrm) - 1, rings)
        nrm = np.array([nrm[int(round(i))] for i in idx])
    side = np.cross(tg, nrm)
    side /= np.maximum(np.linalg.norm(side, axis=1, keepdims=True), 1e-9)
    up = np.cross(side, tg)
    s = np.linspace(0, 1, rings)
    w = np.interp(s, np.linspace(0, 1, len(widths)), widths)
    rng = np.random.default_rng(seed)
    nicks = np.ones(rings)
    if notch > 0:
        for _ in range(3):
            at = rng.uniform(0.25, 0.9)
            nicks *= 1 - notch * np.exp(-((s - at) / 0.025) ** 2)
    # Across the vane: from the leading edge (-1) to the trailing (+1).
    across = np.linspace(-1, 1, 7)
    verts = []
    for i in range(rings):
        lead = w[i] * (1 - lean)
        trail = w[i] * (1 + lean) * nicks[i]
        for sgn in (1, -1):
            for a in across:
                off = (a * lead if a < 0 else a * trail)
                # A shallow camber, thickest at the shaft.
                h = thick * (1 - a * a * 0.8) * sgn + thick * 0.6 * (1 - a * a)
                verts.append(c[i] + side[i] * off + up[i] * h)
    verts = np.array(verts)
    k = len(across)
    per = 2 * k
    faces = []
    for i in range(rings - 1):
        a0, a1 = i * per, (i + 1) * per
        for j in range(k - 1):
            faces.append([a0 + j, a1 + j, a1 + j + 1, a0 + j + 1])
            faces.append([a0 + k + j + 1, a1 + k + j + 1, a1 + k + j, a0 + k + j])
        # the two edges
        faces.append([a0 + k, a1 + k, a1, a0])
        faces.append([a0 + k - 1, a1 + k - 1, a1 + 2 * k - 1, a0 + 2 * k - 1])
    # caps
    last = (rings - 1) * per
    faces.append(list(range(k)) + list(range(2 * k - 1, k - 1, -1)))
    faces.append([last + x for x in (list(range(2 * k - 1, k - 1, -1)) + list(range(k)))][::-1])
    return verts, faces


def across_of(count):
    """For a ``vane``'s vertices: where each is across it, from its leading
    edge (-1) through its shaft (0) to its trailing edge (1)."""
    row = np.concatenate([np.linspace(-1, 1, 7)] * 2)
    return np.resize(row, count)


def along(v):
    """For a hard piece's vertices: 0 at its first ring, 1 at its farthest."""
    v = np.asarray(v)
    d = np.linalg.norm(v - v[0], axis=1)
    return d / max(d.max(), 1e-6)


# ---------------------------------------------------------------------------
# The build
# ---------------------------------------------------------------------------


def make(spec):
    args = sys.argv[1:]
    fast = "--fast" in args
    name = spec.NAME
    scale = getattr(spec, "SCALE", 1.0)
    size = int(args[args.index("--size") + 1]) if "--size" in args else (1024 if fast else 2048)
    look = args[args.index("--look") + 1] if "--look" in args else os.path.join(ROOT, f"target/art/{name}.png")
    work = os.path.join(ROOT, f"target/art/{name}")
    out = os.path.join(ROOT, f"crates/game/assets/{name}.glb")
    os.makedirs(work, exist_ok=True)
    import bpy

    bpy.ops.wm.read_factory_settings(use_empty=True)
    sk = b.Skeleton(os.path.join(ROOT, f"target/art/{name}.json"))

    say("forms")
    cv, ct = b.flesh(spec.forms(), voxel=0.03 * scale, smooth=14)
    surf = b.Surface(cv, ct)
    say(f"coarse skin {len(cv)} verts")
    sh = spec.shells(cv, ct)
    forms = [(cv, ct)] + ([sh] if sh is not None else [])

    say("union")
    hi_voxel = (0.02 if fast else 0.012) * scale
    ob = b.obj("sculpt", b.joined(forms), smooth=False)
    me = ob.data
    kind = np.concatenate([np.zeros(len(cv))] + ([np.ones(len(sh[0]))] if sh is not None else []))
    a = me.attributes.new("kind", "FLOAT", "POINT")
    a.data.foreach_set("value", kind)
    me.remesh_voxel_size = hi_voxel
    me.use_remesh_preserve_attributes = True
    with bpy.context.temp_override(active_object=ob, object=ob, selected_objects=[ob]):
        bpy.ops.object.voxel_remesh()
    hv = np.zeros(len(me.vertices) * 3)
    me.vertices.foreach_get("co", hv)
    hv = b.from_blender(hv.reshape(-1, 3))
    kind = np.zeros(len(me.vertices))
    me.attributes["kind"].data.foreach_get("value", kind)
    _, ht = b.read(ob)
    bpy.data.objects.remove(ob)
    say(f"sculpt {len(hv)} verts, {len(ht)} tris")
    hv = b.taubin(hv, ht, 2)

    lv, lt = b.decimate((hv, ht), getattr(spec, "TRIS", 60000) / len(ht))
    say(f"game mesh {len(lv)} verts, {len(lt)} tris")

    hv, albedo, glow, rough = spec.sculpt(hv, ht, kind)
    pieces = spec.hard(surf)
    hcols, hglows, hroughs = spec.paint_hard(pieces)
    if pieces:
        hard_v, hard_f = b.joined([p[0] for p in pieces])
        hard_t = b.triangles(hard_f)
    else:
        hard_v, hard_t = np.zeros((0, 3)), np.zeros((0, 3), np.int64)
    say(f"{len(pieces)} hard pieces, {len(hard_t)} tris")

    high_v = np.concatenate([hv, hard_v])
    high_t = np.concatenate([ht, hard_t + len(hv)])
    high_c = np.concatenate([albedo] + hcols)
    high_g = np.concatenate([glow] + hglows)
    high_r = np.concatenate([rough] + hroughs)
    low_v = np.concatenate([lv, hard_v])
    low_t = np.concatenate([lt, hard_t + len(lv)])

    high = b.obj("high", (high_v, high_t))
    for nm, vals in (("albedo", b.srgb_to_linear(np.clip(high_c, 0, 1))), ("glow", np.clip(high_g, 0, 4)),
                     ("rough", np.stack([high_r] * 3, 1))):
        ca = high.data.color_attributes.new(nm, "FLOAT_COLOR", "POINT")
        ca.data.foreach_set("color", np.concatenate([vals, np.ones((len(vals), 1))], 1).astype(np.float32).ravel())
    low = b.obj("low", (low_v, low_t))
    say("unwrap")
    extrude = np.concatenate([np.full(len(lv), 0.06 * scale), np.full(len(hard_v), 0.006 * scale)])
    tex = b.bake_textures(low, high, size, work, fast=fast, extrude=extrude)
    say("baked")

    gv, gn, guv, gtan, gt, src = b.gather_low(low)
    w_skin = b.weights(lv, lt, spec.bones(sk), falloff=0.15 * scale, smooth=10)
    w_low = np.zeros((len(low_v), w_skin.shape[1]))
    w_low[:len(lv)] = w_skin
    if pieces:
        near = b.nearest(lv, hard_v)
        off = len(lv)
        for (v, f), kd in pieces:
            k = len(v)
            rootv = int(np.argmin(surf.distance(np.asarray(v[: min(k, 64)]))))
            w_low[off:off + k] = w_skin[near[off - len(lv) + rootv]]
            off += k
    weights = w_low[src]
    part = parts_of(sk, gv)
    say(f"export: {len(gv)} verts, {len(gt)} tris")
    if "--no-export" not in args:
        b.write_beast_glb(out, sk, gv, gn, guv, gtan, gt, weights, part, tex, name)
        say(f"wrote {out}: {os.path.getsize(out) / 1e6:.1f} MB")
    b.studio_textured(low, tex, look, **getattr(spec, "VIEWS", {}))
    say(f"wrote {look}")


def parts_of(sk, points):
    """The nearest of the creature's parts to each point."""
    best = np.full(len(points), np.inf)
    out = np.zeros(len(points), dtype=np.int64)
    for k in range(len(sk.parts)):
        lo, hi = sk.part_box(k)
        d = b.box_distance(points, lo, hi)
        better = d < best
        out[better], best[better] = k, d[better]
    return out
