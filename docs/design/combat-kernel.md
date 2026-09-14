---
status: decided
decided: 2026-09-09
supersedes: partially supersedes docs/archive/combat-design/*
---

# Combat kernel

The rules every class is designed against. Where an older document under
`docs/archive/combat-design/` conflicts with this one, this one wins — those documents are
revisions written against the old system and are kept as source material, not as spec.

## Frame

- **Structure** — peer-to-peer, isolated battle arena. No open world at launch.
- **Modes** — coop against monsters, or versus.
- **Camera** — 3D, third person. Ability spectacle is an explicit design goal: abilities
  should be visually rewarding to use, and the camera exists to show them off.
- **Mechanical test** — Smash, not Tekken. Spatial and temporal commitment, whiff
  punishment, reads, and space control. Not input strings or execution chains.
- **Art thesis** — the closed arena means far less art, writing, and content than an
  open world. Design choices that add depth without adding assets are preferred, and
  this is a real tiebreaker, not a nicety.

## No cooldowns

Abilities have **no cooldowns**. They are balanced by:

1. **Frame data** — startup, active, and recovery windows. The cost of throwing out a
   move is the time you are committed to it and vulnerable during it.
2. **Resource cost** — where a class has a resource.
3. **Positional commitment** — where the move puts you, and what it gives up.

Rationale: cooldowns make the mechanical test *"did I have it available"*, which is an
RPG question. Frame data makes it *"was that the right thing to throw out right now, and
can I survive the recovery"*, which is a fighting-game question. Cooldowns also fight the
third-person camera — the player should be watching the fight, not a row of icons.

## The repeat lockout

*Added 2026-09-14. It is the one thing in the game that stops you using an ability,
so it is written down here next to the decision it has to answer to.*

**You may not throw the same ability twice in a row.** Throwing one locks *that
ability*, and nothing else, for a short window — **30 frames**, half a second,
shared by the whole roster and scaled per ability by a multiplier that is 100%
on everything until play says otherwise.

The problem it solves is the one frame data does not: frame data prices a move
against *time*, and the cheapest move in a kit is therefore the correct one to
throw most of the time. Six abilities that a player uses one of is not a kit,
and "ability spectacle is an explicit design goal" is empty if the spectacle is
the same animation forty times.

### Why this is not a cooldown

Three reasons, and the first is the one that matters.

**It never asks the RPG question.** A cooldown can ask *did I have it available*
because it takes an ability away from you while you are busy doing something
else. This only ever holds the ability you have this moment thrown, so the
answer is always "yes — everything else in the kit". It does not gate access; it
charges for **repetition**. The question it puts to a player is *what else have
I got*, which is a question about the kit rather than about a timer.

**It cannot be waited out profitably, because there is always something better
to do.** `feel::the_lockout_always_leaves_something_faster_to_do_than_wait`
pins that as a relationship: the idle frames a lockout can leave you with must
never outlast the cheapest other move in the kit. The moment they do, the
correct play becomes standing still, and the rule has quietly become the thing
this section rules out.

**There is nothing to watch.** No icon, no row, no timer — the lockout is
shorter than most animations, and the only ability affected is the one you are
currently watching yourself throw. The camera argument above survives intact.

### It is measured from the cast, which is what keeps it honest

The clock starts on the frame the move **comes out**, not when its recovery
ends. So a move that already commits you for longer than its own lockout leaves
recovery with the lockout gone, and never notices the rule at all.

That is not a detail, it is the calibration. At 30 frames, every committed heavy
in the game is in that group and every auto and fast poke is not — which is
exactly the set anybody would spam. A Bash costs 17 frames and is charged 13
more; a Grapple costs 53 and is charged nothing. The rule taxes cheapness, which
is the thing that made repetition correct in the first place.

The `lock` column of `cargo run -p sim --bin frametable` prints it, with the
idle frames as a note beside each move that has any.

### An ability is not "used" until it is spent

Two abilities are activated more than once per cast: **Send shadow** goes out on
one press and comes home on the next, and the **Guillotine lotus** hangs its
blades until a recall drags them home. For those, the lockout does not begin
until the last of it is back — and the press that spends the second activation
is not a new use of the ability, so the lockout must not eat it.

Getting this backwards either deletes the recall or makes the ability free to
spam, so which button reactivates which ability is **declared**, in
`moves::reactivates`, rather than guessed from what a move leaves behind. Send
shadow's own button reactivates it. The lotus's does not — its second activation
is on right click, so pressing `Q` again would be a *second* lotus while the
first is still hanging, and that stays locked.

How soon a reactivation may come is a separate per-ability number
(`Move::reactivate`), zero everywhere, and deliberately not the shared one:
*how long until this comes back* and *how soon may I use the half of it I have
already paid for* are different questions.

### Prior art in the game already

The Ridgeback has played by a softer version of this rule since it was built: a
**variety penalty** that scores down whatever it did last, decaying over
`variety_frames`. The creature was already designed on the principle that
repeating yourself should cost something. This is the player-facing half of it,
and it is harder than the creature's because a player can mash and an appetite
score cannot.

### What is open

The multiplier per ability, and whether 30 is the number at all. Both need
someone to play it — see [feel-log.md](feel-log.md).

## Time to kill

| Mode | TTK |
| --- | --- |
| Versus | ~60 seconds |
| Coop | 1–20 minutes, by fight difficulty |

Consequences that flow from the 60-second versus number:

- A 2–3 second burst window is roughly 5% of a match. Burst states can be short and
  still be decisive.
- Health-as-resource classes (Blood mage, Dual mage) are tunable now that TTK is fixed.
  They were previously blocked on this number.
- Sustain and healing must be small. In a 60-second match, meaningful healing trivially
  becomes a stall strategy.

## What the no-cooldown decision breaks

The following exist in the current documents and are expressed in cooldown terms. Each
needs re-expression before it can be used.

- **The Affinity stat** — currently defined as "increases cdr". With no cooldowns it has
  no meaning. Proposed replacement: **Affinity reduces ability recovery frames**. That is
  the direct analogue — it was always "how soon can I act again" — and it keeps the stat
  meaningful without reintroducing cooldowns.
- **The entire Dual mage resource system** — human/balanced/divine are defined as 20% /
  40% / 95% CDR. Fully reworked in [dual-mage.md](dual-mage.md).
- **Weapon enchantments** — "Taking damage reduces all cooldowns based on the percentage
  of your current health lost" (Evolution 2) and "Ravenous ... grants cooldown reduction"
  (Evolution 3). Both need new effects.
- **Blood mage seals** — "using a seal incurs a universal cooldown on all seals" survives
  in spirit as a **shared lockout window** between the seals, which is a resource
  constraint rather than a per-ability cooldown. Keep the constraint, rename the concept.
- **Shadow Reaver** — "Shadow swap ... goes on ¼ of normal cooldown" and "Rifen ... long
  cooldown" need recosting as frame data and shadow-state constraints.

## Open kernel questions

These block class work and should be settled next.

### Block and parry — proposed, see [defense.md](defense.md)

Was unspecified and blocking two classes. Now proposed: dodge is universal and evasive,
block is shield-gated and positional, parry is the opening frames of block and rewards
with a stagger, and blocking costs space through pushback rather than health through chip
damage. Special attacks are the guard breakers.

### Other open items

- **Arena size and shape.** Directly determines whether a committed, space-denying class
  can ever corner anyone.
- **Stocks or a single health pool** in versus.
- **Progression and equipment are parked** until the core classes are built. See
  [parked.md](parked.md), which also covers the bow.
- **Forced engagement in versus.** At 60-second TTK, two players who both decline to
  approach produce nothing. Whether the answer is a shrinking arena, an objective, or
  purely class-level pressure is undecided.
