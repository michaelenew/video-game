---
status: built 2026-10-15; **opt-in since 2026-10-16** (`--look brushed`), because as the default it was too expensive to play -- see "Cost" below. The look is the target; making it cheap is the open work.
---

# Painting -- how the picture is laid down

[forms.md](forms.md) decided what a thing *is* (masonry, a dyke, a log, a
rock). This decides how it is **painted**: how its light turns to shade, what
colour the shade is, how thick and what colour the air is, which edges catch
the light, and whether there is paper under it all.

The owner's brief (2026-10-14): *a watercolored, almost brushed look at colour
gradients; something that would ship in a high-budget indie game.* Then two
references, which moved the target away from literal paint-on-paper:

- a painted 3D town at dusk (chunky faceted stones with pale tops, soft
  hand-painted gradients on every face, violet shade, warm and teal lights,
  a coloured haze the distance sinks into, dark foreground rocks), and
- a flat-layered 2D scene in one hue (silhouettes, layers separated by air,
  shafts of light), and then
- **Guild Wars 2's concept art**, which is the one the owner named outright:
  *broad brushstrokes amid more classic 3D character art ... made to work
  better in 3D and in motion.* Broad flat-brush strokes, hard-sided, each its
  own colour; dry-brush ends breaking into bristles and bare paper; the
  subject crisp inside it.

## Where it lives

| Half | File | Space | What |
| --- | --- | --- | --- |
| The numbers | `look::wash` | -- | every treatment, as one table of knobs (`Wash`), `--look <name>` picks one |
| On the surface | `game/src/paint.wgsl` | world | the light in a few soft tones with a brushed border; violet shade, warm light; brush **dabs** laid in rows on every face; pigment wandering over metres; tops paler; dilution |
| In the air | `game/src/wash.wgsl` `brush` | screen + depth | the painted-surface filter (anisotropic Kuwahara); corners shaded; **edges caught and creases darkened** (from the depth); near things darkened; height haze in the sky's own colour; glow; sun shafts; the grade |
| On the paper | `game/src/wash.wgsl` `paper` | screen | the watercolour looks only: paper distortion, granulation, dry brush, paper relief |
| Broad strokes | `game/src/paint.wgsl` `broad_strokes`; `wash.wgsl` `broad`, `framed` | world; sky directions; screen | flat-brush strokes fixed to every surface and kept a constant size on screen; the sky's strokes on its own directions; the dry-brush frame |
| Stones | `game/src/shapes.rs` `cut_rock` | mesh | a rock as its box with corners sliced off by planes: flat facets, still inside the box |

The surface half is handed to `paint.wgsl` as shader defs (thousandths), chosen
once per run. The air and paper halves are a uniform on the camera, with the
camera's height, axes and lens and the sun's place on screen kept current by
`wash::follow`.

**Both passes read the depth prepass the outline asks for**, so on the browser
build (WebGL2 cannot sample depth) only the surface half is drawn -- the
strokes, their direction, the contour colour -- and none of the air.

## Brushwork that holds still

A brush laid over the screen -- the obvious way to paint a picture -- swims:
the strokes stay put while the world slides under them (the "shower door").
GW2's look in motion needs the opposite, so each kind of stroke is fixed to
what it paints:

- **On a surface**, the strokes are in world space, so they move with the
  wall. A stroke a fixed number of metres long would shrink to a pixel in the
  distance, which is backwards for painting (the distance is where the
  strokes are broadest), so the length is chosen from the distance in powers
  of two and the two nearest are cross-faded. A stroke stays about the same
  size on screen at any range, and changes scale by fading rather than
  popping. The idea is Bénard et al., *Dynamic 2D Patterns for Shading 3D
  Scenes* (2010).
- **In the sky**, the strokes are laid on the sky's own directions (azimuth
  and elevation from the camera's axes), so they turn with the world.
- **The frame** is the one thing that belongs to the screen, so it is laid on
  the screen.
