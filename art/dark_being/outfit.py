"""What the dark being wears: a viking's kit, reworked, in iron.

The owner's reference, 2026-10-26, matched piece by piece: a warrior bare
to the waist so the obsidian body shows everywhere; on top of each shoulder
a small spiked plate of iron fanned out past it, a wider one under it, the
deltoid bare (``facet``); two heavy chains slung across the chest from
shoulder to shoulder; a broad belt of black hide with an iron boss and a
spike; a kilt of hide panels past the knee; and **metal bands that look
like metal** -- a long faceted bracer on each forearm, a studded band round
its middle, a forged cuff at the wrist (``hard``). The upper arm bare. Bald,
the face bare. No horns.

No cracks yet (the owner, 2026-10-26): they come back from a script that
works out a fracture pattern, angular as broken glass is, with light shining
out of it as a blacklight would.
"""

import bpy
import numpy as np

import character
import facet
import garment as g
import hard
import plate
from wardrobe import J, belt, coat_tails, mat, rigid, stone, w

def irons(body, r):
    """**Metal that reads as metal** (the owner, 2026-10-26), after the
    reference: on each forearm a long bracer of eight flat faces from the
    wrist most of the way to the elbow, its ends framed, a raised band
    studded round its middle, and a forged cuff at the wrist; the upper arm
    left bare, as the reference's is."""
    out = []
    for s in "lr":
        fa, hd, ua = f"forearm.{s}", f"hand.{s}", f"arm.{s}"
        lateral = facet.rest_frame(r, ua, fa).m[0]
        fore = facet.bone_frame(r, fa, hd, lateral)
        L = float(np.linalg.norm(r.o[hd] - r.o[fa]))
        ring = tuple(float(p) for p in range(0, 361, 45))
        # A face is cos 22.5 of its corners: the corners stand that far out.
        # Long, as the reference's, and close: each ring sized to the
        # forearm where it is, so the bracer follows its taper.
        rad = [facet.limb_radius(body, r, fa, hd, t, 0.007) / 0.92 for t in (0.3, 0.6, 0.9)]
        rad[2] = max(rad[2], 0.8 * rad[0])
        rows = [[(p, h, q) for p in ring] for h, q in ((-0.28 * L, rad[0]), (-0.6 * L, rad[1]), (-0.92 * L, rad[2]))]
        inside = fore.o - fore.m[1] * 0.6 * L
        out.append(facet.plate(f"bracer.{s}", facet.round_limb(fore, rows), frame=0.012, trim="iron",
                               material="iron", joint=fa, thick=0.007, edges=(True, True, False, False),
                               inside=inside))
        rows = [[(p, h, rad[1] + 0.008) for p in ring] for h in (-0.53 * L, -0.6 * L)]
        out.append(facet.plate(f"bracer_band.{s}", facet.round_limb(fore, rows), frame=0.0, trim="iron",
                               material="iron", joint=fa, thick=0.006, edges=(False,) * 4, inside=inside))
        pts = facet.round_limb(fore, [[(p + 22.5, -0.565 * L, (rad[1] + 0.014) * 0.92) for p in ring[:-1]]])[0]
        nrm = pts - (fore.o - fore.m[1] * 0.565 * L)
        out += hard.rivets(f"bracer_stud.{s}", pts, nrm, 0.006, "iron", joint=fa)
        out += hard.band(body, r, fa, hd, 0.93, 0.024, 0.02, f"cuff.{s}", "iron", thick=0.009, studs=10)
    return out


def pauldrons(body, r):
    """After the reference: no dome. On top of each shoulder a small plate
    of iron, flat-faced, fanned out past the shoulder and dipping front and
    back, a spike standing up from its inner end by the neck, its outer edge
    cut to hanging points; under it a second, wider and lower, cut the same
    way. The deltoid below is bare."""
    out = []
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        fr = facet.rest_frame(r, arm, el)
        # The joint is the middle of the deltoid's ball: its top and its
        # outside stand the arm's own radius from it. (Measured in the bind
        # pose instead, the trunk's skin turned into the search, 2026-10-26.)
        q = facet.limb_radius(body, r, arm, el, 0.12)
        top = 0.85 * q + 0.022
        side = q + 0.015
        zs = (0.11, 0.055, 0.0, -0.055, -0.11)

        def fan(xs, ys, drop, spike, cut, zk):
            rows = []
            for z in zs:
                f = z / 0.11
                row = []
                for c, (x, y) in enumerate(zip(xs, ys)):
                    y = y - drop * f * f
                    if c == 0 and z == 0.0:
                        y += spike
                    if c == len(xs) - 1 and abs(z) == 0.055:
                        y -= cut
                    row.append((x, y, z * zk))
                rows.append(row)
            return facet.points(fr, rows)

        out.append(facet.plate(f"pauldron.{s}", fan((-0.05, 0.02, 0.08, side + 0.03),
                                                    (top + 0.012, top + 0.02, top - 0.005, 0.03), 0.045, 0.075,
                                                    0.035, 1.0),
                               frame=0.011, material="iron", trim="iron", joint=arm, thick=0.008, bevel=0.003,
                               inside=fr.o))
        out.append(facet.plate(f"pauldron_wing.{s}", fan((0.04, 0.1, side + 0.065),
                                                         (top - 0.02, top - 0.04, -0.005), 0.04, 0.0, 0.04, 1.12),
                               frame=0.01, material="iron", trim="iron", joint=arm, thick=0.007, bevel=0.003,
                               inside=fr.o))
        for z in (0.06, -0.06):
            at = fr((0.02, top + 0.028 - 0.045 * (z / 0.11) ** 2, z))
            out += hard.rivets(f"pauldron_rivet{int(z * 100)}.{s}", [at], [fr.m[1]], 0.008, "iron", joint=arm)
    return out


