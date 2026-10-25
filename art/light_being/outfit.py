"""What the light being wears.

A solar seraph (the owner's pick, 2026-10-25): a white-gold cuirass with
broad plates over the shoulders, over a long white robe that falls to the ankles in four panels, gold vambraces and
greaves, a high gold collar; for a face, a serene mask of gold; behind the
head a sunburst -- a ring and a fan of rays -- that burns. Hair swept back
under the mask's rim, pale gold. Read from across the arena: a tall white
figure, gold at the chest, arms and shins, and a sun behind its head.
"""

import bpy
import numpy as np

import character
import garment as g
from wardrobe import (boots, coat_tails, collar, covered, hair, hair_cards, jerkin, mat, piece, rigid,
                      sleeves, trousers, vambraces, w)


def glowing(ob):
    grp = ob.vertex_groups.new(name="edge")
    grp.add(list(range(len(ob.data.vertices))), 1.0, "REPLACE")
    return ob


def cuirass(body, r):
    return jerkin(body, r, material="gold", gap=0.016, low=0.05, collar=0.025, name="cuirass",
                  faces=2400, close=0.07, sleeved=False, trim=0.02, smooth=10)


def pauldrons(body, r):
    # A plate over each shoulder, standing well off it, so the shoulders read
    # broad and armoured rather than as a top tight to the body.
    out = []
    for s in "lr":
        top = r.o[f"arm.{s}"][1]

        def keep(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}") > 0.35) & (c[:, 1] > top - 0.09)

        out.append(piece(body, f"pauldron.{s}", 0.032, keep, smooth=6, close=0.03, material="gold",
                         thick=0.007, faces=600, trim=0.012))
    return out


def robe(body, r):
    # Four long panels from the waist to the ankles, the front pair split
    # so the legs can stride.
    return coat_tails(body, r, spans=[(14, 92, 0.10), (98, 176, 0.06), (184, 262, 0.06), (268, 346, 0.10)],
                      material="robe", flare=0.40, at=0.0, ripple=0.14)


def legs(body, r):
    return trousers(body, r, material="robe", hem=0.30, gap=0.008, name="leggings")


def arms(body, r):
    return sleeves(body, r, material="robe", to=0.45, gap=0.006, close=0.01, name="sleeve", faces=700)


def bracers(body, r):
    return vambraces(body, r, material="gold", strap="gold_dark", reach=0.15, straps_at=(0.5,))


def greaves(body, r):
    return boots(body, r, material="gold", cuff="gold_dark", strap="gold_dark", top=0.02, straps_at=(),
                 cuffed=False, gap=0.009)


def neck(body, r):
    return collar(body, r, material="gold", height=0.09, gap=0.012)


def mask(body, r):
    e = r.eye

    def keep(c, fw):
        rel = c - e
        return ((w(fw, "head") > 0.5) & (rel[:, 2] > -0.035) & (rel[:, 1] < 0.065) & (rel[:, 1] > -0.115))

    return [piece(body, "mask", 0.006, keep, smooth=5, close=0.02, material="mask", thick=0.004,
                  faces=900, trim=0.006)]


def head_hair(body, r):
    scalp = hair(body, r, gap=0.004, front=0.06, nape=-0.10, side=-0.06, grooves=12, depth=0.003,
                 faces=2200, lift=0.006, material="scalp")
    return [scalp, hair_cards(scalp, r, style="back", length=0.07, width=0.022,
                              layers=(0.003, 0.008, 0.012), counts=(45, 60, 70), seed=21)]


def _add(ob, name, material, joint):
    ob.name = name
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.shade_smooth()
    g.material(ob, mat(material))
    return glowing(rigid(ob, joint))


def sunburst(body, r):
    """A ring behind the head, standing square to the facing, and a fan of
    rays round it, long and short in turn: the sun the being wears."""
    e = r.eye
    centre = e + np.array([0.0, 0.07, -0.16])
    out = []
    bpy.ops.mesh.primitive_torus_add(major_radius=0.17, minor_radius=0.011, major_segments=64,
                                     minor_segments=8, location=tuple(g.g2b(centre)),
                                     rotation=(np.pi / 2, 0.0, 0.0))
    out.append(_add(bpy.context.active_object, "halo", "halo", "head"))
    rays = 16
    for i in range(rays):
        a = i / rays * 2 * np.pi
        long = 0.20 if i % 2 == 0 else 0.12
        d = np.array([np.sin(a), np.cos(a), 0.0])
        base = centre + d * 0.19
        mid = base + d * long / 2
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.016 if i % 2 == 0 else 0.011, radius2=0.0,
                                        depth=long, location=tuple(g.g2b(mid)))
        ob = bpy.context.active_object
        # A cone points up Blender's z; turn it out along its ray.
        from mathutils import Vector
        ob.rotation_mode = "QUATERNION"
        ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(tuple(g.g2b(d))))
        out.append(_add(ob, f"ray{i}", "halo", "head"))
    return out


GARMENTS = {
    "greaves": greaves, "legs": legs, "arms": arms, "cuirass": cuirass, "pauldrons": pauldrons,
    "robe": robe, "bracers": bracers,
    "neck": neck, "mask": mask, "hair": head_hair, "sunburst": sunburst,
}
WARDROBE = list(GARMENTS)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.02)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide)
