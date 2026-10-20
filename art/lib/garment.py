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

# The coat's joints, after the skeleton's: two a panel, the top half and the
# bottom, swung by springs in the game rather than by the simulation.
TAILS = [f"tail.{k}.{s}" for k in range(4) for s in "ab"]
ALL = JOINTS + TAILS


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
    """An ``(n, len(ALL))`` array of an object's joint weights: the
    skeleton's sixteen, then the coat's."""
    n = len(ob.data.vertices)
    W = np.zeros((n, len(ALL)))
    index = {g.index: ALL.index(g.name) for g in ob.vertex_groups if g.name in ALL}
    for v in ob.data.vertices:
        for g in v.groups:
            if g.group in index:
                W[v.index, index[g.group]] = g.weight
    return W


def set_weights(ob, W):
    """Joint weights from an array whose columns are ``ALL``, or its first
    sixteen."""
    names = ALL[: W.shape[1]]
    for name in names:
        if name not in ob.vertex_groups:
            ob.vertex_groups.new(name=name)
    groups = [ob.vertex_groups[n] for n in names]
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
    fw = np.zeros((n, W.shape[1]))
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
    for name in ALL:
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
    # Only the joints' groups: a piece's ``hem`` and ``edge`` are the
    # painter's, and go on to the bake.
    for g in list(ob.vertex_groups):
        if g.name in ALL:
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

# Where shells are kept between runs: a shell is a pure function of the body
# and four numbers, and making one is most of what a build costs.
CACHE = None


def shell(body, gap, voxel=0.004, smooth=2, close=0.0):
    """The body inflated by ``gap`` metres: the surface everything ``gap``
    from the skin lies on, smoothed by ``smooth`` rounds over the voxels.

    This is the robust way to lift cloth off a body. Pushing each vertex out
    along its normal turns every thin thing -- an ear, a toe, a finger --
    into a spike, because their normals fan apart; a surface at a fixed
    distance from the skin simply rounds over them, as cloth does. The body
    is made a distance field (OpenVDB, through Blender's grid nodes), moved
    out, and turned back into a mesh. Each shell carries the body's weights.

    ``close`` bridges hollows narrower than about twice itself: the field is
    pushed out that much further and pulled back in (a morphological
    closing), so cloth spans the dip under a bust or between the shoulder
    blades instead of following the skin into it.
    """
    key = (round(gap, 5), voxel, smooth, round(close, 5))
    if key in _shells:
        return _shells[key]
    name = "shell_%.4f_%.4f_%d_%.4f" % (gap, voxel, smooth, close)
    path = None
    if CACHE:
        import hashlib
        import os
        co = np.zeros(len(body.data.vertices) * 3)
        body.data.vertices.foreach_get("co", co)
        stamp = hashlib.sha1(np.round(co, 5).tobytes()).hexdigest()[:12]
        path = os.path.join(CACHE, f"{name}_{stamp}.npz")
        if os.path.exists(path):
            d = np.load(path)
            me = bpy.data.meshes.new(name)
            me.from_pydata(d["verts"].tolist(), [], d["faces"].tolist())
            ob = link(bpy.data.objects.new(name, me))
            set_weights(ob, d["weights"])
            ob.hide_render = True
            _shells[key] = ob
            return ob
    ob = body.copy()
    ob.data = body.data.copy()
    ob.name = ob.data.name = name
    for m in list(ob.modifiers):
        ob.modifiers.remove(m)
    link(ob)
    tree = bpy.data.node_groups.new(f"inflate_{name}", "GeometryNodeTree")
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
    if path:
        import os
        os.makedirs(CACHE, exist_ok=True)
        co = np.zeros(len(ob.data.vertices) * 3)
        ob.data.vertices.foreach_get("co", co)
        faces = np.array([list(p.vertices) for p in ob.data.polygons], dtype=np.int32)
        np.savez(path, verts=co.reshape(-1, 3), faces=faces, weights=weights_of(ob))
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
    fw = np.zeros((n, W.shape[1]))
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


def _coords(ob):
    co = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("co", co)
    no = np.zeros(len(ob.data.vertices) * 3)
    ob.data.vertices.foreach_get("normal", no)
    return b2g(co.reshape(-1, 3)), b2g(no.reshape(-1, 3))


