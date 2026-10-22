# The cast's look: the owner's rules, and how to repeat them

Every correction the owner made to the characters in the graphics thread of
2026-10-20 to 22, written as a rule that can be checked, with where it lives
and how to look at it. The stories of how each was found are in
[exploration/0024](exploration/0024_a_character_done_properly.md) (the models
and their effects) and [exploration/0025](exploration/0025_painted_in_motion.md)
(the painted cast); this page is the part to follow. **Doing a class the
others have had? Read §1 to §3, then do §4 in order.**

The owner's own words are in italics, with the day they were said.

## 1. The standard

1. **AAA, stylised, plausible.** *"Triple A grade -- not super realistic,
   still somewhat stylized, but the human form being plausible and not
   uncanny, clothing being realistic ... could plausibly live in a Guild Wars
   or a FFXIV"* (10-20). Every part of a class is held to it, not only the
   body: its animations, its effects, its thrown things.
2. **Everything the class shows is in scope.** *"Have you remade all
   auxiliary features of every class in the correct style? ... all elemental
   effects of the elementalist, all the blood mage things"* (10-21). A class
   is done when nothing it puts on screen is still in the old style -- the
   Reaver's shurikens were missed once (*"all the shadow shurikens are still
   their old models"*, 10-20). Check: play every input of the class and look
   for a primitive (a cylinder, a capsule, a flat disc) or an untextured
   shape.
3. **No artefacts.** *"There is still a purple cylinder around her when she
   aerial lclicks. I want all artifacts like that gone"* (10-20). A hit
   volume, a debug rod, another class's effect drawn by a shared number:
   none of it is drawn outside F1. The one found was the Blood mage's scythe
   volume drawn round the Reaver, because a move's number was read without
   its class (`tests/presentation.rs`, `only_the_blood_mage_swings_a_scythe`).
   Check: list every visible mesh over a metre near the fighter on the frame.
4. **Keep the originals.** Each new look is opt-in beside what was there
   (*"don't get rid of the existing reaver model, it's very good"*, 10-20;
   *"again, keep the originals"*, 10-21): the painted cast is
   `--painted-cast`, the plain models are the default.
5. **Ask what it costs the world, not just the character.** *"I can't have
   the characters look like this and the world be a very different style"*
   (10-20); *"I'm worried the game will [be] tens of GB"* (10-21). A look is
   judged on whether it generalises to generated land and bred creatures,
   and measured before and after (0024 §11: about 4 MB on disk and 10 MB of
   GPU memory a character after the squeeze).

## 2. Bodies, people, clothes

1. **Variety of people.** *"You don't have to make every model a white
   woman or a black man"* (10-21), then *"make 2 of the 5 white and 2 more a
   different race"*. The five now: a Northern European Reaver, a white Dual
   mage, a South Asian Champion, an East Asian Elementalist, a Black Blood
   mage, each with their own face shape, skin and hair (`art/<class>/recipe.py`).
   A new character is chosen against the cast, not by default.
