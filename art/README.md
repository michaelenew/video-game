# art — characters built by script

The game's characters are made here, offline, by Python scripts driving
Blender (run headless, as the `bpy` module), and committed as one file each
under `crates/game/assets/`. Nothing here runs in the game, and the game does
not need any of it to build: it embeds the file.

```
./scripts/setup-tools.sh art                  # Blender 5.2 and MakeHuman's CC0 data, once
art/blender.sh art/reaver/build.py            # the Shadow Reaver -> crates/game/assets/reaver.glb
art/blender.sh art/champion/build.py          # the Champion -> champion.glb
art/blender.sh art/elementalist/build.py      # the Elementalist -> elementalist.glb
art/blender.sh art/blood_mage/build.py        # the Blood mage -> blood_mage.glb
art/blender.sh art/dual_mage/build.py         # the Dual mage -> dual_mage.glb
art/blender.sh art/reaver/build.py --look target/art/look.png   # and a turnaround
art/blender.sh art/reaver/build.py --paint    # only repaint and export the last dressed model
art/blender.sh art/reaver/build.py --only hood,eyes --no-export --look x.png   # try a few pieces
art/blender.sh art/reaver/body_check.py target/art/body.png run_forward 6   # the bare body, posed
```

**A class is a spec on one builder.** `lib/character.py` is the five stages
below; each `art/<class>/` is three files: `recipe.py` (the body, as
MakeHuman's targets), `outfit.py` (what it wears, as calls on
`lib/wardrobe.py` and pieces of its own) and `build.py` (the palette, the
glow, which materials are metal, and the painter's knobs where they differ).
`lib/wardrobe.py` is the tailor: a garment cut from the inflated body, a
strap, a band, a cape, a collar, boots, sleeves, a jerkin, coat tails, hair,
eyes, the body's skin and glove masks and the skin nothing sees. The
material the exporter names `team` is the one the game dyes a seat's
colour.

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
| Fitted | `lib/fit.py` | the body moved onto the game's own skeleton (`view::skeleton`) in a bind pose chosen to suit it; MakeHuman's 160 bones folded into the game's twenty-one; the hands closed round a grip |
| Dressed | `lib/garment.py`, `reaver/outfit.py` | every piece cut from the body **inflated** by a distance field (so nothing it covers pokes out as a spike), bridged over hollows, trimmed by planes, its hems relaxed; made twice -- a high piece with its folds, and the game's piece; coat tails dropped under cloth simulation; hard things (blades) lofted in `lib/forge.py` |
| Painted | `lib/paint.py` | occlusion, curvature, facing and position baked with Cycles (the folds from the high piece), and composed in numpy into a hand-painted colour: cool violet shade, warm edges, value falling toward the feet, hems trimmed and stitched, brushwork laid in 3D so it does not break at a seam |
| Exported | `lib/glb.py` | one binary glTF, written by hand so its joints are exactly the game's twenty-one (and the coat's eight) at their bind places |

`lib/stage.py` is a photo studio for looking at any of it without the game.

**A creature is built the same way** (2026-10-23): `cargo run -p anim --bin
export_beast -- ridgeback` writes its skeleton and part boxes at rest, and
`art/blender.sh art/ridgeback/build.py` sculpts a body over them with
`lib/beast.py` -- forms unioned into a skin, carapace shells grown off it, a
two-million-triangle sculpt with its detail and paint, baked onto a light
mesh as colour, normal, glow and roughness maps -- into
`crates/game/assets/ridgeback.glb`. Its joints are the species' bones. About
ten minutes; `--fast` is a coarser draft in six, `--size <px>` the textures'
size, `--no-export` leaves the file be, and it always writes a studio sheet to
`target/art/ridgeback.png` and `ridgeback_near.png`. Why, and what it is for:
[`docs/design/exploration/0029_a_target_body.md`](../docs/design/exploration/0029_a_target_body.md).

**Since 2026-10-24 a creature is a spec on one builder**, `lib/creature.py`:
its forms, any shells, its sculpt and paint, its hard pieces (horns, claws,
feathers, scythes -- `ringed_horn` and `vane`), and the segments its skin is
weighted to; the builder does the union, the bake, the weights and the file.
`art/galewing/build.py` and `art/mantis/build.py` are written that way, about
four minutes each (`SCALE` sizes the voxels to the animal: the Mantis is three
metres, so 0.3). Why: [`docs/design/exploration/0030_two_more_targets.md`](../docs/design/exploration/0030_two_more_targets.md).
The Ridgeback's moved onto it too (2026-10-24, exploration 0031).

**Crossbreed lines** (exploration 0031): `GROWN_LINES=1 cargo test -p game
--bin game grown_lines` grows every stage of the lines in `game::bred::lines`
to `target/lines/`, and `art/blender.sh art/lineage.py` renders each line as
a strip of its stages (`--line <name> --stages 0,3,6` for a few, `--yaw`
and `--pitch` to turn the camera). A stage with hand edits (`game::touch`,
exploration 0032) is drawn as the breeder's body over the edited one.
`BRED_GAIT=<line>:<stage> cargo test -p game --bin game bred_gait` and
`art/blender.sh art/gait.py` show a stage's skeleton through each gait it
has: eight frames, side on over a top view.

## The rules

- **Never edit the `.glb` by hand.** Change the script and rebuild, or the
  next rebuild undoes it.
- **The joints are the game's.** The model's joint nodes are named
  `view::skeleton::Joint::name` and the game sets them in character space;
  a change to the skeleton is a rebuild here.
- **Judge it in the game, lit** (CLAUDE.md, "Judge a palette lit"): the
  turnaround is for shape. Colours that read well in Blender's studio came
  out pale lavender under the arena's sun. In a running game, **F6** puts
  the camera on an orbit round a fighter -- mouse round, wheel in and out,
  Alt and the wheel up and down the body, nobody faded for being close
  (`crates/game/src/inspect.rs`); `SHOT_PORTRAIT` is the same for a
  screenshot, fixed at launch.
- **Everything is CC0 or ours.** MakeHuman's mesh, targets and weights are
  CC0 (pinned to a commit in `scripts/setup-tools.sh`); nothing else is
  taken from anywhere.

## Hair

Hair is **cards**, as it is in every game that has real-time hair: a few
hundred strips of painted strands, layered over a scalp and laid along the
way the hair is combed (`wardrobe.hair_cards`). Each card's texture
(`wardrobe.strand_texture`) is a lock -- thin strands, thickest in the
middle and thinning to its edges so neighbouring cards blend, darker at the
root, a band of sheen a third of the way down, tapering to uneven tips --
cut out between the strands, so the edges and the layers read as hair. They
are the model's fourth material, `hair_cards`, masked and double-sided; the
game keeps it a cut-out through the camera's fade. The scalp under them
(`wardrobe.hair`, in the hair's own colour in shadow) is so nothing bald
shows between cards. A style is a combing: `back` (swept back off the
face), `bob` (from a parting, falling to the jaw, with a fringe if asked),
`knot` (drawn back to a bun, which is `wardrobe.stone` in the hair's
material). Cards are rooted high, as hair grows from the crown and falls
over the rest, and off the scalp's edge they fall rather than following it.

The first three tries were a sculpted mass -- combed, lifted, falling --
and every one read as a helmet or a cap close up. Cards are what fixed it.
