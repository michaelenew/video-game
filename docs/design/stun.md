---
status: built, unplayed
decided: 2026-09-13
built: 2026-10-21
implements: the hit reaction between fighters -- knockback that grows through a round
---

# Stun: what happens to whoever got hit

Every blow one fighter lands on another freezes both of them, interrupts the
victim, and shoves them. This document is about the **shove**, and about how
it changes as a round goes on, which is where the variety in a duel comes from.

This is modelled on Smash, deliberately. The [combat kernel](combat-kernel.md)
already says the mechanical test is "Smash, not Tekken", and the hit reaction
is where that choice lives: a fight made of exchanges you commit to, rather
than a fight made of input strings.

**History.** The design was written and built on 2026-09-13 on a branch that
never merged (`claude/dreamy-hawking-sqnadm`). Two of its pieces reached main
separately and differently: the freeze on contact and the input kept through
it, as the per-move *Impact freeze* column of 2026-09-26
([feel-log.md](feel-log.md), "Give blows weight"). The rest is built here,
reimplemented on the game as it stands on 2026-10-21.

## Not stagger

| | **Stun** | **Stagger** |
| --- | --- | --- |
| **When** | Every time you take damage | Only when you are read: a parry, a Clash, a cash-in |
| **How long** | A few frames to half a second | Long enough to cash in the slowest punish |
| **Where it leaves you** | Wherever the knockback puts you | Where you were standing |
| **Why it happened** | Someone hit you | You made a mistake |

Stun is the tax on being hit. Stagger is the prize for a correct read. See
[defense.md](defense.md).

## What a hit does

1. **Freeze.** Both bodies hold still for the move's *Impact freeze*, and
   buttons pressed during it are kept. Built 2026-09-26; `tests/impact.rs`.
2. **Interrupt.** Whatever the victim was doing ends. `Hit::interrupts`.
3. **Shove.** A velocity along the blow, plus any launch, decaying while the
   stun runs (`Hitstun decay`).

Everything below changes the third, and how long the second lasts.

## The swell: damage makes the next hit bigger

Smash's percent, in a game whose bar counts down instead of up.

A move's `launch` and `hitstun` in the move table are **what it does to
someone untouched**. Its `knockback` starts a little above the table, at
*Knockback at full health* (1.5). Between fighters, all three grow with the
share of their own bar the victim has already lost:

```
hurt      = 1 - health / full health          (0 untouched, 1 at death)
ceiling   = 1 + swell + blow x share of the bar this blow took
knockback = table x (start + hurt x (ceiling - start)) / weight
launch    = table x (1     + hurt x (ceiling - 1))     / weight
hitstun   = table x (1 + hurt x stun swell x decay^hits in the string)
```

**The shove starts higher and climbs more slowly** (2026-10-21, from play:
"0 damage knockback doesn't feel like you can use it at all"). The start
lifts the bottom of the line, and the ceiling at death is where it was, so a
higher start is a gentler climb rather than a bigger finish. The launch keeps
starting at the table, because the follow-ups into the air (the Champion's
leap after an upcut) are tuned to its height. So early in a round a blow
sends somebody a little flatter than its author wrote; by the end the two
agree again.

Two terms in the knockback, as Smash has: one grows with what the victim has
lost, and one with the size of the blow, so a committed move scales harder
against a hurt opponent than a poke does. Without the second, a jab would turn
into a launcher just by being thrown late.

Damage lands **before** the swell is read, so the killing blow of a round
carries the largest swell there is, and both players see the round end.

Knockback swells far faster than stun does. That gap is the combo design: as a
round goes on the stun grows and moves start to link, and the knockback,
growing faster, throws the victim out of reach again.

### And it fades through a string

Each hit taken without getting a turn back keeps `combo decay` of the hitstun
swell the hit before it had. The move table's own number is never touched, so
a hit always holds you for at least what it says. What a string loses is the
bonus the victim's lost health was paying.

Distance ends a melee string, because the victim is thrown out of reach. A
ranged move re-aims at wherever they landed, so distance cannot end its
string. This does. The count is `Player::taken`, and it clears the moment the
victim is free to act again.

## Weight

Each class divides a fighter's knockback by its own weight. The Bulwark is the
heaviest and the Dual mage the lightest.

