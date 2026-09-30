---
status: action plan — opened 2026-09-30, M1–M6 carried out the same day; the human checkpoints C1–C5 are still to be played
opened: 2026-09-30
implements: ../elementalist-v2.md
---

# Elementalist v2 — action plan

This is a complete brief for one implementer. Read [`../elementalist-v2.md`](../elementalist-v2.md)
first: it is the specification, and this document is how to get there and how to know when you
have. Then [`../kits/elementalist.md`](../kits/elementalist.md) for what is built,
[`../controls.md`](../controls.md) for the grammar the new inputs join, and
[`../exploration/0001_control_budget.md`](../exploration/0001_control_budget.md) for why these
four inputs. Read [`../README.md`](../README.md) and [`../../../CLAUDE.md`](../../../CLAUDE.md)
before touching code.

## The objective, in one paragraph

Give the Elementalist four more inputs and make her convert space into damage. Her two
placement buttons charge when held at the crawl — the pillar into a Strike, Raise into Fissure.
Updraft and Downdraft on `F` move her and what is around her up and down, and Downdraft into
fire turns the fire into an expanding ring. Cinder spray on middle click is fire in the air for
her shots to fly through. Quake on a side button and Tremor on `R` are one effect placed two
ways. A dodge into a stone breaks through it. The class is done when a person playing her says
that a full charge felt earned when it landed and deserved when it did not, and that the air
row felt like magic rather than a pause.

## Ground rules

- **`crates/sim` has no floats, no allocation, no I/O, no dependencies.** Every new effect is
  a bounded slot in the snapshot; the 4 KiB cap and `budget.rs` are the checks that matter.
- **Every feel number is an Oven knob**, baked with `cargo run -p sim --bin bake_tuning`.
  Re-bake after every merge from `main`. Never renumber slots.
- **Aiming lives in `aim.rs`.** Cinder spray is a skillshot whose range sphere is the burst;
  Quake is a grounded path; Updraft and Downdraft are not aimed; the Strike and Fissure are
  channels whose line is read every frame and locked on release, as the Grasp's is. Nothing
  here needs a new kind of ray. If it seems to, change `aim.rs`.
- **Hold is a dimension.** A charge deforms one number of the move it began; it never throws a
  different move. The startup is the tap window.
- **The overlay draws what the hit test uses.** The Strike's column is the pillar's two
  volumes; the fire ring is drawn at its live radius; the Updraft cylinder is drawn at the
  radius it lifts.
- **One implementation for Quake and Tremor**, differing in where the centre is.
- **Do not touch any other class** beyond what a shared function forces. The four new bits
  exist for everyone; only she reads them in this thread.
- **Design docs are the specification.** Update `kits/elementalist.md` as each milestone lands
  and write a feel-log entry per milestone.
- Before every push: `cargo fmt --all`, `cargo clippy --workspace --all-targets`,
  `cargo test --workspace`.

## How you will know — the two kinds of evidence

**Measured** evidence is tests, the frame table, and the instrument below. **Felt** evidence
comes from the human checkpoints: stop, push, write a feel-log entry whose verdict is *open*,
hand over a play script, and iterate one knob at a time on what the person says.

**Honesty rules.** A passing test is never "it feels right". Built with unanswered felt criteria
is *built, unverified*. A criterion that needs a `feel.rs` relationship loosened is a design
finding: stop, write it down, ask. Record what you tried and reverted.

## Build the instrument first

Add `cargo run -p sim --bin elemental` beside `frametable` and `essence`. It runs scripted
exchanges and prints:

| Script | What it prints |
| --- | --- |
| `charge` | For holds of 0, ¼, ½, ¾ and full: the Strike's damage and the pillar's remaining burn; the crack's length; her position drift at the crawl; and the **safe charge distance** against a 7.0 m/s walk |
| `lift` | Updraft's apex for each class standing in the cylinder, and for a stone; Downdraft's descent time from a full hop |
| `ring` | The air ring's push on a dummy at three distances; the fire ring's radius over time and damage to a dummy at three distances; that the source pillar or patch is gone |
| `spray` | Cinder burst distance with nothing in the way and against a stone; an Air bolt's damage plain and lit; a Gale's damage plain and lit; a stone lit by the cloud |
| `quake` | Stagger on a walking dummy and none on a standing one; eruption damage; the stone's position — at the crosshair for Quake, under her for Tremor, and her height after Tremor |
| `break` | Structure count before and after a dodge into a stone; rough-terrain slow on a dummy crossing where it stood; burning ground when the stone was lit |

