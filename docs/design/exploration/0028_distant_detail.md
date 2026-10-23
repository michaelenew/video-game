# 0028 — Distant detail: what is drawn far away, and how other games make it cheap

The owner's question (2026-10-21): another thread implied that a very distant
tree is drawn — and held in memory — as the whole tree, and that this may be
where much of the memory and CPU goes. What does the game do today about far
terrain and far things, how do other games cut that cost without the far
world looking worse, and what should we do?

Short answer: **the land and the stone walls already get simpler with
distance; the Great Wood's giant trees do not.** Each giant is one mesh of
55,000–100,000 triangles, drawn at that full detail anywhere within 600 m,
and once built it stays in memory for the rest of the run, twice over. That
is the biggest single gap. The rest of the far world is cheap in triangles
but expensive in *count* — thousands of small separately-drawn pieces.

## 1. What is done today

Everything below is `crates/game/src/stream.rs` unless named otherwise.

**Loading by distance (streaming).** In the valley only what is within the
sky's reach of the camera exists at all: 160–600 m for places, boxes and
giants, 300–700 m for land tiles (`REACH`, `LAND_REACH`). Past that the fog is
already the horizon's colour, so nothing is seen arriving. Things are dropped
60 m further out (`SLACK`) so the edge does not flicker, built nearest first,
and capped at about 3 ms of building a frame (`BUILD_BUDGET_MS`).

**Level of detail (LOD) — drawing a simpler version of a thing when it is
far, because its detail would be smaller than a pixel anyway:**

| Thing | Near | Far | Where |
| --- | --- | --- | --- |
| Land tiles (16 m squares) | a vertex every metre (< 90 m) | every 2, 4, then 8 m (< 200, < 380, beyond) | `land::cells_at`; skirts hide cracks between levels |
| Masonry walls | full relief, every block proud of the wall (< 60 m) | the same colours, flat | `NEAR`, `forms::build`'s `near` |
| Small trees (the ordinary woods) | three shared meshes — trunk, cone, crown | same | `land::Trees`; cheap because one mesh is reused for every tree |
| Brush strokes (painted looks) | metre strokes | strokes up to 64 m | `splats.rs` picks the stroke size by distance |
| **Grown giants** (`grove.rs`, 14 trees) | **full skeleton** | **full skeleton** | `stream::groves` — no far version |
| Every other box | one mesh | same mesh | `draw_solid` |

