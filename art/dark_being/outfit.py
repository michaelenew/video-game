"""What the dark being wears: nothing. It is obsidian.

A void titan (the owner's pick, 2026-10-25): a bare body of black volcanic
glass, cut into facets -- broad flat planes that each catch the light whole,
as knapped stone does, so it reads as carved and not as skin -- with no
face, the head one smooth shape faceted like the rest, and violet light
breaking out of it: a heart burning in the chest, and cracks running out
from it across the body (``build.veins``), thick near the heart and thinning
to hairlines at the hands.
"""

import bpy
import numpy as np

import character
from wardrobe import piece, stone, w

# How many triangles the facets leave: few enough that each plane is a
# hand's breadth, which is what reads as cut stone at the being's size.
BODY_FACETS = 2800
HEAD_FACETS = 260


def faceless(body, r):
    # The head and neck inflated a little and smoothed hard: a skull with
    # no face on it.
    return [piece(body, "head_stone", 0.007, lambda c, fw: w(fw, "head", "neck") > 0.45,
                  smooth=14, close=0.04, material="stone", thick=0.0, faces=1600, trim=0.0)]


def heart(body, r):
    chest = r.o["chest"]
    at = chest + np.array([0.0, 0.07, 0.125])
    core = stone("heart", at, (0.06, 0.08, 0.03), "heart", "chest", seed=3, facets=0, rough=0.9)
    grp = core.vertex_groups.new(name="edge")
    grp.add(list(range(len(core.data.vertices))), 1.0, "REPLACE")
    return [core]


GARMENTS = {"head": faceless, "heart": heart}
WARDROBE = list(GARMENTS)


def hide(r, c, fw):
    # The face under the smooth head, so nothing of it can show through.
    return w(fw, "head") > 0.6


def facet(ob, triangles):
    """Cut ``ob`` into flat planes: collapse it to ``triangles`` and shade
    every face flat. The joint weights survive the collapse."""
    n = sum(len(p.vertices) - 2 for p in ob.data.polygons)
    dec = ob.modifiers.new("facets", "DECIMATE")
    dec.decimate_type = "COLLAPSE"
    dec.ratio = min(1.0, triangles / max(n, 1))
    with bpy.context.temp_override(object=ob, active_object=ob):
        bpy.ops.object.modifier_apply(modifier=dec.name)
    ob.data.shade_flat()


def dress(body, r, only=None):
    pieces = character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, skin_material="stone")
    facet(body, BODY_FACETS)
    for p in pieces:
        if p.name == "head_stone":
            facet(p, HEAD_FACETS)
    # The smooth high piece would bake its roundness back over the facets
    # as a normal map: there is none.
    for o in list(bpy.context.scene.objects):
        if o.get("hi"):
            bpy.data.objects.remove(o)
    return pieces