def chain(body, r):
    """After the reference: two heavy chains slung across the chest from
    the front of one shoulder to the other, lying on it -- one to the
    middle of the pectorals, one shorter, just under the collarbones --
    rings of iron each turned a quarter from the last."""
    from mathutils import Quaternion, Vector
    p = plate.body_points(body)
    arm = r.o["arm.l"]
    out = []
    for k, (drop, link) in enumerate(((0.13, 0.013), (0.075, 0.011))):
        xs = np.linspace(-abs(arm[0]) + 0.045, abs(arm[0]) - 0.045, 60)
        t = (xs - xs[0]) / (xs[-1] - xs[0])
        ys = arm[1] + 0.035 - drop * np.sin(np.pi * t) ** 0.8
        line = []
        for x, y in zip(xs, ys):
            m = (np.abs(p[:, 0] - x) < 0.02) & (np.abs(p[:, 1] - y) < 0.02) & (p[:, 2] > 0)
            z = p[m, 2].max() if m.any() else arm[2] + 0.1
            line.append([x, y, z + link * 0.7 + 0.004 * k])
        line = hard._resample(np.array(line), link * 1.55, False)
        for i in range(len(line) - 1):
            d = line[i + 1] - line[i]
            d /= np.linalg.norm(d)
            bpy.ops.mesh.primitive_torus_add(major_radius=link, minor_radius=link * 0.3, major_segments=10,
                                             minor_segments=6, location=(0, 0, 0))
            ob = bpy.context.active_object
            q = Vector((1, 0, 0)).rotation_difference(Vector(tuple(g.g2b(d))))
            if i % 2:
                q = q @ Quaternion((1, 0, 0), np.pi / 2)
            ob.rotation_mode = "QUATERNION"
            ob.rotation_quaternion = q
            ob.location = tuple(g.g2b(line[i] + d * link * 0.78))
            ob.name = f"link{k}_{i}"
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
            g.material(ob, mat("iron"))
            out.append(ob)
    return [rigid(g.join(out, "chain"), "chest")]


def waist(body, r):
    """A broad belt of black hide, iron plaques riveted round it, and a
    round iron boss at the front with a raised rim and a spike."""
    pieces = belt(body, r, material="hide", sashed=False, at=0.03)
    hip = r.o["thigh.l"][1] + 0.05
    boss = stone("boss", (0.0, hip, 0.13), (0.05, 0.05, 0.016), "iron", "root", seed=2, facets=2, rough=0.05)
    ring = []
    for k in range(48):
        a = k / 48 * 2 * np.pi
        ring.append([np.cos(a) * 0.052, hip + np.sin(a) * 0.052, 0.138])
    rim = rigid(hard.tube("boss_rim", np.array(ring), 0.007, "iron", closed=True), "root")
    # The reference's boss ends in a spike.
    bpy.ops.mesh.primitive_cone_add(vertices=8, radius1=0.02, radius2=0.0, depth=0.045,
                                    location=tuple(g.g2b(np.array([0.0, hip, 0.165]))))
    spike = bpy.context.active_object
    spike.name = "boss_spike"
    spike.rotation_euler = (-np.pi / 2, 0.0, 0.0)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    g.material(spike, mat("iron"))
    return pieces + [boss, rim, rigid(spike, "root")]


def kilt(body, r):
    """A kilt of black hide in four broad panels, past the knee as the
    reference's hangs (the coat's rig
    swings four)."""
    return coat_tails(body, r, spans=[(-40, 40, 0.62), (48, 132, 0.60), (140, 220, 0.62), (228, 312, 0.60)],
                      material="kilt", flare=0.1, at=0.0, ripple=0.12)


def eyes_dark(body, r):
    """Eyes with a faint violet light in them."""
    from wardrobe import eyes
    out = eyes(body, r, iris="eye")
    for ob in out:
        grp = ob.vertex_groups.new(name="edge")
        grp.add(list(range(len(ob.data.vertices))), 0.5, "REPLACE")
    return out


GARMENTS = {"irons": irons, "pauldrons": pauldrons, "chain": chain, "waist": waist, "kilt": kilt,
            "eyes": eyes_dark}
WARDROBE = list(GARMENTS)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, skin_material="skin")
