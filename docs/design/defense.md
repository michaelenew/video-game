---
status: proposed; audited against the build 2026-09-15
decided: 2026-09-09
audited: 2026-09-15
resolves: the block/parry open question in combat-kernel.md
---

# Defense — dodge, block, parry

The defensive layer. Unblocks the [Bulwark](bulwark.md) and the
[Champion](champion.md), both of which were waiting on it.

> **Two things in this document are contradicted by what was built, and neither is a
> drafting slip — both are design decisions somebody has to take deliberately.** They are
> marked ⚠️ below.
>
> 1. **The guard meter.** This document specifies one as the hard bound behind blocking.
>    [README.md](README.md) §1 says there is none, and the simulation has none.
> 2. **The shield as an equipped secondary.** This document, and
>    [gatekeeper-retirement.md](gatekeeper-retirement.md), treat the shield as a loadout
>    choice any class can take, and build the block layer on top of that. The roster, the
>    code and [parked.md](parked.md) make it the **Bulwark's class mechanic** and equipment
>    is parked, so five of six classes have no access to block, parry, or the pushback
>    resistance trait — and "dodge is universal, block is shield-gated" is a split with one
>    class on one side of it.
>
> Everything else here has been reconciled with the build.

## The central split

The temptation is to make one defensive verb that everyone has. That fails in both
directions: universal blocking erases the Bulwark's identity, and shield-only defense
leaves every other class with nothing but dodge timing, which makes offence-heavy
matchups swingy and dull.

**Split the verbs instead.**

| | Dodge | Block |
| --- | --- | --- |
| **Availability** | Universal | Requires an equipped shield |
| **Nature** | Evasive | Positional |
| **Beats attacks by** | Not being there | Stopping them |
| **You are** | Moving, briefly invulnerable | Static, directional |
| **Costs** | Directional commitment, recovery | Space (pushback), guard meter |
| **Beaten by** | Space coverage, delayed attacks, reads | Going around it, guard breaks, grabs |

These are genuinely different verbs with different counters. That is what keeps both
worth having, and it means a shield is a real loadout choice rather than a strict upgrade.

## Dodge

