"""What the Blood mage wears.

Her blood spreads like a sickness and she fights from inside it, with a
war scythe (``kits/blood-mage.md``). Dressed as a reaper and a priestess of
something nobody should worship: a long black coat buttoned to a high
collar, its skirt split into four panels that fall to the ankle and drag;
an oxblood leather bodice laced over it at the waist; long black gloves to
the elbow; tall boots; a sash in her team's colour; ash-pale hair to the
jaw, a pale face, dark lips. Read from across the arena: a tall black
shape, a red waist, a pale face -- and the scythe.
"""

import numpy as np

import character
import garment as g
from wardrobe import (belt, body_masks, boots, coat_tails, collar, covered, eyes, hair, hair_cards, jerkin,  # noqa: F401
                      sleeves, straps, trousers, vambraces)


def coat(body, r):
    return jerkin(body, r, material="coat", gap=0.012, low=0.05, collar=0.03, name="coat", faces=2400,
                  close=0.065, sleeved=True, trim=0.02)


def coat_sleeves(body, r):
    return sleeves(body, r, material="coat", to=0.9, gap=0.009, close=0.012, name="sleeve", faces=900)


def gloves(body, r):
    # Long gloves: a cuff turned back just below the elbow.
    return vambraces(body, r, material="glove", strap="leather", reach=0.15, straps_at=(0.2,))


def bodice(body, r):
    """Laced oxblood leather from under the bust to the hip, over the coat."""
    hip = r.o["thigh.l"][1]
    chest = r.o["chest"][1]

    def lacing(o):
        g.wrinkle(o, np.array([0.0, (hip + chest) / 2, 0.12]), [0, 1, 0], 0.12, 0.003, 0.018, seed=51)

    out = jerkin(body, r, material="leather", gap=0.022, low=0.0, collar=0.0, name="bodice", faces=1400,
                 close=0.05, sleeved=True, trim=0.015,
                 trims=[([0, chest + 0.02, 0], [0, 1, 0.15])])
    lacing(out[0])
    return out


def skirt(body, r):
    return coat_tails(body, r, spans=[(30, 100, 0.10), (104, 178, 0.06), (182, 256, 0.06), (260, 330, 0.10)],
                      material="coat", flare=0.30, at=0.0, ripple=0.22)


def legs(body, r):
    return trousers(body, r, material="trousers", hem=0.25, gap=0.009, name="trousers")


def feet(body, r):
    return boots(body, r, material="boot", cuff="boot", strap="leather", top=0.05, straps_at=(0.4, 0.7),
                 cuffed=False, gap=0.009)


def waist(body, r):
    return belt(body, r, material="leather_dark", sash="team", sashed=True, at=0.0)


def neck(body, r):
    return collar(body, r, material="coat", height=0.09, gap=0.012)


def head_hair(body, r):
    # A heavy bob to the jaw, parted to one side.
    scalp = hair(body, r, gap=0.004, front=0.04, nape=-0.12, side=0.01, part=0.03, grooves=10,
                 depth=0.003, faces=2800, close=0.025, lift=0.004, material="scalp")
    return [scalp, hair_cards(scalp, r, style="bob", part=0.03, jaw=-0.095, length=0.20, width=0.034,
                              layers=(0.004, 0.009, 0.014), counts=(60, 80, 95), seed=13)]


GARMENTS = {
    "feet": feet, "legs": legs, "coat_sleeves": coat_sleeves, "coat": coat, "skirt": skirt, "bodice": bodice,
    "gloves": gloves, "waist": waist, "neck": neck, "hair": head_hair, "eyes": eyes,
}
WARDROBE = list(GARMENTS)


def masks(body, r):
    # Gloved fingers; dark lips and a shadow round the eyes.
    body_masks(body, r, gloved_to=1.6, brow=1.0, kohl=0.5, lips=0.9)


def hide(r, c, fw):
    return covered(r, c, fw, boots_to=0.05)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, hide=hide, masks=masks)
