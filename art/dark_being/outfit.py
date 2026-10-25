"""What the dark being wears: iron, and little else.

A hulking demon (the owner, 2026-10-25): bare to the waist so every mass of
the torso and arms shows; iron bands riveted round each wrist -- a shackle's
pair -- and round each upper arm; horns curling out and up from the temples
and forward at the tips, a bull's; eyes that burn violet; an iron belt and a
ragged loincloth of black hide, front and back.
"""

import numpy as np

import character
import forge
import garment as g
import stage
from wardrobe import band, belt, coat_tails, eyes, mat, rigid


def glowing(ob):
    grp = ob.vertex_groups.new(name="edge")
    grp.add(list(range(len(ob.data.vertices))), 1.0, "REPLACE")
    return ob


def irons(body, r):
    out = []
    for s in "lr":
        fa, hd, ua, el = f"forearm.{s}", f"hand.{s}", f"arm.{s}", f"forearm.{s}"
        # The shackle: two broad bands at the wrist, a narrow one between.
        out.append(band(body, r, fa, hd, 0.80, 0.055, 0.016, f"shackle.{s}", "iron"))
        out.append(band(body, r, fa, hd, 0.66, 0.03, 0.018, f"shackle_rim.{s}", "iron"))
        # The armband, high on the biceps.
        out.append(band(body, r, ua, el, 0.42, 0.07, 0.016, f"armband.{s}", "iron"))
        out.append(band(body, r, ua, el, 0.30, 0.025, 0.02, f"armband_rim.{s}", "iron"))
    return out


def horns(body, r):
    """A bull's horns: out from the temples, sweeping up and back, the tips
    turning forward -- each a loft of shrinking rings along that curve,
    ridged as horn is."""
    e = r.eye
    out = []
    for s, side in (("l", -1.0), ("r", 1.0)):
        # The temple: above and behind the eye, out at the side of the skull.
        root = e + np.array([side * 0.062, 0.055, -0.035])
        steps = 22
        pts = []
        for i in range(steps + 1):
            t = i / steps
            pts.append(root + np.array([
                side * (0.16 * t + 0.05 * np.sin(t * np.pi)),
                0.05 * t + 0.20 * t * t,
                -0.07 * np.sin(t * np.pi) + 0.06 * t ** 3,
            ]))
        pts = np.array(pts)
        sections = []
        for i, p in enumerate(pts):
            t = i / steps
            d = pts[min(i + 1, steps)] - pts[max(i - 1, 0)]
            d /= np.linalg.norm(d)
            a = np.cross(d, [0.0, 0.0, 1.0])
            a /= np.linalg.norm(a)
            b = np.cross(d, a)
            rad = 0.034 * (1 - t) ** 1.1 + 0.0025
            ridge = 1 + 0.08 * np.sin(t * 40.0)
            sections.append(forge.ring(p, d, a, b, rad * ridge, rad * ridge * 0.85, sides=12))
        v, f = forge.loft(sections, cap_start=True, cap_end=True)
        ob = stage.mesh_object(f"horn.{s}", g.g2b(v), f)
        g.material(ob, mat("horn"))
        out.append(rigid(ob, "head"))
    return out


def burning_eyes(body, r):
    return [glowing(ob) for ob in eyes(body, r, iris="eye")]


def waist(body, r):
    return belt(body, r, material="iron", sashed=False, at=0.02)


def loincloth(body, r):
    return coat_tails(body, r, spans=[(-26, 26, 0.48), (154, 206, 0.44)], material="hide", flare=0.18,
                      at=0.01, ripple=0.3)


GARMENTS = {"irons": irons, "horns": horns, "eyes": burning_eyes, "waist": waist, "loincloth": loincloth}
WARDROBE = list(GARMENTS)


def dress(body, r, only=None):
    return character.dress(body, r, WARDROBE, only, GARMENTS, skin_material="skin")
