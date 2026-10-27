"""The Sleeper from where a player stands: the land with the beast laid on
it, drawn by a camera like the game's.

    cargo run --release -p look --example sleeper
    python3 scripts/sleeper/vista.py [out.png]

`sheet.py` judges each step of the pipeline from orthographic angles; this
judges the result the way it will be met -- from the High Plane's edge, the
Pinewood road, the Saddle -- with the ranges in front of it, in the haze.
The land is the simulation's own with no Sleeper on it (`sleeper-bare.txt`),
and the beast is laid on it the way `sim::valley::dragon` and `land.rs` lay
it: the range carved down round it (`terrain.carve`), its lift on top,
shown in by how far the mountain has risen. Approximate in one way: the
land's crags, which the game damps on the beast, are left as they are.
"""
import sys
from pathlib import Path

import numpy as np

import terrain
from render import camera
from sheet import arch_curves

ROOT = Path(__file__).resolve().parents[2]

# Where a player stands to see it, and where they look: (name, eye, target),
# map metres.
VIEWPOINTS = [
    ("High Plane, east edge, standing", (548, 309, 400), (790, 215, 400)),
    ("High Plane, north-east corner", (545, 309, 470), (800, 210, 360)),
    ("Pinewood road, looking north", (770, 70, 10), (790, 190, 400)),
    ("The Saddle, looking north-west", (960, 100, 40), (790, 200, 400)),
    ("The Hollows, looking north", (735, 64, 75), (790, 200, 400)),
    ("From the air, south-west", (560, 420, 170), (790, 230, 410)),
]


def bare():
    """The land with no Sleeper: (west, south, cell), heights and rises."""
    lines = (ROOT / "target" / "sleeper-bare.txt").read_text().split("\n")
    west, south, cell, nx, nz = (float(v) for v in lines[0].split())
    rows = [[tuple(float(n) for n in c.split(",")) for c in line.split()] for line in lines[1 : 1 + int(nz)]]
    a = np.array(rows)
    return (west, south, cell), a[..., 0], a[..., 1]


def sample(grid, lo, cell, xs, zs):
    """A grid (rows from the south) at map points, bilinear; zero off it."""
    gx = (xs - lo[0]) / cell
    gz = (zs - lo[1]) / cell
    i0, j0 = np.floor(gz).astype(int), np.floor(gx).astype(int)
    fz, fx = gz - i0, gx - j0
    nz, nx = grid.shape

    def at(i, j):
        ok = (i >= 0) & (j >= 0) & (i < nz) & (j < nx)
        return np.where(ok, grid[np.clip(i, 0, nz - 1), np.clip(j, 0, nx - 1)], 0.0)

    top = at(i0, j0) * (1 - fx) + at(i0, j0 + 1) * fx
    bottom = at(i0 + 1, j0) * (1 - fx) + at(i0 + 1, j0 + 1) * fx
    return top * (1 - fz) + bottom * fz


def land():
    """The land with the beast on it, as the game lays it: a mesh, its
    colours, and the heights."""
    (west, south, cell), h, rise = bare()
    nz, nx = h.shape
    xs = west + np.arange(nx) * cell
    zs = south + np.arange(nz) * cell
    gx, gz = np.meshgrid(xs, zs)
    (lw, ls), lift = terrain.build()
    carve = terrain.carve(lift)
    shows = np.clip((rise - 40) / 60, 0, 1)
    l = sample(lift, (lw, ls), terrain.CELL, gx, gz) * shows
    c = sample(carve, (lw, ls), terrain.CELL, gx, gz) * shows
    floor = np.minimum(h, terrain.CARVE_TO)
    carved = h - (h - floor) * c
    y = carved + l
    verts = np.c_[gx.ravel(), y.ravel(), gz.ravel()]
    k = np.arange(nx * nz).reshape(nz, nx)
    a, b, cc, d = k[:-1, :-1].ravel(), k[:-1, 1:].ravel(), k[1:, :-1].ravel(), k[1:, 1:].ravel()
    tris = np.r_[np.c_[a, b, d], np.c_[a, d, cc]]
    p = verts[tris]
    n = np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0])
    n /= np.linalg.norm(n, axis=1, keepdims=True)
    steep = np.abs(n[:, 1]) < 0.58
    low = p[:, :, 1].mean(1) < 75
    col = np.where(steep[:, None], [0.40, 0.44, 0.50], [0.90, 0.93, 0.95])
    col = np.where((low & ~steep)[:, None], [0.42, 0.62, 0.40], col)
    return verts, tris, col, ((west, south, cell), carved)


def arch_mesh(heights):
    """The horn's stone, set on the carved mountain with no lift, as the
    game sets it (`dragon::arches`)."""
    (west, south, cell), carved = heights
    out_v, out_t = [], []
    for c in arch_curves():
        base = np.mean([sample(carved, (west, south), cell, np.array(x), np.array(z)) for x, z, _, _ in c])
        pts = np.array([[x, yy + base, z] for x, z, yy, _ in c])
        rad = [r for *_, r in c]
        for i, (p, r) in enumerate(zip(pts, rad)):
            d = pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]
            d /= np.linalg.norm(d)
            s1 = np.cross(d, [0, 1, 0])
            s1 /= np.linalg.norm(s1)
            s2 = np.cross(d, s1)
            start = len(out_v)
            for k in range(16):
                a = 2 * np.pi * k / 16
                out_v.append(p + (s1 * np.cos(a) * 1.25 + s2 * np.sin(a) * 0.8) * r)
            if i:
                for k in range(16):
                    a0 = start - 16 + k
                    out_t += [(a0, start - 16 + (k + 1) % 16, start + (k + 1) % 16), (a0, start + (k + 1) % 16, start + k)]
    if not out_v:
        return np.zeros((0, 3)), np.zeros((0, 3), int), np.zeros((0, 3))
    return np.array(out_v), np.array(out_t), np.tile([0.45, 0.47, 0.52], (len(out_t), 1))


def main():
    from render import sheet

    out = sys.argv[1] if len(sys.argv) > 1 else "target/sleeper-vista.png"
    v, t, c, heights = land()
    av, at, ac = arch_mesh(heights)
    if len(av):
        t = np.r_[t, at + len(v)]
        v = np.r_[v, av]
        c = np.r_[c, ac]
    panels = [(name, camera(v, t, c, eye, target)) for name, eye, target in VIEWPOINTS]
    sheet(panels, columns=2, title="The Sleeper from where a player stands (preview of the game's land, not a screenshot)").save(out)
    print(out)


if __name__ == "__main__":
    main()
