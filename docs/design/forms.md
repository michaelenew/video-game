---
status: proposed 2026-10-08; two passes built, the second on the word
---

# Forms — less box, same collision

Everything in the game is drawn from the box the simulation collides against:
a fighter's sixteen joints are scaled cubes, a creature's parts are boxes in
their bone's frame, an arena's solids are the boxes bodies stop at. That is
the right source (if you can see it, you collide with it) and the wrong
*look*: a meadow of crates, a cat made of bricks.

**The rule this document proposes:** the box stays the truth and the form is
drawn **inside** it. A rounded box is still inside its box; a rock pulled
inward by noise is still inside its box; nothing is ever drawn where a body
would not stop. So the hit test, the overlay, the camera's arm and every pin
are untouched, and the only question is what a thing *looks* like in the
space it already owns.

## The first pass (built, 2026-10-08)

One mesh family, `shapes::soft_box`: the **superellipsoid**, the surface that
runs from a sphere to a box on one number, with exact normals. And one
roughening of it, `shapes::rock`: each vertex pulled inward by a little
smooth noise, so a stone is a stone and never pokes out of its box.

| What | Was | Now |
| --- | --- | --- |
| A fighter's parts | unit cubes, scaled to the joint's box | rounded boxes: head nearly a ball (0.85), hands and feet soft (0.5), limbs rounder (0.6) than the torso (0.35) |
| A creature's parts | unit cubes | rounded boxes (0.3), one mesh for every species |
| Arena solids of bare **rock** | `boxy` | a rock |
| Arena solids of soft ground (earth, grass, sand, snow, ash, peat) | `boxy` | rounded (0.3) |
| Arena solids of dressed **stone**, timber, water | `boxy` | **unchanged**: a wall is cut square |
| Raised rock (the herd's boulders) | unit cube | one of two rocks |

The accent and the crest (`look::edge`) are painted on the new forms the way
they were on the boxes, so an arena's palette is unchanged.

## The second pass (built, 2026-10-08, on the word "do all four")

1. **Creatures' parts rounded by what they are** (`beast::Forms`): a plate --
   armour, the body -- keeps most of its edge (0.22), a breakable limb is
   rounder (0.5), a weak point is soft (0.7). The part's form is chosen the
   way its skin is, from the same declaration, and swapped per frame with it.
2. **Fighters' limbs are round about the bone** (`shapes::soft_cylinder`:
   round in cross-section, blunt at the ends) and **meet in a ball at every
   joint** that bends -- shoulder, elbow, wrist, hip, knee, ankle (`BodyBall`,
   twelve a body, the Reaver's shadow too). Each ball is the narrower
   cross-section of the limb hanging from it, a hair under, so it never shows
   through a straight limb and only fills the wedge a bend opens -- the rule
   the creatures' knuckles already follow. The head is nearly an ellipsoid;
   the torso keeps its shoulders.
3. **Dressed stone is chamfered**, not rounded: 0.12, enough that an arris
   does not catch the light as a wire, not enough to stop being cut stone.
   Timber a little more (0.2). Water stays flat. Props drawn as boxes get
   the same chamfer.
4. **The floor is broken up by what lies on it**, since its height is the
   simulation's: its colour wanders a little (`shapes::mottle`, 7 % over
   features a few metres across), and dressing is scattered over it by a hash
   of the arena -- tussocks in grass, pebbles on earth, peat, ash and sand,
   drifts on snow, chips on rock -- one per twenty-two square metres, never
   inside a solid, never in water, small enough to walk through unnoticed and
   half buried so nothing has an underside to float on. Not on a course,
   which has no floor.

## What to decide

1. Is the amount right? Each is one number: the rock's roughness (0.18 of
   its size), the chamfer (0.12), the mottle (7 %), the scatter's spacing
   (22 m² a piece).
2. The scattered dressing is procedural and the same every time for an
   arena; a creature's document may want its own (the Sandmaw's Pan has
   islands and bones already). The hand-placed props win on the eye, the
   scatter fills behind them.
3. Next, if wanted: silhouettes rather than forms -- a creature's parts
   tapered along the bone (a leg thicker at the haunch), which is a shape per
   part rather than a rounding per kind, and the first thing that needs an
   eye rather than a rule.