- **Crisp near, painted far**: the brush filter's radius grows with distance
  (`crisp`, `painted_from`), so the fighters and the ground they are on stay
  3D and the brushwork lives behind them.

## Which way, and how freely (2026-10-15)

On the owner's word: *strokes on the path should follow the path, strokes on
a hill should point uphill; exactly parallel and uniform is right for brick or
cut stone, along its lines, and the closer to natural the more variation in
size and angle; the grass's strokes much larger, with more variation in size,
colour, shape and angle.*

So every stroke is now placed one by one (`paint.wgsl` `strokes`): a few to
each cell of a grid on the surface, each with its own place, angle, length,
width, curve, tapered dry ends, shade and two colour shifts, the topmost
covering a point being the one seen. Two things decide them:

- **Which way** (`stroke_way`): along a direction the mesh gave (a road --
  the way square to the gradient of the distance from its middle,
  `land::tile_mesh`), else uphill on a slope, along the courses on upright
  built work, and on the flat a direction that wanders slowly.
- **How freely** -- *rigidity*, 0 to 1, with a size: at 1 the strokes sit on
  the grid, parallel and alike; at 0 they are scattered, turned, sized and
  coloured freely. Carried in the mesh's second texture coordinates
  (`land::ground_stroke`, `shapes::lay_natural`), read off the face where a
  mesh says nothing: upright is built, flat is open ground.

| Surface | Rigidity | Size |
| --- | --- | --- |
| Grass, peat | 0 | 2.4 |
| Earth, sand, snow, ash | 0.2 | 1.6 |
| Bare rock, cliffs, boulders | 0.25 | 1.1 to 1.3 |
| A road | 0.45 (along it) | 1.0 |
| Water | 0.5 | 1.8 |
| Timber | 0.8 | 0.7 |
| Dressed stone | 0.9 | 0.6 |
| Unmarked, upright (walls, plaster) | 0.85 | 0.7 |

## Colour that follows the land (2026-10-15)

On the owner's word: *add some blue and red -- in the colour-theory sense,
like salt in cookies -- in a smooth overlay that follows the contour of the
terrain, so deeper colours bring out its curves.*

The land's **bend** (`land::bend`) is its curvature: how far the ground round
a point stands above or below it, at eight metres and at twenty-four, squashed
to -1 on a crest and 1 in a hollow. It is measured on a 4 m lattice when a tile
is built (nine height samples a lattice point; per vertex it kept the
mountains from streaming in) and rides in the last component of each vertex's
tangent. `paint.wgsl` multiplies a hollow's colour toward `hollow` (deeper,
cooler: blue-violet) and a crest's toward `crest` (warmer: red-orange), by
`rich`. The bend is smooth over metres, so the tint is.
`land::bend_tests::the_land_bends_by_degrees` holds the scale: the first one
was ten times too strong and tinted all the land alike.

**Deeper, and on stone too** (the owner, the same day). The tint is
multiplied and also a little *added*, since a multiple of nearly black is
still nearly black. Hollows go toward a deep violet-tinged shade
(`[0.68, 0.64, 0.96]`), crests toward red-gold (`[1.24, 0.97, 0.78]`); a
first, bluer hollow turned the Commons -- a floor in a bowl of rim -- blue
all over, so the narrow scale now leads the broad (0.7 to 0.3).

Rocks, cliffs, standing stones, spires, dry-stone walls, floating islands and
basalt are not height fields, so their bend is their own (`shapes::bend_of`):
corners welded, each point's neighbours' middle measured along its normal
against the stone's own usual bend (a boulder is convex all over, so a ridge
is *sharper than usual*), smoothed over the surface, plus the painter's habit
for stone -- warm toward the top, deeper toward the foot and the undersides,
which is all a flat facet has. `shapes::tests::a_stone_bends_by_degrees`
holds it. The Waterfall's basalt barely shows it: that rock is nearly black
and in its own shadow, which wants a palette or a light, not seasoning.

