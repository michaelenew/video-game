"""PARKED (the owner, 2026-10-20: "facets are by far the best ... keep this
to the side for now"). Not part of any build; kept so the look can be made
again. See docs/design/cast-look.md, "Parked".

The Reaver faceted: her model, posed in the game's run, decimated with the
face and hands protected, every facet one flat colour from her painted
texture, lit in bands. Beside it the original, for comparison.

  target/art/venv/bin/python art/parked/facets.py out.png [clip] [frame] [view]
"""

import io
import json
import math
import os
import struct
import sys
import tempfile

import bpy
import bmesh
import numpy as np
from mathutils import Euler, Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "art/lib"))
import fit  # noqa: E402

OUT = sys.argv[1]
CLIP = sys.argv[2] if len(sys.argv) > 2 else "run_forward"
FRAME = int(sys.argv[3]) if len(sys.argv) > 3 else 5
VIEW = sys.argv[4] if len(sys.argv) > 4 else "three"

# ------------------------------------------------------------------ the glb
raw = open(os.path.join(ROOT, "crates/game/assets/reaver.glb"), "rb").read()
jlen = struct.unpack("<I", raw[12:16])[0]
G = json.loads(raw[20:20 + jlen])
BIN = raw[20 + jlen + 8:]
TYPES = {5126: np.float32, 5125: np.uint32, 5123: np.uint16, 5121: np.uint8}
COMP = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def acc(i):
    a = G["accessors"][i]
    v = G["bufferViews"][a["bufferView"]]
    off = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    dt = TYPES[a["componentType"]]
    n = a["count"] * COMP[a["type"]]
    arr = np.frombuffer(BIN, dtype=dt, count=n, offset=off)
    return arr.reshape(a["count"], COMP[a["type"]]) if COMP[a["type"]] > 1 else arr


PATHS = {}


def image(i):
    v = G["bufferViews"][G["images"][i]["bufferView"]]
    data = BIN[v.get("byteOffset", 0): v.get("byteOffset", 0) + v["byteLength"]]
    ext = ".png" if G["images"][i].get("mimeType") == "image/png" else ".jpg"
    path = os.path.join(tempfile.gettempdir(), f"img{i}{ext}")
    open(path, "wb").write(data)
    im = bpy.data.images.load(path)
    im.colorspace_settings.name = "Non-Color"
    w, h = im.size
    px = np.zeros(w * h * 4, np.float32)
    im.pixels.foreach_get(px)
    # Blender's rows run bottom to top; keep the file's order, top to bottom.
    PATHS[i] = path
    return px.reshape(h, w, 4)[::-1, :, :3].copy()


# ------------------------------------------------------------------ the pose
sk = fit.Skeleton(os.path.join(ROOT, "target/art/anim.json"), "Shadow Reaver")
clip = sk.clips[CLIP]
origin, rot = sk.pose(clip["frames"][FRAME % len(clip["frames"])])


def trs(t, q):
    x, y, z, w = q
    m = np.eye(4)
    m[:3, :3] = [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]
    m[:3, 3] = t
    return m


nodes = G["nodes"]
parent = {}
for i, nd in enumerate(nodes):
    for c in nd.get("children", []):
        parent[c] = i
names = [nd.get("name") for nd in nodes]
world = {}


def node_world(i):
    if i in world:
        return world[i]
    nm = names[i]
    if nm in fit.JOINTS:
        k = fit.JOINTS.index(nm)
        w = trs(origin[k], rot[k])  # rot is [x, y, z, w]
        # Joints sit flat under the model: their transform is the solve's.
        if i in parent and names[parent[i]] not in fit.JOINTS:
            w = node_world(parent[i]) @ w
    else:
        nd = nodes[i]
        local = trs(nd.get("translation", [0, 0, 0]), nd.get("rotation", [0, 0, 0, 1]))
        w = node_world(parent[i]) @ local if i in parent else local
    world[i] = w
    return w


skin = G["skins"][0]
ibm = acc(skin["inverseBindMatrices"]).reshape(-1, 4, 4).transpose(0, 2, 1)
skin_m = np.stack([node_world(j) @ ibm[k] for k, j in enumerate(skin["joints"])])
skin_names = [names[j] for j in skin["joints"]]

