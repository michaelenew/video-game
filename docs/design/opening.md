---
status: plan (stage 1 of delivery.md) -- the first blockout is superseded (§7)
proposed: 2026-10-29
prototype: 2026-10-29
replanned: 2026-11-02
---

# The opening — a prison, a climb, and the first fight

How a new player meets the game: where they wake, the way out to the first
fight, and that fight. Written from a conversation
with the owner on 2026-10-29. It serves [intent](intent.md), and it is also
the first test of a **combat identity** (§2) that the intent leaves unsaid.

## 0 · Why this exists

The owner, on what is wrong with how the game starts now:

- *"I actually really dislike the current valley design as 'progression'. It's
  boring both visually and mechanically to progress through."* The things that
  hold people are the climbing challenges, and most people try those for a
  short while and then give up, so **a hard jump course is not the hook**.
- *"Walking out of a prison cell means something about your circumstance,
  appearing in a bland village center does not."* The model is the start of
  Dark Souls or Tears of the Kingdom: a place that tells you your situation.
- **There have been explicit requests to introduce the clicks one at a
  time.** A whole kit at once is too much to take in. *Introduce* is the
  word: the owner, 2026-11-02, *"Every character has all their tools, we
  don't take any of those away."* Clicks are taught one at a time, never
  locked (§3).

And on what the hook has to be. The owner's children's games
(`michaelenew/kids-games`) worked because the first minutes had one clear
goal, a win that was not too hard, and plain feedback that they were doing
well, followed by short bursts of learning, each paid off by something fun:
building the robot, getting a unicorn. In Mecha Forge the reward is **the
power of the upgrade**, and the rule is *the reward is not a medal screen,
it's watching the hit bounce off*.

This game borrows that rhythm but not its difficulty. The draw is Dark
Souls': *"the Asylum Demon isn't a pushover, it's a genuinely challenging
fight and quite scary the first time you encounter it ... you can beat this
thing, ... but the cards were stacked against you from the start and that's
abundantly clear the moment it walks in."* **The cards stacked against you,
but beatable, so it was fair.**

Two things are therefore wanted from the opening at once:

1. **The world reacts, materially, as you get through each challenge.** Things
   break, open and fall. A panel ticking a box is not the reward
   ([tutorials.md](tutorials.md)'s halls stay as reference, not as the hook).
2. **Room to breathe before the test.** The owner, 2026-11-02, on the first
   blockout: *"exit cell -> you have no room -> do 4 cramped jumps (some
   optional) -> enter boss area. That's not dark souls. It needs a run up,
   it needs some kind of freedom before you get into the fight."*

## 1 · What the opening is for

Finding out what is fun about this game. The owner: *"figuring out 'what's the
walk up to the first fight, and what does that fight look like' I think is
the process of figuring out what's fun about this game."* The Asylum Demon is
the model of a Dark Souls fight in one creature, *big guy smash hard, be good
to smash back*. This game has not found its equivalent yet, its **canonical
fight**, and the first fight is a good place to look for it.

## 2 · The combat identity this tests

What sets this game apart is **movement**: near-total freedom, very large
maneuvers, and a skill in adjusting on the fly and doing it consistently.
That is closer to Mario Odyssey or Sonic Adventure 2 Battle than to Dark
Souls. The fights do not use it yet. In the Ridgeback, the fight that uses
it most, movement is a phase (get up, then ride), not the fabric of the
fight ([monsters.md](monsters.md) §1).

The proposal, for the first fight:

> **The monster owns the floor. You own everything above it.**

- **Staying on the floor is the hard way to fight it.** Its big moves are
  aimed at the ground: sweeps, shockwaves, slams. They can be dodged on the
  floor, and a careful player should be able to win there, slowly.
- **Getting off the floor is how you stop taking hits.** As in a movement
  shooter, you avoid a blow by not being where it lands. This fits what is
  already decided: no cooldowns, *Smash, not Tekken* ([combat
  kernel](combat-kernel.md)).
- **Height is where the damage is.** The soft place opens only from above:
  a dive from a ledge, a perch, its own back. Moving well becomes hitting
  hard.
- **The arena changes under the fight.** It breaks what you stand on, and
  what it breaks becomes a new way up.

That is how a fight can be won by anybody and mastered through movement. A
Mario Odyssey boss can be beaten by jumping its shockwaves; a good player
never touches the floor.

