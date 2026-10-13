---
status: built 2026-10-12 and 13 in three rounds (the autos and the hop; the long holds and the spending majors; the ball autos and tap-or-hold); unplayed
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

## The owner's second round (2026-10-12)

> First, the autos need to be more forward than a sideways sweep. Think big
> maplestory-battle-mage-esque arcs reaching far forward more than a sweep.
>
> Second, the dash on auto feels quite good. I do want you to make the bars grow a little
> faster with autos, and I need the bar thresholds to be even thirds, not half then 3/4.
>
> Third, q and e are not currently spending the bar, they still grow it. I like the idea of
> them spending the bar. They're both a reward for building and an oh shit button for getting
> out of balance.
>
> We need to make sure the jump-auto gives them a second jump in the air for extra height as
> well (still one per airborne period, but ascension lets you have as many jump-autos as you
> want).
>
> I like the hold-to-spell control scheme, we just need to make sure the spells feel good. The
> spells do need to vent as well, just not the whole bar. The reason we need more build on the
> autos is that they're becoming the way to build the bar. The spells become venting the
> energy, so the intended balanced playstyle between autos and spells becomes optimal to stay
> in balance and just sub-ascension.
>
> I like the extra mobility of middle click. I need its jump attack (both aerial and grounded)
> to rise with its blink and keep more momentum. [...] middle click makes sense for the main
> mobility jump-spell because it's balanced so it's not going to affect the dual mage's health
> as they do jump sections.
>
> For the spells I'm thinking more along the lines of Maplestory battle mage again. A big line
> of a flurry of blows, then the dual mage dashes invuln along it. A line of rays coming down
> from above, the first hitting the ground in front of her, then the next further, then again
> and again. A big light disk appears above her, then a moment later a blast of light comes from
> it to where she's aiming. Black light lightning from a cloud above her that chains to nearby
> creatures. A big flurry of blows (basically her autos but many more of them all at once) in
> front of her at slightly longer than auto range.
>
> For now write the disk ray (light long hold), chain lightning (dark long hold) and flurry
> dash (middle long hold) as new moves on the long holds.

**What was built**, all in `kits/dual-mage.md` §"Her hands" and `dual-mage.md` §"Two bars":

- **The wings thrown forward**: the ring's middle moved behind her and onto her own line, the
  reach to 6 m and the span narrowed, so each wing is a crescent out in front reaching seven
  metres dead ahead and five and a half to the punching side -- a slash thrown out, not a
  sweep round her.
- **Autos fill a little faster** (5 → 6 a wing) and **the tiers are at thirds**: the blink at
  33, the second jump at 67. The wings stay at 95 on both bars, for the reason `dual-mage.md`
  gives (at 100 the calm keeps one bar always just under while the other tops out: tried, and
  alternating never ascended). Clean alternating now climbs to the blink in 6.3 s, the second
  jump in 12.9 and the wings in 17.9, from 12.1, 18.8 and 23.8.
- **The majors spend the whole of their bar**, read for power before they spend it. The
  runaway is the high bar climbing while the low drains, so spending the high one lands her
  near level: the oh-no button. From level, the other major is the answer, so the Abyss and
  Judgement in turn are the spectacle and the reset.
- **Spells vent**: the Devour and the Smite spend 9 of their bar, the long holds 20, the
  Flurry dash 20 of each. The takeoffs and the middle click's taps fill like an auto -- they
  are jump attacks.
- **The air hop is a jump**: the first auto-hop of an airtime rises 2 m more; while ascending,
  every one does.
- **The middle click's jump attacks rise and carry**: Phase leaves her moving 9 m/s along the
  blink and rising at 6 m/s at least; the Equinox now blinks too, as she leaves the floor.
- **The three long holds**, as new moves: Chain lightning (dark), the Disk ray (light), the
  Flurry dash (middle). A click held past its wing is the hold, the middle click from its press;
  a hold let go early is nothing (the middle click's is its tap). The Disk ray needed a sixth
  line of effect, `aim::Kind::Overhead`: the skillshot's target, from the disk over her head.

**Not built yet**, from the same message: the line of rays stepping out in front of her, and
the big flurry of wings at a little past auto range. They want a second length of hold (a
short hold and a long one on each click), which is the next question.

## The owner's third round (2026-10-13)

> I don't know how to communicate how the autos need to work, because every iteration is
> building on the same bad implementation. I don't want them to be a sweep. I want them to be a
> ball that starts near the mage's shoulder, but a little (as in a foot or two) outside and a
> foot or two behind her. Then, as the mage punch animation plays, I want that ball to move
> MOSTLY forward out to its current forward-max-distance [...] As it moves forward, its path
> first moves out away from the mage a small bit (again a foot or two at most), then arcs back
> inward, finishing directly in front of the mage's chest N meters out with a sweet spot. The
> ball leaves a trail along its path.
>
> The light attack needs to be a circle whose face faces the same direction as the dual mage,
> not down like it is now.
>
> The thunder cloud and light attacks can both start their animations even if the press isn't
> long enough, and the auto animation can start even if you then hold long enough for a spell.
> It needs to be either / or, never both animations even in part.
>
> And I actually want the mouse spells to increase the bar, not decrease it.
>
> Middle, left, and right click long hold aren't supposed to be charges either. If you press the
> key for long enough you're locked into the spell, the animation will go through, and its
> strength has nothing to do with how long you pressed the button. That's the "MMO spell" part
> of the kit.

**What was built**: the wing taken out whole and the ball built new (`moves::orb_at`, one
function the hit test and the trail both read); the disk upright, facing her facing; every click
an undecided hold until it is let go (the tap) or held full (the spell, a commitment that plays
through and is worth her bar, not the hold); the long holds filling 10. All in
`kits/dual-mage.md` §"Her hands".

## Next

The short holds -- the stepping rays and the big flurry of wings -- and whether every click
gets a short and a long, or only the two that have a spell for each. And play: none of this
has been in a person's hands.
