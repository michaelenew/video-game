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