**It is the rule for this fight, not for every fight.** The owner expects
other fights to work this way too, and some not to. That is fine.

The Asylum Demon already makes the point. The fight a player wins **starts
with a dive from a ledge** that takes a large share of its health: where you
stand decides the fight. That is the moment this opening is built around.

## 3 · The kit: whole, and taught a click at a time

**Every class has its whole kit from the first frame** (the owner,
2026-11-02; [level-design.md](level-design.md), L6). The first draft of this
document locked the clicks and the class movement and handed them out as
rewards; that is withdrawn, and so is everything that leaned on it.

What stays is the request that prompted it: the clicks are **introduced**
one at a time. On the way to the fight (§5) there are a few small
encounters, each one plainly suited to one click, with a line on screen
naming it -- something to hit up close for the left click, something out of
reach for a thrown or ranged one, something above you for a takeoff. Each
is optional and over in seconds, and nothing is locked if you skip it.

## 4 · Scale, against the whole kit

With every tool in hand the movers reach a long way: the stone launch
17-25 m ([courses.md](courses.md) §1a), the Reaver 50-75 m through the Whirl
(courses.md §0), a run and jump 7-9 m level. **No creature can be built big
enough that its back is out of reach**, and the first draft's case for a
creature "about two Ridgebacks" tall (that the shared moves alone could not
reach its back) no longer holds.

So the fight is not hard because the soft place is far up. It is hard
because of what the Warden does to you on the way and while you are there
(§6): the floor it owns, the perches it swats, the back it shakes. Its size
is chosen for how it looks and for how much room it gives a fight on its
back, and written beside the reach tables
([level-design.md](level-design.md), L7; [monster-design.md](monster-design.md), M5).

**As built (2026-11-03): 1.5 times the Ridgeback**, its back a little over
eight metres up (`cargo run -p sim --bin beastcheck -- --species warden`):
the shoulders 8.4 m, the nape 8.2-9.1 m and the ridge 8.3-9.3 m, out of
reach from the floor for every class; stumbled, its shoulders come down to
4.0 m, a standing jump for four classes. Fifteen metres was tried first,
and at 2.7 times the Ridgeback the body is thirty-five metres long; at 1.8
it filled most of a keep fifty across; 1.5 fits the keep the place had room
for (§5, as built). The keep's heights go far past it -- piers at 6 and
10 m, columns at 16 and 22 m, a spiral of ledges to the rampart at 29 m --
so from the top it is small below you: the floor is its, the heights are
yours.

## 5 · The plan: the way to the tower

**Stage 1 of [delivery.md](delivery.md)** -- a plan, with distances, for the
owner to react to before anything is built. The map is
[gallery/opening/plan.svg](gallery/opening/plan.svg).

Distances are along the direct way, at a run (about 7 m/s); exploring makes
all of it longer.

| | Where | From the cell | What it is for |
| --- | --- | --- | --- |
| 1 | **The cell** | 0 m | Your circumstance. The door is already torn off its hinges -- the Warden's work, before you have seen it. |
| 2 | **The cut** | 0-30 m | A passage out through the rock, ten metres wide and open overhead, rising gently to daylight. Room for the camera (L3); walls of rock (L4). |
| 3 | **The yard** | 30-180 m | **The run-up and the freedom** (L1). A broad, ruined terrace on the mountainside, 150 m long and about 100 m wide: low broken walls, a fallen statue, a lookout over the gorge, a collapsed watchtower to climb for the view. Mountain rising on one side, the gorge on the other. The click encounters (§3) are here, each off the straight line. |
| 4 | **The sight of it** | from 30 m | **The tower across the gorge**, sixty metres tall, from the moment you step out of the cut, and the whole way after (L2). |
| 5 | **The causeway** | 180-260 m | A stone causeway across the gorge, twelve metres wide, rising gently to the tower. Halfway, the Warden is seen through a breach in the tower's wall, and heard. Dread on a wide, safe road. |
| 6 | **The keep's rim** | 260 m | You come in **at the top**: a broad rampart round the fight, fifteen to eighteen metres up, and the Warden below. The fight starts with a dive off it (§2, the Asylum Demon's opening) -- or with dropping down any way you like. |
| 7 | **The fight** | -- | §6, in a round floor fifty metres across, walled by the rock the keep is cut into (L4). |