prims = []
for p in G["meshes"][0]["primitives"]:
    a = p["attributes"]
    pos = acc(a["POSITION"]).astype(np.float64)
    jt = acc(a["JOINTS_0"]).astype(int)
    wt = acc(a["WEIGHTS_0"]).astype(np.float64)
    uv = acc(a["TEXCOORD_0"]).astype(np.float64)
    idx = acc(p["indices"]).astype(int).reshape(-1, 3)
    h = np.c_[pos, np.ones(len(pos))]
    posed = np.zeros_like(pos)
    for k in range(4):
        m = skin_m[jt[:, k]]
        posed += wt[:, k:k + 1] * np.einsum("nij,nj->ni", m, h)[:, :3]
    detail = np.zeros(len(pos))
    for k in range(4):
        jn = np.array(skin_names)[jt[:, k]]
        hot = np.isin(jn, ["head", "neck", "hand.l", "hand.r"])
        detail += np.where(hot, wt[:, k], 0)
    mat = G["materials"][p["material"]]
    tex = mat["pbrMetallicRoughness"]["baseColorTexture"]["index"]
    src = G["textures"][tex]["source"]
    img = image(src)
    prims.append(dict(path=PATHS[src], name=mat["name"], pos=posed, uv=uv, idx=idx, img=img, detail=detail,
                      double=mat.get("doubleSided", False)))

# ------------------------------------------------------------------ the stage
bpy.ops.wm.read_factory_settings(use_empty=True)
S = bpy.context.scene
S.render.engine = "CYCLES"
S.cycles.device = "CPU"
S.cycles.samples = 48
S.cycles.use_denoising = True
S.view_settings.view_transform = "AgX"
S.view_settings.look = "AgX - Punchy"
S.view_settings.exposure = 0.45

world_sky = bpy.data.worlds.new("sky")
world_sky.use_nodes = True
nt = world_sky.node_tree
bg = nt.nodes["Background"]
tc = nt.nodes.new("ShaderNodeTexCoord")
sep = nt.nodes.new("ShaderNodeSeparateXYZ")
ramp = nt.nodes.new("ShaderNodeValToRGB")
nt.links.new(tc.outputs["Generated"], sep.inputs[0])
nt.links.new(sep.outputs["Z"], ramp.inputs[0])
els = ramp.color_ramp.elements
els[0].position, els[0].color = 0.0, (0.95, 0.55, 0.35, 1)
els[1].position, els[1].color = 0.35, (0.30, 0.38, 0.75, 1)
els.new(0.12).color = (0.62, 0.42, 0.62, 1)
nt.links.new(ramp.outputs[0], bg.inputs[0])
bg.inputs[1].default_value = 0.6
S.world = world_sky

sun = bpy.data.lights.new("sun", "SUN")
sun.energy = 4.5
sun.color = (1.0, 0.88, 0.72)
sun.angle = math.radians(2)
so = bpy.data.objects.new("sun", sun)
so.rotation_euler = Euler((math.radians(50), 0, math.radians(-150)))
S.collection.objects.link(so)


def to_blender(v):
    # glTF is y-up, the character facing +z; Blender is z-up.
    return np.c_[v[:, 0], -v[:, 2], v[:, 1]]


def textured(name, img_arr):
    m = bpy.data.materials.new("tex_" + name)
    m.use_nodes = True
    t = m.node_tree
    b = t.nodes["Principled BSDF"]
    im = bpy.data.images.new("img_" + name, img_arr.shape[1], img_arr.shape[0])
    rgba = np.concatenate([img_arr[::-1], np.ones(img_arr.shape[:2] + (1,), np.float32)], 2)
    im.pixels.foreach_set(rgba.ravel())
    im.colorspace_settings.name = "sRGB"
    tn = t.nodes.new("ShaderNodeTexImage")
    tn.image = im
    t.links.new(tn.outputs[0], b.inputs["Base Color"])
    b.inputs["Roughness"].default_value = 0.7
    return m


def faceted_material():
    if "facet" in bpy.data.materials:
        return bpy.data.materials["facet"]
    m = bpy.data.materials.new("facet")
    m.use_nodes = True
    t = m.node_tree
    out = t.nodes["Material Output"]
    for n in list(t.nodes):
        if n != out:
            t.nodes.remove(n)
    col = t.nodes.new("ShaderNodeVertexColor")
    col.layer_name = "facet"
    toon = t.nodes.new("ShaderNodeBsdfToon")
    toon.inputs["Size"].default_value = 0.6
    toon.inputs["Smooth"].default_value = 0.04
    t.links.new(col.outputs[0], toon.inputs["Color"])
    # The shade side keeps its colour, cooled toward violet.
    shade = t.nodes.new("ShaderNodeMix")
    shade.data_type = "RGBA"
    shade.blend_type = "MULTIPLY"
    shade.inputs["Factor"].default_value = 1.0
    shade.inputs["B"].default_value = (0.55, 0.45, 0.75, 1)
    t.links.new(col.outputs[0], shade.inputs["A"])
    fill = t.nodes.new("ShaderNodeEmission")
    fill.inputs["Strength"].default_value = 0.45
    t.links.new(shade.outputs["Result"], fill.inputs["Color"])
    gloss = t.nodes.new("ShaderNodeBsdfGlossy")
    gloss.inputs["Roughness"].default_value = 0.35
    gloss.inputs["Color"].default_value = (0.08, 0.07, 0.10, 1)
    add = t.nodes.new("ShaderNodeAddShader")
    t.links.new(toon.outputs[0], add.inputs[0])
    t.links.new(fill.outputs[0], add.inputs[1])
    add2 = t.nodes.new("ShaderNodeAddShader")
    t.links.new(add.outputs[0], add2.inputs[0])
    t.links.new(gloss.outputs[0], add2.inputs[1])
    t.links.new(add2.outputs[0], out.inputs[0])
    return m


