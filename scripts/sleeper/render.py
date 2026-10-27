"""A small software renderer for the Sleeper's sheets: triangles, a depth
buffer, one light, flat shading. Enough to judge a pose or a heightfield
from four sides without starting the game.

`view(...)` draws a mesh from a direction; `sheet(...)` lays several views
side by side with labels.
"""
import numpy as np
from PIL import Image, ImageDraw, ImageFont

try:
    FONT = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
except OSError:
    FONT = ImageFont.load_default()


def look_basis(direction, up=(0, 1, 0)):
    """Camera axes looking along `direction`: right, up, forward."""
    f = np.asarray(direction, float)
    f /= np.linalg.norm(f)
    r = np.cross(f, up)
    if np.linalg.norm(r) < 1e-6:
        r = np.cross(f, (0, 0, 1))
    r /= np.linalg.norm(r)
    u = np.cross(r, f)
    return r, u, f


def view(verts, tris, direction, size=(520, 400), colours=None, light=(-0.4, 0.8, 0.45), bones=None, pad=0.08, frame=None, up=(0, 1, 0), mirror=False, perspective=0.0):
    """Draw `verts`/`tris` looking along `direction` (orthographic), fitted
    to the image. `colours` is one RGB per triangle; `bones` is a list of
    (a, b) world points drawn as lines on top. `frame` fixes the fit to a
    set of points other than the mesh (so views of two meshes match).
    `mirror` flips the picture left to right: looking straight down with
    north up, a right-handed world has east on the left, and the map sheets
    put it on the right."""
    w, h = size
    r, u, f = look_basis(direction, up)
    pts = verts @ np.c_[r, u, f]
    if perspective:
        # A camera back along the view, so near things are bigger: the
        # mesh's far side is `perspective` times as far as its near side.
        depth_all = pts[:, 2]
        near_z, far_z = depth_all.min(), depth_all.max()
        dist = (far_z - near_z) / max(perspective - 1, 1e-6)
        scale_z = dist / (pts[:, 2] - near_z + dist)
        centre = pts[:, :2].mean(0)
        pts = np.c_[(pts[:, :2] - centre) * scale_z[:, None] + centre, pts[:, 2]]
    # Fitted to what is drawn: a point no triangle uses (a cut wing's) is not.
    ref = (frame @ np.c_[r, u, f]) if frame is not None else pts[np.unique(tris)]
    lo, hi = ref[:, :2].min(0), ref[:, :2].max(0)
    span = hi - lo
    scale = min(w * (1 - 2 * pad) / span[0], h * (1 - 2 * pad) / span[1])
    mid = (lo + hi) / 2

    def to_px(p):
        x = (p[..., 0] - mid[0]) * scale + w / 2
        y = h / 2 - (p[..., 1] - mid[1]) * scale
        return x, y

    sx, sy = to_px(pts)
    depth = np.full((h, w), np.inf)
    img = np.zeros((h, w, 3))
    img[:] = (0.93, 0.92, 0.89)
    L = np.asarray(light, float)
    L /= np.linalg.norm(L)
    a, b, c = verts[tris[:, 0]], verts[tris[:, 1]], verts[tris[:, 2]]
    n = np.cross(b - a, c - a)
    n /= np.linalg.norm(n, axis=1, keepdims=True) + 1e-12
    # Two-sided: a face is lit by whichever side faces the camera.
    n *= np.where((n @ f) > 0, -1, 1)[:, None]
    # Half the light from the sky, half from over the viewer's shoulder, so
    # no side of a body is a black shape whichever way it is seen from.
    shade = 0.3 + 0.35 * np.clip(n @ L, 0, 1) + 0.35 * np.clip(-(n @ f), 0, 1)
    if colours is None:
        colours = np.tile([0.62, 0.58, 0.52], (len(tris), 1))
    for k, (i, j, l) in enumerate(tris):
        xs = np.array([sx[i], sx[j], sx[l]])
        ys = np.array([sy[i], sy[j], sy[l]])
        zs = np.array([pts[i, 2], pts[j, 2], pts[l, 2]])
        x0, x1 = int(max(xs.min(), 0)), int(min(xs.max() + 1, w))
        y0, y1 = int(max(ys.min(), 0)), int(min(ys.max() + 1, h))
        if x0 >= x1 or y0 >= y1:
            continue
        gx, gy = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
        d = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
        if abs(d) < 1e-9:
            continue
        w0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / d
        w1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / d
        w2 = 1 - w0 - w1
        inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
        z = w0 * zs[0] + w1 * zs[1] + w2 * zs[2]
        sub = depth[y0:y1, x0:x1]
        hit = inside & (z < sub)
        sub[hit] = z[hit]
        img[y0:y1, x0:x1][hit] = np.asarray(colours[k]) * shade[k]
    out = Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8))
    if mirror:
        out = out.transpose(Image.FLIP_LEFT_RIGHT)
    if bones:
        draw = ImageDraw.Draw(out)
        for p, q in bones:
            (px, py), (qx, qy) = (to_px(np.array(p) @ np.c_[r, u, f]), to_px(np.array(q) @ np.c_[r, u, f]))
            draw.line([(px, py), (qx, qy)], fill=(200, 40, 40), width=2)
    return out


