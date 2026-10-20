# art — characters built by script

The game's characters are made here, offline, by Python scripts driving
Blender (run headless, as the `bpy` module), and committed as one file each
under `crates/game/assets/`. Nothing here runs in the game, and the game does
not need any of it to build: it embeds the file.

```
./scripts/setup-tools.sh art                  # Blender 5.2 and MakeHuman's CC0 data, once
art/blender.sh art/reaver/build.py            # the Shadow Reaver -> crates/game/assets/reaver.glb
art/blender.sh art/reaver/build.py --look target/art/look.png   # and a turnaround of her
art/blender.sh art/reaver/build.py --paint    # only repaint and export the last dressed model
art/blender.sh art/reaver/body_check.py target/art/body.png run_forward 6   # the bare body, posed
```

`build.py` needs the game's skeleton, which the animation exporter writes:
`cargo run -p anim --bin export -- target/art/anim.json`. A full build is
about ten minutes on four cores; the inflated bodies the clothes are cut
from are cached under `target/art/shells`, so a second build is quicker.

**Why it is built rather than modelled** is in
[`docs/design/exploration/0024_a_character_done_properly.md`](../docs/design/exploration/0024_a_character_done_properly.md):
there will not be a developer for this game forever, and a character that is
a script can be changed by the next person, rebuilt for a new skeleton, and
read.

## The stages

| Stage | File | What |
| --- | --- | --- |
| The body | `lib/mh.py` | MakeHuman's base mesh, its shape targets (sex, age, muscle, weight, proportions) and its rig's weights |
| Fitted | `lib/fit.py` | the body moved onto the game's own skeleton (`view::skeleton`) in a bind pose chosen to suit it; MakeHuman's 160 bones folded into the game's sixteen; the hands closed round a grip |
| Dressed | `lib/garment.py`, `reaver/outfit.py` | every piece cut from the body **inflated** by a distance field (so nothing it covers pokes out as a spike), bridged over hollows, trimmed by planes, its hems relaxed; made twice -- a high piece with its folds, and the game's piece; coat tails dropped under cloth simulation; hard things (blades) lofted in `lib/forge.py` |
| Painted | `lib/paint.py` | occlusion, curvature, facing and position baked with Cycles (the folds from the high piece), and composed in numpy into a hand-painted colour: cool violet shade, warm edges, value falling toward the feet, hems trimmed and stitched, brushwork laid in 3D so it does not break at a seam |
| Exported | `lib/glb.py` | one binary glTF, written by hand so its joints are exactly the game's sixteen (and the coat's eight) at their bind places |

`lib/stage.py` is a photo studio for looking at any of it without the game.

## The rules

- **Never edit the `.glb` by hand.** Change the script and rebuild, or the
  next rebuild undoes it.
- **The joints are the game's.** The model's joint nodes are named
  `view::skeleton::Joint::name` and the game sets them in character space;
  a change to the skeleton is a rebuild here.
- **Judge it in the game, lit** (CLAUDE.md, "Judge a palette lit"): the
  turnaround is for shape. Colours that read well in Blender's studio came
  out pale lavender under the arena's sun; `SHOT_PORTRAIT` puts the game's
  camera round a fighter for this.
- **Everything is CC0 or ours.** MakeHuman's mesh, targets and weights are
  CC0 (pinned to a commit in `scripts/setup-tools.sh`); nothing else is
  taken from anywhere.
