"""What the light being wears: plate, made of light.

A celestial knight (the owner, 2026-10-26: "a celestial knight. Not holy
... but light", from a reference "not far off, but probably too heavy on the
gold and a little too ornate -- a cool visual but not so dominant").
Matched to that reference piece by piece ("You're reading 'angular' as 'add
any angle'... match the reference beat for beat"), every plate faceted
(``facet``): broad flat faces, bevelled where they fold, a frame of gold
round each.

The shape (the owner, 2026-10-28: "narrower waist, wider shoulders, less
clunky"): in hard at the waist, the body under it too; the shoulders the
widest thing about it; every plate on a limb the limb's own size.

- Pauldrons: a cop of flat faces flared out to the side of the shoulder
  rather than down the arm, its upper border rising to one blade-like point
  at the height of the ear; one lame under it, pointed at its front lower
  corner.
- Chest: a breastplate of planes meeting at a keel; a kite down the
  sternum with the sun at its widest; over each breast three strips laid up
  and out toward the armpit, lapped as feathers; three chevrons down the
  belly; a slim belt with a kite of a buckle; a tabard to above the knee;
  two close lames of fauld round the hips.
- Arms: a band round the upper arm, a pointed cop at the elbow, a vambrace
  of twelve faces sized to the forearm, a fin up its outside past the elbow,
  its end over the back of the hand.
- Gauntlets: a plate over the back of each hand, a raised guard across the
  knuckles, two lames over the fingers.
- Legs: a cuisse, a pointed knee cop, a greave, each of twelve faces;
  plate boots.
- The closed helm the owner liked, faceted: an egg with a ridge from the
  chin over the crown to the nape; a faceted gorget; a slit of light for
  its eyes.

White plate, gold only in the frames; the plate gives a little light of
its own (``build.LUMINOUS``). No cloak for now. Nothing holy: no halo.
"""

import numpy as np

import character
import facet
import garment as g
import hard
import plate
import stage
from wardrobe import EXTRAS, LEFT, J, body_masks, boots, cape, covered, mat, rigid, trousers, w


def glowing(ob, weight=1.0):
    grp = ob.vertex_groups.get("edge") or ob.vertex_groups.new(name="edge")
    grp.add(list(range(len(ob.data.vertices))), weight, "REPLACE")
    return ob


def lit_rims(pieces, glow=0.35):
    """A plate's rims (every piece after the first ``plate.finish`` made),
    faintly lit."""
    plate_, *rims = pieces
    return [plate_] + [glowing(rim, glow) for rim in rims]


def armour(name, ax, us, vs, radius, lo=None, hi=None, closed=False, joint=None, thick=0.006, rim=0.0065,
           sharp=None):
    ob, borders = plate.surface(name, ax, us, vs, radius, lo, hi, closed=closed)
    return lit_rims(plate.finish(ob, borders, "plate", thick, rim, rim_material="trim", joint=joint,
                                 rim_closed=closed, sharp=sharp))


def ridge(us, centre, height, half):
    """A hard ridge across ``us``: ``height`` at ``centre``, falling straight
    to nothing ``half`` degrees either side -- two planes meeting at a crease,
    not a rounded swell."""
    return height * np.clip(1.0 - np.abs(us - centre) / half, 0.0, 1.0)


# How far its arms hang out from its body at rest, degrees: its own, wider
# than the dark being's (``view::being::REST_SPREAD``), and what every plate
# on the arm is drawn for (``facet.rest_frame``).
SPREAD = 18.0


def trunk_axis(r):
    hip = r.o["thigh.l"][1]
    return plate.Axis([0.0, hip, r.o["chest"][2]], [0, 1, 0], [0, 0, 1], [1, 0, 0])


