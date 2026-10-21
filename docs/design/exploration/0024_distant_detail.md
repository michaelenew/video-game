# 0024 — Distant detail: what is drawn far away, and how other games make it cheap

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

## Open questions

- **LOD-1.** Is a hard switch acceptable for the giants until `paint.wgsl`
  has the dither, or should the dither come first?
- **LOD-2.** Should the far levels obey the painted look's brush size rather
  than the pixel — prune to what one *stroke* covers, which is coarser and
  cheaper again?

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
