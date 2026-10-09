---
status: decided in part; W0, W1 and W2 built
proposed: 2026-09-30
decided: 2026-09-30
built: 2026-10-01 (W0), 2026-10-01 (W1, W2)
---

# The world — a rudimentary one

The creatures in [bestiary.md](bestiary.md) need somewhere to live and a reason
to be fought. This document holds two things: **what is being built now**, which
is deliberately small, and **the valley** it is heading toward, kept so the
reasoning does not have to be re-derived.

## 0 · Decided, 2026-09-30

- **The world is, for now, separate arenas you teleport between from the dev
  harness.** No valley, no trails, no hub, no gates. Each creature's fight is its
  own arena (bestiary P2, arenas as data), and the harness has a picker that puts
  you in one. §2–§3 below are the later picture, not the build.
- **Rewards are trophies and tempered rematches only.** Beating a creature
  records its trophy; a beaten creature can be fought again tempered (§4).
  **Sidegrades stay parked** in [parked.md](parked.md). Every creature document
  proposes one in its §11; those are notes toward the parked system, not work.
- **Defence fights stay.** The Hornback's escort variant and the Siegeshell's
  wall are in the design (bestiary P7).
- **The cast in [bestiary.md](bestiary.md) is the first mix.**

What that means to build is in §6. **All three of the "now" items are built**
(2026-10-01): the picker (W0), trophies (W1) and tempers (W2).

It keeps both of [parked.md](parked.md)'s firm conclusions: **character power
must not be able to get you past a wall**, and **versus must not be affected by
anything earned here**. Trophies and tempers change neither.

---

## 1 · The thesis, restated for a world

**Player skill is the progression.** [parked.md](parked.md) says it, and a world
is where it has to be made true. So:

- **What gates a path is a fight you have won**, never a number you have raised.
  There is no experience, no levels, no damage upgrades, no armour.
- **Every gate is "beat N of these M".** A player stuck on one creature has
  another to try. Arenas are the cheapest content in the game, and breadth at
  each tier is what stops a skill gate from being a wall.
- **Coop is the accessibility path.** A fight you cannot win alone, you can win
  with somebody. That is social, and it does not undermine the premise the way
  grinding would.
- **What you earn is a sidegrade or a key.** Sidegrades change playstyle; keys
  open the world. Neither changes whether you can beat the next thing.

## 2 · Later: a valley of places

*2026-10-09: the valley is **one map** now -- you walk from Hearth to the
Saddle and into every room, with no teleports and no "only one place is
loaded" -- see [atlas.md](atlas.md). The paragraph below about loading one
place at a time describes the earlier build.*

*2026-10-08: [exploration/0006](exploration/0006_valley.md) argued for a line
rather than a graph -- one long climb loaded in reaches -- and **the valley
is built**, the same day, as [valley.md](valley.md): Hearth with a Ring for
fighting each other, five reaches to the Saddle, the creatures' arenas as
rooms off them, the tiers below as waystones. The map below is the graph it
replaced; the Ridgeback's place is the Highlands, a room up a stair in the
Pinewood. Not built from this section: looking into a room to see the
creature's idle life, and the Crossing as a place.*

**A world made of arenas.** The world is a small graph of **places**. Each place
is one arena (made possible by bestiary P2, arenas as data), with a creature or
without one, and exits to its neighbours. Only one place is loaded at a time.
Between fight places there are **trails**: short traversal places with no
creature, where the challenge is the movement system itself — a ledge line that
wants a held jump, a gap that wants the Reaver's shadow or an Elementalist stone.
Trails are cheap to make, they teach movement between fights, and they are the
exploration. **The first six are built** as measuring instruments, with the envelope
they were built against: [courses.md](courses.md) (2026-10-03).

You see a creature before you fight it. Every fight place can be looked into
from the trail that leads to it, and the creature is doing its **idle life**:
the herd grazing, the Galewing circling its tower, the Sandmaw's fin cutting
the dunes, nothing at all in the ash where the Veilstalker is. Walking in starts
the hunt, and the creature's moment of noticing you (the Ridgeback's
`hunt_grace`) covers the transition.

