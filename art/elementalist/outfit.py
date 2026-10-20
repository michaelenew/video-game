"""What the Elementalist wears.

A terrain author (``kits/elementalist.md``): she drops to one knee and
drives both hands into the floor, raises a palm and the air gathers. Dressed
for kneeling on rock in all weathers: a slate tunic belted at the waist over
a long split robe in rust, the panels falling to the shin so they spread
round her on the ground; the sleeves cut at the elbow and the forearms
bound in linen, the hands bare -- they touch the earth; three river stones
on her belt; a stole round the neck and down the front in her team's
colour, which the wind takes; trousers and low boots. Her hair is pulled
back into a knot, out of the way. Read from across the arena: a tall,
narrow figure in earth colours, a long robe, bare hands.
"""

import numpy as np

import character
from wardrobe import (body_masks, boots, coat_tails, covered, eyes, hair, jerkin, piece, sleeves,  # noqa: F401
                      stone, straps, trousers, w, belt)


def tunic(body, r):
    return jerkin(body, r, material="tunic", gap=0.011, low=0.10, collar=0.05, name="tunic", faces=2200,
                  close=0.035, sleeved=True, trim=0.02)


def arms(body, r):
    return sleeves(body, r, material="tunic", to=0.18, gap=0.008, close=0.012, name="sleeve", faces=700)


def wraps(body, r):
    out = []
    for s in "lr":
        el, wr = f"forearm.{s}", f"hand.{s}"
        for i, at in enumerate((0.30, 0.42, 0.54, 0.66, 0.78, 0.90)):
            out.append(straps(body, r, el, wr, at, 0.030, 0.006 + 0.0012 * (i % 2), f"wrap{i}.{s}", "linen",
                              tilt=0.25 if i % 2 else -0.25))
    return out


def robe(body, r):
    return coat_tails(body, r, spans=[(40, 100, 0.22), (104, 178, 0.16), (182, 256, 0.16), (260, 320, 0.22)],
                      material="robe", flare=0.38, at=0.02, ripple=0.2)


def legs(body, r):
    return trousers(body, r, material="trousers", hem=0.45, gap=0.010, name="trousers")


def feet(body, r):
    return boots(body, r, material="boot", cuff="boot", strap="leather", top=0.55, straps_at=(0.80,),
                 cuffed=False, gap=0.008)


def waist(body, r):
    out = belt(body, r, material="leather", sashed=False, at=0.06)
    hip = r.o["thigh.l"][1] + 0.06
    for i, (x, size) in enumerate(((-0.09, 0.022), (-0.045, 0.017), (0.075, 0.019))):
        out.append(stone(f"charm{i}", [x, hip + 0.012, 0.135 - abs(x) * 0.35], [size, size * 1.2, size * 0.7],
                         "stone", "root", seed=i))
    return out


def team_stole(body, r):
    """A long scarf: round the back of the neck, both ends down the front to
    the waist. The shell of the upper trunk, kept in two strips either side
    of the breastbone and a band round the nape."""
    neck = r.o["neck"]
    hip = r.o["thigh.l"][1]

    def keep(c, fw):
        k = w(fw, "chest", "neck", "spine") > 0.4
        x = np.abs(c[:, 0])
        front = (c[:, 2] > 0.0) & (x > 0.035) & (x < 0.095) & (c[:, 1] > hip + 0.12)
        nape = (c[:, 1] > neck[1] - 0.05) & (c[:, 1] < neck[1] + 0.05)
        return k & (front | nape)

    return [piece(body, "team_stole", 0.022, keep, smooth=5, close=0.03, material="team", thick=0.004,
                  faces=1200, trim=0.01, trims=[(neck + np.array([0, 0.06, 0]), [0, 1, 0])])]


def head_hair(body, r):
    e = r.eye
    out = [hair(body, r, gap=0.007, front=0.045, nape=-0.09, side=-0.045, grooves=22, depth=0.003,
                faces=2400)]
    # Gathered into a knot at the back of the head.
    out.append(stone("knot", e + np.array([0.0, 0.02, -0.15]), [0.045, 0.04, 0.035], "hair", "head",
                     seed=7, rough=0.6))
    return out


GARMENTS = {
    "feet": feet, "legs": legs, "arms": arms, "wraps": wraps, "tunic": tunic, "robe": robe,
    "waist": waist, "team_stole": team_stole, "hair": head_hair, "eyes": eyes,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    # Bare hands, brows, a little colour on the lips.
    body_masks(body, r, gloved_to=None, brow=1.0, kohl=0.25, lips=0.35)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.5, sleeves=False)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
