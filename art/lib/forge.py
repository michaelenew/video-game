"""Hard things, lofted: blades, guards, handles, plates.

A loft is a run of cross-sections along a line, each one a closed ring of
points, joined face to face. A blade is a loft of flattened diamonds that
narrow to a point; a handle a loft of circles that swell at each turn of its
wrap. Every shape here is written in its own frame -- ``u`` along it, ``v``
across its width, ``n`` through its thickness -- and placed by ``frame``.
"""

import numpy as np


def loft(sections, cap_start=True, cap_end=True):
    """``sections``: a list of ``(m, 3)`` rings, all the same ``m``.

    Returns ``(verts, faces)``; the ends are closed with a fan when asked.
    """
    m = len(sections[0])
    verts = np.concatenate(sections)
    faces = []
    for i in range(len(sections) - 1):
        a, b = i * m, (i + 1) * m
        for j in range(m):
            k = (j + 1) % m
            faces.append([a + j, a + k, b + k, b + j])
    if cap_start:
        c = len(verts)
        verts = np.vstack([verts, sections[0].mean(0)])
        for j in range(m):
            faces.append([c, (j + 1) % m, j])
    if cap_end:
        c = len(verts)
        verts = np.vstack([verts, sections[-1].mean(0)])
        base = (len(sections) - 1) * m
        for j in range(m):
            faces.append([c, base + j, base + (j + 1) % m])
    return verts, faces


def ring(center, u, v, n, rv, rn, sides=10, phase=0.0):
    """An ellipse round ``center`` in the plane across ``u``."""
    t = np.linspace(0, 2 * np.pi, sides, endpoint=False) + phase
    return center + np.outer(np.cos(t) * rv, v) + np.outer(np.sin(t) * rn, n)


def blade(length=0.30, width=0.042, thick=0.007, curve=0.018, steps=24):
    """A long, slightly recurved dagger's blade, from the guard to the point.

    The section is a flattened hexagon: a spine thickness in the middle, two
    bevels down to a keen edge on each side. ``curve`` sweeps the point
    toward the edge side, as a fighting knife's does.
    Returns verts, faces and, per vertex, how near the edge it is (0 at the
    spine, 1 on the edge) -- the painter's mask for the honed bevel.
    """
    secs = []
    edge = []
    for i in range(steps + 1):
        t = i / steps
        # Leaf profile: full a fifth of the way up, then a long taper.
        w = width * (0.82 + 0.5 * np.sin(np.pi * min(t / 0.45, 1.0) * 0.5)) * (1 - t ** 1.6)
        w = max(w, 0.0008)
        th = thick * (1 - 0.65 * t)
        c = curve * t * t
        u = t * length
        # Points: edge+, bevel+, spine+, edge-, bevel-, spine- (hexagon).
        sec = np.array([
            [u, c + w * 0.5, 0.0],
            [u, c + w * 0.18, th * 0.5],
            [u, c - w * 0.18, th * 0.5],
            [u, c - w * 0.5, 0.0],
            [u, c - w * 0.18, -th * 0.5],
            [u, c + w * 0.18, -th * 0.5],
        ])
        secs.append(sec)
        edge += [1, 0, 0, 1, 0, 0]
    v, f = loft(secs, cap_start=True, cap_end=True)
    edge += [0, 1]
    return v, f, np.array(edge, float)


def handle(length=0.10, radius=0.0135, turns=7, steps=36):
    """A grip wound with cord: a circle that swells at each turn."""
    secs = []
    for i in range(steps + 1):
        t = i / steps
        swell = 0.0012 * (0.5 + 0.5 * np.cos(t * turns * 2 * np.pi))
        taper = 1.0 - 0.08 * np.sin(np.pi * t)
        r = radius * taper + swell
        secs.append(ring(np.array([-t * length, 0, 0]), None, np.array([0, 1.0, 0]),
                         np.array([0, 0, 1.0]), r * 1.1, r * 0.9, sides=10))
    return loft(secs)


def guard(span=0.085, depth=0.016, thick=0.012, steps=16):
    """A crossguard swept back toward the hand at both ends."""
    secs = []
    for i in range(steps + 1):
        t = i / steps * 2 - 1
        y = t * span * 0.5
        x = -0.012 * t * t - 0.004
        r = (0.5 + 0.5 * (1 - abs(t))) * 0.5
        secs.append(ring(np.array([x, y, 0]), None, np.array([1.0, 0, 0]),
                         np.array([0, 0, 1.0]), depth * r + 0.002, thick * r + 0.002, sides=8))
    return loft(secs)


def pommel(at=-0.10, radius=0.016):
    secs = []
    for i in range(9):
        t = i / 8
        r = radius * np.sin(np.pi * max(t, 0.02)) ** 0.7
        secs.append(ring(np.array([at - t * radius * 1.6, 0, 0]), None, np.array([0, 1.0, 0]),
                         np.array([0, 0, 1.0]), max(r, 0.0006), max(r, 0.0006) * 0.85, sides=10))
    return loft(secs)


def frame(points, origin, u, v):
    """Carry points written in (u, v, n) into a frame at ``origin``."""
    u = u / np.linalg.norm(u)
    v = v - u * np.dot(v, u)
    v = v / np.linalg.norm(v)
    n = np.cross(u, v)
    return origin + points[:, 0:1] * u + points[:, 1:2] * v + points[:, 2:3] * n
