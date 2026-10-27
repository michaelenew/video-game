"""A character, built from nothing but its scripts and CC0 data.

Every class is the same five stages (``art/README.md``): a body from
MakeHuman fitted to the class's skeleton, dressed by its ``outfit.py``,
painted, exported. What differs is a class's *spec*, a module with:

- ``NAME``: the file, ``crates/game/assets/<NAME>.glb``;
- ``CLASS``: the class's name as the animation exporter writes it;
- ``BASE`` and ``PROXY`` (optional): a sculpted base mesh under
  ``art/sources`` to build the body from instead, fitted through a
  MakeHuman body of ``PROXY``'s recipe (``base_mesh``); ``BODY``, ``GIRTH``
  and ``HEAD_SCALE`` then go unused.
- ``BODY``, ``GIRTH``, ``HEAD_SCALE``: the body (``mh``, ``fit``), and
  ``FIT`` (optional) how its trunk is fitted (``fit.fit``'s ``trunk``);
- ``COLOURS``: the painter's colour for every material (and ``name/2`` to
  ``name/4`` for its masks); ``GLOW`` the colour of whatever shines;
  ``METAL`` which materials are metal, and how rough;
- ``veins`` (optional): a function from object-space positions (Blender's
  frame, metres) to 0..1, painted into the glow at the texture's own
  resolution -- light showing through cracks, sharp where a mask on the
  vertices would be soft;
- ``EDGE_GLOW`` (optional, default True): whether honed edges shine.
- ``LUMINOUS`` (optional): material -> how much of its own painted colour
  it gives off as light, for a body that is light rather than lit;
- ``SCULPT`` (optional): how much muscle to carve into the body
  (``anatomy.sculpt``): ``SCULPT_SHARE`` of it into the game's mesh, all of
  it into a dense copy the body's normal map is baked from (``sculpt``);
- ``outfit``: a module with ``dress(body, rig, only)``.

    art/blender.sh art/<class>/build.py [--look out.png [--view h,d,lens,at;angle;...]] [--paint] [--size 2048]
"""

import json
import os
import sys

import bpy
import numpy as np

import anatomy
import fit
import garment as g
import glb
import mh
import paint
import stage
import wardrobe

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
g.CACHE = os.path.join(ROOT, "target/art/shells")


def _args():
    return sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]


def _opt(name, n=1):
    args = _args()
    if name in args:
        i = args.index(name)
        return args[i + 1:i + 1 + n]
    return None


def body(spec):
    if getattr(spec, "BASE", None):
        # A sculpted base mesh, fitted through MakeHuman (``base_mesh``).
        import base_mesh
        return base_mesh.body(spec)
    base = mh.Base(os.path.join(ROOT, "target/art/mh"))
    rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
    v = mh.shaped(base, spec.BODY)
    # A chest under a fitted top is one shape, not two (the owner,
    # 2026-10-20 and 2026-10-21): ``BUST`` is how far the breasts are pressed
    # toward ``FLAT``, the same body built male.
    if getattr(spec, "BUST", 0.0):
        v = mh.flatten_bust(base, rig, v, mh.shaped(base, spec.FLAT), spec.BUST)
    sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), spec.CLASS)
    verts, faces, face_uvs, uvs, W, bo, br, marks = fit.fit(base, rig, v, sk, spec.GIRTH, spec.HEAD_SCALE,
                                                            carried=mh.grip(base, rig, v),
                                                            trunk=getattr(spec, "FIT", None))
    ob = stage.mesh_object("body", g.g2b(verts), faces, uvs, face_uvs)
    g.set_weights(ob, W)
    g.normalize(ob)
    return ob, sk, bo, br, marks