```text
                                  ┌──────────── The Shrine (Mantis) ──┐
                                  │                                   │
        The Cliffs (Galewing) ── Saddle ── The Ashfield (Veilstalker) │
                  │                                   │               │
   The Highlands (Ridgeback) ── Pinewood (The Pair) ── The Hollows (Broodmother)
                  │                                   │
         The Dunes (Sandmaw) ─────── Crossing ─────── The Mire (Mireback)
                  │                                   │
        The Commons: Gnawer den ── meadow ── Hornback migration
                                     │
                          ═══════ HEARTH ═══════   ← the Long Valley runs out
                         (the walled town, hub)       from here: the Siegeshell
```

**Hearth** is the hub: the walled town at the valley's mouth. The training dummy,
the sparring bot and versus live here, and so does the armoury where sidegrades
are chosen. Its wall is the one the Siegeshell walks toward, so the last fight in
the world is defending the place the world started from.

## 3 · Later: the tiers and their gates

| Tier | Places | To leave this tier, beat |
| --- | --- | --- |
| 1 · The Commons | Gnawers, Hornback herd | 1 of 2 |
| 2 · The lowlands | Sandmaw (Dunes), Mireback (Mire) | 1 of 2 |
| 3 · The heights and the woods | Ridgeback (Highlands), The Pair (Pinewood), Broodmother (Hollows) | 2 of 3 |
| 4 · The edges | Galewing (Cliffs), Veilstalker (Ashfield) | 1 of 2 |
| 5 · The end | Mantis (Shrine), Siegeshell (the Long Valley) | the Siegeshell ends the world |

**The gate is physical.** Each tier's way out is a **waystone** on the trail. It
lights when you have beaten enough of that tier, and the trail beyond it opens.
The count is the only thing the waystone reads: no fight is required in
particular.

**The Siegeshell comes to you.** It is not a place you walk to. When the Shrine's
waystone lights (or any two of tiers 4 and 5 are beaten — decide in play), the
Siegeshell appears at the far end of the Long Valley, and the next time you are
in Hearth the bells ring. It is the one fight the world starts for you.

**Nothing is locked behind a specific creature**, with one exception worth
arguing about: the Siegeshell is the ending, and it is locked behind tier 4. A
player who cannot beat either tier-4 creature alone can bring a friend.

## 4 · What you earn

Four kinds of reward, in order of how much design they need.

### Trophies — the count the waystones read

