---
status: proposed
decided: 2026-09-09
sources: docs/archive/combat-design/elementalist-skills.md, docs/archive/combat-design/class-builds.md
---

# Elementalist — kit

> **v2 is built, 2026-09-30, and unplayed** — [../elementalist-v2.md](../elementalist-v2.md) is
> the decision, [../plans/elementalist-v2.md](../plans/elementalist-v2.md) the brief it was built
> to. Four inputs, a hold-to-charge on `Q` and `E`, Updraft and Downdraft, Cinder spray, Quake and
> Tremor, lit stones and a dodge that breaks through a stone: §"v2 — space into damage" below
> is what each does as built, and `cargo run -p sim --bin elemental` prints the numbers.

> **On three clicks, 2026-10-09 — built, unplayed.** From the first playtest: the Champion was
> the class that held together, because his three clicks *are* the class, each with an air move
> and a jump attack. She now has the same grid. The sections below this one describe the moves
> as they were built before it; **where they name a key, this table wins.** The direction is in
> [../exploration/0008_elementalist_on_three_clicks.md](../exploration/0008_elementalist_on_three_clicks.md),
> and §"On three clicks" below is what was built.

**Identity.** Terrain author. You build the battlefield, then combo through what you built.
Ranged control that creates its own targets.

## On three clicks — built 2026-10-09

| | Left: earth | Middle: fire | Right: wind |
| --- | --- | --- | --- |
| **On foot, tap** | Raise | Fire pillar | Air ball, small |
| **On foot, held** | Fissure | Strike | Air ball, grown |
| **In the air** | Landfall | **Fire carpet** | Gale |
| **Leaving the floor** (space + click) | **Earth jump** | **Fire fountain** | **Updraft** (a **Thermal** in her fire) |

| Key | On foot | In the air |
| --- | --- | --- |
| `Q` — the weak push | Bolt | Bolt (the Air bolt is unbound since the second playtest) |
| `E` — the strong push | Cataclysm | Cataclysm |
| `F` | Cinder spray | Downdraft |
| Side button | Quake | Quake |
| `R` | Tremor | — |

**The clicks are what she makes and the keys are what she does to it.** Every click is a tap
and a hold on the floor. Which button is which verb is written once, in
`sim::moves::elementalist::keys`, and every test and bot presses through those names.

**The takeoffs** use the Champion's window: space down on the floor holds it open, and a click
in it — or in the first few frames of the jump, which put her back on the floor for it — is a
takeoff. The window and the trip live on the fighter (`Player::rise`), not in her mechanic.
Earth and fire take off from the floor only. **The Updraft also goes from the air, once per
trip, with space held and right click**: it only ever pushes up, so it keeps her run, and it is
what stretches a jump. Its startup no longer slows her (mobility 100).

**Air ball** (`EffectKind::AirBall`). The press raises a ball where the crosshair meets the
floor; holding grows it, visibly, from the tap's size to the full one over the hold; the
release sends it flat toward where the crosshair meets the floor then (`aim::racing_path`, the
line Fissure runs, read for its direction only). Sent, it **shrinks at a steady rate**, and its
**speed follows the size it is now**, so it slows as it shrinks and peters out rather than
stopping. A bigger ball is faster and lasts longer, so how far it goes grows with roughly the
square of how big it was let go (`a_ball_held_twice_as_long_goes_much_further`). Anybody
standing in it is moved with it — her included — and can walk inside it; anybody who jumps
inside it is given its speed, so they leave with it. It holds nobody once it is smaller than
`Air ball, holds a body down to`, so a rider is put down a moment before it vanishes. It
shrinks faster while it carries anybody (`Air ball, shrinks faster carrying`), which is the
answer 0008 leaned to on a charge that moves her: carrying is paid for, and the long trips come
from what she does after she jumps off. It climbs a step up to half its own size. It hurts
nobody. One at a time. *Changed after the second playtest — see below:* off an edge it sinks
rather than drops, a wall or a stone knocks it off rather than stopping it, and she steers it.

### After the second playtest — 2026-10-09

What the person reported from three replays, what the replays showed, and what changed.
`tests/elementalist_playtest.rs` holds each as a property.

- **On a stone is on it.** On a stone that is still moving — an eruption's tail, the earth
  jump's stone coming to its stop — she kept the speed it had carried her at, drifted a hair
  clear of the top, and read as airborne every few frames; a left click on one of those frames
  was Landfall, which slammed her back to the floor. Now a body that was standing on a stone and
  is not leaving it faster than half a jump is put back on its top at its speed
  (`stones::resolve_body`). A jump still leaves it.
- **A takeoff a frame after leaving a stone leaves from the stone.** Space a frame before the
  click put her back on *the arena's* floor for the takeoff — the ground under the stone — and
  she was pushed out of the stone's side. The takeoff window now remembers the height of what
  her feet were last on (`Rise::floor`), and puts her back there; off a stone, the stone
  shatters under the jump as it should.
- **The stone under her feet, and off it at once** — the person's find, kept on purpose.
  Running, look straight down and left-click: Raise puts a stone exactly under her feet. Space
  and left click a frame later is then an earth jump *off a stone*, so the stone shatters and she
  gets the bigger jump (`Earth jump, off a stone`, ×1.35 of her rise, about 23 m/s up against
  17) — and, unlike the plain earth jump, which costs some of her run, it keeps all of it. The
  look straight down is only so the crosshair puts the stone under her; flicking up afterwards
  is for where she is going. Two presses a frame apart and a structure slot, for a long fast jump
  forward. `the_stone_under_her_feet_and_off_it_at_once_is_the_big_running_jump`.
- **`Q` is the Bolt in the air too**, as `E` is Cataclysm in both rows. The Air bolt has no
  button for now.
