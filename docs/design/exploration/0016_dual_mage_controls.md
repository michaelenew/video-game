---
status: first half built 2026-10-12 (the wing autos and the hop); the rest proposed, waiting on the owner
started: 2026-10-12
supersedes: 0015's litanies, its air row (Shadowstep, Descent) and the bare-click half of its map
depends: 0015 (the curse, the purge, the sigils, the stars, the majors -- all kept), ../dual-mage.md (the two bars, the tiers, ascension)
---

# 0016 — The Dual mage's hands: what is missing

## The brief (the owner, 2026-10-12, after the spellbook)

> `Q` and `E` are superb as is, keep those. Spells are better but not great. The 3 click auto
> cycle is a little weird. Put her old autos back, the ones with the circle arcs, but make them
> much much larger so that she can actually hit things with them. I want her mobility to be
> tied to autos, so her game is about keeping distance from the enemy. Let's make her autos like
> Kalista's from LoL where she gets a short dash after each one in the direction she's holding
> to move.
>
> In my head what I want this character to do is jump in with a flurry of hits that come from
> the beings inside them, slowly being lifted off the ground as they dash about. As the energy
> inside builds, they vent it with bigger and bigger abilities, trying to stay on the razor's
> edge of overflowing.
>
> The control scheme here keeps tripping me up. It feels like we don't have enough keys or a
> simple enough scheme to capture this midair-dash-about aesthetic while still having true
> high-spectacle spells and keeping all the keys close enough the user doesn't have to move
> their hand. Build on this, what is missing?

## What was built today (the first half)

- **The wing autos are back** on the bare left and right clicks, on the floor *and* in the air
  (`moves::dual::DARK_AUTO`, `LIGHT_AUTO`). They are the curved blades of the 2026-09 kit, made
  much bigger: reach 3.8 m to the ring, a tip bubble 1.25 m round, so a fighter anywhere from
  half a metre to five metres in front is hit (`tests/dual_spells.rs`,
  `the_autos_reach_a_body_close_and_a_body_four_and_a_half_metres_off`). The tip, out at full
  reach, still hits half again as hard, so the best place to stand is a wing's length away,
  which is the "keep your distance" game. The dark wing lays a curse stack; the light wing
  purges, as 0015's lash and palm did.
- **The hop.** If a direction is held as the wing finishes, she is carried `auto, hop` (3 m)
  that way at `auto, hop speed` (22 m/s), and the carry *replaces* the auto's recovery: she
  can act as she lands (`state::hop_after_the_auto`, `aim::hop_to`). Held direction is read the
  way walking reads it, relative to the camera. Holding nothing, she stays.
- **The lift.** Each hop rises `auto, hop lifts` (0.3 m) plus up to `auto, hop lifts at full`
  (1.2 m) more as her *lower* bar fills (`dual::hop_lift`). With empty bars a string of hops
  stays on the floor; with full ones it carries her up, a hop at a time. That is "slowly being
  lifted off the ground as they dash about", tied to the same number the tiers already read.
- **The `F` key is the Devour and the `R` key the Smite**: one spell of each hand, as a stopgap until the
  question below is answered. The Grasp, the Halo, the Shadowstep and the Descent are in the
  move table with no button.
- Against a creature a wing is its **tip** (`state::part_under`): the blade opening behind it
  sweeps metres of a body that big and spent the swing on the first flank it brushed, so the
  weak point she put her tip on was never the part hit.

## What is missing — the diagnosis

It is not keys. Count them: the left hand rests on `WASD`, `space`, `shift`, `Q`, `E`, `F`, `R`
without moving, and the mouse has three buttons and a hold on each. Three things are missing,
and none of them is a button.

**1. A meaning that stays put while she rises.** 0015 made the *row* the verb: floor = fight,
air = move, leaving the floor = defend. That is a good grammar for a class that is mostly on the
floor. It is the wrong one for this fantasy, because the fantasy is that *succeeding* lifts her
off the floor. Every hop that worked moved her into a row where her clicks meant something
else, so the better she played, the more her buttons changed under her. That is the "tripping
me up". The fix built today is the first half of the cure: the clicks mean the same thing on
the floor and in the air. The rest of the kit should follow — **one meaning per button,
everywhere.**

**2. Movement wants no button at all.** Kalista's answer (League of Legends, *Martial Cadence*)
is that her dash is not an ability; it is a by-product of attacking, pointed by the movement
keys the other hand is already holding. That is why it never costs her a key. The same is now
true here, and it is the reason the air row's movement spells (Shadowstep, Descent) can go:
she moves by attacking. What remains is the dodge (`shift`) and Phase, both already bound.

