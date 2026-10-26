"""Hard surfaces: what makes a cut-out shell read as forged metal.

A garment cut from the inflated body (``wardrobe.piece``) is a good *shape*
for a plate -- it fits -- and a poor *object*: its edge is a cut, its surface
the body's own, and in the game it reads as clay (the owner, 2026-10-26,
"metal bands that look like metal"). What an armourer, and a game's hard-
surface artist, adds is in this file:

- ``rolled_edge``: a bead swept along every open edge of a plate -- the
  rolled, raised rim real plate has at each border, and the single strongest
  cue that a shape is armour (the field armour of Henry VIII; the Greenwich
  armours);
- ``rivets``: domed heads along a line;
- ``band``: a forged band round a limb -- a ring of metal that follows the
  limb's own cross-section a gap out, square-edged with its corners bevelled,
  a raised rim at each edge.

Every function works in the game's frame (``+y`` up, ``+z`` forward) and
returns Blender objects in the character's material, bound like any garment.
"""

import bmesh
import bpy
import numpy as np

import garment as g
import stage
from wardrobe import mat, rigid


def boundary_loops(ob):
    """The open edges of a mesh, as ordered loops of points in the game's
    frame. A thickened shell has two loops per edge (inside and out); only
    the outer is wanted, so loops are found before thickening, or the
    caller keeps the outer one (``outermost``)."""
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bm.edges.ensure_lookup_table()
    edges = [e for e in bm.edges if e.is_boundary]
    nxt = {}
    for e in edges:
        a, b = e.verts[0].index, e.verts[1].index
        nxt.setdefault(a, []).append(b)
        nxt.setdefault(b, []).append(a)
    co = {v.index: np.array(v.co) for v in bm.verts}
    bm.free()
    seen = set()
    loops = []
    for start in nxt:
        if start in seen:
            continue
        loop = [start]
        seen.add(start)
        prev, cur = None, start
        while True:
            options = [n for n in nxt[cur] if n != prev and n not in seen]
            if not options:
                break
            prev, cur = cur, options[0]
            loop.append(cur)
            seen.add(cur)
        if len(loop) > 6:
            loops.append(g.b2g(np.array([co[i] for i in loop])))
    return loops


def _resample(pts, step, closed):
    """A polyline at even spacing, smoothed a little: a hem's raw edge is as
    ragged as the shell's voxels."""
    if closed:
        pts = np.vstack([pts, pts[:1]])
    seg = np.linalg.norm(np.diff(pts, axis=0), axis=1)
    s = np.concatenate([[0.0], np.cumsum(seg)])
    n = max(8, int(s[-1] / step))
    t = np.linspace(0, s[-1], n, endpoint=not closed)
    out = np.stack([np.interp(t, s, pts[:, k]) for k in range(3)], -1)
    for _ in range(3):
        if closed:
            out = 0.5 * out + 0.25 * (np.roll(out, 1, 0) + np.roll(out, -1, 0))
        else:
            out[1:-1] = 0.5 * out[1:-1] + 0.25 * (out[:-2] + out[2:])
    return out


def tube(name, pts, radius, material, closed=True, sides=8, flatten=1.0, step=0.006):
    """A tube swept along ``pts`` (game frame): a rolled edge, a wire, a
    chain's cord. ``flatten`` below one squashes it toward the surface it
    lies on (a bead rather than a pipe)."""
    p = _resample(np.asarray(pts, float), step, closed)
    n = len(p)
    rings = []
    centre = p.mean(axis=0)
    for i in range(n):
        a, b = p[(i - 1) % n] if closed or i > 0 else p[i], p[(i + 1) % n] if closed or i < n - 1 else p[i]
        d = b - a
        d /= max(np.linalg.norm(d), 1e-9)
        out = p[i] - centre
        out -= d * np.dot(out, d)
        out /= max(np.linalg.norm(out), 1e-9)
        side = np.cross(d, out)
        t = np.linspace(0, 2 * np.pi, sides, endpoint=False)
        rings.append(p[i] + np.outer(np.cos(t) * radius, side) + np.outer(np.sin(t) * radius * flatten, out))
    m = sides
    verts = np.concatenate(rings)
    faces = []
    count = n if closed else n - 1
    for i in range(count):
        a, b = i * m, ((i + 1) % n) * m
        for j in range(m):
            k = (j + 1) % m
            faces.append([a + j, a + k, b + k, b + j])
    ob = stage.mesh_object(name, g.g2b(verts), faces)
    g.material(ob, mat(material))
    return ob


