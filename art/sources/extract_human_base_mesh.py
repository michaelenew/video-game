"""Extract the realistic male body from Blender Studio's Human Base Meshes
bundle (v1.0, CC0) into ``human_base_mesh_male.npz``, the dark being's body.

    art/blender.sh art/sources/extract_human_base_mesh.py path/to/human_base_meshes_bundle.blend

Its sculpted detail (the Multires modifier) is folded into the base mesh
first, so the body kept is the sculpted shape at the base's resolution. The
eyes are kept as their centres and radius, for the being's own eyes. Saved
in metres, Blender's frame (``+z`` up, the body facing ``-y``).
"""

import os
import sys

import bpy
import numpy as np

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
bpy.ops.wm.open_mainfile(filepath=args[0], load_ui=False, use_scripts=False)
name = "GEO-body_male_realistic"
ob = bpy.data.objects[name]
bpy.context.view_layer.objects.active = ob
for m in list(ob.modifiers):
    if m.type == "MULTIRES":
        with bpy.context.temp_override(object=ob, active_object=ob):
            bpy.ops.object.multires_base_apply(modifier=m.name)
    ob.modifiers.remove(m)
me = ob.data
mw = np.array(ob.matrix_world)
co = np.zeros(len(me.vertices) * 3)
me.vertices.foreach_get("co", co)
co = co.reshape(-1, 3) @ mw[:3, :3].T + mw[:3, 3]
counts = np.zeros(len(me.polygons), int)
me.polygons.foreach_get("loop_total", counts)
loops = np.zeros(len(me.loops), int)
me.loops.foreach_get("vertex_index", loops)
uv = np.zeros(len(me.loops) * 2)
me.uv_layers[0].data.foreach_get("uv", uv)
eyes = []
for side in ("L", "R"):
    e = bpy.data.objects[f"{name}.eye.{side}"]
    p = np.array([e.matrix_world @ v.co for v in e.data.vertices])
    c = p.mean(0)
    eyes.append(np.r_[c, np.linalg.norm(p - c, axis=1).max()])
# The body is centred on x = 0, standing on z = 0.
centre = np.array([(co[:, 0].max() + co[:, 0].min()) / 2, 0.0, co[:, 2].min()])
co -= centre
eyes = np.array(eyes)
eyes[:, :3] -= centre
out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "human_base_mesh_male.npz")
np.savez_compressed(out, verts=co.astype(np.float32), counts=counts.astype(np.int32), loops=loops.astype(np.int32),
                    uvs=uv.reshape(-1, 2).astype(np.float32), eyes=eyes.astype(np.float32))
print("wrote", out, len(co), "verts", len(counts), "faces", "height", co[:, 2].max())