def cuirass(body, r):
    """The trunk's plate, shaped as an armourer shapes it rather than as the
    body is -- an hourglass, in hard at the navel -- and built up as the
    reference's is (the module's list); behind, a backplate with a ridge
    down the spine; below, the fauld's four lames round the hips; a sun at
    the breast (``build.veins``)."""
    ax = trunk_axis(r)
    hip = r.o["thigh.l"][1]
    H = r.o["arm.l"][1] - hip
    neck = r.o["head"][1] - hip
    waist = 0.36 * H
    us_all = np.linspace(-180, 180, 73)
    vs = np.linspace(0.0, neck + 0.02, 44)
    W = g.weights_of(body)
    trunk = sum(W[:, J[n]] for n in ("root", "spine", "chest", "clavicle.l", "clavicle.r")) > 0.5
    body_r = plate.radii(body, ax, vs, us_all, keep=trunk, smooth_u=6, smooth_v=4, stiff=8)
    # The armourer's section, by height: (height as a share of hips to
    # shoulders, half-width, front, back), metres.
    keys = np.array([
        [0.00, 0.14, 0.11, 0.108],
        [0.36, 0.092, 0.088, 0.082],
        [0.55, 0.16, 0.145, 0.105],
        [0.74, 0.25, 0.175, 0.13],
        [0.92, 0.265, 0.145, 0.125],
        [1.10, 0.15, 0.11, 0.11],
    ])
    h = vs / H
    width = np.interp(h, keys[:, 0], keys[:, 1])
    front = np.interp(h, keys[:, 0], keys[:, 2])
    back = np.interp(h, keys[:, 0], keys[:, 3])
    design = plate.designed(us_all, vs, width, front, back)
    keel = ridge(us_all, 0.0, 0.026, 26.0)[None, :] * plate.smoothstep(waist, 0.6 * H, vs)[:, None]
    R = np.maximum(design + keel, body_r + 0.015)
    out = []
    # The breast, faceted (``facet.Shell``): at each key height a polygon
    # with a corner every 35 degrees -- two flat planes meeting at the keel
    # down the middle, two more turning to the sides -- a little proud of
    # the designed section, so its chords clear the body.
    # Narrow faces, so it reads lean rather than a box (the owner,
    # 2026-10-26: "a little too blocky ... lean and athletic").
    uk = np.array([-105.0, -84.0, -63.0, -42.0, -21.0, 0.0, 21.0, 42.0, 63.0, 84.0, 105.0])
    vk = keys[:, 0] * H
    Rk = plate.designed(uk, vk, keys[:, 1], keys[:, 2], keys[:, 3]) + 0.006
    Rk[:, 5] += 0.03 * plate.smoothstep(0.45 * H, 0.6 * H, vk) * (1 - plate.smoothstep(0.95 * H, 1.1 * H, vk))
    for i, v in enumerate(vk):
        row = np.array([np.interp(v, vs, body_r[:, j]) for j in range(len(us_all))])
        for j, u in enumerate(uk):
            near = np.abs(us_all - u) < 22
            Rk[i, j] = max(Rk[i, j], row[near].max() + 0.01)
    shell = facet.Shell(ax, uk, vk, Rk)
    inside = ax.o + 0.7 * H * ax.a

    def on(name, rows, **kw):
        return facet.plate(name, np.array(rows, float), shape=shell, inside=inside, **kw)

    cols = np.linspace(-100, 100, 17)
    ac = np.abs(cols)
    hi = (neck - 0.05 - 0.05 * (ac / 40.0) ** 2).clip(H - 0.07, None)
    hi = np.where(ac > 58, H - 0.07 - 0.13 * plate.smoothstep(58, 100, ac), hi)
    lo = waist - 0.01 + 0.025 * (ac / 100.0) ** 2
    t = np.linspace(0, 1, 7)[:, None]
    out.append(on("breastplate", np.stack([np.broadcast_to(cols, (7, 17)), lo + (hi - lo) * t,
                                           np.zeros((7, 17))], -1), per=2, frame=0.009, thick=0.007))
    # The kite down the sternum, widest at the breast, where the sun is.
    kite = [(neck - 0.075, 6.0), (0.8 * H, 19.0), (0.64 * H, 10.0), (0.5 * H, 1.5)]
    out.append(on("kite", [[(f * hw, v, 0.016) for f in (-1, -0.5, 0, 0.5, 1)] for v, hw in kite],
                  frame=0.007, per=3, thick=0.006))
    # Over each breast three strips, laid up and out from under the kite
    # toward the armpit, each lapping the one below it as feathers lie.
    for sg in (-1.0, 1.0):
        uu = np.linspace(12, 64, 6)
        for k in range(3):
            b = 0.53 * H + 0.05 * k
            base = b + 0.11 * (uu - 12) / 52.0
            rows = [[(sg * u, v, 0.006 + 0.003 * k) for u, v in zip(uu, base + dv)] for dv in (0.0, 0.065)]
            out.append(on(f"pec{k}.{'lr'[sg > 0]}", rows[::-1] if sg < 0 else rows, frame=0.008, per=3,
                          thick=0.005))
    # Down the belly three chevrons, each pointing down at the middle,
    # each lapping the one below.
    uu = np.linspace(-56, 56, 9)
    for k in range(3):
        top = 0.53 * H - 0.068 * k
        dip = 0.03 * (1 - np.abs(uu) / 56.0)
        rows = [[(u, top - d - dv, 0.011 - 0.002 * k) for u, d in zip(uu, dip)] for dv in (0.0, 0.085)]
        out.append(on(f"chevron{k}", rows, frame=0.008, per=2, thick=0.005))
    # The belt over the fauld's head, and its buckle, a kite pointing down.
    uu = np.linspace(-115, 115, 13)
    out.append(on("belt", [[(u, waist + dv, 0.008) for u in uu] for dv in (0.008, -0.022)], frame=0.006, per=2,
                  thick=0.005))
    buckle = [(waist + 0.04, 5.0), (waist + 0.01, 13.0), (waist - 0.05, 7.0), (waist - 0.1, 1.0)]
    out.append(on("buckle", [[(f * hw, v, 0.016) for f in (-1, -0.5, 0, 0.5, 1)] for v, hw in buckle],
                  frame=0.008, per=3, thick=0.007))
    # The tabard: a long plate hung from the belt over the front of the
    # legs, narrowing to a point above the knee.
    tab = [(waist - 0.04, 15.0), (0.0, 14.0), (-0.22, 12.0), (-0.3, 2.0)]
    out.append(on("tabard", [[(f * hw, v, 0.03 + 0.04 * max(0.0, -v)) for f in (-1, -0.5, 0, 0.5, 1)]
                             for v, hw in tab], frame=0.008, per=3, thick=0.006, joint="root"))
    # The back.
    us = np.linspace(80, 280, 51)
    ub = ((us + 180) % 360) - 180
    rad = np.stack([np.interp(ub, us_all, row) for row in R]) - 0.003
    a = np.abs(np.abs(ub) - 180)
    # A ridge down the spine, as the breast's keel.
    rad = rad + ridge(a, 0.0, 0.016, 22.0)[None, :] * plate.smoothstep(waist, 0.6 * H, vs)[:, None]
    hi = (neck - 0.02 - 0.04 * (a / 40.0) ** 2).clip(H - 0.06, None)
    hi = np.where(a > 60, H - 0.06 - 0.11 * plate.smoothstep(60, 100, a), hi)
    out += armour("backplate", ax, us, vs, rad, np.full(len(us), waist), hi, thick=0.007, rim=0.0075, sharp=24)
    # The fauld: lames from the waist down over the hips, each lapping
    # the one below and flaring past it.
    # All the way round: stopping at the sides, the hose showed over the
    # seat (2026-10-26). Two, close: four flaring read as a block at the
    # hips as wide as the chest (2026-10-28).
    for k in range(2):
        top = waist + 0.005 - k * 0.07
        bot = top - 0.08
        us = np.linspace(-180, 180, 73)[:-1]
        vv = np.linspace(bot, top, 8)
        hh = vv / H
        wd = np.interp(hh, keys[:, 0], keys[:, 1]) + 0.004 + 0.003 * k + 0.012 * (top - vv)
        fr = np.interp(hh, keys[:, 0], keys[:, 2]) + 0.005 + 0.003 * k + 0.012 * (top - vv)
        bk = np.interp(hh, keys[:, 0], keys[:, 3]) + 0.005 + 0.003 * k
        dr = plate.designed(us, vv, wd, fr, bk)
        br = plate.radii(body, ax, vv, us, keep=None, smooth_u=6, smooth_v=2, stiff=8)
        out += armour(f"fauld{k}", ax, us, vv, np.maximum(dr, br + 0.012), np.full(len(us), bot),
                      np.full(len(us), top), closed=True, thick=0.005, rim=0.006)
    EXTRAS["sun"] = shell(np.array([0.0, 0.8 * H, 0.024])).tolist()
    return out


