"""Draw how crossbred creatures move: every move as a strip of frames with
its hit, and every gait as a cycle with its footfall diagram.

    python3 scripts/breed-motion.py <outdir> ridgeback mantis ...          # moves_<name>.png
    python3 scripts/breed-motion.py --gaits <outdir> ridgeback mantis ...  # gaits_<name>.png

Reads `breedcheck -- film` and `breedcheck -- strides`, which it runs. The
bodies are drawn as `breed-sheet.py` draws them, on a ground gridded in
world metres so a move that travels shows the grid sliding under it.

In a move's strip: red spheres are what the strike hits on that frame
(fainter, the frames of the sweep before it), and the orange rings on the
floor through the windup are where it will land -- the floor marker the
bestiary asks for (§1.4), drawn from the hit itself."""
import sys, os, subprocess, importlib.util, math
from PIL import Image, ImageDraw

# Loading breed-sheet.py as a module must not leave a __pycache__ in
# scripts/: the manual lists everything there.
sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("sheet", os.path.join(HERE, "breed-sheet.py"))
sheet = importlib.util.module_from_spec(spec)
sys.argv, argv = sys.argv[:1], sys.argv
spec.loader.exec_module(sheet)
sys.argv = argv

iso, draw_obox, hull, corners, COL, BG = sheet.iso, sheet.draw_obox, sheet.hull, sheet.corners, sheet.COL, sheet.BG
FONT, SMALL, BOLD = sheet.FONT, sheet.SMALL, sheet.BOLD


def run(*args):
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "sim", "--bin", "breedcheck", "--", *args],
        capture_output=True, text=True, check=True, cwd=os.path.dirname(HERE))
    return out.stdout.splitlines()


def obox(line):
    f = line.split()
    return (f[1], f[2], *[float(x) for x in f[3:12]])


def parse_film(lines):
    moves = []
    for line in lines:
        if line.startswith("move "):
            head, name = line.split("|", 1)
            p = head.split()
            moves.append({"kind": name.strip(), "startup": int(p[2]), "active": int(p[3]),
                          "recovery": int(p[4]), "answer": p[5].replace("_", " "), "lane": p[6],
                          "range": p[7], "beyond": float(p[8]), "verdict": p[9], "frames": []})
        elif line.startswith("frame "):
            p = line.split()
            moves[-1]["frames"].append({"n": int(p[1]), "beat": p[2], "boxes": [], "hits": []})
        elif line.startswith("obox "):
            moves[-1]["frames"][-1]["boxes"].append(obox(line))
        elif line.startswith("hit "):
            p = line.split()
            # x y z radius height this-frame: a height is an upright disc
            # on the floor (a shockwave), nought a ball.
            moves[-1]["frames"][-1]["hits"].append((*[float(x) for x in p[1:6]], p[6] == "1"))
    return moves


def parse_strides(lines):
    gaits = []
    for line in lines:
        if line.startswith("gait "):
            p = line.split()
            gaits.append({"kind": p[1], "froude": float(p[2]), "speed": float(p[3]),
                          "stride": float(p[4]), "cycle": int(p[5]), "duty": float(p[6]),
                          "feet": [], "frames": []})
        elif line.startswith("foot "):
            p = line.split()
            gaits[-1]["feet"].append((int(p[1]), int(p[2]), float(p[3])))
        elif line.startswith("frame "):
            gaits[-1]["frames"].append({"phase": float(line.split()[1]), "boxes": [], "hits": []})
        elif line.startswith("obox "):
            gaits[-1]["frames"][-1]["boxes"].append(obox(line))
    return gaits


