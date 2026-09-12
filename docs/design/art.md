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

## Lighting, and the one number it comes from

**How high the sun is, and everything else follows.** Its direction, its
colour, how bright it is, what colour the sky is, how much light comes back
down out of it. Six answers from one parameter, with physics rather than taste
between them.

Air scatters short wavelengths far harder than long ones -- Rayleigh's result,
and the reason the sky is blue at all. So sunlight reaching the ground has been
filtered, by an amount that depends on how much air it crossed: one atmosphere
overhead, nearly forty at the horizon. Run Beer's law per colour channel and
**sunset comes out**. Nobody picks the colour of the sunset and nobody can pick
it wrong.

It replaced three hand-tuned magnitudes -- a key, a fill and an ambient -- with
no relationship to each other, so moving one meant re-judging the other two by
eye. They had also drifted into compensating for placeholder materials: the key
sat at 11,000 lux to make surfaces of 0.02 reflectance read, and the moment the
materials became physical the arena blew out to near-white.

The **fill light is gone**, and that is the point rather than a saving. It was
aimed at the inside faces of the walls, which the key never reaches. A second
sun is a lie that costs a shadow direction; what lights those faces outdoors is
the sky, which is now a real quantity with a real colour. Skylight being cooler
than sunlight is also what makes a lit face and a shadowed face read as
*different surfaces* rather than one surface at two brightnesses.

The scene is now lit in **real lux**, so the camera is set like a real one
pointed at daylight. Physical lights with a film speed meant for a lamplit room
is just a blown-out picture.

### What the engine draws and what this computes

Bevy renders the sky with Hillaire's atmospheric scattering, which is a better
piece of work than anything belonging in this crate and exactly what
[architecture.md](architecture.md) says to take from the engine rather than
write.

The two are not a second implementation of one answer. The engine computes
**in-scattering** -- the light the air sends toward the eye, which is the image
of the sky. `art::sky` computes **transmittance** -- the light that survives the
trip to the ground, which is the lamp. Different quantities from the same
physics, and the renderer has no way to hand back the second in a form a
directional light can wear.

Two things the model will not do, stated rather than discovered: the blue at the
base of a gas flame is chemistry, not temperature, and the blackbody fit bottoms
out at 1667 K, below which it turns around and returns the same colour for two
temperatures. Clamped, and the consequence is right anyway -- an ember fading
past deep orange fades by getting *dimmer*, not redder.

## Spectacle: emission, bloom, and the light a hit throws

A surface emitting more light than white has nowhere to go in an eight-bit image
and simply clips. With a high-dynamic-range camera and bloom, the excess spills
into neighbouring pixels, and **that spill is what the eye reads as brightness**
rather than as pale colour. It is the difference between a fire that is orange
and a fire that is burning, and it costs one component.

A hit spawns a point light that dies over a fifth of a second. Near free, and it
lights the surroundings -- so a hit beside a wall washes the wall, which is the
part that reads as force. It is **warm white rather than the attacker's
colour**: the simulation records that you are in hitstun, not who put you there.
With two fighters "the other one" is a safe guess and it is wrong the moment
coop puts four players and several monsters in the arena. The flash says *a hit
landed*, which is true, instead of *who landed it*, which is not known.

### How a see-through material combines depends on why it is see-through

Three attempts, and the sequence is the lesson.

**Something that glows adds.** Fire is emitted light: it makes what is behind it
brighter and never darker. Alpha-blending it multiplies the background by a
near-black albedo, so every dim part of the flame paints a grey smear -- which
is exactly what the pillar did, rendering as a dark drum with flames on top.

**An additive material needs no alpha at all.** Black adds nothing, so the
silhouette is already carved by the emission. Carving it *again* with alpha
left the flame as a few thin streaks with the body missing.

**Something that blocks light blends.** The Reaver's shadow is an absence, so it
must be able to darken what is behind it, which is the one thing adding cannot
do.

Along the way: marking a flame `unlit` turned it black, because Bevy adds
emission *inside* the lighting pass. Skipping the pass skips the glow, leaving
only the near-black albedo the flame was given precisely because it was supposed
to be glowing.

## Trails, which this architecture gets almost free

