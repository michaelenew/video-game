"""What the light being wears: plate, made of light.

A celestial knight (the owner, 2026-10-26: "a celestial knight. Not holy
... but light", from a reference "not far off, but probably too heavy on the
gold and a little too ornate -- a cool visual but not so dominant"). Plate
as the armourers of Greenwich made it, cut by ``hard``: every plate smooth,
with a rolled rim at each edge; a breastplate with a keel down its middle
and three lames of fauld below it; pauldrons of four lapped lames; plate
down the arms with a cop at the elbow; gauntlets; plate down the legs with
cops at the knees. A closed helm, a comb along its crown, small swept fins
at its temples, and a slit of light for its eyes. Gold only in the rims,
thin, and the rims give off a little light; a sun at the breast, small. A
half-cloak over the left shoulder. Nothing holy: no halo.
"""

import numpy as np

import character
import forge
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


def armour(name, ax, us, vs, radius, lo=None, hi=None, closed=False, joint=None, thick=0.006, rim=0.0065):
    ob, borders = plate.surface(name, ax, us, vs, radius, lo, hi, closed=closed)
    return lit_rims(plate.finish(ob, borders, "plate", thick, rim, rim_material="trim", joint=joint,
                                 rim_closed=closed))


def trunk_axis(r):
    hip = r.o["thigh.l"][1]
    return plate.Axis([0.0, hip, r.o["chest"][2]], [0, 1, 0], [0, 0, 1], [1, 0, 0])


def cuirass(body, r):
    """Breastplate and backplate, each a smooth shell round the trunk with
    a keel down the breast; the fauld's three lames lapping down over the
    hips; a sun at the breast (``build.veins``)."""
    ax = trunk_axis(r)
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1] - hip
    arm = r.o["arm.l"][1] - hip
    waist = 0.07
    us_all = np.linspace(-180, 180, 73)
    vs = np.linspace(-0.25, neck + 0.02, 40)
    W = g.weights_of(body)
    trunk = sum(W[:, J[n]] for n in ("root", "spine", "chest", "clavicle.l", "clavicle.r")) > 0.5
    R = plate.radii(body, ax, vs, us_all, keep=trunk, smooth_u=6, smooth_v=4, stiff=8)
    out = []
    # The breast: from the waist to the collar, the sides scooped down to
    # the armpits, the neck cut in a shallow curve; a keel down its middle.
    us = np.linspace(-98, 98, 45)
    rad = np.stack([np.interp(us, us_all, row) for row in R]) + 0.03
    keel = 0.022 * np.exp(-(us / 11.0) ** 2)[None, :] * plate.smoothstep(waist, arm - 0.12, vs)[:, None]
    swell = 0.03 * np.exp(-((vs - (arm - 0.14)) / 0.14) ** 2)[:, None] * np.cos(np.radians(us))[None, :] ** 2
    # In at the waist, as a breastplate is shaped: the body's own waist is
    # soft, and plate laid over it reads as a barrel.
    pinch = -0.022 * np.exp(-((vs - (waist + 0.04)) / 0.08) ** 2)[:, None] * np.abs(np.sin(np.radians(us)))[None, :]
    rad = rad + keel + swell + pinch
    a = np.abs(us)
    hi = (neck - 0.035 - 0.05 * (a / 40.0) ** 2).clip(arm - 0.06, None)
    hi = np.where(a > 58, arm - 0.06 - 0.12 * plate.smoothstep(58, 98, a), hi)
    lo = np.full(len(us), waist) + 0.01 * (a / 98.0) ** 2
    out += armour("breastplate", ax, us, vs, rad, lo, hi)
    # The back: the same, round the other way, higher at the neck.
    us = np.linspace(82, 278, 45)
    ub = ((us + 180) % 360) - 180
    rad = np.stack([np.interp(ub, us_all, row) for row in R]) + 0.028
    a = np.abs(np.abs(ub) - 180)
    hi = (neck - 0.015 - 0.04 * (a / 40.0) ** 2).clip(arm - 0.05, None)
    hi = np.where(a > 60, arm - 0.05 - 0.1 * plate.smoothstep(60, 98, a), hi)
    out += armour("backplate", ax, us, vs, rad, np.full(len(us), waist + 0.01), hi)
    # The fauld: three lames below the waist, each flaring past the one above.
    for k in range(3):
        us = np.linspace(-115, 115, 47)
        top, bot = waist + 0.015 - k * 0.05, waist - 0.045 - k * 0.05
        vv = np.linspace(bot - 0.02, top + 0.02, 8)
        rk = plate.radii(body, ax, vv, us, keep=None, smooth_u=6, smooth_v=2, stiff=8)
        flare = 0.032 + 0.012 * k + 0.02 * (top - vv)[:, None]
        out += armour(f"fauld{k}", ax, us, vv, rk + flare, np.full(len(us), bot), np.full(len(us), top),
                      thick=0.005, rim=0.0055)
    EXTRAS["sun"] = (ax.point(0.0, arm - 0.1, np.interp(0.0, us_all, R[np.searchsorted(vs, arm - 0.1)]) + 0.045)
                     ).tolist()
    return out