Weight divides knockback and **not** hitstun. Smash derives hitstun from
knockback, so weight moves both; here frame data is authored per move, and a
class-dependent stun would mean the frame table's "on hit" column stopped
being a property of the move. So a heavy takes the same stun and travels less
far out of it, which makes heavies combo food. A platform fighter gives the
same answer.

Weight divides **blocked pushback** too, which is where
[defense.md](defense.md)'s "the Bulwark resists pushback" comes from without
being written twice. It stacks with the shield's own `heavy_pushback`. Blocked
pushback never swells: the swell is about damage taken, and a block takes
none.

## Steering out of a launch

**A fighter launched by another fighter keeps their air control.** That is the
whole mechanic, and it is deliberately not a second one.

Smash calls this directional influence. The game already has the thing it
imitates: Quake air strafing turns your velocity without adding to it. Point
the way you are going and there is nothing to add; point across it and you get
the whole budget. So a launched fighter picks *where* they land, never how
far.

**It may never slow you down.** The air speed cap is about how fast a fighter
may make *themselves* go, and knockback routinely exceeds it. So the flat speed
is put back along the new heading afterwards (`steer_while_stunned`). Pressing
a direction is never worth less knockback than pressing nothing.

**It is an air privilege.** Grounded stun steers not at all: skidding is not a
moment anybody is flying through.

## A grab that launches throws on release

A held fighter is pinned to their captor, with their velocity zeroed every
frame (`drag_the_held`), so a launch handed to them on contact would be gone
before it moved them. It waits instead, in `Player::thrown`: held, you go
where your captor goes; let go, and you are thrown.

No move on 2026-10-21 both grabs and launches, so this changes nothing in play
yet. It is here so the first throw built does not have to discover it. The
Champion's uppercut is the move it was written for: its kit entry once read
"launch, and hold on".

## Between fighters only

**None of this applies to a blow from a creature, a critter or the world**:
only to a `Hit` whose `by` is a fighter. In a hunt, a blow does exactly what
its table says.

This is not a balance decision. The swell is a fighter's share of *their own
bar*, and it is fair only because the other fighter has a bar of the same kind
and runs along it at the same rate. Take turns being hurt and you take turns
being launchable, which is why it reads as an arc rather than as a punishment
for falling behind. A creature carries many times the health and is never held
longer in return, so the swell in a hunt would be a one-way ratchet. When the
branch first measured it on 2026-09-13, it took the scripted hunt from four
wins in six to none. Weight and the steering follow the same line, so a hunt's
numbers stay exactly what its species were tuned against.
`tests/ridgeback_pin.rs` holds every class's hunt bit for bit with the two new
fields left out of the hash.

## What it measures

`tests/knockback.rs`, the Champion's hammer, the victim on open floor:

| Victim | Carried |
| --- | --- |
| Champion, full health | 1.6 m |
| Champion, half | 2.4 m |
| Champion, a fifth | 2.9 m |
| Bulwark, full | 1.1 m |
| Dual mage, full | 2.0 m |

With the shove starting at the table (the first build, the same day) these
were 1.1, 2.2, 2.8, 0.8 and 1.4 m.

## Open numbers

All in the Oven under **Knockback**. None of them can be reasoned out.

- **The swell.** The branch ran +4.5 at death and +5 per blow share, against a
  game whose knockback was half what it is now (the 2026-09-26 pass doubled
  the heavy hits). These start at +2 and +3, so the distances above are about
  what the branch meant. The start, 1.5, came from the first play: at 1 a
  fresh hit's shove was too small to use. Whether 1.5 is enough is the next
  thing to play for.
- **Stun swell +1 at death, decay 0.3.** As built on the branch. Whether a
  Champion string at a quarter bar now links where it did not is the first
  thing to play for.
- **The weights** are the branch's: Bulwark 1.35, Blood mage 1.05,
  Champion 1, Reaver 0.92, Elementalist 0.85, Dual mage 0.78.

## Not in yet

- **The swell on the view.** The HUD prints `launch xN.N` beside a fighter's
  class between fighters. A real reading, for example the body's outline
  warming as it grows, is the view's to design.
- **Tumble, knockdown and teching.** At some knockback a victim should hit the
  ground rather than land on their feet, with a timed input to recover. This
  is the obvious next piece now that hits throw people.
- **Stale moves**: weakening a move that was just used. The stun half of that
  is the string decay above; the damage half is not built.
