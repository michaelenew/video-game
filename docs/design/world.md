---
status: proposed
proposed: 2026-09-30
---

# The world — a rudimentary one

The creatures in [bestiary.md](bestiary.md) need somewhere to live and a reason
to be fought in some order. This is the smallest world that gives them both: a
valley of places, a town at its mouth, paths between them that open as you get
better, and rewards that change how you play rather than how strong you are.

It un-parks part of [parked.md](parked.md), deliberately, and keeps both of its
firm conclusions: **character power must not be able to get you past a wall**,
and **versus must not be affected by anything earned here**.

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

## 2 · The shape: a valley of places

**A world made of arenas.** The world is a small graph of **places**. Each place
is one arena (made possible by bestiary P2, arenas as data), with a creature or
without one, and exits to its neighbours. Only one place is loaded at a time.
Between fight places there are **trails**: short traversal places with no
creature, where the challenge is the movement system itself — a ledge line that
wants a held jump, a gap that wants the Reaver's shadow or an Elementalist stone.
Trails are cheap to make, they teach movement between fights, and they are the
exploration.

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

## 3 · The tiers and their gates

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

### Sidegrades — the parked proposal, made concrete

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

Beyond the bestiary's P1–P8:

- **W1 · Places.** A place is an arena id plus exits. The snapshot holds the
  current place; moving through an exit is a simulation event, so both peers
  change place on the same frame. Loading is the renderer's problem, and it has
  one frame's warning.
- **W2 · Progress.** Trophies, unlocked sidegrades, tempers beaten and notes seen.
  Stored the way a player's settings are stored — a file on the desktop, local
  storage in the browser — so it lives in `crates/game/src/platform.rs` and
  nowhere else ([CLAUDE.md](../../CLAUDE.md), "the browser build is the same
  program"). It is **not** in the snapshot: nothing in a fight depends on it
  except which sidegrades are equipped, and those go in at the start of a hunt
  as part of each fighter's setup, the same way the class does.
- **W3 · Sidegrades.** A sidegrade is a set of Oven overrides applied to one
  fighter for one hunt. That is the cheapest possible implementation, and it
  keeps "every magnitude is a knob in the Oven" true: a sidegrade is a named
  bundle of knob changes plus, where needed, a flag that a kit reads.
- **W4 · Trails.** Traversal places with no creature. Built out of the same solids
  the arenas use.
- **W5 · Hearth.** The hub: the armoury (choose two sidegrades), the trophy wall,
  the notes, the dummy and the sparring bot, the way out to versus.

Order: W1 and W4 as soon as arenas are data (after the Hornback); W2 and W5 once
three creatures exist, because a hub with one trophy on its wall is a menu; W3
once the first sidegrade family is written.

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

## 8 · Questions for you

1. **Is "sidegrades plus keys" the right reward shape**, or do you want the
   world's first version to reward only trophies and tempers, leaving progression
   parked a while longer?
2. **Two slots** — one auto modifier, one mechanic modifier. Enough, or too few?
3. **Tempering** — three tempers per creature on the glance/lead/decisiveness
   knobs. Is "the same creature, cleverer" the ceiling you want?
4. **The Siegeshell as the ending**, arriving on its own and walking at your
   town. Keep, or keep it as a place like the others?
5. **Hearth's wall can be breached.** If the Siegeshell reaches the wall, does
   the hunt simply fail, or does the town carry the scar (a district in ruins
   until you win)? The second is more memorable and costs more.
