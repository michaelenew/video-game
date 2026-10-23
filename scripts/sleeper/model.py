"""The Sleeper's model: load the dragon, pose its skeleton, skin its mesh.

The dragon is `assets/sleeper/dragon.glb`, Drummyfish's CC0 low-poly dragon
(`assets/sleeper/LICENSE.txt`): 2,416 vertices on a 58-bone skeleton. Nothing
here decides a pose; `pose.py` does. This is the arithmetic of glTF skinning,
with numpy and nothing else:

- a node's local transform is translation * rotation * scale;
- a bone's world transform is its parents' times its own;
- a vertex goes where the weighted sum of (bone world * inverse bind) puts it.
"""
import json
import struct
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
GLB = ROOT / "assets" / "sleeper" / "dragon.glb"

_TYPES = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
_COMPONENTS = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}


def quat_matrix(q):
    """A unit quaternion (x, y, z, w) as a 3x3 rotation."""
    x, y, z, w = q / np.linalg.norm(q)
    return np.array(
        [
            [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
            [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
            [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
        ]
    )


def axis_angle(axis, degrees):
    """A quaternion (x, y, z, w) turning `degrees` about `axis`."""
    a = np.asarray(axis, float)
    a = a / np.linalg.norm(a)
    h = np.radians(degrees) / 2
    return np.array([*(a * np.sin(h)), np.cos(h)])


def quat_mul(a, b):
    """The quaternion that turns by `b` and then by `a`."""
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return np.array(
        [
            aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz,
        ]
    )


class Model:
    """The dragon as loaded: its rest pose, its skeleton, its triangles."""

    def __init__(self, path=GLB):
        data = Path(path).read_bytes()
        n = struct.unpack("<I", data[12:16])[0]
        self.gltf = json.loads(data[20 : 20 + n])
        bin_at = 20 + n
        bin_len = struct.unpack("<I", data[bin_at : bin_at + 4])[0]
        self.bin = data[bin_at + 8 : bin_at + 8 + bin_len]
        g = self.gltf
        self.names = [nd.get("name", f"node{i}") for i, nd in enumerate(g["nodes"])]
        self.index = {name: i for i, name in enumerate(self.names)}
        self.parent = [-1] * len(g["nodes"])
        for i, nd in enumerate(g["nodes"]):
            for c in nd.get("children", []):
                self.parent[c] = i
        self.rest = [
            (
                np.array(nd.get("translation", [0, 0, 0]), float),
                np.array(nd.get("rotation", [0, 0, 0, 1]), float),
                np.array(nd.get("scale", [1, 1, 1]), float),
            )
            for nd in g["nodes"]
        ]
        prim = g["meshes"][0]["primitives"][0]
        self.positions = self.accessor(prim["attributes"]["POSITION"]).astype(float)
        self.joints = self.accessor(prim["attributes"]["JOINTS_0"]).astype(int)
        self.weights = self.accessor(prim["attributes"]["WEIGHTS_0"]).astype(float)
        self.triangles = self.accessor(prim["indices"]).astype(int).reshape(-1, 3)
        skin = g["skins"][0]
        self.skin_joints = skin["joints"]
        self.inverse_bind = self.accessor(skin["inverseBindMatrices"]).reshape(-1, 4, 4).transpose(0, 2, 1)
        # The mesh node's own transform, which the skin replaces.
        self.mesh_node = next(i for i, nd in enumerate(g["nodes"]) if "skin" in nd)

    def accessor(self, k):
        a = self.gltf["accessors"][k]
        view = self.gltf["bufferViews"][a["bufferView"]]
        width = _TYPES[a["type"]]
        dtype = _COMPONENTS[a["componentType"]]
        start = view.get("byteOffset", 0) + a.get("byteOffset", 0)
        stride = view.get("byteStride", 0)
        item = np.dtype(dtype).itemsize * width
        if stride and stride != item:
            rows = [
                np.frombuffer(self.bin, dtype, width, start + i * stride)
                for i in range(a["count"])
            ]
            return np.array(rows)
        out = np.frombuffer(self.bin, dtype, a["count"] * width, start)
        return out.reshape(a["count"], width) if width > 1 else out

    def clip_end(self, name):
        """Every bone's (translation, rotation, scale) at the last key of an
        animation, over the rest pose for what it does not move."""
        pose = [tuple(np.copy(v) for v in r) for r in self.rest]
        anim = next(a for a in self.gltf["animations"] if a["name"] == name)
        for ch in anim["channels"]:
            s = anim["samplers"][ch["sampler"]]
            out = self.accessor(s["output"])
            node, path = ch["target"]["node"], ch["target"]["path"]
            last = np.array(out[-1], float)
            t, r, sc = pose[node]
            if path == "translation":
                pose[node] = (last, r, sc)
            elif path == "rotation":
                pose[node] = (t, last, sc)
            elif path == "scale":
                pose[node] = (t, r, last)
        return pose

    def world(self, pose):
        """Each node's world transform, 4x4, for a pose."""
        out = [None] * len(pose)

        def of(i):
            if out[i] is not None:
                return out[i]
            t, r, s = pose[i]
            m = np.eye(4)
            m[:3, :3] = quat_matrix(r) * s
            m[:3, 3] = t
            out[i] = m if self.parent[i] < 0 else of(self.parent[i]) @ m
            return out[i]

        for i in range(len(pose)):
            of(i)
        return out

    def skin(self, pose):
        """The mesh's vertices where a pose puts them, in world space."""
        world = self.world(pose)
        mats = np.array([world[j] @ self.inverse_bind[k] for k, j in enumerate(self.skin_joints)])
        p = np.c_[self.positions, np.ones(len(self.positions))]
        out = np.zeros((len(p), 4))
        for slot in range(4):
            m = mats[self.joints[:, slot]]
            out += self.weights[:, slot, None] * np.einsum("nij,nj->ni", m, p)
        return out[:, :3]

    def bone_points(self, pose):
        """Where each bone starts, in world space, by name: for drawing the
        skeleton over the mesh."""
        world = self.world(pose)
        return {self.names[i]: world[i][:3, 3] for i in range(len(pose))}
