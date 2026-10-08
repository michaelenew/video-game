---
status: built 2026-10-08, unheard
proposed: 2026-10-08 (exploration/0005_sound.md)
built: 2026-10-08
---

# Sound — the voice, derived

The game had no sound. Three creatures were built around the hole
([bestiary.md](bestiary.md) §"The game has no sound"): the Pair's screen-edge
glint standing in for a snarl, the Veilstalker forbidden from decloaking
off-screen because a strike from behind with no sound is a hit from nowhere,
the Sandmaw — a creature *about* noise — drawing its noise. This document is
the voice they were waiting for, built the way the arenas' skies were built:
**from a few numbers, judged on a sheet, with nothing recorded.** The
exploration that led here is [exploration/0005_sound.md](exploration/0005_sound.md).

## 1 · The thesis

**A sound is an excitation shaped by a resonator, and the game's numbers
decide both.** Every sound the game needs is one of a handful of physical
events, and each is described by numbers the simulation already has:

| Event | What it is, acoustically | The numbers, and where they come from |
| --- | --- | --- |
| A blow | A burst of noise (the contact) driving the modes of what was struck | **weight** = the move's impact freeze as a share of the longest (`Move::hitstop`, 13 frames on Earthbreaker), blended with its damage · **edge** = the kit's weapon (a club or a blade) · **material** = flesh, hide, plate, shield, stone, the floor · **size** = the struck thing's height |
| A telegraph | A rise exactly as long as the startup | `Move::startup`; a creature's `Attack::startup` |
| A swing | A short fall, as long as the active frames | `Move::active` |
| A growl | A pulse train through formants, climbing as it gathers | the creature's size (head height); its move's startup |
| A footfall, a landing | A blow of the body's weight on the floor | the arena's `Material` under the foot; the fall speed |
| A parry | A clean chime nothing else makes | — |
| Stone coming up, breaking | Rumble under gravel; a stone's modes struck | a structure appearing, or gone |
| Fire, blood, wind | Crackle; a wet, dull drop; a band sweep | the effect that appeared |

The reference body is a fighter. A struck thing's modes scale as `1 / size`
from where a 1.8 m body puts them, so a knee-high gnawer rings an octave above
a fighter and a nine-metre animal rings below, with nobody authoring either.
Each material is written as its **modes** — a frequency, how long it rings,
how loud — and its **contact** — how bright, how long, how much of a heavy
blow's low thump it carries. A ring *time* rather than a Q, because a Q at a
frequency is a ring time (`Q = π f τ`) and the time is what the ear hears: a
shield rings for half a second whatever note it rings at. Flesh rings for
twelve milliseconds, which is to say it thuds.

**Why this and not recordings.** The repository's thesis is that the cost that
matters is the *next* thing. A recorded library is a cost per sound forever and
needs a pipeline nobody will run; a derived one gives the next creature its
voice from its size and its moves' frame data. It can be scaled by the Oven's
knobs, because it is a function of them. It can be judged on a sheet. And the
browser build gets it for free: the bank is rendered on the spot, on both
platforms, with nothing to download.

## 2 · Where it lives

```
crates/sound/src/synth.rs    The kit: noise, oscillators, a two-pole resonator, envelopes
crates/sound/src/patch.rs    The instruments: a Strike, a Whoosh, a Growl, a Ring, ... and Patch::render
crates/sound/src/cue.rs      What the simulation's transitions mean: cues(before, after) -> Cues
crates/sound/src/spectrum.rs An FFT, a spectrogram, a centroid: how the tests and the sheet look at a sound
crates/sound/examples/sheet.rs  Every sound, as WAVs and one picture
crates/game/src/sound.rs     The Bevy half: a listener on the camera, a source per cue, nothing decided
```

`sound` depends on `sim` and nothing else, and floats are fine in it: nothing
here feeds the simulation. `patch::Patch::render` is **the one place a
waveform is made**, and `cue::cues` **the one place the simulation is read as
sound**. The game crate plays what it is handed and decides nothing about what
a sound should be — the rule `look` already set for colour.

