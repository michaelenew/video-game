"""The dark being's bare body, sculpted (``lib/anatomy.py``), from four sides:
clay, to judge the anatomy, and obsidian, to judge the look.

    art/blender.sh art/dark_being/anatomy_check.py out.png [strength]
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [os.path.join(HERE, "..", "lib"), HERE]

import bpy  # noqa: E402
import numpy as np  # noqa: E402

import anatomy  # noqa: E402
import fit  # noqa: E402
import mh  # noqa: E402
import stage  # noqa: E402
from recipe import BODY, FIT, GIRTH, HEAD_SCALE  # noqa: E402

ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = os.path.join(ROOT, args[0] if args else "target/art/anatomy.png")
strength = float(args[1]) if len(args) > 1 else 1.0
look = args[2] if len(args) > 2 else "clay"

base = mh.Base(os.path.join(ROOT, "target/art/mh"))
rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
v = mh.shaped(base, BODY)
sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), "Bulwark")
verts, faces, face_uvs, uvs, W, bo, br, marks = fit.fit(base, rig, v, sk, GIRTH, HEAD_SCALE,
                                                        carried=mh.grip(base, rig, v), trunk=FIT)
import garment as g  # noqa: E402

stage.reset()
ob = stage.mesh_object("body", stage.y_up_to_blender(verts), faces, uvs, face_uvs)
g.set_weights(ob, W)
# Dense enough for a muscle to have a shape: the base mesh is three
# centimetres a face across the belly.
g.subdivide(ob, 2)
co = np.zeros(len(ob.data.vertices) * 3)
ob.data.vertices.foreach_get("co", co)
p = g.b2g(co.reshape(-1, 3))
Wd = g.weights_of(ob)
push = anatomy.sculpt(p, Wd, bo, strength)
push = anatomy.relax(push, anatomy.mesh_edges(ob.data), rounds=3)
nb = np.zeros(len(ob.data.vertices) * 3)
ob.data.vertices.foreach_get("normal", nb)
nrm = g.b2g(nb.reshape(-1, 3))
p = p + nrm * push[:, None]
ob.data.vertices.foreach_set("co", g.g2b(p).ravel())
ob.data.update()
ob.data.shade_smooth()
if look == "clay":
    ob.data.materials.append(stage.flat_material("clay", (0.62, 0.6, 0.57), 0.6))
else:
    m = stage.flat_material("obsidian", (0.012, 0.009, 0.016), 0.08)
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Coat Weight"].default_value = 0.8
    b.inputs["Coat Roughness"].default_value = 0.03
    b.inputs["Specular IOR Level"].default_value = 0.8
    ob.data.materials.append(m)
stage.lights()
close = len(args) > 3 and args[3] == "close"
if close:
    stage.turnaround(out, angles=(0, 35, 160), size=(520, 620), height=1.3, distance=1.7, look_at=1.22,
                     samples=32)
else:
    stage.turnaround(out, angles=(0, 35, 135, 180), size=(420, 640), height=1.35, distance=3.2, look_at=1.3,
                     samples=32)
print("wrote", out)
