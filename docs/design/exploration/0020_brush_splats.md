# 0020 — The brush as a primitive: strokes at the cost of nothing

**Thread.** The brushed look is the painting the owner wants (the brush
worked out at the pixel, [#165](https://github.com/michaelenew/video-game/pull/165)),
and every cheaper version tried in two days (tiling sheets, a texture baked
per tile) read as *a computer splotching flat shapes on the ground*. The
owner's brief: industry engines draw far richer pictures without the cost,
and bake what they bake exactly; find how, and get this picture for
almost nothing. This note is the research and a proposal.

**The short answer.** Every real-time painterly system that has shipped or
been published keeps the brush the same size on the *screen* and anchored
to the *world*, and none of them does it by asking every pixel which of a
hundred strokes it is inside. They make the stroke a **primitive**: either
a texture built so that hardware mipmapping keeps strokes at one pixel
width at every distance (Praun's tonal art maps, 2001), or a **particle**
placed on the surface once and drawn as a small screen-sized quad every
frame (Meier 1996; Hanson and Todd 2014, 250,000 strokes at 60 fps in a
browser on a 2010 card; Aardman's *11-11 Memories Retold*, 2018). The
second is the one that gives *exactly* this picture, and it is what this
note proposes: bake the stroke *placement* per tile (a few thousand
numbers, microseconds), draw the strokes as quads sized in pixels, and let
the depth test resolve which is on top. The surface shader then does no
stroke work at all.

## 1. What the owner's eye is reading

Three attempts, one verdict each, and they agree on a diagnosis:

| Version | What the owner said | What differed from the original |
| --- | --- | --- |
| Tiling sheets (#162) | "simpler and flatter" | six angles, sixty-four texels a stroke, two unrelated stroke layers blended, a hard switch between angle sheets |
| A texture baked per tile (#164) | "a computer splotched flat shapes onto the ground"; "crisper and flatter, and that's a bad thing" | strokes fixed to the world: they shrink with distance |
| The brush at the pixel (#165, the original) | "looks like a painter painted it" | -- |

The original has two properties the others lose. **The brush is the same
size on the screen everywhere**, because `broad_strokes` scales each
stroke by its distance from the eye: a stroke ten metres away is as wide
on the screen as one at the feet, as a brush is the same brush across a
canvas. And **the strokes are anchored to the world**: they do not slide
when the camera moves. The literature has a name for losing the first
(a decal, a texture: *resolution coherence* lost) and for losing the second
(the **shower-door effect**, Meier 1996: strokes fixed to the glass the
scene is seen through). A painting needs both, and that pair is the whole
problem; it is why the texture bake could never have worked at any
resolution, and why the "two bake sizes" idea, which was right in spirit,
has to be done with primitives rather than texels.

## 2. What the industry does

**Tonal art maps** (Praun, Hoppe, Webb, Finkelstein, *Real-Time Hatching*,
SIGGRAPH 2001; [paper](https://gfx.cs.princeton.edu/proj/hatching/hatching.pdf),
[project](https://gfx.cs.princeton.edu/gfx/pubs/Praun_2001_RH/index.php)).
The texture answer to constant screen-size strokes, and the one piece of
settled theory here. "One expects the strokes to have roughly the same
screen-space width when representing both near and far objects. Thus,
when magnifying an object, we would like to see more strokes appear (so as
to maintain constant tone), whereas ordinary texture mapping simply makes
existing strokes larger." Their fix: build the mip levels by hand so that
**strokes have the same pixel width at every level**, and finer levels
*add* strokes to fill the gaps the coarser ones leave -- every stroke in a
coarse level appears in all finer ones (the *nesting property*). Hardware
mip selection then does the distance scaling, and as the eye approaches
"existing strokes persist while a small number of new strokes fade in".
Direction over the surface comes from **lapped textures** (Praun 2000): the
texture is pasted as overlapping patches each turned to a direction
field, never by turning a lookup (what smeared #162's grass). 20--40 fps
on a 933 MHz Pentium with a GeForce2.

What this says about the sheets: the concept was right (a stroke texture
scaled by distance) and the execution was not. Our two octaves are two
*unrelated* stroke sets blended, where TAM makes the coarse set a *subset*
of the fine one, so nothing fades but new strokes; and we read six turned
sheets with a hard phase switch where lapped textures paste patches along
the field. A TAM done properly would be a legitimate texture route. It is
not the one proposed, because it still cannot match the original's *per
stroke* bristles and dry ends at the feet (a texel is a texel), and the
particle route below gets everything for a comparable cost.

**Stroke particles** (Meier, *Painterly Rendering for Animation*,
SIGGRAPH 1996, [paper](https://www.cs.princeton.edu/courses/archive/fall00/cs597b/papers/meier96.pdf)).
The classical answer and the one every painterly game since has used in
some form: distribute particles over the surface once, and each frame
draw every particle as a **2D brush stroke sized in screen space** at its
projected position, coloured and oriented from the surface under it.
Anchored to the world, sized on the canvas; Meier names the shower-door
effect and this is her cure. Hanson and Todd ([*Object-Space Painterly
Rendering for WebGL*, 2014](https://www.cs.rpi.edu/~cutler/classes/advancedgraphics/S14/final_projects/scott_andy.pdf))
put it on the GPU without sorting: strokes are point sprites, drawn in a
fixed order per layer, orientation and curvature done "using image
transformations in the fragment shader", occlusion by comparing each
stroke's depth to a depth texture and fading it with the difference; three
layers, coarse to fine; **60 fps with 250,000 strokes at 1000 x 500 on a
Radeon HD 5770 in a browser**. That card is fifteen years old.

**Full-screen stroke particles** (*11-11 Memories Retold*, Aardman /
DigixArt, Unity, 2018; Alexander Birke, GDC 2019 [*A Living Painting*](https://www.gdcvault.com/play/1026102/A-Living-Painting-The-Rendering);
[Engadget](https://www.engadget.com/2018-11-15-11-11-memories-retold-paint-art-aardman.html)).
The scene is rendered normally plus a layer carrying "the size, type and
direction of each brushstroke"; a full-screen particle system then paints
strokes over it, placed by the detail a scene needs and by motion,
composited depth-aware "so that foreground objects are never obscured by
brushstrokes depicting elements in the background". Thousands of strokes a
frame on PS4 and Xbox One; they simplified the models underneath ("PS1/PS2
era") because the strokes carry the detail. The one shipped game that is
*exactly* a painting in motion, and its cost was one particle pass.

**Splats** (*Dreams*, Media Molecule, PS4; Alex Evans, *Learning from
Failure*, SIGGRAPH 2015, [slides](https://www.slideshare.net/slideshow/umbra-ignite-2015-alex-evans-learning-from-failure-prototypes-rd-iteration-and-arriving-at-great-by-throwing-away-lots-of-almostgreat/52801357)).
Sculpts become dense point clouds and the renderer "selectively splats
strokes on top" of a coarse rasterised base; the whole look is strokes as
primitives. Far heavier than this game needs (it is the *geometry* too),
but it is the ceiling of the same idea.

**Hand-painted textures** (*Guild Wars 2*, Blizzard, Riot). The look this
game's brushed look is named after is, in those games, artists painting
strokes into albedo textures, mipmapped by the engine. Those strokes are
fixed to the world and shrink with distance; the games read as painted
because the paintings are good, and the camera rarely magnifies ground
five-fold. Not a technique available to a game with no artist, and not
one that keeps the brush on the canvas.

**Unreal** has no painterly mode. Its stylised looks are post-process
materials (Kuwahara and edge filters, as this game's screen pass) over
surfaces whose stroke detail is painted or projected in world space, with
near and far texture scales blended by distance in the landscape material
(the UE docs: "transitioning from detail textures to larger macro textures
depending on the distance"). That is the sheets' two octaves. Its one
general lesson is the one above: strokes that must stay the same size on
the screen are particles or custom mips, never per-pixel searches.

**The screen pass.** The anisotropic Kuwahara (Kyprianidis, Kang, Döllner
2009, [paper](https://umsl.edu/~kangh/Papers/kang_cgf09.pdf); the 2010
polynomial-weight version this game uses) was real-time on 2009 hardware;
the 49-tap version here is about 0.3--1 ms at 1080p on anything current.
It is not the cost to chase, though it can run at half resolution for a
quarter of that.

## 3. Where the cost actually is

Under the software renderer, which is the only GPU this session has (the
Mouth, 1280 x 760, [painting.md](../painting.md)):

| | plain | sheets | brush at the pixel |
| --- | --- | --- | --- |
| surface pass | 98 ms | 119 ms | 254 ms |
| screen pass + outline | -- | 278 ms | 278 ms |
| whole frame | 294 ms | 556 ms | 691 ms |

Two things to hold on to. The brush at the pixel costs **a hundred strokes
a pixel**: two layers of a five-by-five cell search with two strokes a
cell, each with hashes, a rotation and a noise lookup. The splat route
replaces that with **one texture lookup a pixel, a few times over** (the
overdraw of overlapping quads). And the freeze that made the look feel
unplayable was the region ledger ([#163](https://github.com/michaelenew/video-game/pull/163)),
not the paint. **Nothing here has been measured on a real GPU.** The first
action is the owner running `cargo run --release -p game -- --profile
--look brushed` and `--look plain` and pasting the readouts: a 254 ms
pass on lavapipe may be 2 ms on an M-series Mac and the whole question
moot, or 15 ms and worth every line below.

## 4. The proposal: strokes as splats

Keep the painting; change what a stroke *is* at runtime.

**Bake.** Per tile of land, when it streams in (`stream.rs` already owns
the tile's lifetime), place the strokes once on the CPU: for each cell of
each level (level `L` is strokes `2^L` metres long, two a cell, the same
hashes and the same rule as `strokes_exact` -- centre, angle off the
direction field, length, width, curve, order, the three colour axes, the
two bristle seeds). That is *already* what `look::strokes::Brush::mark`
computed in #164; it is a few hundred bytes of arithmetic a stroke and
there are a few thousand strokes a tile: **microseconds**, not the 0.65 s
the texture bake took, because nothing is rasterised. Levels are nested
as a tonal art map is: level `L+1` is level `L` plus the strokes between.
Store each stroke as 16--24 bytes in one GPU buffer per tile; ten
thousand strokes a tile at the finest level, far fewer at the coarse
levels distant tiles get. A tile at 300 m needs only its metre-long
strokes and up; the finest levels exist only near the eye, as the mesh's
`cells_at` already decides fineness by distance.

**Draw.** One instanced draw per tile, after the surface pass, with the
depth buffer bound: each stroke is a quad **lying on the ground** (not a
billboard: it lies in the surface's plane, turned to its angle, so it
foreshortens and is occluded like ground) whose **world size is set in
the vertex shader from its distance to the eye** -- exactly
`broad_strokes`' `BROAD_SIZE * dist * size`, so the brush is the same
size on the screen. The vertex shader also decides whether this stroke is
*in season*: its level against the level the distance asks for, fading
the strokes of the level coming in as the eye approaches, the way TAM
fades new strokes in (and the original's `mix(one, two, t)` between
octaves is this fade, read the other way). The fragment shader is the
original's inner loop for *one* stroke: `u`, `v`, the taper, the bristle
noise at the pixel, `inside` or discard, the colour axes -- so a stroke's
edge is worked out at the pixel, which is where the crispness and the dry
ends live. Which stroke is on top: the original keeps the highest `order`;
here each stroke nudges its depth toward the eye by its order (a few
millimetres of range), and the depth test keeps the topmost, in any draw
order, no sorting.

**Compose.** The stroke writes what `broad_strokes` wrote: not a colour
but the shade and the two colour shifts, which the surface shader applied
to the lit base. So the splat pass reads the surface pass's colour under
it (the pass structure `wash.rs` already has: read one target, write the
next) and writes the base shifted by the stroke, or, simpler and probably
right, the surface pass writes the base and the splat pass multiplies the
stroke's shift into it with blending. The screen pass runs after, as now.

**What stays per pixel.** Nothing of the brush. The surface shader loses
`strokes`, `broad_strokes`, the sheets and their texture array; it keeps
the mottle, the grain and the contour tint, which are a noise lookup
each. The splat pass costs about `(screen area / stroke area) x overdraw`
fragments, each one noise lookup and a dozen flops: roughly 2--4x the
pixels of the screen, against the hundred stroke tests a pixel now. On
the software renderer that is a surface pass back near `plain`'s 98 ms
plus a few tens for the splats; on a GPU the splats are a few hundred
microseconds.

**What the picture loses.** Within the rule as stated, nothing: the same
strokes in the same places with the same shapes, bristles and colours,
topmost by the same order, the same size on the screen. Two differences
to expect and look at: a stroke that overhangs a cliff edge is drawn in
the air beside it (the original clipped it to the surface; clip the quad
to the ground's depth within a tolerance); and the level fade is a fade of
whole strokes rather than a blend of two layers, which is TAM's behaviour
and arguably the better one.

**Everything standing still.** Walls, roofs, stones and trees get the
same treatment from the same code with their own surface plane: a box
face is a tile with a different normal, and `forms.rs` already knows what
each box is. First the land, where the picture lives; then the town.

## 5. Plan

1. **Measure on the owner's machine** (`--profile`, both looks). Decides
   whether 2--6 happen now or go in the drawer.
2. **Placement bake**: `look::strokes::place(tile, level) -> Vec<Stroke>`
   from `Brush::mark`'s arithmetic (pure, tested against `strokes_exact`'s
   choices: the same stroke must be topmost at a sample of points).
3. **The splat pass**: a render-graph node after the surface pass in the
   style of `wash.rs`, one instanced draw per tile buffer, the depth
   buffer read for the test and the nudge. Land only, no fade: look.
4. **Size on the screen and the level fade** in the vertex shader; the
   surface shader's brush removed under `--look brushed`; the sheets kept
   under `--exact 0` for the browser until the splats run there (they
   should: instancing is WebGL2).
5. **Compose and compare**: the same shots as #165's, the owner's eye.
6. **The town and the forms**, then the sky's strokes the same way.

## 5a. Built, 2026-10-16

Steps 2--5 the same day: `look::strokes::place`, `crates/game/src/splats.rs`
and `splats.wgsl`, the two parity textures, the fragment-side depth test and
season. The picture is the brush at the pixel's to 3.7% RMS, at stroke
edges. Under the software renderer the surface pass fell from 144 ms to 45
and the splat pass costs 260 ms for 4.3 million fragments, which is the
software renderer's price per fragment, not a GPU's. The owner's Mac:
`plain` 23 ms, `brushed` 36--39 ms before the splats; Metal gives no
per-pass times, so the next measurement is his. The dead ends are in
[painting.md](../painting.md).

## 6. Open

- **SPLAT-1** Real numbers. The whole brief rests on a GPU none of this
  has run on.
- **SPLAT-2** Overdraw at the feet: strokes 150 px across overlap five
  deep. Fine on a GPU; the software renderer will show it. Depth-nudge
  with early depth test handles most of it if the nearest level is drawn
  first.
- **SPLAT-3** Blending the level fade with a depth nudge in arbitrary
  order is not exact for the few frames a stroke is half in; TAM lived
  with it. If it shows, draw the incoming level after the rest.
- **SPLAT-4** The direction field at a stroke's centre only (not per
  pixel): the original turned per pixel too, by `stroke_way` at `p`;
  a stroke a metre long under a field that wanders over 38 m will not
  tell.
- **SPLAT-5** Memory and bake for the finest level near the eye: ten
  thousand strokes a tile, a few tiles; the texture bake's five megabytes
  a tile becomes two hundred kilobytes.
