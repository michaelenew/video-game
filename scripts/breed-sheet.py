"""Draw lineage sheets from `breedcheck -- boxes` dumps.

    cargo run -p sim --bin breedcheck -- boxes ridgeback galewing feather 1 31 40 > lines/high_ridgeback_galewing_feather.txt
    python3 scripts/breed-sheet.py <dir with lines/>

One row per lineage, the generations across (always the ones where a major
moved), each body isometric on a gridded ground, one scale down the row. The
file name's first word (low, med, high) picks which sheet it goes on. The
sheets in docs/design/gallery/breed/ were made this way."""
import sys, os, glob
from PIL import Image, ImageDraw, ImageFont

S = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("-") else os.getcwd()
# --posed reads lines_posed/ (bodies mid-stride) and writes *_posed sheets.
POSED = "--posed" in sys.argv
# --flight reads lines_flight/ (wings spread, airborne if it flies) and
# writes *_flight sheets.
FLIGHT = "--flight" in sys.argv
FONT = ImageFont.load_default()
try:
    FONT = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 13)
    SMALL = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 11)
    BOLD = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 15)
except Exception:
    SMALL = BOLD = FONT

COL = {
    "torso": (120, 120, 130),
    "head": (200, 90, 70),
    "limb": (80, 120, 180),
    "foot": (50, 80, 130),
    "wing": (90, 170, 120),
    "feathered": (90, 170, 120),
    "membrane": (70, 150, 160),
    "furred": (140, 150, 90),
    "tail": (170, 140, 80),
}
BG = (250, 249, 245)


def parse(path):
    """`gen` lines and `obox` lines: every part as an oriented box -- centre,
    unit axis, half-extents (along, across, up) in the axis's frame."""
    gens = []
    for line in open(path):
        if line.startswith("gen "):
            head, desc = line.split("|", 1)
            parts = head.split()
            caps = " ".join(parts[5:]).strip()
            gens.append({
                "n": int(parts[1]), "stance": parts[2], "plaus": float(parts[3]),
                "tier": parts[4], "caps": caps, "desc": desc.strip(), "boxes": [],
            })
        elif line.startswith("obox "):
            f = line.split()
            tag, name = f[1], f[2]
            nums = [float(x) for x in f[3:12]]
            gens[-1]["boxes"].append((tag, name, *nums))
    for g in gens:
        g["corners"] = [corners(b) for b in g["boxes"]]
    return gens


def frame(u):
    """The frame an axis makes, as the builder makes it: up turned
    perpendicular to the axis (or +x turned, for a vertical axis), and
    across = up x axis."""
    ux, uy, uz = u
    d = uy
    w = (0.0 - ux * d, 1.0 - uy * d, 0.0 - uz * d)
    if math.sqrt(w[0] ** 2 + w[1] ** 2 + w[2] ** 2) < 0.01:
        d = ux
        w = (1.0 - ux * d, 0.0 - uy * d, 0.0 - uz * d)
    l = math.sqrt(w[0] ** 2 + w[1] ** 2 + w[2] ** 2) or 1.0
    w = (w[0] / l, w[1] / l, w[2] / l)
    v = (w[1] * uz - w[2] * uy, w[2] * ux - w[0] * uz, w[0] * uy - w[1] * ux)
    return v, w


def corners(b):
    tag, name, cx, cy, cz, ux, uy, uz, ha, hv, hw = b
    u = (ux, uy, uz)
    v, w = frame(u)
    out = []
    for sa in (-1, 1):
        for sv in (-1, 1):
            for sw in (-1, 1):
                out.append((
                    cx + u[0] * ha * sa + v[0] * hv * sv + w[0] * hw * sw,
                    cy + u[1] * ha * sa + v[1] * hv * sv + w[1] * hw * sw,
                    cz + u[2] * ha * sa + v[2] * hv * sv + w[2] * hw * sw,
                ))
    return out


def majors(desc):
    """The plan's discrete part of a description: stance, head, segments, limbs' role/build."""
    seg = desc.split("|")
    # role and build only: the tip is a minor
    limbs = [":".join(l.strip().split(":")[:2]).rsplit("/", 1)[0] for l in seg[1].split(",") if l.strip()]
    return seg[0].strip(), tuple(limbs), seg[2].strip()


def pick_frames(gens, count=8):
    """Evenly spaced generations, but always the ones where a major moved."""
    idx = set(round(i * (len(gens) - 1) / (count - 1)) for i in range(count))
    prev = None
    moved = []
    for g in gens:
        m = majors(g["desc"])
        if prev is not None and m[:2] != prev[:2]:
            moved.append(g["n"])
        prev = m
    for n in moved:
        idx.add(n)
    idx = sorted(idx)
    # keep to `count` + the moved ones, dropping evenly spaced ones nearest a move
    while len(idx) > count + 4:
        for i in idx:
            if i not in moved and i not in (0, len(gens) - 1):
                idx.remove(i)
                break
    return idx, set(moved)


