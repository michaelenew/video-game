# 0029 — A target body: the Ridgeback formed by hand

The owner (2026-10-23), on the contoured grown body of exploration 0027:
*"Better, and it works probably better in motion than stillframe, but it
still reads like a trash bag over some hay. I think we need a target for the
algorithms to move toward rather than trying to build ground up on the
algorithm. Take the current model, form it by hand into a creature you might
find in Monster Hunter or Dark Souls — something intricate, interesting, with
flare and details, clear posture, well-shaped form on the tail, feet, joints,
head, body. We'll refine that first, then update the algorithms to get there
without manual effort."*

So this is the target, not the method. It is built the way the cast is — a
script, never a file edited by hand — so it can be changed, rebuilt and,
most of all, *read*: every choice that makes it look like an animal is a
line with a reason beside it, and the next step is to turn those lines into
rules a grown body follows.

## How it is made

```
cargo run -p anim --bin export_beast -- ridgeback    # its skeleton and boxes, at rest
art/blender.sh art/ridgeback/build.py                # -> crates/game/assets/ridgeback.glb
```

`export_beast` writes the species' bones and part boxes exactly as the
game's rig places them at rest (`target/art/ridgeback.json`), so the body
is sculpted over the skeleton that will move it. `art/lib/beast.py` is the
toolbox; `art/ridgeback/build.py` is the creature. The stages:

1. **Flesh** — soft forms in numpy, unioned into one watertight skin by a
   voxel remesh and rounded with Taubin smoothing (which rounds without
   shrinking). Nothing can float: the union is one surface by construction.
   - The torso, neck, tail and every leg are **tubes**: a run of
     cross-sections along a curve, each a superellipse with bumps. This is
     the same generalised cylinder `game::contour` grows from (0027), drawn
     by hand.
   - **A muscle is a bump that runs down a section**, swelling and fading
     with a height per station (`muscled`), never a lump stuck on. The
     first attempt used separate ellipsoids for triceps and thighs and they
     read as tumours.
   - **The flanks come down round the tops of the legs**, as an elephant's
     and a rhino's do, so a leg leaves the body near the elbow rather than
     the hip. The legs are thick the whole way (0.75 m across at the
     forearm) and spread into a pad at the floor.
   - Skin folds at the wrists and below the elbows and knees, and the great
     folds behind the shoulder and before the thigh: arcs half sunk in the
     skin, irregular so they read as gathered hide rather than rings.
2. **Plates** — `shell`: a grid in the plate's own coordinates is projected
   onto the skin by rays from its axis, given a flat top, a **cut chamfer**
   (not a rolled edge), a raised back lip so a row overlaps like a
   pangolin's, a keel on the upper row, a rounded outline, and a little
   variety in thickness so no two are copies. Its normals are split where
   faces meet at more than 35°, so its edges read as edges.
   - Two first attempts failed instructively: plates cut from the skin's own
     faces had ragged voxel edges, and plates with a rolled bevel read as
     quilting — "a puffer jacket" — however they were coloured.
3. **Hard parts** — keeled back plates either side of the ridge, tallest
   over the barrel and leaning out; spikes down the tail and the quills it
   sprays; horns over the eyes, a nose horn, a crown of spikes over the
   neck, tusks; four hoof-nails a foot, each over a toe.
4. **The soft places** — the ridge as a row of folds standing off the spine
   between the plates, the nape as a smaller row between the collar plates,
   and the eyes: their own mesh, drawn in a material that glows, so they say
   what they are from across the arena as the pads did.
5. **Paint**, as vertex colours: countershading by which way the skin faces,
   ambient occlusion by rays against the whole body, light on ridges and dark
   in creases by curvature, a slow mottle; slate plates each their own shade,
   worn pale at their chamfers, moss on what faces the sky; horn dark at the
   root and pale at the tip.
6. **Weights** — the skin's from bone segments with a thickness each, smoothed
   over the skin; plates take the skin's under them; a horn or spike moves as
   one, with the weights at its root.

## In the game

`game::grown::model_of` names the file; `beast::setup_bodies` loads it
through Bevy's glTF loader (so a page gets it as a desktop does), the grown
body stands in until it arrives, and `beast::install_models` puts it in
place. The part each vertex belongs to rides in the second texture
coordinates, so a broken foot darkens where it is (`grown::own_colours`).
The boxes are still what is hit and stood on; F1 draws them.
`--grown-creatures` draws the algorithm's body instead, to compare.

![In the game](../gallery/0029_ridgeback_in_game.jpg)
![Up close, in the studio](../gallery/0029_ridgeback_near.jpg)

4.7 MB, about 117,000 triangles: the hide, plates and hard parts in one
mesh, the glowing flesh in another.

## What the algorithm has to learn from it

What made the difference, in the order it did, is the list a grown body
has to reach:

1. **Hard and soft are different things.** The grown body's whole surface
   is one smooth field; this one is a soft skin with hard plates *on* it,
   and the plates' crisp chamfers and overlaps are most of what reads as
   "monster" rather than "blob". A grown body needs a second layer: plates
   laid on the field by rule (where the species' look says armour), not
   the field's own surface coloured to look like plates.
2. **Muscles are sections, not lumps** — a bump that runs along a limb's
   cross-sections — and **legs are thick columns the body comes down
   over**. Both are already how `contour` is written; the numbers here are
   the target its `Column` should reach.
3. **Silhouette parts are separate geometry** — horns, spikes, quills,
   nails — placed by rays onto the skin with rules a species table can
   state (rows along a back, a crown over a skull, nails round a pad).
4. **Paint by occlusion and curvature**, not only by which way the skin
   faces: the grown hide has no occlusion at all, which is a large part of
   why it reads flat.

## Still open

- The head is generic from the front and small for the body; the face
  needs a stronger brow, a nostril and a mouth that reads.
- The legs are long for their thickness — the skeleton's, by design (the
  back is out of reach from the floor) — and the body could come further
  down round them.
- Plates on a joint can cross as it bends; they are weighted to the skin
  under them and nothing keeps two from meeting.
- The ridge's folds are simple; they want to read as exposed, ribbed flesh.