**Shadows** are Bevy's default: four cascades out to 150 m (one cascade in
WebGL 2). So far things do not draw into the shadow maps; that is already
right, and was the subject of an earlier fix (`profile.rs`'s census, "the
valley's walls ... three million triangles into four shadow cascades").

### The giants, measured

Counted from the simulation's own tree skeletons (`Atlas::trees`) using the
same rules `grove::meshes` builds them by — sides per branch by thickness,
four- to eight-triangle leaf clumps, the shelves and roofs:

| Tree | Kind | Branch nodes | Leaf clumps | ≈ Triangles |
| --- | --- | ---: | ---: | ---: |
| 0 | Sentinel | 2,590 | 769 | 100,000 |
| 12 | Strangler | 1,842 | 346 | 92,000 |
| 8 | Elder | 1,755 | 526 | 85,000 |
| … 11 more | | | | 56,000–80,000 each |
| **All 14** | | | | **≈ 1.05 million** |

This is an estimate from counting, not a reading from the running game
(the game was not launched for this note). What follows from it:

- **Every vertex is unshared** (`Kit::poly` pushes its own corners so each
  face can be flat-shaded), at 40 bytes each (position, normal, colour). A
  million triangles is roughly 2.5 million vertices: **on the order of 100 MB
  of mesh**, plus about 12 MB of indices.
- **It is held twice.** `Kit::build` uses `RenderAssetUsages::default()`,
  which keeps a copy in ordinary memory as well as the one on the graphics
  card. So ≈ 200 MB for the Wood alone.
- **It is never let go.** `Stream::grown` caches each giant's meshes "built
  once ... and kept", even after the tree itself is dropped for being out of
  reach. Walk through the Wood once and all fourteen stay resident for the
  session. In the browser this is worse: memory a WebAssembly program frees
  is not handed back to the system, so the high-water mark is what the tab
  costs.
- **A giant 500 m away is drawn whole.** At 1080p a pixel at 500 m is about
  half a metre across; a twig under 18 cm thick — most of a tree's branches
  by count — is a fraction of a pixel there. Those triangles cost time to
  draw and shimmer as they flicker in and out of the pixel grid.

### The other cost: how many pieces

The land within 700 m is up to about **6,000 tiles**, each its own mesh and
its own entity (the thing Bevy tracks and culls each frame), plus a water
plane where there is water. Every box within reach of the 13,500 on the map
is another entity. Far-off tiles are tiny (nine vertices at 8 m spacing), so
the triangles are cheap — but each piece is still extracted, culled and, in
WebGL 2 (which has no way to send many meshes in one call), drawn on its own.
That cost lands on the processor, not the graphics card, which matches the
"high CPU" half of the report.

## 2. How other games handle it

Every technique here is answering the same question: *what is the least
geometry that draws the same pixels?* They differ in what "the same" is
allowed to mean.

**Discrete LOD chains.** The oldest and still most common: an artist (or a
simplification tool) makes three or four versions of a model, and the engine
swaps them at set distances. The visible cost is the *pop* when a level
changes. Hidden today by **dithered crossfade**: for a few metres both levels
are drawn, each with a checkerboard of pixels thrown away, the pattern
shifting from one to the other. Bevy has exactly this built in as
`VisibilityRange` (it calls it "hierarchical LOD"; it works on WebGL 2
through a fallback), as does Godot.

**Trees specifically: shrink the small parts, then swap for a picture.**
SpeedTree, the tree middleware behind most big games, fades a tree in three
stages. Small branches are *shrunk toward their centre line* rather than
removed, so they vanish smoothly; leaves become *fewer, larger clumps* —
some shrink away while the rest grow to cover them; and finally the whole
tree is replaced by a **billboard**: a picture of the tree drawn on a flat
card that faces the camera. Unity's terrain blends the 3D tree into the
billboard over a transition zone. Horizon Zero Dawn's vegetation talk
(GDC 2018) describes the same chain ending in crossed cards at LOD 4–5, and
notes that tying the shadow's LOD to distance caused visible popping — a
warning worth keeping.

**Impostors** are the grown-up billboard. A single picture looks wrong as you
walk round it, so instead you photograph the tree from many directions ahead
of time and pack the pictures into one texture; at run time the card shows
whichever photograph is closest to your viewing direction, blending its
neighbours. The **octahedral impostor** (Ryan Brucks, at Epic, now a UE5
plug-in) arranges those directions by folding a sphere into a square, which
spaces the views evenly with no seam at the top. A *hemi*-octahedral one
photographs only the upper half — twice the detail for the same texture,
right for anything you never see from below.

**Hierarchical LOD (HLOD) — merging many far things into one.** Unreal's
World Partition groups whole cells of the world, far away, into a single
merged and simplified mesh with one material. The point is not fewer
triangles but **fewer pieces**: one draw instead of hundreds. Their layers
chain, each switching at its own distance.

**Terrain: a tree of tiles, not a grid of them.** CDLOD (Strugar, 2010) and
geometry clipmaps lay the ground as a quadtree: near the camera small,
fine tiles; further out each tile covers four times the area at the same
vertex count. The number of pieces stays roughly constant however far you
see. CDLOD also *morphs* each vertex smoothly between levels rather than
using skirts, so there is no pop and no seam.

**Virtual geometry (Nanite).** Unreal 5 breaks every mesh into small clusters
with simplified parents, and the graphics card picks, per frame, the coarsest
clusters whose error is under a pixel. It is the most general answer, and
Bevy has an experimental version (meshlets) — but it needs compute shaders
the browser build does not have. Not for us yet.

**What protects the look in all of these:** the switch is decided by *size
on the screen* (how many pixels of error), not by metres; levels are faded,
not popped; the far version is made *from* the near one, so colours and
silhouette match; and anything the fog already eats is not drawn at all.

## 3. Where this game is unusually well placed

Most games have to *author* or *bake* their far versions. Ours are generated,
and generated from structure that already says what matters at a distance.

**A giant is a skeleton built by the pipe model** — the rule (trees really
follow it, roughly) that a branch's cross-section equals the sum of its
children's. That gives a LOD for free: *prune every branch thinner than what
would cover half a pixel at this distance, and let its parent stand for it.*
The parent already carries the pruned branch's thickness, so the silhouette
does not thin out. Leaves cluster the same way: a pruned limb's clumps merge
into one clump at their middle, sized to cover their combined volume — which
is SpeedTree's "fewer, larger leaves", derived instead of authored. Colours
come from the same palette calls, so near and far match exactly.

Because the far version is a pure function of the skeleton and a size
threshold, it obeys the repository's own rule for looks: derive it, look at
it on a sheet, tweak, fold the tweak back into the derivation.

Rough budget, with a half-pixel threshold at 1080p:

| Level | From | Kept | ≈ Triangles a tree |
| --- | --- | --- | ---: |
| Full | 0 m | everything | 55k–100k |
| Pruned | ~120 m | branches thicker than ~10 cm; leaves merged per limb | ~10k |
| Far | ~300 m | trunk, major limbs, pads; crown as a dozen big clumps | ~1–2k |
| (Impostor, if ever needed) | ~450 m | one card | 2 |

That is roughly a hundredfold cut for the far trees, with nothing smaller
than a pixel lost.

**The land is already a height function** (`Atlas::relief_at`), so a
quadtree of tiles is the same sampling it does now, just with bigger tiles
far off.

**The painted look forgives the far field.** Strokes far out are already up
to 64 m wide (`splats.rs`); detail finer than the brush is not on the screen
in a painted look at all. And the outline pass (`outline.rs`) draws edges
where geometry has them: sub-pixel twigs today make broken, crawling outline
dashes far off; pruning them should make the distant Wood read *cleaner*,
not worse. That is a claim to check on a screenshot, not to assume.

## 4. Action plan

In order; each step is small enough to land, measure and judge on its own.

**Step 0 — measure first (hours).** `--profile` already prints the heaviest
meshes and their triangle counts (`profile::heaviest`). Add to it: the total
mesh memory loaded (vertex and index bytes), the count of streamed entities
by kind, and the process's resident memory. Take a baseline standing at the
edge of the Great Wood, inside it, and in the open valley, on the desktop and
in the browser. Every later step is judged against these numbers.

**Step 1 — stop holding what is not needed (hours).**
- Drop each giant from `Stream::grown` when it goes out of reach, as the
  tree itself is.
- Build streamed meshes (land tiles, giants, box forms) with
  `RenderAssetUsages::RENDER_WORLD`, so the copy in ordinary memory is freed
  once it is on the graphics card. Two things read meshes back and have to be
  handled first: `profile::heaviest` (record the triangle count when the mesh
  is made instead) and `warm.rs` (it reads a mesh's layout to warm its
  pipeline; it must do so before the copy is dropped, or from the layout
  alone).
- Expected: roughly halves the giants' memory immediately, and caps it at
  what is in reach rather than everything ever seen.

**Step 2 — LOD for the giants from their skeletons (a day or two).**
- `grove::meshes(tree, palette, threshold)`: the threshold is the thinnest
  branch kept; pruned limbs hand their leaves to a merged clump.
- Three levels per tree, built lazily (the far one first, since it is what
  is seen first as you approach), switched with `VisibilityRange` and its
  dithered crossfade. The crossfade needs the dither added to `paint.wgsl`,
  which currently has none; until then, switch at a hard distance chosen
  so the pop lands at sub-pixel scale.
- Only the full level casts a shadow (inside 150 m it is the only one there
  anyway), and the see-through ghost (`window::Ghost`) only ever uses the
  near level — it only shows when the tree is between the camera and the
  fighter.
- Judge it with a sheet: one giant drawn at each level, side by side, at the
  distance it switches, through the painted look. A headless screenshot needs
  `./scripts/setup-tools.sh shot`.

**Step 3 — merge the far land (a day).** Beyond ~200 m, build land as
64 m or 128 m tiles (4×4 or 8×8 of today's) at the same vertex counts as
today's far levels. Tiles within 700 m drop from ~6,000 pieces to a few
hundred, with the same triangles. Skirts keep working as they are; CDLOD's
vertex morphing is the upgrade if the level boundaries ever show.

**Step 4 — merge the far boxes (a day).** Beyond the masonry's `NEAR`, every
box in one map tile drawn flat can be one mesh per tile instead of one per
box — Unreal's HLOD in miniature. It is what cuts the entity count in the
built-up places (the town, the Needle).

**Step 5 — impostors, only if step 2 is not enough.** If the far level of the
giants is still the biggest line in the census, bake a hemi-octahedral
impostor per tree (fourteen trees, one texture each, made at load from the far
mesh — no files). It is the most work and the most risk to the painted look,
since a pre-rendered picture of a tree does not get brush strokes laid over it
the way a mesh does, so it comes last.

### What would change the plan

- If step 0 shows memory dominated by something other than meshes —
  textures, the splat buffers, Bevy's own per-entity bookkeeping — steps 3
  and 4 move ahead of step 2.
- If the far Wood is rarely in view (the fog's reach in its sky is short),
  step 1 alone may be most of the win.

## 5. Steps 0 and 1, as built (2026-10-21)

**Step 0.** `--profile`'s census now prints, every two seconds:

```
mesh memory: 7155 meshes, 139.5 MB on the GPU; 1003 of them also in main memory, 20.2 MB
streamed: 22 places, 1069 boxes, 5007 land tiles, 14 giants drawn, their 28 meshes 73.3 MB
process memory: 1069 MB
```

Each mesh is weighed (triangles and bytes) the frame it is made, from
`AssetEvent<Mesh>` (`profile::weigh`), because a mesh made only to be drawn
is gone from `Assets<Mesh>` by the next frame; the list of the heaviest
meshes reads those weights too. Process memory is `platform::memory_bytes`:
resident memory on Linux, the program's memory in the browser (which only
grows, so it is the tab's high-water mark), not read on macOS or Windows yet.

**Step 1.**
- A giant's meshes are let go of when it is dropped (`Stream::grown`), not kept
  for the session; how far each reaches is kept on its own (`Stream::spreads`),
  since that is what it is loaded by.
- Streamed meshes -- the giants, the land's tiles, every box drawn by
  `arenas::draw_solid` -- are made with `stream::drawn_only`
  (`RenderAssetUsages::RENDER_WORLD`): held on the graphics card only. The
  rule that comes with it: such a mesh is spawned the frame it is made and its
  handle never handed to an entity later, because Bevy reads an entity's
  bounds off the mesh in main memory, and so does the warm-up.
- The water over a tile is one shared plane, not one each.

**Measured** with `--profile` and `--open`, headless (Xvfb and lavapipe, 1280×760),
the same build with step 1 taken back out as the "before":

| Where | | Meshes in main memory | Process memory |
| --- | --- | ---: | ---: |
| Hearth's square | before | 6,387 meshes, 158.3 MB | 1,258 MB |
| | after | 972 meshes, 19.9 MB | 1,130 MB |
| In the Wood (`SHOT_AT=12,-210`) | before | 6,846 meshes, 138.7 MB | 1,233 MB |
| | after | 1,003 meshes, 20.2 MB | 1,069 MB |

Screenshots of both, in the town and in the Wood, are the same picture but
for a critter that wanders.

What the numbers say:

- **All fourteen giants are drawn from Hearth's square.** The Wood is 250–460 m
  south of the town, inside the 600 m reach, so a player in town is carrying the
  73 MB of the giants they cannot see through the fog. Letting a giant go when
  it is dropped does not help there -- it never is. That is what step 2 (the
  far levels) is for, and it makes step 2 the next thing.
- **The land is ~4,400–5,000 tiles**, as section 1 guessed. The ~66 MB of
  mesh that is not the giants is the land, the boxes and the town together;
  the census does not yet split it by kind. Step 3 is about the count more
  than the bytes.
- **Most of the process is not meshes.** About a gigabyte remains. Under
  lavapipe the "graphics card" is main memory, so the 140 MB on the GPU is in
  that figure too; the rest (shadow maps, the still world's maps, textures,
  Bevy itself) is not yet broken down, and on a real graphics card the process
  figure will be lower. Breaking it down is the next measurement if memory
  stays the complaint.
- **Frame time did not move** (≈150 ms a frame either way under the software
  rasteriser): step 1 is memory, not speed.

**Not exercised at run time:** dropping a giant when it goes out of reach. The
headless run cannot walk away from the Wood, and from both starting points every
giant stays in reach. It is three lines (`stream::groves`) and the tree is
rebuilt the next time it comes within reach.

## 6. Steps 2, 3 and 4, as built (2026-10-22)

**Step 2: the giants' levels** (`grove::LEVELS`, `grove::detail_at`,
`grove::meshes`' `detail`). A giant is drawn at one of four levels by the
distance to the nearest point of its box: everything; past 100 m, 250 m and
450 m, nothing finer than one pixel at 1080 lines at that distance (plain), or
than the paint's brush (painted: `look::Wash::brush`, its radius at 760 lines;
LOD-2). Four rules apply what "finer" means, every one derived from the skeleton:

- **A branch thinner than the detail is left out.** By the pipe model its parent
  is already as thick as everything it carries.
- **A run of nodes along one limb** (one child each, the same part, borne on
  or not alike) **is one tube** while every node it passes is within the
  detail of the straight line, thickness included -- the line simplification
  every map renderer does to a coastline. It turned out to be the one that
  counts: four-fifths of a giant is wood, and most of the wood was short
  straight pieces with caps between them.
- **Rounds get the fewest faces** whose flats stand no more than the detail
  inside the true round; shelves and terraces the same, never under eight.
- **Leaf clumps closer than four times the detail are one clump**, centred
  among them, its radius the root of the sum of their squares, so it covers the
  same area seen from afar.

Triangles for all fourteen (`grove::tests::every_level_is_a_saving`):

| Look | Up close | Past 100 m | Past 250 m | Past 450 m |
| --- | ---: | ---: | ---: | ---: |
| Plain (a pixel) | 915,548 | 723,964 | 398,556 | 300,800 |
| Painted, brush 2 px | 915,548 | 371,368 | 257,316 | 179,400 |
| Painted, brush 7 px | 915,548 | 215,988 | 92,844 | 65,408 |

The plain look saves least. At 100 m almost nothing on a giant is thinner than
a pixel: the cost of a near giant is real detail, not waste.

Swapping between levels is a hard switch (LOD-1), 12 m further out going coarser than
coming finer. The far levels are kept while the tree is in reach, and the full
one only while it is drawn. Because those meshes are held only on the graphics
card and are handed to new entities on later frames, each level keeps its
bounds and gives them to its entities (`stream::Grown`).

**Step 3: far land in blocks** (`stream::BLOCK`, `land::area_mesh`). Past 200 m
from its nearest point, a block of 4×4 tiles is one mesh, at the fineness its
tiles would have had. It is built in place of its tiles, and turned back into
them, all sixteen in one frame, so there is never a hole or an overlap. Each
tile keeps its water, and where strokes are laid as splats, an entity to carry
them.

**Step 4: far boxes in blocks** (`stream::MERGED_FROM`, `merge_boxes`). Past
160 m, every box in a block is appended to one of a few meshes, one per paint,
shadow and vertex layout, so every attribute a box carries survives
(`stream::tests::every_box_mesh_merges`). A box wider than a block, a tree's
trunk and a giant's box are never merged. At that distance no box is in its
near relief and the window to the fighter never reaches one.

**Measured** headless against the build of steps 0 and 1:

| Where | | Meshes | Entities | Mesh on the GPU | Giants | Process |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Hearth's square | before | 6,388 | 14,696 | 158.8 MB | 73.3 MB | 1,131 MB |
| | after | 2,838 | 10,979 | 124.2 MB | 41.1 MB | 1,014 MB |
| Over the valley (`SHOT_AT=150,60,400`) | before | 7,076 | — | 143.7 MB | 73.3 MB | 1,126 MB |
| | after | 3,005 | — | 105.6 MB | 37.4 MB | 1,021 MB |
| Walking into the Wood | before | 6,178 | 13,910 | 118.5 MB | 73.3 MB | 1,059 MB |
| | after | 2,016 | 8,945 | 116.1 MB | 73.0 MB | 1,060 MB |

Land pieces dropped from about 4,400 to about 1,000 (some 700 tiles plus 270
blocks).

- **Pictures.** Every view is the same picture but for the critter that
  wanders, with one exception: in the Wood the shadows of the giants past 100 m
  move by a pixel at their edges, because they are now cast by the coarser
  level. Side by side the two cannot be told apart.
- **Inside the Wood nothing is saved on the giants, and should not be:** every
  giant's box comes within 100 m of you, so every one is drawn whole.
- **Frame time.** It did not move under the software rasteriser, which is held up by
  its own pixel work. Whether the 25–35% fewer entities buy processor time is
  for a real machine (the owner's Mac) to say.
- **Exercised:** walking into the Wood with W held (`SHOT_PRESS`). Land blocks
  were built and split, a block of boxes split as it came near, and giants
  changed level, with no panic. Not exercised: dropping any of it past reach,
  which takes a walk of more than 700 m.

**Step 5 (impostors) is not needed.** After step 2, the giants past 450 m are
a third of what they were in the plain look and a tenth in a painted one, and
they are drawn in fog. Their biggest cost is now the Wood itself, up close,
where an impostor would not be used.

## Open questions

- **LOD-1.** Is a hard switch acceptable for the giants until `paint.wgsl`
  has the dither, or should the dither come first? **Decided (the owner,
  2026-10-21): a hard switch is fine.** The dither is not a prerequisite.
- **LOD-2.** Should the far levels obey the painted look's brush size rather
  than the pixel — prune to what one *stroke* covers, which is coarser and
  cheaper again? **Decided (the owner, 2026-10-21): the brush stroke, in the
  painted looks.** The threshold is the stroke's size at that distance
  (`splats.rs`); the pixel stays the threshold where no brush is drawn.

## Sources

- Bevy `VisibilityRange` (HLOD and dithered crossfade): Bevy 0.16 source,
  `bevy_render/src/view/visibility/range.rs`;
  [dev docs](https://dev-docs.bevyengine.org/bevy/render/view/struct.VisibilityRange.html)
- Bevy `RenderAssetUsages` (freeing the CPU copy; wasm does not return
  memory): [PR 11399](https://github.com/bevyengine/bevy/pull/11399)
- Bevy shadow cascade defaults (150 m, four; one in WebGL 2): Bevy 0.16
  source, `bevy_pbr/src/light/mod.rs`
- Godot's visibility ranges: [docs](https://docs.godotengine.org/en/4.4/tutorials/3d/visibility_ranges.html)
- SpeedTree runtime LOD: [Unity's SpeedTree SDK manual](https://docs.unity3d.com/speedtree-runtime-sdk/manual/level-of-detail.html),
  [Unity terrain tree LOD](https://docs.unity3d.com/Manual/terrain-Tree-LOD.html)
- Horizon Zero Dawn: Gilbert Sanders, "Between Tech and Art: The Vegetation of
  Horizon Zero Dawn", [GDC 2018](https://gdcvault.com/play/1025066/Between-Tech-and-Art-The)
- Octahedral impostors: [Godot implementation](https://github.com/SIsilicon/Godot-Octahedral-Impostors),
  [a three.js forest](https://discourse.threejs.org/t/a-forest-of-octahedral-impostors/85735)
- Unreal World Partition HLOD: [Epic's documentation](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition---hierarchical-level-of-detail-in-unreal-engine)
- CDLOD: Filip Strugar, 2010, [paper and source](https://github.com/fstrugar/CDLOD)
