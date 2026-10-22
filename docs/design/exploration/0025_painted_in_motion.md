# 0025 -- A character painted, in motion

**Thread.** The characters (0024) read as good modern 3D -- and miss GW2's
concept and loading-screen art, which is the look the owner wants: *broad
brushstrokes amid more classic 3D character art*. GW2 itself never got that
look into the game, and the reason is the whole problem: **a picture is easy
to make look brushed; a turning 3D object with strokes painted onto it looks
wrong.** The owner's guess at the answer: a 3D mesh, and strokes painted
dynamically over it, so the brushwork does not follow the contour exactly
the way a painted texture would. Keep the originals; try it on the Shadow
Reaver and the Champion first.

## 1. What the eye reads in a GW2 concept painting

Looked at as a set of properties rather than a vibe:

1. **Value masses before detail.** Three or four values -- the light, the
   half, the shadow, an accent -- laid as flat shapes. A cloak is one shadow
   shape with two lit planes, not a smooth gradient.
2. **Each stroke is one colour.** A flat brush, hard-sided, its end breaking
   into bristles. The gradient is *made of* strokes of stepped colour, not
   blended inside one.
3. **Strokes go across the form, not along the texture.** They follow the
   planes -- down a sleeve, across a chest -- and at an angle, often a shared
   diagonal: the painter's hand.
4. **Lost and found edges.** The silhouette is broken where strokes run past
   it into the air, and kept hard where it matters (the face, a blade).
5. **Warm light, cool shadow.** Shadows go toward violet and blue, lights
   toward a warm yellow; the hue moves with the value.
6. **Detail where you look.** The face, the hands and the weapon are
   rendered; the cloak's hem is three strokes.

## 2. The three ways to lay a stroke, and what each does in motion

| | Where the stroke lives | In motion | Why it fails or works |
| --- | --- | --- | --- |
| **On the surface** (a painted texture) | in the mesh's UVs | moves with the body | **foreshortens and wraps**: a stroke on a sleeve turning away squashes to a sliver and bends round the cylinder. It reads as a printed fabric -- the "weird" the owner names. It also follows the contour exactly, which is the opposite of 4. |
| **On the screen** (a filter: Kuwahara, a stroke pass over the frame) | in pixels | stays while the body slides under it | the **shower door** (Meier 1996): strokes swim across a moving figure. Fine for a still, wrong for a fight. |
| **A particle** -- anchored to the surface, drawn facing the screen | a point on the body; a card in the picture | **moves with the body, never foreshortens** | Meier, *Painterly Rendering for Animation* (1996); 0020 chose it for the land. The stroke is a point on the skin carried by the skeleton, and drawn as a flat card square to the eye. A sleeve turning away does not squash its strokes; they slide together and the far ones go behind. |

The particle is the only one of the three whose failure is not built in, so
it is the frame. What a *character* adds that the land never had is where
the open work is.

## 3. What a body adds to Meier

- **It deforms.** A particle has to ride the skin. It is a point on the
  bind-pose mesh carrying that point's joint weights, so the GPU skins it
  exactly as it skins the surface it came from -- the same `skin_model` call
  -- and it is where the cloth is, every frame, at no CPU cost.
- **It turns its back.** A stroke on the far side must leave without
  popping. It fades by how squarely its surface faces the eye -- the
  normal, skinned -- through a dithered threshold, so it thins out stroke by
  stroke over the last few degrees rather than all at once.
- **Its strokes need a direction that is the body's.** A direction in screen
  space (always diagonal) is the shower door again, for angle instead of
  place. A direction fixed in the bind pose and skinned with the point turns
  with the limb. Which direction: the long axis of the part the point
  belongs to (its dominant joint's spread of vertices -- along a sleeve, down
  the torso), turned by one shared hand angle and a little jitter. Strokes
  run down an arm and across it at the painter's slant, and keep doing so as
  the arm swings.
- **Each stroke is one colour, chosen as a painter would.** Its colour is
  worked out once a frame, at its centre: the costume's colour under it
  (sampled over the stroke's footprint in the texture, so a stroke is the
  average of what it covers, not a texel), lit in **bands** -- shadow, half,
  light, with a short soft step between -- shadow toward violet and light
  toward warm, a cool rim where it turns away. Every pixel of a stroke is the
  same colour: the gradient is stepped *between* strokes (properties 1, 2, 5).