**`shift` plus a direction**, settled 2026-09-11. It was sketched here as
`shift + jump + direction` or a double-tapped `jump + direction`; space became a plain
vertical takeoff and dodge moved onto shift alone. See
[controls.md](controls.md#why-space-stopped-being-clever).

- A brief invulnerability window — 10 frames of a 22-frame dodge.
- A committed direction — you go where you pointed.
- Recovery frames at the end: the back twelve are vulnerable, which is the punish window.
- **Airborne it is the airdodge**, once per airtime: 16 frames, and it wipes vertical speed
  rather than adding to it, so it can never be a second jump.

Dodge is the universal answer to a read you made correctly. It is punished by attacks
that cover space or arrive late.

The crouch-roll (`crouch → jump`) is **not built** and is not currently wanted: crouch is
its own input and lowers your hurtbox, and a second, slower reposition has nothing to do
that the dodge's own tail does not already do.

## Block

**Held right click**, and it is gated on the *mechanic* rather than on the button: `R`
guards while a shield is in hand and does nothing while it is not, which is why right click
is free to be an attack on the classes that have no shield. Bound as `ctrl+click` on the
secondary-weapon action in the original draft; the control grammar settled in 2026-09-11 and
the secondary-weapon action does not exist.

> ⚠️ **"Requires a shield" means "is the Bulwark" today.** This section was written against a
> weapon system where a shield is an **equipped secondary** any class could bring — *"Shield —
> equip as secondary to give defense and gain stagger resistance while casting"*. Equipment is
> [parked](parked.md), and the shield became the Bulwark's class mechanic instead, so the
> universal-dodge / gated-block split has exactly one class on the gated side.
>
> That is not obviously wrong — a defensive class whose defence is its own is a clean
> identity — but three things in this document were load-bearing on the other reading and
> are currently unreachable for five of six classes: **parry**, **pushback resistance as a
> trait**, and **stagger resistance while casting** as the reason a caster would bring a
> shield. [gatekeeper-retirement.md](gatekeeper-retirement.md) also closes on *"the secondary
> slot itself is not parked — the shield occupies it, and shield-or-nothing is already a real
> choice"*, which is only true under the equipment reading.
>
> **The decision is whether parry is universal.** If block stays Bulwark-only, then the
> parry — the game's one gated-behind-a-read stagger, and the thing
> [ability-spec.md](ability-spec.md) points at when it says hard stops need hard conditions —
> belongs to one class, and the other five have dodge timing and nothing else.

### Facing arc, not a bubble

**Block covers an arc in front of you, not 360°.** You turn slowly while blocking.

This is what makes blocking beatable by movement, which is the entire point of a 3D
arena. It also gives the Bulwark's "slow to turn while held" a real mechanical meaning
rather than a flavour note.

### The cost of blocking is space, not health

**No chip damage. Pushback instead.** Blocked hits shove you backward, scaled by the
attack's weight, and hold you in blockstun while they do.

This matters more than it sounds:

- Chip damage in a 60-second match means blocking slowly kills you, which trains players
  never to block. The option dies.
- Pushback makes blocking cost **the resource the game is actually about**. You can block
  all day and still lose, because you have been shoved to the edge of the arena and have
  nowhere left to go. That is the Smash-like "space is the resource" pressure, expressed
  through the defensive system.

> ⚠️ **A guard meter was specified here and was never built, and the README rules one out.**
> This document proposed one underneath as the hard bound — depleting on blocked hits,
> regenerating out of combat, emptying into a guard-break stagger.
> [README.md](README.md) §1 now reads *"knockback plus stunlock, no chip damage, **no guard
> meter**"*, and the simulation has no such state. What bounds blocking today is pushback and
> the guard breakers below, and nothing else.
>
> That is a coherent position — the meter was a second hard bound on an option this document
> argues should already cost space — but it was never written down as a decision, and the
> two documents have disagreed since. **Pick one.** If the meter comes back it is new
> simulation state in the snapshot and a new HUD element; if it stays out, this paragraph
> should say so rather than being deleted, because "why is there no guard meter" is a
> question that will be asked again.

**The Bulwark resists pushback.** That is the class trait that falls naturally out of this
system rather than being bolted on — everyone else who blocks gets moved, and the wall
does not. **Built 2026-09-23, as a function of weight** ([bulwark-v2.md](bulwark-v2.md)):
blocked knockback is multiplied by a line from one at an empty shield to
`Bulwark · Weight, pushback at the cap` at a full one, so the more it has taken the less it
moves. An empty shield is shoved like anybody. Against the Champion's sword the difference is
mostly invisible, because the swing's own step pushes both bodies — see the feel log.

### Two different things called "the shield"

Worth stating explicitly, because the Bulwark depends on the distinction:

- **The shield as a world volume** stops projectiles by *collision*. No timing, no state.
  This is why a Bulwark's planted or thrown shield would keep blocking projectiles while the
  Bulwark is somewhere else entirely. **Not built** — nothing in the simulation collides with
  a shield, in flight or planted. See [kits/bulwark.md](kits/bulwark.md).
- **The shield as a guard state** handles melee — pushback, blockstun, parry timing. This
  only exists while the character is actively blocking, and it is the half that is built.

Same object, two systems. Elementalist structures use the first one, which is the shared
implementation noted in [elementalist.md](elementalist.md) — and structures *do* stop bodies
and shots, so the shared half exists and the shield is the side that has not been wired into
it.

### Stagger resistance while casting

Carried forward from the weapons document unchanged, and **unbuilt**. A shield equipped as
secondary makes you harder to interrupt mid-cast, whether or not you are actively blocking.
This is what makes shields attractive to casters and keeps the secondary slot a real
decision — and it is squarely inside the equipment question flagged above: there is no
secondary slot, and the one class with a shield is not a caster.

## Parry

**The parry is the first few frames of raising the block.** There is no separate input.

- Time it right → **parry**.
- Time it wrong → you are simply blocking, taking pushback and guard damage.

One input, two outcomes, decided by timing. The risk of attempting a parry is that you are
now committed to blocking, which is a real cost in a game where blocking cedes space. No
extra punishment layer is needed.

### The reward is a stagger

**A successful parry staggers the attacker** while the parrying player is free to act. Built:
the window is 4 frames and the stagger is 34, which `feel.rs` holds long enough to land the
slowest punish in the game.

> **The recovery mini-game is not built.** This said the attacker *"enters the recovery
> mini-game — press to recover, window scaled by stagger strength"*. A stagger is a plain
> countdown; there is nothing to press and nothing that shortens it. Whether the mini-game is
> still wanted is open — it is the only place in the design where a player acts during a state
> that has taken their controls away, and the argument in the next bullet ("the staggered
> player can see themselves recovering") is doing work it cannot currently do.

This is the right reward because:

- It reuses the game's existing signature mechanic instead of inventing a new one.
- The punish window is legible to both players. The staggered player can see themselves
  recovering; the parrying player can see how long they have.
- It makes the [Bulwark](bulwark.md)'s design line — true staggers gated behind hard
  conditions like a successful parry — actually work.

The Champion's existing "parrying gives a stacking increased resistance buff" survives as a
**class-specific** addition on top of the universal stagger reward, not as the base
behaviour.

## Guard breaks

If blocking is strong, something must beat it, or defensive play stalls a 60-second match.

**Special attacks are the guard breakers.** The weapons document already establishes these:
"Special attacks — these occur by shift attacking; only certain weapons have these (some
weapons may have one naturally and some may obtain them through forging)."

Giving them the guard-break role means:

- No new universal verb is added.
- The weapon system gets a job in the defensive metagame.
- Whether to bring a guard breaker becomes a **loadout decision with real consequence** —
  and one that forging and reforging can change.

Grabs remain a separate, class-level anti-block tool for the classes that have them
(notably the Bulwark's grapple). Command grabs beat blocking; they lose hard to dodge.
That is the correct triangle.

## How this lands on each class

- **Bulwark** — the whole kit becomes specifiable. Facing-arc block, pushback resistance,
  parry-gated staggers, grapple as the anti-turtle read, and the planted-shield projectile
  volume all now have rules.
- **Champion** — "raise shield, scroll back to brace" fits as a shield-form behaviour; the
  stacking resistance buff sits on top of the universal parry stagger.
- **Dual mage** — ascension explicitly removes dodge *and* block. This system is what makes
  that loss concrete and severe.
- **Everyone else** — dodge is the baseline defensive verb; a shield secondary is an
  option that trades offence for the block/parry layer plus cast stagger resistance.

## Open questions

- **Frame numbers.** Parry window length, dodge invulnerability, guard-break stagger
  duration. All need playtesting, none can be reasoned out.
- **Guard meter regeneration.** Out of combat only, or a slow trickle in combat? Trickle
  is friendlier; out-of-combat-only makes sustained pressure more meaningful in a
  60-second match.
- **Input conflict.** `shift` is currently crouch, and dodge is `shift + jump + direction`.
  Block is proposed on the secondary-weapon action (`ctrl+click`). Worth a full pass over
  the control scheme once these are settled, since the old key map predates all of it.
- **Does dodge cost a resource?** Free dodge with recovery frames is the Smash model and
  is probably right, but if dodge proves too safe, a small shared resource is the lever.
- **Directional blocking in coop.** Facing-arc block against several monsters at once may
  be too weak. Possibly the answer is that it is, and that is the Bulwark's job.