def pauldrons(body, r):
    """Each pauldron after the owner's reference, plate by plate: a cop of
    five flat faces round the shoulder, its upper border swept up into one
    blade-like point over the outside of the shoulder at the height of the
    ear; below it two lames, each tucked under the one above, each of
    five faces, each running to a point at its front lower corner. Every
    face is flat and framed in trim; every fold is a bevelled edge."""
    out = []
    phis = (-110.0, -55.0, -14.0, 0.0, 14.0, 55.0, 110.0)
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        fr = facet.rest_frame(r, arm, el, SPREAD)
        # The cop: rows from its upper border down to its hem. Its border
        # rises to a blade only at the very outside -- a point, not a slope.
        top = {-110: (0.10, 0.072), -55: (0.125, 0.085), -14: (0.145, 0.11), 0: (0.255, 0.14),
               14: (0.145, 0.11), 55: (0.125, 0.085), 110: (0.10, 0.072)}
        # Flared well out from the body, as the reference's are: the
        # shoulders are the widest thing about it.
        # Out to the side, not down the arm: widest straight out from the
        # shoulder, close at the front and back, its hem high, so the
        # shoulders read broad and the arm shows under it (2026-10-28).
        def reach(p, side, rest):
            return side if abs(p) < 30 else (rest + (side - rest) * 0.45 if abs(p) < 70 else rest)

        rows = [
            [(p, *top[int(p)]) for p in phis],
            [(p, 0.07, reach(p, 0.15, 0.108)) for p in phis],
            [(p, 0.0, reach(p, 0.185, 0.118)) for p in phis],
            [(p, -0.04 - (0.03 if p == 55.0 else 0.0), reach(p, 0.2, 0.122)) for p in phis],
        ]
        out.append(facet.plate(f"pauldron.{s}", facet.round_limb(fr, rows), frame=0.009, joint=arm, thick=0.007))
        # The lames close round the arm, each under the one above, the
        # lowest the smallest.
        # Two, not three: the arm shows under them (the owner, 2026-10-27:
        # "make the pauldrons not drown the arms quite so much").
        # One, now: the arm shows under it.
        for k in range(1):
            h0 = -0.03
            rad = 0.1
            span = (-100.0, -50.0, 0.0, 50.0, 100.0)
            rows = [
                [(p, h0, rad - (0.012 if abs(p) > 90 else 0.0)) for p in span],
                [(p, h0 - 0.06 - (0.03 if p == 50.0 else 0.0), rad + 0.012 - (0.012 if abs(p) > 90 else 0.0))
                 for p in span],
            ]
            out.append(facet.plate(f"lame{k}.{s}", facet.round_limb(fr, rows), frame=0.007, per=4, joint=arm))
    return out


