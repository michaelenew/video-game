---
status: built 2026-10-08
proposed: 2026-10-08
built: 2026-10-08
---

# Replays — a fight somebody played, judged like the bot's

Every fight in this game was tuned by a scripted hunter with a quarter-second
reaction, and every kit, creature and course is marked *unplayed*. The thing
standing between a person's play and the harness was that nothing a person
did in the game was kept. This closes that gap.

## 1 · What a replay is

The simulation is a pure function of `(state, inputs)`, so a fight is
completely described by **where it started and what was pressed**. A replay is
exactly that: a [`Start`](../../crates/sim/src/replay.rs) — both classes, the
creature, its temper, the arena, how many seats are played — and one line per
frame of the two fighters' inputs. Nothing about the fight is recorded that
the simulation does not need to rebuild it, because the simulation rebuilds
it: a replay is played by constructing the start world and calling `advance`
once per line.

It is **text**: a header, then one line per frame, eight hex words (bits,
aim, pitch and travel for each fighter), `xN` where a line repeats. A
ten-minute hunt is about a megabyte; a minute of versus a hundred kilobytes.
Text so that it can be read, diffed, pasted and committed.

```text
replay 1
build 921711c
tuning 3f1c…
start p1=champion p2=bulwark hunt=ridgeback temper=0 seats=1
world 0 9a2c…
0100 3f2a fff0 00 0000 8000 0000 00 x12
0110 3f31 ffe8 00 0000 8000 0000 00
…
end 3412 c01d…
```

Two checksums bracket it. **`world`** is the hash of the start world as the
game had it, so a start the settings cannot describe (a capture's `SHOT_BARS`)
is refused rather than replayed wrong. **`end`** is the frame and hash the game
ended on, so a replay *proves* it reproduced the fight, bit for bit — or says
it did not, and why: the header names the build and the Oven's hash, and a
different tuning is a different fight from the same inputs.

## 2 · Recording

**The tape is always running.** From the first frame, the game keeps every
input of the fight on screen (`Sim::tape`), so a moment that felt wrong can be
saved after it happened: `Y`, or the Esc menu's *Save replay*. `--record`
saves on its own at the end of every round. On a desktop a replay is a file in
`~/.config/arena/replays/` (`ARENA_REPLAYS` moves the folder); in a browser it
is a download — the one addition to `platform.rs`, where a difference between
the two goes.

**The tape describes exactly the world on screen.** It begins again whenever
the world is replaced outside a tick (a match starting from an agreed world,
the rehearsal), and it is cut back when `[` rewinds. Everything else that
changes the fight already travels as input: a restart, a class change, a trip
to another arena or creature are all a `Travel` byte on the wire
([arenas.md](arenas.md)), so a replay crosses them without being told. Which
made one thing visible: in training, Backspace used to rebuild the world
*beside* the tick while online it went on the wire. It goes on the wire in
both now, one path.

**Online, the tape holds what was confirmed.** The rollback session advances
frames with predicted inputs and re-advances them when the truth arrives;
the tape records by frame number, so the last word on a frame is the confirmed
one (`net::handle_requests_watched`). Both players' inputs are on it, as they
really were.

## 3 · Seats, and the one thing the simulation had to learn

A fighter nobody drives is a body the creature walks across the arena to stand
over, so the harness has always taken absent hunters out of a hunt, and the
game did the same for its dummy — *outside* the simulation, after building the
world. A replay of inputs alone could not know to.

So the world knows: **`World::seats`** is how many fighters are played, and
`World::seated` applies it as a fight is built — including a fresh fight built
inside a tick by a trip or a restart, which used to need the game's hand
afterwards. It is not hashed: it is read only as a fight is built, and what it
changes (a fighter's health) is. Against a person both seats are played and it
is two on both machines. The dummy keys (`1`–`7`) change it, and the tape notes
the change as an event (`@frame seats N`) so the replay changes it on the same
frame.

`sim::replay::Start::world` is now **the** way a world is built from its
settings: the picker calls it, and so does a replay, so the two cannot drift.

## 4 · Judging

```
cargo run -p hunt --bin replay -- <file> [--trace]
```

puts the replay through the simulation and hands every frame to the same
[`Report`](../../crates/hunt/src/report.rs) that judges the scripted hunter —
the creature's move table, the four windows, the threat shares, the
unanswerable hits — over a person's frames. The report asks a hunter three
things: which seat, what it last pressed (its camera, for *was that on your
screen*), and what it meant. A person (`Hunter::person`) answers the first two
exactly and the third not at all, which is honest: the report's intent column
says `a person`.

Beside the creature's report, **what the person did with their hands**
([`replay::Hands`](../../crates/hunt/src/replay.rs)), counted off the world
rather than off a plan: every move by name, thrown and landed, dodges, guards,
jumps, blows and damage taken, share of the fight airborne. A versus replay
gets the rounds and both fighters' tables and no creature.

```
cargo run -p game -- --replay <file>
```

plays a replay back in the game: both seats' inputs come off the file and the
camera follows the look it recorded, until it runs out and the controls are
yours. A desktop only — a page has no file to open.

## 5 · What it is for

The backlog's word is *unplayed*. The loop this makes possible:

1. The owner plays a fight (a link, `?hunt=<creature>`, or the desktop).
2. Something feels wrong. `Y`. A file.
3. The file is committed or sent, and `replay` says what the creature did,
   what the person did, where the windows were, and what landed from off
   screen — the same numbers the design documents are written against.
4. A change is made against those numbers, and the same replay is run again:
   a change to the creature changes the end hash (the person's inputs now meet
   a different fight), which is the point — the replay is a *question* ("what
   does this build do with what I pressed") rather than a recording.

Ghosts on the courses, a spectator's view and a replay viewer with stepping
are all the same file with a different reader, and none is built.

## 6 · Measured

- `crates/sim/src/replay.rs` tests: a tape round-trips through its text; a
  replay reproduces the fight to the bit; repeats collapse and expand; a rewind
  forgets; a re-advanced frame overwrites; every world a start builds reads
  back as that start; an edited start is refused with a reason.
- `crates/hunt/tests/replay.rs`: a taped scripted Ridgeback hunt replays to
  the same end and the same report; the hands count what the hunter threw; a
  pack hunt replays; a versus tape is judged without a creature.
- The scripted pins (`hunt/tests/pin.rs`, `sim/tests/ridgeback_pin.rs`) are
  unmoved: `seats` is unhashed and the harness seats its hunters as it always
  did.