def sculpt(ob, bo, amount, game_share=0.65):
    """Carve muscle into the body (``anatomy.sculpt``): ``game_share`` of it
    into the game's own mesh, which is too coarse for a groove but takes the
    masses -- the silhouette -- and all of it into a copy subdivided twice,
    ``body~hi``, which the body's normal map and painted light are baked
    from, so the grooves are in the texture."""
    def push(o, share):
        co = np.zeros(len(o.data.vertices) * 3)
        o.data.vertices.foreach_get("co", co)
        p = g.b2g(co.reshape(-1, 3))
        nb = np.zeros(len(o.data.vertices) * 3)
        o.data.vertices.foreach_get("normal", nb)
        n = g.b2g(nb.reshape(-1, 3))
        d = anatomy.sculpt(p, g.weights_of(o), bo, amount * share)
        d = anatomy.relax(d, anatomy.mesh_edges(o.data), rounds=3 if o is hi else 2)
        o.data.vertices.foreach_set("co", g.g2b(p + n * d[:, None]).ravel())
        o.data.update()

    hi = ob.copy()
    hi.data = ob.data.copy()
    hi.name = hi.data.name = "body~hi"
    g.link(hi)
    g.subdivide(hi, 2)
    push(hi, 1.0)
    hi["hi"] = True
    hi.hide_render = True
    push(ob, game_share)
    return hi