2. **Chests.** *"Fix all of the women's chests ... and remove the blood
   mage's breasts"* (10-21). A fighter in athletic clothes has one
   compressed shape under the cloth, not two outlines; the Blood mage has
   none (`recipe.py`, `wardrobe.py`'s pressed shape).
3. **Hair is real hair.** *"Fix the hair so it looks like real hair"*
   (10-21): cards -- strips of painted strands, layered, rooted high, thinning
   at their edges (`wardrobe.hair_cards`, `art/README.md` "Hair").
4. **Colour direction is the owner's call, and it is specific.** *"The
   reaver more toward the purple side and the shadow more towards the grey
   side by desaturating"* (10-20). When the owner names a hue shift, move it
   in the class's palette (`art/<class>/build.py`) and say how far.
5. **See-through means see-through.** *"The shadow needs to be transparent,
   not opaque, but I like the highlights and colors"* (10-20): the ink reads
   the world through it and keeps its rim light (`ink.rs`).
6. **A model is built, never edited**: `art/<class>/build.py` under Blender
   (CLAUDE.md, "A character is a script").

## 3. Motion

1. **Attacks reach.** *"The lclick animations need to extend her much more.
   She should lunge all the way out ... and extend her stabbing hand all the
   way forward"* (10-20). A strike's arm goes to nearly all of its reach
   (`anim::track::full_reach`) and the body travels with it.
2. **Sell the hit.** *"A brief effect upon impact would help to sell the
   strength of the hit ... look for other similar opportunities"* (10-20):
   `impact.rs`, on the frame a blow lands, on the simulation's clock.
3. **More joints where a real body bends.** *"Add more bones"* (10-20): a
   neck, clavicles, toes -- twenty-one joints (`animation.md`).
4. **A walk is a whole body.** *"Walking is still really stiff. The head and
   shoulders don't move at all and their elbows are right next to their
   sides"* (10-20); *"those don't look accurate at all. Read about how to make
   a natural walk and run, then go do it"* (10-22). Done from the reference
   gait numbers, not by eye (feel log, 2026-10-22, "a walk and a run a person
   would recognise"): strides of 1.5 m walking and 3.9 m running, so the
   legs go at a person's cadence; the hips riding on the standing leg, which
   straightens; arms close to the body, about twenty degrees each way
   walking from soft elbows and forty-five running with the elbows near a
   right angle; a walk upright to three degrees, a run to seven.
5. **Lean into the going.** *"They need to lean forward slightly while
   moving forward, and lean into their acceleration"* (10-22): `view::lean`,
   from the ankles, feet held (`animation.md`, "Leaning into the going").
6. **Where two motion styles disagree, take the one that is easier to
   see.** *"Take their motion blur where it's easier to see and everything
   else from this thread"* (10-21, merging main).

## 4. The painted cast: how to paint a class

The look the owner is after (10-21): *GW2's brushed concept and
loading-screen art, in 3D, in motion. "A picture is easy to make look
brushed, a rotating 3d object comes across weird when the brush strokes are
painted onto the object."* The answer is strokes that are particles riding
the skin, square to the eye, colour read from the rendered picture under
them (0025 §2 to §3, `crates/game/src/brushwork.rs`, `brushwork.wgsl`).
And the image to hold in mind (10-22): *a cloak that wisps into flame at its
hem -- with paint for flame -- and the whole body held about a quarter of the
way from fabric to flame.*

### 4.1 The rules, each a thing to check on the screen

| # | Rule | The owner | Where it lives |
| --- | --- | --- | --- |
| P1 | **Small, even strokes, like the land's.** A few centimetres wide, near-parallel along each part, one hand's slant over the whole figure. Not coarse, not random. | *"Much too coarse and random looking. Make the strokes more like the terrain painted look ... much smaller"* (10-22) | `BROAD` 3.4 cm, `LOOSE` 0.3, `HAND` |
| P2 | **Bigger brush on broad open cloth only.** A coat's panels, a cape, a skirt: about twice. Chest, back, thighs a quarter more. | *"For broad open areas, like a cape, you can make the strokes a bit bigger"* (10-22) | `focus()` |
| P3 | **Colour varies inside a stroke** -- streaks lane by lane, richer where the brush was loaded, paler at the dry tail. | *"Add some variation in the color inside the stroke"* (10-22) | `brushwork.wgsl` |
| P4 | **Thin in the middle.** About half the paint across a stroke's middle, fuller at its edges; the cloth shows through. | *"Turn down the opacity a bit in the middle of the stroke"* (10-22) | `brushwork.wgsl` |
| P5 | **Clothes only.** No strokes on skin (the `body` piece), none on hair cards; the face left nearly as modelled; what the hands hold left nearly alone. | *"Don't do the skin yet"* (10-22) | `paint()`, `focus()` |
| P6 | **Inside the lines.** A stroke paints only where its own body is drawn: never past the outline into the air or onto the wall. | *"It's like you're colouring outside the lines. Any spillage needs to be intentional, like at the hem of a shirt or a cape where the object terminates, not just goes out of view"* (10-22) | the depth test in `brushwork.wgsl` |
| P7 | **Inside its own colour shape.** A stroke does not cross from one material onto another lying beside it at the same depth (a mantle onto a shirt, a bodice onto a coat). | *"The strokes around the champion's shoulder cloth are still messy ... those shouldn't bleed because they're 'inside' the character outline"* (10-22) | `same` in `brushwork.wgsl` |
| P8 | **The hem frays, softly.** Where the cloth really ends -- a hem, a cuff, a hood's mouth -- about one stroke in eight near it becomes a short tongue of paint past the edge, into the air and never onto other cloth. A soft fray, never spines. A thickened garment's hem is found where it folds back; a band (belt, strap) has none. | the fabric-to-flame image (10-22) | `WISPS`, `WISP_REACH`, `openness` (`FOLD`, `BAND`) |
| P9 | **In motion it stays on the body.** On a turntable and a walk, no stroke slides across the cloth, swims, or pops as a part turns away. | *"It needs to work in motion too"* (10-21) | the whole design (0025 §2) |
| P10 | **Dark cloth shows its brushwork.** A stroke's variation is never smaller than it would be on a colour a third of the way up the eye's scale, so black reads as painted black, not as no paint. | found on the Blood mage's coat (10-22) | `shift`, `DARK` in `brushwork.wgsl` |

### 4.2 What was tried and refused -- do not do these again

| Tried | Why it failed |
| --- | --- |
| Strokes painted into the texture | foreshortens and wraps as a limb turns: printed cloth (0025 §2) |
| A filter over the screen | the body slides under the strokes (0025 §2) |
| Colour from the texture, lit by the stroke's own bands | strokes and the gaps between them never agreed: confetti (0025 §7) |
| Hard value bands, rectangular strokes | digital tiles (0025 §7) |
| Strokes leaning out past the silhouette | rags; then, as long wisps off the outline, a porcupine (0025 §7 to §8) |
| Big, loose, every-which-way strokes | *"too coarse and random"* (0025 §8) |
| Strokes on the skin | *"that's worse"* (0025 §9) |
| A blended (transparent) stroke material | reads a white placeholder unless something else asks for the picture's copy: keep it cut out and blend in the shader (0025 §9) |
| A stroke's colour averaged across a seam | a mix of two materials; it reads as bleed (0025 §10) |
| Finding a hem only as an edge with one triangle on it | every garment is built thickened, so it found none: no class frayed (0025 §11) |
| Every rim of a thickened shell as a hem | a belt, a strap, a collar are nearly all rim (0025 §11) |
| Variation as a plain multiply of the colour | on black cloth it is nothing (0025 §11) |

### 4.3 The recipe

1. **The model exists and its pieces are named** by material in
   `art/<class>/outfit.py`: the skin is `body`, hair is `hair*`/`cards*`
   (both skipped), coat panels are skinned to joints past `JOINT_COUNT`
   (the bigger brush, P2). If a broad cloth of the new class is not a coat
   panel -- a robe, a cape on a body joint -- it gets the normal brush, which
   may be the first thing to fix.
2. **Turn it on**: add the class to `painted()` in
   `crates/game/src/brushwork.rs`, and to the `--painted-cast` line in the
   manual (`crates/manual/src/lib.rs`).
3. **Film it** (`cargo build -p game`, then §5): a front and back
   portrait; a close-up of its busiest colour boundary (P7); a close-up of
   its longest hem (P8); a turntable; a walk from the side. Always beside the
   unpainted model at the same moment, so what the strokes did is visible.
4. **Judge** every row of 4.1 on those pictures, and say per row what was
   seen. Two numbers worth printing once from `place` while judging: how
   many strokes each piece got and how many are wisps, and how many corners
   `openness` put on a hem -- a hem count of nought is what hid P8 from
   every class until the Blood mage (0025 §11). Then F6 in a running game for anything a still did not settle.
5. **Fix in the general system**, not with a per-class branch, unless the
   class is the only one with that case -- a fix to `brushwork` is true of
   every painted class, and the others are re-filmed after it.
6. **Write it down**: a section in 0025 (what was asked, what changed, what
   was tried and reverted), and a row here if the owner made a new rule.

## 5. The judging kit

```text
cargo build -p game
# a portrait: seat, degrees round from the front, metres away, height looked at
./scripts/reel.sh bm_front 91 1 1 DEMO=0 SHOT_PORTRAIT=0,0,2.6,1.0 -- --p1 blood --painted-cast
# a close-up: 0.8 m, at the shoulder
./scripts/reel.sh bm_close 92 1 1 DEMO=0 SHOT_PORTRAIT=0,30,0.8,1.4 -- --p1 blood --painted-cast
# a turntable: SHOT_SPIN degrees a second round them
./scripts/reel.sh bm_spin 93 24 1.0 DEMO=0 SHOT_PORTRAIT=0,0,2.6,1.0 SHOT_SPIN=15 -- --p1 blood --painted-cast
# a walk from the side, slowed (SHOT_SLOW) so consecutive pictures are close
WARM=45 DELAY=12 ./scripts/reel.sh bm_walk 94 48 1.2 DEMO=1 "SHOT_PRESS=_:1500,W:50,_:80" \
    "SHOT_SLOW=3,1490" SHOT_PORTRAIT=0,90,5,1.0 -- --p1 blood --painted-cast
```

Each writes `target/reel/<name>/`, a GIF and a contact sheet. Crop with
ImageMagick (`convert f01.png -crop 360x400+460+220 +repage`) to look at
the figure. In a running game, **F6** orbits a fighter with nobody faded
(mouse round, wheel in and out, Alt and wheel up and down); P, `[` and `]`
pause and step.

How the owner wants to be shown work: *"show me the shoulder close-up when
it's done"*, *"show me a walk and a turntable of both"* (10-22) -- send the
pictures, not a description of them, and say plainly what has not been
looked at.

## 6. Parked

- **Facets** (10-20): the owner's favourite of the style proposals --
  *"Facets are by far the best ... I would rather it read like a Faceting of
  a real mesh not a cartoonists proportioned humanoid. Take the shadow reaver
  triple A model, put a somewhat coarse mesh over it, denser at detailed
  sections, and draw the faceting using those vertices"* -- then *"keep this
  to the side for now"*. Made in Blender only: a shell wrapped over the real
  model (its clothes are open sheets, so it must be wrapped, not decimated
  directly), reduced in two steps to about 3,600 triangles with the face,
  hands and joints protected, each facet one flat colour from the texture
  under it, banded light. `art/parked/facets.py`; nothing of it is in the
  game. It is the answer to §1.5 that the painted cast is not: it
  generalises to generated land and bred creatures by construction.
- **Strokes on the skin**: waiting on the owner (P5).