def original(pr, x_off):
    pos = to_blender(pr["pos"]) + np.array([x_off, 0, 0])
    me = bpy.data.meshes.new(pr["name"])
    me.from_pydata(pos.tolist(), [], pr["idx"].tolist())
    uvl = me.uv_layers.new(name="UV")
    loops_v = np.zeros(len(me.loops), int)
    me.loops.foreach_get("vertex_index", loops_v)
    uv = pr["uv"][loops_v]
    uvl.data.foreach_set("uv", np.c_[uv[:, 0], 1 - uv[:, 1]].ravel())
    ob = bpy.data.objects.new(pr["name"], me)
    S.collection.objects.link(ob)
    m = bpy.data.materials.new("tex_" + pr["name"])
    m.use_nodes = True
    t = m.node_tree
    tn = t.nodes.new("ShaderNodeTexImage")
    tn.image = bpy.data.images.load(pr["path"])
    t.links.new(tn.outputs[0], t.nodes["Principled BSDF"].inputs["Base Color"])
    t.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.7
    me.materials.append(m)
    for p in me.polygons:
        p.use_smooth = True
    return ob


# Everything posed, as one soup in Blender space, with what each triangle is:
# which piece (so which texture) and its corners' UVs.
ALLV, ALLF, ALLUV, ALLP = [], [], [], []
base = 0
for k, pr in enumerate(prims):
    ALLV.append(to_blender(pr["pos"]))
    ALLF.append(pr["idx"] + base)
    ALLUV.append(pr["uv"])
    ALLP.append(np.full(len(pr["idx"]), k))
    base += len(pr["pos"])
ALLV = np.concatenate(ALLV)
ALLF = np.concatenate(ALLF)
ALLUV = np.concatenate(ALLUV)
ALLP = np.concatenate(ALLP)

from mathutils.bvhtree import BVHTree  # noqa: E402

TREE = BVHTree.FromPolygons([Vector(v) for v in ALLV], ALLF.tolist())

# Where the detail is: the face and the hands, from the solve.
def middle(j):
    i = fit.JOINTS.index(j)
    return to_blender(np.array([(origin[i] + sk.tip(origin, rot, i)) / 2]))[0]


# (where, how far the care reaches): the face more than the hands.
def at(j):
    return to_blender(np.array([origin[fit.JOINTS.index(j)]]))[0]


HOT = [(middle("head"), 0.13), (middle("hand.l"), 0.07), (middle("hand.r"), 0.07)]
# The joints, where the shape turns: shoulders, elbows, knees, ankles.
HOT += [(at(j), 0.07) for j in ("arm.l", "arm.r", "forearm.l", "forearm.r",
                                 "shin.l", "shin.r", "foot.l", "foot.r")]


FACTOR = 3.0


