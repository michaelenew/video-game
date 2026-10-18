"""Draw lineage sheets from `breedcheck -- boxes` dumps.

    cargo run -p sim --bin breedcheck -- boxes ridgeback galewing feather 1 31 40 > lines/high_ridgeback_galewing_feather.txt
    python3 scripts/breed-sheet.py <dir with lines/>

One row per lineage, the generations across (always the ones where a major
moved), each body isometric on a gridded ground, one scale down the row. The
file name's first word (low, med, high) picks which sheet it goes on. The
sheets in docs/design/gallery/breed/ were made this way."""
import sys, os, glob
from PIL import Image, ImageDraw, ImageFont

S = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
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
    "tail": (170, 140, 80),
}
BG = (250, 249, 245)


def parse(path):
    gens = []
    for line in open(path):
        if line.startswith("gen "):
            head, desc = line.split("|", 1)
            parts = head.split()
            caps = " ".join(parts[5:]).strip()
            gens.append({
                "n": int(parts[1]), "stance": parts[2], "plaus": float(parts[3]),
                "tier": parts[4], "caps": caps, "desc": desc.strip(), "parts": [],
            })
        elif line.startswith("part "):
            _, tag, name, x0, y0, x1, y1, z0, z1 = line.split()
            gens[-1]["parts"].append((tag, name, float(x0), float(y0), float(x1), float(y1), float(z0), float(z1)))
    return gens


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
        pts = []
        for (_, _, x0, y0, x1, y1, z0, z1) in gens[i]["parts"]:
            for x in (x0, x1):
                for y in (0, y0, y1):
                    for z in (z0, z1):
                        pts.append(iso(x, y, z))
        w = max(p[0] for p in pts) - min(p[0] for p in pts)
        h = max(p[1] for p in pts) - min(p[1] for p in pts)
        biggest = max(biggest, w, h)
    scale = (min(cell_w, cell_h) - 70) / biggest
    row_extent = max(max(x1 for (_, _, _, _, x1, _, _, _) in gens[i]["parts"]) -
                     min(x0 for (_, _, x0, _, _, _, _, _) in gens[i]["parts"]) for i in frames)
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
        # the picture: clip to the cell by drawing on a tile and pasting
        tile = Image.new("RGB", (cell_w - 6, cell_h), BG)
        td = ImageDraw.Draw(tile)
        parts = g["parts"]
        xs = [x for (_, _, x0, _, x1, _, _, _) in parts for x in (x0, x1)]
        zs = [z for (_, _, _, _, _, _, z0, z1) in parts for z in (z0, z1)]
        cx = (min(xs) + max(xs)) / 2
        cz = (min(zs) + max(zs)) / 2
        ys = max(y1 for (_, _, _, _, _, y1, _, _) in parts)
        # centre the body's middle, half its height up, in the tile
        mx, my = iso(cx, ys / 2, cz)
        origin = (tile.width / 2 - mx * scale, cell_h / 2 - my * scale + 10)
        extent = max(max(xs) - min(xs), max(zs) - min(zs)) * 0.9 + 2
        draw_ground(td, (origin[0] - iso(cx, 0, cz)[0] * scale + iso(0, 0, 0)[0], origin[1] - iso(cx, 0, cz)[1] * scale), scale, extent, step)
        # shadows: each box's footprint on the ground
        for (_, _, x0, y0, x1, y1, z0, z1) in parts:
            pts = [iso(x0, 0, z0), iso(x1, 0, z0), iso(x1, 0, z1), iso(x0, 0, z1)]
            td.polygon([(origin[0] + px * scale, origin[1] + py * scale) for px, py in pts], fill=(205, 200, 184))
        # the boxes, far to near
        order = sorted(parts, key=lambda p: (p[2] + p[6] + p[3] * 0.5))
        for tag, pname, x0, y0, x1, y1, z0, z1 in order:
            draw_box(td, (x0, y0, x1, y1, z0, z1), COL.get(tag, (100, 100, 100)), origin, scale)
        im.paste(tile, (ox + 3, top + head_h))
        d.text((ox + 8, top + 4), f"gen {g['n']}  {g['stance']}  p={g['plaus']:.2f}  tier {g['tier']}", fill=(30, 30, 30), font=FONT)
        d.text((ox + 8, top + 20), g["caps"][:52], fill=(90, 90, 90), font=SMALL)
        d.text((ox + 8, top + head_h + cell_h - 14), f"grid {step:g} m", fill=(150, 150, 150), font=SMALL)
        st, limbs, minors = majors(g["desc"])
        d.text((ox + 8, top + head_h + cell_h + 2), short_limbs(limbs)[:56], fill=(60, 80, 120), font=SMALL)
        d.text((ox + 8, top + head_h + cell_h + 14), minors[:56], fill=(120, 100, 60), font=SMALL)
    return im


def main():
    files = sorted(glob.glob(os.path.join(S, "lines", "*.txt")))
    groups = {"low": [], "med": [], "high": []}
    for f in files:
        base = os.path.basename(f)[:-4]
        strip = draw_strip(base, parse(f))
        groups[base.split("_")[0]].append(strip)
    legend_h = 30
    for key, title in [("low", "Low: drift alone, no trophies, 60 generations"),
                       ("med", "Medium: one trophy every four generations, 60 generations"),
                       ("high", "High: a trophy every generation, 40 generations")]:
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
        out = os.path.join(S, f"lineages_{key}.png")
        sheet.save(out)
        print(out, sheet.size)


main()