**3. A way to spend.** This is the real gap. Today her bars only *fill*: autos, spells and
majors all push the bar up, and the only things that bring it down are time and the other
hand. So the "razor's edge" is a band she keeps herself inside by alternating and pausing —
the opposite of venting. The fantasy says the reverse: **the flurry fills, the spells vent.**
A big spell should *spend* bar, and be big *because* of what it spent. Then the edge is a real
decision with a real shape — how full do I let it get before I let it out — and overflowing is
what happens when you get greedy, not a timer you manage.

## Prior work, and where it stops applying

- **FFXIV's Black Mage** is the closest thing in the corpus to "stay on the edge": Astral Fire
  makes fire spells stronger and drains her faster, and the rotation is about spending the last
  of it (*Despair*, *Flare*) at exactly the right moment before swapping. What does not carry
  over: it is a stand-still caster whose skill is *not moving*. The part worth taking is that
  the biggest spell is the one that empties the tank.
- **Monster Hunter's Charge Blade** is the closest in melee: sword hits fill phials, and the
  axe's discharges (up to the *Super Amped Element Discharge*) spend them, bigger the more are
  spent. Over-fill the phials and they turn red and the sword *bounces* — overflow punishes
  greed. That is almost exactly "autos fill, spells vent, overflow hurts".
- **Kalista** for the hop, as above. What does not carry over: her hop is cancelled by
  *movement input during the attack*, which needs a click-to-move control scheme to be clean.
  Here the stick is always held, so the hop has to fire on the attack's last frame, which is
  what was built.
- **The Elementalist** in this repository: Raise is a *tap*, and holding the same click turns
  it into Fissure, charged by the hold (`exploration/0008`). That is the in-house precedent for
  one click carrying two weights of the same idea, and players of that class already know it.
- **Guild Wars 2's Elementalist** swaps attunements to change what every key means. That is the
  thing to *avoid* — it is the row problem with a nicer coat.

## Recommended scheme

| Input | Meaning, everywhere |
| --- | --- |
| Left click, tapped | Dark wing, and a hop the way you hold. Fills the dark bar a little |
| Left click, held past the wing | **Dark vent**: the wing's last frame becomes a cast that charges while held and fires on release — a Grasp at a light hold, a Devour at a full one. **Spends** dark bar; bigger the more it spends |
| Right click, tapped / held | The same with light: the light wing; held, the Halo then the Smite, spending light bar |
| Middle click | Twilight on herself, as now: Binary on the floor, Phase in the air; `space` + click are the takeoffs (Veil, Equinox, Sanctuary) |
| `Q` / `E` | The Abyss and Judgement, unchanged. The biggest vents: they **spend the whole bar** they are made of, and are sized by what they spent |
| `shift` | The dodge |
| The `F` and `R` keys | Freed. Thumb buttons (`M4`/`M5`) can mirror the holds for anyone who prefers a tap |

Why a **hold after the tap**, not a hold instead of it: the wing must come out on the press,
or every auto feels a beat late. So the press is always the wing; *keeping* the button down past
the wing's end is the decision to vent. Nobody holds a button by accident for a third of a
second, so the hold never steals a hop, and a hop never steals a hold: the hop is read on the
wing's last frame only if the button is up.

**The edge, with venting in.** Autos fill, holds and majors spend, and ascension is overflow:
let the higher bar touch the top and the beings come out on their own (Apotheosis, as 0015
built it). The band between the bars stays — it is still what the tiers are read from — but
the thing she watches is no longer "am I level", it is "how close to the top have I let this
get, and what is the biggest thing I can spend it on before it gets away from me".

## Open questions

1. **Does venting replace the band, or sit beside it?** The cheap version keeps the band and
   only adds spending. The clean version makes the higher bar's distance from the top the whole
   tension. The second is the fantasy; the first is a smaller change to test.
2. **Holding nothing, does she hop at all?** Kalista hops backwards from her target by default
   in practice. A hop *away from the crosshair* when no direction is held would make "keep your
   distance" the default rather than a skill. It also makes standing still impossible, which may
   be too much.
3. **How much lift is too much?** At a full lower bar each hop rises 1.5 m. Five hops in a row
   is seven metres, over the heads of the Bulwark and the Champion, who cannot follow. That is
   the fantasy, and it is also the thing most likely to make her unfun to fight.
4. **The air row.** If the clicks mean the same everywhere, the Shadowstep and the Descent have
   no home. The Descent could be what a light hold does *in the air* — the vent comes down with
   her — which keeps one meaning per button (a light hold is always "the light comes out of
   you") while the shape differs.
5. **Does `F`/`R` stay?** If the holds land, `F` and `R` go back to being free, which the
   Elementalist and the Reaver will want.

## Next

Built so far: the wings, the hop, the lift, `F`/`R`. The holds, the spending, and overflow as
ascension are the next pass, **waiting on the owner's answer** to question 1.
