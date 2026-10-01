---
status: proposed; cast accepted 2026-09-30
proposed: 2026-09-30
---

# The bestiary — ten creatures after the Ridgeback

The Ridgeback proved one monster fight can be good, and [monsters.md](monsters.md)
records how it got there: eight play reports, four rebuilds, and a harness that
caught most of what was wrong before a person did. This document plans the rest
of the cast. It has three parts:

1. **The contract.** What the Ridgeback taught about what a monster must be,
   written as a checklist every new creature is designed and measured against.
2. **The cast.** Ten creatures, chosen so that between them (and the Ridgeback)
   they vary size, pack size, difficulty and the kind of approach you need to beat
   them. Each has its own document under [creatures/](creatures/).
3. **What has to be built first.** The Ridgeback's code has one monster slot,
   one arena, and no idea that there could be a second species. Some shared
   machinery comes before any of the ten.

The world these creatures live in is [world.md](world.md).

---

## 1 · The contract

Every one of these is a lesson the Ridgeback paid for, usually more than once.
A creature design is not finished until it has an answer to each.

1. **One sentence says what the fight is.** The Ridgeback's is *"you cannot
   reach the thing that kills it from the ground, and you cannot get off the
   ground without earning it."* If the sentence is "it has a lot of health and
   hits hard", there is no fight yet.
2. **Every move has a different answer.** The table of moves has a column
   called *the answer*, and no two rows in it say the same thing. Dodge, jump,
   sidestep, get out of the lane, get behind a solid, crouch, hit it first,
   leave its back. A creature whose every move is answered by the dodge is
   testing one skill.
3. **There is a threat at every range a class fights from.** Close, mid, long,
   and — for the creatures you can climb — aboard. The Ridgeback could be
   backpedalled away from and shot for free until it grew a charge and a spray.
   Every creature says what it does to somebody standing at each range.
4. **The windup is the move, and it is drawn on the floor.** Most of a move's
   frames are its tell, the silhouette changes early and a lot, and the floor
   marker is the hit test's own answer (`Monster::telegraph`). A tell under
   about fifteen frames can't be answered on sight, so a move that fast must be
   answerable by *where you stand*, and the fight report counts the hits that
   are neither.
5. **It commits, it telegraphs, and it does not read your buttons.** It sees
   by glancing (a sample of position and velocity every few frames, projected
   forward), not by watching. The hit locks where it was aimed, so you can make
   it miss. Anything that looks like a reaction to you is a reaction to
   something it *saw*, at least fifteen frames old.
6. **Something you do changes it for the rest of the fight.** A broken foot
   makes the Ridgeback turn worse on that side and kneel lower. Each creature
   has at least one thing like that, and the player can see the consequence of
   a thing they did.
7. **There is a big window, and the approach is how you earn it.** The topple
   is what the Ridgeback's climb is *for*. Without a reward window the fight is
   arithmetic. Each creature names its window and the route to it.
8. **Crowd control is a negotiation, not a switch.** The strain thresholds
   (recent damage buys susceptibility, then an interrupt) apply to every
   creature, so half of every kit works in a hunt without making the creature
   something you lock down. Small creatures sit lower on the same scale.
9. **Nothing safe, nowhere free.** The tail root was a safe spot for a week.
   Every creature says where the tempting safe spot is and what covers it.
10. **It is measured.** The fight report (`cargo run -p hunt --bin fight`)
    measures reactable share, openings, entropy, coverage, landed/thrown per
    move, the four windows (threatening / poke / way in / walk up), and
    **unanswerable hits, which must be zero**. Each creature adds the measures
    its own idea needs, and says what the scripted hunter's plan is. The
    scripted hunter is a decent player with one plan; its win rate is a floor.
11. **Each class is looked at.** The Dual mage floats over the Ridgeback's
    ground game, the Bulwark can reach only its two lowest routes, and the
    Elementalist can't win. Each creature has a paragraph per class saying who
    has it easy, who has it hard, and whether that is identity or a hole.
12. **Coop is a different monster.** Numbers are set for one hunter; each
    creature says what changes for two.

## 2 · The cast

Eleven creatures in total with the Ridgeback, on five tiers of difficulty. The
tier is a target for the scripted hunter (the "decent player") against a solo
fight, and a target length of fight inside the one-to-twenty-minute coop band
the [combat kernel](combat-kernel.md) asks for.

| Tier | Scripted hunter wins | Fight length (solo, won) |
| --- | --- | --- |
| 1 | about nine in ten | 1–2 min |
| 2 | about two in three | 2–4 min |
| 3 | about one in three (the Ridgeback today: four in twelve) | 3–6 min |
| 4 | about one in six | 5–10 min |
| 5 | about one in twenty — built for two | 8–20 min |

