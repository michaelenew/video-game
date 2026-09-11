---
status: partly unsettled
decided: 2026-09-10
revised: 2026-09-11
---

# Controls

Mouse and keyboard first. A controller scheme is more standard and easier and can follow.

The reference points are platform fighters, which get their feel by **removing a dimension**
so the controls stay small. This game keeps 3D movement, so the compensation is that
direction is *discretised* — four inputs, not a stick — and modifiers do the rest.

> **Revised 2026-09-11.** Sentence 4 below used to read *"space means you move more than you
> otherwise would — alone it jumps, with a direction it dodges"*, and there was a sixth rule,
> *"shift beats WASD when both are held"*. Neither survived the sandbox. Dodge has moved to
> shift and space is now only ever a jump; the knock-on effects are listed under
> [Open](#open-since-the-dodge-moved) and are **not settled**.

## The grammar

Five sentences, and everything else follows:

1. **Click means attack.**
2. **Shift means use an ability** when a click comes with it, and **dodge** when only a
   direction does.
3. **WASD means move.**
4. **Space means jump.** A vertical takeoff, every time, whatever your feet are doing.
5. **The mouse means *where*.** You look with it, you are pointed where you look, and
   your attacks go where you are pointed.

### Why space stopped being clever

Space used to mean "you move more than you otherwise would": alone it jumped, with a direction
it dodged. That is a tidy sentence and it is wrong in the hand. You are holding a direction
almost all the time, so the jump button mostly did not jump — you pressed it expecting to
leave the ground and dodged sideways instead. A button whose meaning depends on whether you
happen to be walking is a button you cannot trust.

Shift was already the "stronger version of this" modifier, so dodge went there.

### What disambiguates shift

**A click.** Shift with a click is the stronger version of that attack. Shift with only a
direction is a dodge. The click is checked first, so a committed move thrown while walking
never comes out as a dodge.

### In the air

- **Airdodge**: shift plus a direction, **once per airtime**. It wipes vertical speed rather
  than adding to it, so it is a sideways commitment and never a second jump. A second one
  would turn a jump into flight.
- Space while airborne does nothing yet.
- Airborne attacks are currently the grounded ones. That is a placeholder, not a decision —
  see below.

## Open since the dodge moved

Moving dodge onto shift retired **"shift beats WASD when both are held"**, which was the rule
that guaranteed a move-while-casting option always existed. These are consequences, and none
of them is settled:

- **Move + heavy attack is currently impossible.** This is the sharp edge. Shift + direction is
  a dodge and shift + click is the heavy version of an attack, so *holding a direction and
  throwing a heavy* has no input — the dodge takes it. Known and accepted for now; it is the
  first thing the attack grammar has to solve once movement is settled.
- **Differentiating move + attack.** Directional attacks (`w`/`a`/`d`/`s` + click) still work,
  but the modifier space is tighter than it was and the option table below was written under
  the old rule.
- **Aerials as variants.** The intended direction is that an airborne attack is a *variant of
  its grounded counterpart* — the same move with different frame data — rather than a separate
  move list. Nothing is implemented.
- **Neutral shift.** Shift with no direction and no click does nothing. A spot dodge in place
  is the obvious candidate.
- **Double jump.** Space while airborne does nothing. The airdodge is the only air commitment
  at present, which may be too few or exactly right.

## Everything is relative to the camera

`W` is away from the camera, not along some world axis. `D` is to the camera's right.
Aim is the camera direction. The player turns by turning the camera; there is no separate
turn control and no auto-facing.

Two consequences are worth stating because they are design, not implementation:

- **Facing locks the instant a move starts.** During startup, active and recovery frames
  the mouse moves the camera but not the fighter. Otherwise you could drag a live hitbox
  around during its active frames and rescue a whiff by turning after the fact, and whiff
  punishment is most of the game. You commit to a direction when you commit to the move.
- **Guard turns slowly.** Guard covers an arc rather than a bubble ([defense.md](defense.md)),
  and an arc you can flip instantly *is* a bubble. The camera still goes wherever the mouse
  goes; it is the character who cannot reorient that fast.

### Hindrance is proportional to commitment

A move takes your feet away in proportion to how much it commits you.

| | Speed |
| --- | --- |
| Free | 7.0 |
| Throwing a poke | 4.2 |
| Crouching | 3.0 |
| Guarding | 2.0 |
| Committed move | 0 — rooted |

Rooting is what commitment *means*, and it is right for the heavy moves: spacing only matters
if choosing to swing costs you the ability to reposition. It is wrong for a fast poke. The
poke is the neutral tool, thrown constantly, and stopping dead for every one makes neutral
sticky and reads as the game taking the controls away. Slowing you keeps the cost — you
cannot close or escape at full speed while swinging — without the lurch.

Even where rooting is correct, arriving at rooted takes about four frames rather than one.
The snap from a full walk to nothing was the jarring part, not the rooting. The distance slid
while bleeding off is about twenty centimetres: nothing for spacing, everything for how it
reads.

`cargo run -p sim --bin frametable` prints these alongside the frame data, and marks which
moves root you.

### The crosshair tells you where the attack goes, not where the camera points

Those two are the same most of the time, and deliberately not the same during a committed
move or a lagging guard. The reticle is placed by projecting the point the fighter is
actually pointed at, so it sits still in the middle of the screen while facing tracks aim
and slides off to the side when it does not. It dims while you are committed to something
and the button will not answer.

A reticle that says "here" when the answer is "not there" is worse than no reticle, and the
moments it would lie are exactly the moments the answer matters.

### Settings

Three numbers are adjustable mid-match. They show above the control legend and are written
to `~/.config/arena/settings.conf` immediately. `ARENA_SETTINGS` overrides the path, which is
how two people on one machine keep separate settings without a profile system.

| Keys | Setting | Step |
| --- | --- | --- |
| `-` / `=` | mouse sensitivity | multiplicative |
| `F3` / `F4` | vertical field of view, degrees | 2° |
| `F5` / `F6` | camera distance, metres | 0.4 m |

Sensitivity steps by a **ratio** and the other two by a fixed amount, because that is how
each is perceived: a given ratio of sensitivity feels like the same change at any value,
whereas two degrees of view is two degrees of view whether you are at 45 or at 90.

Field of view and camera distance are settings rather than constants for a plain reason —
they are the first numbers anyone reaches for when a camera feels wrong, and a value you have
to rebuild to try is a value that gets tried once.

Aim is quantised to 1/65536 of a turn and sent over the wire alongside the buttons, because
where you look decides where you move and what you hit — which makes it gameplay, and
gameplay has to match on both machines exactly. See [architecture.md](architecture.md).

**Shift beats WASD when both are held.** So holding a direction while pressing shift+click
still gives you the shift ability, and you keep moving during it (where the ability allows).
This guarantees a move-while-casting option always exists, and it costs nothing, since
directional abilities and shift abilities were never going to be used simultaneously anyway.

## The option space

| Modifier | × | Button |
| --- | --- | --- |
| none · `w` · `a`/`d` · `s` · `shift` | | `L` · `R` · `M` (scroll click) · `LR` (both) |

Five by four is **20 distinct offensive inputs at any moment**, before airborne variants.
`a` and `d` mirror each other, which is why they count once.

> **This table predates the dodge move** and assumed "shift beats WASD". The counting still
> holds — shift plus a click is still an ability — but the reasoning behind it should be
> re-derived rather than trusted. See [Open](#open-since-the-dodge-moved).

That is a large space. It is not a target.

### Do not fill all twenty

Ship each class with autos, the six-ability kit, and a handful of directional basics. Leave
slots empty. Smash has roughly eighteen moves per character and they are only legible
because they group into families — tilts, smashes, aerials, specials. Twenty times two for
airborne, filled in at prototype stage, produces a class nobody can read.

Empty slots are headroom for the later specialisation layer, not a gap.

## What each region means

This is the part that keeps twenty inputs learnable — each region has one job, and it is the
same job on every class.

| Region | Meaning |
| --- | --- |
| **Unmodified `L`/`R`/`M`** | Autos **and the class mechanic.** This is where identity lives, and it is different on every class. |
| **Direction + click** | Basic moves. A shared vocabulary — roughly the same shapes on every class. The two casters are the exception. |
| **Shift + click** | The six-ability kit. |
| **Shift + direction** | Dodge — **or the class's own mobility mechanic, where it has one.** Airborne, the once-per-jump airdodge. |

That last row does real work. The Bellator's Rush and the Reaver's Shadow dash *are* their
dodges rather than extra inputs. For the Reaver this is what makes movement and shadow
placement the same action, which is the fix that keeps the class from being denied its
mobility.

## Movement

Movement is the core of how the game feels, so it gets pinned down before the attack and
ability grammar is settled around it.

### The jump is variable and floaty

Space is a vertical takeoff. **Hold it to go higher** — while the button is down and you are
still rising, gravity is reduced, up to a cap. Releasing ends the sustain *for good*: a second
press cannot resurrect a jump you already cut short, or the height stops being something you
chose.

Sustain rather than a cut-on-release. Both produce variable height; a cut makes the short hop
feel like the jump was taken away from you, a sustain makes the tall one feel earned.

Floaty on purpose, and **high** on purpose. Verticality is part of the positioning game, and
Smash characters routinely jump four or more times their own height. A full hop here reaches
**2.3 to 4.8 body heights** depending on class.

There is deliberately **no ceiling on airtime**. What a jump has to be is four separate things,
and each is worth stating on its own rather than collapsing into one frame count:

- **High enough to clear another fighter**, on the *short* hop, or verticality belongs only to
  the committed option.
- **Fast enough not to be a sitting duck** — every class gets above a standing opponent's head
  within 12 frames. Vulnerability while jumping is about the time spent at head height where
  you can be hit, not about total airtime.
- **Steerable enough to dodge** — strafing across a jump carries you at least two body widths
  from where you took off, so a hitbox aimed where you started can be left behind.
- **Slow enough to be punished** — airtime exceeds reaction time plus the fastest poke, so a
  jump can be seen, answered and hit.

`cargo run -p sim --bin frametable` prints the height and airtime for each class.

### Classes differ in the air first

| | Jump | Gravity | Fall cap | Steering |
| --- | --- | --- | --- | --- |
| Bulwark | ×0.9 | ×1.2 | ×1.1 | 0.9 |
| Bellator | ×1.0 | ×1.0 | ×1.0 | 1.2 |
| Shadow Reaver | ×1.1 | ×0.9 | ×1.0 | 1.7 |
| Elementalist | ×1.1 | ×0.9 | ×0.9 | 1.0 |
| Blood mage | ×1.0 | ×1.0 | ×1.0 | 1.3 |
| Dual mage | ×1.1 | ×0.8 | ×0.9 | 1.4 |

Weight is the most legible difference a character can have. You can read it across the arena
in the first second of a match, before you know a single one of their moves — so it carries
identity for free, and every class sharing one jump arc would waste the channel. The Bulwark
gets 45 frames of airtime and the Dual mage 74.

`cargo run -p sim --bin frametable` prints these.

### Air control is Quake's, not a second walk

In the air, input **accelerates** rather than assigns, and the acceleration is granted against
*the component of your motion you have not already spent*:

```text
head_room = air_speed − (velocity · wish_direction)
```

Point where you are already going and that projection is large, so there is nothing left to
add — **holding forward in the air does essentially nothing**. Point across your motion and
the projection is near zero, so you get the full budget, which **turns** your velocity without
spending it.

That is the whole reason air movement has a skill ceiling. A single strafe spends a fixed
budget and stops; turning the camera while holding it keeps redefining which direction counts
as perpendicular, so the budget refills against the new heading. A player who does not turn
gets one nudge. A player who does can carve.

**One deliberate divergence from Source:** horizontal air speed is capped at 1.5× the walk. In
Source the gain is unbounded and that unboundedness became the genre; in a fighter built on
spacing, a player who can reach any part of the arena from any other has removed spacing from
the game. The cap is set high enough that good strafing is still rewarded, and whether it is
set right is open.

Momentum carries. Letting go of the stick mid-jump does not stop you — the difference between
a jump being a commitment and a jump being a hover.

### Aerials hang

An attack thrown in the air holds gravity off for a few frames, and **slows** whatever vertical
speed you had rather than deleting it. It is a **per-move property**,
because the hang *is* a move's air identity: a rising strike that holds you up for a beat plays
completely differently from one that drops you through it, and both are worth having. Gravity
is skipped outright rather than reduced, so the hang is a flat number of frames a player can
learn rather than a curve they have to feel.

Deleting the rise was the first attempt and it read as the game snatching the jump out from
under you. The extra control over jump height that attacking gives is worth keeping — it has to
arrive as a slowing rather than a stop, which is also what makes the hang read as float instead
of a pause. Gravity staying off through the window is what keeps it punchy: you hang, you do
not sag.

**A poke also shoves you the way you are holding.** The basic attack is the one you throw
constantly, so this is what makes attacking *part of* air movement rather than a pause in it:
the hang supplies the float, the shove supplies the punch. It needs a direction held, it is
clamped by the air speed cap like any other air movement, and the committed moves get none —
they are already a commitment, and one that also repositioned you would be strictly better than
a poke.

Every aerial hangs a little by default — an attack that drops you straight through gives the
air nothing to offer. Moves that want more say so; the Bulwark's Slam hangs twice as long as a
poke.

### Airborne attacks — ⚠️ open

Grounded and airborne should differ, as they do in every platform fighter. The intended shape,
**not yet settled and not yet implemented**: an aerial is a *variant of its grounded
counterpart* — the same move and the same identity, with different frame data — rather than a
second move list per class. That keeps the vocabulary a player has learned on the ground worth
something in the air, and keeps six classes from turning into twelve movesets.

What exists today: **space is a vertical takeoff**, and **shift plus a direction is an
airdodge, once per airtime**, which wipes vertical speed so it can never be a second jump.
Airborne attacks are still the grounded ones.

---

## Dual mage

The mechanic is on the primary buttons, and it is not optional.

**`L` always moves you darker. `R` always moves you lighter.** Every input, not just autos.

| Input | Result |
| --- | --- |
| `L` / `R` | Dark / light auto. **Changes your mode on contact** — a whiff steers nothing |
| direction + `L`/`R` | Basic moves, in dark or light form |
| `shift` + `L`/`R` | Abilities, in dark or light form |
| `M` / `LR` | Gated finishers — Judgement and Eclipse |
| `shift` + `M` / `shift` + `LR` | Ordinary abilities. Push further along your current path |

`M` and `LR` are neither left nor right, so they cannot pick a direction. They push you
further down whichever path you are already on. The grammar stays consistent: **direction
comes from side-ness, and only left and right have it.**

**Autos have a slight range boost** — the beings inside extend your reach. This matters
mechanically, not just as flavour: steering requires landing hits, so the class needs the
reach to steer under pressure.

**Steering is not optional.** You cannot cast without moving the bar, and you cannot move the
bar without committing to a side. This replaces the earlier tap-versus-hold proposal, which
put the direction choice in a modifier the player could ignore.

> **One reservation, for the prototype to settle.** `M` and `LR` are the slowest and least
> reliable inputs on most mice, and they are carrying the finishers — the class's payoff.
> The argument for keeping them there is Smash's: a heavy, deliberate input suits a heavy,
> deliberate move, and the finishers are depth-gated so the input is dead most of the time.
> The argument against is that a dropped payoff in a 60-second match feels terrible. If it
> proves bad, swap the finishers onto `shift`+`L`/`R` and move the ordinary abilities out.

## Bellator

**`L` / `M` / `R` are sword / hammer / spear.** Pressing a form you are not currently in
triggers the switch, animated from whatever the current context is.

**This means the mid-animation swap needs no new input.** Press a different form's button
during active frames and you get the cross-form ending. The mechanic and the control are the
same thing, which is the strongest argument that the mechanic is right.

| Input | Result |
| --- | --- |
| `L` / `M` / `R` | Auto in that form, or switch to it if you are in another |
| `L`/`M`/`R` during active frames | The swap. Changes the move's tail |
| `w` + any click | Charge attack |
| `a`/`d` + any click | Lateral. Directional knockback — the combo backbone |
| `s` + any click | Low / grounded attack |
| `shift` + click | Abilities. Shift versus no-shift is **bigger versus smaller, and different in kind** |
| `space` + direction | **Rush.** The class's dodge is its chargeable, bankable dash, and it still cancels recovery |

Direction plus click carries most of the class's feel. These want varied, semi-directional
knockback so that where you hit from determines where they go — that is what makes the
combo game read.

## Bulwark

| Input | Result |
| --- | --- |
| `L` | Off-hand melee auto |
| `R` (hold) | **Guard.** Opening frames are the parry |
| `M` | **Throw** when held, **Recall** when planted. Reactivate mid-flight to leap to it |
| `shift` + `L` | Bash |
| `shift` + `R` | Slam |
| `shift` + `M` | Grapple |
| direction + `L` | Basic moves |
| `LR` | Reserved — the candidate slot for a dedicated ally-cover stance |

Shield position lives on `M`, so the whole three-state mechanic is one button with context.
Guard on the right button matches every game where alt-fire is the defensive option.

## Shadow Reaver

A half-caster. Click abilities should feel like real melee — strong individually rather than
combo-dependent — and the shadow abilities should reward being close and fast.

| Input | Result |
| --- | --- |
| `L` / `R` | Melee autos, alternating hands. First auto after reclaiming the shadow hits harder |
| direction + click | **Shadow control** — send it, swap to it, recall it |
| `shift` + click | Executioner, Guillotine lotus, Deadly mistake |
| `space` + direction | **Shadow dash.** The dodge places the shadow |

Putting shadow control on the directional basics is what makes the class read as a
half-caster: the shadow is steered with the same inputs that other classes use for basic
attacks.

## Elementalist and Blood mage

The two full casters. **Every open slot is a unique spell, including the directional
basics** — these classes have no generic basic moves at all, which is itself the
differentiation.

### Elementalist

| Input | Result |
| --- | --- |
| `L` | Ranged bolt auto |
| `R` | **Raise.** Spawn a structure — the mechanic on a primary button |
| direction + click | Fissure, Quake, Ice blast |
| `shift` + click | Fire pillar, Flame spitter, and the heavier elemental work |

### Blood mage

| Input | Result |
| --- | --- |
| `L` | Melee auto with lifesteal |
| `R` | **Rend.** Press again to reactivate — the signature second decision |
| direction + click | Cripple, Black spike, Affliction |
| `shift` + click | Reaper's debt, Seal of the unforgiven |

## Open questions

- **`q` and `e`.** Currently unused. Twenty inputs is already more than a prototype needs, so
  they stay free. If `M` and `LR` prove unreliable, `q` and `e` are the natural replacements
  — they are fast, adjacent to WASD, and cost no finger travel.
- **Does `s` + click mean "low attack" or "defensive option"?** It should mean one thing
  across all classes. Low attack is the platform-fighter convention.
- **Camera-relative or character-relative direction?** Determines whether `a`/`d` really do
  mirror. Camera-relative is standard in third person and probably correct.
- **Held versus tapped clicks.** Several mechanics already want hold (Guard, charge attacks,
  Rush). Whether hold is a universal modifier or per-ability is unresolved.
