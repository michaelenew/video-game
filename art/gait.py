"""Render a bred stage moving: a strip of its gait cycle, side on.

    BRED_GAIT=<line>:<stage>[,...] cargo test -p game --bin game bred_gait
    art/blender.sh art/gait.py [--size 300x220] [--only <prefix>]

Reads ``target/gait/<line>_<stage>_<gait>_<k>.obj`` (the body's boxes at
eight points of a cycle, coloured by what each part is) and writes
``target/gait/<line>_<stage>_<gait>.png``: the eight frames left to right,
from the side, with one camera for all eight so the movement reads, and a
top view under them, where the feet and the body's footprint show.
"""

import glob
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "lib")]

import numpy as np  # noqa: E402

import beast as b  # noqa: E402

DIR = os.path.join(HERE, "..", "target", "gait")


def load(path):
    v, c, f = [], [], []
    for line in open(path):
        if line.startswith("v "):
            x = [float(a) for a in line.split()[1:]]
            v.append(x[:3])
            c.append(x[3:6])
        elif line.startswith("f "):
            f.append([int(a) - 1 for a in line.split()[1:]])
    return np.array(v), np.array(c), f


def frame(path, out, size, centre, radius, view):
    import bpy
    import mathutils

    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    v, col, f = load(path)
    ob = b.obj("body", (v, f))
    ca = ob.data.color_attributes.new("col", "FLOAT_COLOR", "POINT")
    ca.data.foreach_set("color", np.concatenate([col, np.ones((len(col), 1))], 1).astype(np.float32).ravel())
    m = bpy.data.materials.new("m")
    m.use_nodes = True
    nt = m.node_tree
    attr = nt.nodes.new("ShaderNodeAttribute")
    attr.attribute_name = "col"
    bsdf = nt.nodes["Principled BSDF"]
    nt.links.new(attr.outputs["Color"], bsdf.inputs["Base Color"])
    bsdf.inputs["Roughness"].default_value = 0.7
    ob.data.materials.append(m)
    # The floor, chequered by the metre.
    floor = b.obj("floor", (np.array([[-60, 0, -60], [60, 0, -60], [60, 0, 60], [-60, 0, 60]], float), [[0, 3, 2, 1]]))
    fm = bpy.data.materials.new("floor")
    fm.use_nodes = True
    ft = fm.node_tree
    tc = ft.nodes.new("ShaderNodeTexCoord")
    ck = ft.nodes.new("ShaderNodeTexChecker")
    ck.inputs["Scale"].default_value = 1.0
    ck.inputs["Color1"].default_value = (0.42, 0.41, 0.39, 1)
    ck.inputs["Color2"].default_value = (0.5, 0.49, 0.47, 1)
    ft.links.new(tc.outputs["Object"], ck.inputs["Vector"])
    ft.links.new(ck.outputs["Color"], ft.nodes["Principled BSDF"].inputs["Base Color"])
    floor.data.materials.append(fm)
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.6, 0.64, 0.7, 1)
    s.world = world
    light = bpy.data.lights.new("key", "SUN")
    light.energy = 3.0
    lo_ = bpy.data.objects.new("key", light)
    s.collection.objects.link(lo_)
    lo_.rotation_euler = mathutils.Vector((-0.4, -0.5, -1.0)).to_track_quat("-Z", "Y").to_euler()
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = radius * 2.4
    o = bpy.data.objects.new("cam", cam)
    s.collection.objects.link(o)
    if view == "side":
        eye = centre + np.array([0.0, 0.0, 1.0]) * radius * 4
    else:
        eye = centre + np.array([0.0, 1.0, 0.0]) * radius * 4
    o.location = mathutils.Vector(b.to_blender(np.array([eye]))[0])
    target = mathutils.Vector(b.to_blender(np.array([centre]))[0])
    up = "Y"
    o.rotation_euler = (target - o.location).to_track_quat("-Z", up).to_euler()
    s.camera = o
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = 8
    s.cycles.use_denoising = True
    s.view_settings.view_transform = "Standard"
    s.render.resolution_x, s.render.resolution_y = size
    s.render.filepath = out
    bpy.ops.render.render(write_still=True)


def main():
    args = sys.argv[1:]
    size = tuple(int(x) for x in (args[args.index("--size") + 1] if "--size" in args else "300x220").split("x"))
    only = args[args.index("--only") + 1] if "--only" in args else ""
    groups = {}
    for p in glob.glob(os.path.join(DIR, "*_[0-7].obj")):
        name = os.path.basename(p)
        if not name.startswith(only):
            continue
        key = re.sub(r"_[0-7]\.obj$", "", name)
        groups.setdefault(key, []).append(p)
    for key, paths in sorted(groups.items()):
        paths.sort()
        # One framing for the whole cycle.
        allv = np.concatenate([load(p)[0] for p in paths])
        lo, hi = allv.min(0), allv.max(0)
        lo[1] = min(lo[1], 0.0)
        centre = (lo + hi) / 2
        radius = max(hi - lo) / 2 * 1.05
        rows = []
        for view in ("side", "top"):
            tiles = []
            for k, p in enumerate(paths):
                out = os.path.join(DIR, f"_{key}_{view}_{k}.png")
                frame(p, out, size, centre, radius, view)
                tiles.append(out)
            row = os.path.join(DIR, f"_{key}_{view}.png")
            subprocess.run(["convert", *tiles, "+append", row], check=True)
            rows.append(row)
        out = os.path.join(DIR, f"{key}.png")
        subprocess.run(["convert", *rows, "-append", "-gravity", "north", "-background", "#202024",
                        "-fill", "#e8e4dc", "-pointsize", "16", "-splice", "0x24", "-annotate", "+0+4", key, out],
                       check=True)
        print("wrote", key, flush=True)


if __name__ == "__main__":
    main()