- **A stone she did not point down at stays at her level** (`aim::grounded_kept`, hers only).
  Raise clicked out past an edge, or at the side of an island across a pit, put its stone on the
  floor far below — often a course's void. Now the edge of the range, and the floor under a wall
  she pointed at, come back toward her to the last footing no more than
  `Placed, at most below her` (2 m) under her own (`aim::kept_up`). A crosshair *on* the floor
  below still puts it there. A Fissure held from a stone up top runs along that ground and stops
  at the edge (`aim::kept_along`), so its stone erupts at the lip, not in the pit; one started
  down there runs down there. Landfall's slab never comes up off a ledge (`aim::planted_ahead`):
  past one it comes back to her level, and with no footing between there is no slab. The Reaver's
  send, the Black spike and Judgement keep the plain rule — the send's refusal past an edge was
  decided from play on 2026-10-04.
- **The Air ball rolls off edges**: past one it keeps going and sinks at
  `Air ball, sinks off an edge` (2.5 m/s), holding up whoever is in it, until it meets the floor.
  **She steers it** by walking sideways in it — A and D, looking where it goes; the sideways
  share of her walk bends its heading (`Air ball, steered by her walk`). **A wall or a stone
  knocks it off** rather than stopping it: it turns off the surface and keeps
  `Air ball, size kept off a wall` (0.7) of its size, so of its speed and what is left of its
  life.

**Fire carpet** (`EffectKind::FireCarpet`). A strip of fire laid out from a little ahead of her
along the line the crosshair solved (`aim::skillshot_path`), its length the move's reach and
its half-width the move's radius. It hangs for `Fire carpet, hangs for` and burns whoever
touches it, on the tick. **It is fire**: a shot flown down it comes out lit
(`a_gale_thrown_straight_down_the_carpet_comes_out_lit`), and a Downdraft landed in it is a
ring of fire. It is drawn as a stream of flames carried from the near end to the far one, so
the fire is seen being pushed outward. One at a time. *Known from the start to be awkward at
speed — she can outrun a carpet she has just laid — and built anyway so it can be played and
something better found.*

**Thermal.** Whenever her Updraft's column and her own fire share space, whichever came first,
the column throws her up at `Thermal, lift` instead of the Updraft's lift — once per column. If
the fire is a carpet the Thermal also throws her along it at `Thermal, push along a carpet`,
and the carpet is used up. Fire she can make and use at once: the Fire carpet in the air, then
space and right click into it; or the pillar at her own feet, then space and right click in
its base.

**Fire fountain** (`EffectKind::Fountain`). A burst at her feet as she leaves the floor — the
move's own hit, once per body, at the move's radius, for `Fire fountain, burst lasts` — and a
wash of fire left standing where she took off, burning on the tick for `Fire fountain, burns
for`. A jump about as high as an ordinary one.

**Earth jump.** An ordinary jump that brings a stone up with her. What it does depends on what
she is standing on:

- **The floor.** A stone comes up with her (`Structure::brought_up`): its top at her feet,
  leaving with a share of her rise (`Earth jump, stone keeps of her rise`, 0.85) and of her
  run (`…of her run`, 0.4), and falling more gently than she does (`Earth jump, stone falls
  at`, 0.55 of gravity). It is the gentler fall that brings it back up under her: measured,
  her feet meet its top about 27 frames in, at about 3.9 m, just short of her apex, and it
  carries her on up to about 4.4 m before it starts to sink.
  **Holding forward keeps it under her.** Nothing in this game slows a body in the air, and a
  held strafe adds only about a metre a second, so a stone that was merely a little slower
  would always end up under her or always behind her; neither is a choice. So while she is
  in the air holding forward (and only forward), her aloft stone keeps her run. Let go, or
  strafe, and it carries on at its own 0.4 of her run and drops behind her — an ordinary
  stone in the air to Gale, or kick, back down at somebody. Stood on, an aloft stone stops
  travelling and only sinks, so it does not slide out from under her. It spends one of the
  three. *This is a departure from the playtest note's "slightly less velocity", for the reason
  above; the tests are `a_straight_earth_jump_comes_down_on_its_stone` and
  `an_earth_jump_without_forward_held_leaves_its_stone_behind`.*
- **A stone on the floor.** It shatters outward under her (`debris::shatter`, a ring of the
  same pieces Cataclysm breaks a stone into) and the jump is bigger (`Earth jump, off a
  stone`).
- **A stone in the air** — the one she landed on. It is driven back into the ground
  (`Earth jump, stone driven down`) and shatters where it lands.

**Meteors.** A stone in the air driven down — off an earth jump, or by a shot aimed down at it
— is a meteor, and shatters where it lands (`stones::step` reports it; the world throws the
pieces). A stone on the floor cannot be driven down, as before.

**Cinder spray** kept `F`, the key the Updraft left, since the pillar took its click.

**Open, for play:**
- Whether the carpet can be used at all at speed, and what replaces it if not.
- Whether a full Air ball that carries her is too much reach for a charge (the courses rule:
  reach paid in execution, never in waiting). The carried shrink is the first knob.
- Whether the earth jump's meeting point reads — the stone arriving under her feet on the way
  up rather than at the top. `Earth jump, stone falls at` moves it. And whether "hold forward
  to keep it" reads, or wants a different input.
- The earth jump off her own stone straight after landing on it waits out the 30-frame
  repeat lockout (she lands about 27 frames after the first): pressed at once, it is an
  ordinary jump off the stone.
