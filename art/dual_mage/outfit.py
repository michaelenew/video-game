"""What the Dual mage wears.

Two forces in one body, light in her right hand and dark in her left
(``dual-mage.md``; exploration 0017: "light is always her right hand,
dark always her left"). So she is dressed in halves, split down the
middle: everything on her right ivory and gold, everything on her left
obsidian and violet -- a fitted high-necked tunic, long sleeves flared at
the wrist, a skirt of four long panels that lift as she floats, soft boots;
a bracer on each forearm, gold on the right and black glass on the left,
the containment; a sash in her team's colour; her hair cut to the jaw and
split the same way, white on the right and black on the left. Read from
across the arena: one figure, two halves, and which hand is which.
"""

import character
from wardrobe import (RIGHT, LEFT, band, belt, body_masks, boots, coat_tails, collar, covered, eyes,  # noqa: F401
                      hair, jerkin, side_mask, sleeves, trousers)


def halves(pieces, r):
    for p in pieces:
        side_mask(p, r)
    return pieces


def tunic(body, r):
    return halves(jerkin(body, r, material="vessel", gap=0.009, low=0.06, collar=0.02, name="tunic",
                         faces=2200, close=0.035, sleeved=True, trim=0.02), r)


def arms(body, r):
    out = []
    for s in "lr":
        out += sleeves(body, r, material="vessel_r" if s == RIGHT else "vessel_l", to=1.02, gap=0.010,
                       close=0.014, name="sleeve", faces=900, sides=s)
    return out


def bracers(body, r):
    out = []
    for s in "lr":
        m = "gold" if s == RIGHT else "obsidian"
        out.append(band(body, r, f"forearm.{s}", f"hand.{s}", 0.62, 0.05, 0.016, f"bracer.{s}", m))
        out.append(band(body, r, f"forearm.{s}", f"hand.{s}", 0.80, 0.02, 0.018, f"bracer_rim.{s}", m))
    return out


def skirt(body, r):
    return halves(coat_tails(body, r, spans=[(18, 92, 0.14), (98, 176, 0.10), (184, 262, 0.10),
                                             (268, 342, 0.14)],
                             material="vessel", flare=0.36, at=0.02, ripple=0.15), r)


def legs(body, r):
    return halves(trousers(body, r, material="vessel", hem=0.30, gap=0.007, name="trousers"), r)


def feet(body, r):
    out = []
    for p in boots(body, r, material="boot", cuff="boot", strap="boot", top=0.25, straps_at=(),
                   cuffed=False, gap=0.007):
        out.append(p)
    return halves(out, r)


def waist(body, r):
    return belt(body, r, material="gold", sash="team", sashed=True, at=0.03)


def neck(body, r):
    return halves(collar(body, r, material="vessel", height=0.085, gap=0.009), r)


def head_hair(body, r):
    # Cut to the jaw, a straight fringe: and split, white and black.
    return halves([hair(body, r, gap=0.009, front=0.025, nape=-0.10, side=0.02, grooves=13,
                        depth=0.007, faces=2800, close=0.02)], r)


GARMENTS = {
    "feet": feet, "legs": legs, "arms": arms, "tunic": tunic, "skirt": skirt, "bracers": bracers,
    "waist": waist, "neck": neck, "hair": head_hair, "eyes": eyes,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    body_masks(body, r, gloved_to=None, brow=1.0, kohl=0.35, lips=0.4)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.25)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
