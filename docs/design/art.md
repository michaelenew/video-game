---
status: decided
decided: 2026-09-11
---

# Art

There is no artist and there will not be one. Everything visible has to come
out of a formula.

That sounds like a constraint and is mostly a different set of trades. You give
up the ability to draw one specific thing; you get a thousand variations of
anything you *can* describe, for free, at any resolution, with no import step
and nothing to keep in a folder. For a closed arena with six fighters it is a
good trade, and the design already leans on it — [the combat
kernel](combat-kernel.md) says the closed arena exists so there is less art,
and that design choices adding depth without adding assets are "a real
tiebreaker, not a nicety".

## The scarce resource is decisions, not assets

The tempting reading of "no artist" is "we need cheap textures", and taking it
leads to a folder of noise shaders exactly as unmanageable as a folder of PNGs.

Procedural generation makes variations free. It does nothing whatsoever about
the fact that **somebody still has to decide which variation is right**, and on
a team of one that decision cost is what actually runs out. A system that can
generate ten thousand rocks and cannot say which rock is correct has moved the
problem, not solved it.

So the whole thing is arranged to spend as few decisions as possible:

- **Few parameters.** A material with forty knobs is worse than one with eight
  when there is one person turning them.
- **Parameters derived from things already decided.** Fire's colour comes from
  its temperature. A player's colour comes from their slot. An armour's tint
  comes from the player wearing it.
- **Rules that can be tested rather than judged.** Whether the green player is
  still distinguishable from a monster to a colour-blind viewer is a
  computation. So it is a test that fails the build, not an opinion someone
  has to re-form every time a colour moves.

That last one is the load-bearing substitute for an art director. An art
director's real job is consistency and legibility; a large part of legibility
is arithmetic, and the arithmetic can be checked on every commit.

## One evaluator, and materials are points in its parameter space

`crates/art` has a single function from a point in space to a surface. A
material is the numbers you hand it. Adding stone after fire costs a row of
constants — not a shader, not an asset, not a filename, not an import.

```text
surface(p) = ramp( shape( form(p + warp) , detail(p) ) )
```

A **form** field gives the large structure, a **detail** field the fine grain,
a **warp** displaces the form's lookup so features wander instead of running
straight, and a **ramp** turns the resulting number into colour. Roughness
comes off the same number, so a material that is mottled in colour is mottled
in gloss — which is what real surfaces do and most of what sells them.