def sheet(panels, columns=2, title=None):
    """Lay (label, image) panels in a grid, a label over each."""
    w, h = panels[0][1].size
    rows = (len(panels) + columns - 1) // columns
    top = 30 if title else 0
    out = Image.new("RGB", (w * columns, top + rows * (h + 22)), (255, 255, 255))
    draw = ImageDraw.Draw(out)
    if title:
        draw.text((8, 6), title, fill=(0, 0, 0), font=FONT)
    for k, (label, im) in enumerate(panels):
        x, y = (k % columns) * w, top + (k // columns) * (h + 22)
        draw.text((x + 6, y + 3), label, fill=(0, 0, 0), font=FONT)
        out.paste(im, (x, y + 22))
    return out


def camera(verts, tris, colours, eye, target, fov=58.0, size=(640, 380), sky=(0.80, 0.86, 0.86), reach=700.0,
           light=(-0.4, 0.8, 0.45)):
    """Draw `verts`/`tris` as the game's camera would see them: from `eye`
    looking at `target`, `fov` degrees top to bottom, hazed toward `sky`
    with distance as the game's fog is, gone by `reach` metres. For judging
    terrain from where a player stands, not from an orthographic sheet."""
    w, h = size
    eye = np.asarray(eye, float)
    r, u, f = look_basis(np.asarray(target, float) - eye)
    rel = verts - eye
    cam = np.c_[rel @ r, rel @ u, rel @ f]
    focal = (h / 2) / np.tan(np.radians(fov) / 2)
    near = 1.0
    depth = np.full((h, w), np.inf)
    img = np.zeros((h, w, 3))
    img[:] = sky
    L = np.asarray(light, float)
    L /= np.linalg.norm(L)
    a, b, c = verts[tris[:, 0]], verts[tris[:, 1]], verts[tris[:, 2]]
    n = np.cross(b - a, c - a)
    n /= np.linalg.norm(n, axis=1, keepdims=True) + 1e-12
    n *= np.where(np.einsum("ij,ij->i", n, (a + b + c) / 3 - eye) > 0, -1, 1)[:, None]
    shade = 0.35 + 0.65 * np.clip(n @ L, 0, 1)
    z = cam[:, 2]
    sx = w / 2 + focal * cam[:, 0] / np.maximum(z, near)
    sy = h / 2 - focal * cam[:, 1] / np.maximum(z, near)
    keep = (z[tris] > near).all(1)
    sky = np.asarray(sky)
    for k in np.nonzero(keep)[0]:
        i, j, l = tris[k]
        xs = np.array([sx[i], sx[j], sx[l]])
        ys = np.array([sy[i], sy[j], sy[l]])
        x0, x1 = int(max(xs.min(), 0)), int(min(xs.max() + 1, w))
        y0, y1 = int(max(ys.min(), 0)), int(min(ys.max() + 1, h))
        if x0 >= x1 or y0 >= y1:
            continue
        gx, gy = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
        d = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
        if abs(d) < 1e-9:
            continue
        w0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / d
        w1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / d
        w2 = 1 - w0 - w1
        inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
        # Depth interpolated as 1/z, as a camera sees it.
        iz = w0 / z[i] + w1 / z[j] + w2 / z[l]
        zz = 1 / np.maximum(iz, 1e-9)
        sub = depth[y0:y1, x0:x1]
        hit = inside & (zz < sub)
        sub[hit] = zz[hit]
        fog = np.clip(zz[hit] / reach, 0, 1) ** 1.5
        col = np.asarray(colours[k]) * shade[k]
        img[y0:y1, x0:x1][hit] = col * (1 - fog[:, None]) + sky * fog[:, None]
    return Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8))
