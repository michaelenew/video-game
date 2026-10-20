"""Painting a model's textures from what the model knows about itself.

A hand-painted game texture -- Guild Wars 2's, World of Warcraft's -- is not a
photograph of a material. It is a painter's decisions about light, laid into
the colour: folds and undersides fall into a cool shade, edges that face the
light catch a warm highlight, the upper body is lit more than the boots, and
none of it is flat because a brush was never flat. Each of those decisions
follows from something the model can measure, so here they are baked out of
it with Cycles and then painted together in numpy:

- ``ao``: how open each point is to the sky (ambient occlusion), across
  every piece, so a coat's inside and the skin under a hood are in shade;
- ``curv``: whether a point is on a ridge or in a crease (Cycles'
  "pointiness"), which is where a painter puts highlights and dark lines;
- ``nrm``: which way the surface faces, for light painted from above;
- ``pos``: where on the body it is, for the value falling off toward the feet
  and for brushwork that does not break at a seam;
- ``base``: the flat colour of each material.

``compose`` is the painting. Its numbers are the art direction, so they are
named for what they do.
"""

import os

import bpy
import numpy as np


def uv_unwrap(ob, margin=0.004, angle=66.0):
    bpy.context.view_layer.objects.active = ob
    for o in bpy.context.selected_objects:
        o.select_set(False)
    ob.select_set(True)
    with bpy.context.temp_override(active_object=ob, object=ob, selected_objects=[ob]):
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.smart_project(angle_limit=np.radians(angle), island_margin=margin,
                                 area_weight=0.0, correct_aspect=True, scale_to_bounds=False)
        bpy.ops.uv.pack_islands(margin=margin, rotate=True)
        bpy.ops.object.mode_set(mode="OBJECT")
    return ob


