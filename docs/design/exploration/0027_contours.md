# 0027 -- Contours: growing a creature's parts in the shapes of what they are

**Asked** (the owner, 2026-10-23, on the grown Ridgeback of 0025 §14): *"It's
decent but we can't have floating sections of the body plan. I also need a
more naturalistic shape, not just an algorithmic smoothing. Thin creatures
show their ribs. A ridgeback's legs should be like a rhino's or an
elephant's -- thick, round, but with obvious muscle underneath and shaped to
distribute load. ... Different algorithms for smoothing that tend towards
different natural detail and patterns depending on the creature ... some way
to build a gradient between them. Heavily muscled, weight distributing, long
and whiplike, narrow and birdlike, winglike, etc. Start by reading research
on this."*

## 1. What the research says

Four lines of work, and what each gives us. (Much of the primary literature is
outside this environment's network -- the UCSC and IRIT servers and the
Infinigen repository refused -- so these are read from abstracts, secondary
accounts and the papers' own summaries.)

- **Anatomy-based modelling** (Wilhelms & Van Gelder, SIGGRAPH 1997; Scheepers
  et al. 1997). Bones, then muscles as deformable cylinders strung from origin
  to insertion, then skin as an isosurface over the voxelised anatomy, relaxed
  as a membrane. The right answer, and far too much: a muscle per attachment
  for every species, hand-placed. What survives of it here is the *order* --
  structure, then flesh, then a skin that follows both -- and the idea that
  muscle is a shape added along a bone, not a texture.
- **Implicit surfaces for bodies** (Spore's creature skin, Hecker; implicit
  skinning, Vaillant et al. 2013; gradient-based blending, Gourmel et al.
  2013). A blobby field is *topologically robust* -- it never tears and never
  floats, since it is one function -- and its price is *no local control*:
  blend two blobs and you get a blob. Vaillant's answer is composition
  operators that blend at a joint but keep a bulge where flesh meets flesh.
  That is where the owner's first complaint and the second meet: the field
  guarantees one piece, so long as the pieces are *in* the field; the
  smoothing is what has to be replaced.
- **Generalized cylinders and factored parts** (Bloomenthal 1985; Infinigen,
  Raistrick et al. 2023). A limb is a curve, a radius profile along it and a
  cross-section round it. Infinigen's lesson is the one that decides the
  design here: randomising the surface (NURBS handles) gives *lumpy,
  unrealistic creatures*; randomising a factored space (a polar skeleton of
  radii and angles, profiles placed round it) gives large-scale, plausible
  change. **Detail belongs in the parameters of a profile, never on the
  mesh.**
- **The animals themselves.** Graviportal limbs (Biewener 1989; Mallet et al.
  2022; the finite-element elephant humerus): columnar, thick, straight, the
  muscle proximal, the foot a pad that spreads the load -- robust bone over
  slender. Cursorial limbs (the horse): muscle at the top, long tendons
  below, so the swinging end is light. Body-condition scoring (the equine and
  bovine charts): ribs show below a middling condition, the spine and the hips
  after them -- a *fat* dial over a *structure* that is always there.
  Countershading is in 0025 §14 already.

What none of them gives: a way for *one* table-driven grower to produce an
elephant's leg here and a whip there and something in between, for a species
that is only a table of boxes -- which is this game's constraint, since a
bred creature is nothing else.

## 2. The design

**A contour is three functions on one domain.** Along a part's long axis,
`t` from its joint to its far end: a radius profile `r(t)`. Round it, `θ`
from up: a cross-section `s(t, θ)`. Over both, detail -- muscle bellies, ribs,
a foot's pad -- written into those same two functions. A part is grown as the
signed distance to the surface `r(t)·s(t, θ)` times its box's ellipse.

**Seven kinds** (`crates/game/src/contour.rs`), each those three functions:

| Kind | From | Profile | Section and detail |
| --- | --- | --- | --- |
| Column | graviportal limb | near straight; a slight waist at the knee; a pad flaring at the ground | two bellies, fore and aft, with a groove between, fading toward the foot |
| Taper | cursorial limb | heavy top, thin cannon | round over the muscle; the cannon flat across |
| Barrel | a trunk | fullest in the middle | a spine ridge above, a belly sag below, and ribs across the flank by how lean |
| Whip | a tail | thins to a point | round |
| Keel | a bird's body | fusiform | deeper than wide, keeled |
| Wing | a membrane | keeps its planform | flat with a thick leading edge |
| Wedge | a head | narrows to the muzzle | a brow, a flatter jaw |

**Because every kind is the same three functions on the same domain, a
contour between kinds is their weighted sum** -- a point in the simplex of
kinds -- and it is as smooth as either end (`Contour::between`). Two dials are
shared: `fat` (how much flesh hides the structure: ribs and bellies fade out
as it rises) and `muscle` (how pronounced the bellies are). Infinigen's rule
holds: nothing random is added to the surface; every variation is a parameter
of a profile.

**Which kind a part is** (`grown::inferred`): the species' own word for it
(`species::Build`, beside its look, as a tint is) or, failing that, inferred
from its bone's name and which way its long axis points -- `head` a wedge,
`tail` a whip, `wing` a wing, `neck` between column and taper, a vertical
part a limb between taper and column by the species' `heavy`, the rest a
barrel. The Ridgeback says: fat 0.65, muscle 0.8, heavy 1.0 -- an elephant.

**A chain of parts is one contour** (`grown::chain_span`). A tail is three
boxes; grown as three whips it was three stumps, each thinning to its own
point. The parts along one line of bones that are the same kind share one
profile, each taking its share of the chain's length -- so the tail thins to
one point, and a leg's two boxes are one column that flares only at the
ground.

**One piece** (`grown::Sinew`): a capsule along every bone from its joint to
its parent's, and from each part's middle to its joint, in the same field --
the body is connected along the skeleton by construction, whatever the boxes
leave open. A flood fill over the field keeps the largest component and
counts what it dropped; the test asserts nought for the Ridgeback, and the
game says so in its log if a species ever loses any.

## 3. What it looks like

The Ridgeback, before and after, is in 0025 §14 and this: columnar legs with
two bellies and a pad, a tail that is one whip, a barrel with a belly and a
rib line on the flank, a head narrowing forward.

## 4. What is open

- **Ribs read weakly** at fighting distance: the hoops are a few centimetres
  proud on a six-metre flank. A lean species will say so with `fat` near
  nought; whether the brushwork should pick up the ribs as strokes is the
  next question.
- **The detail is anatomy by rule, not by muscle.** Two bellies on a column
  is the elephant's silhouette; a real forelimb has a triceps mass behind and
  a pectoral in front, and a species that wants that would want a muscle
  list, which is Wilhelms' road. Not taken until a creature needs it.
- **Only the Ridgeback is grown.** The kinds the Ridgeback does not use --
  keel, wing -- are drawn on the sheet (`cargo test -p game contour sheet`
  writes `target/contours.pgm`) and nowhere else yet. The Galewing is the
  species that will tell whether a wing works.
