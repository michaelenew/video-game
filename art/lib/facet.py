"""Faceted plate: armour drawn as flat faces meeting at crisp edges.

``plate`` lays a surface round the body at the body's own radius, which is
right for a vambrace and wrong for the plate of a game's knight: the owner's
reference (2026-10-26) has pauldrons and a breastplate made of broad *flat*
faces, each meeting the next at a hard, bevelled edge that catches one line
of light, its silhouette running to points -- and a frame of a second metal
round each face. "Angular" there is not ridges added to a dome; it is a
handful of planes.

So a faceted plate here is what a hard-surface artist starts from: a coarse
**control grid** of points (rows by columns, written by hand in a local
frame), every control quad a flat face, refined only so it bends with the
skin no differently from the rest. A **frame** -- a band ``frame`` metres
wide round the grid's border, raised a little and in the trim's material --
is cut from the same surface, so frame and field are one piece with a step
between them. Then it is given its thickness and a **bevel** on every edge
folded more than ``crease`` degrees: the chamfer is what draws the line of
light along an edge.

Everything is in the game's frame, ``+y`` up, ``+z`` forward.
"""

import bpy
import numpy as np

import garment as g
import plate as plates
import stage
from wardrobe import J, mat, rigid


class Frame:
    """A local frame: ``origin``, and the directions that local ``x``
    (out), ``y`` (up) and ``z`` (forward) point in the game's frame."""

    def __init__(self, origin, x, y, z):
        self.o = np.asarray(origin, float)
        self.m = np.stack([np.asarray(v, float) for v in (x, y, z)])

    def __call__(self, p):
        return self.o + np.asarray(p, float) @ self.m


def _refine(ctrl, per):
    """Each control quad split ``per`` by ``per`` (bilinearly, so a flat
    quad stays flat)."""
    m, n, _ = ctrl.shape
    rows = (m - 1) * per + 1
    cols = (n - 1) * per + 1
    out = np.zeros((rows, cols, 3))
    for i in range(rows):
        a, s = divmod(i, per)
        if a == m - 1:
            a, s = m - 2, per
        t = s / per
        for j in range(cols):
            b, q = divmod(j, per)
            if b == n - 1:
                b, q = n - 2, per
            u = q / per
            p00, p01, p10, p11 = ctrl[a, b], ctrl[a, b + 1], ctrl[a + 1, b], ctrl[a + 1, b + 1]
            out[i, j] = (p00 * (1 - u) + p01 * u) * (1 - t) + (p10 * (1 - u) + p11 * u) * t
    return out


def _inset(grid, frame):
    """The grid with a row and a column added ``frame`` metres inside each
    border, so the band between can be the frame."""
    def cut(g_, axis):
        a = np.moveaxis(g_, axis, 0)
        first = np.linalg.norm(a[1] - a[0], axis=-1)
        last = np.linalg.norm(a[-1] - a[-2], axis=-1)
        t0 = np.clip(frame / np.maximum(first, 1e-6), 0, 0.9)[..., None]
        t1 = np.clip(frame / np.maximum(last, 1e-6), 0, 0.9)[..., None]
        lo = a[0] + (a[1] - a[0]) * t0
        hi = a[-1] + (a[-2] - a[-1]) * t1
        a = np.concatenate([a[:1], lo[None], a[1:-1], hi[None], a[-1:]])
        return np.moveaxis(a, 0, axis)
    return cut(cut(grid, 0), 1)


