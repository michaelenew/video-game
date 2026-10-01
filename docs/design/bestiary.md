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

Tar, web, smoke, burning ground, a sinkhole's pull, vented steam. A small fixed
list of circular floor areas — kind, centre, radius, age, state — about eight
bytes each, sixteen of them. The fighters' `effects` list is the precedent. The
list is what the renderer draws and what the hit test reads, which keeps the
rule that the overlay draws what the hit test uses.

### P5 · Perception beyond sight (Sandmaw, Veilstalker, The Pair)

The glance samples every target. Three creatures need it to sample less. The
Sandmaw hears movement, not bodies. The Veilstalker is *un*seen, which is a
renderer fact that the simulation has to own, because a decloak is a tell and
tells are gameplay. One of the Pair can be half-blinded. The change is small and
central: the glance takes a **perception filter** per species, and what it
cannot perceive it does not sample.

### P6 · Flight (Galewing), and the long ride (Galewing, Siegeshell)

A 3D steering controller alongside the ground one, and a ride whose surface is
tens of metres off the floor. Falling damage stops being an afterthought. This
is the largest single piece of new machinery in the cast, and only two
creatures use it, which is why they are late.

### P7 · Objectives (Hornback's cart variant, Siegeshell)

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
| A3 | [Siegeshell](creatures/siegeshell.md) §6 | Seen from above, the top face of a mountable part counts as a place, so aiming at your feet on a shell does not land on the floor below | **Shots at the Ridgeback's back change** |
| A4 | [Galewing](creatures/galewing.md) §6 | `swing_path` and `origin` measure the dead zone against the surface underfoot, so a swing on a banked back is level with the back | Nothing on the floor: bit-identical. The Ridgeback's shake gets it too |
| A5 | [Veilstalker](creatures/veilstalker.md) §6 | `aim::in_view`: whether a point is inside a fighter's view, so it can never reveal itself off-screen | Nothing: a new question, not a changed answer |

One more is open rather than proposed: the [Broodmother](creatures/broodmother.md)'s
sacs are part of her body, so they are not on the crosshair's ray, and whether a
ranged shot at a sac lands depends on `first_along`. Her document leaves it to a
test.

### One fall-damage rule, not two

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
