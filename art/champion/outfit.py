"""What the Champion wears.

A duellist, dressed to be watched: a high-collared doublet in deep blue,
cut close, its skirt split into four short panels that kick as he moves;
the right sleeve fitted tight to the wrist, because that is the arm the
gold coils up between moves and the weapon forms in (``kits/champion.md``),
and nothing should hide it; the left sleeve the same, under a leather
bracer; a short cape over the left shoulder in his team's colour; pale
breeches into tall riding boots; full gloves. Read from across the arena:
a dark, square-shouldered torso, a flash of colour on one shoulder, pale
legs, and the gold.
"""

import numpy as np

import character
import garment as g
from wardrobe import (LEFT, belt, body_masks, boots, cape, coat_tails, collar, covered, eyes,  # noqa: F401
                      hair, hair_cards, jerkin, sleeves, trousers, vambraces)


def doublet(body, r):
    def folds_extra(o):
        g.wrinkle(o, r.o["spine"] + np.array([0, 0.02, 0.0]), [0, 1, 0], 0.05, 0.003, 0.04, seed=41)

    out = jerkin(body, r, material="doublet", gap=0.010, low=0.06, collar=0.02, name="doublet",
                 faces=2200, close=0.035, sleeved=True, trim=0.02)
    return out


def arms(body, r):
    return sleeves(body, r, material="doublet", to=1.0, gap=0.005, close=0.008, name="sleeve", faces=800)


def bracer(body, r):
    return vambraces(body, r, material="leather", strap="leather_dark", reach=0.25,
                     straps_at=(0.45, 0.78), sides=LEFT)


def skirt(body, r):
    # Four panels round the hips to mid-thigh, open at the front.
    return coat_tails(body, r, spans=[(28, 98, 0.66), (102, 176, 0.62), (184, 258, 0.62), (262, 332, 0.66)],
                      material="doublet", flare=0.30, at=0.03, ripple=0.10)


def legs(body, r):
    return trousers(body, r, material="breeches", hem=0.20, gap=0.008, name="breeches")


def riding_boots(body, r):
    return boots(body, r, material="boot", cuff="leather", strap="leather_dark", top=0.02,
                 straps_at=(0.70,), cuffed=True, gap=0.009)


def waist(body, r):
    return belt(body, r, material="leather", sashed=False, at=0.02)


def shoulder(body, r):
    return cape(body, r, side=LEFT, low=0.30, gap=0.032)


def neck(body, r):
    return collar(body, r, material="doublet", height=0.075, gap=0.011)


def head_hair(body, r):
    # Short, swept back from a high forehead, close at the sides.
    scalp = hair(body, r, gap=0.004, front=0.05, nape=-0.07, side=-0.075, grooves=10, depth=0.003,
                 faces=2200, lift=0.004)
    return [scalp, hair_cards(scalp, r, style="back", length=0.055, width=0.022,
                              layers=(0.003, 0.007, 0.011), counts=(45, 60, 70), seed=11)]


GARMENTS = {
    "riding_boots": riding_boots, "legs": legs, "arms": arms, "doublet": doublet, "skirt": skirt,
    "waist": waist, "bracer": bracer, "neck": neck, "shoulder": shoulder, "hair": head_hair,
    "eyes": eyes,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    body_masks(body, r, gloved_to=1.6, brow=1.0)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.0)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
