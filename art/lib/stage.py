"""A photo studio in Blender, for looking at a character without the game.

Cycles on the CPU, because it is the renderer that runs with no display.
``turnaround`` renders a character from several sides and tiles the
pictures into one sheet, which is how a character artist checks a model:
front, three-quarter, side and back, under the same light.
"""

import math
import os
import subprocess

import bpy
import numpy as np


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    s = bpy.context.scene
    s.render.engine = "CYCLES"
    s.cycles.device = "CPU"
    s.cycles.samples = 24
    s.cycles.use_denoising = True
    s.view_settings.view_transform = "AgX"
    s.view_settings.look = "AgX - Medium High Contrast"
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs[0].default_value = (0.42, 0.45, 0.55, 1)
    bg.inputs[1].default_value = 0.6
    s.world = world
    return s


def mesh_object(name, verts, faces, uvs=None, face_uvs=None, smooth=True):
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(map(float, v)) for v in verts], [], [tuple(f) for f in faces])
    if uvs is not None and face_uvs is not None:
        layer = me.uv_layers.new(name="UVMap")
        flat = [uv for fu in face_uvs for uv in fu]
        data = np.array([uvs[i] if i >= 0 else (0.0, 0.0) for i in flat], dtype=np.float32)
        layer.data.foreach_set("uv", data.ravel())
    me.shade_smooth() if smooth else None
    me.validate()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def y_up_to_blender(v):
    """The game's +y up, +z forward to Blender's +z up, -y forward."""
    v = np.asarray(v, float)
    return np.stack([v[..., 0], -v[..., 2], v[..., 1]], axis=-1)


def lights(target_height=1.0):
    s = bpy.context.scene

    def light(name, kind, energy, loc, color=(1, 1, 1), size=1.0):
        l = bpy.data.lights.new(name, kind)
        l.energy = energy
        l.color = color
        if kind == "AREA":
            l.size = size
        o = bpy.data.objects.new(name, l)
        s.collection.objects.link(o)
        o.location = loc
        d = np.array([0, 0, target_height]) - np.array(loc)
        o.rotation_euler = _look(d)
        return o

    # Key warm from above-front-left, rim cool from behind, fill from the sky.
    light("key", "SUN", 3.2, (-3, -4, 6), (1.0, 0.93, 0.82))
    light("rim", "SUN", 2.4, (3, 5, 4), (0.62, 0.7, 1.0))
    light("fill", "SUN", 0.5, (4, -3, 1), (0.8, 0.8, 1.0))


def _look(d):
    d = d / np.linalg.norm(d)
    # A light or camera looks down its -z; Euler XYZ that turns -z onto d.
    pitch = math.acos(max(-1.0, min(1.0, -d[2])))
    yaw = math.atan2(-d[0], d[1])
    return (pitch, 0.0, yaw)


def camera(distance=4.2, height=1.0, lens=55, angle_deg=0.0, look_at=1.0):
    s = bpy.context.scene
    cam = bpy.data.cameras.new("cam")
    cam.lens = lens
    o = bpy.data.objects.new("cam", cam)
    s.collection.objects.link(o)
    a = math.radians(angle_deg)
    # angle 0: in front of the character, who faces Blender's -y.
    o.location = (distance * math.sin(a), -distance * math.cos(a), height)
    o.rotation_euler = _look(np.array([0, 0, look_at]) - np.array(o.location))
    s.camera = o
    return o


def turnaround(out, angles=(0, 35, 90, 180), size=(420, 640), height=1.0, distance=4.2,
               lens=55, look_at=0.95, samples=24):
    s = bpy.context.scene
    s.cycles.samples = samples
    s.render.resolution_x, s.render.resolution_y = size
    tiles = []
    base, _ = os.path.splitext(out)
    for a in angles:
        cam = camera(distance, height, lens, a, look_at)
        path = f"{base}_{a:03d}.png"
        s.render.filepath = path
        bpy.ops.render.render(write_still=True)
        bpy.data.objects.remove(cam)
        tiles.append(path)
    subprocess.run(["convert", *tiles, "+append", out], check=True)
    for t in tiles:
        os.remove(t)
    return out


def flat_material(name, color, roughness=0.6):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*color, 1)
    b.inputs["Roughness"].default_value = roughness
    return m