The usual way to draw a swing trail is to record where the weapon was each frame
and join the dots. Under rollback a history buffer is a liability: re-simulating
frame 90 twice appends the same point twice, and an eight-frame rollback leaves
eight stale points smearing through a swing that never happened.

None of that is necessary. **Pose is a pure function of simulation state**,
which rollback already requires -- and a pure function can be asked about the
past. Where the hand was eight frames ago is the pose function evaluated at
`frames_into - 8`. No buffer, nothing in the snapshot, nothing to invalidate,
and correct across a rollback by construction, because the recomputation uses
the corrected state. A stateful feature becomes a stateless one.

Two things it taught:

**The reach comes from whatever drives the pose.** The first version capped it
by `frames_into`, which counts within a *phase* -- so at the start of an active
window it is nearly zero, which is exactly when the swing is fastest and the
trail should be longest. Every swing got a one-quad ribbon a few centimetres
long. A clip's elapsed counter runs across the whole move, so walking that back
crosses into the wind-up where the arc actually is.

**A trail is only as good as the animation under it.** The procedural fallback
poses are *one pose per phase*: the whole active window is a single static arm
position, so a trail sampled across it is the same edge eight times. Baked clips
move, because the spring solver fills in every frame between the keys. So trails
are drawn where there is a clip and nowhere else -- which makes generating clips
from frame data a prerequisite rather than a nicety.

## Every move has its own clip now

Each move carries a startup, an active and a recovery count -- three numbers
that already decide the entire neutral game. They also completely determine the
*shape* of the animation: wind up for the startup, be extended for the active
window, return during recovery. So the clip does not need authoring. It needs
deriving.

Before this, every attack played one of two hand-authored clips -- a 17-frame
poke or a 42-frame overhead -- indexed by the move's own elapsed frames. When
the lengths matched, that worked. The Bulwark's **Grapple** runs 53 frames, so
it played the poke and then **stood frozen for thirty-six of them**. The
Champion's **Sweep** is 6/3/12 against a clip that strikes on frame 7, so the
arm arrived a frame after the hitbox.

The second is the one that matters. **A telegraph that does not line up with
the frame data is a telegraph that lies**, and the design rests on moves being
readable: [combat-kernel.md](combat-kernel.md) makes startup length the thing an
opponent reacts to. An animation is the same kind of claim the debug overlay
makes, addressed to the player instead of the developer -- and the overlay
exists precisely because one that can drift from the rules is worse than none.

Deriving the keys means **the extended pose lands on the first active frame by
construction**, for all eighteen moves, and stays there when the frame data is
retuned in the Oven. A test asserts the arm reaches further during the active
window than at any point before it, for every shape and every plausible frame
count; another fails if a committed clip's length stops matching its move.

The shape comes from the move's own flags rather than a declaration, so a move
that stops being an overhead stops animating like one at the same moment.

What it does not do is make a move look like *itself*. A derived clip knows how
long a move takes and roughly what shape it is; it does not know the Reaver's
Guillotine is a downward chop with a shadow behind it. This is a floor, not a
ceiling.

## Bodies are a parameter vector, and silhouette is now a test

Six boxes with fixed sizes made every class the same person in a different
colour. That is a readability problem before it is an aesthetic one: **colour is
already spoken for** as the channel that says *which player*, which leaves
silhouette as the channel that says *which class* -- and six identical
silhouettes answer that question with nothing. In a four-player fight with two
people on the same class, it is the only channel there is.

So a body is eight numbers, and six classes are six vectors. The real argument
for that on a team with no artist is not that it is cheap, though it is. It is
that **silhouette becomes measurable**: render two builds flat black in
orthographic projection, compare the covered cells, and "can you tell these two
apart at a glance" stops being a judgement somebody re-forms every time a
proportion moves.

Written by hand from the kits -- the Bulwark holds a line, the Reaver is a
duellist who is not there when you swing -- the worst pair sat at **0.228**,
which is distinguishable if you are looking for it and not much more. A
hill-climb over the roster, maximising the *worst* pair inside bounds that keep
each class recognisably itself, took that to **0.392**. Bounds rather than a
free search, for the reason the palette search taught: an unconstrained
optimiser has no taste, and will happily produce a three-metre fighter with
pencil arms.