def faceted(x_off, voxel, ratio, hard=False):
    me = bpy.data.meshes.new("shell")
    me.from_pydata(ALLV.tolist(), [], ALLF.tolist())
    ob = bpy.data.objects.new("shell", me)
    S.collection.objects.link(ob)
    bpy.context.view_layer.objects.active = ob
    ob.select_set(True)
    # One closed skin over everything: the whole figure as a distance field
    # (OpenVDB, as `garment::shell` makes her clothes), a few millimetres
    # out, gaps narrower than a centimetre bridged, back to a mesh. Open
    # cloth becomes a thin slab rather than confusing inside with out.
    gap, close = 0.004, 0.002
    tree = bpy.data.node_groups.new("over", "GeometryNodeTree")
    tree.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    tree.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    n = tree.nodes
    gi, go = n.new("NodeGroupInput"), n.new("NodeGroupOutput")
    # Points over every surface, a ball round each: the union is the figure
    # thickened, whether a piece was closed or a single sheet of cloth.
    pts = n.new("GeometryNodeDistributePointsOnFaces")
    pts.distribute_method = "RANDOM"
    pts.inputs["Density"].default_value = 60000.0
    sdf = n.new("GeometryNodePointsToSDFGrid")
    sdf.inputs["Radius"].default_value = gap + close
    sdf.inputs["Voxel Size"].default_value = voxel
    back = n.new("GeometryNodeSDFGridOffset")
    back.inputs["Distance"].default_value = -close
    to = n.new("GeometryNodeGridToMesh")
    L = tree.links
    L.new(gi.outputs[0], pts.inputs["Mesh"])
    L.new(pts.outputs["Points"], sdf.inputs["Points"])
    L.new(sdf.outputs[0], back.inputs["Grid"])
    L.new(back.outputs[0], to.inputs["Grid"])
    L.new(to.outputs[0], go.inputs[0])
    gm = ob.modifiers.new("over", "NODES")
    gm.node_group = tree
    bpy.ops.object.modifier_apply(modifier="over")
    print("SHELL", len(ob.data.polygons))
    # Detail: weight 1 near the face and hands, falling off over 12 cm.
    vg = ob.vertex_groups.new(name="detail")
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    co = co.reshape(-1, 3)
    w = np.max([np.clip(1.0 - np.linalg.norm(co - h, axis=1) / r, 0, 1) for h, r in HOT], axis=0)
    for i in np.nonzero(w > 0.01)[0]:
        vg.add([int(i)], float(w[i]), "REPLACE")
    # In two steps: straight to a few thousand from four hundred thousand,
    # the collapse caves a figure in.
    for r in (0.06, ratio / 0.06):
        dec = ob.modifiers.new("dec", "DECIMATE")
        dec.decimate_type = "COLLAPSE"
        dec.ratio = r
        dec.vertex_group = "detail"
        dec.invert_vertex_group = True
        dec.vertex_group_factor = FACTOR
        bpy.ops.object.modifier_apply(modifier="dec")
    ob.select_set(False)
    if hard:
        # Hems drawn to points: every vertex below the knee that is lower
        # than all its neighbours is pulled down and out a little, so the
        # coat's edge hangs in points rather than a line.
        me0 = ob.data
        n = len(me0.vertices)
        co = np.zeros(n * 3)
        me0.vertices.foreach_get("co", co)
        co = co.reshape(-1, 3)
        ev = np.zeros(len(me0.edges) * 2, int)
        me0.edges.foreach_get("vertices", ev)
        ev = ev.reshape(-1, 2)
        low = np.full(n, np.inf)
        np.minimum.at(low, ev[:, 0], co[ev[:, 1], 2])
        np.minimum.at(low, ev[:, 1], co[ev[:, 0], 2])
        centre = co.mean(0)
        tip = (co[:, 2] < low) & (co[:, 2] > 0.12) & (co[:, 2] < 0.75)
        out = co[:, :2] - centre[:2]
        out /= np.maximum(np.linalg.norm(out, axis=1, keepdims=True), 1e-6)
        co[tip, 2] -= 0.07
        co[tip, :2] += out[tip] * 0.025
        me0.vertices.foreach_set("co", co.ravel())
        me0.update()
        print("HEM POINTS", int(tip.sum()))
    me = ob.data
    # Each facet's colour: the original surface under the facet's middle,
    # and a few points toward its corners, averaged.
    attr = me.color_attributes.new("facet", "FLOAT_COLOR", "CORNER")
    cols = np.zeros((len(me.loops), 4))
    verts = np.zeros(len(me.vertices) * 3)
    me.vertices.foreach_get("co", verts)
    verts = verts.reshape(-1, 3)
    for p in me.polygons:
        vs = verts[list(p.vertices)]
        mid = vs.mean(0)
        acc_c = np.zeros(3)
        pts = [mid] + [mid * 0.5 + v * 0.5 for v in vs]
        for q in pts:
            loc, nrm, fi, dist = TREE.find_nearest(Vector(q))
            if fi is None:
                continue
            tri = ALLF[fi]
            a, b, c = ALLV[tri]
            # barycentric
            v0, v1, v2 = b - a, c - a, np.array(loc) - a
            d00, d01, d11 = v0 @ v0, v0 @ v1, v1 @ v1
            d20, d21 = v2 @ v0, v2 @ v1
            den = d00 * d11 - d01 * d01 or 1e-12
            bv = (d11 * d20 - d01 * d21) / den
            cv = (d00 * d21 - d01 * d20) / den
            av = 1 - bv - cv
            uv = ALLUV[tri[0]] * av + ALLUV[tri[1]] * bv + ALLUV[tri[2]] * cv
            img = prims[ALLP[fi]]["img"]
            hh, ww = img.shape[:2]
            acc_c += img[int(np.clip(uv[1], 0, 0.9999) * hh), int(np.clip(uv[0], 0, 0.9999) * ww)]
        c = (acc_c / len(pts)) ** 2.2
        for li in p.loop_indices:
            cols[li, :3] = c
            cols[li, 3] = 1
    attr.data.foreach_set("color", cols.ravel())
    me.materials.clear()
    me.materials.append(faceted_material())
    for p in me.polygons:
        p.use_smooth = False
    if hard:
        # Gem-cut: every edge sharper than forty degrees bevelled into a
        # narrow strip of its own, in a lighter tone, which catches the
        # light the way a cut stone's edge does.
        me.materials.append(edge_material())
        bpy.context.view_layer.objects.active = ob
        ob.select_set(True)
        bv = ob.modifiers.new("cut", "BEVEL")
        bv.limit_method = "ANGLE"
        bv.angle_limit = math.radians(32)
        bv.width = 0.013
        bv.segments = 1
        bv.material = 1
        bv.harden_normals = False
        bpy.ops.object.modifier_apply(modifier="cut")
        ob.select_set(False)
        for p in ob.data.polygons:
            p.use_smooth = False
    ob.location.x = x_off
    return ob