## 3 · The sheet

```
cargo run -p sound --example sheet
```

renders every instrument at a few parameter values each — a blow by weight, by
edge, by what it struck, by the size of the animal; a footfall by floor; a
telegraph by startup; a growl by throat and by startup; the parry, stone, fire,
blood and wind — to a WAV per cell in `target/sound-sheet/` and one picture of
spectrograms (time across, pitch up on a log scale, loudness as brightness).
A second's work. **Listen to the WAVs, look at the sheet, change a number in
`patch.rs`, run it again.** That is the loop; a sound that needs the game
running to be judged is a sound nobody tunes.

The tests in `patch.rs` hold the instruments to their physics rather than to
numbers: a heavier blow is louder and longer; a bigger body rings lower; an
edge is brighter than a club; a shield rings longer than flesh; a telegraph is
loudest in the last tenth before its cut and quiet after. Each was wrong in
the first build and the sheet showed where.

## 4 · Under rollback

The simulation is re-run — up to eight frames inside one picture — whenever a
remote input arrives late. **A frame is sounded once: the first time it is
simulated, never again when it is re-simulated.** So a hit that was predicted
and then taken back has made its sound. That is the price every rollback game
pays, and the alternative, waiting for confirmation, would put the whole
rollback window between a parry and its chime — a confirmation 130 ms late is
not one. Inputs are mispredicted constantly; outcomes seldom.

Mechanically: cues are read off every tick as it is advanced (inside the tick
loop, since a picture can carry several ticks), into a fixed-size list with no
allocation; the game keeps the newest frame it has sounded and drops cues from
any frame at or below it. The rewind key `[` lowers that mark on purpose, so
stepping through a moment again sounds it again. Nothing here touches `World`:
a sound affects no gameplay, so it is not in the snapshot.

## 5 · In the game

A listener sits on the camera, ears a head's width apart, and each cue is a
spatial source at its place in the arena, so a creature winding up behind you
is heard behind you. Distance is scaled down for the mixer (an arena is thirty
metres across and the camera a dozen back) and the fall-off is the game's own:
full to eight metres, a quarter at forty. **Page Up and Page Down** set the
volume, a tenth a step, kept with the sensitivity and the field of view.

The Linux desktop build now needs ALSA's headers (`libasound2-dev`;
`./scripts/setup-tools.sh desktop`). The browser build needs nothing new.

## 6 · What this changes for the three creatures

Each of the stand-ins can now be re-asked, and none is removed here:

- **The Pair's glint** stays. A glint and a snarl are better than either, and
  the growl is already heard from the cat's head, which is where the glint
  points.
- **The Veilstalker's rule** — never decloak off-screen — was written because
  a hit from behind had no sound. It has one now (the growl, as long as the
  spear's startup). Whether the rule relaxes is a decision for after it has
  been heard, not before; `VEIL` questions in [review.md](review.md).
- **The Sandmaw's drawn noise** is still drawn. Its sound is the fighters'
  own footfalls and landings — louder on sand by the material table — which
  is the first thing the creature is listening to anyway.

## 7 · Open, and unheard

Nothing here has been heard by a person. The sheet was judged by its
spectrograms and by the tests that measure it. The first things to listen for:

1. **Is a thud a thud?** The flesh strike at weight 0.5, then at 1.0. If it
   reads as a 1980s arcade, the fix is in the contact's brightness and the
   thump's level in `Material::burst`.
2. **Does the parry chime stand out** from everything else at once, on a
   speaker, in a fight? It is the one sound that matters most and the only
   pure tone.
3. **Do the growls read as size?** The six on the sheet's growl row should
   sound like six animals, not one pitched up and down.
4. **Footfall cadence** is a fixed period from the frame count, not the gait
   clock the snapshot does not keep. If feet and steps visibly disagree, the
   clip's foot contacts are the right source.
5. **Volume and fall-off**: a dozen metres from the fight should still be
   loud, forty should not be silent.
6. The **three creatures** above, in that order.