def ground(d, origin, scale, cx, cz, extent, step):
    """A patch of floor round (cx, cz), its grid on world multiples of
    `step`, so the grid stays put while the body moves over it."""
    ox, oy = origin

    def P(x, z):
        px, py = iso(x, 0, z)
        return (ox + px * scale, oy + py * scale)

    lo_x, hi_x, lo_z, hi_z = cx - extent, cx + extent, cz - extent, cz + extent
    d.polygon([P(lo_x, lo_z), P(hi_x, lo_z), P(hi_x, hi_z), P(lo_x, hi_z)], fill=(232, 228, 214))
    v = math.floor(lo_x / step) * step
    while v <= hi_x:
        if v >= lo_x:
            d.line([P(v, lo_z), P(v, hi_z)], fill=(214, 209, 192))
        v += step
    v = math.floor(lo_z / step) * step
    while v <= hi_z:
        if v >= lo_z:
            d.line([P(lo_x, v), P(hi_x, v)], fill=(214, 209, 192))
        v += step


def ring(d, origin, scale, x, z, r, col, width=2):
    ox, oy = origin
    pts = []
    for k in range(24):
        a = 2 * math.pi * k / 24
        px, py = iso(x + r * math.cos(a), 0, z + r * math.sin(a))
        pts.append((ox + px * scale, oy + py * scale))
    d.line(pts + [pts[0]], fill=col, width=width)


def draw_cell(frame, scale, centre, cell_w, cell_h, step, marks=(), extent=None):
    """One body on its patch of floor, with its hit spheres and floor marks."""
    tile = Image.new("RGB", (cell_w, cell_h), BG)
    td = ImageDraw.Draw(tile)
    cs = [corners(b) for b in frame["boxes"]]
    allc = [c for cc in cs for c in cc]
    cx, cz, cy = centre
    mx, my = iso(cx, cy, cz)
    origin = (cell_w / 2 - mx * scale, cell_h / 2 - my * scale + 8)
    ground(td, origin, scale, cx, cz, extent, step)
    for (x, z, r) in marks:
        ring(td, origin, scale, x, z, r, (235, 150, 40))
    for cc in cs:
        poly = hull([(round(iso(x, 0, z)[0], 4), round(iso(x, 0, z)[1], 4)) for (x, _, z) in cc])
        if len(poly) >= 3:
            td.polygon([(origin[0] + px * scale, origin[1] + py * scale) for px, py in poly], fill=(205, 200, 184))
    for b in sorted(frame["boxes"], key=lambda b: (b[2] + b[4] + b[3] * 0.5)):
        draw_obox(td, b, COL.get(b[0], (100, 100, 100)), origin, scale)
    if frame["hits"]:
        over = Image.new("RGBA", tile.size, (0, 0, 0, 0))
        od = ImageDraw.Draw(over)
        for (x, y, z, r, hgt, now) in frame["hits"]:
            fill = (220, 40, 40, 110) if now else (220, 40, 40, 30)
            edge = (170, 20, 20, 220) if now else None
            if hgt > 0:
                # A disc on the floor: its rim at the bottom and the top.
                for yy in (y, y + hgt):
                    pts = []
                    for k in range(32):
                        a = 2 * math.pi * k / 32
                        qx, qy = iso(x + r * math.cos(a), yy, z + r * math.sin(a))
                        pts.append((origin[0] + qx * scale, origin[1] + qy * scale))
                    od.polygon(pts, fill=fill, outline=edge)
                continue
            px, py = iso(x, y, z)
            px, py = origin[0] + px * scale, origin[1] + py * scale
            rr = max(2.0, r * scale)
            od.ellipse([px - rr, py - rr, px + rr, py + rr], fill=fill, outline=edge)
        tile = Image.alpha_composite(tile.convert("RGBA"), over).convert("RGB")
    return tile


def fit(frames, cell_w, cell_h):
    """One scale for a strip, from the largest single frame; each frame is
    centred on itself (the grid shows any travel)."""
    biggest = 1.0
    for fr in frames:
        pts = [iso(x, y, z) for b in fr["boxes"] for (x, y, z) in corners(b)]
        pts += [iso(x, 0, z) for b in fr["boxes"] for (x, y, z) in corners(b)]
        w = max(p[0] for p in pts) - min(p[0] for p in pts)
        h = max(p[1] for p in pts) - min(p[1] for p in pts)
        biggest = max(biggest, w * cell_h / cell_w, h)
    return (min(cell_w, cell_h) - 50) / biggest


