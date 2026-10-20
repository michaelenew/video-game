"""The bare body on the Reaver's skeleton, rendered from four sides.

    art/blender.sh art/reaver/body_check.py [out.png] [clip] [frame]

With a clip, the body is posed at that frame of it (by linear blend skinning,
exactly as the game will), so a fit that tears at a joint shows.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "lib"))

import numpy as np  # noqa: E402

import fit  # noqa: E402
import mh  # noqa: E402
import stage  # noqa: E402
from reaver_recipe import BODY, GIRTH, HEAD_SCALE  # noqa: E402

ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else os.path.join(ROOT, "target/art/body.png")
clip = args[1] if len(args) > 1 else None
frame = int(args[2]) if len(args) > 2 else 0

base = mh.Base(os.path.join(ROOT, "target/art/mh"))
rig = mh.Rig(os.path.join(ROOT, "target/art/mh"))
v = mh.shaped(base, BODY)
sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), "Shadow Reaver")
verts, faces, face_uvs, uvs, W, bo, br, marks = fit.fit(base, rig, v, sk, GIRTH, HEAD_SCALE)

if clip:
    po, pr = sk.pose(sk.clips[clip]["frames"][frame])
    posed = np.zeros_like(verts)
    for j in range(16):
        m = W[:, j] > 0
        if not m.any():
            continue
        inv = fit.q_mat(br[j]).T
        local = (verts[m] - bo[j]) @ inv.T
        posed[m] += W[m, j:j + 1] * (po[j] + local @ fit.q_mat(pr[j]).T)
    verts = posed

stage.reset()
ob = stage.mesh_object("body", stage.y_up_to_blender(verts), faces, uvs, face_uvs)
ob.data.materials.append(stage.flat_material("skin", (0.62, 0.48, 0.42), 0.55))
stage.lights()
stage.turnaround(out)
print("wrote", out, "height", verts[:, 1].max())