Every criterion below is a number this bin prints. `frametable` gains the safe charge
distance beside the two charged moves' rows.

## Milestones

### M1 — The four inputs, and fire in the air

**Build.**
- Four bits in `Input`: `SIDE_A`, `SIDE_B`, `F`, `R` (names to taste; they are the last four
  free). `read_input` reads the two side buttons and `F`/`R`, with keyboard stand-ins for the
  side buttons, on the desk and in the browser. The manual's tables and the web controls panel
  list them.
- **Cinder spray** on middle click, both rows: a skillshot that bursts at its range sphere or
  on the first thing the line meets. Airborne burst leaves a hanging **Cinder cloud**, the
  first effect with a height; grounded burst leaves a burning patch. Both are fire areas.
- **Ignition**: an Air bolt or Gale whose path crosses a fire area — pillar, patch, cloud —
  comes out lit: the bolt with more damage and a small burst on hit, the Gale burning and
  leaving embers along its path. One branch in `gust`'s dispatch.
- **Knockback pass** on Air bolt and Gale; the Gale **pushes stones** it passes, by the beam's
  kick rule (speed relative to target).

**Measured criteria.**
- `spray` prints a burst at the range sphere with nothing in the way and at the stone with one;
  lit damage above plain for both shots; a stone under the cloud lit.
- The Gale moves a stone and the stone's damage to a dummy follows its speed.
- The frame table prints an `aimed` kind for Cinder spray; `one_aim.rs` and `knobs.rs` pass.
- `budget.rs` passes in `sim` and `view` with three clouds live.

**Human checkpoint C1.** Play script: *Jump, spray ahead, fire an Air bolt through the cloud.
Spray behind you while fleeing. Gale through your own pillar into a stone.* Questions: Did the
air row feel like it had teeth? Did the cloud read as yours? Is the flare a decision?

### M2 — The charges

**Build.**
- `channel` becomes per-move in what it chooses: the Grasp's reach, the Strike's **fraction**,
  Fissure's **distance**. The startup is the tap window: a release inside the startup throws
  the ordinary move.
- **`Q` held — the Strike.** Fire gathers in her hands (a pose and a glow that scale with the
  hold). Release lands a column in the pillar's two volumes at the crosshair's point for a few
  frames, dealing the held fraction of the pillar's total burn at once, and leaves a pillar
  worth the remainder. A Strike on a stone's top lights it (the flag lands in M4; here it is a
  no-op with a test that will flip).
- **`E` held — Fissure.** The patch churns and holds at the rise's first half. Release runs the
  crack along her look direction for a distance set by the hold, staggering and slowing along
  it, and erupts the stone where it stops. The crack's length is **rough terrain** for a few
  seconds — a new grounded slow effect, reused below.
- Crawl through both holds; live aim; lock on release. `frametable` prints the safe charge
  distance.

**Measured criteria.**
- `charge` prints Strike damage monotone in the hold, remaining burn monotone the other way,
  and their sum within a knob of the pillar's total.
- The crack's length is monotone in the hold, from Raise's own spot at a tap to full reach.
- A tap of `Q` is byte-for-byte the pillar as built; a tap of `E` is Raise as built.
- `every_attack_is_punishable_on_block` and `committed_moves_are_more_punishable_than_pokes`
  pass at every hold; a new `a_hold_deforms_one_axis` fails if a charged release ever throws a
  move of a different kind.
- Safe charge distance at a full hold is between five and nine metres at first values.

**Human checkpoint C2.** Play script: *Raise, then hold `E` and run a crack at the dummy. Hold
`Q` for a second and strike where it stands. Have the dummy walk at you and charge anyway.*
Questions: Did the full Strike feel earned? Did being hit out of a charge feel deserved? Could
you tell how long you had?

### M3 — Updraft and Downdraft

**Build.**
- **`F` standing — Updraft.** A cylinder centred on her (offset knob at zero). Medium wind-up,
  then everyone and every stone inside is launched, each by their own class gravity, her
  included.
- **`F` airborne — Downdraft.** The same cylinder under her; everything in it driven down. On
  landing while it blows: over plain ground, an **air ring** that shoves nearby fighters
  outward; into any fire area, the fire is **consumed** and a **fire ring** expands outward
  from her — fast, low, brief, damage and a small outward knockback.

**Measured criteria.**
- `lift` prints apexes ordered by class gravity, hers a full hop, the Bulwark's under a body
  height; Downdraft's descent faster than a plain fall.
- `ring` prints the air ring's push falling with distance; the fire ring's radius growing to its
  cap on the knob's clock; the source gone the frame the ring starts.