def centre_of(fr):
    cs = [c for b in fr["boxes"] for c in corners(b)]
    xs = [c[0] for c in cs]
    zs = [c[2] for c in cs]
    return ((min(xs) + max(xs)) / 2, (min(zs) + max(zs)) / 2, max(c[1] for c in cs) / 2,
            max(max(xs) - min(xs), max(zs) - min(zs)))


BEATS = ["rest", "a third in", "cocked (the tell)", "mid-strike", "struck", "end of the punish window", "recovered"]


def move_sheet(name, moves, cell_w=230, cell_h=230):
    rows = []
    for m in moves:
        frames = m["frames"]
        scale = fit(frames, cell_w, cell_h)
        size = max(centre_of(fr)[3] for fr in frames)
        step = 5.0 if size > 12 else 1.0
        # Where it will land: the last strike frame's spheres, on the floor.
        strike = [fr for fr in frames if fr["beat"] == "Strike"]
        marks = [(x, z, r) for (x, y, z, r, hgt, now) in (strike[-1]["hits"] if strike else []) if now]
        head_h, foot_h = 40, 20
        im = Image.new("RGB", (cell_w * len(frames), head_h + cell_h + foot_h), BG)
        d = ImageDraw.Draw(im)
        col = (190, 30, 30) if m["verdict"] == "UNANSWERABLE" else (30, 30, 30)
        d.text((8, 4), f"{m['kind']}   {m['startup']}f windup, {m['active']}f strike, {m['recovery']}f recovery"
                       f"   answer: {m['answer']}   {m['lane']}, {m['range']} ({m['beyond']:.1f} m)   {m['verdict']}",
               fill=col, font=FONT)
        # Centred on each frame along the floor (the grid shows travel) but
        # at the rest frame's height, so a leap shows as height.
        rest_cy = centre_of(frames[0])[2]
        for k, fr in enumerate(frames):
            cx, cz, _, sz = centre_of(fr)
            cy = rest_cy
            extent = sz * 0.6 + 2
            show = marks if fr["beat"] == "Windup" else ()
            tile = draw_cell(fr, scale, (cx, cz, cy), cell_w - 4, cell_h, step, show, extent)
            im.paste(tile, (k * cell_w + 2, head_h))
            outline = {"Windup": (235, 150, 40), "Strike": (200, 40, 40), "Stuck": (120, 60, 160)}.get(fr["beat"], (210, 208, 200))
            d.rectangle([k * cell_w + 1, head_h - 1, (k + 1) * cell_w - 2, head_h + cell_h], outline=outline)
            d.text((k * cell_w + 6, 22), f"f{fr['n']}  {BEATS[k] if k < len(BEATS) else fr['beat']}", fill=(80, 80, 80), font=SMALL)
            d.text((k * cell_w + 6, head_h + cell_h + 3), f"grid {step:g} m", fill=(150, 150, 150), font=SMALL)
        rows.append(im)
    return stack(f"{name}: every move, windup to recovery", rows,
                 "orange frame = windup (orange rings: where it lands), red = strike (spheres: what it hits), purple = held, the punish window")


