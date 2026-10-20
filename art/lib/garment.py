"""Clothes cut from the body they are worn on.

A tailored garment follows the body under it, at a distance, with a hem. So
the cheapest way to make one that fits is the way a costume artist starts in
a sculpting program: take the part of the body it covers, lift it off along
the surface's normals, give it a thickness, and cut its edges clean with a
plane. What hangs instead of fitting -- a hood's drape, coat tails -- is laid
out the same way and then dropped under a cloth simulation (``drape``).

Every function works on Blender objects in Blender's frame (+z up, -y
forward, the character's right at +x). ``g2b`` turns a point in the game's
frame into it. Each piece keeps the body's sixteen joint weights by copying
them from the nearest point of the body (``bind_to``), so it moves as the
skin under it moves.
"""

import bmesh
import bpy
import numpy as np
from mathutils import Vector

JOINTS = [
    "root", "spine", "chest", "head",
    "arm.l", "forearm.l", "hand.l",
    "arm.r", "forearm.r", "hand.r",
    "thigh.l", "shin.l", "foot.l",
    "thigh.r", "shin.r", "foot.r",
]


def g2b(p):
    p = np.asarray(p, float)
    return np.stack([p[..., 0], -p[..., 2], p[..., 1]], axis=-1)


def b2g(p):
    p = np.asarray(p, float)
    return np.stack([p[..., 0], p[..., 2], -p[..., 1]], axis=-1)


def link(ob):
    bpy.context.scene.collection.objects.link(ob)
    return ob


def weights_of(ob):
    """An ``(n, 16)`` array of an object's joint weights."""
    n = len(ob.data.vertices)
    W = np.zeros((n, 16))
    index = {g.index: JOINTS.index(g.name) for g in ob.vertex_groups if g.name in JOINTS}
    for v in ob.data.vertices:
        for g in v.groups:
            if g.group in index:
                W[v.index, index[g.group]] = g.weight
    return W


def set_weights(ob, W):
    for name in JOINTS:
        if name not in ob.vertex_groups:
            ob.vertex_groups.new(name=name)
    groups = [ob.vertex_groups[n] for n in JOINTS]
    for j, g in enumerate(groups):
        idx = np.nonzero(W[:, j] > 1e-4)[0]
        for i in idx:
            g.add([int(i)], float(W[i, j]), "REPLACE")


def cut(body, keep_face, name):
    """A copy of the body with only the faces ``keep_face(centroid, weights)``
    says to keep. Centroids and weights are per face, in the game's frame."""
    me = body.data
    n = len(me.polygons)
    cents = np.zeros((n, 3))
    me.polygons.foreach_get("center", cents.ravel())
    cents = b2g(cents)
    W = weights_of(body)
    fw = np.zeros((n, 16))
    for p in me.polygons:
        fw[p.index] = W[list(p.vertices)].mean(axis=0)
    keep = keep_face(cents, fw)
    ob = body.copy()
    ob.data = me.copy()
    ob.name = ob.data.name = name
    ob.hide_render = False
    for m in list(ob.modifiers):
        ob.modifiers.remove(m)
    link(ob)
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    bm.faces.ensure_lookup_table()
    gone = [f for f in bm.faces if not keep[f.index]]
    bmesh.ops.delete(bm, geom=gone, context="FACES")
    loose = [v for v in bm.verts if not v.link_faces]
    bmesh.ops.delete(bm, geom=loose, context="VERTS")
    bm.to_mesh(ob.data)
    bm.free()
    ob.data.materials.clear()
    return ob


def lift(ob, gap, smooth=0):
    """Move every vertex out along its normal by ``gap`` metres, after
    ``smooth`` rounds of relaxing the shape (cloth does not follow every
    dip of the body under it)."""
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    if smooth:
        bmesh.ops.smooth_vert(bm, verts=bm.verts, factor=0.5, use_axis_x=True,
                              use_axis_y=True, use_axis_z=True)
        for _ in range(smooth - 1):
            bmesh.ops.smooth_vert(bm, verts=bm.verts, factor=0.5, use_axis_x=True,
                                  use_axis_y=True, use_axis_z=True)
    bm.normal_update()
    if callable(gap):
        for v in bm.verts:
            v.co += v.normal * gap(b2g(np.array(v.co)))
    else:
        for v in bm.verts:
            v.co += v.normal * gap
    bm.to_mesh(ob.data)
    bm.free()
    return ob


def trim(ob, point, normal):
    """Cut along a plane (game frame) and throw away what is on the
    ``normal`` side: a hem, a cuff, a boot's top."""
    co = Vector(g2b(point).tolist())
    no = Vector(g2b(normal).tolist())
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    geom = list(bm.verts) + list(bm.edges) + list(bm.faces)
    bmesh.ops.bisect_plane(bm, geom=geom, plane_co=co, plane_no=no, clear_outer=True)
    bm.to_mesh(ob.data)
    bm.free()
    return ob


def keep_side(ob, point, normal):
    """Keep only what is on the ``normal`` side of a plane."""
    return trim(ob, point, -np.asarray(normal, float))


