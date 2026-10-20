"""A skinned character as one binary glTF file, written by hand.

Blender has a glTF exporter, but it builds its skeleton from Blender's own
bones, which point along +y and carry a roll, and the game's joints are
neither. Here the joints are exactly the game's sixteen, each a node at its
bind pose, all children of the scene's root, so the game can set a joint's
transform straight from ``view::skeleton::Skin`` -- origin and rotation in
character space -- and the inverse bind matrices make the rest true.

The frame is the game's: +y up, +z forward, metres. One mesh, one primitive
per material; four joints a vertex.
"""

import json
import struct

import bmesh
import bpy
import numpy as np

import garment as g

JOINTS = g.JOINTS


def _pad(b, fill=b"\x00"):
    return b + fill * ((4 - len(b) % 4) % 4)


def _mat4(origin, quat):
    x, y, z, w = quat
    R = np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ])
    M = np.eye(4)
    M[:3, :3] = R
    M[:3, 3] = origin
    return M


def gather(objects):
    """Triangles of every object, split where a corner's uv or normal
    differs, with joint weights. Returns per material name a dict of arrays."""
    out = {}
    dg = bpy.context.evaluated_depsgraph_get()
    for ob in objects:
        ev = ob.evaluated_get(dg)
        me = ev.to_mesh()
        bm = bmesh.new()
        bm.from_mesh(me)
        bmesh.ops.triangulate(bm, faces=bm.faces)
        bm.to_mesh(me)
        bm.free()
        me.update()
        W = g.weights_of(ob)
        n = len(me.vertices)
        co = np.zeros(n * 3)
        me.vertices.foreach_get("co", co)
        co = g.b2g(co.reshape(-1, 3))
        loops = len(me.loops)
        lv = np.zeros(loops, dtype=np.int64)
        me.loops.foreach_get("vertex_index", lv)
        ln = np.zeros(loops * 3)
        me.corner_normals.foreach_get("vector", ln)
        ln = g.b2g(ln.reshape(-1, 3))
        if me.uv_layers.active:
            uv = np.zeros(loops * 2)
            me.uv_layers.active.data.foreach_get("uv", uv)
            uv = uv.reshape(-1, 2)
        else:
            uv = np.zeros((loops, 2))
        polys = len(me.polygons)
        pm = np.zeros(polys, dtype=np.int64)
        me.polygons.foreach_get("material_index", pm)
        ploop = np.zeros(polys, dtype=np.int64)
        me.polygons.foreach_get("loop_start", ploop)
        names = [m.name if m else "default" for m in me.materials] or ["default"]
        for mi, mname in enumerate(names):
            sel = np.nonzero(pm == mi)[0]
            if len(sel) == 0:
                continue
            corner = (ploop[sel][:, None] + np.arange(3)[None]).ravel()
            key = np.concatenate([
                lv[corner][:, None].astype(float),
                np.round(uv[corner] * 8192),
                np.round(ln[corner] * 512),
            ], axis=1)
            uniq, first, inverse = np.unique(key, axis=0, return_index=True, return_inverse=True)
            c = corner[first]
            vi = lv[c]
            d = out.setdefault(mname, {"pos": [], "nrm": [], "uv": [], "w": [], "idx": [], "base": 0})
            d["pos"].append(co[vi])
            d["nrm"].append(ln[c])
            d["uv"].append(uv[c])
            d["w"].append(W[vi])
            d["idx"].append(inverse.ravel() + d["base"])
            d["base"] += len(uniq)
        ev.to_mesh_clear()
    for d in out.values():
        for k in ("pos", "nrm", "uv", "w", "idx"):
            d[k] = np.concatenate(d[k])
    return out