def arms(body, r):
    """Each arm after the reference: a band of plate round the upper arm
    under the pauldron's lames; a pointed cop at the back of the elbow; and
    a vambrace of eight flat faces flaring to a cuff over the wrist, a
    blade-like fin running up its outside to a point past the elbow, its end
    reaching forward over the back of the hand to a point."""
    out = []
    # Twelve faces round a limb: eight read boxy, as clunk (2026-10-28).
    ring = tuple(float(p) for p in range(0, 361, 30))
    for s in "lr":
        arm, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"
        lateral = facet.rest_frame(r, arm, el, SPREAD).m[0]
        up_arm = facet.bone_frame(r, arm, el, lateral)
        # Each plate round the middle of the limb's flesh, its own size, a
        # little clear of it (``facet.flesh_line``; 2026-10-28, "less
        # clunky").
        line, rad, hh = facet.flesh_line(body, r, arm, el, 0.6, 0.92, lateral)
        rows = [[(p, hh(t), rad.toward(t, p, 0.008) / 0.966) for p in ring] for t in (0.62, 0.9)]
        out.append(facet.plate(f"rerebrace.{s}", facet.round_limb(line, rows), frame=0.007, joint=arm,
                               edges=(True, True, False, False), inside=line.o + line.m[1] * hh(0.76)))
        fore = facet.bone_frame(r, el, wr, lateral)
        line, rad, hh = facet.flesh_line(body, r, el, wr, 0.06, 0.92, lateral)
        rf = [rad(t, 0.006) / 0.966 for t in (0.06, 0.5, 0.92)]
        shape = ((hh(0.06), rf[0]), (hh(0.5), rf[1]), (hh(0.92), rf[2]), (hh(0.92) - 0.03, rf[2] + 0.01))
        rows = [[(p, h, q) for p in ring] for h, q in shape[:-1]]
        # Its end reaches forward over the back of the hand to a point (the
        # owner, 2026-10-26: "bracers don't come far enough forward"); the
        # back of the hanging hand faces out, where phi is nothing.
        h_end, r_end = shape[-1]
        rows.append([(p, h_end - 0.065 * max(np.cos(np.radians(p)), 0.0) ** 3,
                      r_end - 0.01 * max(np.cos(np.radians(p)), 0.0) ** 3) for p in ring])
        out.append(facet.plate(f"vambrace.{s}", facet.round_limb(line, rows), frame=0.008, joint=el,
                               edges=(True, True, False, False), inside=line.o + line.m[1] * hh(0.5)))
        # The fin, up the outside and a little behind it, to a point past
        # the elbow.
        hs = np.array([hh(0.95), hh(0.62), hh(0.3), hh(0.0), hh(0.0) + 0.055])
        # On a face, not a corner: a twelve-sided ring's faces stand at
        # about 0.97 of its corners.
        base = np.interp(-hs, [-h for h, _ in shape[:3]], [q for _, q in shape[:3]]) * 0.97
        rise = np.array([0.006, 0.012, 0.017, 0.024, 0.003])
        rows = [[(-25.0, h, b0) for h, b0 in zip(hs, base)], [(-25.0, h, b0 + d) for h, b0, d in zip(hs, base, rise)]]
        out.append(facet.plate(f"fin.{s}", facet.round_limb(line, rows), frame=0.007, per=2, joint=el,
                               edges=(False, True, False, False), inside=line.o))
        # The cop: a diamond over the point of the elbow, raised to a ridge.
        cop = ((0.065, 3.0), (0.0, 42.0), (-0.075, 3.0))
        rc = facet.limb_radius(body, r, el, wr, 0.05, 0.012) / 0.92
        rows = [[(-90.0 + f * hw, h, rc + 0.012 * (1 - abs(f))) for f in (-1, -0.5, 0, 0.5, 1)] for h, hw in cop]
        out.append(facet.plate(f"cop.{s}", facet.round_limb(fore, rows), frame=0.008, joint=el, inside=fore.o))
    return out