- Every air-row move changes a height: a new feel test walks her air row and fails on one that
  does not.

**Human checkpoint C3.** Play script: *Updraft beside the dummy. Get juggled by the bot, then
Downdraft. Plant a pillar, jump, Downdraft into it.* Questions: Did Downdraft feel like a way
out? Did the fire ring feel like spending the pillar or like losing it? Does Updraft compete
with the structure jump or sit beside it?

### M4 — Quake, Tremor, and fire on earth

**Build.**
- **Quake** effect: a patch that shakes (movers stagger, standers do not), then erupts
  (damage), then leaves a stone at its centre. `M5` places it at the crosshair; `R` places it
  at her feet and the stone carries her, and pops the stone she stands on if she is on one.
- **Lit stones.** A flag on a structure set by a Strike on its top, a Cinder burst beside it or
  a burning Gale; while lit it burns whoever stands on it and **bursts into burning debris**
  when kicked, pushed, broken or broken through. Drawn at the seams.

**Measured criteria.**
- `quake` prints a stagger on the walker, none on the stander, and the stone where the spec
  says, with her height raised after Tremor.
- A lit stone kicked by the beam deals debris damage plus burn; an unlit one deals debris only.
- The structure cap still holds with Quake, Tremor, Fissure and Landfall all placing stones.

**Human checkpoint C4.** Play script: *Quake under a bot that likes to walk. Tremor when it
closes. Light a stone the bot is hiding behind and kick it.* Questions: Did the shake read as a
"stand still" rule? Did Tremor feel like the structure jump with a cost? Did lighting the stone
feel like taking your cover back?

### M5 — The dodge into a stone

**Build.**
- The dodge, pointed at a stone within its reach, **breaks through**: she passes through where
  the stone stood, the stone is gone and its slot freed, and rough terrain is left there —
  burning ground if the stone was lit. The class redirect the Reaver's and the Blood mage's
  dodges already use.

**Measured criteria.**
- `break` prints the structure count down by one, the slow on a dummy crossing the patch, and
  burn damage when the stone was lit.
- `the_dodge_outruns_a_walk` still passes; the break-through covers no more ground than the
  ordinary dodge.

**Human checkpoint C5.** Play script: *Let the bot start a string on you with a stone behind
you. Dodge into it.* Questions: Did it beat the string? Did losing the stone feel like the
right price?

### M6 — Knobs, clips, done

**Build.** Nothing new. The knob pass; the clips the bake demands (both charge poses, the Quake
stamp, the Updraft palm, the Cinder throw, the break-through); the sparring bot reading the new
buttons if the hunt report can reach them.

**Done when:**
- Every measured criterion passes and `elemental` agrees with the tests.
- `kits/elementalist.md` describes the inputs, the charges, the air row, Updraft and Downdraft,
  Cinder spray, Quake and Tremor, lit stones and the break-through as built; `controls.md`'s
  Elementalist section and option table are current; the README's roster row, documents table
  and open questions are current.
- One feel-log entry per milestone with a verdict, including everything reverted.
- `cargo test --workspace` green, tuning baked, the web build runs the class with the four
  inputs.

## The iteration loop

1. Run `elemental` and the tests. Write down what the numbers say before forming an opinion.
2. Compare to the criteria and to what the person said. Change the one knob most likely to
   move the felt result. Bake.
3. Write or update the feel-log entry with the `elemental` summary after the change.
4. Push and hand over the next play script.

Stop conditions: a `feel.rs` relationship that would have to be loosened; a merge from another
thread that changes a shared function's meaning; a beat that cannot be hit without a new button
(there are none left). And one specific to this class: **if after C2 the person says a full
Strike does not feel earned, stop and report** — the charges are the centre, and a centre that
does not hold is a new document rather than a knob.

## Knobs you should expect to add

Under *Elementalist*: Strike full-hold fraction; charge length (each hand); charge crawl, if it
is not the committed speed; Fissure reach and rough-terrain duration and slow; Cinder burst
distance, cloud radius and life, patch life; lit bolt bonus and burst radius; lit Gale bonus;
Gale stone push; Updraft radius, lift, wind-up, and the facing offset (zero); Downdraft drive;
air ring push and radius; fire ring radius, speed, life and damage; Quake radius, shake length,
eruption damage; lit-stone life and burst damage; break-through rough-terrain size.

## Handoff

The pull request describes the class in the proposal's terms, lists every knob and its first
value, links the feel-log entries, and says which criteria are measured and which are felt and
answered. It does not claim the class feels right. It says what the person said.
