---
status: exploration
started: 2026-10-08
---

# 0006 — The valley and the haven

*A wider world to explore with a friend, with natural climbing that forces
the treacherous movement, navigable end to end in a few minutes. What would
it be, built out of what exists, and what is the first thing to build?*

## What is asked, as three numbers

- **Wider.** Today a session is one arena, 28 to 300 m across, chosen from a
  picker. The ask is a world you *move through*: places that lead to places.
- **Climbing that forces treacherous movement.** Not jump courses hanging in
  the air ([courses.md](../courses.md)): ground that happens to be steep,
  where the route up is read off the rock, a fall costs something, and the
  class mechanics are the shortcuts rather than the route.
- **End to end in a few minutes.** Call it **three minutes for a runner who
  knows it, five for a pair who does not**. At the ground speed the Oven
  gives (7 m/s) three minutes is 1,260 m of running; climbing is slower than
  running by about three to one (a ledge a second, two to three metres each),
  so a valley that is *a third climb* is about **900 m of route with 150 m
  of rise**. That is five or six places of the size the arenas already are.

The **haven** is Hearth, which [world.md](../world.md) §2 already names: the
walled town at the valley's mouth, with the dummy, the sparring bot and
versus in it. Nothing below renames it; "haven" is what it is for.

## What exists to build it from

Everything in this note is assembled from things that are already data.

| Thing | What it gives the valley | Where |
| --- | --- | --- |
| An arena is a table: bounds, a floor of materials, up to 64 boxes, spawn marks | A place is an arena. Cliffs, ledges, scree and chimneys are boxes; a trail is an arena with no creature | [arenas.md](../arenas.md) §1 |
| Relief: hills and hollows the simulation stands on | The ground between the rock is not flat; a slope reads as a slope | [forms.md](../forms.md) §"Relief" |
| The trip byte: one byte of input that moves both peers to another arena on the same frame | Walking out of a place and into the next is a trip, predicted and rolled back like a button, so two players change place together | `sim::input::Travel`, [arenas.md](../arenas.md) §3 |
| Courses: a route of gates with checkpoints, a pit, a fall that stands you back, a clock | The spine of the valley is a course whose islands are the ground | `sim::course`, [courses.md](../courses.md) |
| The fall rule: free to 9 m, 25 a metre past it, halved if landed slowly | A cliff is dangerous by its height and nothing else has to be written | [hazards.md](../hazards.md) §4 |
| Trophies and waystones | What opens the next tier, kept per player, off the snapshot | [world.md](../world.md) §3, §6 |
| Replays and the harness | A pair's run through the valley, judged offline: time, falls, where they stopped | [replays.md](../replays.md) |
| The jump: 7–9 m level off a run, a full hop 1.5 to 3.3 body heights, one airdodge | The grammar every gate is written in | [controls.md](../controls.md) §"The jump" |

What does **not** exist and is worth noticing before the shape is chosen:
**the sim has no moving solids, no ropes or vines, and no "edge grab".** A
body on a ledge is a body whose feet are on a box's top; it got there by a
jump that cleared the top or it did not. That is the whole climbing model,
and it is a good one for *reading* a route — the rock tells you exactly
what the jump has to be — as long as the rock is honest about it (below).

## The thesis: one long place, in loaded pieces

There are two ways to make a world out of arenas, and world.md §2 chose one
without saying why: a **graph** of places with trails between them, each a
room. The alternative is a **line**: the valley is one long piece of ground
from the haven's gate to the far end, and the fight places are *in* it
rather than *off* it — the meadow is the valley floor where the herd happens
to graze; the Commons is the bank you have to climb past the den to get
over. The line is what makes "end to end in a few minutes" a sentence that
means something: there is an end, and you can run to it.

The line still loads in pieces, because a `World` stays under 4 KiB and an
arena stops at 64 boxes. A piece is a **reach** of the valley (a bend, a
rise, a shelf), 150 to 240 m long, the size the Siegeshell's valley already
is. The seam between two reaches is placed where the ground hides it anyway
— a notch, a cave mouth, a bend behind a buttress — and crossing it is a
trip. Nothing about that is new; what is new is that the trip is fired by
**walking into the seam** rather than by a key, and that the next reach's
spawn marks are at the seam's other side, facing on.

