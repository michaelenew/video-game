# Design

Working design documents. These supersede the material under
[`../combat-design/`](../combat-design/), which is the 2016–2019 source archive exported
from Google Drive and is kept as reference, not as spec.

Start with the [combat kernel](combat-kernel.md) — every class document assumes it.

| Document | Status |
| --- | --- |
| [combat-kernel.md](combat-kernel.md) | Decided |
| [defense.md](defense.md) | Proposed |
| [dual-mage.md](dual-mage.md) | Meter decided; ascension is an open proposal |
| [bulwark.md](bulwark.md) | Proposed |
| [bellator.md](bellator.md) | Decided |
| [gatekeeper-retirement.md](gatekeeper-retirement.md) | Decided |
| [elementalist.md](elementalist.md) | Decided |
| [ability-spec.md](ability-spec.md) | Proposed |
| [parked.md](parked.md) | Parked — progression and equipment |

## Class kits

Six abilities each, plus an auto and the mechanic input. Written against
[ability-spec.md](ability-spec.md).

| Kit | Mechanic / what abilities spend |
| --- | --- |
| [Shadow Reaver](kits/shadow-reaver.md) | Shadow position |
| [Elementalist](kits/elementalist.md) | Structure slots on the field |
| [Blood mage](kits/blood-mage.md) | Health |
| [Dual mage](kits/dual-mage.md) | Meter position — ascension still open |
| [Bellator](kits/bellator.md) | Rush charge and which form you end in |
| [Bulwark](kits/bulwark.md) | Shield position |

## Roster

| Class | State |
| --- | --- |
| Shadow Reaver | Strong. Mobility no longer gated on shadow placement. |
| Elementalist | Strong. Structure ruling settled; archive versions reconciled. |
| Blood mage | Decent. Now tunable — was blocked on TTK. |
| Dual mage | *Formerly Statera.* Meter reworked to two forms per ability. Ascension unresolved. |
| Bellator | *Formerly Shifter.* Mid-animation form swap is the core addition. |
| Bulwark | New. Replaces the Gatekeeper. |
| ~~Gatekeeper~~ | Retired. |

## Next decisions

The focus is **building the six core classes**. Progression and equipment are parked; see
[parked.md](parked.md).

1. **Dual mage ascension.** The one unresolved core mechanic. Proposal is in
   [dual-mage.md](dual-mage.md).
2. **A prototype**, to start putting real numbers on the frame vocabulary. The Bulwark
   exercises the whole defensive layer; the Bellator exercises the swap window, which
   nothing else uses.
3. **Arena size and shape.** Determines whether a space-denying class can corner anyone, and
   whether block pushback has teeth.
4. **Control scheme pass.** The archived key map predates the new system, no cooldowns, the
   block proposal, and the Dual mage's tap/hold form select.

## Per-class blockers

| Class | What it still needs |
| --- | --- |
| Dual mage | Ascension resolved. The kit is written around the gap |
| Bulwark | Possibly a seventh slot for a dedicated ally-cover stance |
| Bellator | Whether the mid-animation swap costs Rush |
| Shadow Reaver | Whether the shadow has collision |
| Elementalist | Structure cap of three is a readability guess, not a balance one |
| Blood mage | Health cost as flat or percentage. Downstream of TTK either way |

All six need frame counts and damage numbers, which need a prototype.
