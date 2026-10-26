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

import bpy
import numpy as np

import character
import forge
import garment as g
import hard
import stage
from wardrobe import (EXTRAS, LEFT, body_masks, boots, cape, covered, mat, rigid, trousers, w)


def glowing(ob, weight=1.0):
    grp = ob.vertex_groups.get("edge") or ob.vertex_groups.new(name="edge")
    grp.add(list(range(len(ob.data.vertices))), weight, "REPLACE")
    return ob


def rimmed(pieces, glow=0.35):
    """The rolled rims of a plate (every piece after the first that
    ``hard.plate`` made) in gold, and faintly lit."""
    plate, *rims = pieces
    for rim in rims:
        g.material(rim, mat("trim"))
        glowing(rim, glow)
    return [plate] + rims


def cuirass(body, r):
    hip = r.o["thigh.l"][1]
    neck = r.o["head"][1]
    chest = r.o["chest"]

    def keep(c, fw):
        return w(fw, "root", "spine", "chest", "clavicle.l", "clavicle.r", "arm.l", "arm.r") > 0.45

    trims = [([0, hip + 0.05, 0], [0, -1, 0]), ([0, neck - 0.045, 0], [0, 1, 0.3])]
    for s, sign in (("l", -1), ("r", 1)):
        sh = r.o[f"arm.{s}"]
        trims.append(([sh[0] - sign * 0.045, sh[1] - 0.04, 0.0], [sign * 0.85, 0.5, 0.0]))
    pieces = hard.plate(body, "cuirass", 0.026, keep, "plate", trims=trims, thick=0.008, rim=0.008, smooth=14,
                        close=0.09, faces=2600)
    hard.ridge(pieces[0], 0.0, 0.05, 0.012)
    out = rimmed(pieces)
    # The fauld: three lames lapping down over the hips.
    for k in range(3):
        top = hip + 0.06 - k * 0.045
        out += rimmed(hard.plate(body, f"fauld{k}", 0.030 + k * 0.005,
                                 lambda c, fw: w(fw, "root", "spine", "thigh.l", "thigh.r") > 0.35, "plate",
                                 trims=[([0, top, 0], [0, 1, 0]), ([0, top - 0.055, 0], [0, -1, 0])],
                                 thick=0.006, rim=0.006, smooth=8, close=0.05, faces=900))
    # The sun at the breast.
    EXTRAS["sun"] = (chest + np.array([0.0, 0.10, 0.16])).tolist()
    return out


def pauldrons(body, r):
    out = []
    for s in "lr":
        arm, el = f"arm.{s}", f"forearm.{s}"
        top = r.o[arm][1]
        down = r.dir(arm, el)

        def cap(c, fw, s=s, top=top):
            return (w(fw, f"arm.{s}", f"clavicle.{s}", "chest") > 0.4) & (c[:, 1] > top - 0.03) \
                & (np.abs(c[:, 0]) > abs(r.o[arm][0]) - 0.09)

        out += rimmed(hard.plate(body, f"pauldron.{s}", 0.042, cap, "plate", thick=0.008, rim=0.008, smooth=12,
                                 close=0.04, faces=1000))
        for k, (at, gap) in enumerate(((0.10, 0.036), (0.19, 0.030), (0.28, 0.024))):
            mid = r.along(arm, el, at)
            out += rimmed(hard.plate(body, f"lame{k}.{s}", gap, lambda c, fw, arm=arm: w(fw, arm) > 0.4, "plate",
                                     trims=[(mid + down * 0.042, down)], keeps=[(mid - down * 0.03, down)],
                                     thick=0.006, rim=0.006, smooth=6, close=0.02, faces=500))
    return out


def arms(body, r):
    out = []
    for s in "lr":
        arm, el, wr = f"arm.{s}", f"forearm.{s}", f"hand.{s}"
        # The rerebrace: a tube of plate down the upper arm.
        a0, a1 = r.along(arm, el, 0.36), r.along(arm, el, 0.86)
        d = r.dir(arm, el)
        out += rimmed(hard.plate(body, f"rerebrace.{s}", 0.016, lambda c, fw, arm=arm: w(fw, arm) > 0.4, "plate",
                                 trims=[(a1, d)], keeps=[(a0, d)], thick=0.006, rim=0.005, smooth=5,
                                 close=0.02, faces=700))
        # The vambrace.
        f0, f1 = r.along(el, wr, 0.18), r.along(el, wr, 0.92)
        fd = r.dir(el, wr)
        out += rimmed(hard.plate(body, f"vambrace.{s}", 0.014, lambda c, fw, el=el, wr=wr: w(fw, el, wr) > 0.3,
                                 "plate", trims=[(f1, fd)], keeps=[(f0, fd)], thick=0.006, rim=0.005, smooth=5,
                                 close=0.02, faces=700))
        # The cop at the elbow: a domed disc on its outside, rimmed.
        elbow = r.o[el]
        outward = np.array([np.sign(elbow[0]), 0.0, -0.4])
        outward /= np.linalg.norm(outward)
        at = elbow + outward * 0.05
        cop = hard.rivets(f"couter.{s}", [at], [outward], 0.045, "plate", joint=el)
        ring = [at + (np.cos(q) * np.cross(outward, [0, 1, 0]) + np.sin(q) * np.array([0, 1, 0])) * 0.045
                for q in np.linspace(0, 2 * np.pi, 40, endpoint=False)]
        out += cop + [glowing(rigid(hard.tube(f"couter_rim.{s}", np.array(ring), 0.005, "trim"), el), 0.35)]
        # The gauntlet's cuff, and a plate over the back of the hand.
        out += hard.band(body, r, el, wr, 0.97, 0.05, 0.016, f"cuff.{s}", "plate", thick=0.007, rims=True)
    return out


def legs(body, r):
    out = trousers(body, r, material="skirt", hem=0.30, gap=0.009, name="hose")
    for s in "lr":
        th, sh, ft = f"thigh.{s}", f"shin.{s}", f"foot.{s}"
        d = r.dir(th, sh)
        out += rimmed(hard.plate(body, f"cuisse.{s}", 0.02, lambda c, fw, th=th: w(fw, th) > 0.4, "plate",
                                 trims=[(r.along(th, sh, 0.88), d)], keeps=[(r.along(th, sh, 0.25), d)],
                                 thick=0.006, rim=0.006, smooth=6, close=0.03, faces=900))
        knee = r.o[sh] + np.array([0.0, 0.01, 0.07])
        out += hard.rivets(f"poleyn.{s}", [knee], [[0.0, 0.1, 1.0]], 0.055, "plate", joint=sh)
    out += boots(body, r, material="plate", cuff="trim", strap="trim", top=0.0, straps_at=(), cuffed=True,
                 gap=0.012)
    return out


def helm(body, r):
    """A closed helm: the head and neck in one smooth shell with a rolled
    rim at the neck, a comb along its crown, small fins swept back from its
    temples, and a slit of light across its face (``build.veins``)."""
    e = r.eye
    out = rimmed(hard.plate(body, "helm", 0.024, lambda c, fw: w(fw, "head", "neck") > 0.45, "plate",
                            thick=0.006, rim=0.007, smooth=14, close=0.05, faces=1800))
    hard.ridge(out[0], 0.0, 0.016, 0.012, front_only=False)
    for s, sign in (("l", -1.0), ("r", 1.0)):
        root = e + np.array([sign * 0.075, 0.035, -0.04])
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
