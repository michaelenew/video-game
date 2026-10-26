"""What the dark being wears: a viking's kit, reworked, in iron.

The owner's reference, 2026-10-26, "rejiggered": a warrior bare to the
waist so the obsidian body shows everywhere, and on it small iron plates
over the shoulders with a spike each; a heavy chain hung round the neck; a
broad iron belt with a round boss; a dark kilt to the knee; and **metal
bands that look like metal** -- a wide bracer round each forearm with a band
at each end, and two bands round each upper arm. Bald, the face bare. No
horns: the bands carry it.

And **rifts** (``RIFTS``): places where the obsidian has shattered and light
is coming out -- a shatter pattern painted round each impact (``build.veins``,
reading ``EXTRAS["rifts"]``) and shafts of light out of it, which the game
draws from pairs of markers (``beam.<n>.from`` and ``.to``).
"""

import bpy
import numpy as np

import character
import garment as g
from wardrobe import EXTRAS, belt, coat_tails, mat, piece, rigid, stone, w

# Where the obsidian has broken, as (joint, offset from it in the game's
# frame, metres, radius): the left breast, the right shoulder, the left
# upper back, the right forearm. Each is moved onto the skin.
RIFTS = [
    ("chest", (-0.085, 0.04, 0.12), 0.13),
    ("arm.l", (0.03, 0.02, 0.05), 0.09),
    ("chest", (0.07, 0.07, -0.12), 0.11),
    ("forearm.r", (0.0, -0.10, 0.04), 0.07),
]


def metal_band(body, r, a, b, at, width, gap, name, thick=0.009):
    """A band of iron round a limb: thick, so its edges catch the light as
    metal does, square to the bone."""
    up = r.dir(b, a)
    mid = r.along(a, b, at)
    return piece(body, name, gap, lambda c, fw: w(fw, a, b) > 0.3,
                 trims=[(mid + up * width / 2, up)], keeps=[(mid - up * width / 2, up)],
                 material="iron", thick=thick, faces=360, trim=0.004, smooth=3)


def irons(body, r):
    out = []
    for s in "lr":
        fa, hd, ua, el = f"forearm.{s}", f"hand.{s}", f"arm.{s}", f"forearm.{s}"
        out.append(metal_band(body, r, fa, hd, 0.62, 0.13, 0.012, f"bracer.{s}", thick=0.007))
        out.append(metal_band(body, r, fa, hd, 0.52, 0.022, 0.02, f"bracer_top.{s}"))
        out.append(metal_band(body, r, fa, hd, 0.84, 0.022, 0.02, f"bracer_end.{s}"))
        out.append(metal_band(body, r, ua, el, 0.40, 0.028, 0.012, f"armband_a.{s}"))
        out.append(metal_band(body, r, ua, el, 0.52, 0.028, 0.012, f"armband_b.{s}"))
    return out


def pauldrons(body, r):
    out = []
    for s in "lr":
        top = r.o[f"arm.{s}"][1]

        def keep(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}") > 0.45) & (c[:, 1] > top - 0.05)

        out.append(piece(body, f"pauldron.{s}", 0.024, keep, smooth=6, close=0.02, material="iron",
                         thick=0.008, faces=500, trim=0.01))
        # A spike off the top of each, up and out.
        sh = r.o[f"arm.{s}"]
        side = np.sign(sh[0])
        base = sh + np.array([side * 0.01, 0.07, -0.01])
        tip = base + np.array([side * 0.06, 0.11, -0.02])
        bpy.ops.mesh.primitive_cone_add(vertices=8, radius1=0.022, radius2=0.0, depth=1.0,
                                        location=(0, 0, 0))
        cone = bpy.context.active_object
        from mathutils import Vector
        d = tip - base
        cone.scale = (1.0, 1.0, float(np.linalg.norm(d)))
        cone.rotation_mode = "QUATERNION"
        cone.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(tuple(g.g2b(d))))
        cone.location = tuple(g.g2b((base + tip) / 2))
        cone.name = f"spike.{s}"
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        g.material(cone, mat("iron"))
        out.append(rigid(cone, f"clavicle.{s}"))
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
    pieces = belt(body, r, material="iron", sashed=False, at=0.03)
    hip = r.o["thigh.l"][1] + 0.05
    boss = stone("boss", (0.0, hip, 0.125), (0.045, 0.045, 0.014), "iron", "root", seed=2, facets=1, rough=0.1)
    return pieces + [boss]


def kilt(body, r):
    return coat_tails(body, r, spans=[(-40, 40, 0.48), (44, 136, 0.50), (140, 220, 0.48), (224, 316, 0.50)],
                      material="kilt", flare=0.28, at=0.0, ripple=0.2)


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
    from wardrobe import eyes
    return eyes(body, r, iris="eye")


GARMENTS = {"irons": irons, "pauldrons": pauldrons, "chain": chain, "waist": waist, "kilt": kilt,
            "eyes": eyes_dark, "rifts": rifts}
WARDROBE = list(GARMENTS)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, skin_material="skin")