def rolled_edge(ob, radius, material, name=None, flatten=0.7):
    """A rolled rim along every open edge of ``ob`` (before it is
    thickened), as one object per edge."""
    out = []
    for i, loop in enumerate(boundary_loops(ob)):
        closed = np.linalg.norm(loop[0] - loop[-1]) < 0.05
        out.append(tube(f"{name or ob.name}_rim{i}", loop, radius, material, closed=closed, flatten=flatten))
    return out


def rivets(name, pts, normals, radius, material, joint=None):
    """Domed rivet heads at ``pts``, facing out along ``normals``."""
    obs = []
    for i, (p, n) in enumerate(zip(pts, normals)):
        bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=6, radius=radius,
                                             location=tuple(g.g2b(np.asarray(p))))
        ob = bpy.context.active_object
        ob.name = f"{name}{i}"
        n = np.asarray(n, float) / max(np.linalg.norm(n), 1e-9)
        from mathutils import Vector
        ob.rotation_mode = "QUATERNION"
        ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(tuple(g.g2b(n))))
        ob.scale = (1.0, 1.0, 0.5)
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        bpy.ops.object.shade_smooth()
        g.material(ob, mat(material))
        obs.append(rigid(ob, joint) if joint else ob)
    if not obs:
        return []
    return [g.join(obs, name)]


def section(body, r, a, b, at, gap):
    """The limb's cross-section where a band goes: the body's vertices near
    the plane across the bone ``a`` -> ``b`` at ``at`` of its length, as a
    radius (from the bone, plus ``gap``) at each of 72 angles round it, and
    the frame the angles are measured in."""
    co = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    root, tip = r.o[a], r.o[b]
    axis = (tip - root) / np.linalg.norm(tip - root)
    centre = root + (tip - root) * at
    front = np.array([0.0, 0.0, 1.0]) - axis * axis[2]
    front /= np.linalg.norm(front)
    side = np.cross(axis, front)
    rel = p - centre
    along = rel @ axis
    near = np.abs(along) < 0.02
    flat = rel[near] - np.outer(along[near], axis)
    dist = np.linalg.norm(flat, axis=1)
    near_idx = dist < 0.25
    flat, dist = flat[near_idx], dist[near_idx]
    ang = np.arctan2(flat @ side, flat @ front)
    bins = np.linspace(-np.pi, np.pi, 73)
    radius = np.zeros(72)
    for i in range(72):
        m = (ang >= bins[i]) & (ang < bins[i + 1])
        radius[i] = dist[m].max() if m.any() else np.nan
    good = ~np.isnan(radius)
    idx = np.arange(72)
    radius = np.interp(idx, idx[good], radius[good], period=72)
    # A band is stiff: it spans the hollows rather than following them.
    for _ in range(6):
        radius = np.maximum(radius, 0.5 * (np.roll(radius, 1) + np.roll(radius, -1)))
    return centre, axis, front, side, radius + gap


