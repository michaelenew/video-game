---
status: proposed
decided: 2026-09-09
resolves: the block/parry open question in combat-kernel.md
---

# Defense — dodge, block, parry

The defensive layer. Unblocks the [Bulwark](bulwark.md) and the
[Shifter](shifter.md), both of which were waiting on it.

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

Already sketched in the source notes as `shift + jump + direction`, or double-tap
`jump + direction`. Keep it, with:

- A brief invulnerability window.
- A committed direction — you go where you pointed.
- Recovery frames on landing.

Dodge is the universal answer to a read you made correctly. It is punished by attacks
that cover space or arrive late.

The existing crouch-roll (`crouch → jump`) stays as a distinct, slower, lower-commitment
reposition.

## Block

Blocking requires a shield, which is already an **equipped secondary** in the weapon
system ("Shield — equip as secondary to give defense and gain stagger resistance while
casting"). Bind block to the held secondary-weapon action, which the control notes already
map to `ctrl+click`.

### Facing arc, not a bubble

**Block covers an arc in front of you, not 360°.** You turn slowly while blocking.

This is what makes blocking beatable by movement, which is the entire point of a 3D
arena. It also gives the Bulwark's "slow to turn while held" a real mechanical meaning
rather than a flavour note.

### The cost of blocking is space, not health

**No chip damage. Pushback instead.** Blocked hits shove you backward, scaled by the
attack's weight.

This matters more than it sounds:

- Chip damage in a 60-second match means blocking slowly kills you, which trains players
  never to block. The option dies.
- Pushback makes blocking cost **the resource the game is actually about**. You can block
  all day and still lose, because you have been shoved to the edge of the arena and have
  nowhere left to go. That is the Smash-like "space is the resource" pressure, expressed
  through the defensive system.

A **guard meter** sits underneath as the hard bound. It depletes on blocked hits and
regenerates out of combat. Emptying it is a guard break: a long, punishable stagger.

**The Bulwark resists pushback.** That is the class trait that falls naturally out of this
system rather than being bolted on — everyone else who blocks gets moved, and the wall
does not.

### Two different things called "the shield"

Worth stating explicitly, because the Bulwark depends on the distinction:

- **The shield as a world volume** stops projectiles by *collision*. No guard meter, no
  timing. This is why a Bulwark's planted or thrown shield keeps blocking projectiles
  while the Bulwark is somewhere else entirely.
- **The shield as a guard state** handles melee — guard meter, pushback, parry timing.
  This only exists while the character is actively blocking.

Same object, two systems. Elementalist structures use the first one, which is the shared
implementation noted in [elementalist.md](elementalist.md).

### Stagger resistance while casting

Carried forward from the weapons document unchanged. A shield equipped as secondary makes
you harder to interrupt mid-cast, whether or not you are actively blocking. This is what
makes shields attractive to casters and keeps the secondary slot a real decision.

## Parry

**The parry is the first few frames of raising the block.** There is no separate input.

- Time it right → **parry**.
- Time it wrong → you are simply blocking, taking pushback and guard damage.

One input, two outcomes, decided by timing. The risk of attempting a parry is that you are
now committed to blocking, which is a real cost in a game where blocking cedes space. No
extra punishment layer is needed.

### The reward is a stagger

**A successful parry staggers the attacker**, using the stagger system exactly as already
specified — the attacker enters the recovery mini-game (press to recover, window scaled by
stagger strength) while the parrying player is free to act.

This is the right reward because:

- It reuses the game's existing signature mechanic instead of inventing a new one.
- The punish window is legible to both players. The staggered player can see themselves
  recovering; the parrying player can see how long they have.
- It makes the [Bulwark](bulwark.md)'s design line — true staggers gated behind hard
  conditions like a successful parry — actually work.

The Shifter's existing "parrying gives a stacking increased resistance buff" survives as a
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
- **Shifter** — "raise shield, scroll back to brace" fits as a shield-form behaviour; the
  stacking resistance buff sits on top of the universal parry stagger.
- **Statera** — ascension explicitly removes dodge *and* block. This system is what makes
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