def gauntlets(body, r):
    """Each hand in plate (the owner, 2026-10-28: "add gauntlets to the
    knight"), in the armour's own manner -- flat faces, gold frames: over
    the back of the hand a plate from the wrist to the knuckles; across the
    knuckles a raised guard; over the fingers two lames, each lapping the
    next. Each is laid on the hand's own surface, a little clear of it,
    across the back -- the side away from the grip's hollow."""
    out = []
    W = g.weights_of(body)
    pts = plate.body_points(body)
    for s in "lr":
        hd = f"hand.{s}"
        wrist, tip = r.o[hd], r.tip[hd]
        L = float(np.linalg.norm(tip - wrist))
        axis = (tip - wrist) / L
        hollow = r.marks[f"grip.{s}"] - wrist
        back = -(hollow - axis * (hollow @ axis))
        back /= np.linalg.norm(back)
        side = np.cross(axis, back)
        fr = facet.Frame(wrist, back, -axis, side)
        own = pts[W[:, J[hd]] > 0.4] - wrist
        along = own @ axis
        flat = own - np.outer(along, axis)
        ang = np.degrees(np.arctan2(flat @ side, flat @ back))
        dist = np.linalg.norm(flat, axis=1)

        def reach(t, phi, gap=0.006):
            m = (np.abs(along - t * L) < 0.012) & (np.abs((ang - phi + 180) % 360 - 180) < 14)
            return (float(dist[m].max()) if m.any() else 0.03) + gap

        def piece(name, ts, phis, lift=0.0, frame=0.005):
            rows = [[(p, -t * L, reach(t, p) + lift) for p in phis] for t in ts]
            return facet.plate(f"{name}.{s}", facet.round_limb(fr, rows), frame=frame, per=2, joint=hd,
                               thick=0.004, bevel=0.0015, inside=wrist + axis * 0.4 * L - back * 0.02)

        across = (-70.0, -35.0, 0.0, 35.0, 70.0)
        out.append(piece("gauntlet", (0.02, 0.3, 0.55), across))
        out.append(piece("knuckles", (0.52, 0.6, 0.68), across, lift=0.004))
        out.append(piece("finger0", (0.66, 0.8), across, lift=0.002, frame=0.004))
        out.append(piece("finger1", (0.78, 0.94), across, frame=0.004))
    return out


