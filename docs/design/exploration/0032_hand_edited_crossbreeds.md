# 0032 — Crossbreeds edited by hand, and the breeder taught from the edits

*2026-10-24.* The owner, after 0031: *"Take each of the intermediate steps,
hand edit them to make each a reasonable creature in its own right. Once
that's done, we need to edit the crossbreeding algorithm to interpolate more
like the hand manipulated versions."*

## The hand edits

A stage of a line is a genome's body as the breeder builds it
(`sim::breed::build`). A hand edit is a short list of operations on that
body, applied before it is grown (`game::touch`):

| Edit | What it does |
| --- | --- |
| `Size(name, along, across, up)` | parts scaled about their own middles |
| `Move(name, x, y, z)` | parts moved, in the body's frame |
| `Aim(name, x, y, z)` | parts turned to a new direction about their joint |
| `Limb(pair, k)` | a limb pair longer or shorter below the hip |
| `Stance(pair, k)` | a pair's hips brought toward the middle line |
| `Fold(pair, upper, lower, foot)` | a pair posed segment by segment |
| `Chord(name, k)` | a flat part (a wing) deeper, its edge at the bone kept |
| `Drop`, `Shape`, `Knobs` | a part gone; a contour of its own; the growing's knobs |

`"torso"` names the torso's boxes; any other name matches parts by their
start (`"hind"` is both hind legs). A grounded body is stood back on the
floor afterwards.

The edits are written as data, not sculpted, so that across every stage they
are a record of what the breeder gets wrong. They turned out to be six
recurring groups (`game::touch::TOUCHES`), every stage of every line taking
one or more:

- **Heavy** (a four-legged Ridgeback-like body): the torso 1.7 times as
  deep and 0.8 as wide; the legs 0.85 as long and as thick, the hind hips
  brought in to 0.6 of the way out and the fore to 0.7; the head turned
  level, nose a little down, and moved ahead of its joint.
- **Flier** (any stage grown with its wings spread): the torso twice as
  deep and 1.3 times as wide; the head level; the wing's chord half as deep
  again; the arm under the wing 0.85 as long.
- **Feathered**: the arm under the feathers shorter again, the torso and
  head larger -- a bird's share of its own span.
- **Scythes** (a raptorial fore pair): the arms 0.6 as long and 0.7 as
  thick, folded at the chest, the head looking forward.
- **Stalk** (stood upright): the neck a stalk half as thick, the head wider
  and looking forward.
- **Stilts** (a level body hung with scythes and spindle legs): a body big
  enough to hang them from.

One fix turned up that was not a stage's but the growing's, and went into
it: the leg muscle the growing adds to a column leg (the deltoid, the thigh,
the calf) was sized off the leg's *world* bounds, which on a leaning bred
leg are mostly its length -- the "pills" 0031 showed on the bred Ridgeback's
upper legs. It is sized off the part's own thickness now (`grown::grow`), and
the raw rows below already have it.

Each tile is the breeder's body (top, "raw") over the edited one (bottom,
"hand"):

![Ridgeback fed toward the Galewing, raw and hand](../gallery/0032_ridgeback_to_galewing_hand.jpg)
![Galewing fed toward the Mantis, raw and hand](../gallery/0032_galewing_to_mantis_hand.jpg)
![Mantis fed toward the Ridgeback, raw and hand](../gallery/0032_mantis_to_ridgeback_hand.jpg)
![Ridgeback backcrossed into the Mantis, raw and hand](../gallery/0032_ridgeback_x_mantis_hand.jpg)

`GROWN_LINES=1 cargo test -p game --bin game grown_lines` writes both, the
breeder's as `<line>_<stage>_raw.obj`; `GROWN_STAGES=0,3` grows those stages
alone; `BRED_DUMP=<line>:<stage>` prints a stage's boxes with its edits; and
`art/lineage.py` takes `--yaw` and `--pitch`.