| Creature | Size | Pack | Tier | The fight in one line | The approach it teaches |
| --- | --- | --- | --- | --- | --- |
| [Gnawers](creatures/gnawers.md) | knee-high, 0.6 m | 5–7 and a leader | 1 | Nothing in the pack can hurt you much; the pack can | Keep them in front of you; count them; kill the leader and the rest break |
| [Hornback herd](creatures/hornback.md) | 2.2 m cows, a 3.2 m bull | 8 and the bull | 1 | The herd is the terrain; the bull is the fight | Bait a charge into something solid; get out of a stampede's lane, or ride it |
| [Sandmaw](creatures/sandmaw.md) | 11 m worm, 2.4 m thick | 1 | 2 | It is only there when it chooses to be — so make it choose | Make noise where you want it to surface; stand still to vanish |
| [Mireback](creatures/mireback.md) | 7 m toad, 12 m wide | 1 | 2 | It does not chase you; it takes the floor away | Burn the tar it spreads — which clears the floor and builds the steps up it |
| **Ridgeback** ([monsters.md](monsters.md)) | 13 m, 5.5 m at the back | 1 | 3 | You can't reach what kills it from the ground, and you can't leave the ground without earning it | Break a foot, climb, ride the buck, topple it |
| [The Pair](creatures/the-pair.md) | two 1.8 m cats | 2 | 3 | Either one alone is fair; together, one is always behind you | Never commit to one while you can't see the other; split them |
| [Broodmother](creatures/broodmother.md) | 6 m, egg sacs on her back | 1, spawning | 3 | Every second on the little ones she makes more; every second ignoring them, they eat you | Choosing a target; popping sacs in the window her slam opens |
| [Galewing](creatures/galewing.md) | 18 m wingspan | 1 | 4 | It lives where you can't reach — make it come down, then choose whether to go up with it | Wing-strike it on a low pass; ride it into the sky and get off in time |
| [Veilstalker](creatures/veilstalker.md) | 4 m, unseen | 1 | 4 | You never see it; you see what it touches | Reading the world instead of the body: footprints, fog, the shimmer before a strike |
| [Mantis](creatures/mantis.md) | 3 m, two blades | 1 | 5 | It fights like a player: it blocks, it parries, and it punishes what you repeat | Versus fundamentals — spacing, whiff punishing, mixing up, guard breaks |
| [Siegeshell](creatures/siegeshell.md) | 24 m tall, 40 m long | 1, and what lives on it | 5 | It isn't fighting you; it is walking to the wall, and you are the only thing that can stop it | Break its legs to bring it low, climb a moving mountain, break its anchors before it arrives |

### What the cast covers

**Size** runs from a creature you look down at to one whose foot is taller than
you — knee-high, human, horse, house, hill. Two things change with size and the
cast uses both. A small creature is hit by aiming; a big one is hit by *going
somewhere* on it. And the camera's framing problems flip: a gnawer is hard to
see under your own character, and the Siegeshell is hard to see at all.

**Pack size** runs 1, 1 + spawn, 2, 5–7, 9, and "a colossus with a population".
The shape of a pack is its own design problem. Gnawers take turns (only two may
attack at once) and break when their leader dies. The herd moves as one body
with a guardian. The Pair coordinate as roles. The brood are a clock you can
race or stop.

**Kinds of approach** — no two creatures are beaten the same way:

| Approach | Where it is taught |
| --- | --- |
| Spacing and crowd control | Gnawers |
| Using the arena: solids, lanes, high ground | Hornback herd |
| Baiting, and controlling what the creature perceives | Sandmaw (noise), Veilstalker (sight) |
| Changing the floor, and racing a clock you can push back | Mireback |
| Breaking parts to open a route up; riding | Ridgeback, Galewing, Siegeshell |
| Target priority | Gnawers (the leader), Broodmother (the sacs) |
| Awareness of what you can't see | The Pair, Veilstalker |
| Pure fighting-game skill | Mantis |
| Defending something | Hornback (the cart variant), Siegeshell (the wall) |

**Where the fights happen** — high ground, open floor, sand, tar, sky, snow, a
long valley — is also most of what makes [the world](world.md) a world rather
than a menu of arenas.

**Reuse.** The art thesis says to prefer depth that costs no assets, and the
cast leans on it. The Broodmother's brood are gnawers. The Pair are the
Ridgeback's skeleton re-proportioned. Gnawers live on the Siegeshell's back.
The herd's cows are the bull without horns. Every creature reuses the glance,
the scoring, the strain thresholds, the telegraph markers and the fight report.

## 3 · What has to be built first