def legs(body, r):
    """The legs: no tassets (over the thighs they made the hips a block,
    2026-10-28); a cuisse round the thigh and a greave round the shin, each
    of twelve faces round the middle of the limb (``facet.flesh_line``); a
    pointed cop on the knee; plate boots."""
    # In the steel under the plate, not cloth: between the plates the cloth
    # showed brown (2026-10-28).
    # Close to the skin: the plate now hugs the limb, thickened inward, and
    # hose standing further out came through it in patches (2026-10-28).
    out = trousers(body, r, material="knit", hem=0.30, gap=0.002, name="hose")
    for s in "lr":
        th, sh, ft = f"thigh.{s}", f"shin.{s}", f"foot.{s}"
        lateral = np.array([np.sign(r.o[th][0]), 0.0, 0.0])
        # Round the middle of the thigh, not its bone (``facet.flesh_line``).
        line, rad, hh = facet.flesh_line(body, r, th, sh, 0.12, 0.86, lateral)
        ring = tuple(float(p) for p in range(0, 361, 30))
        rows = [[(p, hh(t), rad.toward(t, p, 0.008) / 0.966 * k) for p in ring]
                for t, k in ((0.12, 1.0), (0.45, 1.0), (0.86, 0.97))]
        out.append(facet.plate(f"cuisse.{s}", facet.round_limb(line, rows), frame=0.007, joint=th,
                               edges=(True, True, False, False), inside=line.o + line.m[1] * hh(0.5),
                               weld=True))
        shin = facet.bone_frame(r, sh, ft, lateral)
        knee = facet.limb_radius(body, r, sh, ft, 0.05, 0.022) / 0.92
        cop = ((0.07, 3.0), (0.0, 38.0), (-0.08, 3.0))
        rows = [[(90.0 + f * hw, h, knee + 0.02 * (1 - abs(f))) for f in (-1, -0.5, 0, 0.5, 1)] for h, hw in cop]
        out.append(facet.plate(f"poleyn.{s}", facet.round_limb(shin, rows), frame=0.007, joint=sh, inside=shin.o))
        line, rad, hh = facet.flesh_line(body, r, sh, ft, 0.1, 0.88, lateral)
        ring = tuple(float(p) for p in range(0, 361, 30))
        rows = [[(p, hh(t), rad.toward(t, p, 0.008) / 0.966) for p in ring] for t in (0.1, 0.35, 0.88)]
        out.append(facet.plate(f"greave.{s}", facet.round_limb(line, rows), frame=0.007, joint=sh,
                               edges=(True, True, False, False), inside=line.o + line.m[1] * hh(0.5)))
    out += boots(body, r, material="plate", cuff="trim", strap="trim", top=0.75, straps_at=(), cuffed=False,
                 gap=0.012)
    return out