- **Its edge is lost.** A stroke centred near the silhouette is a card half
  of which is in the air: it runs past the outline by its own size, and the
  outline is broken by brushwork rather than drawn (4). Where it must stay
  found -- the face -- strokes are fewer and smaller and the model shows.
- **Which stroke is on top.** Strokes write depth and cut out by their brush
  shape, so the nearest stroke wins at every pixel and order needs no
  sorting. Each card is pulled a little toward the eye, small strokes more
  than large, so detail sits on the masses as a painter lays it last.

Under the strokes the model is still drawn, as it is now: the
**underpainting**. Where strokes leave a gap, what shows is the painted 3D
character rather than a hole -- the "classic 3D character art amid
brushstrokes" of the brief, literally.

## 4. What it costs

About three thousand strokes a character, four corners each: twelve
thousand vertices and a few overlapping cards a pixel over a character's
share of the screen. Placed once per model from its mesh when it loads
(`game::brushwork`), skinned and lit on the GPU every frame, nothing on the
CPU per frame and nothing allocated.

## 5. What could still be wrong, in order

1. **Boiling or swimming at the size of a stroke** when the camera turns
   round a still character: cards stay square to the eye, so as the view
   turns, each card turns in place on its point. Meier's answer was that this
   reads as paint; it is the first thing to look at in motion.
2. **Density with distance.** Strokes are sized in metres, so a far
   character is a smaller painting with smaller strokes, and a near one gets
   bigger strokes with gaps between. The camera is mostly over the
   shoulder, at one distance; if it matters, 0020's nesting (a coarse set
   that is a subset of a fine one, the fine fading in) is the known answer.
3. **Strokes on hands and blades** may turn the readable silhouettes of a
   fight -- what a blade is doing -- into mush. The weapon is the one thing
   a fight is read by; it may want to stay found.
4. **The outline pass** draws a line round every depth step. The strokes are
   kept out of the depth prepass it reads, so it outlines the body, not the
   brushwork.

## 6. Built, and how to see it

`--painted-cast`: the Shadow Reaver and the Champion with their brushwork
over them; without it, every class as before. Judged in motion as well as
still: `SHOT_SPIN=<degrees a second>` turns the portrait camera round a
fighter, and `SHOT_SLOW=1` with captures a second apart gives consecutive
frames of a walk or a combo.

## 7. What the first day taught (2026-10-21)

Five rounds, each judged on a portrait, a turntable and a walk:

