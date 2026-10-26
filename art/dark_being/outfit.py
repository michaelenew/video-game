"""What the dark being wears: a viking's kit, reworked, in iron.

The owner's reference, 2026-10-26, "rejiggered": a warrior bare to the
waist so the obsidian body shows everywhere, and on it pauldrons of three
lapped iron plates over each shoulder; a heavy chain hung round the neck; a
broad belt of black hide with an iron boss; a kilt of hide strips to the
knee; and **metal bands that look like metal** (``hard``) -- a riveted
bracer on each forearm with a forged cuff at the wrist, and two forged bands
round each upper arm. Bald, the face bare. No horns: the bands carry it.

And **rifts** (``RIFTS``): places where the obsidian has shattered and light
is coming out -- a shatter pattern painted round each impact (``build.veins``,
reading ``EXTRAS["rifts"]``) and shafts of light out of it, which the game
draws from pairs of markers (``beam.<n>.from`` and ``.to``).
"""

import bpy
import numpy as np

import character
import garment as g
import hard
import plate
from wardrobe import EXTRAS, J, belt, coat_tails, mat, rigid, stone, w

# Where the obsidian has broken, as (joint, offset from it in the game's
# frame, metres, radius): the left breast, the right shoulder, the left
# upper back, the right forearm. Each is moved onto the skin.
RIFTS = [
    ("chest", (-0.085, 0.04, 0.12), 0.13),
    ("arm.l", (0.03, 0.02, 0.05), 0.09),
    ("chest", (0.07, 0.07, -0.12), 0.11),
    ("shin.r", (0.0, 0.10, 0.06), 0.07),
]


def irons(body, r):
    """**Metal that reads as metal** (the owner, 2026-10-26): forged bands,
    not shells. On each forearm a wide bracer -- a plate round the forearm
    with a rolled rim at both ends and a band of studs -- and on each upper
    arm two narrow forged bands, riveted."""
    out = []
    for s in "lr":
        fa, hd, ua, el = f"forearm.{s}", f"hand.{s}", f"arm.{s}", f"forearm.{s}"
        mid = r.along(fa, hd, 0.62)
        up = r.dir(hd, fa)
        out += hard.plate(body, f"bracer.{s}", 0.012, lambda c, fw, fa=fa, hd=hd: w(fw, fa, hd) > 0.3, "iron",
                          trims=[(mid + up * 0.075, up)], keeps=[(mid - up * 0.075, up)], thick=0.006,
                          rim=0.006, smooth=4, close=0.02, faces=700)
        out += hard.band(body, r, fa, hd, 0.86, 0.026, 0.02, f"cuff.{s}", "iron", thick=0.009, studs=10)
        out += hard.band(body, r, ua, el, 0.38, 0.03, 0.006, f"armband_a.{s}", "iron", thick=0.008, studs=8)
        out += hard.band(body, r, ua, el, 0.52, 0.022, 0.006, f"armband_b.{s}", "iron", thick=0.007)
    return out


def pauldrons(body, r):
    """Over each shoulder a dome of iron, laid close (``plate``), open toward
    the neck, with two lames below it down the arm; every edge rolled, a
    rivet at the front and back of each."""
    out = []
    W = g.weights_of(body)
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        sh = r.o[arm]
        down = r.dir(arm, el)
        ax = plate.Axis(sh + np.array([0.0, 0.015, 0.0]), down, [0, 1, 0], [0, 0, 1])
        near = (W[:, J[arm]] + W[:, J[f"clavicle.{s}"]]) > 0.3
        us = np.linspace(-140, 140, 47)
        vs = np.linspace(-0.08, 0.12, 22)
        R = plate.radii(body, ax, vs, us, keep=near, smooth_u=5, smooth_v=3, stiff=8)
        a = np.abs(us)
        lo = -0.075 + 0.045 * plate.smoothstep(90, 140, a)
        hi = np.full(len(us), 0.06) - 0.015 * (a / 140.0) ** 2
        cap, borders = plate.surface(f"pauldron.{s}", ax, us, vs, R + 0.03, lo, hi)
        out += plate.finish(cap, borders, "iron", 0.008, 0.008, joint=arm)
        for k in range(2):
            top, bot = 0.05 + k * 0.04, 0.095 + k * 0.04
            vv = np.linspace(top - 0.01, bot + 0.01, 8)
            rk = plate.radii(body, ax, vv, us, keep=near, smooth_u=5, smooth_v=2, stiff=8)
            lame, lb = plate.surface(f"lame{k}.{s}", ax, us, vv, rk + 0.026 - 0.004 * k, np.full(len(us), top),
                                     np.full(len(us), bot))
            out += plate.finish(lame, lb, "iron", 0.006, 0.006, joint=arm)
        for u in (-70.0, 70.0):
            v = 0.0
            row = int(np.searchsorted(vs, v))
            j = int(np.argmin(np.abs(us - u)))
            pt = ax.point(np.array(u), np.array(v), np.array(R[row, j] + 0.038))
            out += hard.rivets(f"pauldron_rivet{int(u)}.{s}", [pt], [ax.dir(np.array(u))], 0.009, "iron",
                               joint=arm)
    return out


