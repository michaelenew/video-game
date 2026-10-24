"""Render crossbreed lines, stage by stage (exploration 0031).

    GROWN_LINES=1 cargo test -p game --bin game grown_lines     # the stages
    art/blender.sh art/lineage.py [--samples 24] [--size 420x320] [--line <name>] [--stages 0,3,6] [--yaw 35] [--pitch 16]

Reads every ``target/lines/<line>_<stage>.obj`` (a grown body, coloured as
the game colours it) and ``target/lines/lines.tsv`` (what each stage is),
and writes ``target/lines/<line>.png``: the stages left to right, each in
the same three-quarter view, framed to its own size (its length is in its
label), with what changed written under it.
"""

import glob
import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "lib")]

import numpy as np  # noqa: E402

import beast as b  # noqa: E402

ROOT = os.path.join(HERE, "..")
DIR = os.path.join(ROOT, "target/lines")


def load(path):
    v, c, f = [], [], []
    for line in open(path):
        if line.startswith("v "):
            x = [float(a) for a in line.split()[1:]]
            v.append(x[:3])
            c.append(x[3:6] if len(x) >= 6 else [0.6, 0.6, 0.6])
        elif line.startswith("f "):
            f.append([int(a) - 1 for a in line.split()[1:4]])
    return np.array(v), np.array(c), f


def stage(path, out, size, samples, yaw=35, pitch=16):
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
    bsdf.inputs["Roughness"].default_value = 0.65
    ob.data.materials.append(m)
    lo, hi = v.min(0), v.max(0)
    centre = (lo + hi) / 2
    radius = np.linalg.norm(hi - lo) / 2
    ground = b.obj("ground", (np.array([[-80, 0, -80], [80, 0, -80], [80, 0, 80], [-80, 0, 80]], float),
                              [[0, 3, 2, 1]]))
    gm = bpy.data.materials.new("g")
    gm.use_nodes = True
    gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.36, 0.33, 0.29, 1)
    ground.data.materials.append(gm)
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.55, 0.6, 0.68, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.9
    s.world = world
    for name, energy, d in (("key", 3.4, (-0.5, -0.6, -1.0)), ("rim", 1.8, (0.6, 0.8, -0.5))):
        light = bpy.data.lights.new(name, "SUN")
        light.energy = energy
        light.angle = 0.1
        o = bpy.data.objects.new(name, light)
        s.collection.objects.link(o)
        o.rotation_euler = mathutils.Vector(d).to_track_quat("-Z", "Y").to_euler()
    cam = bpy.data.cameras.new("cam")
    cam.lens = 50
    o = bpy.data.objects.new("cam", cam)
    s.collection.objects.link(o)
    yaw, pitch = math.radians(yaw), math.radians(pitch)
    dist = radius * 3.0
    eye = centre + np.array([math.cos(yaw) * math.cos(pitch), math.sin(pitch), -math.sin(yaw) * math.cos(pitch)]) * dist
    o.location = mathutils.Vector(b.to_blender(np.array([eye]))[0])
    target = mathutils.Vector(b.to_blender(np.array([centre]))[0])
    o.rotation_euler = (target - o.location).to_track_quat("-Z", "Y").to_euler()
    s.camera = o
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = samples
    s.cycles.use_denoising = True
    s.view_settings.view_transform = "AgX"
    s.render.resolution_x, s.render.resolution_y = size
    s.render.filepath = out
    bpy.ops.render.render(write_still=True)


def main():
    args = sys.argv[1:]
    samples = int(args[args.index("--samples") + 1]) if "--samples" in args else 24
    pitch = float(args[args.index("--pitch") + 1]) if "--pitch" in args else 16
    yaw = float(args[args.index("--yaw") + 1]) if "--yaw" in args else 35
    size = tuple(int(x) for x in (args[args.index("--size") + 1] if "--size" in args else "420x320").split("x"))
    rows = [r.rstrip("\n").split("\t") for r in open(os.path.join(DIR, "lines.tsv"))][1:]
    # ``--line <name>`` and ``--stages 0,3,6``: a few, to look at quickly.
    if "--line" in args:
        rows = [r for r in rows if r[0] == args[args.index("--line") + 1]]
    if "--stages" in args:
        keep = {int(k) for k in args[args.index("--stages") + 1].split(",")}
        rows = [r for r in rows if int(r[1]) in keep]
    by_line = {}
    for r in rows:
        by_line.setdefault(r[0], []).append(r)
    for name, stages in by_line.items():
        tiles = []
        for r in stages:
            k = int(r[1])
            obj = os.path.join(DIR, f"{name}_{k:02d}.obj")
            if not os.path.exists(obj):
                continue
            png = os.path.join(DIR, f"{name}_{k:02d}.png")
            stage(obj, png, size, samples, yaw, pitch)
            raw = obj.replace(".obj", "_raw.obj")
            if os.path.exists(raw) and os.path.getmtime(raw) >= os.path.getmtime(obj) - 600:
                rpng = png.replace(".png", "_raw.png")
                stage(raw, rpng, size, samples, yaw, pitch)
                for p, word in ((rpng, "raw"), (png, "hand")):
                    subprocess.run(["convert", p, "-gravity", "northwest", "-fill", "#f0ece4", "-pointsize", "15",
                                    "-annotate", "+8+6", word, p], check=True)
                subprocess.run(["convert", rpng, png, "-append", png], check=True)
            said = r[6].split(": ", 1)[-1] if ": " in r[6] else r[6]
            said = said if said.strip() else "(sliders only)"
            label = f"gen {r[2]}  {float(r[3]):.1f} m\n{said}"
            labelled = png.replace(".png", "_l.png")
            subprocess.run(["convert", png, "-gravity", "south", "-background", "#202024", "-fill", "#e8e4dc",
                            "-font", "DejaVu-Sans", "-pointsize", "13", "-size", f"{size[0]}x44",
                            f"caption:{label}", "-append", labelled], check=True)
            tiles.append(labelled)
        if tiles:
            subprocess.run(["montage", *tiles, "-tile", f"{min(len(tiles), 7)}x", "-geometry", "+3+3",
                            "-background", "#121214", os.path.join(DIR, f"{name}.png")], check=True)
            print("wrote", name, flush=True)


if __name__ == "__main__":
    main()
