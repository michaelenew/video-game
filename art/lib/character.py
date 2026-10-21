"""A character, built from nothing but its scripts and CC0 data.

Every class is the same five stages (``art/README.md``): a body from
MakeHuman fitted to the class's skeleton, dressed by its ``outfit.py``,
painted, exported. What differs is a class's *spec*, a module with:

- ``NAME``: the file, ``crates/game/assets/<NAME>.glb``;
- ``CLASS``: the class's name as the animation exporter writes it;
- ``BODY``, ``GIRTH``, ``HEAD_SCALE``: the body (``mh``, ``fit``);
- ``COLOURS``: the painter's colour for every material (and ``name/2`` to
  ``name/4`` for its masks); ``GLOW`` the colour of whatever shines;
  ``METAL`` which materials are metal, and how rough;
- ``outfit``: a module with ``dress(body, rig, only)``.

    art/blender.sh art/<class>/build.py [--look out.png] [--paint] [--size 2048]
"""

import json
import os
import sys

import bpy
import numpy as np

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
    base = mh.Base(os.path.join(ROOT, "target/art/mh"))
    rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
    v = mh.shaped(base, spec.BODY)
    sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), spec.CLASS)
    verts, faces, face_uvs, uvs, W, bo, br, marks = fit.fit(base, rig, v, sk, spec.GIRTH, spec.HEAD_SCALE,
                                                            carried=mh.grip(base, rig, v))
    ob = stage.mesh_object("body", g.g2b(verts), faces, uvs, face_uvs)
    g.set_weights(ob, W)
    g.normalize(ob)
    return ob, sk, bo, br, marks


def finish(spec, bo, br, size):
    """Paint and export: the body on its own textures, the costume joined into
    one atlas, the team-coloured piece on a third so the game can dye it a
    seat's colour."""
    tex = os.path.join(ROOT, "target/art/tex", spec.NAME)
    out = os.path.join(ROOT, "crates/game/assets", f"{spec.NAME}.glb")
    objs = [o for o in bpy.context.scene.objects if o.type == "MESH" and not o.hide_render]
    body = bpy.data.objects["body"]
    team = [o for o in objs if o.name.startswith(("sash", "team"))]
    costume = [o for o in objs if o is not body and o not in team]
    for o in costume + team:
        g.group_to_colour(o, ["hem", "edge"])
    costume = g.join(costume, "costume")
    highs = [o for o in bpy.context.scene.objects if o.get("hi")]
    for o in highs:
        o.hide_render = False
    high = g.join(highs, "costume~hi") if highs else None
    team = g.join(team, "team")
    paint.uv_unwrap(costume)
    paint.uv_unwrap(team)
    paint.ensure_dir(os.path.join(tex, "x"))
    art = dict(paint.ART, **getattr(spec, "ART", {}))
    images = []
    materials = {}
    for ob, name, res in ((body, "body", size), (costume, "costume", size), (team, "team", size // 4)):
        slot_names = [s.material.name for s in ob.material_slots]
        p = paint.passes(ob, res, spec.COLOURS, high=high if ob is costume else None)
        col = paint.compose(p, art)
        images.append(paint.to_png(col, os.path.join(tex, f"{name}.jpg"), quality=92))
        entry = {"color": len(images) - 1}
        if "normal" in p:
            images.append(paint.to_png(p["normal"], os.path.join(tex, f"{name}_normal.png"), srgb=False))
            entry["normal"] = len(images) - 1
        shine = paint.glow(p, spec.GLOW)
        if shine is not None and shine.max() > 0:
            images.append(paint.to_png(np.clip(shine, 0, 1), os.path.join(tex, f"{name}_glow.jpg"), quality=90))
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
            images.append(paint.to_png(mr, os.path.join(tex, f"{name}_mr.png"), srgb=False))
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
    prims = glb.gather([body, costume, team])
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
        stage.lights()
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
