---
status: standard -- the owner's, from play, 2026-11-02
---

# Level design -- the standard every place is built to

What a place has to be before anybody is asked to load it. Written after
the opening's first blockout ([opening.md](opening.md)) reached the owner
with a camera that jumped, walls cut away round the fighter, a cramped climb
and a creature that was only a picture -- each of which a check could have
caught first. The process that goes with these rules (stages, the play
check, the report) is [delivery.md](delivery.md); creatures are
[monster-design.md](monster-design.md).

**Every rule here holds unless otherwise instructed.** An instruction for a
particular piece of work wins over any of them; where one is set aside, the
report says which and why ([delivery.md](delivery.md), D6).

## The rules

**L1 · A run-up and freedom before a boss.** Unless otherwise instructed,
the way to a boss is not a corridor. Between leaving somewhere confined (a
cell, a cave, a gate) and the fight there is open ground: room to run,
look round and choose a way, with things to find off the straight line, and
time -- tens of seconds at a run, not a few jumps. Dark Souls' asylum yard,
Tears of the Kingdom's plateau: the player is let go before they are tested.
*Exit the cell, four cramped jumps, the boss room* is the failure this rule
exists for (the owner, 2026-11-02).

**L2 · The goal is seen before it is reached.** Unless otherwise
instructed, a player can see where they are going -- a landmark, the boss's
building, the thing on the horizon -- from early on, and the route keeps
showing it. Getting there is a choice of how, not a search for where.

**L3 · Room for the camera.** Unless otherwise instructed, any space a
fighter moves through gives the follow camera room to sit behind them and
swing round: as a starting guide, about eight metres across and five metres
to any ceiling. Tighter is allowed only where the camera check
([delivery.md](delivery.md), D3) passes in it.

**L4 · Walls that enclose are land or thick stone, and spaces are wide
enough that the camera seldom meets them.** Unless otherwise instructed, a
wall that bounds a space is the arena's rim or the valley's land
(`arena::rim`, as `CLAUDE.md` already requires of a fight's edge), or stone
thick enough to read as a wall. The camera keeps out of any box at least
the Oven's `Camera held off by a box this thick` and `... this broad` (1 m
and 4 m as of 2026-11-03): **it snaps its arm in when the view swings into
one, and lets it out again as fast once it is clear.** That is the camera's
designed answer and some of it is unavoidable beside anything solid; in a
narrow space with solid on every side it happens on every turn of the view,
which is what the first blockout's ten-metre well did and what a player
calls the camera jumping. A smaller box (a post, a thin ledge) is cut away
round the fighter instead. An opening in a wall is a gap in thick stone or
in the land.

*Corrected 2026-11-03: this rule first said 8 m and 20 m, which are the top
of those two knobs' ranges, not their values.*

**L5 · Every start is clear.** Unless otherwise instructed, every fighter's
start stands on something, inside nothing, apart from the other's, and
nowhere near a creature's body or under it. Held by
`crates/sim/tests/spawns.rs` for what the simulation knows; a drawn thing
the simulation does not know about is never placed over a start.

**L6 · Every class has its whole kit.** Unless otherwise instructed, no
place takes a class's tools away or is built on the assumption that it will
(the owner, 2026-11-02: "Every character has all their tools, we don't take
any of those away"). Movement is sized to the classes that move
(`sim::class::MOVERS`), never to the Bulwark, as `CLAUDE.md` says.

**L7 · Scale is measured against the whole kit.** Unless otherwise
instructed, a height, a gap or a creature's size is set against what the
five movers can do with everything they have -- the reach tables in
[courses.md](courses.md) and `cargo run -p sim --bin frametable` -- and
written down beside the number. With every tool in hand a ten-metre wall is
a step for some classes; a place that wants to be a climb is built for
that, or is not a climb.

**L8 · Mobility as a challenge follows the intent.** Unless otherwise
instructed, a movement section that gates progress costs little to fail and
shows how little before you try ([intent.md](intent.md) §3), and a hard one
is short and off the main way. A climb is not a hook: the owner's
observation is that people try the climbing challenges for a while and give
up.

**L9 · Nothing in a playable place is a picture of something.** Unless
otherwise instructed, everything a fighter can walk up to in a place they
can load behaves as what it looks like: what looks solid collides, what
looks like a creature is one ([monster-design.md](monster-design.md), M1).
A stand-in drawn for a screenshot lives in that screenshot set only.

**L10 · Judge it from the fighter's camera.** Unless otherwise instructed,
how a place looks is judged from the fight's own camera, along the route,
at the moments that matter. A camera placed by hand is for an establishing
view and is labelled so; it is never the evidence that a place works.
