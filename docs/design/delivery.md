---
status: standard -- the owner's, 2026-11-02
---

# Delivery -- how broad and exploratory work is built and handed over

The owner's time is the scarce thing. They do not have the time to give
precise direction on something that is not close, so nothing reaches them
until it has been played -- by a check that plays it, or by the builder
from the fighter's own camera -- and the report says what was and was not
checked. Places are held to [level-design.md](level-design.md), creatures
to [monster-design.md](monster-design.md).

**Every rule here holds unless otherwise instructed.** An instruction for a
particular piece of work wins over any of them; where one is set aside, the
report says which and why (D6).

## The rules

**D1 · Stages, with a cheap checkpoint before each dear one.** Unless
otherwise instructed, broad work goes in three stages, and the owner sees
each before the next is started:

1. **Plan**: what it is for, a sketch with distances, what it will be
   tested by. Minutes to react to.
2. **Rough and playable**: passes the play check (D3), with a report (D5).
3. **Polish**: only once the owner has said the rough one is going the
   right way.

**D2 · A small whole over a large half.** Unless otherwise instructed, a
small slice that works end to end is delivered before a larger one that
half works: a courtyard with a run-up and a camera that behaves, before a
whole tower with neither.

**D3 · The play check.** Unless otherwise instructed, anything the owner
will load passes all of these first, and the report lists each with its
result:

- `cargo fmt --all`, `cargo clippy --workspace --all-targets` and
  `cargo test --workspace` (`CLAUDE.md`, "Checks before you push").
- **Every route walked** by the five movers, as `tests/valley.rs` walks the
  valley: a route that cannot be finished fails.
- **The camera check** along every route, a turn of the view at each stop:
  `cargo run -p view --example camcheck -- <arena> "x,z,from; ..."`
  (`view::camcheck`). It fails on any unexplained camera jump, the eye in
  the land, the arm snapping in or out against something solid on one frame
  in a hundred or more, a wall cut away on one frame in fifty or more, or
  the arm pulled in close on one frame in twenty or more.
- **Every start clear**: `crates/sim/tests/spawns.rs`.
- **For a fight**, the scripted hunter's report
  ([monster-design.md](monster-design.md), M6).
- **Screenshots, and a clip where motion matters, from the fight's own
  camera** along the route ([level-design.md](level-design.md), L10).

**D4 · Load it before handing it over.** Unless otherwise instructed, the
builder starts the game in the new thing and looks -- headless, from the
fighter's own camera -- before saying it is ready. What could not be looked
at is said, not implied.

**D5 · The report.** Unless otherwise instructed, a delivery is reported
in plain words with: what was built; what was checked, how, and the
result; what was not checked; known problems; and decisions made on the
owner's behalf.

**D6 · Say which principles were followed.** Unless otherwise instructed,
every summary written after implementing something says, in so many words,
"I followed these principles for this:" and names them by their IDs here
and in [level-design.md](level-design.md) and
[monster-design.md](monster-design.md), with a phrase each on how -- and
names any it set aside, and on whose instruction.

**D7 · A kit is held to what was promised, and played before it is
handed over** (proposed 2026-11-06, after the Blood mage's slice 1 was
handed over on green tests and was "nothing like what we talked about").
Unless otherwise instructed, a new class kit or a change to one:

- **Promises before code.** Every sentence of the agreed design that says
  what a move does -- what it costs, how far, how high, how fast, what it
  does when pressed early or held -- becomes one line of a promise card
  with the number it must meet, written before the code by whoever agreed
  the design with the owner. The Blood mage's is
  `cargo run --release -p sim --example kitcheck`.
- **Tests from the promise, played the way a player plays.** The second
  press early, the button held through the move, a string mashed, a move
  out of the air -- not let go, waited and pressed again once the move is
  ready for it. A test written by the same hand as the code, from the code,
  is not evidence the promise was kept.
- **The play check is never set aside by the builder.** Every new or
  changed move is filmed from the fighter's own camera at its wind-up, its
  contact and what it leaves behind (`SHOT_PRESS`, `SHOT_FRAME`), and the
  pictures are looked at. A move with nothing on screen is not done.
- **Delegated work is checked, not relayed.** Whoever hands work to the
  owner runs the promise card and looks at the film themselves; the
  builder's report is a claim.
- **The report pastes the card**, every line kept or missed, beside the
  pictures.
