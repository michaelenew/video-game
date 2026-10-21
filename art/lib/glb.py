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
ALL = g.ALL


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


def _quat(R):
    """A rotation matrix's quaternion, [x, y, z, w]."""
    t = np.trace(R)
    if t > 0:
        s = np.sqrt(t + 1.0) * 2
        return np.array([(R[2, 1] - R[1, 2]) / s, (R[0, 2] - R[2, 0]) / s, (R[1, 0] - R[0, 1]) / s, 0.25 * s])
    i = int(np.argmax(np.diag(R)))
    j, k = (i + 1) % 3, (i + 2) % 3
    s = np.sqrt(1.0 + R[i, i] - R[j, j] - R[k, k]) * 2
    q = np.zeros(4)
    q[i] = 0.25 * s
    q[j] = (R[j, i] + R[i, j]) / s
    q[k] = (R[k, i] + R[i, k]) / s
    q[3] = (R[k, j] - R[j, k]) / s
    return q


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
        # Where the object stands, too: a piece placed by its object rather
        # than in its mesh would otherwise come out at the feet.
        mw = np.array(ob.matrix_world)
        co = co.reshape(-1, 3) @ mw[:3, :3].T + mw[:3, 3]
        co = g.b2g(co)
        loops = len(me.loops)
        lv = np.zeros(loops, dtype=np.int64)
        me.loops.foreach_get("vertex_index", lv)
        ln = np.zeros(loops * 3)
        me.corner_normals.foreach_get("vector", ln)
        ln = g.b2g(ln.reshape(-1, 3) @ np.linalg.inv(mw[:3, :3]))
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


def write(path, prims, materials, images, bind_origin, bind_rot, name="character", extras=None):
    """``prims``: from ``gather``. ``materials``: name -> dict with
    ``color`` (image index or None), ``mr`` (image index or None),
    ``emissive`` (image index or None), ``factor`` (rgba). ``images``: a list
    of PNG bytes.

    ``extras``: ``{"tails": [(name, parent, origin)], "markers": [(name,
    parent, point)]}`` -- the coat's joints, in ``garment.TAILS`` order, each
    a child node of its parent at its bind place (unturned); and points a
    joint carries that the game reads (a blade's tip), as children of it.
    The sixteen joints of the skeleton are top-level nodes, as the game
    sets them in character space; anything under one moves with it."""
    extras = extras or {"tails": [], "markers": []}
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

    def acc(arr, ctype, kind, target=None, minmax=False, normalized=False):
        a = np.ascontiguousarray(arr)
        view = add(a.tobytes(), target)
        e = {"bufferView": view, "componentType": ctype, "count": len(a), "type": kind}
        if normalized:
            e["normalized"] = True
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
        if m.get("normal") is not None:
            gm["normalTexture"] = {"index": m["normal"]}
        if m.get("emissive") is not None:
            gm["emissiveTexture"] = {"index": m["emissive"]}
            gm["emissiveFactor"] = [1.0, 1.0, 1.0]
        if m.get("double"):
            gm["doubleSided"] = True
        if m.get("mask"):
            gm["alphaMode"] = "MASK"
            gm["alphaCutoff"] = m["mask"]
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
        # Weights in a byte each, summing to exactly 255 (the rounding's
        # leftover put on the heaviest): four floats a vertex was a tenth
        # of the file, for a precision nobody can see.
        qw = np.round(tw * 255).astype(np.int32)
        qw[:, 0] += 255 - qw.sum(axis=1)
        uv = np.stack([d["uv"][:, 0], 1 - d["uv"][:, 1]], 1)
        # Texture coordinates in two bytes each when they lie in the
        # texture, as an atlas's do (a sixty-fifth of a texel at 2048).
        if uv.min() >= 0.0 and uv.max() <= 1.0:
            uv_acc = acc(np.round(uv * 65535).astype(np.uint16), U16, "VEC2", 34962, normalized=True)
        else:
            uv_acc = acc(uv.astype(np.float32), FLOAT, "VEC2", 34962)
        idx = d["idx"]
        small = idx.max() < 65536
        primitives.append({
            "attributes": {
                "POSITION": acc(d["pos"].astype(np.float32), FLOAT, "VEC3", 34962, True),
                "NORMAL": acc(d["nrm"].astype(np.float32) / np.maximum(
                    np.linalg.norm(d["nrm"], axis=1, keepdims=True), 1e-9).astype(np.float32),
                    FLOAT, "VEC3", 34962),
                "TEXCOORD_0": uv_acc,
                "JOINTS_0": acc(joints, U8, "VEC4", 34962),
                "WEIGHTS_0": acc(qw.astype(np.uint8), U8, "VEC4", 34962, normalized=True),
            },
            "indices": acc(idx.astype(np.uint16 if small else np.uint32), U16 if small else U32,
                           "SCALAR", 34963),
            "material": mi,
        })

    # Every joint's bind matrix in character space: the skeleton's, then the
    # coat's, which are unturned at their bind places.
    world = {jn: _mat4(o, q) for jn, o, q in zip(JOINTS, bind_origin, bind_rot)}
    parent_of = {}
    tails = {t[0]: t for t in extras["tails"]}
    for tn in g.TAILS:
        _, parent, origin = tails.get(tn, (tn, "root", list(bind_origin[0])))
        world[tn] = _mat4(np.asarray(origin, float), (0.0, 0.0, 0.0, 1.0))
        parent_of[tn] = parent
    count = len(ALL)
    ibm = np.stack([np.linalg.inv(world[jn]).T.astype(np.float32) for jn in ALL])
    ibm_acc = acc(ibm.reshape(count, 16), FLOAT, "MAT4")

    def local(child_world, parent):
        m = np.linalg.inv(world[parent]) @ child_world
        q = _quat(m[:3, :3])
        return [float(x) for x in m[:3, 3]], [float(x) for x in q]

    nodes = []
    index = {}
    for jn in ALL:
        index[jn] = len(nodes)
        if jn in parent_of:
            t, r = local(world[jn], parent_of[jn])
        else:
            j = JOINTS.index(jn)
            q = np.asarray(bind_rot[j], float)
            t, r = [float(x) for x in bind_origin[j]], [float(x) for x in q / np.linalg.norm(q)]
        nodes.append({"name": jn, "translation": t, "rotation": r})
    for mname, parent, point in extras["markers"]:
        m = np.eye(4)
        m[:3, 3] = point
        t, _ = local(m, parent)
        index[mname] = len(nodes)
        nodes.append({"name": mname, "translation": t})
        parent_of[mname] = parent
    for child, parent in parent_of.items():
        nodes[index[parent]].setdefault("children", []).append(index[child])
    roots = [i for i, nd in enumerate(nodes) if nd["name"] not in parent_of]
    nodes.append({"name": name, "mesh": 0, "skin": 0})
    roots.append(len(nodes) - 1)

    gl_images, textures = [], []
    for i, png in enumerate(images):
        view = add(png)
        mime = "image/jpeg" if png[:3] == b"\xff\xd8\xff" else "image/png"
        gl_images.append({"bufferView": view, "mimeType": mime})
        textures.append({"source": i, "sampler": 0})

    doc = {
        "asset": {"version": "2.0", "generator": "art/lib/glb.py"},
        "scene": 0,
        "scenes": [{"name": name, "nodes": roots}],
        "nodes": nodes,
        "meshes": [{"name": name, "primitives": primitives}],
        "skins": [{"joints": [index[jn] for jn in ALL], "inverseBindMatrices": ibm_acc}],
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
