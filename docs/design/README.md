# Design

Working design documents. These supersede the material under
[`../combat-design/`](../combat-design/), which is the 2016–2019 source archive exported
from Google Drive and is kept as reference, not as spec.

Start with the [combat kernel](combat-kernel.md) — every class document assumes it.

| Document | Status |
| --- | --- |
| [combat-kernel.md](combat-kernel.md) | Decided |
| [defense.md](defense.md) | Proposed |
| [statera.md](statera.md) | Meter decided; ascension is an open proposal |
| [bulwark.md](bulwark.md) | Proposed |
| [shifter.md](shifter.md) | Decided |
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
| [Statera](kits/statera.md) | Meter position — ascension still open |
| [Shifter](kits/shifter.md) | Rush charge and which form you end in |
| [Bulwark](kits/bulwark.md) | Shield position |

## Roster

| Class | State |
| --- | --- |
| Shadow Reaver | Strong. Edge work only — check that mobility is not fully gated on shadow placement. |
| Elementalist | Strong. Structure ruling settled. |
| Blood mage | Decent. Now tunable — was blocked on TTK. |
| Statera | Meter reworked. Ascension unresolved. |
| Shifter | Foundation good; mid-animation swap added. Rename pending. |
| Bulwark | New. Replaces the Gatekeeper. |
| ~~Gatekeeper~~ | Retired. |

## Next decisions

The focus is **building the six core classes**. Progression and equipment are parked; see
[parked.md](parked.md).

1. **An ability spec template.** Every class needs recosting for no cooldowns, and without
   a shared format for what an ability *is* — frame windows, resource cost, mechanic
   interaction — six kits get written six different ways. Cheap, and it unblocks all of them.
2. **Statera ascension.** The one unresolved core mechanic. Proposal is in
   [statera.md](statera.md).
3. **Class kits**, written against the template. The Bulwark is greenfield and stress-tests
   the new defensive system; the Reaver, Elementalist, and Blood mage mostly need recosting
   and reconciling of duplicate versions in the archive.
4. **Arena size and shape.** Determines whether a space-denying class can corner anyone, and
   whether pushback has teeth.
5. **Control scheme pass.** The archived key map predates the new system, no cooldowns, and
   the block proposal.

## Per-class blockers

| Class | What it still needs |
| --- | --- |
| Statera | Ascension resolved. The kit is written around the gap |
| Bulwark | Possibly a seventh slot for a dedicated ally-cover stance |
| Shifter | Whether the mid-animation swap costs Rush. Rename |
| Shadow Reaver | Whether the shadow has collision |
| Elementalist | Structure cap of three is a readability guess, not a balance one |
| Blood mage | Health cost as flat or percentage. Downstream of TTK either way |

All six need frame counts and damage numbers, which need a prototype.