## Built on open source

| From | What we took | Licence |
| --- | --- | --- |
| [MNPR](https://github.com/semontesdeoca/MNPR) (Montesdeoca et al., Expressive 2018) | the watercolour model: colour as transmittance to a pigment density, edge darkening, pigment turbulence, pigment application over a paper heightmap (granulation, dry brush), paper distortion, paper relief, dilution; and **the paper scan** (`paper.jpg`) | MIT |
| [Acerola's Post-Processing](https://github.com/GarrettGunnell/Post-Processing) (after Kyprianidis et al., GPU Pro 2010) | the anisotropic Kuwahara filter, eight sectors; its structure tensor gathered in place rather than in three passes | MIT |

Notices are in `crates/game/THIRD_PARTY.md`.

## The candidates

Run any of them with `--look <name>`; shots with
`DEMO=0 SHOT_FRAME=150 SHOT_PITCH=0.1 SHOT_YAW=<y> GAME_ARGS="<place> --look <name>" ./scripts/screenshot.sh`
and a wait of 100 s under the software renderer (the land streams by frame,
and a slow frame loads fewer tiles -- a 45 s wait leaves mountains out).

| Look | After | In a line |
| --- | --- | --- |
| `plain` | -- | as it was |
| `painted` | reference one | soft painted tones, violet shade, warm light, brush dabs, caught edges, coloured height haze, glow, cut stones, almost no line |
| `depths` | reference two | the same pushed to one hue (blue) and to layers: thick air, dark foreground, strong shafts |
| `wash` | the first brief | transparent watercolour on paper: pooled edges, granulation, dry brush, wavering sepia line |
| `brushed` (the target; opt-in) | GW2's concept art | crisp near, broad flat-brush strokes far and on every surface, sky strokes. Retuned on the owner's word (2026-10-15: *not paper texture; big areas are too busy, make the strokes much bigger for large uninterrupted areas*): strokes a fifth of their distance long, no fine marks over them, no paper frame -- so a big area is a few strokes and a small thing sits inside one |

![Hearth in plain, painted, depths and wash](gallery/painting-hearth.jpg)
![The falls in the same four](gallery/painting-falls.jpg)
![Brushed: the Mouth](gallery/painting-brushed-mouth.jpg)
![Brushed: Hearth](gallery/painting-brushed-hearth.jpg)
![Brushed: the falls](gallery/painting-brushed-falls.jpg)

## How it was made, in order

The look was found by building candidates in the real renderer and showing the
owner screenshots of the same frames, not mockups. Each step below is a round:
what was asked, what was built, what the owner said.

1. **Brief: "watercoloured, almost brushed at colour gradients".** First pass:
   a surface half (light in soft tones with a brushed border, cool shade, warm
   light, wandering pigment) and a screen half (pigment pooling at edges,
   colour bleed, paper grain, a paper-coloured vignette, a wavering sepia
   line), as three presets. Too dark, muddy, smoky edges.
2. **Owner: "find open source and start from there".** Ported MNPR's
   watercolour chain and its paper scan, and Acerola's anisotropic Kuwahara.
   The `wash` look is that, and it is a fair watercolour.
3. **Owner sends references** (a painted dusk town; a one-hue layered 2D
   scene). The target was never paint on paper: it was *painted surfaces,
   coloured light, atmospheric depth*. Built `painted` and `depths`: height
   haze in the sky's own colour, near things darkened, contact shading from
   depth, glow, sun shafts, a split-tone grade, edges caught and creases
   darkened from the depth, brush dabs on surfaces, cut stones.
4. **Owner: "GW2's brush aesthetic", then "like this but working in 3D and in
   motion".** Built `brushed`: crisp near and painted far, broad flat-brush
   strokes fixed to the world at a constant screen size (Bénard et al. 2010),
   sky strokes on the sky's directions, a dry-brush frame.
5. **Owner: "I don't mean paper texture; big areas are too busy, make the
   strokes much bigger".** Frame off, strokes three times larger, all fine
   marks over them removed. *This was the turn*: the look is a few broad
   strokes over large areas and clean small things, not texture everywhere.
6. **Owner: strokes should follow the path and point uphill; rigid for
   brick, varied for nature; grass much bigger and freer.** Strokes placed one
   by one with a direction field and a rigidity per surface.
7. **Owner: "a pinch of blue and red, following the contours".** Colour that
   follows the land's curvature; then deeper, and on rocks and cliffs.

## What did not work, and why

Kept so nobody walks into the same wall twice. Most of these cost a round.

**Look and direction**

- **Literal watercolour on paper** (paper grain, granulation, pooled edges,
  a paper vignette, sepia line). Technically sound -- it is MNPR -- but it was
  not what the owner meant by "watercolour". He meant painterly colour
  gradients, and paper read as a filter over a game. It survives as `wash`.
- **The paper-coloured vignette** read as smoke or fog at the edges, at every
  setting tried; a hard ragged edge read as damage. Dropped.
- **The dry-brush frame** (the picture's edges breaking into bare paper, after
  the GW2 concept art): the first version's bristles were two axis-aligned
  noises and read as a glitch grid; drawn before the outline, the outline was
  drawn over the bare paper. Fixed both, and the owner still did not want
  paper. Off (`frame: 0`), code kept.
- **Uniform filters** (`painted`, `depths`): they recolour the scene but the
  surfaces still read as flat CG fills. A filter cannot invent brushwork the
  geometry does not have; the strokes had to be on the surfaces.
- **Small dabs everywhere** (`grain`): reads as camouflage or stucco, "too
  busy". The rule that came out of it: *stroke size follows the size of the
  uninterrupted area*, and fine marks only where something is genuinely
  detailed.

**Technique**

- **Screen-space strokes swim.** Any brush laid over the screen stays put
  while the world slides under it. Strokes are in world space on surfaces and
  in direction space on the sky; only a frame may be screen-space.
- **World-space strokes of a fixed size** shrink to noise in the distance,
  which is backwards for painting. Size from distance in powers of two,
  cross-faded (and the fade loses contrast mid-way: divided back by
  `sqrt(t^2 + (1-t)^2)`).
- **Rows of strokes** (the first flat-brush): nearly square cells read as a
  mosaic, "Minecraft". Long thin strokes (5:1) fixed that, but rows still
  look combed; individually placed strokes with jitter scaled by rigidity are
  what reads as painted.
- **Swirly brush marks** came from turning the stroke direction by a noise of
  the same position; iso-lines curl into wood grain. Turn a little, slowly.
- **Rotating the stroke frame per pixel by a direction field** scrambles
  strokes far from the origin (a small turn at 500 m shifts the grid
  metres). The grid is fixed to the plane the surface faces; only each
  stroke's angle follows the field.
- **Banded light ("tones") with a hard border** looked like toon shading,
  not paint; it needs a soft border pushed about by a brush noise, and at
  `brushed`'s strength it is mostly a soft gradient.
- **Edges caught from the depth** first lit every fold between two triangles
  of the land; thresholds had to rise (0.002 to 0.008 relative depth). On
  cobbles the effect at 0.9 made them glassy; 0.25 in `brushed`.
- **The haze** at first sampled the sky by "depth is zero" -- but the sky is a
  dome mesh 480 m out, so it found nothing and the air went pure tint
  (violet). The sky is "past the dome" now. Its first density hid the
  mountains completely.
- **The curvature scale** for the contour colour was first ten times too
  strong: pinned at plus or minus one almost everywhere, a flat tint. And the
  24 m scale reads a whole arena floor in its rim as one hollow, which turned
  the Commons blue all over: the 8 m scale leads now and the hollow colour is
  deep violet-grey rather than blue. Both are held by tests.
- **A convex stone measured absolutely** is all ridge. Measured against its
  own median bend, it has ridges and hollows.
- **Multiplying a tint into a near-black rock** changes nothing; the contour
  colour adds a little as well. The Waterfall's basalt is still too dark to
  season: that wants a lighter rock or light on it, not more tint.

**Engineering traps**

- **A uniform whose Rust and WGSL layouts differ** draws a black screen with
  no error (a `Vec3` pad aligns to 16 bytes in `encase`). The uniform is all
  `Vec4`s now.
- **A shader that fails to compile is silent** in this game (no log plugin):
  the picture just goes black or a thing vanishes. `cargo run --release -p
  game --features bevy/bevy_log` prints the error; otherwise, look at a
  screenshot after every shader change.
- **Per-vertex height sampling** (nine samples a vertex for the curvature)
  slowed land streaming so the mountains never loaded; a 4 m lattice per
  tile, read between, costs forty times less.
- **Captures under the software renderer** (`lavapipe`) take 100 s and more;
  the land streams by *frame*, so a heavier look loads fewer tiles in the same
  wait and mountains go missing. Use `WAIT=300` for the valley's wide shots.
  It is not a bug in the look; on a GPU there are many frames a second.

## Picking it back up

- **Run it**: `--look brushed` (the game is `plain` by default);
  `painted|depths|wash` are the others, `?look=` on the web.
- **Take shots**: `DEMO=0 SHOT_FRAME=150 SHOT_PITCH=0.1 SHOT_YAW=<y>
  GAME_ARGS="<args> --look <name>" ./scripts/screenshot.sh <out.png> <wait>`.
  The frames used throughout: Hearth (`SHOT_YAW=0.6`, no args) for
  buildings; the Mouth (`--arena mouth`, wait 300) for grass, path and
  mountains; the Waterfall (`--arena pinewood --open`, `SHOT_AT=772,-4`) for
  cliffs; the Commons (`--arena commons`) for standing stones and a rim.
- **Turn a knob**: every number is a field of `look::wash::Wash`, one row per
  look. The surface ones reach `paint.wgsl` as shader defs (thousandths),
  the rest as the camera's `wash::Wash` uniform.
- **Say what a surface is**: `land::ground_stroke` (rigidity and size by
  ground material), `shapes::lay_natural` and `shapes::lay_bend` (a mesh of
  natural stone), and for a direction, the first texture coordinates.
  Unmarked meshes are read by their faces: upright is built, flat is open.

### Open, in rough order of what would help most

1. **Light.** Most of the dusk-town reference is its lighting: warm lamps and
   glowing windows against violet air. The places are lit by one midday sun.
   A dusk sky per place and lamps that light their surroundings would do more
   than any further brushwork.
2. **The Waterfall cliff** and other dark basalt: a lighter, warmer rock so the
   contour colour and the strokes can work on it, and less of a box grid.
3. **Brick and paving locked to their joints**: strokes on masonry are level
   but not aligned to the actual courses; the town's paving is not marked as
   dressed stone yet and is brushed as open ground.
4. **Steep land**: strokes are laid on the plane a surface most nearly faces,
   so a 45-degree slope gets stretched strokes in the projection.
5. **The browser build** draws the surface half only (the passes need the
   depth prepass WebGL2 cannot sample). A depth-free fallback, or depth
   written by the main pass.
6. **Cost**: the Kuwahara (up to ~150 taps a pixel at radius 7 in the
   distance) and the stroke placement (36 strokes tested a pixel on painted
   surfaces) are not yet measured against the frame budget on a GPU.
7. **In motion**: everything is built to hold still under a moving camera,
   but it has only been judged in stills.
8. **Characters** are untouched by the surface half (they are not drawn in
   `Paint`) and only lightly by the passes. GW2's subject-crisp-in-a-painting
   is the intent, but the fighters may want a painted treatment of their own.
