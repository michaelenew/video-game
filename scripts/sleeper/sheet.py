"""The Sleeper, three ways: the posed model, the shape made of it, and the
land it became.

    cargo run --release -p look --example sleeper
    python3 scripts/sleeper/sheet.py [out.png]

Both are drawn on the map, from the same four directions, so a row reads
across: the model as it lies; the shape `terrain.py` makes of it, alone on
flat ground; and the mountain with that shape in it -- the simulation's own
heights (`Land::height`), noise, crags and all.
"""
import sys
from pathlib import Path

import numpy as np

from pose import asleep
from render import sheet, view
from terrain import ARCH, CELL, build, placed

ROOT = Path(__file__).resolve().parents[2]

# Where each view looks from, as a direction to look along on the map
# (x east, y up, z north), and what it is.
VIEWS = [
    ("from above, north up, east right (as the map sheet)", (0.0001, -1, 0.0001), (0, 0, 1), True),
    ("from the High Plane: west of it, looking east and down", (1, -0.45, 0.15), (0, 1, 0), False),
    ("from the valley: south of it, looking north", (0.1, -0.2, 1), (0, 1, 0), False),
    ("from the east, looking west", (-1, -0.3, -0.2), (0, 1, 0), False),
]

# How far past the beast's footprint the land is cut out, metres.
MARGIN = 30.0


def grid_mesh(west, south, cell, h):
    """A heightfield as a mesh, coloured rock where steep and snow where not."""
    nz, nx = h.shape
    gx, gz = np.meshgrid(west + np.arange(nx) * cell, south + np.arange(nz) * cell)
    verts = np.c_[gx.ravel(), h.ravel(), gz.ravel()]
    k = np.arange(nx * nz).reshape(nz, nx)
    a, b, c, d = k[:-1, :-1].ravel(), k[:-1, 1:].ravel(), k[1:, :-1].ravel(), k[1:, 1:].ravel()
    tris = np.r_[np.c_[a, b, d], np.c_[a, d, c]]
    p = verts[tris]
    n = np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0])
    n /= np.linalg.norm(n, axis=1, keepdims=True)
    steep = np.abs(n[:, 1]) < 0.62
    col = np.where(steep[:, None], np.array([0.45, 0.47, 0.52]), np.array([0.90, 0.92, 0.95]))
    return verts, tris, col


def shape(lo, hi):
    """What `terrain.py` hands the simulation: the lift alone, on flat
    ground, cut to the same block, every other cell (it is fine)."""
    (west, south), lift = build()
    xs = west + np.arange(lift.shape[1]) * CELL
    zs = south + np.arange(lift.shape[0]) * CELL
    kx = (xs >= lo[0]) & (xs <= hi[0])
    kz = (zs >= lo[1]) & (zs <= hi[1])
    lift = lift[kz][:, kx][::2, ::2]
    return grid_mesh(xs[kx][0], zs[kz][0], CELL * 2, lift)


def land_heights():
    """The land's own height at a point, from the dump, nearest cell."""
    lines = (ROOT / "target" / "sleeper-land.txt").read_text().split("\n")
    west, south, cell, nx, nz = (float(v) for v in lines[0].split())
    h = np.array([[float(v) for v in line.split()] for line in lines[1 : 1 + int(nz)]])

    def at(x, z):
        i = int(np.clip(round((z - south) / cell), 0, h.shape[0] - 1))
        j = int(np.clip(round((x - west) / cell), 0, h.shape[1] - 1))
        return h[i, j]

    return at


def arch_curves():
    """The horns' stone as `terrain.py` wrote it: a list of points (x, z,
    height over the mountain, radius), metres, a horn."""
    out, cur = [], []
    for line in ARCH.read_text().splitlines():
        line = line.strip()
        if line.startswith("#"):
            continue
        if not line:
            if cur:
                out.append(cur)
            cur = []
            continue
        cur.append([int(n) / 100 for n in line.split()])
    if cur:
        out.append(cur)
    return out