def band(body, r, a, b, at, width, gap, name, material, thick=0.008, bevel=0.003, rims=True,
         studs=0, stud_material=None):
    """A forged band round a limb: a ring of metal ``width`` wide following
    the limb's cross-section ``gap`` out (``section``), ``thick`` thick, its
    four corners bevelled by ``bevel``; a raised rim at each edge; and
    ``studs`` rivets round its middle. Carried whole by joint ``a``."""
    centre, axis, front, side, radius = section(body, r, a, b, at, gap)
    n = 72
    ang = np.linspace(-np.pi, np.pi, n, endpoint=False) + np.pi / n
    # The profile across the band, (along, out), a rounded rectangle.
    w, t, c = width / 2, thick, bevel
    prof = [(-w, 0.0), (-w, t - c), (-w + c * 0.3, t - c * 0.3), (-w + c, t), (w - c, t),
            (w - c * 0.3, t - c * 0.3), (w, t - c), (w, 0.0)]
    rings = []
    for i in range(n):
        d = np.cos(ang[i]) * front + np.sin(ang[i]) * side
        rings.append(np.array([centre + axis * x + d * (radius[i] + y) for x, y in prof]))
    m = len(prof)
    verts = np.concatenate(rings)
    faces = []
    for i in range(n):
        a0, b0 = i * m, ((i + 1) % n) * m
        for j in range(m - 1):
            faces.append([a0 + j, a0 + j + 1, b0 + j + 1, b0 + j])
        # The inside.
        faces.append([a0 + m - 1, a0, b0, b0 + m - 1])
    ob = stage.mesh_object(name, g.g2b(verts), faces)
    g.material(ob, mat(material))
    out = [rigid(ob, a)]
    if rims:
        for k, x in enumerate((-w, w)):
            pts = [centre + axis * x * 0.92 + (np.cos(q) * front + np.sin(q) * side) * (radius[i] + t)
                   for i, q in enumerate(ang)]
            out.append(rigid(tube(f"{name}_rim{k}", np.array(pts), thick * 0.45, material, closed=True,
                                  flatten=0.8), a))
    if studs:
        pick = np.linspace(0, n, studs, endpoint=False).astype(int)
        pts = [centre + (np.cos(ang[i]) * front + np.sin(ang[i]) * side) * (radius[i] + t) for i in pick]
        nrm = [np.cos(ang[i]) * front + np.sin(ang[i]) * side for i in pick]
        out += rivets(f"{name}_stud", pts, nrm, thick * 0.6, stud_material or material, joint=a)
    return out


def plate(body, name, gap, keep, material, trims=(), keeps=(), thick=0.007, rim=0.006, smooth=8, close=0.03,
          faces=900):
    """A plate cut from the body (``wardrobe.piece``): smoothed hard, as
    beaten metal is smooth, with a rolled rim along every edge (``rim``,
    the bead's radius), then given its thickness."""
    from wardrobe import piece
    ob = piece(body, name, gap, keep, trims=trims, keeps=keeps, thick=0.0, smooth=smooth, close=close,
               material=material, faces=faces, trim=0.0)
    hi = bpy.data.objects.get(name + "~hi")
    if hi is not None:
        bpy.data.objects.remove(hi)
    g.smooth_shape(ob, 0.5, 4)
    rims = rolled_edge(ob, rim, material, name=name) if rim else []
    g.thicken(ob, thick)
    ob.data.shade_smooth()
    return [ob] + rims


def ridge(ob, centre_x, half, height, front_only=True):
    """A keel raised down the middle of a plate (a breastplate's medial
    ridge, a helm's comb): vertices within ``half`` of ``x = centre_x``
    pushed out along their normals by up to ``height``, in a sharp-topped
    profile, on the front of the plate only if asked."""
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    nb = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("normal", nb)
    n = g.b2g(nb.reshape(-1, 3))
    d = np.abs(p[:, 0] - centre_x) / half
    push = height * np.clip(1.0 - d, 0.0, 1.0) ** 1.6
    if front_only:
        push *= np.clip(n[:, 2] * 3.0, 0.0, 1.0)
    ob.data.vertices.foreach_set("co", g.g2b(p + n * push[:, None]).ravel())
    ob.data.update()
    return ob
