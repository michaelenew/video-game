---
status: in progress
started: 2026-10-01
branch: claude/polish-fights
---

# Polishing the fights the builds left open

The plan file for the pass after all ten bestiary creatures landed: the
defects their "Where it landed" sections name, fixed in the creatures (the
parallel `polish-hunter` pass teaches the scripted hunter; this one does not
restructure `crates/hunt`). Kept so the work can resume after an interruption.

## Order

1. **Contract breach -- pack critters wind up off-screen.** Mandatory. Zero
   unanswerable for every pack creature, six classes, solo and coop.
2. **Threatening share as an artefact of the measure**: the Pair, the
   Galewing, the Sandmaw. Per creature: the measure or the creature.
3. **Specific misses**: Galewing, Siegeshell, Gnawers, Broodmother, Mireback,
   Veilstalker, Mantis -- in that order, as far as time allows.
4. Merge origin/main, re-run every creature changed with all six classes,
   update the "Where it landed" tables and bestiary.md's cross-cast table.

Every item: a dated feel-log entry, the creature doc's §13, tests that pin it.

## How to measure

`scratchpad/pf/all.sh <species> <repeats> <hunters>` runs the six classes in
parallel (`target/release/fight`). Solo 24 seeds, coop `--hunters 2` 12.

## Status

- [x] 1 contract breach -- `pack::watch`/`unanswered`, glance keeps a committed target, report asks the fighter reached, Mireback's fresh-tar rule fixed. Zero for Gnawers, Hornback, Mireback, Broodmother, Siegeshell, six classes, solo 24 / coop 12.
- [x] 2 threatening share -- Pair: measure (per body; `together` line). Galewing: measure (windowed in reach; lift/perch not threats). Sandmaw: target (its tally's §9 measure is right; stand throws moves).
- [ ] 3 specific misses -- Galewing done (measure: walkable floor; creature levers reverted, cost the ride). Next: Siegeshell, Gnawers, Broodmother, Mireback, Veilstalker, Mantis.
- [ ] 4 merge and re-measure

## Log

- 2026-10-01: item 1. Data: every brood hidden hit began behind a Hollows
  pillar (sight blocked from the chest) or below the screen of a hunter
  pitched up at a sac. Rule kept by the referee (exchange), not the brain.
- 2026-10-01: item 2. Single-body creatures' windows verified identical
  after the report change (same seeds, same lines).
- 2026-10-01: merged origin/main (class-playing hunter). Hornback/Mireback
  unanswerable zero on this branch. Fire pillar investigated: bestiary §8.
- 2026-10-01: Galewing. Coop losses are the scripted pair's jumps (falls out
  of the jump under 2 m), not the bird.
- 2026-10-01: Siegeshell alone anchors 5900 (committed). Merged #127.
  Gnawers: health tried, reverted (costs weak classes). Broodmother: plan's
  (0 % time on sacs). Mireback: pause 60, health 11000 (committed).
  Next: Veilstalker way-in, Mantis Bulwark, then final re-measure + review.md.
