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
from wardrobe import EXTRAS, belt, coat_tails, mat, rigid, stone, w

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
    """Three plates over each shoulder, each lapping the one below: a cap
    over the top of the shoulder and two lames down the arm, each with a
    rolled rim and a rivet at its front and back."""
    out = []
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        top = r.o[arm][1]
        down = r.dir(arm, el)

        def cap(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}") > 0.4) & (c[:, 1] > top - 0.035)

        out += hard.plate(body, f"pauldron.{s}", 0.03, cap, "iron", thick=0.007, rim=0.007, smooth=10,
                          close=0.03, faces=900)
        for k, (at, gap) in enumerate(((0.13, 0.024), (0.24, 0.018))):
            mid = r.along(arm, el, at)
            out += hard.plate(body, f"lame{k}.{s}", gap, lambda c, fw, arm=arm: w(fw, arm) > 0.4, "iron",
                              trims=[(mid + down * 0.04, down)], keeps=[(mid - down * 0.035, down)],
                              thick=0.006, rim=0.005, smooth=6, close=0.02, faces=500)
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
    """A kilt of black hide in eight strips, to the knee."""
    return coat_tails(body, r, spans=[(a - 20, a + 20, 0.50) for a in range(0, 360, 45)],
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
