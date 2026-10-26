"""Plate, designed rather than inflated.

``wardrobe.piece`` cuts a garment from the body inflated: right for cloth,
which follows the body, and wrong for plate, which does not -- in the game a
breastplate cut that way was the body's own soft, stocky shape a few
centimetres out, a snowsuit, and its rims followed the inflated shell's
ragged edge (the owner, 2026-10-26). A plate's form is its own: a breastplate
a smooth convex shell narrowing to the waist, a pauldron an oversized dome
over the shoulder, a vambrace a clean tube.

So a plate here is a **parametric surface** round one of the body's axes --
the trunk, a limb, the shoulder, the head: a point for each angle round the
axis (``u``) and distance along it (``v``), at the body's own measured
radius there (``radii``), smoothed into a shape, plus a gap, plus whatever
the design adds (a keel, a flare). Its borders are curves written in
``(u, v)`` -- a neckline, an armpit's scoop, a hem -- so its edges are clean,
and ``hard.tube`` rolls a rim along each.

Everything is in the game's frame, ``+y`` up, ``+z`` forward.
"""

import numpy as np

import garment as g
import hard
import stage
from wardrobe import mat, rigid


class Axis:
    """A frame round which a plate is laid: an origin, the axis along which
    ``v`` runs (metres from the origin), and the two directions ``u`` is
    measured between -- 0 toward ``front``, 90 degrees toward ``side``."""

    def __init__(self, origin, along, front, side):
        self.o = np.asarray(origin, float)
        self.a = np.asarray(along, float) / np.linalg.norm(along)
        f = np.asarray(front, float)
        f = f - self.a * np.dot(f, self.a)
        self.f = f / np.linalg.norm(f)
        s = np.asarray(side, float)
        s = s - self.a * np.dot(s, self.a) - self.f * np.dot(s, self.f)
        self.s = s / np.linalg.norm(s)

    def dir(self, u):
        u = np.radians(u)
        return np.cos(u)[..., None] * self.f + np.sin(u)[..., None] * self.s

    def point(self, u, v, radius):
        return self.o + np.asarray(v)[..., None] * self.a + self.dir(u) * np.asarray(radius)[..., None]


def body_points(body):
    co = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("co", co)
    return g.b2g(co.reshape(-1, 3))


def radii(body, ax, vs, us, keep=None, slab=0.02, reach=0.35, smooth_u=4, smooth_v=2, stiff=4):
    """The body's radius round ``ax`` at each ``(u, v)``: for each ``v`` the
    farthest vertex of the body in each direction ``u``, within ``slab`` of
    that height; then made stiff -- spanning hollows, as metal does -- and
    smoothed. ``keep`` (a mask on the body's vertices) limits which count."""
    p = body_points(body)
    if keep is not None:
        p = p[keep]
    rel = p - ax.o
    along = rel @ ax.a
    flat = rel - np.outer(along, ax.a)
    dist = np.linalg.norm(flat, axis=1)
    ang = np.degrees(np.arctan2(flat @ ax.s, flat @ ax.f))
    out = np.full((len(vs), len(us)), np.nan)
    for i, v in enumerate(vs):
        m = (np.abs(along - v) < slab) & (dist < reach)
        if not m.any():
            continue
        da, dd = ang[m], dist[m]
        for j, u in enumerate(us):
            near = np.abs((da - u + 180) % 360 - 180) < (360 / len(us)) * 0.75 + 4
            if near.any():
                out[i, j] = dd[near].max()
    # Fill what no vertex reached from its neighbours.
    for _ in range(40):
        bad = np.isnan(out)
        if not bad.any():
            break
        pad = np.pad(out, 1, mode="edge")
        nb = np.stack([pad[:-2, 1:-1], pad[2:, 1:-1], pad[1:-1, :-2], pad[1:-1, 2:]])
        fill = np.nanmean(nb, axis=0)
        out[bad] = fill[bad]
    out = np.nan_to_num(out, nan=np.nanmean(out))
    for _ in range(stiff):
        out = np.maximum(out, 0.5 * (np.roll(out, 1, 1) + np.roll(out, -1, 1)))
    for _ in range(smooth_u):
        out = 0.5 * out + 0.25 * (np.roll(out, 1, 1) + np.roll(out, -1, 1))
    for _ in range(smooth_v):
        out[1:-1] = 0.5 * out[1:-1] + 0.25 * (out[:-2] + out[2:])
    return out


def surface(name, ax, us, vs, radius, v_lo=None, v_hi=None, closed=False):
    """A plate's surface: ``radius[i, j]`` at ``(us[j], vs[i])``; each column
    ``u`` running from ``v_lo(u)`` to ``v_hi(u)`` (arrays the length of
    ``us``, or None for the full range). Returns the object and its borders
    as point arrays (for rims)."""
    nu, nv = len(us), len(vs)
    lo = np.full(nu, vs[0]) if v_lo is None else np.asarray(v_lo, float)
    hi = np.full(nu, vs[-1]) if v_hi is None else np.asarray(v_hi, float)
    rows = 24
    grid = np.zeros((rows, nu, 3))
    for j in range(nu):
        vv = np.linspace(lo[j], hi[j], rows)
        rr = np.interp(vv, vs, radius[:, j])
        grid[:, j] = ax.point(np.full(rows, us[j]), vv, rr)
    verts = grid.reshape(-1, 3)
    faces = []
    cols = nu if closed else nu - 1
    for i in range(rows - 1):
        for j in range(cols):
            a, b = i * nu + j, i * nu + (j + 1) % nu
            faces.append([a, b, b + nu, a + nu])
    ob = stage.mesh_object(name, g.g2b(verts), faces)
    ob.data.shade_smooth()
    borders = [grid[0], grid[-1]]
    if not closed:
        borders += [grid[:, 0], grid[:, -1]]
    return ob, borders


def finish(ob, borders, material, thick, rim, rim_material=None, joint=None, rim_closed=False, sharp=None):
    """Give a plate its thickness and a rolled rim along each border, carried
    by ``joint`` if given (otherwise bound to the body like a garment).
    ``sharp`` (degrees): edges folded more than this are hard creases, as an
    angular plate's ridges are; otherwise it is smooth all over."""
    g.material(ob, mat(material))
    g.thicken(ob, thick)
    ob.data.shade_smooth()
    if sharp:
        ob.data.set_sharp_from_angle(angle=float(np.radians(sharp)))
    out = [ob]
    for k, b in enumerate(borders):
        if rim:
            t = hard.tube(f"{ob.name}_rim{k}", b, rim, rim_material or material, closed=rim_closed, flatten=0.85,
                          step=0.014, sides=6)
            out.append(t)
    if joint:
        out = [rigid(o, joint) for o in out]
    return out


def smoothstep(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3 - 2 * t)


def designed(us, vs, half_width, front, back, power=2.4):
    """A designed section round the trunk at each height: a rounded box
    (a superellipse of ``power``) ``half_width`` to each side, ``front``
    ahead of the axis and ``back`` behind it -- each a function of ``vs``
    -- as a radius at each angle ``us`` (degrees from the front)."""
    u = np.radians(us)[None, :]
    w = np.asarray(half_width, float)[:, None]
    d = np.where(np.cos(u) >= 0, np.asarray(front, float)[:, None], np.asarray(back, float)[:, None])
    c, s = np.abs(np.cos(u)), np.abs(np.sin(u))
    return 1.0 / ((c / d) ** power + (s / w) ** power) ** (1.0 / power)