def short_limbs(limbs):
    out = []
    for l in limbs:
        st, rest = l.split(":", 1)
        role, build = rest.split("/")
        role = {"Both": "walk+strike", "Walk": "walk", "Strike": "strike", "Wing": "wing"}.get(role, role)
        st = {"Fore": "F", "MidFore": "MF", "MidHind": "MH", "Hind": "H"}.get(st, st)
        out.append(f"{st}:{role}/{build.lower()}")
    return "  ".join(out)


import math
C30 = math.cos(math.radians(30))
S30 = math.sin(math.radians(30))


def iso(x, y, z):
    """World (x along the body, y up, z across) to the picture plane, metres."""
    return (x - z) * C30, (x + z) * S30 - y


def shade(col, k):
    return tuple(max(0, min(255, int(c * k))) for c in col)


def draw_box(d, box, col, origin, scale):
    """A cuboid as its three visible faces: top, the +x end, the +z side."""
    x0, y0, x1, y1, z0, z1 = box
    ox, oy = origin

    def P(x, y, z):
        px, py = iso(x, y, z)
        return (ox + px * scale, oy + py * scale)

    top = [P(x0, y1, z0), P(x1, y1, z0), P(x1, y1, z1), P(x0, y1, z1)]
    front = [P(x1, y0, z0), P(x1, y0, z1), P(x1, y1, z1), P(x1, y1, z0)]
    side = [P(x0, y0, z1), P(x1, y0, z1), P(x1, y1, z1), P(x0, y1, z1)]
    edge = shade(col, 0.55)
    d.polygon(side, fill=shade(col, 0.78), outline=edge)
    d.polygon(front, fill=shade(col, 0.92), outline=edge)
    d.polygon(top, fill=shade(col, 1.15), outline=edge)


def draw_obox(d, b, col, origin, scale):
    """An oriented cuboid, its six faces far to near, lit by which way
    each faces."""
    tag, name, cx, cy, cz, ux, uy, uz, ha, hv, hw = b
    u = (ux, uy, uz)
    v, w = frame(u)
    ox, oy = origin
    halves = (ha, hv, hw)
    basis = (u, v, w)

    def corner(s):
        x = cx + sum(basis[i][0] * halves[i] * s[i] for i in range(3))
        y = cy + sum(basis[i][1] * halves[i] * s[i] for i in range(3))
        z = cz + sum(basis[i][2] * halves[i] * s[i] for i in range(3))
        px, py = iso(x, y, z)
        return (ox + px * scale, oy + py * scale, x + z + y * 0.5)

    faces = []
    for fixed in range(3):
        for sign in (-1, 1):
            quad = []
            for a, bb in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
                s = [0, 0, 0]
                s[fixed] = sign
                others = [i for i in range(3) if i != fixed]
                s[others[0]] = a
                s[others[1]] = bb
                quad.append(corner(s))
            depth = sum(q[2] for q in quad) / 4
            n = tuple(basis[fixed][i] * sign for i in range(3))
            # lit from above and a little from the front-right
            k = 0.72 + 0.42 * max(0.0, n[1]) + 0.14 * max(0.0, n[0]) + 0.1 * max(0.0, n[2])
            faces.append((depth, [(q[0], q[1]) for q in quad], shade(col, k)))
    faces.sort(key=lambda f: f[0])
    for _, quad, c in faces:
        d.polygon(quad, fill=c, outline=shade(col, 0.5))


