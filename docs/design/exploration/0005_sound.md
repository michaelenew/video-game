---
status: exploration
started: 2026-10-08
---

# 0005 — Sound, derived

*The game has none. Three creatures were built around the hole. What would a
sound system look like that is built the way the arenas' skies are: from a
few numbers, judged on a sheet, with the tweaks folded back?*

## The hole, as the documents found it

[bestiary.md](../bestiary.md) §"The game has no sound" lists three stand-ins:
the Pair's screen-edge glint for a snarl, the Veilstalker forbidden from
decloaking off-screen because a strike from behind with no sound is a hit from
nowhere, and the Sandmaw — a creature *about* noise — drawing its noise. Every
one is a visual patch over an auditory fact, and "unanswerable" in the fight
report counts a tell that was never on the victim's screen, which is the
report saying, in its own words, that a tell has to be heard when it cannot
be seen.

Versus has the same hole with less excuse. The impact freeze of 2026-09-26
gave blows weight you can *see*; weight is mostly heard. A parry window of
four frames is "findable" only if finding it is confirmed, and the clearest
confirmation a game can give is a sound that nothing else makes. A startup
you can hear is a startup you can answer from behind.

## The thesis: a sound is an excitation shaped by a resonator

Everything the game needs to make is one of a very small number of physical
events, and each is described by numbers the simulation already has.

**A blow** is a short burst of energy (the contact) driving a body that rings
in a few modes (what was struck). The burst's length and spectrum say how
*hard* and how *sharp*: a hammer is long and dull, a sword short and bright.
The modes say the *material*: flesh is a damped thud with almost no ring,
stone is a few high-Q partials, a shield is bright and metallic, snow swallows
everything. The modes' *pitch* says the *size*: a knee-high gnawer rings an
octave above a fighter, a creature's plate an octave below. So a blow is four
numbers — weight, sharpness, material, size — and the simulation knows all
four at the moment it resolves the hit: the move's damage and knockback, its
shape, what it touched (`state::hitbox` meets a body, a stone, a shield, a
critter's box, a creature's part) and how big that was.

**A telegraph** is a rising sound exactly as long as the startup. That is the
whole design: the sound *is* the frame data. A thirty-frame windup is a
half-second rise; a twelve-frame one is a snap you cannot answer, and should
sound like one. Filtered noise with a sweep — a whoosh — for a swing; a growl
(a pulse train through two or three formant filters, their frequencies set by
the creature's size) for an animal gathering itself. Where the sound comes
from is where the body is, so a creature winding up behind you is heard
behind you, and the report's "off screen" column stops being a death
sentence.

**A footfall** is a blow of the body's weight on the floor's material. The
arenas already carry a material under every point (`Material`: ground, grass,
rock, stone, sand, snow, ash, peat, water, wood), so footsteps, landings and a
forty-metre colossus's walk are the same function as a hit, with the floor as
the thing struck. The Siegeshell's walk is a clock; a clock you hear is one you
can jump to with your back turned.

**Fire** is crackling: low noise with sparse pops. **Stone erupting** is
rumble under gravel. **Blood** is wet: a short, dull, low-passed burst with a
little pitch drop. **Wind** (the Elementalist's air, the Galewing's downwash)
is broadband noise through a slow filter sweep. None needs a recording; each
is a handful of lines of synthesis with the game's numbers as its inputs.

This is the same move the arenas made: a sky from one colour. Here, a sound
from four numbers. And the same payoff: the next creature gets its voice from
its size and its moves' frame data, and anything a person changes by hand
about one creature's sound is a tweak to look for the rule in.

## Why synthesise rather than record

- **There is no asset pipeline and nobody to run one.** The repository's
  thesis is that the cost that matters is the next thing, not these; a
  recorded library is a cost per sound forever.
- **The numbers already exist.** A recorded hit cannot be scaled by the
  Oven's knockback knob; a synthesised one is a function of it.
- **It can be judged on a sheet.** `cargo run -p look --example skies` draws
  every sky in a second. `cargo run -p sound --example sheet` can render
  every cue to a WAV and a spectrogram in the same second, and a person can
  listen to the whole game's voice in a minute without starting it.
- **The browser build gets it for free.** No downloads, no fetches: the bank
  is rendered at startup, in a few hundred milliseconds, on both platforms.

The risk is that synthesis sounds like a 1980s arcade. The answer is the
same as the palette's: judge it *lit* — through the renderer's response
curve — which here means through a speaker, with a spectrogram beside it, and
tune the resonators until a thud is a thud.

## What rollback does to it

The simulation is re-run: up to eight frames, inside one picture, whenever a
remote input arrives late. A sound played from a frame that is then taken
back is a sound for something that did not happen. The usual answer in
rollback fighters is the honest one: **play the sounds of frames as they are
first simulated, and never again for the same frame.** A hit mispredicted is
rare (inputs are mispredicted constantly; *outcomes* seldom), and a spurious
thud now and then is the price every rollback game pays. The alternative —
waiting for confirmation — adds the whole rollback window of latency to
every sound, which is worse: a parry confirmed 130 ms late is not a
confirmation.

So: cues are derived by **diffing** the world before and after each tick —
the way the fight report and the replay judge already do — into a fixed-size
list (no allocation, in the frame path), and the game plays a cue only when
its frame is newer than any frame already played. Anything that affects
gameplay stays in the snapshot; a sound affects nothing, so none of this
touches `World`.

## Where it lives

A `sound` crate beside `look`, depending on `sim` and nothing else: the
cues (what the simulation's transitions mean), the patches (what numbers
each cue is), the synthesis (a few oscillators, a noise source, two-pole
resonators, envelopes), and the renderer to samples. Floats are fine: nothing
here feeds the simulation. `crates/game/src/sound.rs` is the Bevy half: a
bank of rendered sources, a listener on the camera, and a system that plays
cues at their positions. Nothing in it decides what a sound should be.

## Next

Build the crate and the sheet first; listen; then the game half. The first
cues: hits by weight and material, the parry, blocks, footfalls on every
material, jump and land, the dodge, each class's mechanic (the shield's
throw and plant, a stone's eruption, the fire pillar, a pool, the shadow's
send), and a telegraph per creature move sized from its startup. Then the
three creatures that were built around the hole get their stand-ins'
reasons revisited: the glint stays (a glint and a snarl are better than
either), the Veilstalker's off-screen rule is re-asked, and the Sandmaw's
drawn noise gets its sound.