def pauldrons(body, r):
    """Over each shoulder an oversized dome of plate, open toward the neck,
    and three lames below it down the arm, each lapping the next."""
    out = []
    W = g.weights_of(body)
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        sh = r.o[arm]
        down = r.dir(arm, el)
        ax = plate.Axis(sh + np.array([0.0, 0.02, 0.0]), down, [0, 1, 0], [0, 0, 1])
        near = (W[:, J[arm]] + W[:, J[f"clavicle.{s}"]]) > 0.3
        us = np.linspace(-150, 150, 51)
        vs = np.linspace(-0.09, 0.16, 26)
        R = plate.radii(body, ax, vs, us, keep=near, smooth_u=5, smooth_v=3, stiff=8)
        dome = 0.05 + 0.02 * np.cos(np.radians(us))[None, :] ** 2
        rad = R + dome
        a = np.abs(us)
        lo = -0.085 + 0.05 * plate.smoothstep(90, 150, a)
        hi = np.full(len(us), 0.085) - 0.02 * (a / 150.0) ** 2
        out += armour(f"pauldron.{s}", ax, us, vs, rad, lo, hi, joint=arm, thick=0.007, rim=0.0075)
        for k in range(3):
            top, bot = 0.075 + k * 0.045, 0.125 + k * 0.045
            vv = np.linspace(top - 0.01, bot + 0.01, 8)
            rk = plate.radii(body, ax, vv, us, keep=near, smooth_u=5, smooth_v=2, stiff=8)
            out += armour(f"lame{k}.{s}", ax, us, vv, rk + 0.04 - 0.006 * k, np.full(len(us), top),
                          np.full(len(us), bot), joint=arm, thick=0.005, rim=0.0055)
    return out


def limb_tube(body, r, a, b, name, t0, t1, gap, joint, u0=-180, u1=180, flare=0.0):
    ax = plate.Axis(r.o[a], r.o[b] - r.o[a], [0, 0, 1], [1, 0, 0])
    length = np.linalg.norm(r.o[b] - r.o[a])
    closed = (u1 - u0) >= 359
    us = np.linspace(u0, u1, 49, endpoint=not closed)
    vs = np.linspace(t0 * length - 0.02, t1 * length + 0.02, 16)
    W = g.weights_of(body)
    keep = W[:, J[a]] > 0.3
    R = plate.radii(body, ax, vs, us, keep=keep, smooth_u=6, smooth_v=3, stiff=8)
    rad = R + gap + flare * plate.smoothstep(t0 * length, t1 * length, vs)[:, None]
    return armour(name, ax, us, vs, rad, np.full(len(us), t0 * length), np.full(len(us), t1 * length),
                  closed=closed, joint=joint)


def arms(body, r):
    out = []
    for s in "lr":
        arm, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"
        out += limb_tube(body, r, arm, el, f"rerebrace.{s}", 0.42, 0.86, 0.016, arm)
        out += limb_tube(body, r, el, wr, f"vambrace.{s}", 0.14, 0.9, 0.014, el, flare=0.006)
        # The cop at the elbow: a dome on its outside, rimmed.
        elbow = r.o[el]
        outward = np.array([np.sign(elbow[0]), 0.0, -0.5])
        outward /= np.linalg.norm(outward)
        at = elbow + outward * 0.045
        out += hard.rivets(f"couter.{s}", [at], [outward], 0.042, "plate", joint=el)
        side = np.cross(outward, [0, 1, 0])
        side /= np.linalg.norm(side)
        ring = [at + (np.cos(q) * side + np.sin(q) * np.array([0, 1, 0])) * 0.042
                for q in np.linspace(0, 2 * np.pi, 40, endpoint=False)]
        out.append(glowing(rigid(hard.tube(f"couter_rim.{s}", np.array(ring), 0.005, "trim"), el), 0.35))
        # The gauntlet's cuff.
        out += limb_tube(body, r, el, wr, f"cuff.{s}", 0.9, 1.05, 0.02, el, flare=0.012)
    return out