def chain(body, r):
    """A heavy chain hung round the neck, lying on the chest and the back:
    rings of iron along a loop, each turned a quarter from the last."""
    neck = r.o["neck"]
    chest = r.o["chest"]
    out = []
    links = 26
    for i in range(links):
        a = i / links * 2 * np.pi
        # A loop round the base of the neck, dropping lower at the front.
        drop = 0.07 * (0.5 + 0.5 * np.cos(a))
        at = np.array([np.sin(a) * 0.115, neck[1] - 0.03 - drop, chest[2] + np.cos(a) * 0.10])
        nxt = (i + 1) / links * 2 * np.pi
        d = np.array([np.cos(nxt), 0.0, -np.sin(nxt)])
        bpy.ops.mesh.primitive_torus_add(major_radius=0.012, minor_radius=0.0035, major_segments=10,
                                         minor_segments=6, location=(0, 0, 0))
        ob = bpy.context.active_object
        from mathutils import Vector, Quaternion
        q = Vector((1, 0, 0)).rotation_difference(Vector(tuple(g.g2b(d))))
        if i % 2:
            q = q @ Quaternion((1, 0, 0), np.pi / 2)
        ob.rotation_mode = "QUATERNION"
        ob.rotation_quaternion = q
        ob.location = tuple(g.g2b(at))
        ob.name = f"link{i}"
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        g.material(ob, mat("iron"))
        out.append(rigid(ob, "chest"))
    return out


def waist(body, r):
    """A broad belt of black hide, iron plaques riveted round it, and a
    round iron boss at the front with a raised rim."""
    pieces = belt(body, r, material="hide", sashed=False, at=0.03)
    hip = r.o["thigh.l"][1] + 0.05
    boss = stone("boss", (0.0, hip, 0.13), (0.05, 0.05, 0.016), "iron", "root", seed=2, facets=2, rough=0.05)
    ring = []
    for k in range(48):
        a = k / 48 * 2 * np.pi
        ring.append([np.cos(a) * 0.052, hip + np.sin(a) * 0.052, 0.138])
    rim = rigid(hard.tube("boss_rim", np.array(ring), 0.007, "iron", closed=True), "root")
    return pieces + [boss, rim]


def kilt(body, r):
    """A kilt of black hide in four broad panels, to the knee (the coat's rig
    swings four)."""
    return coat_tails(body, r, spans=[(-40, 40, 0.50), (48, 132, 0.52), (140, 220, 0.50), (228, 312, 0.52)],
                      material="kilt", flare=0.24, at=0.0, ripple=0.12)


def rifts(body, r):
    """Each rift moved onto the skin, and its shafts of light as markers on
    the joint that carries it: three or four, out of the crack along the
    skin's facing, fanned."""
    co = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("co", co)
    p = g.b2g(co.reshape(-1, 3))
    nb = np.zeros(len(body.data.vertices) * 3)
    body.data.vertices.foreach_get("normal", nb)
    nrm = g.b2g(nb.reshape(-1, 3))
    W = g.weights_of(body)
    rnd = np.random.default_rng(7)
    EXTRAS["rifts"] = []
    for i, (joint, off, radius) in enumerate(RIFTS):
        guess = r.o[joint] + np.array(off)
        k = int(np.argmin(np.linalg.norm(p - guess, axis=1)))
        centre, n = p[k], nrm[k] / max(np.linalg.norm(nrm[k]), 1e-6)
        carrier = g.JOINTS[int(np.argmax(W[k, :len(g.JOINTS)]))]
        EXTRAS["rifts"].append([centre.tolist(), radius])
        t1 = np.cross(n, [0.0, 1.0, 0.0])
        t1 /= max(np.linalg.norm(t1), 1e-6)
        t2 = np.cross(n, t1)
        for b in range(2 + i % 2):
            a = rnd.uniform(0, 2 * np.pi)
            spread = rnd.uniform(0.25, 0.6)
            d = n + spread * (np.cos(a) * t1 + np.sin(a) * t2)
            d /= np.linalg.norm(d)
            jitter = (np.cos(a) * t1 + np.sin(a) * t2) * radius * 0.25 * rnd.uniform()
            start = centre + jitter + n * 0.004
            end = start + d * radius * rnd.uniform(1.8, 2.8)
            EXTRAS["markers"].append((f"beam.{i}{b}.from", carrier, start.tolist()))
            EXTRAS["markers"].append((f"beam.{i}{b}.to", carrier, end.tolist()))
    return []


def eyes_dark(body, r):
    """Eyes that are the rifts' light, faintly."""
    from wardrobe import eyes
    out = eyes(body, r, iris="eye")
    for ob in out:
        grp = ob.vertex_groups.new(name="edge")
        grp.add(list(range(len(ob.data.vertices))), 0.5, "REPLACE")
    return out


GARMENTS = {"irons": irons, "pauldrons": pauldrons, "chain": chain, "waist": waist, "kilt": kilt,
            "eyes": eyes_dark, "rifts": rifts}
WARDROBE = list(GARMENTS)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, skin_material="skin")