def plate(name, ctrl, frame=0.012, per=3, raise_frame=0.0025, thick=0.006, bevel=0.0025, crease=24,
          material="plate", trim="trim", joint=None, edges=(True, True, True, True), shape=None, inside=None,
          weld=False):
    """A faceted plate from control points ``ctrl`` (rows x columns x 3, in
    the game's frame -- or, with ``shape``, in whatever coordinates
    ``shape`` maps to it, refined there first: a plate laid on a ``Shell``
    follows its facets exactly). ``frame``: the width of the trim round its
    border (``edges`` says which borders have one: top row, bottom row,
    first column, last column). ``inside``: a point behind the plate, which
    its outside faces away from (the middle of the grid if not given).
    Returns the plate, carried by ``joint``."""
    grid = _refine(np.asarray(ctrl, float), per)
    if shape is not None:
        grid = shape(grid)
    if frame:
        grid = _inset(grid, frame)
    rows, cols, _ = grid.shape
    # Each vertex's normal, for raising the frame.
    du = np.gradient(grid, axis=1)
    dv = np.gradient(grid, axis=0)
    nrm = np.cross(du, dv)
    nrm /= np.maximum(np.linalg.norm(nrm, axis=-1, keepdims=True), 1e-9)
    # Outward is away from the plate's middle.
    behind = grid.mean(axis=(0, 1)) if inside is None else np.asarray(inside, float)
    if np.mean(np.sum(nrm * (grid - behind), axis=-1)) < 0:
        nrm = -nrm
    in_frame = np.zeros((rows, cols), bool)
    if frame:
        top, bottom, first, last = edges
        if top:
            in_frame[0] = True
        if bottom:
            in_frame[-1] = True
        if first:
            in_frame[:, 0] = True
        if last:
            in_frame[:, -1] = True
    verts = grid + nrm * (raise_frame * in_frame)[..., None]
    faces, kinds = [], []
    for i in range(rows - 1):
        for j in range(cols - 1):
            faces.append([i * cols + j, i * cols + j + 1, (i + 1) * cols + j + 1, (i + 1) * cols + j])
            framed = frame and ((edges[0] and i == 0) or (edges[1] and i == rows - 2)
                                or (edges[2] and j == 0) or (edges[3] and j == cols - 2))
            kinds.append(1 if framed else 0)
    ob = stage.mesh_object(name, g.g2b(verts.reshape(-1, 3)), faces)
    ob.data.materials.append(mat(material))
    ob.data.materials.append(mat(trim))
    ob.data.polygons.foreach_set("material_index", kinds)
    if weld:
        import bmesh
        bm = bmesh.new()
        bm.from_mesh(ob.data)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        bm.to_mesh(ob.data)
        bm.free()
    _thicken_bevel(ob, thick, bevel, crease)
    return rigid(ob, joint) if joint else ob


def _thicken_bevel(ob, thick, bevel, crease):
    m = ob.modifiers.new("solid", "SOLIDIFY")
    m.thickness = thick
    m.offset = -1.0
    m.use_even_offset = True
    m.thickness_clamp = 2.0
    m.use_rim = True
    g.apply(ob, m)
    if bevel:
        b = ob.modifiers.new("bevel", "BEVEL")
        b.width = bevel
        b.segments = 2
        b.limit_method = "ANGLE"
        b.angle_limit = float(np.radians(crease))
        b.harden_normals = False
        g.apply(ob, b)
    ob.data.shade_smooth()
    ob.data.set_sharp_from_angle(angle=float(np.radians(crease)))
    g.tame(ob)


def points(frame, rows):
    """Control points from rows of local ``(x, y, z)`` triples."""
    return np.array([[frame(p) for p in row] for row in rows])