About forty seconds straight there; minutes if you look round, which is the
point.

**As built (2026-11-03, `--hunt warden`, the Esc menu's Prototypes row;
`sim::arena::warden`).** What changed from the plan above, and why:

- **It is a floating island** (the owner, 2026-11-03: "instead of being
  bounded by mountains, it's a floating island ... a little more interesting
  with some curves"). Six round masses of rock overlap, their tops thirty
  metres over the floor far below and narrowing beneath, so the edge
  curves; past it is the drop. **A fall off it is not a death**: you are
  stood on the nearest of six safe spots on the top, 120 health the
  poorer (a stomp's worth), as a jump course stands you on its last gate
  (`species::warden::stand_back`, L8).
- **It is half the plan's length.** The whole place is 157 by 65 metres --
  cell and cut 26 m, yard 56, the gap and causeway 18, keep 56 -- about
  twenty-five seconds at a run. An arena fought alone keeps its diagonal
  under 170 m ([fight-spaces.md](fight-spaces.md), F8): past about 181 m a
  squared distance overflows fixed point. The plan's yard is the valley's
  map's to give (OPENING-10).
- **Things in the yard that move you and turn your view** (the owner,
  2026-11-03: "walls, buildings that actually make the user take a different
  path and change their view"):
  - **The curtain wall**, seven metres tall across the yard, whose one gap
    (six metres) is at its north end against the island's edge: the way
    bends to the edge, and through the gap the keep stands framed.
  - **The chapel**, roofless, its west and east doors at different places:
    go through it and the keep is hidden until you come out of the east
    door; go round it and you are on the island's southern edge.
  - The arch's posts and a fallen statue, from before.
- **The gap and the causeway**: between the yard's lobes and the keep's
  island the drop shows through, and the causeway bridges it.
- **The keep is twenty-nine metres tall**, not sixty (the owner,
  2026-11-03: "much higher walls and be more climbable ... without class
  abilities, and it needs to give the scale and verticality of the game"):
  eight walls of stone six metres thick round a floor forty across.
  - **Inside, a spiral**: a ledge four and a half metres deep on every
    face, each a held jump (3.25 m; the shortest held jump is 3.9) above
    the one before it round the keep, meeting it at the corner, from the
    floor to the top -- climbed with the moves every class shares, no
    class tool (`tests/opening.rs`, every mover). Four piers (6 and 10 m)
    and two columns (16 and 22 m) are the other perches.
  - **Outside, the climb** (the owner, 2026-11-03: start the fight from
    high up only through "a second agility section ... It should teach
    that the mobility sections offer an extra reward even when optional",
    and "a small section where missing could mean they fall off the
    island"): from the keep island's lip by its south-west wall, **four
    pillars rising out of the drop**, each a held jump above the last --
    miss one and you are off the island -- then seven ledges up the south
    wall's outside face, over the lip, and round its corner to the
    rampart. Optional: the gate is the other way in.
  - **The reward**: woken by a blow while a hunter is more than six metres
    up -- from the rampart, or diving off it -- the Warden goes straight
    over, and the fight opens with it toppled (`species::warden`,
    `HIGH`). Walked in through the gate, it simply wakes.
- **It runs as a hunt**, so a fighter nobody plays is out of the world (no
  second body standing in the cell) and the two of you cannot hurt each
  other. Lose and you start in the cell again.
- **No click encounters yet** (§3, OPENING-6): there is nothing in the yard
  to hit.

**How it would be built** (stage 2, after the owner has seen this): one
arena whose ground is a height function, as the King of the Hill maps'
hills are, with a rim for its edges and few boxes -- the rock, the gorge and
the terrace are land, not stacked boxes ([level-design.md](level-design.md),
L4). In two slices, each checked and handed over on its own
([delivery.md](delivery.md), D2):

1. **The way there**: cell, cut, yard, causeway and keep, with no creature.
   Walked end to end by the five movers, the camera check along it, the
   starts clear (D3).
2. **The Warden**, as a creature that is whole by
   [monster-design.md](monster-design.md) M1 -- body, hit parts, moves with
   telegraphs, the hunter's report -- in the keep.

## 6 · The first fight

**The arena.** The keep's floor: round, forty metres across (fifty in the
plan), walled by stone. Around it, things to get up on: **thick stone
piers** below the Warden's back and above it, and **high ledges** above its
back for the ranged classes -- as built, piers at 6 and 10 m, columns at
16 and 22 m, the spiral's ledges every 3.25 m up the walls, and the
rampart at 29 m.

**What it does.** It owns the floor. Its moves are aimed at the ground, so
each has a floor answer (a dodge, a step out) and a better answer up off it:

- **A sweep** across the floor at a height you jump.
- **A slam that throws a shockwave** out along the floor, jumped or dodged
  (Mario Odyssey's teaching move).
- **A move that takes the floor away** for a while: a quake that cracks it,
  something that pools on it. Built for this fight: the game's floor
  hazards today are things to step round on the ground, not reasons to
  leave it.

**What it does to you up high.** Being up cannot be free, or the fight is
over at the first perch. **It swats what you are standing on.** A pier
struck breaks. That tells you to move on, and **the arena changes**: every
perch it breaks is one fewer, and the rubble is a new way up, often onto
it. On its back it bucks and shakes.

**Bringing it down: the back, not the feet** (the owner, 2026-11-02).

- **Its feet take reduced damage.** Hitting them is the floor's way and the
  slow one.
- **Its back takes real damage**, and that damage fills a **stagger**. When
  the stagger fills it staggers and **falls**: whoever is on its back comes
  down with it (and lands without fall damage), and with it down there is
  a window for more hits, for everybody, from the floor.

**For the ranged classes** (proposed, OPENING-2): the soft place on its back
is open **only from above** -- the plates and shoulders cover it from below
and from the side, by their shape, so the aim model's straight line from
the hand meets armour first from the floor (`aim::first_along`), and no rule
for ranged attacks is needed. Melee classes reach it on its back; ranged
ones from the high ledges and piers, where it swats them. Back damage
fills the stagger whoever deals it.

**The contract.** The [bestiary](bestiary.md)'s, unchanged: every threat
announced before it lands, every opening real, a loss the player's and never
the creature's. *Tough*, in the intent's sense: a fight you do not win the
first time and can see yourself winning.

**The Warden as built (2026-11-03, `sim::species::warden`)**, against
[monster-design.md](monster-design.md):

- **Its body is the Ridgeback's**: skeleton, parts and their flags, legs,
  clips and baked poses are read from the Ridgeback's tables. That is
  plumbing -- a back to stand on, soft places on top that fill its poise,
  feet that break, a topple -- and also its motion, so **it moves like a
  big Ridgeback**. That sets aside the 2026-10-29 instruction that no
  existing creature be adapted, on the 2026-11-03 one to build as much as
  possible; its own moves (the shockwave, a floor taken away, the swat at a
  perch) need clips authored, and are the next work.
- **Its own**: its size (1.5x), its hide (feet 0.25x, upper legs 0.3x,
  ridge 2.4x, nape 3x; the rest as armoured as the Ridgeback's), 9000
  health, a topple of five seconds, and its choice of moves -- the stomp,
  the sweep and the slam, which go for the floor, and the kick and the
  shake; it never chooses the bite, the charge or the spray. **The slam and
  the sweep reach only 1.2 and 1.5 m off the floor**: they are jumped, not
  run from. The stomp's windup is 24 frames, so it can be reacted to.
- **It sleeps** until a hunter comes into the keep or hits it
  (`warden::frame`); a hunter crossing the yard finds it where it was
  (`tests/opening.rs`).

**The hunter's report** (`cargo run -p hunt --bin fight -- --species
warden --repeats 6`): the Ridgeback's hunter, walked along the way and
taught to jump the slam as it jumps the sweep, **wins three of six**, in
57 to 126 seconds in the keep, each win by getting onto its back -- one to
ten rides, four topples -- and loses three from the floor. Landed of
thrown: stomp 22/53, sweep 8/43, slam 0/97, kick 0/45; nothing
unanswerable. Before the island the same hunter lost all six from the
floor: the Warden had been sinking into its floor, a body that climbs what
it meets taking the ground for something to climb. It **keeps its own
height** now (`FightDecl::keeps_height`), held at the island's top by its
hook. How the fight goes from the rampart and the spiral is for a person to
find.

## 7 · The first blockout, and why it is superseded

**Nothing in the game today is a starting point for the substance.** The
owner, 2026-10-29: *"Nothing in the game today has the right substance that I
am looking for. Every fight and arena keeps you on the ground almost
entirely."* No existing arena, creature, move set or hazard is to be adapted
into this fight. What can be used is plumbing: a creature is a table
([species.md](species.md)), an arena is a table ([arenas.md](arenas.md)),
`tutorial::Seen` reads what a player just did, and the scripted hunter
measures a fight ([bestiary.md](bestiary.md) §8).

**The first blockout** (`--arena warden`, 2026-10-29, still in the game for
reference) was a tower: a cell at the bottom of a well, four tight ledges up
it, a gallery into a shaft, and the Warden drawn as boxes nothing collides
with. From play (the owner, 2026-11-02) and the checks written since, it
breaks these standards:

- **L1**: no run-up and no freedom -- cell, cramped jumps, boss room.
- **L3, L4**: a well ten metres across with solid on every side, so the
  camera snapped its arm in and out against the walls on every turn of the
  view, and window and door pieces too small to hold it were cut away round
  the fighter. *Corrected 2026-11-03*: this said the camera passed through
  the two-metre walls, reading the top of the camera knobs' ranges (8 m,
  20 m) as their values (1 m, 4 m); it holds off a two-metre wall. What
  thickening the outer walls changed was how far the arm could swing
  before meeting one.
- **L9, M1**: the Warden was a picture in a place you could load, so the
  owner found a creature with no collision, no hit parts and no moves.
- **L6**: it assumed the kit would be locked.
- **L10**: it was judged from cameras placed by hand
  ([gallery/opening/](gallery/opening/README.md)), which hid all of the
  above.

It was replaced on 2026-11-03 by the place in §5, under the same
`--arena warden`; its pictures stay in
[gallery/opening/first-blockout/](gallery/opening/first-blockout/README.md).

## 8 · Open

Answer by ID.

- **OPENING-1: is "the monster owns the floor" what people enjoy in the
  movement?** It was agreed for the intro fight. Whether the pull of the
  movement is that, or speed, flow, or style, the fight should show.
- **OPENING-2: ranged classes and the back.** Proposed in §6: the soft
  place open only from above, by the shape of its plates; ranged classes
  shoot down from high ledges and piers; ranged back damage fills the
  stagger as melee's does. Is that the way, and should the Elementalist's
  own stones count as perches?
- **OPENING-4: the Warden's size.** 1.5 times the Ridgeback, its back eight
  metres up (§4): bigger, and it fills its keep.
- **OPENING-5: skipping the opening.** A returning player, a second player
  joining, someone who has played it on another machine. And how the opening
  is played by two people at once.
- **OPENING-6: how the clicks are introduced.** The plan is small optional
  encounters in the yard, one suited to each click, with a line naming it.
- **OPENING-7: what is outside the tower, and what the valley becomes.** The
  owner does not want the valley as progression. What replaces it is not
  decided here; this document only covers the start.
- **OPENING-8: in from above, or a first meeting on its floor?** The plan
  enters at the keep's rim and opens with the dive. The Asylum Demon's way
  is a first meeting on the floor you flee, then a return from above; that
  is a longer loop and wants a reason to flee that is not a locked door.
- **OPENING-9: what is found off the line in the yard.** There is no reward
  system to hand anything out. A view, a glimpse of the Warden, a hint of
  the story are the candidates until there is.
- **OPENING-10: the run-up is half the plan's.** One arena holds 170 m of
  diagonal (§5, as built), so the yard is 56 m, not 150. The plan's length
  needs the opening on the valley's map, which keeps each place's
  coordinates local. Worth it, or is twenty seconds enough?
- **OPENING-11: the stomp.** It lands on the hunter a third of the times
  it is thrown, with a 24-frame windup. Slower, narrower, or the hunter's
  problem?
- **OPENING-12: its own moves.** The shockwave, a floor taken away and the
  swat at a perch (§6) need clips of its own; until then it moves like a big
  Ridgeback.
- **OPENING-13: the camera on the outside climb.** It fails one bar of the
  camera check: turning round on the wall's ledges, the arm pulls
  in against the wall on 1.5% of frames against a bar of 1%. Close to what
  any climb up a wall face does; accept it for climbs, or give the ledges
  more room?

