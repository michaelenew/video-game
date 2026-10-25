"""What the light being wears.

An angel-knight (the owner, 2026-10-25): a polished steel cuirass, sculpted
and standing off the body as plate does; layered plates over each shoulder;
the upper arms bare, so their strength shows; steel vambraces and gauntlets;
a gold belt; steel tassets over a white skirt; steel greaves. The angel is
in a white half-cloak over the left shoulder only, falling down that side of
the back -- part of a cloak, so the body is still seen from behind -- and a
ring of light behind the head. Short hair, the face bare and calm.
"""

import bpy
import numpy as np

import character
import garment as g
from wardrobe import (LEFT, belt, boots, cape, coat_tails, collar, covered, eyes, hair, hair_cards, jerkin,
                      mat, piece, rigid, trousers, vambraces, w, body_masks)


def glowing(ob):
    grp = ob.vertex_groups.new(name="edge")
    grp.add(list(range(len(ob.data.vertices))), 1.0, "REPLACE")
    return ob


def cuirass(body, r):
    return jerkin(body, r, material="steel", gap=0.024, low=0.02, collar=0.035, name="cuirass",
                  faces=2600, close=0.09, sleeved=False, trim=0.025, smooth=12, inset=0.045)


def pauldrons(body, r):
    """Two lames a shoulder: a great round plate over the top of the
    shoulder standing well off it, and a narrower one below it on the arm."""
    out = []
    for s in "lr":
        top = r.o[f"arm.{s}"][1]
        arm, el = f"arm.{s}", f"forearm.{s}"

        def upper(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}") > 0.35) & (c[:, 1] > top - 0.10)

        out.append(piece(body, f"pauldron.{s}", 0.045, upper, smooth=8, close=0.04, material="steel",
                         thick=0.008, faces=700, trim=0.015))
        mid = r.along(arm, el, 0.28)
        down = r.dir(arm, el)
        out.append(piece(body, f"lame.{s}", 0.034, lambda c, fw, arm=arm: w(fw, arm) > 0.4,
                         trims=[(mid + down * 0.05, down)], keeps=[(mid - down * 0.04, down)], smooth=5,
                         close=0.02, material="steel", thick=0.006, faces=400, trim=0.01))
    return out


def arms(body, r):
    return vambraces(body, r, material="steel", strap="gold", reach=0.12, straps_at=(0.25, 0.9))


def gorget(body, r):
    return collar(body, r, material="steel", height=0.06, gap=0.02)


def waist(body, r):
    return belt(body, r, material="gold", sashed=False, at=0.03)


def tassets(body, r):
    # Steel plates over the hips to mid-thigh, open at the front and back.
    return coat_tails(body, r, spans=[(20, 88, 0.62), (92, 160, 0.64), (200, 268, 0.64), (272, 340, 0.62)],
                      material="steel", flare=0.22, at=0.0, ripple=0.04)


def legs(body, r):
    return trousers(body, r, material="skirt", hem=0.30, gap=0.009, name="hose")


def greaves(body, r):
    return boots(body, r, material="steel", cuff="gold", strap="gold", top=0.0, straps_at=(), cuffed=True,
                 gap=0.011)


def cloak(body, r):
    return cape(body, r, side=LEFT, name="cloak", material="cloak", low=0.72, gap=0.05, faces=1800)


def head_hair(body, r):
    scalp = hair(body, r, gap=0.004, front=0.05, nape=-0.07, side=-0.075, grooves=10, depth=0.003,
                 faces=2200, lift=0.004, material="scalp")
    return [scalp, hair_cards(scalp, r, style="back", length=0.05, width=0.022,
                              layers=(0.003, 0.007, 0.011), counts=(45, 60, 70), seed=31)]


def halo(body, r):
    e = r.eye
    centre = e + np.array([0.0, 0.10, -0.14])
    bpy.ops.mesh.primitive_torus_add(major_radius=0.16, minor_radius=0.012, major_segments=64,
                                     minor_segments=10, location=tuple(g.g2b(centre)),
                                     rotation=(np.pi / 2, 0.0, 0.0))
    ob = bpy.context.active_object
    ob.name = "halo"
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.shade_smooth()
    g.material(ob, mat("halo"))
    return [glowing(rigid(ob, "head"))]


GARMENTS = {
    "greaves": greaves, "legs": legs, "tassets": tassets, "cuirass": cuirass, "pauldrons": pauldrons,
    "arms": arms, "waist": waist, "neck": gorget, "cloak": cloak, "hair": head_hair, "eyes": eyes,
    "halo": halo,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    body_masks(body, r, gloved_to=1.05, brow=0.6)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.0, sleeves=False)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