def hull(points):
    """The convex hull of 2D points (monotone chain), for a shadow."""
    pts = sorted(set(points))
    if len(pts) < 3:
        return pts
    def cross2(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    lower = []
    for p in pts:
        while len(lower) >= 2 and cross2(lower[-2], lower[-1], p) <= 0:
            lower.pop()
        lower.append(p)
    upper = []
    for p in reversed(pts):
        while len(upper) >= 2 and cross2(upper[-2], upper[-1], p) <= 0:
            upper.pop()
        upper.append(p)
    return lower[:-1] + upper[:-1]


def draw_ground(d, origin, scale, extent, step):
    ox, oy = origin
    lo = -extent
    hi = extent

    def P(x, z):
        px, py = iso(x, 0, z)
        return (ox + px * scale, oy + py * scale)

    d.polygon([P(lo, lo), P(hi, lo), P(hi, hi), P(lo, hi)], fill=(232, 228, 214))
    n = int((hi - lo) / step)
    for i in range(n + 1):
        v = lo + i * step
        d.line([P(v, lo), P(v, hi)], fill=(214, 209, 192))
        d.line([P(lo, v), P(hi, v)], fill=(214, 209, 192))


def draw_strip(name, gens, cell_w=300, cell_h=300):
    frames, moved = pick_frames(gens)
    # One scale for the row, from the largest picture any frame makes.
    biggest = 1.0
    for i in frames:
        pts = [iso(x, y, z) for cs in gens[i]["corners"] for (x, y, z) in cs]
        pts += [iso(x, 0, z) for cs in gens[i]["corners"] for (x, y, z) in cs]
        w = max(p[0] for p in pts) - min(p[0] for p in pts)
        h = max(p[1] for p in pts) - min(p[1] for p in pts)
        biggest = max(biggest, w, h)
    scale = (min(cell_w, cell_h) - 70) / biggest
    row_extent = max(max(x for cs in gens[i]["corners"] for (x, _, _) in cs) -
                     min(x for cs in gens[i]["corners"] for (x, _, _) in cs) for i in frames)
    step = 5.0 if row_extent > 12 else 1.0
    head_h = 34
    foot_h = 30
    total_h = head_h + cell_h + foot_h
    W = cell_w * len(frames)
    H = total_h + 44
    im = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(im)
    d.text((8, 6), name.replace("_", " "), fill=(30, 30, 30), font=BOLD)
    d.text((8, 24), gens[0]["desc"][:260], fill=(90, 90, 90), font=SMALL)
    for k, i in enumerate(frames):
        g = gens[i]
        ox = k * cell_w
        top = 44
        outline = (200, 60, 60) if g["n"] in moved else (220, 218, 210)
        d.rectangle([ox + 2, top, ox + cell_w - 2, top + total_h - 2], outline=outline, width=2 if g["n"] in moved else 1)
        tile = Image.new("RGB", (cell_w - 6, cell_h), BG)
        td = ImageDraw.Draw(tile)
        allc = [c for cs in g["corners"] for c in cs]
        xs = [c[0] for c in allc]
        zs = [c[2] for c in allc]
        cx = (min(xs) + max(xs)) / 2
        cz = (min(zs) + max(zs)) / 2
        ys = max(c[1] for c in allc)
        mx, my = iso(cx, ys / 2, cz)
        origin = (tile.width / 2 - mx * scale, cell_h / 2 - my * scale + 10)
        extent = max(max(xs) - min(xs), max(zs) - min(zs)) * 0.9 + 2
        draw_ground(td, (origin[0] - iso(cx, 0, cz)[0] * scale + iso(0, 0, 0)[0], origin[1] - iso(cx, 0, cz)[1] * scale), scale, extent, step)
        # shadows: each box's footprint on the ground
        for cs in g["corners"]:
            foot = [iso(x, 0, z) for (x, _, z) in cs]
            poly = hull([(round(px, 4), round(py, 4)) for px, py in foot])
            if len(poly) >= 3:
                td.polygon([(origin[0] + px * scale, origin[1] + py * scale) for px, py in poly], fill=(205, 200, 184))
        # the boxes, far to near
        order = sorted(g["boxes"], key=lambda b: (b[2] + b[4] + b[3] * 0.5))
        for b in order:
            draw_obox(td, b, COL.get(b[0], (100, 100, 100)), origin, scale)
        im.paste(tile, (ox + 3, top + head_h))
        d.text((ox + 8, top + 4), f"gen {g['n']}  {g['stance']}  p={g['plaus']:.2f}  tier {g['tier']}", fill=(30, 30, 30), font=FONT)
        d.text((ox + 8, top + 20), g["caps"][:52], fill=(90, 90, 90), font=SMALL)
        d.text((ox + 8, top + head_h + cell_h - 14), f"grid {step:g} m", fill=(150, 150, 150), font=SMALL)
        st, limbs, minors = majors(g["desc"])
        d.text((ox + 8, top + head_h + cell_h + 2), short_limbs(limbs)[:56], fill=(60, 80, 120), font=SMALL)
        d.text((ox + 8, top + head_h + cell_h + 14), minors[:56], fill=(120, 100, 60), font=SMALL)
    return im


def main():
    sub = "lines_flight" if FLIGHT else "lines_posed" if POSED else "lines"
    files = sorted(glob.glob(os.path.join(S, sub, "*.txt")))
    groups = {"low": [], "med": [], "high": []}
    for f in files:
        base = os.path.basename(f)[:-4]
        strip = draw_strip(base, parse(f))
        groups[base.split("_")[0]].append(strip)
    legend_h = 30
    posed = " -- in flight (wings spread; flared on the ground if they cannot lift it)" if FLIGHT else " -- posed mid-stride" if POSED else " -- at rest"
    for key, title in [("low", "Low: drift alone, no trophies, 60 generations" + posed),
                       ("med", "Medium: one trophy every four generations, 60 generations" + posed),
                       ("high", "High: a trophy every generation, 40 generations" + posed)]:
        strips = groups[key]
        W = max(s.width for s in strips)
        H = sum(s.height for s in strips) + 40 + legend_h
        sheet = Image.new("RGB", (W, H), BG)
        d = ImageDraw.Draw(sheet)
        d.text((8, 8), title, fill=(20, 20, 20), font=BOLD)
        x = 8
        for tag, col in COL.items():
            d.rectangle([x, 30, x + 14, 44], fill=col)
            d.text((x + 18, 30), tag, fill=(60, 60, 60), font=SMALL)
            x += 80
        d.text((x + 10, 30), "red frame = a major moved that generation", fill=(200, 60, 60), font=SMALL)
        y = 40 + legend_h
        for s in strips:
            sheet.paste(s, (0, y))
            y += s.height
        suffix = "_flight" if FLIGHT else "_posed" if POSED else ""
        out = os.path.join(S, f"lineages_{key}{suffix}.png")
        sheet.save(out)
        print(out, sheet.size)


if __name__ == "__main__":
    main()