Two things that went wrong, both worth having written down:

**Poses are applied as a deviation from rest, not as absolute positions.** The
poses are authored in metres against the even body; scaling those positions onto
a different one pulls arms off shoulders and leaves legs hanging. With a rest
position computed per build, the joint goes where the body says and the
animation still moves it the distance it was authored to move.

**Parts are unit cubes scaled by the pose system**, not meshes built at size. A
class can change mid-session, and baking size into the mesh would mean rebuilding
six meshes on every switch. It costs nothing in texturing, because Bevy's cuboid
faces each use the whole texture regardless of size.

## The triplanar shader is not the answer, and the measurement says why

This was scheduled as "when the floor starts reading as wallpaper and not
before". It does not read as wallpaper -- but the reason is not the one that
would justify leaving it alone.

The tile is four metres and the camera sits eleven metres back, so a repeat
would be plainly visible **if there were anything left to repeat**. Tileable
baking buys its seamlessness by blending the tile against three shifted copies
of itself, and measured, that keeps only **65-67% of the contrast**. The floor
does not show its period because the period has already been flattened out of
it. The cost of tiling has been paid in full; it was just paid in flatness
rather than in repetition.

Which means the fix is not triplanar projection. Triplanar removes the repeat
and makes patterns wrap corners, and it costs a custom material, a custom
pipeline and a WGSL file that drifts from the Rust beside it -- and it does
nothing about the contrast, because it is still a world-space projection of a
surface pattern.

**Baking each surface as its intersection with a three-dimensional volume does
all of it.** No repeat, because every point of every object reads a different
part of the volume. No blending, so no contrast lost. Patterns that wrap corners
because they were never on the surface to begin with. And it stays a processor
bake, so there is still no shader. That is the next item rather than this one.

## Stone is a volume

A rock is not a picture on a wall. It is a **solid**, and every face you can see
is a cut through it. That is not a philosophical point -- it is why surface
stone looks wrong in ways nobody can name. The pattern repeats, because a tile
repeats. It does not turn corners, because the two faces of a corner are two
separate pictures. And it has no *history*: real stone records how it formed and
what has happened to it since, and those are three-dimensional facts.

So `art::stone` models the volume. Feed it a point and it says what the rock is
like there. A texture is that volume sampled where an object actually sits --
the intersection of the object with the stone -- which makes every wall in the
arena a different piece of rock with no extra parameter, and makes a pattern run
round a corner because it was never on the surface to begin with.