def _half(a):
    """An image at half its size, each pixel the mean of the four under it:
    for maps that carry no fine detail -- what glows, what is metal -- and
    cost a full-size picture's memory for nothing (exploration 0024 sec. 11)."""
    h, w = a.shape[:2]
    return a[:h // 2 * 2, :w // 2 * 2].reshape(h // 2, 2, w // 2, 2, *a.shape[2:]).mean(axis=(1, 3))


def finish(spec, bo, br, size):
    """Paint and export: the body on its own textures, the costume joined into
    one atlas, the team-coloured piece on a third so the game can dye it a
    seat's colour."""
    tex = os.path.join(ROOT, "target/art/tex", spec.NAME)
    out = os.path.join(ROOT, "crates/game/assets", f"{spec.NAME}.glb")
    objs = [o for o in bpy.context.scene.objects if o.type == "MESH" and not o.hide_render]
    body = bpy.data.objects["body"]
    # The sculpted body's dense copy (``sculpt``), if there is one: what the
    # body's normal map is baked from, not part of the costume's.
    body_hi = bpy.data.objects.get("body~hi")
    team = [o for o in objs if o.name.startswith(("sash", "team"))]
    # Hair cards are not painted: their texture is the strands
    # (``wardrobe.strand_texture``), cut out where there are none.
    cards = [o for o in objs if o.name.startswith("cards")]
    costume = [o for o in objs if o is not body and o not in team and o not in cards]
    for o in costume + team:
        g.group_to_colour(o, ["hem", "edge"])
    costume = g.join(costume, "costume")
    highs = [o for o in bpy.context.scene.objects if o.get("hi") and o is not body_hi]
    for o in highs:
        o.hide_render = False
    high = g.join(highs, "costume~hi") if highs else None
    # A partial build (``--only``) may have no team piece.
    team = g.join(team, "team") if team else None
    paint.uv_unwrap(costume)
    if team:
        paint.uv_unwrap(team)
    paint.ensure_dir(os.path.join(tex, "x"))
    art = dict(paint.ART, **getattr(spec, "ART", {}))
    images = []
    materials = {}
    pieces = [(body, "body", size), (costume, "costume", size)] + ([(team, "team", size // 4)] if team else [])
    for ob, name, res in pieces:
        slot_names = [s.material.name for s in ob.material_slots]
        source = high if ob is costume else body_hi if ob is body else None
        if source is not None:
            source.hide_render = False
        p = paint.passes(ob, res, spec.COLOURS, high=source)
        col = paint.compose(p, art)
        images.append(paint.to_png(col, os.path.join(tex, f"{name}.jpg"), quality=92))
        entry = {"color": len(images) - 1}
        if "normal" in p:
            # A JPEG, not a PNG: a normal map is noise to a lossless
            # compressor (three megabytes of each file's eight), and the
            # game compresses it lossily on the GPU anyway.
            images.append(paint.to_png(p["normal"], os.path.join(tex, f"{name}_normal.jpg"), srgb=False,
                                       quality=95))
            entry["normal"] = len(images) - 1
        # Honed edges shine (``paint.glow``) unless the spec says not:
        # on a being every plate's rim is an edge, and lit they read as
        # white wire (2026-10-26).
        shine = paint.glow(p, spec.GLOW) if getattr(spec, "EDGE_GLOW", True) else None
        veins = getattr(spec, "veins", None)
        if veins is not None:
            v = np.clip(veins(p["pos"]), 0, 1)[..., None] * p["covered"][..., None]
            lit = v * np.array(spec.GLOW) * 4.0
            shine = lit if shine is None else np.maximum(shine, lit)
            # The crack is lit from inside: its own colour, not the stone's.
            col = col * (1 - v) + v * np.array(spec.GLOW)
            images[entry["color"]] = paint.to_png(col, os.path.join(tex, f"{name}.jpg"), quality=92)
        lum = getattr(spec, "LUMINOUS", {})
        if lum:
            names = sorted(spec.COLOURS.keys())
            mid = (p["id"] * 64).astype(int)
            own = np.zeros(col.shape[:2])
            for n, k in lum.items():
                if n in names:
                    own[mid == names.index(n)] = k
            lit = col[..., :3] * own[..., None] * p["covered"][..., None]
            shine = lit if shine is None else np.maximum(shine, lit)
        if shine is not None and shine.max() > 0:
            # A body whose light is drawing (``veins``, ``LUMINOUS``) keeps
            # its glow whole and lossless: the game multiplies it many times
            # over, and a JPEG's noise multiplied is coloured speckle.
            if veins is not None or lum:
                # Scaled, not clipped, so a violet stays violet rather than
                # going white where it is brightest.
                over = np.maximum(shine.max(axis=-1, keepdims=True), 1.0)
                images.append(paint.to_png(shine / over, os.path.join(tex, f"{name}_glow.png")))
            else:
                images.append(paint.to_png(_half(np.clip(shine, 0, 1)), os.path.join(tex, f"{name}_glow.jpg"),
                                           quality=90))
            entry["emissive"] = len(images) - 1
        if any(n in spec.METAL for n in slot_names):
            names = sorted(spec.COLOURS.keys())
            mid = (p["id"] * 64).astype(int)
            mr = np.zeros(col.shape)
            mr[..., 1] = 0.82
            for n, (metal, rough) in spec.METAL.items():
                if n not in names:
                    continue
                m = mid == names.index(n)
                mr[m, 1] = rough
                mr[m, 2] = metal
            images.append(paint.to_png(_half(mr), os.path.join(tex, f"{name}_mr.png"), srgb=False))
            entry["mr"] = len(images) - 1
        if name == "team":
            entry["double"] = True
        materials[name] = entry
        # One material a piece for the export, named exactly: the game finds
        # the team piece by it, so a placeholder of the same name moves over.
        if name in bpy.data.materials:
            bpy.data.materials[name].name = name + "~placeholder"
        m = bpy.data.materials.new(name)
        ob.data.materials.clear()
        ob.data.materials.append(m)
        for poly in ob.data.polygons:
            poly.material_index = 0
    if high:
        bpy.data.objects.remove(high)
    if body_hi:
        bpy.data.objects.remove(body_hi)
    parts = [body, costume] + ([team] if team else [])
    if cards:
        cards = g.join(cards, "cards")
        strands = wardrobe.strand_texture(spec.COLOURS["hair"], spec.COLOURS.get("hair/2"))
        images.append(paint.to_png(strands, os.path.join(tex, "hair_cards.png")))
        materials["hair_cards"] = {"color": len(images) - 1, "double": True, "mask": 0.4}
        parts.append(cards)
    prims = glb.gather(parts)
    paint.ensure_dir(out)
    glb.write(out, prims, materials, images, bo, br, name=spec.NAME, extras=wardrobe.EXTRAS)
    print("wrote", out, os.path.getsize(out) // 1024, "KiB",
          {k: len(v["idx"]) // 3 for k, v in prims.items()}, "triangles")


def build(spec):
    dressed = os.path.join(ROOT, "target/art", f"{spec.NAME}_dressed.blend")
    wardrobe.PALETTE.update({k: v for k, v in spec.COLOURS.items() if "/" not in k})
    if _opt("--paint") is not None and os.path.exists(dressed):
        bpy.ops.wm.open_mainfile(filepath=dressed)
        meta = json.loads(bpy.context.scene["bind"])
        bo, br = np.array(meta["o"]), np.array(meta["r"])
        wardrobe.EXTRAS.update(meta.get("extras", {}))
    else:
        stage.reset()
        ob, sk, bo, br, marks = body(spec)
        if getattr(spec, "SCULPT", 0.0):
            sculpt(ob, bo, spec.SCULPT, getattr(spec, "SCULPT_SHARE", 0.65))
        rig = wardrobe.Rig(sk, bo, br, marks)
        only = _opt("--only")
        spec.outfit.dress(ob, rig, only[0].split(",") if only else None)
        bpy.context.scene["bind"] = json.dumps({"o": np.asarray(bo).tolist(), "r": np.asarray(br).tolist(),
                                                "extras": wardrobe.EXTRAS})
        for o in list(bpy.context.scene.objects):
            if o.type == "MESH" and o.hide_render and not o.get("hi"):
                bpy.data.objects.remove(o)
        bpy.ops.wm.save_as_mainfile(filepath=dressed)
    look = _opt("--look")
    if look:
        # The materials as they will be: metal where the spec says so, at
        # its roughness, so a plate's form reads in the turnaround as it
        # will in the game.
        for m in bpy.data.materials:
            if m.name in getattr(spec, "METAL", {}) and m.use_nodes and "Principled BSDF" in m.node_tree.nodes:
                metal, rough = spec.METAL[m.name]
                b = m.node_tree.nodes["Principled BSDF"]
                b.inputs["Metallic"].default_value = metal
                b.inputs["Roughness"].default_value = rough
                if m.name in spec.COLOURS:
                    b.inputs["Base Color"].default_value = (*spec.COLOURS[m.name], 1)
        if "--clay" in _args():
            # Every surface one grey, to judge the form alone.
            clay = stage.flat_material("clay", (0.62, 0.6, 0.57), 0.6)
            for o in bpy.context.scene.objects:
                if o.type == "MESH":
                    o.data.materials.clear()
                    o.data.materials.append(clay)
        stage.lights()
        # ``--view height,distance,lens,look_at,angle;angle...``: a close
        # look at one piece, framed as its reference is.
        view = _opt("--view")
        if view:
            nums, _, angles = view[0].partition(";")
            h, d, lens, at = (float(x) for x in nums.split(","))
            stage.turnaround(os.path.join(ROOT, look[0]), angles=tuple(int(a) for a in angles.split(";")),
                             size=(560, 640), height=h, distance=d, lens=lens, look_at=at)
        else:
            stage.turnaround(os.path.join(ROOT, look[0]))
    if "--no-export" not in _args():
        size = int((_opt("--size") or ["2048"])[0])
        finish(spec, bo, br, size)


def dress(body, r, wardrobe_list, only, garments, hide=None, masks=None, skin_material="knit"):
    """Put on every garment in ``wardrobe_list`` (names looked up in
    ``garments``), bind what is not rigid to the body, and paint the body's
    masks."""
    import time
    pieces = []
    for name in wardrobe_list:
        if only and name not in only:
            continue
        t = time.time()
        pieces += garments[name](body, r)
        print(f"  {name}: {time.time() - t:.1f}s", flush=True)
    for p in pieces:
        if not (p.get("rigid") or p.get("weighted")):
            g.bind_to(p, body)
    body.data.materials.append(wardrobe.mat(skin_material))
    if masks:
        masks(body, r)
    if hide and not only:
        wardrobe.hide_covered(body, lambda c, fw: hide(r, c, fw))
    return pieces
