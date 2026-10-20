"""The Shadow Reaver, built from nothing but this script and CC0 data.

    art/blender.sh art/reaver/build.py [--look out.png] [--pose clip frame]

Body (MakeHuman, fitted to her skeleton) -> costume (cut from the body, cloth
draped) -> blades -> painted textures -> ``crates/game/assets/reaver.glb``.
The design it builds is ``docs/design/exploration/0024_a_character_done_properly.md``.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import bpy  # noqa: E402
import numpy as np  # noqa: E402

import fit  # noqa: E402
import garment as g  # noqa: E402

g.CACHE = os.path.join(ROOT, "target/art/shells")
import glb  # noqa: E402
import mh  # noqa: E402
import outfit  # noqa: E402
import paint  # noqa: E402
import stage  # noqa: E402
from reaver_recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]


def opt(name, n=1):
    if name in args:
        i = args.index(name)
        return args[i + 1:i + 1 + n]
    return None


def body():
    base = mh.Base(os.path.join(ROOT, "target/art/mh"))
    rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
    v = mh.shaped(base, BODY)
    sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), "Shadow Reaver")
    verts, faces, face_uvs, uvs, W, bo, br, marks = fit.fit(base, rig, v, sk, GIRTH, HEAD_SCALE,
                                                            carried=mh.grip(base, rig, v))
    ob = stage.mesh_object("body", g.g2b(verts), faces, uvs, face_uvs)
    g.set_weights(ob, W)
    g.normalize(ob)
    return ob, sk, bo, br, marks


# The painter's colours: what each material is, before light. Brighter and
# more saturated than the thing itself, as a painted figure's are, because
# the painting and then the game's light both take value out.
# Linear albedo, judged lit in the game (CLAUDE.md, "Judge a palette lit"):
# the first set, three times this, came out pale lavender and tan under the
# arena's sun.
COLOURS = {
    "knit": (0.035, 0.032, 0.05),
    "knit/2": (0.50, 0.30, 0.22),      # skin, warm
    "knit/4": (0.06, 0.03, 0.05),      # brows and the smudge round her eyes
    "knit/3": (0.08, 0.045, 0.04),     # glove
    "leather": (0.11, 0.05, 0.035),
    "leather_dark": (0.04, 0.03, 0.035),
    "hood": (0.035, 0.022, 0.085),
    "mask": (0.022, 0.018, 0.03),
    "metal": (0.62, 0.64, 0.70),
    "metal_dark": (0.08, 0.075, 0.09),
    "wrap": (0.14, 0.09, 0.06),
    "trousers": (0.03, 0.024, 0.03),
    "eye": (0.70, 0.68, 0.66),
    "eye/2": (0.22, 0.14, 0.42),
    "eye/3": (0.01, 0.01, 0.015),
    "sash": (0.80, 0.80, 0.80),
}
# Her colour of light: the shadow's violet, on the blades' edges.
GLOW = (0.55, 0.25, 1.0)
METAL = {"metal": (1.0, 0.32), "metal_dark": (0.9, 0.45)}

DRESSED = os.path.join(ROOT, "target/art/reaver_dressed.blend")
OUT = os.path.join(ROOT, "crates/game/assets/reaver.glb")
TEX = os.path.join(ROOT, "target/art/tex")


def finish(bo, br, size):
    """Paint and export: the body on its own textures, the costume joined into
    one atlas, the sash on a third so the game can dye it a seat's colour."""
    objs = [o for o in bpy.context.scene.objects if o.type == "MESH" and not o.hide_render]
    body = bpy.data.objects["body"]
    team = [o for o in objs if o.name.startswith("sash")]
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
    paint.ensure_dir(os.path.join(TEX, "x"))
    images = []
    materials = {}
    for ob, name, res in ((body, "body", size), (costume, "costume", size), (team, "team", size // 4)):
        slot_names = [s.material.name for s in ob.material_slots]
        p = paint.passes(ob, res, COLOURS, high=high if ob is costume else None)
        np.save(os.path.join(TEX, f"{name}_passes.npy"), {k: v for k, v in p.items()}, allow_pickle=True)
        col = paint.compose(p, paint.ART)
        images.append(paint.to_png(col, os.path.join(TEX, f"{name}.jpg"), quality=92))
        entry = {"color": len(images) - 1}
        if "normal" in p:
            images.append(paint.to_png(p["normal"], os.path.join(TEX, f"{name}_normal.png"), srgb=False))
            entry["normal"] = len(images) - 1
        shine = paint.glow(p, GLOW)
        if shine is not None and shine.max() > 0:
            images.append(paint.to_png(np.clip(shine, 0, 1), os.path.join(TEX, f"{name}_glow.jpg"), quality=90))
            entry["emissive"] = len(images) - 1
        if any(n in METAL for n in slot_names):
            names = sorted(COLOURS.keys())
            mid = (p["id"] * 64).astype(int)
            mr = np.zeros(col.shape)
            mr[..., 1] = 0.82
            for n, (metal, rough) in METAL.items():
                i = names.index(n)
                m = mid == i
                mr[m, 1] = rough
                mr[m, 2] = metal
            images.append(paint.to_png(mr, os.path.join(TEX, f"{name}_mr.png"), srgb=False))
            entry["mr"] = len(images) - 1
        if name == "team":
            entry["double"] = True
        materials[name] = entry
        # One material a piece for the export.
        m = bpy.data.materials.new(name)
        ob.data.materials.clear()
        ob.data.materials.append(m)
        for poly in ob.data.polygons:
            poly.material_index = 0
    if high:
        bpy.data.objects.remove(high)
    prims = glb.gather([body, costume, team])
    paint.ensure_dir(OUT)
    glb.write(OUT, prims, materials, images, bo, br, name="reaver", extras=outfit.EXTRAS)
    print("wrote", OUT, os.path.getsize(OUT) // 1024, "KiB",
          {k: len(v["idx"]) // 3 for k, v in prims.items()}, "triangles")


def main():
    if opt("--paint") and os.path.exists(DRESSED):
        bpy.ops.wm.open_mainfile(filepath=DRESSED)
        import json
        meta = json.loads(bpy.context.scene["reaver_bind"])
        bo, br = np.array(meta["o"]), np.array(meta["r"])
        outfit.EXTRAS.update(meta.get("extras", {}))
    else:
        stage.reset()
        ob, sk, bo, br, marks = body()
        rig = outfit.Rig(sk, bo, br, marks)
        only = opt("--only")
        outfit.dress(ob, rig, only[0].split(",") if only else None)
        import json
        bpy.context.scene["reaver_bind"] = json.dumps({"o": np.asarray(bo).tolist(), "r": np.asarray(br).tolist(),
                                                       "extras": outfit.EXTRAS})
        for o in list(bpy.context.scene.objects):
            if o.type == "MESH" and o.hide_render and not o.get("hi"):
                bpy.data.objects.remove(o)
        bpy.ops.wm.save_as_mainfile(filepath=DRESSED)
    look = opt("--look")
    if look:
        stage.lights()
        stage.turnaround(os.path.join(ROOT, look[0]))
    if not opt("--no-export", 0) is not None:
        pass
    if "--no-export" not in args:
        size = int((opt("--size") or ["2048"])[0])
        finish(bo, br, size)


main()
