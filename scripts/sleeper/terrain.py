"""The Sleeper's terrain: the posed model, laid on the map as a heightfield.

    python3 scripts/sleeper/terrain.py

Writes `crates/sim/src/valley/sleeper.bin`, which `sim::valley::dragon`
reads: how far the beast raises the land, on a grid of cells over its
footprint. The steps, each a number below:

1. **Pose** it (`pose.py`).
2. **Size** it: so long, nose to tail, in metres.
3. **Turn** it to face a bearing on the map, and put it where it lies.
4. **Look down on it**: the highest point of the body over each cell.
5. **Sink** it: take a depth off everything, so the lowest of it -- the
   legs folded under, the belly -- is under the mountain, and what stands
   out is its back, its haunch, its curled tail and its head.
6. **Settle** it: nothing on it is steeper than [`SETTLE`] -- a body's
   flank seen from above is a wall, and a mountain's is a slope, with scree
   and snow built up against it.
7. **Soften** it a little, so a low-poly facet does not alias on the land's
   mesh.

The file: a header of six little-endian i32 -- the grid's west and south
edge on the map in centimetres, the cell's side in centimetres, the number
of cells east and north, and the scale of a height (heights are in units of
1/that metre) -- and then one u16 a cell, row by row from the south.
"""
import struct
from pathlib import Path

import numpy as np

from pose import mesh

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "crates" / "sim" / "src" / "valley" / "sleeper.bin"

# How long it is, nose to tail along its longest side, metres.
LENGTH = 300.0
# How much of its height it keeps: below one, it lies lower than the model
# stands -- flatter, more mountain than animal.
SQUASH = 0.6
# Where it lies on the map (metres) -- the middle of its footprint -- and
# which way it faces: the bearing of its nose, degrees clockwise from north
# (+z). 180 is south, at the valley.
AT = (790.0, 400.0)
FACING = 200.0
# How deep it is sunk into the mountain, metres.
SINK = 18.0
# The steepest any of it stands, rise over run: a flank's wall settles to
# this. The valley's mountainsides are 1.4.
SETTLE = 1.4
# The cell of the grid, metres.
CELL = 2.0
# How many cells the softening reaches.
SOFTEN = 1
# A height's unit: 1/HEIGHT_SCALE metres.
HEIGHT_SCALE = 100


def placed():
    """The posed mesh in map metres: x east, y up, z north."""
    v, tris = mesh()
    span = v[:, [0, 2]].max(0) - v[:, [0, 2]].min(0)
    k = LENGTH / span.max()
    v = v * k
    v[:, 1] *= SQUASH
    # The model faces +z with its left at +x. Turned so its nose is on the
    # bearing: a bearing b (clockwise from north) is the direction
    # (sin b, cos b) on the map's (x, z).
    b = np.radians(FACING)
    fwd = np.array([np.sin(b), np.cos(b)])
    # The map and the model are both x, y up, z, right-handed: facing +z
    # its left is +x, so facing any bearing its left is (fwd.z, -fwd.x).
    left = np.array([fwd[1], -fwd[0]])
    xz = np.outer(v[:, 0], left) + np.outer(v[:, 2], fwd)
    mid = (xz.max(0) + xz.min(0)) / 2
    xz = xz - mid + np.array(AT)
    out = np.c_[xz[:, 0], v[:, 1], xz[:, 1]]
    return out, tris


def heightfield(v, tris):
    """The highest point of the body over each cell: (west, south) of the
    grid in metres, and the heights, rows from the south."""
    lo = np.floor(v[:, [0, 2]].min(0) / CELL) * CELL - CELL * (SOFTEN + 2)
    hi = np.ceil(v[:, [0, 2]].max(0) / CELL) * CELL + CELL * (SOFTEN + 2)
    nx, nz = ((hi - lo) / CELL).astype(int) + 1
    top = np.zeros((nz, nx))
    gx = (v[:, 0] - lo[0]) / CELL
    gz = (v[:, 2] - lo[1]) / CELL
    for i, j, l in tris:
        xs, zs, ys = gx[[i, j, l]], gz[[i, j, l]], v[[i, j, l], 1]
        x0, x1 = int(np.floor(xs.min())), int(np.ceil(xs.max())) + 1
        z0, z1 = int(np.floor(zs.min())), int(np.ceil(zs.max())) + 1
        px, pz = np.meshgrid(np.arange(x0, x1), np.arange(z0, z1))
        d = (zs[1] - zs[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (zs[0] - zs[2])
        if abs(d) < 1e-12:
            continue
        w0 = ((zs[1] - zs[2]) * (px - xs[2]) + (xs[2] - xs[1]) * (pz - zs[2])) / d
        w1 = ((zs[2] - zs[0]) * (px - xs[2]) + (xs[0] - xs[2]) * (pz - zs[2])) / d
        w2 = 1 - w0 - w1
        inside = (w0 >= -1e-6) & (w1 >= -1e-6) & (w2 >= -1e-6)
        y = w0 * ys[0] + w1 * ys[1] + w2 * ys[2]
        sub = top[z0:z1, x0:x1]
        np.maximum(sub, np.where(inside, y, 0), out=sub)
    return lo, top


def settle(h, slope):
    """Raise the ground round anything steeper than `slope` until it is not:
    each cell at least its neighbours' height less `slope` times the step
    to them, until nothing changes."""
    diag = np.sqrt(2)
    steps = [((0, 1), 1), ((0, -1), 1), ((1, 0), 1), ((-1, 0), 1), ((1, 1), diag), ((1, -1), diag), ((-1, 1), diag), ((-1, -1), diag)]
    while True:
        before = h
        for (dz, dx), d in steps:
            h = np.maximum(h, np.roll(np.roll(h, dz, 0), dx, 1) - slope * CELL * d)
        if np.array_equal(h, before):
            return h


def soften(h, n):
    """A box blur `n` cells each way, twice: near enough a Gaussian."""
    for _ in range(2):
        for axis in (0, 1):
            acc = np.zeros_like(h)
            for k in range(-n, n + 1):
                acc += np.roll(h, k, axis=axis)
            h = acc / (2 * n + 1)
    return h


def build():
    """The heightfield as the simulation will read it: (west, south) in
    metres, and the lift a cell, metres."""
    v, tris = placed()
    lo, top = heightfield(v, tris)
    lift = np.maximum(top - SINK, 0)
    # Room round it for the slopes to settle into.
    pad = int(np.ceil(lift.max() / SETTLE / CELL)) + 2
    lift = np.pad(lift, pad)
    lo = lo - pad * CELL
    return lo, soften(settle(lift, SETTLE), SOFTEN)


def main():
    lo, lift = build()
    nz, nx = lift.shape
    units = np.clip(np.round(lift * HEIGHT_SCALE), 0, 65535).astype("<u2")
    header = struct.pack(
        "<6i",
        int(round(lo[0] * 100)),
        int(round(lo[1] * 100)),
        int(round(CELL * 100)),
        nx,
        nz,
        HEIGHT_SCALE,
    )
    OUT.write_bytes(header + units.tobytes())
    print(f"{OUT.relative_to(ROOT)}: {nx} x {nz} cells of {CELL} m from ({lo[0]:.0f}, {lo[1]:.0f}), highest {lift.max():.1f} m")


if __name__ == "__main__":
    main()