def tubes(curves, base, colour=(0.62, 0.63, 0.66)):
    """The horns' stone as a mesh: rings round each curve, `base(x, z)`
    added to each horn's heights (one height a horn, as the game sets it)."""
    verts, tris = [], []
    for c in curves:
        b = np.mean([base(x, z) for x, z, _, _ in c])
        pts = np.array([[x, y + b, z] for x, z, y, _ in c])
        rad = [r for _, _, _, r in c]
        for i, (p, r) in enumerate(zip(pts, rad)):
            d = pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]
            d /= np.linalg.norm(d)
            s1 = np.cross(d, [0, 1, 0]) if abs(d[1]) < 0.9 else np.cross(d, [1, 0, 0])
            s1 /= np.linalg.norm(s1)
            s2 = np.cross(d, s1)
            start = len(verts)
            for k in range(16):
                a = 2 * np.pi * k / 16
                verts.append(p + (s1 * np.cos(a) + s2 * np.sin(a)) * r)
            if i:
                for k in range(16):
                    a0, a1 = start - 16 + k, start - 16 + (k + 1) % 16
                    b0, b1 = start + k, start + (k + 1) % 16
                    tris += [(a0, a1, b1), (a0, b1, b0)]
    if not verts:
        return np.zeros((0, 3)), np.zeros((0, 3), int), np.zeros((0, 3))
    return np.array(verts), np.array(tris), np.tile(colour, (len(tris), 1))


def joined(*meshes):
    """Meshes as one: vertices, triangles, colours."""
    vs, ts, cs, n = [], [], [], 0
    for v, t, c in meshes:
        vs.append(v)
        ts.append(t + n)
        cs.append(c)
        n += len(v)
    return np.concatenate(vs), np.concatenate(ts), np.concatenate(cs)


def land(lo, hi):
    """The land round the Sleeper as a mesh, cut out as a block from `lo`
    to `hi` (x, z), and a colour a triangle."""
    path = ROOT / "target" / "sleeper-land.txt"
    lines = path.read_text().split("\n")
    west, south, cell, nx, nz = (float(v) for v in lines[0].split())
    nx, nz = int(nx), int(nz)
    h = np.array([[float(v) for v in line.split()] for line in lines[1 : 1 + nz]])
    xs = west + np.arange(nx) * cell
    zs = south + np.arange(nz) * cell
    keep_x = (xs >= lo[0]) & (xs <= hi[0])
    keep_z = (zs >= lo[1]) & (zs <= hi[1])
    h, xs, zs = h[keep_z][:, keep_x], xs[keep_x], zs[keep_z]
    nx, nz = len(xs), len(zs)
    gx, gz = np.meshgrid(xs, zs)
    verts = np.c_[gx.ravel(), h.ravel(), gz.ravel()]
    k = np.arange(nx * nz).reshape(nz, nx)
    a, b, c, d = k[:-1, :-1].ravel(), k[:-1, 1:].ravel(), k[1:, :-1].ravel(), k[1:, 1:].ravel()
    tris = np.r_[np.c_[a, b, d], np.c_[a, d, c]]
    # Rock where it is steep, snow where it is not, darker lower down.
    p = verts[tris]
    n = np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0])
    n /= np.linalg.norm(n, axis=1, keepdims=True)
    steep = np.abs(n[:, 1]) < 0.62
    rock = np.array([0.45, 0.47, 0.52])
    snow = np.array([0.90, 0.92, 0.95])
    col = np.where(steep[:, None], rock, snow)
    return verts, tris, col


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "target/sleeper-sheet.png"
    mv, mt = placed()
    _, _, mc = asleep()
    lo = mv[:, [0, 2]].min(0) - MARGIN
    hi = mv[:, [0, 2]].max(0) + MARGIN
    lv, lt, lc = land(lo, hi)
    sv, st, sc = shape(lo, hi)
    # The horns' stone, on the shape (heights over the mountain) and on the
    # mountain (on the land's own height there, less the beast's lift).
    curves = arch_curves()
    sv, st, sc = joined((sv, st, sc), tubes(curves, lambda x, z: 0.0))
    (west, south), lift = build()
    land_h = land_heights()

    def bare(x, z):
        i, j = int(round((z - south) / CELL)), int(round((x - west) / CELL))
        under = lift[i, j] if 0 <= i < lift.shape[0] and 0 <= j < lift.shape[1] else 0.0
        return land_h(x, z) - under

    lv, lt, lc = joined((lv, lt, lc), tubes(curves, bare))
    panels = []
    size = (480, 360)
    for label, d, up, mirror in VIEWS:
        panels.append((f"1 Model: {label}", view(mv, mt, d, size=size, colours=mc, up=up, mirror=mirror)))
        panels.append(("2 Its shape alone, on flat ground", view(sv, st, d, size=size, colours=sc, up=up, mirror=mirror)))
        panels.append(("3 On the mountain, as the game has it", view(lv, lt, d, size=size, colours=lc, up=up, mirror=mirror)))
    title = "The Sleeper. 1: the beast asleep (pale: bone). 2: the shape made from it, its horns as stone. 3: that on the mountain."
    sheet(panels, columns=3, title=title).save(out)
    print(out)


if __name__ == "__main__":
    main()
