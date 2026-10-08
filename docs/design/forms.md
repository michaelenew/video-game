---
status: proposed 2026-10-08; a first pass built, waiting on a word
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

## What to decide

1. Is this the right amount? The boulders read softer than stone from a
   distance; the roughness is one number (`rock`, 0.18 of the size).
2. Dressed stone stays crisp on purpose. Chamfer it, or leave it?
3. Next, if wanted: creatures' parts rounded **per part** (a flank rounder
   than a plate), fighters' limbs as true capsules joined at the knuckles,
   and the arenas' floors broken up by dressing rather than by geometry, since
   the floor's height is the simulation's.
