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
| [parked.md](parked.md) | Parked — progression and equipment |

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

| Class | What it needs |
| --- | --- |
| Statera | Ascension resolved, then a kit written against the new meter |
| Bulwark | A full kit. Unblocked by [defense.md](defense.md) |
| Shifter | Kit written out, including the form-swap ending matrix. Rename |
| Shadow Reaver | Decide whether there is any mobility with no shadow placed, then recost |
| Elementalist | Reconcile the two overlapping kit versions in the archive, then recost |
| Blood mage | Recost. Now tunable — was blocked on TTK |