def write(path, prims, materials, images, bind_origin, bind_rot, name="character"):
    """``prims``: from ``gather``. ``materials``: name -> dict with
    ``color`` (image index or None), ``mr`` (image index or None),
    ``emissive`` (image index or None), ``factor`` (rgba). ``images``: a list
    of PNG bytes."""
    blob = bytearray()
    views, accessors = [], []

    def add(data, target=None):
        off = len(blob)
        blob.extend(_pad(data))
        v = {"buffer": 0, "byteOffset": off, "byteLength": len(data)}
        if target:
            v["target"] = target
        views.append(v)
        return len(views) - 1

    def acc(arr, ctype, kind, target=None, minmax=False):
        a = np.ascontiguousarray(arr)
        view = add(a.tobytes(), target)
        e = {"bufferView": view, "componentType": ctype, "count": len(a), "type": kind}
        if minmax:
            e["min"] = a.min(axis=0).tolist()
            e["max"] = a.max(axis=0).tolist()
        accessors.append(e)
        return len(accessors) - 1

    FLOAT, U16, U32, U8 = 5126, 5123, 5125, 5121
    order = list(prims.keys())
    gl_mats = []
    for mname in order:
        m = materials.get(mname, {})
        pbr = {"baseColorFactor": list(m.get("factor", (1, 1, 1, 1))),
               "metallicFactor": 1.0 if m.get("mr") is not None else 0.0,
               "roughnessFactor": 1.0 if m.get("mr") is not None else 0.8}
        if m.get("color") is not None:
            pbr["baseColorTexture"] = {"index": m["color"]}
        if m.get("mr") is not None:
            pbr["metallicRoughnessTexture"] = {"index": m["mr"]}
        gm = {"name": mname, "pbrMetallicRoughness": pbr}
        if m.get("emissive") is not None:
            gm["emissiveTexture"] = {"index": m["emissive"]}
            gm["emissiveFactor"] = [1.0, 1.0, 1.0]
        if m.get("double"):
            gm["doubleSided"] = True
        gl_mats.append(gm)

    primitives = []
    for mi, mname in enumerate(order):
        d = prims[mname]
        W = d["w"]
        top = np.argsort(-W, axis=1)[:, :4]
        tw = np.take_along_axis(W, top, axis=1)
        tw = tw / np.maximum(tw.sum(axis=1, keepdims=True), 1e-9)
        joints = top.astype(np.uint8)
        joints[tw == 0] = 0
        primitives.append({
            "attributes": {
                "POSITION": acc(d["pos"].astype(np.float32), FLOAT, "VEC3", 34962, True),
                "NORMAL": acc(d["nrm"].astype(np.float32) / np.maximum(
                    np.linalg.norm(d["nrm"], axis=1, keepdims=True), 1e-9).astype(np.float32),
                    FLOAT, "VEC3", 34962),
                "TEXCOORD_0": acc(np.stack([d["uv"][:, 0], 1 - d["uv"][:, 1]], 1).astype(np.float32),
                                  FLOAT, "VEC2", 34962),
                "JOINTS_0": acc(joints, U8, "VEC4", 34962),
                "WEIGHTS_0": acc(tw.astype(np.float32), FLOAT, "VEC4", 34962),
            },
            "indices": acc(d["idx"].astype(np.uint32), U32, "SCALAR", 34963),
            "material": mi,
        })

    ibm = np.stack([np.linalg.inv(_mat4(o, q)).T.astype(np.float32) for o, q in zip(bind_origin, bind_rot)])
    ibm_acc = acc(ibm.reshape(16, 16), FLOAT, "MAT4")

    nodes = []
    for j, jn in enumerate(JOINTS):
        q = np.asarray(bind_rot[j], float)
        nodes.append({"name": jn, "translation": [float(x) for x in bind_origin[j]],
                      "rotation": [float(x) for x in q / np.linalg.norm(q)]})
    nodes.append({"name": name, "mesh": 0, "skin": 0})

    gl_images, textures = [], []
    for i, png in enumerate(images):
        view = add(png)
        mime = "image/jpeg" if png[:3] == b"\xff\xd8\xff" else "image/png"
        gl_images.append({"bufferView": view, "mimeType": mime})
        textures.append({"source": i, "sampler": 0})

    doc = {
        "asset": {"version": "2.0", "generator": "art/lib/glb.py"},
        "scene": 0,
        "scenes": [{"name": name, "nodes": list(range(len(nodes)))}],
        "nodes": nodes,
        "meshes": [{"name": name, "primitives": primitives}],
        "skins": [{"joints": list(range(16)), "inverseBindMatrices": ibm_acc}],
        "materials": gl_mats,
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": len(blob)}],
    }
    if gl_images:
        doc["images"] = gl_images
        doc["textures"] = textures
        doc["samplers"] = [{"magFilter": 9729, "minFilter": 9987, "wrapS": 33071, "wrapT": 33071}]
    js = _pad(json.dumps(doc, separators=(",", ":")).encode(), b" ")
    bin_ = _pad(bytes(blob))
    total = 12 + 8 + len(js) + 8 + len(bin_)
    with open(path, "wb") as f:
        f.write(struct.pack("<III", 0x46546C67, 2, total))
        f.write(struct.pack("<II", len(js), 0x4E4F534A))
        f.write(js)
        f.write(struct.pack("<II", len(bin_), 0x004E4942))
        f.write(bin_)
    return path