def footfall(gait, w, h):
    """A Hildebrand diagram: one bar per foot, dark where it is on the floor."""
    im = Image.new("RGB", (w, h), BG)
    d = ImageDraw.Draw(im)
    names = ["fore", "midfore", "midhind", "hind"]
    feet = gait["feet"]
    d.text((6, 4), "footfalls, one cycle", fill=(60, 60, 60), font=SMALL)
    if not feet:
        d.text((6, 24), "(no feet: a slither or a flight)", fill=(120, 120, 120), font=SMALL)
        return im
    top, left, right = 22, 74, w - 10
    bar = min(16, (h - top - 20) // max(1, len(feet)))
    for k, (st, side, off) in enumerate(feet):
        y = top + k * bar
        d.text((6, y), f"{names[st]} {'L' if side == 0 else 'R'}", fill=(60, 60, 60), font=SMALL)
        d.rectangle([left, y + 2, right, y + bar - 2], fill=(236, 232, 220))
        # Planted from (1 - offset) for a duty's share of the cycle.
        start = (1.0 - off) % 1.0
        for (a, b) in [(start, start + gait["duty"])]:
            for (s, e) in [(a, min(b, 1.0)), (0.0, max(0.0, b - 1.0))]:
                if e > s:
                    d.rectangle([left + (right - left) * s, y + 2, left + (right - left) * e, y + bar - 2], fill=(70, 90, 130))
    y = top + len(feet) * bar + 4
    d.text((left, y), "0", fill=(120, 120, 120), font=SMALL)
    d.text((right - 8, y), "1", fill=(120, 120, 120), font=SMALL)
    return im


def gait_sheet(name, gaits, cell_w=190, cell_h=190):
    rows = []
    for g in gaits:
        frames = g["frames"]
        scale = fit(frames, cell_w, cell_h)
        size = max(centre_of(fr)[3] for fr in frames)
        step = 5.0 if size > 12 else 1.0
        head_h = 26
        im = Image.new("RGB", (cell_w * (len(frames) + 2), head_h + cell_h + 4), BG)
        d = ImageDraw.Draw(im)
        d.text((8, 4), f"{g['kind']}   Froude {g['froude']:.2f}, {g['speed']:.1f} m/s, stride {g['stride']:.2f} m,"
                       f" {g['cycle']} frames a cycle, each foot down {g['duty'] * 100:.0f}% of it", fill=(30, 30, 30), font=FONT)
        for k, fr in enumerate(frames):
            cx, cz, cy, sz = centre_of(frames[0])
            tile = draw_cell(fr, scale, (cx, cz, cy), cell_w - 4, cell_h, step, (), sz * 0.6 + 2)
            im.paste(tile, (k * cell_w + 2, head_h))
            d.text((k * cell_w + 6, head_h + 2), f"{fr['phase']:.3f}", fill=(120, 120, 120), font=SMALL)
        im.paste(footfall(g, cell_w * 2 - 8, cell_h), (len(frames) * cell_w + 4, head_h))
        rows.append(im)
    return stack(f"{name}: every gait, slowest first, eight moments of its cycle", rows,
                 "the gait is picked by the Froude number (speed squared over gravity times hip height), as animals pick theirs")


def stack(title, rows, note):
    W = max(r.width for r in rows)
    H = sum(r.height for r in rows) + 50
    out = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(out)
    d.text((8, 6), title, fill=(20, 20, 20), font=BOLD)
    d.text((8, 26), note, fill=(110, 110, 110), font=SMALL)
    y = 46
    for r in rows:
        out.paste(r, (0, y))
        y += r.height
        d.line([(0, y - 1), (W, y - 1)], fill=(200, 198, 190))
    return out


def main():
    args = sys.argv[1:]
    gaits = "--gaits" in args
    args = [a for a in args if not a.startswith("--")]
    outdir, names = args[0], args[1:]
    os.makedirs(outdir, exist_ok=True)
    for name in names:
        if gaits:
            im = gait_sheet(name, parse_strides(run("strides", name)))
            path = os.path.join(outdir, f"gaits_{name.lower()}.png")
        else:
            im = move_sheet(name, parse_film(run("film", name)))
            path = os.path.join(outdir, f"moves_{name.lower()}.png")
        im.save(path)
        print(path, im.size)


if __name__ == "__main__":
    main()