- The takeoffs reuse the Champion's window knobs (`Takeoff window`, `Rising attack, still from
  the floor`); a click pressed just *before* space is not turned into a takeoff for her yet.
- The sparring bot and the hunt's class layer press her buttons through `keys` and do not yet
  use the takeoffs, the ball or the carpet.

**Shape of every ability — settled 2026-09-11, from play.** Long, telegraphed startups;
devastating, large follow-through; and only *moderate* frames after, because the cost was
already paid on the front end. That last clause is the part that makes the class playable
rather than merely slow: a long wind-up you also pay for afterwards is a move nobody throws.

The telegraph is not a drawback to be minimised. It is what makes her terrain *fair* — the
opponent gets to see it coming and decide — and it is what makes landing one feel earned. It is
also the first thing the curve harness was built for: a structure now holds barely out of the
floor for the first half of its rise, then erupts, and the duration did not change.

## Mechanic — structures

Physical objects you spawn. Every ability has a second behaviour when it hits one.

- **Cap of three on the field.** Spawning a fourth collapses the oldest. That cap is the
  resource — structures are spent by being consumed in combos, and hoarding them costs you
  new ones.

> **Implemented** (`E`). One press raises one — they **climb out of the floor** over about a
> quarter second, because they are earth. Structures stand in the arena and **have no clock** — the cap is the
> only cost, exactly as written above. They briefly had a lifetime, which meant the fire
> pillar (gated on having one out) silently stopped working ten seconds after you raised one.
>
> **They are solid, and they move.** A stone stops whoever walks into it, holds up whoever
> stands on it, and is pushed by whatever arrives where it already is — up if the new stone
> comes up underneath, sideways if it comes up beside. One raised under another pops it about
> a metre clear; one raised under its *edge* flips it away instead of balancing it there. A
> stone thrown into another hands over the speed it was carrying and both come out slower, so
> a stone cannot be relayed the length of the arena through a row of them.
>
> **Standing on them is the class's floor.** Terrain you cannot get on top of is only cover. A
> stone still climbing carries whoever is on it at the speed its top is climbing — that is the
> seed of the mobility the class is meant to get, not the finished thing. It waits on moves
> that launch stones properly. (That ride was worth about 2.9 m against a 2.2 m full hop when
> this was written, and the jump was rebuilt after that. It is not a step any more; see below.)

### The structure jump, and the double — **the class's movement**

**Ride an eruption and press jump while it is still climbing.** The stone's top
is under her feet and coming up fast, the carry hands her its speed, and the
takeoff stacks on to that (`state::advance`). A stone can then catch her *again*
on the way up and hand her another takeoff, which is what makes this a technique
rather than a tall jump: **25.9 m** off one stone, against her own 5.0 m full
hop.

**The double is the real thing.** Cast two structures two or three frames apart
and time the jump while *both* are still coming out of the floor, and the
eruptions catch her in turn: **44.8 m**, on a three-frame window, for two of her
three structure slots and a setup that telegraphs itself twice.

> **Watching one.** `--dev` and press `G`: it plays the input rather than
> describing it, and `P`, `[` and `]` step through it in either direction with a
> readout under the crosshair saying what each frame is doing — where her feet
> are, how fast she is rising, each stone's age and climb rate, and the one
> frame that says **CAUGHT**, which is the stone overtaking her feet and handing
> the still-held jump button another takeoff. That frame is the technique; the
> rest is a jump.

There is a gradient behind that and it is the more interesting half. Two or
three frames apart is the peak; the payout falls away as the gap opens, and past
about nine frames the second stone arrives too late to chain at all. Nobody
designed that curve — it falls out of the shape of the rise.

> ⚠️ **Two things were done to this on 2026-09-17 and both were undone.**
>
> It was **deleted**, on the reading that a fighter who has already jumped off a
> stone should not be re-grounded by it — `../README.md` §4, *space while
> airborne does nothing*. That rule is about space doing something with
> **nothing under you**; a stone still climbing is a surface, and a body on a
> surface can jump off it. Restored the same day and specified in
> `stones::resolve_body`, with three tests that fail if it goes again.
>
> Then its **height** was cut, by slowing the eruption (the rise 14 → 17 frames,
> the lift kept 0.40 → 0.44). Put back on 2026-09-18: it played much worse, and
> the target it was tuned to had been computed against a baseline measured a
> quarter too low.
>
> **The stone knobs are not the way to move this.** The chain is a resonance
> between how fast she rises and how fast the stone grows — the eruption has to
> outrun her by just enough to catch her feet again — so the number of links is
> discontinuous in the tuning and not even monotonic. Three frames on the rise
> roughly halved the double while the single barely moved; raising *her* jump
> makes the single **fall**, because she outruns the stone that was going to
> catch her. [../feel-log.md](../feel-log.md) has the measurements. If these need
> to come down, somebody plays each setting rather than solving for a target.

**The cast-wide jump nerf did move them**, because every link of the chain is a
takeoff and a smaller takeoff is subtracted once per link. The double went from
58.8 m to 44.8; the single went the other way, 17.5 to 25.9, because the slower
takeoff lets the eruption catch her for an extra link. Against her own full hop
— which is what "special" means — both are further ahead than they were.

- **Contested, not owned.** Enemies can use them as cover, destroy them, and displace them
  short distances with attacks, but cannot combo through them nearly as well. See
  [../elementalist.md](../elementalist.md).
- **The rise is a telegraph with teeth.** The first half is the ground **churning** underfoot —
  a slight slow on anyone standing over it, felt before it is seen, and cleared by getting off
  the floor. The second half is the **eruption**: a little damage and a stagger, once per
  fighter, because a stone erupts once. Never the Elementalist herself — she raises them under
  her own feet on purpose.

> **Implemented.** The two phases come off the rise *curve* rather than a frame count, so
> reshaping the rise moves the telegraph with it. A warning that can drift out of step with
> the thing it is warning about is worse than no warning.

The two overlapping kit versions in the archive are reconciled here; where they disagreed
this kit takes the `elementalist-skills.md` version, which is the later document.

## Element loadout

**Earth is always equipped** — it is what generates structures. A second element slots
alongside it. This kit specifies **earth plus fire**; ice, air, and lightning are the
specialisation axis for later.

Input map in [../controls.md](../controls.md).

## Auto attack

> **Changed 2026-10-02, from play: both standing clicks hit only what she built.**
> The Bolt and Cataclysm pass through fighters, creatures and critters — no damage,
> no interrupt — and act only on her stones and on fire. A solid wall ends the line.
> With Cinder spray, the pillar, lit stones and three ways to raise a stone, the
> beam's job had become setting off what she put on the field, and in a crowd the
> poke kept landing on whoever stood between her and the burning stone she was
> aiming at. What reaches people now is what the beam sets off: the fire bolt out
> of a pillar, a lit stone's burst, the kicked stone, the debris, the tornado. The
> air row (Air bolt, Gale) is unchanged. `bolt::targets` is the list.

A **beam**, not a bolt. A short wind-up, and then an instant line from her hand
to *whatever the crosshair is on*, out to a short-to-middle distance. Nothing
travels, so there is nothing to lead and nothing to dodge once it is thrown —
what there is instead is a shot that goes exactly where you are pointing.

It is the game's one **skillshot** in the sense [../aiming.md](../aiming.md)
means it, and that document is where the rule lives. The short version: the shot
ends on the point the camera's ray through the crosshair reaches first, and when
that point is the floor it is raised **off that floor** by a fixed height — the
middle of a fighter standing there — so a shot aimed at the ground goes through
whoever is on the spot rather than into the dirt.

*Corrected 2026-09-14.* This said "raised to the height the shot leaves her at",
which is the same number on flat ground and nothing like it anywhere else: from
a platform, or from the air, it meant level over the plane **she** was on rather
than over the place the crosshair was on, and the shot sailed over everybody.
The rule was always meant to be a lift measured from the ground the ray met; the
code says so now and so does this.

> **Implemented** (`L`). What the line reaches **first** is the whole move,
> checked every active frame and spending the move's one hit on whatever it
> finds:
>
> - **a fighter — nothing, since 2026-10-02.** It was small damage and the
>   move they were winding up, with no stagger. The line now goes through
>   every body, so a fighter in front of a stone does not shield it;
> - **a solid wall** — the line ends there;
> - **a structure** — the stone is sent **along the line**: through the ground
>   when she is aimed down it, and up into the air when she is aimed above it.
>   The kick dies off over the back quarter of its travel rather than skidding
>   to a stop on friction alone, so a stone caught early in its flight hits like
>   a boulder and one caught late barely nudges anyone. What it deals to whoever
>   it is still moving fast enough to catch — damage and a stagger — is a
>   function of its speed **relative to the target**, the same quantity two
>   colliding stones already hand each other. The beam never reaches past the
>   structure: aimed through one, it does not also poke whoever is standing
>   beyond it;
> - **fire** — a pillar is a hazard, not a wall, so it does not stop the beam.
>   It **lights** one. A fire bolt leaves the pillar along the same line: fast,
>   small, long range, low-to-middling damage and a little stagger. It is the
>   one thing she throws that has a speed, and it starts *at the fire* — a bolt
>   that came out of her hand instead would make the whole interaction
>   invisible.
>
> **The range is the move's own.** The move table's `reach` is the max-range
> sphere the aiming ray stops at and its `radius` is the line's thickness,
> rather than knobs of their own: the beam *is* the move, and a second copy of
> its range would only be a number the frame table could disagree with. The
> shot is often shorter than the sphere, because it ends on whatever the
> crosshair found. The fire bolt is what carries it *past* that range, which is
> the trade a pillar buys — put fire between you and them and the poke stops
> being a point-blank tool.
>
> **Height decides this one.** It is the first fighter-on-fighter hit in the
> game where it does: a crouch ducks a shot aimed over the head, a shot aimed
> over a stone passes over it instead of kicking it, and someone standing on a
> platform is out of reach of a level shot and in reach of one pointed at them.
> Every other attack compares flat distance and says nothing about height.
>
> **What the shot looks like is what the shot is.** The line is drawn in the
> game as the thin cylinder it is, from her chest to wherever it stopped, and
> her body tilts on to it — spine, chest, shoulders and head — so a shot fired
> forty degrees up is thrown forty degrees up. Both come off `state::hitbox`
> and the aim in the snapshot, so the picture and the rule cannot drift.
>
> This replaces the version recorded in [../feel-log.md](../feel-log.md) as
> "the autos are due a pass": a flat circle at a fixed distance in front of
> her, which meant aiming up did nothing whatsoever, and a fire interaction
> that was an instant long-range hit rather than a projectile. The melee
> classes' equivalent pass, and the timing pass across all six, are still
> outstanding.

**Open, deliberately not built now.** `L` and `R` could both become autos with
longer, independent animation frames — enough that a single click reads as
committed on its own, but landing both in quick succession (left then right,
or the reverse) chains into a stronger combo that a single button mashed
twice cannot reach. The two windows would not overlap, so it is a skill
input — a real read-and-execute — rather than a way to double the DPS of
spamming one button. `R` is Cataclysm now and Raise is on `E`, so nothing
would have to be rehomed to do it; what it would cost is right click's heavy
and, with it, the reason `shift` + left carries Fissure. It is a kit-wide
control question, not an Elementalist one, and belongs with the rest of [the
open control questions](../controls.md#open-since-the-dodge-moved) rather than
being decided here.

## Abilities

### Raise — mechanic input
**Startup** fast · **Recovery** short · **Range** short · **Mechanic** spawns a structure

Spawn a structure at the cursor. Cast beneath yourself to launch into the air. Cast at an
existing uncaptured structure to kick it forward through the ground for low damage.

> **Implemented, at the cursor.** "At the cursor" is now literal — the stone comes up on the
> first thing the crosshair's line meets, out to Raise's reach. "Beneath yourself" is looking
> down, and the eruption carries you with it.
>
> **Her stones are not on that line** (2026-10-02, from play): it goes through them to the floor,
> so a stone is never stacked on another's cap — one aimed at the floor under a stone comes up
> beneath it and lifts it. A stone under the crosshair moving where the next one landed was faster
> than anyone could react to, and raising stones to spoil aim was a strategy. See
> [../aiming.md](../aiming.md#ground-and-everything-else).

### Fissure — `E` held
**Startup** medium · **Recovery** medium · **Range** long · **Mechanic** spawns a structure
at the point of impact

A skillshot that races forward through the ground and stops at the first enemy hit,
staggering them. Leaves a slowing field along its path for several seconds.

> **Built 2026-09-30, on a hold.** `E` tapped is Raise; `E` held past the stone's churn keeps
> it churning and becomes the crack's hold, and letting go races the crack out from the stone
> toward the crosshair's spot on the ground (along her facing until 2026-10-08) — two metres for a tap of a hold, ten for the full second — and the stone
> erupts at its end. It stops at the first body and hits it, and its line is **rough terrain**
> for four seconds, which slows whoever crosses it and never her. See §"v2".

### Quake — second mouse side button; Tremor — `R`
**Startup** slow, telegraphed · **Recovery** medium · **Range** medium · **Mechanic** spawns
a structure at the centre

A small area shakes immediately, staggering anything moving through it, then erupts after a
delay for moderate damage. The telegraph is the point — it is an area denial tool that
punishes movement, not a damage spell.

> **Built 2026-09-30, as one effect placed two ways.** Quake puts the patch where the crosshair
> is, out to nine metres; Tremor puts the same patch on her own feet, so the stone comes up under
> her and takes her with it — a jump she does not have to aim, out of a string she does not want
> to be in. Standing still in the patch is the answer; anything moving faster than a metre a
> second is staggered once. See §"v2".

### Fire pillar
**Startup** medium · **Recovery** medium · **Range** medium

A focused pillar of flame with a small staggering core and a moderate surrounding area,
planted wherever the crosshair is. It does not need a structure out, and does not touch one
if there happens to be one there.

> **Implemented** (`Q`). The pillar is two volumes rather than one: it starts narrow and
> short, then the **base spreads out** while the **column reaches up** and widens only
> slightly. The base is what catches someone walking past it; the column is what stops them
> jumping over. It burns everyone but the Elementalist, on a tick rather than every frame, and
> it stands long after her recovery frames are over.
>
> **Does not require a structure.** It briefly did — gated on having one out, the same way
> Ice blast and Flame spitter below are written to need one — which was wrong for this move
> specifically: nothing about a pillar of flame planted at the cursor has anything to do with
> a structure being there, and it does not detonate or otherwise touch one that happens to be.
> Earth plus Fire is the loadout, not a dependency chain where Fire only works after Earth has
> gone first.
>
> **One at a time, since 2026-09-25.** While a pillar of hers is burning, pressing `Q` does
> nothing: no startup, no eruption, no second pillar. Stacking them was a free win, a wall of
> fire at a button press each. It is the same gate the Reaver's shadow and lotus use
> (`moves::lingers`), so the repeat lockout starts when the pillar burns out rather than when
> it was cast. A tornado Cataclysm tore loose from a pillar is no longer a pillar and does not
> hold the button.

### Cataclysm
**Startup** slow · **Recovery** slow · **Range** long, skillshot

The right-click heavy. A long wind-up, thrown along the same line the auto follows, that
turns whatever field effect it meets into something worse rather than just damaging it.

> **Implemented** (right click). It reads the same beam as the auto and Fire pillar's
> targeting, and shares their aim -- point it, don't lock onto anything. What it meets
> along that line decides what it does, and neither answer invents a new hitbox to do it:
> both reuse something that already exists rather than detonating an instant bubble.
>
> - **A structure** is destroyed outright and thrown outward as several pieces of debris, in a
>   cone around the line Cataclysm was aimed rather than one blast that lands everywhere at
>   once -- a real cone standing in space, square to the line of effect however it is pitched,
>   not an arc swept flat around the world's vertical axis. Each piece is its own small
>   projectile with its own flight time, so what actually connects depends on how close you
>   were standing and whether you were inside the cone -- a shotgun rather than a bomb, and one
>   you can see coming rather than one that has already landed by the time you notice it. See
>   `crate::debris`.
> - **A fire pillar** is not damaged -- it is transformed. The same `Effect`, the same two
>   volumes a standing pillar already tests against, cut loose from the ground and sent
>   racing along the direction Cataclysm was aimed. It grows on exactly the curve the pillar
>   it came from was already growing on -- one that had barely erupted keeps widening as it
>   goes, one that was already mature stays that size -- rather than snapping to full size or
>   back to nothing the instant it starts moving. The first frame it reaches somebody it lands
>   a real stagger, the same eruption a fire pillar already throws the moment it is cast, and
>   that stagger is not a flourish: a fighter free to act sets his own velocity from the stick
>   every frame, which would cancel the pull below before it ever moved him. Stunned, his
>   velocity only decays, and the pull can win inside that window -- toward the tornado's own
>   live centre, for as long as he is standing in either volume, dragging him along with it
>   rather than merely burning him where he stands. Once the stagger runs out he is free again:
>   walking clear means outrunning the wide base, and jumping or air-dodging clear of the
>   narrower column above it is the faster way out. See
>   `crate::effects::EffectKind::FireTornado`.
> - **A fighter** is not met at all since 2026-10-02: the line goes through bodies, as the
>   auto's does, and a wall ends it. It was a real hit with its own stagger. What hurts
>   people is the debris and the tornado.
>
> The wind-up is long enough to be read and punished; the payoff is why you would still
> throw it. Earth plus Fire again: Raise or Fissure to seed a structure, then Cataclysm to
> decide whether it becomes a spray of debris or, by way of a fire pillar first, a moving
> hazard that keeps threatening the space after the swing is over -- the tornado in particular
> is the class's answer to somebody standing at mid range refusing to close: catch him with it
> and he is dragged further from you as it travels, not toward you, which is the opposite of
> what every other catch in this kit does and is the point of it.

### Flame spitter
**Startup** fast · **Recovery** medium · **Range** medium, channelled · **Mechanic** on a
structure, melts it into a lasting magma field that damages and slows

Channelled flame toward the cursor. Damage increases further from the caster, so the
spacing is inverted from most channels — you want them at the tip.

### Ice blast
**Startup** fast · **Recovery** short · **Range** short cone · **Mechanic** launches
structures it hits as projectiles

A quick cone. Structures caught in it are knocked forward, dealing extra damage and
staggering whatever they hit. This is the class's answer to a blocking opponent — the
structure is the guard breaker.

## In the air — built 2026-09-14

**Implemented.** Three moves, and they are the same three buttons she already uses. The row
is where her feet are:

```text
                left click      right click     E                middle click    F            R / side B
  standing      Bolt            Cataclysm       Raise (held:     Cinder spray    Updraft      Tremor / Quake
                                                Fissure)
  in the air    Air bolt        Gale            Landfall         Cinder spray    Downdraft    -- / Quake
                (Q: Fire pillar, held: Strike -- the same pillar, standing or not)