The Ridgeback code was written to be shared ("a second monster should reuse the
body, ride and control machinery and supply only its own parts and moves"), but
nothing has tested that promise, and a survey of the code says it is true of the
*machinery* and false of the *data*. Every creature table is a Ridgeback
constant. These are the shared pieces, in the order the cast needs them. Each
creature's document says which ones it depends on.

### P1 · Species (every creature needs it)

**Built 2026-10-01**, with the Ridgeback bit-identical and the world holding two
creatures. What was built, the decisions, and the recipe for adding a creature
are in [species.md](species.md).

`Monster` has no species field. The bones, parts, legs, clips and moves are
`const` arrays, and a handful of moves are special-cased by name (`kind ==
SHAKE`, `kind == SWEEP`; the weak points are `RIDGE || NAPE`; the hunter bot
reads `monster::RIDGE`). The work:

- A `Species` table: skeleton, part boxes and their flags (mountable, solid,
  breakable, weak), legs, moves, clips, and the brain's numbers. `Monster`
  carries a species index; the rig reads the table instead of globals.
- The per-name special cases become **move flags** declared in the move table
  (`mirrors_to_target_side`, `scaled_by_shake_force`, `weak_point`,
  `breakable`), the same way `Move::aim()` is declared for fighters rather than
  inferred.
- The Oven's Ridgeback knobs become per-species families. The bake of
  `tuned.rs` has to keep working, and `tests/oven.rs` will say if it does not.
- The Ridgeback must come out of this bit-identical. `determinism.rs` and the
  fight report's twelve seeds are the check: the same seeds, the same wins, the
  same numbers.

### P2 · Arenas as data (everything past the first two creatures needs it)

There is one arena, `arena.rs`: 28 × 28 m, 1.5 m walls, two platforms, `const`
solids. The Siegeshell needs a valley two hundred metres long, the Galewing a
perch tower, the Sandmaw soft ground with rock islands, the Veilstalker a floor
that takes footprints. The work: an `Arena` value (bounds, solids, floor
material per region, spawns), chosen by id in the snapshot, with
`resolve`/`ground_under`/`inside` reading it. The current arena becomes one
entry. This is also the first half of [the world](world.md).

**Built 2026-10-01**, ahead of the Hornback so the creature branches can each
author their own arena: see [arenas.md](arenas.md) for what was built and the
recipe. The proving ground is the old arena bit for bit; a dev arena, the range,
has one of everything at the Siegeshell's size. Ceilings are solids hanging from
the roof.

### P3 · Small bodies and packs (Gnawers, Hornback, Broodmother, Siegeshell)

A full `Monster` is about two hundred bytes of a 4 KiB snapshot that is already
1.8 KiB full. Seven of them is most of what is left. Small creatures need a
**critter** form: one capsule, a position, a velocity, a yaw, health, a state,
a timer, and an animation clock — about thirty-two bytes. No skeleton in the
simulation (nothing stands on a gnawer; its pose is presentation, driven from
the clock in the snapshot so a rollback does not pop). Beside it, a **pack
brain**: attack tokens (how many may attack at once), roles, morale, and a
leader. One pack brain drives up to ten critters.

**Built 2026-10-01**, with the Ridgeback and versus bit-identical, ahead of the
four creatures that use it: see [critters.md](critters.md) for what was built,
the recipe, and what a person should review. A critter is 48 bytes, not 32 (a
vertical velocity for leaps and launches, and a perch for standing on a
creature); the snapshot is 3,520 bytes with ten of them and the brain. A1 and
A2 below landed with it.

### P4 · Floor hazards (Mireback, Broodmother, Veilstalker, Sandmaw, Siegeshell)

**Built 2026-10-01** with P5, P7 and the shared half of P6, the Ridgeback and
versus bit-identical: see [hazards.md](hazards.md) for what was built, the
recipe, and what a person should review. Sixteen bytes a hazard, not eight (a
strand's second end and a point that reaches the far end of the valley), in
the hunt's **lore** -- one region of the snapshot each fight lays out
differently -- rather than a fixed list every fight pays for. Discs and
strands; kinds and their effects declared by the species, with generic slow,
root, damage, pull, lift, blocks-sight, fire and runtime solids; anchored to a
creature part when they live on one.

Tar, web, smoke, burning ground, a sinkhole's pull, vented steam. A small fixed
list of circular floor areas — kind, centre, radius, age, state — about eight
bytes each, sixteen of them. The fighters' `effects` list is the precedent. The
list is what the renderer draws and what the hit test reads, which keeps the
rule that the overlay draws what the hit test uses.

### P5 · Perception beyond sight (Sandmaw, Veilstalker, The Pair)

**Built 2026-10-01**: a perception filter per species (cone, blind arc, line
of sight through `aim::sight_clear`, feel), hearing from a noise ring made by
diffing each fighter's body across the frame, `World::shown` for a creature
that is unseen, and A5. See [hazards.md](hazards.md) §3.

The glance samples every target. Three creatures need it to sample less. The
Sandmaw hears movement, not bodies. The Veilstalker is *un*seen, which is a
renderer fact that the simulation has to own, because a decloak is a tell and
tells are gameplay. One of the Pair can be half-blinded. The change is small and
central: the glance takes a **perception filter** per species, and what it
cannot perceive it does not sample.

### P6 · Flight (Galewing), and the long ride (Galewing, Siegeshell)

**The shared half built 2026-10-01: one fall rule** (§6 below, and
[hazards.md](hazards.md) §4). Flight and the long ride are still the
Galewing's and the Siegeshell's.

A 3D steering controller alongside the ground one, and a ride whose surface is
tens of metres off the floor. Falling damage stops being an afterthought. This
is the largest single piece of new machinery in the cast, and only two
creatures use it, which is why they are late.

### P7 · Objectives (Hornback's cart variant, Siegeshell)

**Built 2026-10-01**: a defended thing -- a box with health at a site the
arena declares, rolling along its route while escorted if it says so, a solid
if it says so -- reached by a creature's move with the volume it reaches a
fighter with, losing or winning the hunt, drawn, and counted by the fight
report. See [hazards.md](hazards.md) §5. The harness learning to defend is the
creature's.

Something besides the hunters that can lose: a wall with health, a cart with a
route. Small in code; large in what it changes about the harness, which has to
learn to defend.

### P8 · The harness per creature

**Built 2026-10-01**: a `Plan` and a `Card` per species in `crates/hunt/src/plans/`,
the report's lines read off the species table, `fight --species`. See
[species.md](species.md).

`crates/hunt` plays one plan against one creature and reads Ridgeback parts by
name. It needs a **plan per species** (the scripted hunter learns each fight the
way a person does, and each creature's document says what that plan is) and a
report whose per-move and per-part lines come from the species table. Without
this, every creature after the first is unmeasured, and the Ridgeback's history
says the unmeasured bugs are the ones a person reports as "it feels random".

## 4 · The budget

The snapshot is capped at 4 KiB. It was 2,680 bytes before P1 (the "1.8 KiB"
this paragraph first said was stale) and is **2,856 bytes** since, the second
creature slot included; one `Monster` is 180 bytes. Estimates, to be replaced by
`size_of` once each is built:

| Fight | What is in the snapshot | Estimate, from its document |
| --- | --- | --- |
| Ridgeback | one monster | ~200 B |
| Gnawers | pack brain + 7 critters | ~330 B (~390 B in coop) |
| Hornback | pack brain + 9 critters (the bull is a critter with extra state) | ~365 B |
| Sandmaw | one monster + noise ring + hazards | ~300 B |
| Mireback | one monster + 16 hazards | ~350 B |
| The Pair | two monsters + the pair's shared brain | ~430 B |
| Broodmother | one monster + pack brain + 8 critters + hazards and web strands | ~610 B |
| Galewing | one monster + flight state + a fall height per fighter | ~260 B |
| Veilstalker | one monster + 32 footfalls + hazards | ~390 B |
| Mantis | one monster + habit memory | ~280 B |
| Siegeshell | one monster with a 40-part rig + a pack of parasites + hazards + the wall | ~680 B |

Only one fight is loaded at a time, so the worst case is the Siegeshell at
about 2.5 KiB total. It fits, and it is the fight to watch.

**Measured 2026-10-01, with everything shared built** (P1 to P8): the `World`
is **3,912 bytes**, 184 to spare. Every creature's state beyond the two
monster slots, the ten critters and the pack brain lives in the hunt's
**lore**: 24 cells of 16 bytes that each fight lays out for itself -- so many
for hazards, noises and defended things, the rest its own. The itemised worst
cases are the Siegeshell (19 cells) and the Mireback (18); every creature has
five or more cells to grow into. `tests/lore.rs` checks every layout fits, and
architecture.md §"Where the 4 KiB goes" has the table.

The frame budget matters more than the bytes: a rollback re-simulates up to
eight frames. A pack of ten critters doing ten ray queries each is a hundred
queries a frame, times eight. Each creature's document says what its
per-frame cost is dominated by.

## 5 · Build order

The order teaches the machinery one piece at a time, so each creature is mostly
reuse plus one new idea. Each is a few hours of AI time, as expected.

1. **P1 Species + P8 the harness per species.** Proves the promise in
   monsters.md. The Ridgeback comes out bit-identical.
2. **Gnawers.** Brings P3, critters and the pack brain. The first fight that is
   not one body.
3. **Hornback herd.** Reuses P3; adds herding and the charge into a solid.
   Brings P2, arenas as data, since the herd wants room.
4. **Mireback.** Brings P4, floor hazards, and the first creature that changes
   the arena.
5. **Sandmaw.** Brings P5, perception. Reuses P4 for the sinkhole.
6. **The Pair.** Two full monsters on the Ridgeback's skeleton, re-proportioned —
   the test that the rig really is data.
7. **Broodmother.** P1 + P3 + P4 together: a monster that makes critters.
8. **Veilstalker.** P5 again, on sight rather than sound.
9. **Mantis.** No new machinery; all new brain. The hardest one to make fair.
10. **Galewing.** Brings P6, flight and the sky ride.
11. **Siegeshell.** Everything, at scale, plus P7. Last because it is the one
    that will find every assumption the others did not.

The [world](world.md) is, for now, an arena picker in the dev harness (its W0),
and it arrives with P2, after the Hornback.

## 6 · What the ten documents found together

The creature documents were written in parallel, so some of what they decided
touches the others. Collected here, because each of these is one rule for the
whole cast rather than a detail of one fight.

### Changes the cast asks of `aim.rs`

[CLAUDE.md](../../CLAUDE.md) says a creature that breaks the aiming model changes
`aim.rs` rather than working around it. Five do. Each is a change true of every
ability at once, so each lands as **its own milestone**, after P1's
bit-identical step and never inside it, with the Ridgeback's twelve seeds and
`one_aim.rs` as the check.

| # | Asked by | The change | What it touches today |
| --- | --- | --- | --- |
| A1 **built** | [Gnawers](creatures/gnawers.md) §1a | `aim::stands_at`: the height of the last body the crosshair's ray passed through. A ground-aimed skillshot goes to *that* body's middle, and the standing swing's dead zone reads it. Bodies still do not stop the ray. | Nothing where only fighters stand: bit-identical |
| A2 **built** | [Hornback](creatures/hornback.md) §10 | Critters join what `aim::first_along` can run into | Nothing until critters exist |
| A3 **built** | [Siegeshell](creatures/siegeshell.md) §6 | Seen from above, the top face of a mountable part counts as a place, so aiming at your feet on a shell does not land on the floor below | **Shots at the Ridgeback's back change**: measured, its pins did not move, and of its twelve-seed reports only the Elementalist's and the Blood mage's landed counts did, by a hit or two |
| A4 | [Galewing](creatures/galewing.md) §6 | `swing_path` and `origin` measure the dead zone against the surface underfoot, so a swing on a banked back is level with the back | Nothing on the floor: bit-identical. The Ridgeback's shake gets it too |
| A5 **built** | [Veilstalker](creatures/veilstalker.md) §6 | `aim::in_view`: whether a point is inside a fighter's view, so it can never reveal itself off-screen | Nothing: a new question, not a changed answer |

One more is open rather than proposed: the [Broodmother](creatures/broodmother.md)'s
sacs are part of her body, so they are not on the crosshair's ray, and whether a
ranged shot at a sac lands depends on `first_along`. Her document leaves it to a
test.

### One fall-damage rule, not two

**Decided 2026-10-01** ([hazards.md](hazards.md) §4): a fall is measured from
**the last thing stood on**, lowered by any push up on the way down; free up
to **9 m**, **25 a metre** past it, halved for a landing slower than 12 m/s.
Not 7.5 m, because the Ridgeback's rear lifts a rider to 8.73 m and the pinned
hunts must not take fall damage; not the apex of a jump, because a fighter's
own jump is not a fall. Below, as the two documents had it.

The Galewing and the Siegeshell each wrote a fall-damage rule, and they
disagree: the Galewing's is free below 7.5 m and 30 per metre above, and the
Siegeshell's is free below 6 m and 18 per metre. Both key it to **height
fallen** rather than landing speed, which is right: a fighter reaches terminal
velocity after about four metres, so every fall above that lands at the same
speed. It is one rule for the whole game (P6), so it gets one pair of knobs,
settled when P6 is built. The Ridgeback's back (5.5 m) must stay free.

### The game has no sound

Three documents ran into it. The Pair add a glint at the screen edge while an
unseen cat winds up, standing in for a snarl. The Veilstalker never reveals
itself off-screen, because a strike from behind with no sound would be a hit
from nowhere. The Sandmaw is built on noise and draws it instead. Each is a
stand-in that should become a sound when there is audio, and until then
"unanswerable" counts a tell that was never on the victim's screen.

### Smaller things

- **Critter hit shape.** P3 planned a capsule; the Hornback wants a box that
  reaches the floor, so a level shot does not pass over a cow's back. **Decided
  2026-10-01: the box**, standing on the critter's feet and yawed with it --
  the Hornback's reason, at no cost to the gnawer. See
  [critters.md](critters.md) §1. The Gnawers' aiming problem (A1) is built
  beside it.
- **Fight length at tier 5.** The Mantis is written for five to eight minutes
  solo, below the tier table's eight to twenty. A duel that long is a slog; the
  tier table was written with the Siegeshell in mind. Probably the table is what
  moves.
- **Every document is longer than asked** — 570 to 680 lines against 250 to
  400. They are complete rather than padded. Each ends with its own open
  questions.

## 7 · Questions for you

**Answered 2026-09-30:** the cast is a good first mix; defence fights stay;
rewards are trophies and tempered rematches only, so the sidegrade in each
creature's §11 is a note for [parked.md](parked.md) rather than work; and the
world is separate arenas reached from the dev harness. See
[world.md](world.md) §0.

Still open, and cutting across the cast (each creature document ends with its
own):

1. **Tier 5 is built for two.** The Siegeshell and the Mantis are tuned so that
   a decent player alone mostly loses. Is that the right ceiling, or should every
   creature be soloable by a decent player?
2. **The Mantis reads habits.** It remembers what you hit its guard with and
   counters a repeat sooner. That is not reading inputs — it reacts to what it
   saw land — but it is the nearest thing in the cast to it, and the contract
   (§1.5) is load-bearing. Keep it or cut it?
3. **Riding the herd, riding the Galewing into the sky, climbing the
   Siegeshell** — three new rides. Is the ride the Ridgeback's thing, or the
   game's?

## 8 · Where the cast landed

*2026-10-01.* Ten creatures and the Ridgeback are built, and until today the
scripted hunter played two of the six classes: the Champion well and the
Bulwark less well. It pressed a poke, a heavy and a dodge, which is the
Champion's whole fight and a fraction of anybody else's -- so four classes lost
almost every hunt against every creature, and every creature's §13 said the
same sentence about it. **The hunter now plays all six**, and this section is
where each creature landed for each of them.

### How the hunter plays a class

A creature's plan (`crates/hunt/src/plans/<x>.rs`) still says *what* to do --
hit the foot now, go in on this window, get out, wait. A per-class layer,
`crates/hunt/src/class.rs` (`hunt::Hands`), says *how* this class does it.
Each plan calls it at the places it already had: where it swung (`hit`), where
it waited (`idle`), where it walked in on a window (`close_in`), where it
dodged (`leave`), and where a Bulwark could block (`guard`); the hunter runs
every frame's input through it last (`finish`). Which button throws which move
is learned by pressing them, as the sparring bot learns its kit
(`duel::Kit::learn`); aim is `aim::look_onto`, as the sparring bot's is. It
reads the creature only through the plan's fifteen-frame delay line, and its
own body -- its shadow, its pools, its bars, its shield -- as a player does;
where a plan keeps a camera (the Pair, the Veilstalker, the Mantis), the layer
acts only on what is on that screen and moves the mouse off it by a flick.

| Class | What it does now |
| --- | --- |
| Champion | **Exactly what the plan says** -- the plans were written for him, and the Ridgeback's pin proves his hunts did not move. |
| Bulwark | The plans' own guards where they had them (the Ridgeback, the Mireback, the Sandmaw, the Broodmother), and now the Hornback's hook and shoulder taken on the shield; a Slam answers a blow the shield took. A guard on the Pair's answers was tried and took him from 12 wins in 24 to 3 -- two cats, one shield -- and taken out. **The shield thrown** (2026-10-01, later the same day): on a window five to eleven metres off, it is thrown at the work, he leaps to it as it flies and Slams out of the leap; a shield left planted is recalled. The shield strikes the creatures and critters it meets (`CLASS-5`, resolved), but the leap comes first, so this is mostly his way across the floor and only now and then a ranged blow. |
| Shadow Reaver | Sends the shadow beside the work between openings, and calls it home when the work has moved; swings at nothing now and then so its copy marks the creature; **the lotus** on a shadow standing at the work, dragged home through it by a recall; **dashes** to a shadow standing there to go in, and to one lying the way out to leave; the **Executioner** to cash marks. |
| Elementalist | **Bolts and the Cataclysm aimed** through the crosshair onto the point (the plans had levelled them at the horizon), the **fire pillar** planted where the creature stands in any window or quiet moment long enough. A cover stone raised between her and the creature was tried and cost her hunts on every creature it was tried on (they block her own bolts and her way); not kept. The plans that raised stones already (the Hornback's lanes, the Sandmaw's beaches, the Pair's split) still do. |
| Blood mage | The scythe where the plan pressed her Haemorrhage in reach (four in a hundred of her red for thirty, against something that does not bleed), measured by the scythe's own reach -- several plans measured her by her poke slot, the Bloodletter, and swung the scythe at the air from seven metres. **The spike on a pool under it** worth the price (eighty of essence and up), **blinks** to a pool to go in or out, the **Grasp** as a way in on a long window, the cut from range with red to spare. |
| Dual mage | **Both hands**: every auto in the hand that keeps the bars inside the band; **never into ascension** (the first hunts that let her climb there lost seven hundred health to it on the Mireback); goads the bars up between openings; **Judgement and the Sweep** in a window; the **second jump** off the three-quarter tier when the plan holds jump for height; the punch turned so the line from that shoulder meets the point. |

The report's new section **THE CLASS** (and `the class, over the runs` under
`fight --repeats`) counts what each class did, so a zero says a tool was never
found a use for. `crates/hunt/tests/class.rs` holds that every class uses its
own kit against a creature, that the Champion's input comes through unchanged,
and that the Dual mage never ascends on a hunt; `tests/pin.rs` pins a Reaver,
an Elementalist and a Blood mage hunt beside the Champion's, the Bulwark's and
the Dual mage's.

### The numbers

`cargo run -p hunt --bin fight -- --species <x> --class <c> --repeats 24`,
solo; the Hornback's crossing `--arena crossing --repeats 12`; the Galewing's
plan B `--gamble`; the Broodmother `cargo run -p hunt --bin brood -- --all
--repeats 24` (the balanced plan here; the other two in her §13). *Before* is
the same command on main the hour before the class layer; for the Siegeshell,
which arrived later, the same build before its plan was wired to the layer. Alone and as a pair (`--hunters 2 --repeats 12`, both of the class).
**The Bulwark's rows were re-run on 2026-10-01** after his shield throw was
added to the layer; the guard-only build reproduced every Bulwark row above to
the hunt before it moved (see *Across the cast*, below, and the feel log).
**And again later the same day, when the thrown shield began to strike
creatures and critters** (`CLASS-5`, resolved): the rows above are that build's.

| Creature | Class | Won | Mean win | Unanswerable | Before (won) |
| --- | --- | --- | --- | --- | --- |
| Ridgeback | Champion | 14/24 | 64 s | 0 | 14/24 |
|  | Bulwark | 1/24 | 103 s | 0 | 0/24 |
|  | Reaver | 12/24 | 64 s | 0 | 5/24 |
|  | Elementalist | 6/24 | 141 s | 0 | 0/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Gnawers | Champion | 24/24 | 35 s | 0 | 24/24 |
|  | Bulwark | 24/24 | 35 s | 0 | 24/24 |
|  | Reaver | 24/24 | 33 s | 0 | 24/24 |
|  | Elementalist | 24/24 | 24 s | 0 | 24/24 |
|  | Blood mage | 21/24 | 57 s | 0 | 24/24 |
|  | Dual mage | 20/24 | 87 s | 0 | 2/24 |
| Hornback, meadow | Champion | 22/24 | 109 s | 0 | 22/24 |
|  | Bulwark | 20/24 | 154 s | 1 | 19/24 |
|  | Reaver | 15/24 | 163 s | 1 | 16/24 |
|  | Elementalist | 24/24 | 36 s | 0 | 23/24 |
|  | Blood mage | 21/24 | 109 s | 1 | 5/24 |
|  | Dual mage | 14/24 | 219 s | 1 | 1/24 |
| Hornback, crossing | Champion | 11/12 | 56 s | 0 | 11/12 |
|  | Bulwark | 8/12 | 59 s | 0 | 9/12 |
|  | Reaver | 9/12 | 56 s | 0 | 9/12 |
|  | Elementalist | 11/12 | 49 s | 0 | 7/12 |
|  | Blood mage | 4/12 | 58 s | 0 | 8/12 |
|  | Dual mage | 9/12 | 67 s | 0 | 11/12 |
| Mireback | Champion | 21/24 | 120 s | 1 | 21/24 |
|  | Bulwark | 18/24 | 102 s | 0 | 16/24 |
|  | Reaver | 22/24 | 89 s | 2 | 23/24 |
|  | Elementalist | 24/24 | 42 s | 0 | 24/24 |
|  | Blood mage | 5/24 | 149 s | 3 | 5/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Sandmaw | Champion | 18/24 | 191 s | 0 | 18/24 |
|  | Bulwark | 22/24 | 209 s | 0 | 24/24 |
|  | Reaver | 10/24 | 223 s | 0 | 8/24 |
|  | Elementalist | 11/24 | 338 s | 0 | 4/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 3/24 | 209 s | 0 | 0/24 |
| The Pair | Champion | 16/24 | 159 s | 0 | 16/24 |
|  | Bulwark | 23/24 | 163 s | 0 | 12/24 |
|  | Reaver | 7/24 | 111 s | 0 | 4/24 |
|  | Elementalist | 24/24 | 46 s | 0 | 4/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 4/24 | 128 s | 0 | 0/24 |
| Broodmother (balanced) | Champion | 7/24 | 143 s | 2 | 7/24 |
|  | Bulwark | 0/24 | -- | 0 | 0/24 |
|  | Reaver | 0/24 | -- | 0 | 0/24 |
|  | Elementalist | 1/24 | 142 s | 1 | 0/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 10/24 | 152 s | 3 | 0/24 |
| Veilstalker | Champion | 6/24 | 336 s | 0 | 6/24 |
|  | Bulwark | 9/24 | 483 s | 0 | 8/24 |
|  | Reaver | 6/24 | 491 s | 0 | 3/24 |
|  | Elementalist | 22/24 | 208 s | 0 | 5/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Mantis | Champion | 0/24 | -- | 0 | 0/24 |
|  | Bulwark | 18/24 | 246 s | 0 | 18/24 |
|  | Reaver | 0/24 | -- | 0 | 0/24 |
|  | Elementalist | 24/24 | 71 s | 0 | 0/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 2/24 | 212 s | 0 | 0/24 |
| Galewing, plan A | Champion | 4/24 | 482 s | 0 | 4/24 |
|  | Bulwark | 0/24 | -- | 0 | 0/24 |
|  | Reaver | 9/24 | 646 s | 0 | 0/24 |
|  | Elementalist | 17/24 | 176 s | 0 | 0/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Galewing, plan B | Champion | 7/24 | 491 s | 0 | 7/24 |
|  | Bulwark | 1/24 | 538 s | 0 | 2/24 |
|  | Reaver | 11/24 | 580 s | 0 | 2/24 |
|  | Elementalist | 18/24 | 230 s | 0 | 0/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Siegeshell, alone | Champion | 0/24 | -- | 1 | 0/24 |
|  | Bulwark | 0/24 | -- | 0 | 0/24 |
|  | Reaver | 3/24 | 265 s | 0 | 3/24 |
|  | Elementalist | 2/24 | 407 s | 0 | 1/24 |
|  | Blood mage | 0/24 | -- | 0 | 0/24 |
|  | Dual mage | 0/24 | -- | 0 | 0/24 |
| Siegeshell, two | Champion | 5/12 | 468 s | 5 | 5/12 |
|  | Bulwark | 0/12 | -- | 1 | 0/12 |
|  | Reaver | 12/12 | 223 s | 0 | 11/12 |
|  | Elementalist | 6/12 | 339 s | 0 | 7/12 |
|  | Blood mage | 0/12 | -- | 0 | 0/12 |
|  | Dual mage | 9/12 | 452 s | 0 | 1/12 |

The Mantis's ablations, every class 24 hunts (`MANTIS_PLAN=repeater|jumper|dodger`,
`MANTIS_HABIT=off`): the Elementalist wins 23-24 of each but the jumper's (1);
the Bulwark 11 (repeater), 2 (habit off), 1 (dodger), 0 (jumper); everybody
else 0-2. The Champion's are 0 of 48 for the plan and every ablation, on main
before this change as after it.

### Across the cast, by class

- **The Elementalist's pillar decides most fights.** 24 of 24 against the
  Gnawers (24 s), the Hornback (36 s), the Mireback (41 s), the Pair (46 s)
  and the Mantis (71 s); 22 the Veilstalker, 17-18 the Galewing, 11 the
  Sandmaw; 22 the Broodmother's mother-only plan against 1 for the balanced
  one. **No creature steps out of a fire pillar**, so its whole burn -- 390
  over the pillar's 175 -- lands on every one planted under it, from nine
  metres, out of reach of most of what it can do. Either creatures learn
  fire, or the pillar's burn on a creature comes down, or this is her
  identity (the Mireback's and the Veilstalker's §12 already ask): a person's
  call. Her worst are the Siegeshell (2 alone, 6 of 12 as a pair: a pillar
  fits under a walking leg almost never) and the Ridgeback (6), whose windows
  are short and whose ride she cannot work as well from its back. Main's aim
  A3 -- the top of a creature's part seen from above is a place -- cost her
  on both creatures she shoots from above or onto a back: the Sandmaw 20 to
  11, the Ridgeback 7 to 6, after the layer learned to send such a shot past
  the edge (`Hands::spot`; before it, 1 and 3).
- **The Blood mage loses wherever the Champion has to out-damage something.**
  21 against the Gnawers and the Hornback (whose stun holds the bull over a
  spike), 5 the Mireback, none anywhere else. Against a creature her kit is
  thin: the scythe is 22 (35 at full grey) against a sword's 64; the Grasp
  closes on a creature with one arm's worth -- forty or fifty, measured on the
  Pair, for seven in a hundred of her red; the Haemorrhage is thirty for four;
  and the spike pays only on a big pool, which only big hits make. Every
  number is the kit document's; "the creature does not bleed" is the sentence
  that costs her most. For [kits/blood-mage.md](kits/blood-mage.md) and a
  person.
- **The Dual mage lives on tempo.** 20 the Gnawers, 14 the Hornback, 10 the
  Broodmother, 2-4 the Pair, the Sandmaw and the Mantis, none the Ridgeback,
  the Mireback, the Veilstalker or the Galewing -- and 9 of 12 as a pair
  against the Siegeshell, from 1, her hands turned onto the ankle beside her. Where the plan swings often
  her bars climb and her hands keep them level; where windows are scarce they
  sit near empty and her punches are thin. Her punch also **passes over what
  is at her feet** -- the Ridgeback's ridge from where the plan stands her to
  work it, the Mireback's warts from its crown -- where a sword's arc does
  not: zero ridge hits in her Ridgeback hunts, though her hop puts her on its
  back more than anybody's.
- **The Reaver gained most where the shadow can stand by the work**: the
  Ridgeback 5 to 12, the Pair 4 to 7, the Galewing's two plans 0 and 2 to 9
  and 11, the Veilstalker 3 to 6, a pair against the Siegeshell 11 to 12. Her marks are cashed rarely (a herd's
  bodies carry none), and she is 750 health against everybody's 1000. The
  Mantis is still 0: her shadow out beside its guard is not yet the fight §7
  of its document describes.
- **The Bulwark** moved first only where the layer guards for him or answers
  a guarded blow with Slam: the Hornback (19 to 20, 191 s to 170 s), the
  Sandmaw (the same 24 wins, with more health), the Broodmother and the
  crossing (one fewer). **Then the throw** (2026-10-01, the rows above): the
  shield thrown at a window five to eleven metres off, the leap to it, and the
  Slam out of the leap. **The Pair 12 to 21**, in 161 s from 186 -- a leap
  closes on a cat in its recovery before its mate comes round -- the Mireback
  16 to 18, the Ridgeback 0 to 1, the Veilstalker 8 to 9, the Broodmother 0
  to 1; the Sandmaw 24 to 22 and the Hornback and the crossing one fewer
  each, where a throw given up for a dodge leaves him without his guard
  until it is home. The Gnawers, the Mantis and the Siegeshell never call for
  it (no plan there walks in on a window from range), and the Galewing's
  windows are too short for the leap and the Slam (0 and 2, as before).
  **Then the shield struck creatures** (`CLASS-5`, resolved later the same
  day): thrown, it strikes the first body it meets and plants there empty;
  recalled, each it passes through once. Same seeds, before and after: the
  Hornback 19 to 20 (163 s to 154 s) and the crossing 7 to 8 of 12 -- a throw
  at the work now meets a cow on the way and is a blow rather than a way
  across (29 throws, 8 leaps, against the Hornback) -- **the Pair 21 to 23**,
  the Broodmother 1 to 0 (sac pops 49 to 43), the Galewing's plan B 2 to 1;
  every other row the same, to the hunt. The leap usually comes before the
  shield reaches anything (it is six frames out, two metres), and the shield
  that turns to meet him passes through what lies between, so the throw is
  still mostly his way across. The Siegeshell's planted climb (a shield that
  plants *on* a part) is not built: it needs P1.
- **The Champion** did not move on any creature.

### The fire pillar, looked into

*2026-10-01, [plans/polish-fights.md](plans/polish-fights.md).* Asked of the
five creatures she beats 24 of 24: does any of them fail to answer the pillar
the way its own document says it should -- walk into it, ignore the burn, or
have no move for it? Measured over eight Elementalist hunts each, every frame
a pillar burned within reach of a creature.

**How the pillar lands.** Almost always on a creature already flinching from
her bolts: of the pillars that came up under the Mantis, the Pair, the
Veilstalker, two thirds or more found it in a flinch on their first frame. So
the first part of the burn is a window she earned, as the class layer means
it to be.

**What it does once it can move** is where the creatures differ, and only two
of the eleven documents give a creature anything to do about fire:

- **The Mireback** flees burning ground next to its tar (§5, `flee`), and
  does: a pillar under it lights its tar, and it flops away (its pillar
  frames are a flop's recovery and startup, 51 %, and a topple). Its fight is
  short because a pillar is a fuse in a floor of tar, which its §7 calls her
  identity. **As designed.**
- **The Veilstalker** sees fire and will not walk into it, and twenty frames
  in fire panic it (§4). A pillar under a flinching animal panics it: it
  retreats inside the base and is down by the thirtieth frame. §7 already
  names the lever -- "a slinking animal has walked out of the base by then"
  assumes it was slinking, not flinched by a bolt -- and leaves it to a person.
- **The Mantis, the Pair, the Hornback's bull and the Gnawers** have no rule
  about fire, and stand in it. Once its flinch is over the Mantis is still
  inside the base 85 % of the pillar's frames (prowling 31 %, its guard up
  against her bolts 29 %); a cat 57 % (prowling, through its pause between
  moves); the bull winds up its charge in it. Nothing in their documents says
  they should leave -- the Mantis's even says the pillar "comes from below the
  cone and lands" -- so **none of this is a creature failing its design**,
  and nothing was changed.

**So the pillar is simply strong against anything that does not know fire.**
The creature-side answer would be one rule for the cast -- *a creature that
can move does not stand in fire it has seen*, as the Veilstalker's already
does -- which would cut a pillar on a flinched body to the flinch and the
frames it takes to walk out; the class-side one is the pillar's burn on a
creature. Either is a decision about the Elementalist across every fight, and
it is the owner's.

### Unanswerable

**Zero for every creature since 2026-10-01** ([plans/polish-fights.md](plans/polish-fights.md)),
six classes, solo and coop: the Hornback's four were its bull's windups begun
off the hunter's screen, which every pack now lands only through a marker
under you for a reaction ([critters.md](critters.md) §2); the Mireback's were
its own measure counting tar that was not laid after the commit -- the crash's
own ring, or old tar a hunter stepped or floated into -- and it asks for tar
laid after the commit now. What follows is how it stood before.

Zero, as before, for every class on the Ridgeback, the Gnawers, the Sandmaw,
the Pair, the Veilstalker, the Mantis and the Galewing. **The Mireback's
moved**: its §13 recorded 1/1/1/0/0/0 by class; main measured 1/0/1/0/1/2
before the class layer -- after the Mantis made the shared rule ask whether the
hurt hunter was inside the move's reach, and after other merges -- and
1/0/2/0/3/0 after it. **The Hornback has its first**: one in 24 for the
Bulwark, the Reaver, the Blood mage and the Dual mage. The one traced, the
Blood mage's, was the bull's windup begun out of her sight while she walked to
the rock she posts at, her crosshair on the bull: the plan's walk, on a course
of the fight the Champion's hunts never take. The Broodmother's are where
they were for the Champion (2) and 0-3 for the rest. The Siegeshell's are
its parasites' (its §13): the Champion's 1 alone and 5 as a pair, unchanged,
the Bulwark's pair 1, and none for the four classes that keep moving.

### Open, by creature

- **The Ridgeback** -- the Blood mage, the Dual mage and the Bulwark win
  nothing; the Dual mage's punch misses the ridge from where the plan works it.
- **The Gnawers** -- nothing new; everyone wins, the Elementalist in 24 s.
- **The Hornback** -- the Elementalist's 36 s with nearly all her health is
  trivial; four classes take their first unanswerable hit, one each in 24.
  On the crossing the Blood mage fell from 8 to 4 of 12.
- **The Mireback** -- the Dual mage climbs onto it and cannot burst a wart from
  its crown (0 of 24); the Blood mage's unanswerable hits rose to three.
- **The Sandmaw** -- the Blood mage still wins nothing; the Elementalist now
  wins 11, slowly (338 s), and was 20 before aim A3 merged.
- **The Pair** -- the Elementalist wins every hunt in 46 s.
- **The Broodmother** -- the Elementalist wins by ignoring the brood (22 of 24
  mother only, 1 balanced): the dilemma does not hold for her.
- **The Veilstalker** -- §7's worry is confirmed: the Elementalist wins 22 of
  24.
- **The Mantis** -- the Elementalist wins every hunt from outside its guard;
  the Reaver, the Blood mage and the Champion win none solo.
- **The Galewing** -- the Elementalist and now the Reaver win; the Blood mage
  and the Dual mage nothing, the Bulwark next to nothing.
- **The Siegeshell** -- the Bulwark and the Blood mage win nothing alone or as
  a pair: nothing on it is a blow to guard, and her cuts are all of the Blood
  mage's kit that fits on a walking leg. A pair of Dual mages went from 1 to 9
  of 12; alone only the Reaver (3) and the Elementalist (2) win.

