"""What the light being wears: plate, made of light.

A celestial knight (the owner, 2026-10-26, from a reference "not far off,
but probably too heavy on the gold and a little too ornate -- this should be
a cool visual but not so dominant visually"): full plate in pale silver,
light running along the edges of every plate (``glowing_edges``); a closed helm with a crest and a
slit of light for its eyes; broad pointed plates over the shoulders; plate
down the arms to gauntlets; tassets over a pale skirt; greaves. Gold only at
the belt. A short half-cloak over the left shoulder. Nothing holy: no halo.
"""

import bpy
import numpy as np

import character
import forge
import garment as g
import stage
from wardrobe import (EXTRAS, LEFT, belt, body_masks, boots, cape, coat_tails, covered, jerkin, mat, piece,
                      rigid, sleeves, trousers, vambraces, w)


def facet(ob, triangles):
    """Cut a plate into planes: collapse it to ``triangles`` and shade every
    face flat, so it catches the light as cut crystal does."""
    n = sum(len(p.vertices) - 2 for p in ob.data.polygons)
    dec = ob.modifiers.new("facets", "DECIMATE")
    dec.decimate_type = "COLLAPSE"
    dec.ratio = min(1.0, triangles / max(n, 1))
    with bpy.context.temp_override(object=ob, active_object=ob):
        bpy.ops.object.modifier_apply(modifier=dec.name)
    ob.data.shade_flat()
    return ob


def glowing_edges(ob, power=6.0):
    """Light along a plate's edge: its ``hem`` (the open edge, falling off
    inward) copied to ``edge``, which is what glows, tightened to the rim."""
    hem = ob.vertex_groups.get("hem")
    if hem is None:
        return ob
    edge = ob.vertex_groups.get("edge") or ob.vertex_groups.new(name="edge")
    for v in ob.data.vertices:
        for ge in v.groups:
            if ge.group == hem.index:
                edge.add([v.index], ge.weight ** power, "REPLACE")
    return ob


# Plate is smooth: cut into big flat facets, each one either caught the light
# whole or did not, and in the game that read as white blotches on grey
# rather than as cut crystal (the first build).
FACETED = False


def plate(pieces, triangles=None):
    for p in pieces:
        if triangles and FACETED:
            facet(p, triangles)
            # Its smooth high piece would bake the roundness back over the
            # planes as a normal map.
            hi = bpy.data.objects.get(p.name + "~hi")
            if hi is not None:
                bpy.data.objects.remove(hi)
        glowing_edges(p)
    return pieces


def cuirass(body, r):
    return plate(jerkin(body, r, material="plate", gap=0.018, low=0.03, collar=0.04, name="cuirass",
                        faces=2400, close=0.08, sleeved=False, trim=0.03, smooth=10, inset=0.04), 900)


def pauldrons(body, r):
    out = []
    for s in "lr":
        top = r.o[f"arm.{s}"][1]
        arm, el = f"arm.{s}", f"forearm.{s}"

        def upper(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}") > 0.4) & (c[:, 1] > top - 0.08)

        out += plate([piece(body, f"pauldron.{s}", 0.034, upper, smooth=8, close=0.03, material="plate",
                            thick=0.008, faces=800, trim=0.02)], 160)
        mid = r.along(arm, el, 0.30)
        down = r.dir(arm, el)
        out += plate([piece(body, f"lame.{s}", 0.026, lambda c, fw, arm=arm: w(fw, arm) > 0.4,
                            trims=[(mid + down * 0.045, down)], keeps=[(mid - down * 0.04, down)], smooth=5,
                            close=0.02, material="plate", thick=0.006, faces=400, trim=0.012)], 90)
        # The point: a blade of plate swept up off the shoulder.
        sh = r.o[arm]
        side = np.sign(sh[0])
        root = sh + np.array([side * 0.03, 0.08, -0.01])
        sections = []
        steps = 8
        for i in range(steps + 1):
            t = i / steps
            c = root + np.array([side * 0.07 * t, 0.10 * t, -0.03 * t * t])
            wdt = 0.05 * (1 - t) + 0.002
            sections.append(forge.ring(c, np.array([0.0, 1.0, 0.0]), np.array([0.0, 0.0, 1.0]),
                                       np.array([1.0, 0.0, 0.0]), wdt, 0.006, sides=6))
        v, f = forge.loft(sections)
        fin = stage.mesh_object(f"fin.{s}", g.g2b(v), f, smooth=False)
        g.material(fin, mat("plate"))
        out.append(rigid(fin, f"clavicle.{s}"))
    return out


def arms(body, r):
    return (plate(sleeves(body, r, material="plate", to=0.08, gap=0.014, close=0.014, name="rerebrace",
                          faces=700), 220)
            + plate(vambraces(body, r, material="plate", strap="plate", reach=0.1, straps_at=()), 160))


def helm(body, r):
    """A closed helm over the head and neck, cut in planes, and a crest of
    plate from the brow back over the crown."""
    out = plate([piece(body, "helm", 0.022, lambda c, fw: w(fw, "head", "neck") > 0.45, smooth=12,
                       close=0.05, material="plate", thick=0.006, faces=1800, trim=0.0)], 420)
    e = r.eye
    head = r.o["head"]
    sections = []
    steps = 14
    for i in range(steps + 1):
        t = i / steps
        a = -0.5 + 2.2 * t
        c = head + np.array([0.0, 0.10, -0.01]) + np.array([0.0, np.cos(a) * 0.14, np.sin(a) * 0.13])
        tall = 0.035 * np.sin(np.pi * min(1.0, t * 1.3)) + 0.004
        out_dir = np.array([0.0, np.cos(a), np.sin(a)])
        along = np.array([0.0, -np.sin(a), np.cos(a)])
        sections.append(forge.ring(c + out_dir * tall * 0.5, along, out_dir, np.array([1.0, 0.0, 0.0]),
                                   tall * 0.6, 0.004, sides=6))
    v, f = forge.loft(sections)
    crest = stage.mesh_object("crest", g.g2b(v), f, smooth=False)
    g.material(crest, mat("plate"))
    out.append(rigid(crest, "head"))
    EXTRAS["visor"] = (e + np.array([0.0, 0.0, 0.02])).tolist()
    return out


def waist(body, r):
    return belt(body, r, material="gold", sashed=False, at=0.03)


def tassets(body, r):
    return plate(coat_tails(body, r, spans=[(18, 86, 0.66), (94, 160, 0.68), (200, 266, 0.68), (274, 342, 0.66)],
                            material="plate", flare=0.2, at=0.0, ripple=0.03))


def legs(body, r):
    return trousers(body, r, material="skirt", hem=0.30, gap=0.009, name="hose")


def greaves(body, r):
    return plate(boots(body, r, material="plate", cuff="plate", strap="plate", top=0.0, straps_at=(), cuffed=True,
                       gap=0.012), 300)


def cloak(body, r):
    return cape(body, r, side=LEFT, name="cloak", material="cloak", low=0.55, gap=0.06, faces=1600)


GARMENTS = {
    "greaves": greaves, "legs": legs, "tassets": tassets, "cuirass": cuirass, "pauldrons": pauldrons,
    "arms": arms, "waist": waist, "helm": helm, "cloak": cloak,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    body_masks(body, r, gloved_to=1.05, brow=0.0)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.0) | (w(fw, "head") > 0.5)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
