"""What the light being wears: plate, made of light.

A celestial knight (the owner, 2026-10-26: "a celestial knight. Not holy
... but light", from a reference "not far off, but probably too heavy on the
gold and a little too ornate -- a cool visual but not so dominant").
Matched to that reference piece by piece ("You're reading 'angular' as 'add
any angle'... match the reference beat for beat"), every plate faceted
(``facet``): broad flat faces, bevelled where they fold, a frame of gold
round each.

- Pauldrons: a cop of flat faces flared well out from the shoulder, its
  upper border rising to one blade-like point at the height of the ear;
  two lames under it, each tucked under the one above and pointed at its
  front lower corner.
- Chest: a breastplate of planes meeting at a keel; a kite down the
  sternum with the sun at its widest; over each breast three strips laid up
  and out toward the armpit, lapped as feathers; three chevrons down the
  belly; a belt with a kite of a buckle; a broad tabard to above the knee.
- Arms: a band round the upper arm, a pointed cop at the elbow, a vambrace
  of six faces flaring to a wide cuff, a fin up its outside past the elbow.
- Legs: two lapped tassets over each thigh, a cuisse, a pointed knee cop,
  a greave of eight faces, plate boots.
- The closed helm the owner liked: an egg with a comb and a keel, small
  swept fins, a slit of light for its eyes.

White plate, gold only in the frames; the plate gives a little light of
its own (``build.LUMINOUS``). A cloak hung from the back of the left
shoulder only, under the pauldron. Nothing holy: no halo.
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
        [0.00, 0.155, 0.118, 0.115],
        [0.36, 0.112, 0.1, 0.092],
        [0.55, 0.168, 0.152, 0.112],
        [0.74, 0.232, 0.18, 0.135],
        [0.92, 0.228, 0.15, 0.13],
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
            Rk[i, j] = max(Rk[i, j], row[near].max() + 0.016)
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
    out.append(on("belt", [[(u, waist + dv, 0.016) for u in uu] for dv in (0.012, -0.03)], frame=0.007, per=2,
                  thick=0.006))
    buckle = [(waist + 0.04, 5.0), (waist + 0.01, 13.0), (waist - 0.05, 7.0), (waist - 0.1, 1.0)]
    out.append(on("buckle", [[(f * hw, v, 0.026) for f in (-1, -0.5, 0, 0.5, 1)] for v, hw in buckle],
                  frame=0.008, per=3, thick=0.007))
    # The tabard: a long plate hung from the belt over the front of the
    # legs, narrowing to a point above the knee.
    tab = [(waist - 0.04, 24.0), (0.0, 22.0), (-0.22, 20.0), (-0.3, 3.0)]
    out.append(on("tabard", [[(f * hw, v, 0.045 + 0.05 * max(0.0, -v)) for f in (-1, -0.5, 0, 0.5, 1)]
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
    # Four, all the way round: stopping at the sides, the hose showed over
    # the seat (2026-10-26).
    for k in range(4):
        top = waist + 0.01 - k * 0.075
        bot = top - 0.085
        us = np.linspace(-180, 180, 73)[:-1]
        vv = np.linspace(bot, top, 8)
        hh = vv / H
        wd = np.interp(hh, keys[:, 0], keys[:, 1]) + 0.006 + 0.004 * k + 0.03 * (top - vv)
        fr = np.interp(hh, keys[:, 0], keys[:, 2]) + 0.008 + 0.004 * k + 0.03 * (top - vv)
        bk = np.interp(hh, keys[:, 0], keys[:, 3]) + 0.008 + 0.006 * k
        dr = plate.designed(us, vv, wd, fr, bk)
        br = plate.radii(body, ax, vv, us, keep=None, smooth_u=6, smooth_v=2, stiff=8)
        out += armour(f"fauld{k}", ax, us, vv, np.maximum(dr, br + 0.02), np.full(len(us), bot),
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
        rows = [
            [(p, *top[int(p)]) for p in phis],
            [(p, 0.075, 0.125 if abs(p) < 100 else 0.112) for p in phis],
            [(p, 0.0, 0.142 if abs(p) < 100 else 0.124) for p in phis],
            [(p, -0.06 - (0.035 if p == 55.0 else 0.0), 0.152 if abs(p) < 100 else 0.13) for p in phis],
        ]
        out.append(facet.plate(f"pauldron.{s}", facet.round_limb(fr, rows), frame=0.009, joint=arm, thick=0.007))
        # The lames close round the arm, each under the one above, the
        # lowest the smallest.
        # Two, not three: the arm shows under them (the owner, 2026-10-27:
        # "make the pauldrons not drown the arms quite so much").
        for k in range(2):
            h0 = -0.045 - 0.045 * k
            rad = 0.112 - 0.008 * k
            span = (-100.0, -50.0, 0.0, 50.0, 100.0)
            rows = [
                [(p, h0, rad - (0.012 if abs(p) > 90 else 0.0)) for p in span],
                [(p, h0 - 0.075 - (0.034 if p == 50.0 else 0.0), rad + 0.02 - (0.012 if abs(p) > 90 else 0.0))
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
    ring = tuple(float(p) for p in range(0, 361, 45))
    for s in "lr":
        arm, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"
        lateral = facet.rest_frame(r, arm, el, SPREAD).m[0]
        up_arm = facet.bone_frame(r, arm, el, lateral)
        rows = [[(p, h, rad) for p in ring] for h, rad in ((-0.2, 0.066), (-0.275, 0.062))]
        out.append(facet.plate(f"rerebrace.{s}", facet.round_limb(up_arm, rows), frame=0.007, joint=arm,
                               edges=(True, True, False, False), inside=up_arm.o + up_arm.m[1] * -0.24))
        fore = facet.bone_frame(r, el, wr, lateral)
        L = np.linalg.norm(r.o[wr] - r.o[el])
        shape = ((-0.03, 0.05), (-0.13, 0.048), (-L + 0.06, 0.052), (-L - 0.015, 0.064))
        rows = [[(p, h, rad) for p in ring] for h, rad in shape[:-1]]
        # Its end reaches forward over the back of the hand to a point (the
        # owner, 2026-10-26: "bracers don't come far enough forward"); the
        # back of the hanging hand faces out, where phi is nothing.
        h_end, r_end = shape[-1]
        rows.append([(p, h_end - 0.065 * max(np.cos(np.radians(p)), 0.0) ** 3,
                      r_end - 0.01 * max(np.cos(np.radians(p)), 0.0) ** 3) for p in ring])
        out.append(facet.plate(f"vambrace.{s}", facet.round_limb(fore, rows), frame=0.008, joint=el,
                               edges=(True, True, False, False), inside=fore.o + fore.m[1] * -L / 2))
        # The fin, up the outside and a little behind it.
        hs = np.array([-L + 0.02, -0.17, -0.08, 0.02, 0.075])
        # On a face, not a corner: an octagon's face stands at about 0.92
        # of its corners there.
        base = np.interp(-hs, [-h for h, _ in shape], [rad for _, rad in shape]) * 0.92
        rise = np.array([0.012, 0.022, 0.03, 0.04, 0.004])
        rows = [[(-25.0, h, b0) for h, b0 in zip(hs, base)], [(-25.0, h, b0 + d) for h, b0, d in zip(hs, base, rise)]]
        out.append(facet.plate(f"fin.{s}", facet.round_limb(fore, rows), frame=0.007, per=2, joint=el,
                               edges=(False, True, False, False), inside=fore.o))
        # The cop: a diamond over the point of the elbow, raised to a ridge.
        cop = ((0.065, 3.0), (0.0, 42.0), (-0.075, 3.0))
        rows = [[(-90.0 + f * hw, h, 0.056 + 0.018 * (1 - abs(f))) for f in (-1, -0.5, 0, 0.5, 1)] for h, hw in cop]
        out.append(facet.plate(f"cop.{s}", facet.round_limb(fore, rows), frame=0.008, joint=el, inside=fore.o))
    return out


def legs(body, r):
    """The legs after the reference: from the belt over the front and
    outside of each thigh two tassets, lapped, the lower pointed; a cuisse
    of flat faces round the front of the thigh; a pointed cop on the knee;
    a greave of eight faces; plate boots."""
    out = trousers(body, r, material="skirt", hem=0.30, gap=0.009, name="hose")
    for s in "lr":
        th, sh, ft = f"thigh.{s}", f"shin.{s}", f"foot.{s}"
        lateral = np.array([np.sign(r.o[th][0]), 0.0, 0.0])
        thigh = facet.bone_frame(r, th, sh, lateral)
        T = float(np.linalg.norm(r.o[sh] - r.o[th]))
        q = facet.limb_radius(body, r, th, sh, 0.2, 0.02) / 0.92
        cols = (-10.0, 35.0, 80.0, 120.0)
        for k, (h0, h1) in enumerate(((0.07, -0.11), (-0.08, -0.27))):
            rows = [[(p, h0, q - 0.008 * k) for p in cols],
                    [(p, h1 - (0.04 if p == 80.0 else 0.0), q + 0.022 - 0.008 * k) for p in cols]]
            out.append(facet.plate(f"tasset{k}.{s}", facet.round_limb(thigh, rows), frame=0.008, joint=th,
                                   inside=thigh.o))
        q = facet.limb_radius(body, r, th, sh, 0.55, 0.016) / 0.92
        # All the way round: open at the back, the hose under it showed.
        ring = tuple(float(p) for p in range(0, 361, 45))
        rows = [[(p, h, q) for p in ring] for h in (-0.3 * T, -0.86 * T)]
        out.append(facet.plate(f"cuisse.{s}", facet.round_limb(thigh, rows), frame=0.007, joint=th,
                               edges=(True, True, False, False), inside=thigh.o - thigh.m[1] * 0.6 * T,
                               weld=True))
        shin = facet.bone_frame(r, sh, ft, lateral)
        S = float(np.linalg.norm(r.o[ft] - r.o[sh]))
        knee = facet.limb_radius(body, r, sh, ft, 0.05, 0.022) / 0.92
        cop = ((0.07, 3.0), (0.0, 38.0), (-0.08, 3.0))
        rows = [[(90.0 + f * hw, h, knee + 0.02 * (1 - abs(f))) for f in (-1, -0.5, 0, 0.5, 1)] for h, hw in cop]
        out.append(facet.plate(f"poleyn.{s}", facet.round_limb(shin, rows), frame=0.007, joint=sh, inside=shin.o))
        q = facet.limb_radius(body, r, sh, ft, 0.35, 0.014) / 0.92
        ring = tuple(float(p) for p in range(0, 361, 45))
        rows = [[(p, h, rad) for p in ring] for h, rad in ((-0.1 * S, q), (-0.5 * S, q), (-0.88 * S, 0.88 * q))]
        out.append(facet.plate(f"greave.{s}", facet.round_limb(shin, rows), frame=0.007, joint=sh,
                               edges=(True, True, False, False), inside=shin.o - shin.m[1] * 0.5 * S))
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
    "legs": legs, "cuirass": cuirass, "pauldrons": pauldrons, "arms": arms, "helm": helm, "cloak": cloak,
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