**The two of you cross together.** The trip byte already puts both peers in
the new place on the same frame; the question is *when it is sent*. Rule:
the seam fires when **both fighters are in it**, or one is in it and the
other has fallen (stood on a checkpoint, as a course does) — never when one
is still climbing behind. A friend who is stuck is the reason you are there.

## Natural climbing, as a vocabulary

The courses taught the jump its grammar with floating islands. The valley
spells the same words in rock. Each is boxes and relief, honest about what
the jump has to be, and each has a **tell** you can read from below.

| Word | What it is | The jump it asks | Tell |
| --- | --- | --- | --- |
| **Scree** | A slope of risers under a metre, a stride apart, with relief under them | None: run up it | Loose stone, pale, wide |
| **Shelf** | A ledge 2 to 3 m up, a body wide, running along a face | A full hop, held | A dark lip with grass on it |
| **Stair** | Shelves stepping up a face, each one a hop from the last | Hops in a rhythm; the fall is to the shelf below, free | The lips in a diagonal |
| **Traverse** | A shelf along a face with the drop beside it, 10 to 20 m to the floor | Running along a half-metre ledge; the fall rule bites past 9 m | The drop, nothing else |
| **Gap** | A break in a shelf, 5 to 9 m, with air under it | A run and a jump; a held jump or the airdodge at the long ones | The far lip's grass |
| **Chimney** | Two faces a few metres apart, shelves alternating up them | A hop across and up, then across and up again: a jump and an airdodge, timed | A slot of sky |
| **Bluff** | A face 5 m and more, no shelf | Not a jump: a **mechanic** (a stone, a shadow, a Grasp, a vault) — or the long way round, which always exists | A smooth face |
| **Cairn** | Snow on a top, as on a course | A checkpoint: a fall stands you here | A pile of stones |

Three rules make it treacherous rather than tedious:

1. **The route is never the only route, but the only route for everyone is
   the slow one.** A bluff has a mechanic over it and a stair round it; the
   stair takes thirty seconds, the mechanic three. That is "paid for in
   execution, never in waiting" ([courses.md](../courses.md) §0) applied to
   a map.
2. **Height is the cost.** A traverse at 6 m is free to fall from and tedious
   to climb back; at 12 m it costs 75 and the same climb. The fall rule does
   the arithmetic; the author only chooses the height.
3. **A fall is a checkpoint, not a death**, exactly as on a course: you are
   stood on the last cairn with what you had out taken back. In coop the
   seam waits for you.

Two props from [courses.md](../courses.md) §4 would widen the vocabulary a
lot and are small: **vines** (a face anybody can climb, slowly: the Bulwark,
the Blood mage and the Champion have no answer to a 9 m rise, and a vine is
their answer at a time cost) and **updraft vents** (an Elementalist's
Updraft for everyone, placed by the author). Neither is needed for the first
reach. The third, drifting rocks, needs moving solids and stays parked.

## The valley, reach by reach

World.md's map, bent into a line, climbing the whole way. Rise is cumulative.