def _set_coords(ob, p):
    ob.data.vertices.foreach_set("co", g2b(p).ravel())
    ob.data.update()


def _noise(p, scale, seed):
    """Cheap smooth noise in 3D, -1 to 1: a sum of a few skewed sines."""
    q = p * scale
    out = np.zeros(len(p))
    rng = np.random.default_rng(seed)
    for _ in range(4):
        d = rng.normal(size=3)
        d /= np.linalg.norm(d)
        out += np.sin(q @ d * rng.uniform(0.7, 1.4) + rng.uniform(0, 6.28))
    return out / 4


def wrinkle(ob, centre, axis, reach, amp, wavelength, skew=0.6, seed=0, sharp=3.0, bias=0.35):
    """Folds where cloth is pushed together at a joint: ridges running round
    the limb, strongest at the bend and gone ``reach`` metres along it,
    wandering diagonally and in and out, as an elbow's or a knee's do.

    Displaces along the surface's normal by up to ``amp`` metres, so the
    piece it is run on is the *high* one, baked down to the game's mesh as a
    normal map -- folds the light falls into, for no triangles."""
    p, n = _coords(ob)
    axis = np.asarray(axis, float) / np.linalg.norm(axis)
    e1 = np.cross(axis, [0.0, 0.0, 1.0] if abs(axis[2]) < 0.9 else [1.0, 0.0, 0.0])
    e1 /= np.linalg.norm(e1)
    e2 = np.cross(axis, e1)
    d = p - np.asarray(centre, float)
    t = d @ axis
    theta = np.arctan2(d @ e2, d @ e1)
    fall = np.exp(-(t / reach) ** 2)
    phase = 2 * np.pi * t / wavelength + skew * np.sin(theta * 2 + seed) * 2.2 + 1.8 * _noise(p, 18.0, seed)
    ridge = (0.5 + 0.5 * np.sin(phase)) ** sharp
    # Ridges broken up round the limb, so they read as folds not rings.
    patchy = np.clip(0.55 + 0.6 * _noise(p, 9.0, seed + 5), 0, 1)
    _set_coords(ob, p + n * (amp * fall * patchy * (ridge - bias))[:, None])
    return ob


def drape_folds(ob, top, bottom, amp, count, seed=0, axis=(0.0, 1.0, 0.0), centre=(0.0, 0.0, 0.0)):
    """Long folds that hang from where cloth is held: round a vertical axis
    through ``centre``, ``count`` of them, growing from nothing at ``top`` to
    full at ``bottom`` (heights)."""
    p, n = _coords(ob)
    c = np.asarray(centre, float)
    theta = np.arctan2(p[:, 0] - c[0], p[:, 2] - c[2])
    h = np.clip((top - p[:, 1]) / max(top - bottom, 1e-3), 0, 1)
    phase = theta * count + 1.5 * _noise(p, 6.0, seed) + seed
    fold = np.sin(phase) ** 3
    _set_coords(ob, p + n * (amp * h * fold)[:, None])
    return ob


def fabric(ob, amp=0.0012, scale=60.0, seed=0):
    """A little unevenness all over, so no surface is machined smooth."""
    p, n = _coords(ob)
    _set_coords(ob, p + n * (amp * _noise(p, scale, seed))[:, None])
    return ob


def smooth_border(ob, rounds=12, factor=0.5):
    """Relax a piece's open edges into clean curves: each border vertex
    moved toward the middle of its two neighbours along the border, round
    after round. A cut across the voxels leaves a hem stepped like a stair,
    and a painter's trim along it shows every step."""
    bm = bmesh.new()
    bm.from_mesh(ob.data)
    border = [v for v in bm.verts if v.is_boundary]
    links = {}
    for v in border:
        links[v] = [e.other_vert(v) for e in v.link_edges if e.is_boundary]
    for _ in range(rounds):
        moved = {}
        for v, ns in links.items():
            if len(ns) == 2:
                mid = (ns[0].co + ns[1].co) * 0.5
                moved[v] = v.co.lerp(mid, factor)
        for v, co in moved.items():
            v.co = co
    bm.to_mesh(ob.data)
    bm.free()
    return ob