**Solid texturing** is old ([Peachey and Perlin, both 1985](https://dl.acm.org/doi/10.1145/325165.325246))
and was always the right answer for rock. What is new here is that the
parameters are *geological* rather than arithmetic: you ask for a coarse-grained
igneous rock with two joint sets and heavy weathering, not for four octaves at
0.55 cycles per metre.

### The three fabrics are three mechanisms, not three presets

Rocks are classified by how they formed, and how they formed is exactly what
their texture *is*. So each is different arithmetic, and a test asserts they
have not collapsed into one function with different constants.

**Igneous** -- frozen from a melt. Crystals grow until they run into each other,
so grains **interlock** with no gaps and meet at angular boundaries. Several
minerals crystallise at once, which is why granite is speckled rather than
plain: it is a *population* of grains, each hashing to a mineral.

**Sedimentary** -- settled and buried. Each depositional episode leaves a **bed**
with its own grain size and composition; gravity makes them parallel and later
movement buckles them.

**Metamorphic** -- cooked and squeezed. Stress recrystallises platy minerals
aligned perpendicular to the squeeze, and with enough of it light and dark
minerals segregate into bands. The grains are *flattened into the foliation*,
which is the same statement as "the minerals are aligned" -- implemented by
squashing the cell lookup along the bedding normal, one line rather than a
separate mechanism.

Then **joints** (rock breaks along planes) and **weathering** (water gets in at
the joints and works outward, so alteration is strongest where the rock is most
broken).

### Joints are planes, not cells

The obvious way to break rock into blocks is a cellular basis, and it is wrong.
Cells give *a* partition -- blocks of random shape in random orientations. Real
jointing gives **sets**: families of near-parallel planes, two or three at
angles, and every block in the mass shares those orientations. That shared
orientation is most of what makes a rock face read as rock rather than as crazy
paving, and no cellular function produces it at any setting.

### Band-limiting has two halves, and they point opposite ways

This is the part that took the most attempts and it generalises well beyond
stone.

**A field that fills an area fades toward its mean.** Grain finer than a texel
cannot be drawn; the only question is whether it becomes noise or becomes the
colour the rock averages to. Noise is the default and on a wall it is a grey
fizz that crawls when the camera moves. So as the grain drops below a few texels
the crystals fade into their own mean -- which is also just *true*: a distant
granite is not speckled, it is grey.

**A line gets wider and fainter.** Doing the same thing to a joint deletes it,
because a joint's mean over a texel is almost entirely rock. At arena scale that
was not subtle: a thirty-metre wall on a 256-pixel texture gives a twelve
centimetre texel, and a one-centimetre joint is a twelfth of one. Every crack
disappeared. So a joint is drawn at least a texel wide and darkened in
proportion to how much of that texel it really occupies, which conserves the
light it removes at any resolution -- what a correct mipmap of a line does.

Two more things the same principle caught. **Faces of one object must agree**:
an arena wall is thirty metres long and one metre thick, so at equal texture
sizes its end cap resolves thirty times finer than its side, and the same rock
came out crystalline on one face and smooth grey on the other. A caller baking
several faces passes one density for all of them. And **measuring a rock needs
the grain filtered out too** -- the first version of the bedding test sampled at
full resolution and every rock came back isotropic, not because the beds were
missing but because millimetre speckle contributes far more variation along a
line than metre-scale layering does. It was measuring the grain and calling it
the fabric.

### What it cost elsewhere

Real granite reflects about 0.12 of the light falling on it. The arena floor was
sitting at 0.026 -- darker than asphalt -- because it had been pushed down to
stop the scene blowing out, which was itself a symptom of lighting tuned against
placeholders. Against a floor four times darker, a real granite wall reads as
poured concrete. Both are plausible now and they agree.

That exposed a genuine unit bug underneath. Bevy multiplies
`AmbientLight.brightness` straight into the shaded result, while a directional
light's diffuse goes through the Lambertian `1/pi`. Both are documented in lux
and they are not the same units: handing over an illuminance lands about three
times too bright. The signature was surfaces blown out while the sky above them
-- which the atmosphere renders independently -- was exposed correctly.

### Looking at it is still part of the loop

`cargo run --release -p art --bin quarry` cuts every rock open: three slices at
right angles through the same stone, a fourth parallel to the first, and a
close-up. The three orthogonal cuts are the check that it is really a solid --
if they do not agree at a corner it is three pictures. The parallel pair is the
check for the other failure, a 2D field extruded.

### It is a terrain generator too

Nothing in the model is about walls. It answers "what is the rock like at this
point" for any point, so the same volume that a wall is cut from is the volume a
hillside would be carved out of -- the strata a cliff exposes are the same strata
the arena floor is standing on. That was not designed for; it is what modelling
the solid instead of the surface gives you.

## The arena is one rock, and there is a landscape behind a button

**Everything you stand on or walk into is a face cut into a single stone
volume.** Not a floor plus some walls that share a texture: the floor slab, the
walls and the platforms all sample one volume *where they actually are*, so the
grain runs continuously from the floor up the wall it meets, a joint that
reaches a corner comes out the other side, and no two surfaces are the same
piece of rock.

That is how a rock-cut temple is built and it is why they read as they do.
Kailasa at Ellora was carved downward out of one basalt outcrop rather than
assembled, so no two of its surfaces disagree about what the hill was made of.
Nothing in the renderer has to arrange this; the continuity is a consequence of
every face asking the same volume where it is.

The ground beyond the arena now sits *below* it, so the arena reads as a plinth
cut out of bedrock. It also had to: the floor slab's top and the ground plane
were both at zero, which is two coplanar surfaces fighting over every pixel. The
fix and the look wanted the same thing, which is usually a sign the look was
right.

### A landscape, on a toggle

A button at the top of the screen swaps the arena for hills, ruins and scattered
boulders — all cut from the same stone volume, which is the claim `stone` was
written to support finally being cashed.

`art::terrain` says only *where the ground is and which way it faces*. It makes
no colour and no texture; the rock comes from sampling the stone volume at the
surface. Slope decides what can rest where — loose rock has already rolled off a
steep face, and soil does not stay on one either, which is why steep ground
shows bare stone. Scattering is a jittered grid rather than independent random
points, because independent points clump and leave holes; that is what random
actually looks like and it reads as a mistake.

It is a **look** toggle and nothing else. Collision geometry is a constant in
`sim::arena` and neither scene touches it, so a fighter on a hillside is really
standing on the arena floor with the walls hidden. The button says so, because a
world you can walk through is the kind of thing someone reports as a bug.

### The hills are where the hybrid argument gets cashed

At four metres between vertices, the stone volume's own structure — joints a
metre or two apart — is entirely below the sampling rate. Asking the volume for
colour at each vertex correctly returns the rock's *mean*, and the hills came
out as smooth dunes with every joint averaged away.

The volume is still right for the large scale: it says which hillside is pale
and which is stained, across hundreds of metres, uniquely. What it cannot do at
that vertex spacing is grain. So a small tiling texture goes underneath and the
vertex colours multiply it — repetition invisible, because a tile with no large
features has no period to see. This is the split that was predicted when the
bake-on-the-processor decision was made, arriving exactly where it was expected.

Two bugs on the way, both worth keeping:

**A vertex colour is a tint, not an albedo.** Bevy multiplies the base colour
texture by the vertex colour, so handing it the rock's actual reflectance
multiplies two albedos together — a 0.09 texture times a 0.10 tint is 0.009, and
the hills came out about ten times too dark.

**A mesh carrying its own world-space coordinates still needs repeat
addressing.** The material builder decided whether to sample with repeat from
how many times it was *told* the tile repeats, and a mesh that encodes the
repeat in its own coordinates says "once". The sampler clamped and the whole
landscape got a single stretched texel — which looks exactly like the texture
having failed to load.

### Marble, and what a ruin needs

Marble is limestone recrystallised until nothing of the original grain is left.
Its whole character is the veining, and veining is *not* banding — it is
impurities smeared into wandering seams while the rock flowed. Which is the
shape the joint system already makes, so marble's veins are joints with the
width turned down and the roughness turned up. No new mechanism.

It is pale, which is a deliberate exception to the world staying dark: a ruin
only reads as a ruin if it is obviously not the hillside it is standing on. It
earns that by having almost no chroma, which is the half of the palette rule
that actually matters.

**A group per site, not a column per site.** A single shaft on a hill is a
monolith. What says *ruin* is a row at different stages of falling down — one
nearly intact, one snapped halfway, one down to a footing, and the drums that
came off them lying where they rolled. The intact one is what tells you what the
stumps used to be; without it the stumps are cylinders.

And boulders are spheres pushed around until they stop being spheres. An
icosphere is a ball and scaling it unevenly gives an egg; what makes a rock a
rock is facets and hollows at several scales, which is the same noise the stone
volume is built from applied to the radius instead of the colour.

### One measurement that corrected me

The terrain test first asserted that ridged noise makes a landscape
*bottom*-heavy, on the reasoning that ridges are narrow high ground over broad
valleys. Measured, it goes the other way: folding concentrates gradient noise
near its zero set, so a ridged landscape is broad high ground cut by narrow
hollows. The look was right and the explanation was not. What actually
distinguishes ridged from rolling is **curvature** — how sharply the surface
bends — by about seventy per cent, and that is what the test checks now.

## Next, in order

1. **Fine detail on the arena's own surfaces.** The hills got a tiling detail
   layer under their volume colour; the walls have not, and at eight texels per
   metre a wall shows structure and no grain.
2. **Collision for the landscape**, if the terrain ever stops being a showcase.
   Today it is scenery over the arena's own geometry and the button says so.
3. **Per-move keys for the moves that carry a class's identity.** Derived clips
   are correctly timed and recognisably attacks; they do not know the Reaver's
   Guillotine is a downward chop with a shadow behind it.
