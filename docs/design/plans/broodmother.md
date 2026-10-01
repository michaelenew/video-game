---
status: action plan — one implementation thread
opened: 2026-10-01
implements: ../creatures/broodmother.md
---

# Broodmother — action plan

> **State, 2026-10-01 (built).** Every milestone done; the numbers and what
> fell short are the creature's §13. Open: zero unanswerable hits (one or two
> hidden broodling bites in 24 per class), the fight's length (two and a half
> minutes for a won Champion fight, not three to six), and the roster's other
> classes, whose kits the hunter does not play. Branch
> `claude/creature-broodmother`.

The specification is [`../creatures/broodmother.md`](../creatures/broodmother.md);
the recipes are [`../species.md`](../species.md), [`../critters.md`](../critters.md)
§6, [`../arenas.md`](../arenas.md) and [`../hazards.md`](../hazards.md) §7. Parity
bar: [`../monsters.md`](../monsters.md), contract [`../bestiary.md`](../bestiary.md) §1.

## Decisions taken up front

- **Her brood are the Gnawers' gnawer**: her pack declares
  `gnawers::GNAWER_KIND` as its one kind and her move table starts with the
  gnawer's five moves (`gnawers::MOVES[0..5]`, marked `never_chosen` for her
  body), so the gnawer's mind (`gnawers::Mind`) drives them unchanged. Her
  own pack mind wraps it for what is hers: the screech's recall, the Brood
  guard, the rooted cap, the collapse. No second gnawer.
- **Her state** is the hunt's lore (sac sites, list, strands' bookkeeping)
  plus four words on the body (`Monster::own`) for what the hit path and the
  rig read: which sacs are there, which popped this frame.
- **Sacs are breakable soft parts** with `weak_point`-free high vulnerability;
  a hit on a sac goes to the sac (her species' `struck` hook refunds her own
  health), and a sac with no body (empty, scarred) is absent through
  `FightDecl::presence`.
- **The stab is one move**, the leg chosen at commit (`FightDecl::commit`) and
  lifted procedurally by the last word on the pose (a new `FightDecl::repose`
  seam -- named so rather than `layer`, which it is not), the hit lobbed to
  that leg's disc. The flurry is a second, never-chosen stab the species
  chains on.
- **The screech → slam chain is declared**: `MoveDecl::then(SLAM)`, run by the
  shared tick (a new generic seam, `None` everywhere else).
- **Web patches and strands are P4 hazards** (disc and strand shapes already
  exist); the trip and the root are her rules.
- **Signs, not marks** for the floor (rings under red sacs, landing discs,
  lanes, strands), so the report's "marked" rule counts them; marks folded
  into signs only if cheap.

## Milestones

- [x] **M1 · The body and the cave.** Species table (bones, parts, 8 legs,
      moves, clips, knobs), bootstrap files, registry lines, the Hollows arena
      with its vault, the reach probe and the two geometry tests,
      `beastcheck --species broodmother`.
- [x] **M2 · The clock.** Sac sites, ripen, hold, hatch, pop, regrow, the cap;
      the brood as gnawers under her. Clock/cap/pop tests.
- [x] **M3 · Her close moves.** Leg stab + flurry, fang lunge, slam, screech
      and the chain; floor signs. Chain test; screenshots.
- [x] **M4 · The web.** Web shot and patches, web line and strands, root and
      cutting free, fire burns web. Root and strand tests.
- [x] **M5 · The arc.** Brood guard, legs and the list, the clutch, the
      collapse, enrage; the brain's new terms. List and collapse tests.
- [x] **M6 · Animation.** Clips in `anim/src/beast/broodmother/`, baked, sheets
      reviewed.
- [x] **M7 · Measured.** Hunter plan (three plans), report lines, tuning
      passes; feel log.
- [x] **M8 · Reading it, world, docs, browser.** Look, dressing, telegraph
      screenshots, trophy, docs, manual, web smoke, merge main, checks, push.