This is deliberately the same shape as [the Oven](architecture.md#the-oven-tuning-while-it-runs):
one representation, one editor, one bake. Uniformity is what made three hundred
feel numbers cost an afternoon instead of a project, and it does the same here.

### With one difference that matters

**Tuning values are in the simulation's checksum. Art must never be.**

Mistuned peers desync loudly and immediately, which is right for anything that
decides what *happens* and wrong for anything that decides what you *see*. Two
people playing each other have to be able to run different texture resolutions
and still agree on the fight.

Art sits on the same side of that line as [the camera](architecture.md#why-the-camera-is-not-in-the-oven),
for the same reason and with the same consequence: `art` is free to use
floating point, and `sim` is not.

## Hue is a gameplay channel, so materials may not spend it

One rule does more for how the game reads than any texture will.

In third person with four players and several monsters, the questions answered
in a glance are *which one is me* and *which of that is dangerous*. Colour is
the fastest channel the eye has, and it only works if nothing else competes for
it.

| Band | Belongs to |
| --- | --- |
| Saturated red | **Hostile.** Monsters, active hitboxes, incoming danger. No player is ever assigned it |
| Four spaced hues | **Players.** One each |
| Near-zero chroma | **The world.** Stone, floor, walls, sky |

The arena being almost colourless is the load-bearing part. It is not a
stylistic preference — on a grey stage, anything with colour in it is by
definition a thing that matters. It is also the cheapest art direction
available, which is not a coincidence: it is the same reason the closed arena
was chosen.

**Class identity is carried by silhouette, material and effect shape, not by
hue.** Which class the opponent is playing is learned once at the start of a
round; which fighter is yours is needed every frame. The urgent question gets
the fast channel.

### The reserved band is a hue *and* a chroma

The first version of the rule reserved a hue band, and the first material built
against it found the hole: **human skin is red-orange.** So is leather, so is
rust, so is firelight on a wall. A hue band alone bans half of what a fighter
is made of.

Saying what was actually meant fixes it. Danger is not *reddish*, it is
**saturated red** — the eye separates a vivid red from a tan long before it
separates two hues at the same chroma. So the reserved region has a chroma
floor, and a desaturated warm walks underneath it without ever competing.

### Identity is never hue alone

Around one man in twelve has a shifted green response, and for them two colours
differing only in hue can land on top of each other. The fix is not to avoid
green. It is that every identity must differ in **lightness as well**, because
lightness survives every form of colour blindness.

The four player colours and the hostile red were **searched for, not chosen**
(`cargo run --release -p art --example hunt`). Two things that search taught,
each costing an attempt:

- **Unconstrained, it has no taste.** Maximising separation alone puts the
  green at lightness 0.92, which separates beautifully and looks like pale
  mint. Lightness is bounded to where a colour still reads as its own name.
- **Telling two players apart and telling a player from a wall are not the same
  task.** Folding both into one objective collapsed every identity onto the
  same lightness, chasing an arena term that was never in danger — the arena
  has almost no chroma, so anything coloured is far from all of it by
  construction. The arena is a floor to clear, not a quantity to maximise.

What came out: every pair at least **0.136** apart under red-green colour
blindness, every identity at least **0.122** from anything the arena is made
of. The stubborn pair is green against hostile red, and it is carried entirely
by lightness. Tests pin both numbers.

## Colour arithmetic happens in Oklab

Blending two colours by averaging their sRGB channels goes through grey. Orange
to blue passes through mud. That is not a taste failure, it is arithmetic: sRGB
is a display encoding and the straight line between two points in it is not the
path the eye reads as "between".

**Oklab** (Björn Ottosson, 2020) is built so that distance approximates
perceived difference and a straight line looks like an even fade. Its polar
form splits a colour into lightness, chroma and hue, which are the three things
worth talking about separately — and holding hue and chroma while moving
lightness is exactly the "same colour, darker" operation that scaling an RGB
triple does not give you.

Measured on the ramp the test uses: the Oklab midpoint keeps chroma 0.160 where
the naive blend keeps 0.105. A third of the colour, lost in the middle of every
gradient, for free.

## Fire's colour is physics, not a colour picker

A luminous flame is glowing soot, and glowing soot is close enough to a
**blackbody** — something that emits purely because of its temperature — that
the standard curve fits it.

So fire's colour is a function of one number, and that number means something:
1700 K is a dull ember, 2600 K a hot flame, 3000 K the warm white of a filament
bulb, 6500 K daylight. That collapses "what colour is the fire" into "how hot
is the fire", which is a question the *game* already has an opinion about — a
spell meant to read as hotter simply is. The gradient from core to edge falls
out of a temperature falloff rather than being three colours somebody picked.

Two honest limits, both worth knowing rather than discovering:

- **The blue at the base of a gas flame is not thermal.** It is light emitted
  by excited molecular fragments as they burn, and no temperature curve
  produces it. Where a design wants that blue it is added as a tint.
- **The fit bottoms out at 1667 K.** Below that the published curve turns
  around and returns the same colour for two different temperatures, so a
  cooling ember would stop changing and nothing would say why. Clamped, and the
  consequence is correct anyway: the coolest fire is a deep orange-red, and an
  ember fading past it fades by getting *dimmer*, not redder. Which is what
  embers do.

## The maths runs on the processor, not in a shader

The instinct with procedural materials is to evaluate them per pixel in a
fragment shader. That is right for a large open world and wrong here, for four
reasons that are all about this project specifically.

- **One implementation.** Shader code cannot be unit tested, stepped through,
  or shared with a tool. In Rust it is ordinary code with ordinary tests, and
  the same function serves the game, the preview sheet and the checks.
- **No render pipeline.** A baked texture is a `StandardMaterial` field. A live
  one is a custom material, a custom pipeline and a WGSL file that drifts from
  the Rust beside it — a large amount of engine surface bought for a small
  arena.
- **It runs anywhere.** Baked textures work on an integrated laptop chip and in
  a browser. Six octaves of noise triplanar-projected per pixel do not,
  reliably.
- **It is still live enough.** Measured on four cores: all ten materials at 256
  pixels in about 280 ms, one ordinary material in about 20 ms.

None of that is permanent. **The parameters are the asset**; moving one
material into a shader later means writing that shader, not redoing the work.
The bet is on the parameter space, not on where it is evaluated.

### What the limit actually is

Tiling. A baked texture repeated across a forty-metre floor shows its period,
and no number of octaves fixes that — it is a property of repeating a finite
thing. The moment the floor reads as wallpaper is the moment one triplanar
shader earns its place, and it will be the environment that needs it and never
the characters. Said plainly so it is a scheduled cost rather than a surprise.

The other limit is re-baking on every frame of a slider drag, which 20 ms per
material does not support. Re-baking on release does.

## The two clocks

A consequence of rollback that is easy to get backwards and expensive when you
do.

**Anything carrying information runs on the simulation's frame counter.** How
big the fire pillar is, how far into its startup a move is, whether a hitbox is
live. These are read by the player as facts about the fight, must be identical
on both machines, and must survive being re-simulated after a rollback.

**Anything that is only texture runs on the wall clock.** Flicker, drift,
shimmer, sparks. A rollback is one to eight frames — sixteen to a hundred and
thirty milliseconds — and a flame whose flicker hiccups across that window is
imperceptible. Paying for it in the snapshot would be absurd.

Get it the wrong way round and there is no crash. There is a **tell that
lies**: a flame that looks bigger than the thing that hits you. Which is the
same failure the debug overlay was built to avoid, arriving through a different
door.

The corollary, for when effects start perturbing their own shape: **visual
perturbation is inward only.** The simulation owns the volume; the renderer may
eat into it and may not reach past it. Then anything that looks like it hits
you does.

## Structure, not octaves

The failure mode of procedural materials is that everything comes out looking
like the same grey marble, and the instinct — add more octaves — is the wrong
one. **Octaves add detail. They never add structure.**

Three things add structure, and all three earned their place by a material
going wrong without them:

**Anisotropy.** A frequency per axis, not one scalar. Stone has strata; cloth
has a weave direction; armour has plates. Squashing the noise four times harder
along the vertical is the entire difference between rock and marble.

**Layering.** Two fields with different jobs, not one field with more of
itself. Stone is strata plus cracks. Cloth is weave plus slub. Armour is plate
plus wear. Skin is mottle plus blotch.

**Warping.** A field evaluated at a position displaced by another field, so
features wander instead of running straight. Two evaluations buy more than four
extra octaves — and the warp field must be *low* frequency, or it shakes the
form apart instead of bending it.

### The detail layer can drown the structure, and did

Stone's form field is 2.6-to-1 in favour of its vertical, which is what makes
it strata. With an isotropic crack layer at 30% grain, the finished surface
measured **1.1-to-1**: the strata were still in the parameters and no longer on
the screen. Flattening the crack cells the same way the strata are flattened
put it back. A test now measures the ratio and fails under 1.3.

### Detail finer than the texture is worse than no detail

Sampling theory's oldest result, and it cost two materials before it was
written down. A feature landing on fewer than about four texels does not come
out fine — it comes out as **moiré**, a coarse interference pattern that is not
in the material at all and gets coarser the finer you make the thing producing
it.

Cloth's weave at 34 cycles per metre resolved to 3.8 texels and came out as
watered silk. Skin's pores resolved to 2.9 and came out as a sheen. Neither
looked like a sampling problem; both looked like a bad material.

The other half of that lesson is cheaper: **pores are invisible at fighting
distance anyway.** The finest layer on a fighter should be whatever reads on a
limb across an arena, which is blotch, not pores. A test now checks every
material against the resolution it is baked at.

### Summed octaves cluster in the middle

The central limit theorem, arriving where it was not invited. The more octaves
you add, the narrower the output distribution gets around its mean — so the
ends of the colour ramp become unreachable and the material comes out flat
whatever the ramp says.

The floor found it: four octaves spanning 0.40 to 0.64 of a ramp running 0 to
1, which is three quarters of the colour thrown away. One `contrast` number
that stretches the scalar away from its middle fixes it, and is cheaper than a
table of per-field normalisation constants that would go stale the first time a
frequency moved.

## What exists now

Ten materials, all of them rows of numbers: **stone, ground, skin, leather,
cloth, armour, fire, blood, shadow, arcane**. Cloth, armour and arcane take a
tint, so they are one material each rather than one per player.

The fighters are dressed by part rather than painted one colour — bare head,
armoured torso, sleeved arms, booted legs. That is most of what makes a box
read as a person and it costs nothing, since the materials already exist.

`cargo run -p art --bin sheet` writes a contact sheet of every material and
every map. **Looking at it is part of the loop.** The test suite says whether
stone is anisotropic and whether the palette separates; it does not say whether
stone looks like stone, and the gap between those two is exactly where
procedural materials go wrong. Every material bug above was found by looking.

## Next, in order

1. **Lighting and atmosphere, before any more materials.** A flat-shaded scene
   with good light, fog and a real sky reads as deliberate; a well-textured one
   with bad lighting reads as broken. Both are nearly free of assets, and an
   atmospheric sky is pure maths. This is the highest ratio of result to effort
   left on the board and it is not more texture.
2. **Emission and bloom as the spectacle budget.** [The combat
   kernel](combat-kernel.md) makes ability spectacle an explicit goal. Emissive
   geometry plus bloom is the highest ratio of spectacle to effort in real-time
   graphics, and the fire and arcane materials already emit above white in
   order to trigger it.
3. **Weapon trails, which this architecture gets almost free.** Pose is a pure
   function of state, so the position of a weapon eight frames ago can be
   *recomputed on demand* rather than recorded. No history buffer, nothing in
   the snapshot, and correct across a rollback by construction. The one caveat
   is that it only reaches back to the start of the current action, which is
   exactly as far as a swing goes.
4. **Animation generated from frame data that already exists.** Every move has
   a startup, an active and a recovery window. Anticipate, extend, settle —
   with [the spring solver](architecture.md#the-animation-factory) filling in
   between — gives every move a passable clip from numbers already in the Oven.
   Then hand-tune the ones that matter.
5. **Procedural bodies.** The fighters are six boxes; the natural next step is
   not glTF but a parameter vector — proportions, plate coverage, palette. Six
   classes become six vectors and coop monsters become more of them. And since
   silhouette would then be a parameter, *silhouette distinctness becomes
   measurable*: render two classes in orthographic black and compare. Whether
   you can tell two fighters apart stops being a judgement call, the same way
   the palette already has.
6. **A triplanar shader for the environment**, when the floor starts reading as
   wallpaper and not before.
