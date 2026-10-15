---
status: exploring 2026-10-15 -- four looks built to compare, none chosen
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
build (WebGL2 cannot sample depth) there is no treatment and the picture is
the plain one. That has to change before any of this ships -- see below.

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
| `brushed` | GW2's concept art | crisp near, broad flat-brush strokes far and on every surface, sky strokes. Retuned on the owner's word (2026-10-15: *not paper texture; big areas are too busy, make the strokes much bigger for large uninterrupted areas*): strokes a fifth of their distance long, no fine marks over them, no paper frame -- so a big area is a few strokes and a small thing sits inside one |

![Hearth in plain, painted, depths and wash](gallery/painting-hearth.jpg)
![The falls in the same four](gallery/painting-falls.jpg)
![Brushed: the Mouth](gallery/painting-brushed-mouth.jpg)
![Brushed: Hearth](gallery/painting-brushed-hearth.jpg)
![Brushed: the falls](gallery/painting-brushed-falls.jpg)

## What has to be decided

1. **Which look**, or which parts of which. The knobs are independent; any
   mix is a new row in `look::wash::STYLES`.
2. **Light, not just paint.** Most of the first reference is its lighting:
   dusk, warm lamps and glowing windows against violet air. The arenas are
   lit by one sun at midday. A dusk sky per place and lamps that light their
   surroundings (point lights, or a cheap glow volume) would do more than any
   filter.
3. **The browser build.** A depth-free fallback (no haze, no edges) or a
   depth texture written by the main pass itself.
4. **What is cut.** Only `shapes::rock` is cut so far; the spires, cliffs,
   dykes and the Elementalist's stones are not.
5. **Steep faces.** A surface's strokes are laid on the plane it most
   nearly faces, so a slope at forty-five degrees gets stretched strokes (the
   Mouth's left wall). Strokes laid along the slope's own direction would fix
   it.
6. **Cost.** The Kuwahara is the expensive part (up to ~120 taps a pixel at
   radius 5). Not yet measured against the frame budget.
