"""Draw lineage sheets from `breedcheck -- boxes` dumps.

    cargo run -p sim --bin breedcheck -- boxes ridgeback galewing feather 1 31 40 > lines/high_ridgeback_galewing_feather.txt
    python3 scripts/breed-sheet.py <dir with lines/>

One row per lineage, the generations across (always the ones where a major
moved), each body in side view and top view at one scale down the row. The
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


def draw_strip(name, gens, cell_w=250, side_h=150, top_h=70):
    frames, moved = pick_frames(gens)
    xs = [x for i in frames for (_, _, x0, _, x1, _, _, _) in gens[i]["parts"] for x in (x0, x1)]
    span = max(xs) - min(xs)
    tallest = max(y1 for i in frames for (_, _, _, _, _, y1, _, _) in gens[i]["parts"])
    widest = max(z1 - z0 for i in frames for (_, _, _, _, _, _, z0, z1) in gens[i]["parts"])
    zspan = max(max(z1 for (_, _, _, _, _, _, _, z1) in gens[i]["parts"]) -
                min(z0 for (_, _, _, _, _, _, z0, _) in gens[i]["parts"]) for i in frames)
    scale = min((cell_w - 16) / max(span, 1.0), (side_h - 8) / max(tallest, 1.0))
    tscale = min(scale, (top_h - 8) / max(zspan, 1.0))
    head_h = 34
    foot_h = 30
    cell_h = head_h + side_h + top_h + foot_h
    W = cell_w * len(frames)
    H = cell_h + 44
    im = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(im)
    d.text((8, 6), name.replace("_", " "), fill=(30, 30, 30), font=BOLD)
    d.text((8, 24), gens[0]["desc"][:230], fill=(90, 90, 90), font=SMALL)
    order = {"tail": 0, "limb": 1, "foot": 1, "torso": 2, "wing": 3, "head": 4}
    for k, i in enumerate(frames):
        g = gens[i]
        ox = k * cell_w
        top = 44
        ground = top + head_h + side_h
        tmid = ground + top_h / 2 + 2
        outline = (200, 60, 60) if g["n"] in moved else (220, 218, 210)
        d.rectangle([ox + 2, top, ox + cell_w - 2, top + cell_h - 2], outline=outline, width=2 if g["n"] in moved else 1)
        d.line([ox + 6, ground, ox + cell_w - 6, ground], fill=(150, 150, 150))
        d.line([ox + 6, ground + top_h + 2, ox + cell_w - 6, ground + top_h + 2], fill=(225, 223, 215))
        px = [x for (_, _, x0, _, x1, _, _, _) in g["parts"] for x in (x0, x1)]
        cx = (min(px) + max(px)) / 2
        parts = sorted(g["parts"], key=lambda p: order.get(p[0], 2))
        for tag, pname, x0, y0, x1, y1, z0, z1 in parts:
            col = COL.get(tag, (100, 100, 100))
            edge = tuple(max(0, c - 50) for c in col)
            X0 = ox + cell_w / 2 + (x0 - cx) * scale
            X1 = ox + cell_w / 2 + (x1 - cx) * scale
            d.rectangle([X0, ground - y1 * scale, X1, ground - y0 * scale], fill=col, outline=edge)
        for tag, pname, x0, y0, x1, y1, z0, z1 in parts:
            col = tuple(min(255, c + 35) for c in COL.get(tag, (100, 100, 100)))
            X0 = ox + cell_w / 2 + (x0 - cx) * tscale
            X1 = ox + cell_w / 2 + (x1 - cx) * tscale
            d.rectangle([X0, tmid + z0 * tscale, X1, tmid + z1 * tscale], fill=col)
        bar = 5.0 if span > 14 else 1.0
        d.line([ox + 10, ground - 6, ox + 10 + bar * scale, ground - 6], fill=(60, 60, 60), width=2)
        d.text((ox + 10, ground - 20), f"{bar:g} m", fill=(60, 60, 60), font=SMALL)
        d.text((ox + 8, top + 4), f"gen {g['n']}  {g['stance']}  p={g['plaus']:.2f}  tier {g['tier']}", fill=(30, 30, 30), font=FONT)
        d.text((ox + 8, top + 20), g["caps"][:44], fill=(90, 90, 90), font=SMALL)
        d.text((ox + 8, ground + 4), "top view", fill=(170, 170, 170), font=SMALL)
        st, limbs, minors = majors(g["desc"])
        d.text((ox + 8, ground + top_h + 6), short_limbs(limbs)[:46], fill=(60, 80, 120), font=SMALL)
        d.text((ox + 8, ground + top_h + 18), minors[:46], fill=(120, 100, 60), font=SMALL)
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