```

The three new columns are v2, built 2026-09-30, and are read the same way: the button is the
element and the row is where her feet are. Middle click is the one that does not change with
the row, on purpose — fire in the air is what the air shots fly through.

That shape is the Champion's grid read one class further, and it is deliberately the
[README's](../README.md) open **Aerials** question answered rather than dodged: *airborne
attacks are variants of their grounded counterparts, not a separate move list.* Left click is
still the cheap thing you throw constantly, right click is still the committed one, `E` is
still earth. A player who has learnt her standing up has learnt most of her in the air.

**Shift does not reach up there — and does not reach anywhere now.** Shift plus a click
stopped being an attack input on 2026-09-16, on every class, so Fissure has no button at all
until it is given one; see [../controls.md](../controls.md#shift-is-one-verb-now-2026-09-16).
The paragraph below is why it was never going to be the air row's answer either.

Shift plus left click was Fissure, a crack that races along
the *ground*; there is no airborne version of it to reach for, so the modifier is ignored and
left click means what left click means. Ignoring it has to come out as the Air bolt rather
than as silence, or the input is simply eaten.

### Why air

Earth is the thing she is standing on, and off the floor she is not standing on it. So the two
things she throws with her hands up there are **air** — the element listed above as a later
specialisation axis, borrowed for the one situation where the element she has cannot reach.
The way *back* to earth is to go and hit it, which is Landfall, and it is the only one of the
three that leaves a structure behind.

**Both shots travel**, which nothing she throws standing up does. Bolt and Cataclysm are
instant lines resolved on the frame they come out; these have a speed. The reason is the
situation rather than the element: she is falling while she throws them, and an instant hit
taken from a position she cannot hold would be free. A flight time is what makes being in the
air a trade. See `crate::gust`.

### Air bolt — left click, airborne
**Startup** fast · **Recovery** short · **Range** long, skillshot

A small, fast bolt of air thrown along the crosshair. Low damage and a little stagger — the
poke, and the same trade the grounded auto makes with a body added to it.

> **What it buys is reach.** It is the longest thing in the kit by some way, longer than
> Cataclysm and twice the beam, and that is the whole of what leaving the floor pays for: she
> is committed to an arc, she cannot walk out of what she started, and what she gets for it is
> the ability to touch somebody who thought they were out of the fight. It is slower than a
> fire bolt on purpose — a poke you can see coming is a poke that can be answered, and a
> long-range one that could not be would simply be the correct button.
>
> It carries the aerial shove the other pokes in the game get, so throwing it is *part of*
> moving in the air rather than a pause in it.

### Gale — right click, airborne
**Startup** slow · **Recovery** medium · **Range** long, skillshot

A **frisbee** of air, thrown flat, that opens as it goes and hits harder the wider it has got.

> **It is thrown, not pushed.** The disc's own axis — the one a frisbee spins about — is square
> to the line of effect that aimed it and lies in the *vertical plane containing that line*. So
> a Gale thrown level lies flat, and one thrown down at the floor is tipped nose-down by exactly
> the angle it was thrown at: it slices along its own path, edge leading. It is a thing going
> past you, not a wall of air coming at you.
>
> The hit test agrees and always did. What the shot occupies is a thin disc of its current
> radius riding its line, with its width lying *across* the throw — the victim's standing
> cylinder is swollen by the girth in radius and never in height — so the same clearance that
> saves you by stepping aside does not save you by standing above it, and vice versa. Tipping
> the disc into the plane its path already lies in costs that nothing: the width runs along
> `dir × axis`, which is horizontal whichever way the throw is pitched.
>
> Getting there took two wrong drawings. It was face-on to its own travel first — a picture of
> the one volume the game does not have, and it read from the seat as a disc turned to face
> her. Then it was a flattened *sphere*, which in a barely-opaque material has no flat face and
> no rim and so read as a glowing orb whatever it was scaled to. It is a disc mesh now, with an
> edge you can see it turn on. [../../../CLAUDE.md](../../../CLAUDE.md)'s overlay rule is what
> all of this is in service of, in a place that is not technically an overlay.
>
> **The size is the move.** It leaves her hand at a fraction of its listed radius and comes up
> to full size over a distance of its own, and damage and knockback ride that same fraction —
> so a disc caught at point blank is a puff of air and one caught out at size is the heaviest
> shove in the kit. Every other projectile in the game is worth the same wherever it lands;
> this one is worth what it has *become*.
>
> **How far it goes and how fast it opens are two knobs**, not one. They were one number for a
> day — the growth measured against the reach — and the trouble showed up the first time the
> range was bumped: doubling how far it flew silently halved how big it was everywhere a
> fighter actually stands. `Gale full size after (m)` owns the opening; the move row's `reach`
> owns the flight. It is full size well before it expires, and stays that way, which is the
> shape a thrown disc has anyway: it opens, and then it is open.
>
> That inverts the spacing, and it is the same sentence **Flame spitter** below is already
> written around — *"you want them at the tip"* — said as a thing that flies. The answer to it
> is to close, which is the answer this class least wants you to have and most deserves to be
> given.
>
> **The stun does not scale.** How long a hit holds somebody is frame data, and frame data
> that changed with distance would be a move nobody could learn. Only what it is worth moves.
>
> It is still the slowest thing she throws, and the longest-lived: a disc that has to be walked
> away from is only a decision if there is time to make one.

### Landfall — `E`, airborne
**Startup** slow, telegraphed · **Recovery** medium · **Range** melee, around where she lands
· **Mechanic** drives a structure up in front of her

A descending slam. She hangs, then comes down hard; the arrival staggers a patch of floor for
low damage and levers a slab of rock out of the ground in front of her at forty-five degrees,
throwing whatever was standing there up and away.

> **The wind-up ends on the floor, not on a number.** Landfall is the only move in the game
> whose startup does — the frames in its row are what she is guaranteed to owe (a hang at the
> top, then the drop), and the descent is over when her feet arrive, which from four metres up
> takes longer than from one. So the telegraph is exactly as long as the height she chose to
> open up. Going higher is buying reward with time the opponent gets to use.
>
> **And they can use it.** Everything about the descent is an ordinary startup, so a hit lands
> on her the way it lands on anybody mid-wind-up: she is knocked out of it, and the slab she
> was about to drive up never appears. That is the whole of it, and it needed no rule of its
> own. Contesting the space she is coming down into is the counterplay, and the length of the
> plunge is what makes it findable rather than a read.
>
> **The slab is the payoff, and it is not raised — it is driven.** Two differences from an
> ordinary structure, and they are the same difference twice. It comes out of the floor in
> well under half the time, because the telegraph was the plunge rather than the rise. And it
> comes out **leaning**, forty-five degrees above the floor pointing away from her, so whoever
> is standing over it is thrown up and back along the lean instead of merely staggered where
> they stand. An ordinary eruption does damage and a stagger and leaves you where you were;
> this one clears the space she has just landed in.
>
> It is a structure like any other and it spends the **cap of three**, collapsing the oldest
> if she is already carrying three. A move that raised a fourth for free would be a way around
> the only cost the mechanic has.
>
> **It comes up in front of her, not at the crosshair** — the one placement in the class the
> mouse does not decide, because she is landing rather than aiming. That is `aim::planted_ahead`,
> which lives with the rest of the aiming model for the reason everything else there does; see
> [../aiming.md](../aiming.md).

### What it is for

The slam and the slab are one option, not two: the stagger holds somebody still exactly where
a slab is coming up beside them, and the lean throws them out of the space rather than into
it. Against somebody who has closed on her — which is the position this class least wants to
be in — that is a reset she can take from above rather than a trade she has to win on the
floor. The two shots are the other half: the Air bolt is how she pokes at range she cannot
reach standing up, and the Gale is what she throws at somebody who has to come *through* it.

## v2 — space into damage — built 2026-09-30

**What it is.** The payoff for authoring the field well: she converts distance into damage.
[../elementalist-v2.md](../elementalist-v2.md) is the decision and the reasons; this is what is
built. Every number below is one `cargo run -p sim --bin elemental` prints, and every one of
them is a knob.

### The four inputs

Both mouse side buttons, `F` and `R` (`I` and `O` stand in for the side buttons on a keyboard
without them). They are pressed buttons, not modifiers: each does one thing on its own and the
edge of the press is what counts, so a held side button throws one Quake and not a stream.
The design is [../exploration/0001_control_budget.md](../exploration/0001_control_budget.md).

### The charges — `Q` held and `E` held

**`Q` held is the Strike.** A tap is the pillar as built. Held past the startup, the pillar
waits in her hands with the aim live, at the crawl (a fifth of walking speed, the same as any
committed move), for up to a second. Letting go plants the pillar with its remaining burn *taken
off it and paid as one hit*: a full hold is worth the pillar's whole 390 of burn on top of its
175, and leaves nothing standing; a half hold is worth half, and leaves half a pillar. Getting hit
during the hold ends it with nothing placed. What a hold costs is ground: an opponent walking at
full speed closes about seven metres in the second she stands there, so a full Strike is only
safe from nine metres, and the frame table prints that beside the row.

**`E` held is Fissure.** Above, under its own heading. The same crawl, the same second, and the
same price in ground; what it buys is a crack from two to ten metres with a stone at its end
and rough ground along it.

### Cinder spray — middle click, both rows

A skillshot: an ember out of her hand at twenty-two metres a second that bursts at the first
thing it meets or at twelve metres, into a **cloud of embers** three metres across that hangs
for two and a half seconds and burns everyone but her on a tick. On the floor the cloud stands
on it; in the air it hangs where it burst. It is the same move in both rows because its job is
the same in both: **fire for the air shots to fly through.**

### Fire in the air

An Air bolt or a Gale whose line crosses fire — a pillar, a burning patch, a cloud — comes out
**lit**: half again the damage, and a burst into a small cloud where it lands. The Gale also
**shoves each stone it passes** once, by the beam's kick rule at four fifths of its speed, and
keeps flying. Both shots got a knockback pass at the same time; the air row has teeth now.

### Updraft and Downdraft — `F`

**Standing, `F` is the Updraft:** a column of air two metres across and three and a half tall
on her own body, for four tenths of a second. Everyone in it — her included — is lifted once by
their own weight, so a Bulwark rises less than a Dual mage (1.9 m against 2.8; she rises 2.6),
and a stone in it is lofted. The column is on her because a column at range cannot be aimed at
somebody who can walk out of it; on her it is a jump she shares, and the shared floor is what
she then does something with.

**Airborne, `F` is the Downdraft:** the same column, following her down. It drives her at the
floor, spikes any airborne body in it, presses a lofted stone into the ground and breaks a
resting one. When she lands while it blows the air **breaks outward** as a ring three metres
across that shoves and staggers; when she lands *in fire* — her own pillar, a patch, a cloud —
the fire goes out and a **ring of fire** races out from her feet at eighteen metres a second to
six metres, hitting once for sixty and shoving. Landing after the column has died bursts
nothing: the timing is the move.

### Quake and Tremor — second side button, `R`

Above, under Quake. One effect, two placements: five metres across, forty frames of shake in
which anything moving faster than a metre a second is staggered, then an eruption for ninety on
everyone still inside and a stone at the centre. Tremor's stone comes up under her and carries
her three metres. `R` in the air is nothing yet; Hover is the candidate and waits on a word.

### Fire on a stone

A stone is lit by a pillar whose footprint takes in its base (since 2026-10-02 a pillar aimed
at a stone goes through it to the floor beyond, so light one by aiming at the floor beside it), by an ember bursting beside it, or by a lit shot passing
it, and burns for five seconds. A **lit stone burns whoever stands on it** — her terrain denied
to the opponent as cover, which is the one ruling in [../elementalist.md](../elementalist.md)
answered — and anything that shoves or breaks it while lit (the beam's kick, a Gale's shove,
Cataclysm) **bursts it** into burning debris and a cloud of embers where it stood. An unlit stone
kicked is still only kicked.

### The dodge into a stone

A dodge thrown *toward* one of her own stones, with the crosshair on it, within a dodge's reach:
the stone breaks down as she passes, its slot is freed, and its footprint is rough ground along
her line — burning ground and a cloud of embers if it was lit. In the air it is the airdodge and
costs it. The dodge has to be toward the stone, because the camera behind her shoulder can sit
inside a stone at her back with the crosshair reading as on it, and a dodge away from a stone is
a dodge. It is the one thing in the kit that beats a string rather than a hit.

### Not built, waiting on a word

Blast on the first side button, Hover on `R` in the air, and the fire Trail on a dodge through
fire — all carried in [../elementalist-v2.md](../elementalist-v2.md) §"Inputs" and §"Open questions".

## Playing it

Raise or Fissure to seed the field, then read the opponent's position and detonate the
structure that catches them. Structures are simultaneously your damage, your cover, and
their cover — the skill is placing them where they serve you more than the opponent.

## Open questions

- **The air row wants playing before any of its numbers are believed.** Three in particular.
  Whether the Gale's near end should be as weak as it is, or whether a disc that is nearly
  worthless at her own feet reads as a bug rather than as spacing. Whether the plunge's length
  *scaling with height* is the right trade or a free reward for pressing it low down — from a
  short hop the telegraph is barely longer than the hang. And whether a fourth structure's
  worth of terrain arriving every time she leaves the floor is more than the cap of three can
  absorb.
- **Landfall's aerials share the grounded clips.** The Air bolt plays Bolt's flick, the Gale
  plays Cataclysm's two-handed throw and Landfall plays Fissure's *hands driven into the
  ground* — each of which is recognisably the right shape, and none of which was authored for
  the air. Clips are a contract (`view::clips`) and the bake refuses to run with one missing,
  so four new ones is a real piece of work rather than a line; it is the obvious next step and
  is logged in [../feel-log.md](../feel-log.md).
- **Answered, 2026-09-30: the air shots interact with fire, and the Gale moves a stone.** A
  bolt or a Gale flown through a pillar, a burning patch or a cloud of embers comes out lit —
  half again the damage, and a small cloud where it lands — and the Gale shoves each stone it
  passes once, by the beam's kick rule, and keeps flying. Both are in §"v2".
- **The v2 numbers want playing before any of them are believed.** Five in particular, from
  `elemental`. Whether a full Strike at 565 off a pillar's own 175 is *earned* at the nine
  metres of gap it costs, or whether the crawl makes the hold read as a trap she set for herself.
  Whether the crack's two-to-ten metres is the right span for a hold of a second. Whether the
  Updraft's two and a half metres is a jump she wanted or a jump she is stuck at the top of.
  Whether the fire ring at sixty is a punishment for chasing her into her own fire or free
  damage on a landing. And whether the break-through's scar, one stone's width, is enough to
  matter to a pursuer. The plan's stop condition still stands: if a full Strike does not feel
  earned, that is a new document, not a knob.
- **Raising a stone mid-air is gone, and nobody has missed it yet.** `E` off the floor used to
  be Raise, which made a stone under your own feet a sort of second jump. It is Landfall now.
  Whether that pseudo-double-jump was load-bearing for her mobility is a thing to find out by
  playing, not at a desk — and the **Double jump** row in [../README.md](../README.md) is
  where the general version of the question lives.
- **Settled for the auto, open for everything else.** Yes — the beam is blocked by a
  structure in its way, and that self-obstruction is a real cost worth keeping. Fissure,
  Quake and Ice blast are unbuilt skillshots and have not been given the same answer;
  Ice blast in particular *wants* to reach structures rather than be stopped by the
  nearest one, so "blocks" cannot simply mean the same thing for every ability that
  travels.
- **The beam ignores the arena, and so does everything else she throws.** Walls and
  platforms are not traced against, so a shot aimed down at the floor passes through it and
  whiffs rather than stopping short of one. The same is true of Cataclysm's debris, of the
  fire bolt and of both air shots: the only solid any of them knows about is a stone.
  Deliberate for now — stones are the one piece of terrain the shots are *for* — but it is
  the obvious thing to revisit once the arena is more than a blockout, and it has already
  cost one confusing test failure. A stone raised inside the dais's footprint is pushed up
  **on to** it and stands a metre and a half in the air; debris then flies under the stone
  *and through the platform holding it up*, which looks from the outside exactly like debris
  punching through a stone. See `CLEAR_LANE` in `crates/sim/tests/cataclysm.rs`.
- Should the first-thing-it-meets check on the auto also read *black spike*, or anything
  else a future element adds to `effects.rs`? Right now it only recognises fire pillars,
  because fire is the only element that currently ships with the class. The dispatch is
  written so a second kind is one more branch, not a rewrite — nothing has needed the
  second branch yet.
- A fire bolt is stopped by a structure and expires at its range. Whether it should
  instead *kick* one the way the beam does has not been played against: a bolt of fire is
  not a shove, but a stone taking a hit and not moving reads oddly.
- ~~Raise places a stone 2.5 m ahead, so "cast beneath yourself to launch into the air" has no
  input.~~ **Answered.** Raise is a *reach*: the stone comes up on the first thing the
  crosshair's line meets, out to 6 m, so looking down puts one under your own feet and the
  eruption carries you with it. What is open is whether that lift is worth anything — see the
  note on the mechanic above.
- Do structures block your own projectiles? Almost certainly yes, and that self-obstruction
  is a real cost worth keeping. They block *bodies* now, the Elementalist's included.
- A stone lifted off centre rides up on the shoulder of the one below rather than sliding off
  it. That is what the arena's own platforms do, and it may want revisiting once moves are
  throwing stones around in earnest.
- A stone stands a whole body height and abilities come out of the chest, so from the ground
  you only ever see a stone's *side*. Stacking one on another by aiming needs you above the
  cap — honest geometry, but it may want an answer.
- Structure durability and displacement force are the tuning knobs, per
  [../elementalist.md](../elementalist.md). Both need a prototype.
- Three may be the wrong cap. It is the number that keeps the arena readable in third
  person, which matters more than the combo ceiling.