def thicken(ob, thickness, rim=True):
    m = ob.modifiers.new("solid", "SOLIDIFY")
    m.thickness = thickness
    m.offset = 1.0
    m.use_rim = rim
    m.use_even_offset = True
    m.use_quality_normals = True
    return apply(ob, m)


def subdivide(ob, levels=1):
    m = ob.modifiers.new("subd", "SUBSURF")
    m.levels = levels
    m.render_levels = levels
    return apply(ob, m)


def apply(ob, modifier):
    bpy.context.view_layer.objects.active = ob
    with bpy.context.temp_override(object=ob, active_object=ob):
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    return ob


def bind_to(ob, body):
    """Copy the body's joint weights onto a piece, from the nearest point of
    the body's surface."""
    for name in JOINTS:
        if name not in ob.vertex_groups:
            ob.vertex_groups.new(name=name)
    m = ob.modifiers.new("weights", "DATA_TRANSFER")
    m.object = body
    m.use_vert_data = True
    m.data_types_verts = {"VGROUP_WEIGHTS"}
    m.vert_mapping = "POLYINTERP_NEAREST"
    m.layers_vgroup_select_src = "ALL"
    m.layers_vgroup_select_dst = "NAME"
    apply(ob, m)
    normalize(ob)
    return ob


def normalize(ob, top=4):
    W = weights_of(ob)
    # Four joints a vertex at most, as a game's skinning takes.
    order = np.argsort(-W, axis=1)
    mask = np.zeros_like(W, dtype=bool)
    rows = np.arange(len(W))[:, None]
    mask[rows, order[:, :top]] = True
    W = np.where(mask, W, 0.0)
    s = W.sum(axis=1, keepdims=True)
    W = np.where(s > 0, W / np.maximum(s, 1e-9), 0.0)
    for g in list(ob.vertex_groups):
        ob.vertex_groups.remove(g)
    set_weights(ob, W)


def join(objs, name):
    for o in bpy.context.selected_objects:
        o.select_set(False)
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    objs[0].name = name
    return objs[0]


def material(ob, mat):
    ob.data.materials.clear()
    ob.data.materials.append(mat)
    return ob


def collide(ob, thickness=0.004):
    m = ob.modifiers.new("collide", "COLLISION")
    ob.collision.thickness_outer = thickness
    ob.collision.cloth_friction = 5.0
    return m


def drape(ob, pin_group, frames=60, mass=0.25, stiff=15.0, bend=0.5, shrink=0.0):
    """Let a piece hang under gravity for ``frames`` frames, held by the
    vertices in ``pin_group``, against every object that has a collision
    modifier, and keep the shape it ends in."""
    s = bpy.context.scene
    m = ob.modifiers.new("cloth", "CLOTH")
    c = m.settings
    c.quality = 8
    c.mass = mass
    c.tension_stiffness = stiff
    c.compression_stiffness = stiff
    c.shear_stiffness = stiff
    c.bending_stiffness = bend
    c.air_damping = 2.0
    c.vertex_group_mass = pin_group
    c.shrink_min = shrink
    m.collision_settings.use_self_collision = False
    m.collision_settings.distance_min = 0.004
    m.point_cache.frame_start = 1
    m.point_cache.frame_end = frames
    s.frame_start = 1
    s.frame_end = frames
    for f in range(1, frames + 1):
        s.frame_set(f)
    dg = bpy.context.evaluated_depsgraph_get()
    ev = ob.evaluated_get(dg)
    co = np.zeros(len(ev.data.vertices) * 3)
    ev.data.vertices.foreach_get("co", co)
    ob.modifiers.remove(m)
    ob.data.vertices.foreach_set("co", co)
    ob.data.update()
    s.frame_set(1)
    return ob


_shells = {}