1. **Colour from the texture, lit by the stroke's own bands** read as
   confetti: the strokes' lighting and the model's lighting in the gaps
   never agreed. **Colour from the picture** (Meier's own answer) fixed it
   at once -- a stroke reads the lit character under its centre from the
   copy Bevy keeps of everything solid (`reads_view_transmission_texture`),
   so stroke and gap are one picture, shadows, hair and steel included, and
   no texture is needed. Each read is kept only where the depth there is
   the stroke's own body, so a stroke at the silhouette never takes the
   wall's colour; a stroke with none of its body under it is not laid.
2. **Strokes poking out of the silhouette** read as rags. At the edge of
   the form a stroke now turns to lie along the outline and narrows; at an
   open edge of the cloth (a hem, a cuff, the hood's mouth: an edge with one
   triangle on it once seams are welded) it lies along that edge, no wider
   than the cloth beside it.
3. **Every stroke its own way** read as noise. Each is leaned toward one
   shared slant on the canvas -- the painter's hand -- by its angle only, so
   it still rides the body.
4. **Hard value bands and rectangles** read as digital tiles. The bands are
   now gentle (a stroke stays close to what is under it, and reads through
   its texture: a bowed, tapered shape, bristle streaks, a dry tail), and
   GW2's strong brushwork is where strokes meet the outline.
5. **Strokes over a blade** smeared it: what the hands hold is nearly left
   alone.

**In motion** the strokes stay on the body -- a walk and a sword combo show
no sliding, and the turntable no swimming; that was the part a texture or a
screen filter cannot do. What is not there yet: the look is a painted
*surface*, not yet a GW2 *painting*. Next, in order: the underpainting --
the model itself simplified in value under the strokes, so the gaps are
paint too; strokes sized by where the eye goes (broad on the cloak, fine at
the face); and a few strokes flicked past the silhouette where GW2 lets an
edge go.

## 8. Smaller, and a quarter of the way to flame (2026-10-22)

The owner's verdict on §7: *much too coarse and random looking; make the
strokes more like the terrain's, and much smaller; bigger for broad open
areas like a cape.* And the vision: the moment in animation when a cloak
wisps and becomes flame at its hem -- with paint for flame, and the whole
body held about a quarter of the way from fabric to flame.

- **The land's stroke.** The body's strokes are now `splats.wgsl`'s: a
  flat brush three to four times as long as it is wide, tapered and dry at
  its ends by how loose it is, its edge broken by bristle noise, its colour
  only a shift of what is under it (lighter or darker, warmer or cooler).
  Loose at 0.3, so they lie near-parallel along each part, as the land's
  built work does, and read as one hand's texture rather than scatter.
- **A third the size** (3.4 cm wide on most of the body, about five times
  as many), **twice that on the coat's panels**, a quarter more over the
  chest, back and thighs, half on the face and hands, and fewer on the face
  so the modelled face shows. None on the hair: its cards are strands
  already, and a stroke over them was a block on the head.
- **Wisps.** At the cloth's open edges -- the hem, a cuff, the hood's mouth
  -- about one stroke in eight becomes a tongue of paint rooted on the cloth
  and running out past the edge, down the slope of the distance to it,
  lifted, curling, swaying and breathing in length on the renderer's clock;
  full from its root, rounding off, its bristles giving out toward a dry
  tip. Very rarely one lifts off the silhouette where it faces up. The first
  try had a third of the outline's strokes lift off as long points and read
  as a porcupine; a quarter of the way to flame is a soft fray, not spines.

## 9. Colour inside the stroke, thin in its middle, clothes only (2026-10-22)

The owner, on §8: *worse; don't do the skin yet, add some variation in the
colour inside the stroke, and turn down the opacity a bit in the middle of
the stroke.*

- **Clothes only**: no strokes on the body piece (skin) for now.
- **Colour inside a stroke**: each bristle's lane a little lighter or
  darker, warmer or cooler than its neighbours -- streaks along the stroke
  -- and the paint richer where the brush was loaded, paler toward the dry
  tail.
- **Thin in the middle**: a stroke's paint is about half there across its
  middle and fuller at its edges, where a loaded brush leaves its ridge, soft
  at the very edge and thinning at the tail; the cloth shows through.

**A trap on the way.** Made a blended material, a stroke is drawn in Bevy's
transparent pass, and the copy of the picture it reads is only made when
something in the *transmissive* pass asks for it -- the Reaver's ink shadow
did, the Champion's fight had nothing, and his strokes read a white
placeholder. So a stroke stays cut out (drawn in the transmissive pass,
where the copy is always made) and does its own blending: its paint over
the picture at its own pixel, which it already reads.

## 10. Inside the lines (2026-10-22)

The owner: *it's like you're colouring outside the lines. Any spillage
needs to be intentional, like at the hem of a shirt or a cape where the
object terminates, not just goes out of view.*

The outline is where a form turns out of view; the hem is where it ends.
Only the second may fray. So a stroke now paints a pixel only where its own
body is drawn there (the depth under it within a few centimetres of its
own -- not the wall past the outline, and not an arm crossing in front),
and the wisps that lifted off the outline are gone. The hem's wisps are the
one exception: they run on past the cloth's real edge by intent.

**Inside its own colour shape, too** (the owner, the same day: the
Champion's shoulder cloth "shouldn't bleed because they're inside the
character outline, it reads messy"). The mantle lies on the shirt at the
same depth, so depth alone let its strokes run onto the shirt. A stroke now
keeps to its own *stuff*, as a painter keeps a colour shape: it fades out
where the picture under a pixel is clearly another colour than the one it
was taken from (`same`, a relative difference past about a third), and the
points it averages its colour from count only where they are its own stuff,
so a stroke at a seam is one material's colour, not a mix of two.

## 11. The Blood mage, by the recipe (2026-10-22)

The owner: *write down my corrections and guidance in a way that's going to
be reproducible, then try to one-shot the blood mage's painted-cast look.*
The rules went into [`cast-look.md`](../cast-look.md) first, committed, and
the Blood mage was then painted by following its recipe and nothing else.
Step 1 needed nothing (her model's pieces are named as the Champion's, her
skirt's panels on the panel joints), step 2 was one line. Judging (step 4)
found three things, all general, all fixed in `brushwork` for every painted
class (step 5):

- **No hem on any class had ever frayed.** Every garment is built
  thickened (`g.thicken`), so no edge has one triangle on it, and
  `openness` -- which looked for exactly that -- found no edge on any
  costume: zero corners of 39,586 on hers, of 28,832 on the Champion's. The
  wisps of §8 and the edge-hugging of §7 had not been on screen since the
  thickened models. A thickened sheet's edge is now found where it folds
  back (an edge no longer than 7 mm whose ends face apart), and only on a
  shell that is a sheet: a band -- belt, strap, collar trim -- is mostly
  rim and keeps none. About a sixth of each costume's corners are now on a
  hem; the fray is soft, about one stroke in a hundred, as P8 asks.
- **On black cloth the strokes were invisible.** Every variation inside a
  stroke multiplied the colour under it; a multiple of near-black is
  near-black. They are now made on the eye's scale with a floor (`shift`,
  `DARK` in `brushwork.wgsl`): exactly the old multiply at a third of full
  value and up, a visible step below it. Her coat now reads as brushwork.
- **A hem wisp ran up the coat.** The bodice's top edge became a hem, and a
  red tongue off it lay on the black coat -- the Champion's shoulder
  mistake again (P7). A wisp may run on into the air past the cloth, never
  onto other cloth of its own body.

Re-filmed after: the Reaver and the Champion (front, shoulder, back hem),
unchanged but for softer panel hems; the Champion's mantle still keeps
inside its colour shape. Her turntable shows no popping. Her walk was
filmed at the side-on distance the gaits are judged at, which is too far to
judge sliding on her dark coat: that one is for F6 in a running game.

## 12. Free hems past the line: what blocked it (2026-10-22)

The owner, on §11: *"I'd like to see the very extremals of the clothing be
brushed a little outside the lines to soften and give that hint of
brushwork. Free hems that affect the outline and smooth transitions should
be brushed over is the rule of thumb. You have the latter, but the former
doesn't seem to happen by default."* Then: *"you say it's WISPS. Try that
first."*

Nothing of this round is in the code; what it found, in order:

1. **`WISPS` from one in eight to one in two changed nothing visible.**
   Drawing every wisp green showed why: they were made, and then erased.
   The check that keeps a wisp off other cloth (§11) asks `ours`, whose
   tolerance is 20 to 45 cm of depth -- and the floor just behind a hem is
   that close, so it counted as her, and as another colour than her coat.
   A tight tolerance (4 to 8 cm) let them show: one in two then softened
   the skirt's hem visibly. **This fix is right and wanted.**
2. **But a tight tolerance lets the bleed back**: a wisp off the
   Champion's shoulder cloth reached the shirt, which at a slant is more
   than 8 cm behind it along the eye's line; at 10 to 15 cm the floor is
   too, and the skirt's wisps went again. Depth cannot tell the floor
   behind a hem from cloth behind an edge.
3. **Asked at placement instead** (`hem_is_free`: is any other garment
   just past this edge?) it can, and it kept the shoulder and the bodice
   clean -- but only for edges `openness` finds.
4. **Most of her skirt's bottom edge is not found.** Three of its four
   panels hang to the floor, crumpled; there the cloth's two faces have
   been merged into one edge (no 4 mm thickness left to find). Counting
   such knife edges -- two triangles facing opposite ways -- found the
   bottom, and also every crease of the crumpled cloth: green all over.
5. **The strokes near a hem are few.** They are scattered over the cloth,
   so about 27 lie within 3 cm of her whole bottom hem; laying strokes
   along every found hem at a steady step fixed the count, not the finding.
6. **Asked of the picture instead** -- a stroke facing the eye may paint
   past its cloth where what lies beyond is further off, and the cloth just
   inside is at its own depth -- either let the Champion's mantle and her
   bodice smear (lax), or refused her pleated, crumpled hem (strict).

Two rules that came out of it and stand: **a band is told from a sheet by
width** (its middle a few centimetres from its edge), not by its share of
corners on the rim, which could not tell the Reaver's leg wraps (52%) from
the Blood mage's panels (48%); and **only a free hem frays**, asked of the
mesh, not of the depth.

**The fix that would work is in the build, not the painting**: the
wardrobe knows exactly where each garment ends in the open -- a panel's
bottom and sides, a coat's hem, a cuff -- before the cloth is draped,
crumpled and squeezed. Marked there as a vertex attribute, a stroke would
know it is on a free hem without guessing from the mesh or the depth, and
could run a little past it every time. That is a rebuild of every model.
