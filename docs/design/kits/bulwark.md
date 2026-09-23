---
status: proposed
decided: 2026-09-09
depends: ../bulwark.md, ../defense.md
---

# Bulwark — kit

> ⚠️ **A v2 of the mechanic is being built, 2026-09-23** — see [../bulwark-v2.md](../bulwark-v2.md)
> and [../plans/bulwark-v2.md](../plans/bulwark-v2.md). Every hit taken on the shield is stored as
> **weight**; Slam (on middle click) and Throw spend it, and a planted shield becomes a real
> structure sized by it. **Built so far (M1):** weight itself — loading, the drain, the
> pushback it resists, drawn on the shield — and the Bulwark's health, highest on the roster.
> See [Weight](#weight--built-m1) below. Nothing spends it yet, and Slam still has no button.

**Identity.** The wall. Wins by denying space and funnelling the opponent into where you are
already aimed. Committed, not slow.

Greenfield — no archive material. Read [../bulwark.md](../bulwark.md) for the class rationale
and [../defense.md](../defense.md) for the block system it is built on.

## Mechanic — the shield has its own position

Three states, and every ability reads differently in each:

| State | You | Shield |
| --- | --- | --- |
| **Held** | Protected in a facing arc, slow to turn | With you |
| **Planted** | Mobile, exposed, faster | Holding space where you left it |
| **Returning** | Can leap to it | In flight, damaging along its path |

**Traits:** reduced block stunlock, and a shield wide enough to cover an ally. Carrying it
costs movement speed — which you get back when it is planted.

**Two systems are meant to share the object.** As a *world volume* it stops projectiles by
collision, with no timing or state involved — which is why a planted shield would keep
blocking while you are elsewhere. As a *guard state* it handles melee, with stunlock and
parry timing. Only the second requires you to be holding it.

> ⚠️ **Only the guard state is built.** A shield in flight is a damage source, and a planted
> one is a marker you can recall and leap to — nothing collides with either. No projectile
> path in the simulation knows the shield exists, so *"it holds space over there"* is the
> class's central promise and is currently fiction. It is the largest unbuilt thing in this
> document, and [../elementalist.md](../elementalist.md)'s "shared blocking implementation"
> is half-done in the other direction: structures do block bodies and shots.

## Weight — built (M1)

**Every blow taken on the shield is stored in it**, in health: a blocked hit stores its damage,
a parried one `Weight, a parry loads` times that, up to `Weight, the most the shield holds`.
It drains on its own, a full shield emptying in `Weight, a full shield empties in`, so it is
about the current exchange. It rides on every shield state — held, in flight, planted — so a
shield thrown with three hits in it lands with them. All of it is `crate::bulwark`, and the
only place that loads it is the one place a blocked blow is resolved (`state::apply_hit`, plus
the Elementalist's poke), which is why the creature's blows load it with no code of their own.

**What it does today, unspent:** a heavy guard is pushed back less — blocked knockback times a
line from one at empty to `Weight, pushback at the cap` at full. That is the trait
[../defense.md](../defense.md) promised and never built.

**Drawn**: the shield thickens and darkens as it fills, and the HUD's mechanic line reads
`shield held, weight: N`. `cargo run -p sim --bin weight` prints what each class's opener and
each creature move deposits, the drain, and the shove at five weights; the frame table prints
the four knobs under the Bulwark.

**Health**: 1250 against everyone else's 1000, from the `Health` family in the Oven — one
multiplier per class, shared with the Reaver's v2.

**What does not spend it yet**: Slam (M2) and the throw and the planted wall (M3). Until then
weight is a number you can see that changes one thing.

## What is bound today

Three moves and the mechanic. The shared grammar puts the poke on the bare click, so Bash sits
one rung lower than the six-ability sketch below implies — and Slam sat on the modifier until
shift stopped modifying clicks on 2026-09-16, which left it with no input at all.

| Input | Move | Bound |
| --- | --- | --- |
| `L` | **Bash** — the shield strike | yes |
| ~~`shift` + `L`~~ | **Slam** — the overhead | **no button since 2026-09-16** |
| `Q` | **Grapple** — the command grab | yes |
| `R` (hold) | **Guard**, with the parry in its opening frames | yes |
| `E` | **Throw** / **Recall** / **leap to it**, by shield state | yes |

`cargo run -p sim --bin frametable` prints the live numbers. There is **no separate off-hand
auto**: `L` is Bash, which is the shield itself.

Input map in [../controls.md](../controls.md).

## Auto attack

Intended as a short melee with the off hand — unremarkable on purpose, because the class does
not win by autoing. **Not built as its own move:** left click is Bash, so the poke and the
shield strike are one thing today. Whether the class wants both is open.

## Abilities

### Guard — right click, held
**Startup** instant · **Recovery** short · **Range** facing arc · **Mechanic** requires the
shield held

Hold to block. Blocked hits knock you back and stunlock you briefly; the Bulwark's stunlock
is short. **The opening frames are a parry** — timed correctly it staggers the attacker
instead. One input, two outcomes.

### Bash — `L`
**Startup** fast · **Recovery** short · **Range** melee · **Mechanic** requires the shield
held

A short forward shield strike. The safe poke, and the natural follow-up to a parry stagger.

> **Implemented**, on the bare click rather than on `shift` + click. 4/3/10, −4 on block and
> +2 on hit, and it is the one move in the game the repeat lockout actually charges: it costs
> 17 frames and the lockout is 30, so throwing it twice in a row waits 13. See
> [../combat-kernel.md](../combat-kernel.md) §"The repeat lockout".

### Slam — and it has no input at the moment
**Startup** slow · **Recovery** committed · **Range** short area · **Mechanic** requires the
shield held

> **Stranded, 2026-09-16.** Shift plus a click is not an attack input any more, on any class —
> see [../controls.md](../controls.md#shift-is-one-verb-now-2026-09-16). Slam is still in the
> table, still tuned and still printed by the frame table, and there is no button that throws
> it. Finding it a home is its own job: the right answer is different per class, and guessing
> three of them at once is how a grammar gets worse.

Drive the shield into the ground. Shakes the ground and staggers everything close.

> **Implemented** as a flat overhead — 14/4/24, and a crouching opponent does not duck it.
>
> **The velocity scaling is not built.** The intent is that damage and stagger strength scale
> with how fast you were falling, so the move rewards being thrown out of a jump or a leap
> recall; today it deals the same wherever it is thrown from. The Champion's aerial hammer has
> the machinery this wants (`champion::slam_damage_per_m/s` charges a spiked victim for the
> speed they land at), so it is a knob and a branch rather than a system.

### Throw / Recall — `E`, the mechanic
**Startup** instant · **Recovery** none · **Range** the throw's own reach · **Mechanic**
cycles the shield through its three states

One key, and which of three things it does depends on where the shield is.

- **Held → thrown.** It flies at what the crosshair is on — a skillshot, not a ray from the
  chest — damaging along its path, and plants where it lands. You are now faster, exposed,
  and unable to block.
- **Planted → recalled.** It flies back, damaging everything along the return path.
- **In flight → leap to it.** Pressed while it travels, you are thrown toward it and arrive
  where it is.

> **Implemented**, and **with no frames at all**. The kit above specified a startup and a
> recovery for Throw and Recall as though they were abilities; they are the class mechanic,
> and the mechanic fires on the press like every other class's. That is a real difference from
> the Reaver's Send shadow, which *is* a move in the table precisely because throwing a second
> body across the arena has a wind-up somebody can punish — see
> [../controls.md](../controls.md#where-e-is-an-ability). **Whether the shield should pay the
> same price is open**, and it is the one place this class gets something for nothing.
>
> What is not built is the planting: a planted shield stops nothing, per the warning above.

Recall is the class's mobility and its approach tool — throw to commit, leap to follow. It is
*meant* to set up Slam, since arriving from a leap carries downward velocity; that only pays
once Slam scales with it.

### Grapple — `Q`
**Startup** slow · **Recovery** committed · **Range** melee · **Mechanic** requires the
shield held

A command grab. Beats blocking outright, loses badly to dodge. Long startup and a punishing
tail — this is the payoff for a read on a turtling opponent, not something to throw out.

> **Implemented** (`Q`). It grabs: the victim is pinned at the Bulwark's arm's length and goes
> where he goes until the hold ends. Not hitstun with a longer timer — hitstun is something you
> recover from where you stand, and a grab is something that takes you somewhere.

## Playing it

Hold space with Guard and Bash, aimed so that the only open approach is the one you want
them to take. Throw and leap to close, land Slam out of the leap, Grapple when they start
blocking instead of moving.

In coop, plant the shield in front of the party as cover, or hold it wide to body-block for
someone who cannot take the hit — **both of which want the planted volume that is not built
yet**. Today a thrown shield is a projectile and a landing spot, and nothing more.

## Open questions

- **Should the mechanic cost frames?** It costs none, which makes the Bulwark the one class
  whose mechanic press is free. The Reaver's is a move in the table for exactly the opposite
  reason.
- **Can the planted shield be destroyed, or only bypassed?** Bypassed is simpler and makes
  positioning the only counterplay. Moot until it stops anything.
- Does the shield block allied projectiles? A real coordination mechanic in coop and
  potentially maddening.
- Does holding the shield wide for an ally need its own input, or is it just the Guard arc
  being large? Probably the latter — fewer inputs is better.
- Is six enough here? This class has the most state and may need a seventh for a dedicated
  ally-cover stance.