def edge_material():
    if "edge" in bpy.data.materials:
        return bpy.data.materials["edge"]
    m = faceted_material().copy()
    m.name = "edge"
    t = m.node_tree
    col = next(n for n in t.nodes if n.bl_idname == "ShaderNodeVertexColor")
    lift = t.nodes.new("ShaderNodeMix")
    lift.data_type = "RGBA"
    lift.blend_type = "SCREEN"
    lift.inputs["Factor"].default_value = 0.85
    lift.inputs["B"].default_value = (0.75, 0.6, 1.0, 1)
    t.links.new(col.outputs[0], lift.inputs["A"])
    for link in list(t.links):
        if link.from_node == col:
            to = link.to_socket
            t.links.remove(link)
            t.links.new(lift.outputs["Result"], to)
    t.links.new(col.outputs[0], lift.inputs["A"])
    return m


counts = {}
layout = {"three": [(-1.15, None, "original"), (0.0, (0.005, 0.0042), "faceted"), (1.15, (0.005, 0.0042, True), "hard")],
          "one": [(0.0, (0.005, 0.0042, True), "hard")]}[VIEW]
for x, how, label in layout:
    if how is None:
        tri = 0
        for pr in prims:
            ob = original(pr, x)
            tri += len(ob.data.polygons)
    else:
        ob = faceted(x, *how)
        tri = sum(len(p.vertices) - 2 for p in ob.data.polygons)
    counts[label] = tri
print("TRIANGLES", counts)

bm = bmesh.new()
bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=30)
me = bpy.data.meshes.new("floor")
bm.to_mesh(me)
floor = bpy.data.objects.new("floor", me)
fm = bpy.data.materials.new("sand")
fm.use_nodes = True
fm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.82, 0.40, 0.20, 1)
me.materials.append(fm)
S.collection.objects.link(floor)

S.render.use_freestyle = True
S.render.line_thickness_mode = "ABSOLUTE"
S.render.line_thickness = 1.1
fs = S.view_layers[0].freestyle_settings
ls = fs.linesets[0] if len(fs.linesets) else fs.linesets.new("lines")
if ls.linestyle is None:
    ls.linestyle = bpy.data.linestyles.new("ink")
ls.select_crease = True
fs.crease_angle = math.radians(118)
ls.select_border = False
ls.linestyle.color = (0.06, 0.03, 0.09)
ls.linestyle.alpha = 0.8

cam = bpy.data.cameras.new("cam")
co = bpy.data.objects.new("cam", cam)
S.collection.objects.link(co)
S.camera = co
if VIEW == "three":
    S.render.resolution_x, S.render.resolution_y = (1800, 1000)
    cam.lens = 42
    eye, look = Vector((-1.6, -4.4, 1.45)), Vector((0.0, 0.0, 0.92))
else:
    S.render.resolution_x, S.render.resolution_y = (900, 1100)
    cam.lens = 60
    eye, look = Vector((-1.5, -2.8, 1.6)), Vector((0.0, 0.0, 1.0))
co.location = eye
co.rotation_euler = (look - eye).to_track_quat("-Z", "Y").to_euler()
S.render.filepath = OUT
bpy.ops.render.render(write_still=True)
print("WROTE", OUT)