def legs(body, r):
    out = trousers(body, r, material="skirt", hem=0.30, gap=0.009, name="hose")
    for s in "lr":
        th, sh, ft = f"thigh.{s}", f"shin.{s}", f"foot.{s}"
        out += limb_tube(body, r, th, sh, f"cuisse.{s}", 0.18, 0.88, 0.02, th, u0=-130, u1=130)
        out += limb_tube(body, r, sh, ft, f"greave.{s}", 0.08, 0.92, 0.016, sh)
        knee = r.o[sh] + np.array([0.0, 0.01, 0.07])
        out += hard.rivets(f"poleyn.{s}", [knee], [[0.0, 0.1, 1.0]], 0.055, "plate", joint=sh)
    out += boots(body, r, material="plate", cuff="trim", strap="trim", top=0.75, straps_at=(), cuffed=False,
                 gap=0.012)
    return out


def helm(body, r):
    """A closed helm: a smooth shell round the head and down to the neck,
    capped, a comb along its crown, small fins swept back from its temples,
    a gorget below, and a slit of light across its face (``build.veins``)."""
    e = r.eye
    head = r.o["head"]
    W = g.weights_of(body)
    keep = (W[:, J["head"]] + W[:, J["neck"]]) > 0.4
    ax = plate.Axis([0.0, head[1], head[2]], [0, 1, 0], [0, 0, 1], [1, 0, 0])
    top = plate.body_points(body)[keep][:, 1].max() - head[1]
    us = np.linspace(-180, 180, 61)[:-1]
    vs = np.linspace(-0.03, top, 30)
    R = plate.radii(body, ax, vs, us, keep=keep, smooth_u=6, smooth_v=4, stiff=8)
    rad = R + 0.022
    # Closed over the crown: the last rows drawn in to a point.
    cap = plate.smoothstep(top - 0.06, top + 0.01, vs)[:, None]
    rad = rad * (1 - 0.9 * cap)
    comb = (0.014 * np.exp(-(np.sin(np.radians(us)) / 0.12) ** 2)[None, :]
            * plate.smoothstep(top - 0.12, top - 0.03, vs)[:, None])
    rad = rad + comb
    out = armour("helm", ax, us, vs, rad, None, None, closed=True, thick=0.006, rim=0.007)
    # The gorget round the neck.
    neck_ax = plate.Axis([0.0, r.o["neck"][1] - 0.06, r.o["neck"][2]], [0, 1, 0], [0, 0, 1], [1, 0, 0])
    vs = np.linspace(0.0, 0.1, 8)
    keep_n = (W[:, J["neck"]] + W[:, J["chest"]]) > 0.4
    Rn = plate.radii(body, neck_ax, vs, us, keep=keep_n, smooth_u=6, smooth_v=2, stiff=8)
    out += armour("gorget", neck_ax, us, vs, Rn + 0.03 + 0.03 * (0.1 - vs)[:, None], closed=True,
                  thick=0.005, rim=0.006)
    for s, sign in (("l", -1.0), ("r", 1.0)):
        root = e + np.array([sign * 0.08, 0.035, -0.04])
        sections = []
        steps = 10
        for i in range(steps + 1):
            t = i / steps
            c = root + np.array([sign * 0.03 * t, 0.06 * t, -0.11 * t])
            tall = 0.03 * (1 - t) + 0.003
            sections.append(forge.ring(c, np.array([0.0, 0.5, -1.0]) / np.sqrt(1.25), np.array([0.0, 1.0, 0.5]),
                                       np.array([1.0, 0.0, 0.0]), tall, 0.003, sides=6))
        v, f = forge.loft(sections)
        fin = stage.mesh_object(f"fin.{s}", g.g2b(v), f, smooth=False)
        g.material(fin, mat("trim"))
        out.append(glowing(rigid(fin, "head"), 0.5))
    EXTRAS["visor"] = (e + np.array([0.0, 0.0, 0.02])).tolist()
    return out


def cloak(body, r):
    return cape(body, r, side=LEFT, name="cloak", material="cloak", low=0.6, gap=0.075, faces=1600)


GARMENTS = {
    "legs": legs, "cuirass": cuirass, "pauldrons": pauldrons, "arms": arms, "helm": helm, "cloak": cloak,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    body_masks(body, r, gloved_to=1.05, brow=0.0)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.0) | (w(fw, "head") > 0.5)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