Beating a creature earns its trophy: a part of it, hung in Hearth (a gnawer
leader's skull, the bull's horn, a Sandmaw tooth, a slag-crusted Mireback wart, a
Ridgeback nape plate, the Pair's two collars, a Broodmother sac husk, a Galewing
pinion, a Veilstalker hide patch that still shimmers, a Mantis blade, a piece of
the Siegeshell's anchor). Trophies are the gate count and nothing else. They are
also the one reward that costs almost nothing to build and that every player
understands at once.

### Sidegrades — ⚠️ parked, 2026-09-30

*Not being built. Kept as the shape the parked system would take if it is
un-parked; the per-creature proposals are in each document's §11.*

[parked.md](parked.md)'s leading proposal was **auto modifiers**: items that
change what your auto attack does, as playstyle rather than power, with the
unlocks forming the early ramp. Its second was **mechanic modifiers**: one or
two numbers on the class mechanic, which propagate through the whole kit. Both
survive here, and the world adds the rule for how you get them:

- **Each creature teaches a sidegrade family.** Beating it unlocks one sidegrade
  per class, themed on what the creature is. The family is the same idea on six
  kits (for example, the Gnawers' family is about crowds; the Veilstalker's is
  about marking). Eleven creatures, six classes, about sixty-six sidegrades —
  which is too many to author at once, so the first pass is **one family per
  creature, written for whichever classes it is obvious for**, and the gaps are
  filled as kits settle.
- **The slot budget is the mechanism.** You equip **one auto modifier and one
  mechanic modifier**. Two slots, chosen in Hearth. Without a tight budget,
  "which do I want" collapses into "all the good ones" and there is no decision.
- **Every sidegrade is a trade.** "Longer leash, slower return." Each is written
  with its cost in the same sentence, or it is not a sidegrade.
- **Versus ignores all of it** by default: a ranked or plain versus match uses
  the base kits. A "loadout versus" in which everyone may equip anything —
  unlocked or not — can exist later, and would be a different ruleset rather
  than a reward for grinding.

Each creature's document proposes its trophy and one sidegrade in §11; the table
below collects them once the documents are in.

| Creature | Trophy | Sidegrade family (theme) |
| --- | --- | --- |
| Gnawers | the leader's skull | crowds — hits that spread |
| Hornback herd | the bull's horn | momentum — charging, knockback into walls |
| Sandmaw | a ring tooth | noise and silence — moving quietly, baiting |
| Mireback | a slag-crusted wart | the floor — leaving or clearing ground effects |
| Ridgeback | a nape plate | the climb — reach, footing, grip |
| The Pair | two collars | the flank — rewards for hitting from behind or beside |
| Broodmother | a sac husk | priority — marking a target, splitting damage |
| Galewing | a pinion | the air — hang time, aerial control |
| Veilstalker | a patch of hide | sight — marks and reveals |
| Mantis | a blade | the guard — parry windows, counters |
| Siegeshell | a shard of anchor | *(none: it ends the world)* |

### Tempered hunts — the part with no roof

A beaten creature can be fought again **tempered**, and tempering is the
Ridgeback's own difficulty model: **its glance shortens, its lead lengthens,
its decisiveness rises**, and its strain thresholds stop falling as far with its
health. Those are the knobs [monsters.md](monsters.md) §7 says *are* the
difficulty model, and they change how clever it is rather than how much health
it has. Three tempers per creature, each with its own trophy mark. This is the
"effectively infinite skill ceiling" parked.md asks for, and it costs no content.

### Knowledge — a hunter's notes

The first time you survive a move, its row appears in a notes page for that
creature: the move's name, a drawing of its tell, and — once you have answered
it — the answer you used. It is the fight report's table, player-facing. It
costs little, it teaches, and it is the thing a person shows somebody else.

## 5 · Coop in the world

Two players in the same place fight the same creature. **Each keeps their own
progress**; a fight counts for both. A player may join a friend's hunt of a
creature they have not unlocked — coop is the accessibility path, and a locked
place is locked to the lone player, not to the pair. The creature's coop tuning
(each creature's §8) applies whenever two are present.

## 6 · What it needs built

**Now:**

- **W0 · The arena picker.** The dev harness gets a list of arenas, one per
  creature, and puts both players into the chosen one with its creature. It is a
  dev-harness command and a `--hunt <creature>` flag (and `?hunt=<creature>` in
  the browser, since the query string does what the flags do). Which arena is
  loaded is in the snapshot, so both peers change on the same frame.
  **Built 2026-10-01** with P2 ([arenas.md](arenas.md) §3): `--hunt <creature>`,
  `?hunt=<creature>`, `--arena <name>`, and `H` / `Shift+H` in game, which
  travel by a byte on the wire so a peer goes on the same frame. The "list" is
  the cycle for now; a menu is for when there are creatures to list.
- **W1 · Trophies.** A record of which creatures a player has beaten, and at
  which temper. Stored the way a player's settings are stored — a file on the
  desktop, local storage in the browser — so it lives in
  `crates/game/src/platform.rs` and nowhere else ([CLAUDE.md](../../CLAUDE.md),
  "the browser build is the same program"). It is **not** in the snapshot:
  nothing in a fight depends on it. Shown in the picker beside each creature.
- **W2 · Tempers.** A temper is a set of Oven overrides on a creature's brain
  knobs (glance, lead, decisiveness, how far its strain thresholds fall), applied
  when the hunt starts. That keeps "every magnitude is a knob in the Oven" true.
  The picker offers the tempers a player has earned.
  **Built 2026-10-01**, with W1:
  - **A temper is four shares, and a creature gets three by existing.** The
    Oven's "Tempers" family holds, per temper, the creature's glance and lead as
    a percentage of its own, points added to its decisiveness, and how far its
    strain thresholds fall as a percentage of its own fall
    (`crates/sim/src/temper.rs`). Shares rather than per-species overrides so a
    new species declares nothing; species.md §5 step 11 is what it must not do.
    A pack's glance, lead and critters' thinking cadence take the same shares,
    so the Gnawers and the herd are tempered the same way the Ridgeback is.
  - **It is in the snapshot**: a byte on each creature and on the pack, hashed
    only when non-zero (a hunt as tuned hashes as it always did), and it
    **travels in the picker's byte** -- `1tt sssss`, two bits of temper beside
    five of species -- so both peers build the same tempered hunt on the same
    frame and a rollback replays it (`net/tests/rollback.rs`). A restart keeps
    it; `H` and `Shift+H` start a creature as tuned; versus has nothing to
    temper.
  - **The ladder, measured** (`fight --temper n`, Champion, forty seeds): 20,
    12, 8 and 1 hunts won of 40 at tempers 0 to III. Two Champions: 15 and 13
    of 20 at 0 and III -- coop is the accessibility path. The first values
    (I = 80/120/+10/75) made temper I nearly as hard as III; see the feel log.
  - **`T`** steps the hunted creature to the next temper on offer: temper N
    once N-1 of that creature is beaten. `--temper <n>` (`?temper=`) starts at
    any temper and lets `T` offer all of them.
- **W1, as built.** A record, per species id, of the tempers beaten:
  `crates/game/src/trophies.rs` for what it is and `platform.rs` for where
  (`~/.config/arena/trophies.conf`, `ARENA_TROPHIES` to move it, or the
  browser's `arena.trophies` key). Keyed by id, so a record written by a build
  with more creatures keeps their lines. **Won** is generic: the round ends in
  the hunters' favour -- every creature down and every small body dead or gone,
  or a defended thing that wins it arriving (`World::hunt_won`). It is written
  sixteen frames into the round-over pause, past any rollback, so a predicted
  kill that is taken back never reaches the disk. Each peer writes its own,
  and a player may go along on a temper they have not earned (§5). The picker's
  list -- every creature, its trophies, its tempers, which is being hunted --
  is a small text block at the right of the HUD.

**The valley, built 2026-10-08** ([valley.md](valley.md)): places with exits
(seams you walk into together), the reaches as the trails, Hearth and its Ring,
waystones reading what the pair has beaten, cairns. It is where the game starts.
**Later:** the hunter's notes (§4), the Siegeshell coming to you (§3), and — if
progression is un-parked — sidegrades as named bundles of Oven overrides.

W0 comes with P2 (after the Hornback in the bestiary's build order); W1 and W2
are small and can follow any time after.

## 7 · Deliberately not yet

- **Creatures meeting each other.** A Galewing taking a cow from the herd, gnawers
  scattering from a Sandmaw's ripple. It is exactly the kind of thing that makes a
  world feel alive, and it needs two creatures loaded at once, which the budget
  and the harness both argue against for now.
- **A story.** The world has a premise (a town at the mouth of a valley full of
  things, and something walking toward it) and no more. Writing is expensive and
  the art thesis says spend it last.
- **Open terrain between places.** Trails are arenas-without-creatures on
  purpose. An open world is the thing the combat kernel's closed arena was chosen
  to avoid.
- **Economy.** No currency, no crafting. A trophy is not spent.

## 8 · Questions still open

1. **Tempering** — three tempers per creature on the glance / lead /
   decisiveness knobs. Is "the same creature, cleverer" the ceiling you want?
2. **The Siegeshell's breach.** Settled for now by the Siegeshell document: the
   first beam takes the gate and the second loses the hunt. Whether a town
   carries the scar only matters once there is a town.

*Answered 2026-09-30:* rewards are trophies and tempers only; sidegrades stay
parked; defence fights stay; the world is separate arenas for now.