def helm(body, r):
    """A closed helm, faceless, in the same plate as the rest: an egg of
    flat faces round the head, never inside it, the corners down its middle
    -- from the chin over the crown to the nape -- standing proud of their
    neighbours as a ridge, as the pauldrons' are (the owner, 2026-10-26: "it
    needs a ridge in the middle, similar to the ridges on the pauldrons");
    its rim framed in gold; a gorget of flat faces flaring from under it to
    the collar; a slit of light across its face (``build.veins``)."""
    e = r.eye
    W = g.weights_of(body)
    keep = (W[:, J["head"]] + W[:, J["neck"]]) > 0.4
    centre = np.array([0.0, e[1] + 0.005, e[2] - 0.075])
    ax = plate.Axis(centre, [0, 1, 0], [0, 0, 1], [1, 0, 0])
    us_all = np.linspace(-180, 180, 73)
    vs_all = np.linspace(-0.16, 0.17, 34)
    body_r = plate.radii(body, ax, vs_all, us_all, keep=keep, smooth_u=6, smooth_v=4, stiff=8, reach=0.25)
    # The egg's corners: every 30 degrees round, at heights from under the
    # jaw to the crown, where all but the ridge's close in.
    uk = np.arange(-180.0, 181.0, 30.0)
    vk = np.array([-0.15, -0.09, -0.03, 0.03, 0.08, 0.12, 0.148, 0.168])
    t = vk / 0.168
    profile = np.sqrt(np.clip(1.0 - np.clip(t, 0.0, 1.0) ** 2.4, 0.0, 1.0))
    profile = np.maximum(np.where(t < 0, 1.0 - 0.2 * (-t) ** 2, profile), 0.03)
    Rk = plate.designed(uk, vk, 0.118 * profile, 0.132 * profile, 0.126 * profile, power=2.2)
    for i, v in enumerate(vk):
        row = np.array([np.interp(v, vs_all, body_r[:, j]) for j in range(len(us_all))])
        for j, u in enumerate(uk):
            near = np.abs(((us_all - u + 180) % 360) - 180) < 20
            if v < 0.14:
                Rk[i, j] = max(Rk[i, j], row[near].max() + 0.02)
    # The ridge: the front corner from the chin up, the back one from the
    # nape, both carried over the crown.
    for j, u in enumerate(uk):
        if u == 0.0:
            Rk[:, j] += 0.03 * plate.smoothstep(-0.15, -0.1, vk)
        elif abs(u) == 180.0:
            Rk[:, j] += 0.026 * plate.smoothstep(-0.08, -0.02, vk)
    shell = facet.Shell(ax, uk, vk, Rk)
    cols = np.linspace(-180, 180, 49)
    rows = np.linspace(-0.15, 0.168, 18)
    grid = np.stack([np.broadcast_to(cols, (len(rows), len(cols))),
                     np.broadcast_to(rows[:, None], (len(rows), len(cols))), np.zeros((len(rows), len(cols)))], -1)
    out = [facet.plate("helm", grid, shape=shell, per=1, frame=0.009, edges=(True, False, False, False),
                       inside=centre, weld=True, joint="head", thick=0.006)]
    # The gorget: from under the helm's rim, flaring down onto the collar.
    neck = r.o["neck"]
    fr = facet.Frame([0.0, neck[1] - 0.07, neck[2]], [1, 0, 0], [0, 1, 0], [0, 0, 1])
    rn = facet.limb_radius(body, r, "neck", "head", 0.3, 0.022) / 0.96
    ring = tuple(float(p) for p in range(0, 361, 30))
    rows = [[(p, h, q) for p in ring] for h, q in ((0.12, rn), (0.05, rn + 0.018), (-0.005, rn + 0.06))]
    out.append(facet.plate("gorget", facet.round_limb(fr, rows), frame=0.007, edges=(True, True, False, False),
                           inside=fr.o + np.array([0.0, 0.06, 0.0]), weld=True, joint="neck", thick=0.005))
    EXTRAS["visor"] = (e + np.array([0.0, 0.0, 0.04])).tolist()
    return out


def cloak(body, r):
    # Behind only: the pauldron is the shoulder's, and nothing lies over it.
    return cape(body, r, side=LEFT, name="cloak", material="cloak", low=0.6, gap=0.075, faces=1600,
                back_only=True)


GARMENTS = {
    "legs": legs, "cuirass": cuirass, "pauldrons": pauldrons, "arms": arms, "gauntlets": gauntlets, "helm": helm,
    "cloak": cloak,
}
# Not the cloak, for now: cut from the inflated body it hung as a lumpy
# brown sack down the back (2026-10-26), and it comes back made properly.
WARDROBE = [n for n in GARMENTS if n != "cloak"]


def masks(body, r):
    body_masks(body, r, gloved_to=1.05, brow=0.0)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.0) | (w(fw, "head") > 0.5)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