class Shell:
    """A faceted body of plate round an upright axis -- a breastplate's
    form: at each key height ``vs`` a polygon with a corner at each key
    angle ``us`` (degrees from the front, toward ``ax.s``), ``R[i, j]`` out
    from the axis; between key heights the corners run straight. Points
    are asked for as ``(u, v, lift)``: on the polygon's face at angle ``u``,
    height ``v``, ``lift`` metres proud of it -- so a plate laid on it
    (``plate(..., shape=shell)``) lies flat on its faces."""

    def __init__(self, ax, us, vs, R):
        self.ax, self.us, self.vs, self.R = ax, np.asarray(us, float), np.asarray(vs, float), np.asarray(R, float)

    def corners(self, v):
        """The key polygon's radii at height ``v`` (linear between keys)."""
        return np.array([np.interp(v, self.vs, self.R[:, j]) for j in range(len(self.us))])

    def __call__(self, q):
        q = np.asarray(q, float)
        flat = q.reshape(-1, 3)
        out = np.zeros_like(flat)
        uk = np.radians(self.us)
        for n, (u, v, lift) in enumerate(flat):
            rad = self.corners(v)
            j = int(np.clip(np.searchsorted(self.us, u) - 1, 0, len(self.us) - 2))
            # The corners either side, in the section's plane, and where
            # the ray at angle u crosses the edge between them.
            a = rad[j] * np.array([np.cos(uk[j]), np.sin(uk[j])])
            b = rad[j + 1] * np.array([np.cos(uk[j + 1]), np.sin(uk[j + 1])])
            d = np.array([np.cos(np.radians(u)), np.sin(np.radians(u))])
            e = b - a
            den = d[0] * e[1] - d[1] * e[0]
            t = (a[0] * e[1] - a[1] * e[0]) / den if abs(den) > 1e-9 else np.linalg.norm(a)
            normal = np.array([e[1], -e[0]]) / max(np.linalg.norm(e), 1e-9)
            if np.dot(normal, a + b) < 0:
                normal = -normal
            p2 = d * t + normal * lift
            out[n] = self.ax.o + v * self.ax.a + p2[0] * self.ax.f + p2[1] * self.ax.s
        return out.reshape(q.shape)


REST_SPREAD = 11.0  # degrees: view::being::REST_SPREAD[0], the dark being's arms as they hang


def rest_frame(r, arm, el, spread=REST_SPREAD):
    """A frame at the shoulder in which a plate is drawn as it is seen -- the
    arm hanging ``spread`` degrees off the body, as a being stands --
    turned into the bind pose the arm is built in, so that carried by the
    arm it is where it was drawn. ``x`` out, ``y`` up, ``z`` forward."""
    sx = np.sign(r.o[arm][0])
    q = np.radians(spread)
    down = np.array([sx * np.sin(q), -np.cos(q), 0.0])
    out = np.array([sx * np.cos(q), np.sin(q), 0.0])
    bind = r.dir(arm, el)
    axis = np.cross(down, bind)
    sin, cos = np.linalg.norm(axis), float(np.dot(down, bind))
    k = axis / max(sin, 1e-9)
    K = np.array([[0, -k[2], k[1]], [k[2], 0, -k[0]], [-k[1], k[0], 0]])
    turn = np.eye(3) + sin * K + (1 - cos) * K @ K
    return Frame(r.o[arm], turn @ out, turn @ -down, turn @ np.array([0.0, 0.0, 1.0]))


def bone_frame(r, a, b, lateral):
    """A frame at joint ``a`` along the bone to ``b``: ``x`` toward
    ``lateral`` (the outside of the limb), ``y`` back up the bone, ``z``
    forward."""
    up = -r.dir(a, b)
    x = np.asarray(lateral, float) - up * np.dot(lateral, up)
    x /= np.linalg.norm(x)
    z = np.array([0.0, 0.0, 1.0]) - x * x[2] - up * up[2]
    z /= np.linalg.norm(z)
    return Frame(r.o[a], x, up, z)


def round_limb(fr, rows):
    """Control points round a limb's frame (``y`` along it): each row a
    list of ``(phi, h, radius)`` -- ``phi`` degrees from ``x`` toward ``z``,
    ``h`` metres along ``y``."""
    return np.array([[fr([rad * np.cos(np.radians(phi)), h, rad * np.sin(np.radians(phi))])
                      for phi, h, rad in row] for row in rows])


def limb_radius(body, r, a, b, at, gap=0.0):
    """How far the limb's skin stands from the bone ``a`` -> ``b`` at ``at``
    of its length, plus ``gap``: only the limb's own skin is measured (the
    trunk beside a hanging arm is not the arm)."""
    p = plates.body_points(body)
    own = g.weights_of(body)[:, J[a]] > 0.4
    axis = r.dir(a, b)
    centre = r.o[a] + (r.o[b] - r.o[a]) * at
    rel = p[own] - centre
    along = rel @ axis
    flat = np.linalg.norm(rel - np.outer(along, axis), axis=1)
    m = np.abs(along) < 0.025
    # Not the very largest: at the elbow a few of the forearm's vertices
    # are the biceps folded over it.
    return float(np.percentile(flat[m], 92)) + gap