def shell(body, gap, voxel=0.004, smooth=2, close=0.0):
    """The body inflated by ``gap`` metres: the surface everything ``gap``
    from the skin lies on, smoothed by ``smooth`` rounds over the voxels.

    This is the robust way to lift cloth off a body. Pushing each vertex out
    along its normal turns every thin thing -- an ear, a toe, a finger --
    into a spike, because their normals fan apart; a surface at a fixed
    distance from the skin simply rounds over them, as cloth does. The body
    is made a distance field (OpenVDB, through Blender's grid nodes), moved
    out, and turned back into a mesh. Each shell carries the body's weights.
    """
    key = (round(gap, 5), voxel, smooth)
    if key in _shells:
        return _shells[key]
    ob = body.copy()
    ob.data = body.data.copy()
    ob.name = ob.data.name = f"shell_{gap:.3f}_{close:.3f}"
    for m in list(ob.modifiers):
        ob.modifiers.remove(m)
    link(ob)
    tree = bpy.data.node_groups.new(f"inflate_{gap}", "GeometryNodeTree")
    tree.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    tree.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    n = tree.nodes
    gi = n.new("NodeGroupInput")
    go = n.new("NodeGroupOutput")
    sdf = n.new("GeometryNodeMeshToSDFGrid")
    sdf.inputs["Voxel Size"].default_value = voxel
    sdf.inputs["Band Width"].default_value = max(3, int((gap + close) / voxel) + 4)
    off = n.new("GeometryNodeSDFGridOffset")
    off.inputs["Distance"].default_value = gap + close
    back = n.new("GeometryNodeSDFGridOffset")
    back.inputs["Distance"].default_value = -close
    mean = n.new("GeometryNodeSDFGridMean")
    mean.inputs["Width"].default_value = 1
    mean.inputs["Iterations"].default_value = smooth
    to = n.new("GeometryNodeGridToMesh")
    to.inputs["Threshold"].default_value = 0.0
    to.inputs["Adaptivity"].default_value = 0.0
    L = tree.links
    L.new(gi.outputs[0], sdf.inputs["Mesh"])
    L.new(sdf.outputs[0], off.inputs["Grid"])
    L.new(off.outputs[0], back.inputs["Grid"])
    L.new(back.outputs[0], mean.inputs["Grid"])
    L.new(mean.outputs[0], to.inputs["Grid"])
    L.new(to.outputs[0], go.inputs[0])
    m = ob.modifiers.new("inflate", "NODES")
    m.node_group = tree
    for gname in list(ob.vertex_groups):
        ob.vertex_groups.remove(gname)
    apply(ob, m)
    ob.data.materials.clear()
    bind_to(ob, body)
    ob.hide_render = True
    _shells[key] = ob
    return ob


def decimate(ob, ratio):
    m = ob.modifiers.new("dec", "DECIMATE")
    m.ratio = ratio
    m.use_collapse_triangulate = False
    return apply(ob, m)


def smooth_shape(ob, factor=0.5, repeat=5):
    m = ob.modifiers.new("smooth", "SMOOTH")
    m.factor = factor
    m.iterations = repeat
    return apply(ob, m)


def vertex_mask(ob, r, gch=None, b=None):
    """Per-vertex masks for the painter, as a colour attribute ``paint_mask``:
    red picks a material's second colour, green its third."""
    n = len(ob.data.vertices)
    gch = np.zeros(n) if gch is None else gch
    b = np.zeros(n) if b is None else b
    if "paint_mask" in ob.data.color_attributes:
        ob.data.color_attributes.remove(ob.data.color_attributes["paint_mask"])
    attr = ob.data.color_attributes.new("paint_mask", "FLOAT_COLOR", "POINT")
    rgba = np.stack([r, gch, b, np.ones(n)], 1).astype(np.float32)
    attr.data.foreach_set("color", rgba.ravel())


def drop_faces(ob, gone):
    """Delete the faces ``gone(centroids, weights)`` names (game frame)."""
    me = ob.data
    n = len(me.polygons)
    cents = np.zeros((n, 3))
    me.polygons.foreach_get("center", cents.ravel())
    cents = b2g(cents)
    W = weights_of(ob)
    fw = np.zeros((n, 16))
    for p in me.polygons:
        fw[p.index] = W[list(p.vertices)].mean(axis=0)
    kill = gone(cents, fw)
    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if kill[f.index]], context="FACES")
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bm.to_mesh(me)
    bm.free()


def hem(ob, reach=0.03):
    """A vertex group ``hem``: 1 on a piece's open edge, falling to 0
    ``reach`` metres in from it -- where a painter puts the trim, the
    stitching and the wear. Measured on the open shell, before it is given
    a thickness, and carried through the thickening with the other groups."""
    from mathutils.kdtree import KDTree
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    edge = [v.co.copy() for v in bm.verts if v.is_boundary]
    bm.free()
    g = ob.vertex_groups.get("hem") or ob.vertex_groups.new(name="hem")
    if not edge:
        return ob
    tree = KDTree(len(edge))
    for i, c in enumerate(edge):
        tree.insert(c, i)
    tree.balance()
    for v in ob.data.vertices:
        _, _, d = tree.find(v.co)
        wgt = max(0.0, 1.0 - d / reach)
        if wgt > 0:
            g.add([v.index], wgt, "REPLACE")
    return ob


def group_to_colour(ob, groups, name="detail"):
    """Copy up to three vertex groups into one colour attribute (r, g, b),
    so a shader -- and so a bake -- can read them."""
    n = len(ob.data.vertices)
    rgb = np.zeros((n, 4), dtype=np.float32)
    rgb[:, 3] = 1
    for c, gname in enumerate(groups):
        grp = ob.vertex_groups.get(gname)
        if grp is None:
            continue
        for v in ob.data.vertices:
            for ge in v.groups:
                if ge.group == grp.index:
                    rgb[v.index, c] = ge.weight
    if name in ob.data.color_attributes:
        ob.data.color_attributes.remove(ob.data.color_attributes[name])
    attr = ob.data.color_attributes.new(name, "FLOAT_COLOR", "POINT")
    attr.data.foreach_set("color", rgb.ravel())
