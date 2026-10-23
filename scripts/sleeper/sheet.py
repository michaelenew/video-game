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
from terrain import CELL, build, placed

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
    panels = []
    size = (480, 360)
    for label, d, up, mirror in VIEWS:
        panels.append((f"1 Model: {label}", view(mv, mt, d, size=size, colours=mc, up=up, mirror=mirror)))
        panels.append(("2 Its shape alone, on flat ground", view(sv, st, d, size=size, colours=sc, up=up, mirror=mirror)))
        panels.append(("3 On the mountain, as the game has it", view(lv, lt, d, size=size, colours=lc, up=up, mirror=mirror)))
    title = "The Sleeper. 1: the beast asleep (pale: bone). 2: the shape made from it. 3: that shape on the mountain."
    sheet(panels, columns=3, title=title).save(out)
    print(out)


if __name__ == "__main__":
    main()