| # | Reach | Length | Rise | What it is | Fight in it |
| --- | --- | --- | --- | --- | --- |
| 0 | **Hearth** (the haven) | 60 m | 0 | The walled town: the dummy, the bot, the armoury, a gate in the far wall and a lookout over the whole valley from the wall's walk | versus, sparring |
| 1 | **The Mouth** | 180 m | 20 m | Out of the gate, a river terrace, the ford, then the valley narrows: scree up to the first shelf, a traverse above the river, the first cairn | the Hornback herd grazes the terrace: the low meadow is this reach's floor |
| 2 | **The Commons** | 150 m | 30 m | The bank the den is dug into is the climb: a stair of shelves past the den's mouth, a gap over the trail, the waystone of tier 1 on the bank's top | the Gnawers, who come out when you climb past |
| 3 | **The Shelves** | 200 m | 40 m | A long traverse along one face with two gaps and a chimney to the next level; the Mire in the hollow below it (a side way down, a slow way back up); the Dunes beyond a bluff | the Mireback (below), the Sandmaw (beyond the bluff): tier 2's "1 of 2" |
| 4 | **The Pinewood** | 180 m | 30 m | Trunks as pillars (the Ashwood's already are), a chimney between two of them, the Hollows under a root cave; the Highlands up a stair | the Pair, the Broodmother, the Ridgeback: tier 3 |
| 5 | **The Saddle** | 150 m | 40 m | The one open traverse, 20 m up, wind: the Cliffs on one side, the Ashfield on the other; the Shrine at the top of the last chimney | the Galewing, the Veilstalker, the Mantis: tiers 4 and 5 |
| — | **The Long Valley** | 300 m | −160 m | Runs *back* down to Hearth's wall from the far end. It exists already (`arena/siegeshell.rs`); the Siegeshell walks it | the Siegeshell, the ending |

Six reaches, ~920 m, 160 m of rise: three minutes for a runner, which was the
ask. The fights are *in* the reaches, looked into from the route and started
by walking in, as world.md §2 wanted; the route past each is the climb, so a
pair can run the valley end to end **without fighting anything once the
waystones are lit**, and before they are, the waystone on the bank's top is
the first wall they meet.

Two things change from world.md's map, and both are the line's doing:
Hearth is at the bottom of the climb *and* at the end of the Long Valley, so
the world is a loop and the ending walks home; and the Hornback herd is in
the first reach rather than beside the Commons, because a terrace with a
ford on it is the valley's floor and the Commons' bank is its first wall.

## What a reach costs, and the caps

A reach is one arena. The caps that bind:

- **64 boxes.** A stair of twelve shelves, a traverse with two gaps, a
  chimney of eight, the walls that close the sides, the seam's buttress:
  about forty. Relief carries the ground for free. The Shelves reach, with
  its two fights' own solids (the Mire's and the Pan's tables are 13 and 19
  boxes), would not fit *with the fights in it*; so a fight place is still
  its own arena, entered through a seam off the reach, exactly as world.md
  §2 has it. The reach is the trail; the fight is the room off it.
- **4 KiB.** A reach with no creature has the lore region free; a course's
  `Run` is 12 bytes a fighter. Nothing new goes in the snapshot except
  *which reach* (the arena byte, already there) and the seam state (a byte).
- **The budget test** measures the range. The Siegeshell's valley at 300 m
  is already its biggest scenario; a reach is smaller.
- **64 arenas** fit the trip byte's six bits. Ten fights, nine courses, the
  proving ground, the range, the bench and the lab are twenty-three; six
  reaches and Hearth make thirty.

## What to build first, and how to know it worked

**V1: the Mouth.** One reach, Hearth's gate to the Commons' waystone, with
the herd's meadow as its floor. It is the shortest path to every new thing
at once: a seam that fires by walking, a reach that is ground rather than
islands, a fight *in* a trail, a climb spelt in scree, a shelf, a traverse
and one gap, and a cairn. Built as an arena table plus a course table (the
route and its cairns), plus one new rule in `World::advance`: a seam zone
in the arena, and the trip it sends when both are in it.

Then **play it, two of you, recording**: `Y` saves the replay, and
`cargo run -p hunt --bin replay` already reports a hunt. Teach it a reach:
the clock from the gate to the waystone, the falls and where, the gates in
order, and whether the pair arrived together. That is the measurement; the
courses were measured by search and the owner said that was the wrong
instrument. A pair's replay is the right one.

Then the **vine**, if the Mouth shows a class with no way up a bluff but the
stair, and the second reach.

## Open

1. **Does a fight place pause the valley?** Walking into the meadow starts
   the herd's hunt; the route past it is the climb. Can you climb past a
   hunt you have not finished, or does walking in commit you? World.md's
   "the creature's moment of noticing you covers the transition" suggests
   the hunt starts and the climb is the way out.
2. **The seam when one has fallen.** "One in the seam, the other on a
   cairn" fires the trip and leaves the fallen one's climb undone. Right
   for a pair who has given up on it; wrong for one who wanted to try
   again. A short hold (three seconds in the seam) would let them say.
3. **Hearth's lookout.** Seeing the whole valley from the wall needs the
   whole valley drawn at once, which no reach can afford as geometry. A
   painted sky — the far reaches as silhouettes in the fog gradient,
   derived from the reaches' own heights — is the look crate's kind of
   answer, and cheap.
4. **Where the waystones' state lives** in coop: trophies are per player;
   a seam that one of the pair has not earned is open to the pair (world.md
   §5). So a seam asks the *more* travelled of the two.