def _image(name, size):
    if name in bpy.data.images:
        bpy.data.images.remove(bpy.data.images[name])
    img = bpy.data.images.new(name, size, size, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    return img


def _emit_material(ob, build):
    """Give every material slot of ``ob`` a temporary emission node tree from
    ``build(nodes, links, material_name) -> socket``, keeping the originals."""
    saved = [s.material for s in ob.material_slots]
    temps = []
    for slot in ob.material_slots:
        orig = slot.material
        m = bpy.data.materials.new(f"bake_{orig.name if orig else 'none'}")
        m.use_nodes = True
        nt = m.node_tree
        for nd in list(nt.nodes):
            nt.nodes.remove(nd)
        out = nt.nodes.new("ShaderNodeOutputMaterial")
        em = nt.nodes.new("ShaderNodeEmission")
        sock = build(nt.nodes, nt.links, orig.name if orig else "none")
        nt.links.new(sock, em.inputs["Color"])
        nt.links.new(em.outputs[0], out.inputs["Surface"])
        slot.material = m
        temps.append(m)
    return saved, temps


def _restore(ob, saved, temps):
    for slot, m in zip(ob.material_slots, saved):
        slot.material = m
    for m in temps:
        bpy.data.materials.remove(m)


def _target(ob, img):
    for slot in ob.material_slots:
        nt = slot.material.node_tree
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = img
        nt.nodes.active = tex


def bake(ob, kind, size, samples=32, build=None, margin=12):
    """Bake one pass of ``ob`` into a float image and return it as an
    ``(size, size, 3)`` array, row 0 at the top of the texture."""
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.samples = samples
    s.cycles.use_denoising = False
    img = _image(f"{ob.name}_{kind}", size)
    saved, temps = (None, None)
    if build is not None:
        saved, temps = _emit_material(ob, build)
    else:
        temps = []
        saved = [sl.material for sl in ob.material_slots]
        for slot in ob.material_slots:
            m = slot.material.copy()
            slot.material = m
            temps.append(m)
    _target(ob, img)
    for o in bpy.context.selected_objects:
        o.select_set(False)
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    s.render.bake.margin = margin
    s.render.bake.use_selected_to_active = False
    if kind == "AO":
        s.world.light_settings.distance = 0.35
        bpy.ops.object.bake(type="AO")
    else:
        bpy.ops.object.bake(type="EMIT")
    px = np.array(img.pixels[:]).reshape(size, size, 4)[::-1, :, :3].copy()
    _restore(ob, saved, temps)
    bpy.data.images.remove(img)
    return px


def attr(nodes, name, kind="GEOMETRY"):
    a = nodes.new("ShaderNodeAttribute")
    a.attribute_name = name
    return a


def passes(ob, size, colours):
    """Every pass ``compose`` reads. ``colours``: material name -> rgb."""

    def base(nodes, links, mname):
        rgb = nodes.new("ShaderNodeRGB")
        rgb.outputs[0].default_value = (*colours.get(mname, (1, 0, 1)), 1)
        if "paint_mask" in ob.data.color_attributes:
            # A painted mask on the mesh picks the second colour: skin
            # showing through a glove, a stripe.
            a = nodes.new("ShaderNodeVertexColor")
            a.layer_name = "paint_mask"
            mix = nodes.new("ShaderNodeMix")
            mix.data_type = "RGBA"
            second = nodes.new("ShaderNodeRGB")
            second.outputs[0].default_value = (*colours.get(mname + "/2", colours.get(mname, (1, 0, 1))), 1)
            third = nodes.new("ShaderNodeRGB")
            third.outputs[0].default_value = (*colours.get(mname + "/3", colours.get(mname, (1, 0, 1))), 1)
            sep = nodes.new("ShaderNodeSeparateColor")
            links.new(a.outputs["Color"], sep.inputs[0])
            links.new(sep.outputs[0], mix.inputs["Factor"])
            links.new(rgb.outputs[0], mix.inputs["A"])
            links.new(second.outputs[0], mix.inputs["B"])
            mix2 = nodes.new("ShaderNodeMix")
            mix2.data_type = "RGBA"
            links.new(sep.outputs[1], mix2.inputs["Factor"])
            links.new(mix.outputs["Result"], mix2.inputs["A"])
            links.new(third.outputs[0], mix2.inputs["B"])
            return mix2.outputs["Result"]
        return rgb.outputs[0]

    def material_id(nodes, links, mname):
        rgb = nodes.new("ShaderNodeRGB")
        names = sorted(colours.keys())
        i = names.index(mname) if mname in names else 0
        rgb.outputs[0].default_value = ((i + 0.5) / 64.0, 0, 0, 1)
        return rgb.outputs[0]

    def detail(nodes, links, mname):
        a = nodes.new("ShaderNodeVertexColor")
        a.layer_name = "detail"
        return a.outputs["Color"]

    def curvature(nodes, links, mname):
        geo = nodes.new("ShaderNodeNewGeometry")
        return geo.outputs["Pointiness"]

    def normal(nodes, links, mname):
        geo = nodes.new("ShaderNodeNewGeometry")
        # Object-space, mapped to 0..1.
        vt = nodes.new("ShaderNodeVectorTransform")
        vt.vector_type = "NORMAL"
        vt.convert_from = "WORLD"
        vt.convert_to = "OBJECT"
        links.new(geo.outputs["Normal"], vt.inputs[0])
        mp = nodes.new("ShaderNodeVectorMath")
        mp.operation = "MULTIPLY_ADD"
        mp.inputs[1].default_value = (0.5, 0.5, 0.5)
        mp.inputs[2].default_value = (0.5, 0.5, 0.5)
        links.new(vt.outputs[0], mp.inputs[0])
        return mp.outputs[0]

    def position(nodes, links, mname):
        tc = nodes.new("ShaderNodeTexCoord")
        mp = nodes.new("ShaderNodeVectorMath")
        mp.operation = "MULTIPLY_ADD"
        # Metres to 0..1 over a box round the body.
        mp.inputs[1].default_value = (0.4, 0.4, 0.45)
        mp.inputs[2].default_value = (0.5, 0.5, 0.0)
        links.new(tc.outputs["Object"], mp.inputs[0])
        return mp.outputs[0]

    out = {}
    out["base"] = bake(ob, "EMIT", size, 1, base)
    out["id"] = bake(ob, "EMIT", size, 1, material_id)[:, :, 0]
    out["curv"] = bake(ob, "EMIT", size, 4, curvature)[:, :, 0]
    out["nrm"] = bake(ob, "EMIT", size, 1, normal) * 2 - 1
    out["pos"] = (bake(ob, "EMIT", size, 1, position) - np.array([0.5, 0.5, 0.0])) / np.array([0.4, 0.4, 0.45])
    out["ao"] = bake(ob, "AO", size, 48)[:, :, 0]
    if "detail" in ob.data.color_attributes:
        out["detail"] = bake(ob, "EMIT", size, 1, detail)
    out["covered"] = coverage(out)
    return out


def coverage(p):
    """Texels no face was baked into: the base pass is exactly black there
    only if nothing wrote it, which ``margin`` mostly fills."""
    return (np.abs(p["base"]).sum(axis=2) > 0).astype(float)


# --- The painting ------------------------------------------------------------

def _hash_noise(pos, scale, seed=0.0):
    """Smooth value noise over object space: brushwork that is continuous
    across uv seams because it is laid in 3D, not on the texture."""
    p = pos * scale + seed
    i = np.floor(p)
    f = p - i
    f = f * f * (3 - 2 * f)

    def h(ix, iy, iz):
        n = np.sin(ix * 127.1 + iy * 311.7 + iz * 74.7 + seed * 13.1) * 43758.5453
        return n - np.floor(n)

    x0, y0, z0 = i[..., 0], i[..., 1], i[..., 2]
    fx, fy, fz = f[..., 0], f[..., 1], f[..., 2]
    out = 0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (fx if dx else 1 - fx) * (fy if dy else 1 - fy) * (fz if dz else 1 - fz)
                out = out + w * h(x0 + dx, y0 + dy, z0 + dz)
    return out


def strokes(pos, nrm, scale, length=4.0, seed=0.0):
    """Brush strokes: noise stretched along the surface, across the light,
    so value breaks into dabs rather than drifting in blobs. On cloth the
    stroke runs down the drape; elsewhere round the form."""
    # Squash the noise along the vertical so it reads as strokes laid down
    # the body, which is how a painter models a figure.
    p = pos.copy()
    p[..., 2] = p[..., 2] / length
    a = _hash_noise(p, scale, seed)
    b = _hash_noise(p, scale * 2.3, seed + 7.0)
    return (a * 0.65 + b * 0.35) * 2 - 1


def compose(p, art):
    """The painted colour. ``art`` is a dict of knobs (see ``ART``)."""
    base = p["base"]
    ao = np.clip(p["ao"], 0, 1)
    curv = p["curv"]
    nrm = p["nrm"]
    pos = p["pos"]
    up = np.clip(nrm[..., 2], -1, 1)
    height = np.clip(pos[..., 2] / art["height"], 0, 1)

    # Occlusion: not darker only, but cooler -- shade is violet.
    shade_t = np.clip(1 - ao, 0, 1) ** art["ao_gamma"]
    shade = base * np.array(art["shade_tint"])
    col = base * (1 - shade_t[..., None] * art["ao_strength"]) + shade * shade_t[..., None] * art["ao_strength"] * 0.35

    # Light painted in from above and the front-left, warm.
    lit = np.clip(up * 0.5 + 0.5, 0, 1)
    col = col * (art["under"] + (1 - art["under"]) * lit[..., None])
    col = col * (1 + art["top_warm"] * (lit[..., None] ** 2) * (np.array(art["warm"]) - 1))

    # Value falls toward the feet: the eye goes to the head and the hands.
    col = col * (art["feet"] + (1 - art["feet"]) * height[..., None])

    # Edges: ridges catch the light, creases go dark.
    c = (curv - 0.5) * art["curv_gain"]
    ridge = np.clip(c, 0, 1)[..., None]
    crease = np.clip(-c, 0, 1)[..., None]
    col = col + ridge * art["edge_light"] * np.array(art["warm"]) * (0.4 + 0.6 * lit[..., None])
    col = col * (1 - crease * art["crease_dark"])

    # Hems: a band of trim along every open edge, a line of stitching where
    # it meets the cloth, and the very edge worn pale -- what a costume
    # painter puts on every border, so a garment reads as made.
    if "detail" in p:
        hem = p["detail"][..., 0]
        band = np.clip((hem - art["trim_from"]) / 0.05, 0, 1)[..., None]
        col = col * (1 + band * (np.array(art["trim"]) - 1))
        stitch = np.exp(-((hem - art["trim_from"]) / 0.035) ** 2)[..., None]
        col = col * (1 - stitch * art["stitch_dark"])
        worn = np.clip((hem - 0.9) / 0.1, 0, 1)[..., None]
        col = col + worn * art["edge_light"] * 0.6 * np.array(art["warm"]) * base

    # The brush.
    s = strokes(pos, nrm, art["stroke_scale"])
    s2 = strokes(pos * np.array([1.0, 1.0, 0.6]), nrm, art["stroke_scale"] * 0.45, seed=3.0)
    val = 1 + art["stroke_value"] * (0.65 * s + 0.35 * s2)
    hue = np.stack([1 + art["stroke_hue"] * s2, 1 - art["stroke_hue"] * 0.3 * s, 1 - art["stroke_hue"] * s2], -1)
    col = col * val[..., None] * hue
    return np.clip(col, 0, 1)


ART = {
    "shade_tint": (0.55, 0.5, 0.95),
    "ao_strength": 0.75,
    "ao_gamma": 0.9,
    "under": 0.62,
    "top_warm": 0.12,
    "warm": (1.18, 1.05, 0.88),
    "height": 1.8,
    "feet": 0.72,
    "curv_gain": 6.0,
    "edge_light": 0.22,
    "crease_dark": 0.45,
    "stroke_scale": 28.0,
    "stroke_value": 0.10,
    "stroke_hue": 0.05,
    "trim_from": 0.62,
    "trim": (1.45, 1.3, 1.2),
    "stitch_dark": 0.45,
}


def glow(p, colour, strength=4.0):
    """What shines: the blades' honed edges, in her violet."""
    if "detail" not in p:
        return None
    e = np.clip(p["detail"][..., 1], 0, 1)[..., None]
    return e * np.array(colour) * strength


def to_png(arr, path, srgb=True, quality=None):
    """Write a linear float image as an 8-bit file (PNG, or JPEG when the
    path says so); returns the file's bytes."""
    a = np.clip(arr, 0, 1)
    if srgb:
        a = np.where(a <= 0.0031308, a * 12.92, 1.055 * np.power(a, 1 / 2.4) - 0.055)
    h, w = a.shape[:2]
    img = bpy.data.images.new("out", w, h, alpha=True)
    rgba = np.ones((h, w, 4))
    rgba[:, :, :3] = a[::-1] if a.ndim == 3 else a[::-1, :, None]
    img.colorspace_settings.name = "Non-Color"
    img.pixels[:] = rgba.ravel()
    img.filepath_raw = path
    img.file_format = "JPEG" if path.endswith(".jpg") else "PNG"
    if quality:
        bpy.context.scene.render.image_settings.quality = quality
    img.save()
    bpy.data.images.remove(img)
    with open(path, "rb") as f:
        return f.read()


def ensure_dir(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
