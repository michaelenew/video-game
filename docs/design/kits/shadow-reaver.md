---
status: proposed
decided: 2026-09-09
revised: 2026-09-14
sources: docs/archive/combat-design/shadow-reaver-skills.md, docs/archive/combat-design/class-builds.md
---

# Shadow Reaver — kit

**Identity.** Two bodies. Every option is a function of the line between you and your
shadow. Tactical positioning in a second dimension, cashed out in burst.

## Mechanic — the shadow

**A second body that is never away.** It is at your shoulder, or it is out on the field.
There is no third state.

That changed on 2026-09-13 and it is the most load-bearing decision in the class. The
shadow used to be a `Some`/`None`: half a match was spent with no shadow at all, which
meant half the kit was greyed out, and every ability that read the mechanic carried a
branch for the case where the mechanic did not exist. A line needs two ends. Now it always
has them, and *where* the far end is is the only question the class ever asks.

| State | What it is |
| --- | --- |
| **Attending** | A translucent grey copy of you, trailing a metre behind your shoulder |
| **Going out** | Racing to the spot the crosshair picked, ten frames, and then it stops |
| **Waiting** | Standing there. The state everything else in the kit is aimed at |
| **Coming home** | Dashing back through anything in the way, cutting and slowing it |

**Leash.** Stray far enough from a waiting shadow and it comes and finds you, on the same
path a recall takes and doing the same damage. The leash is deliberately longer than the
throw, so a shadow arrives with room to spare rather than turning round on the frame it
lands.

### It copies you

**Every swing you throw, the shadow throws too, a few frames later, for a quarter of the
damage.** From wherever it is standing.

That is one sentence and it is most of the class:

- **Holding the shadow is a flat 1.25× on everything your body does.** The cost is that
  the second body is at your shoulder instead of somewhere useful.
- **Sending it out is trading that quarter for reach.** The copy still comes out — it
  comes out over *there*, so a Reaver with the shadow well placed threatens ground she is
  not standing on, and a fight in two places at once is what the class is for.

The copy is blocked, ducked and spaced by exactly the rules your own swing is. It cannot
be **parried**, because a parry is a stagger paid by the attacker and there is nobody at
that end of the blow to stagger; reading it perfectly still stops it dead.

It copies **swings only**. The other two things you throw are already the shadow's — the
Guillotine erupts *at* it, and Send shadow *is* it — and a copy of either would be the
same ability fired twice from the same place.

Input map in [../controls.md](../controls.md).

## Auto attack

Melee, on left click. Decent damage, thrown constantly, and it is the move the shadow
copies most.

> The archive's *first auto after reclaiming your shadow deals bonus damage scaled by
> dexterity* is unbuilt, and now sits oddly: reclaiming is no longer a moment, it is a
> dash you chose. Parked rather than cut.

## Abilities

### Send shadow — right click
**Startup** fast · **Recovery** short · **Range** the throw's own reach · **Mechanic**
sends the shadow out, or calls it home

A real move with a wind-up you can be punished during, rather than an instant state flip —
which is what it used to be, and what made the class's whole setup free.

**Out:** the second body races to where the crosshair is pointing and stops there.

**Home:** pressed again, it dashes back through anything between the two of you, damaging
and slowing it. **And it takes an open Guillotine lotus with it**, which is the
combination the kit is built around — see below.

#### The press answers whatever else she is doing — added 2026-09-14

Two rules, and they are the same rule twice:

- **It cuts a recovery short.** Press it during the tail of any move and that tail ends.
  This is the Champion's Rush cancel, arrived at independently, and it is the second one
  in the game.
- **The press is remembered** for twenty-four frames, so one thrown early comes out on the
  first frame she can spend it instead of vanishing. Being hit throws it away — the memory
  is there so her own kit cannot eat the mechanic, and a stun is not her own kit.

Every other button in this kit is an attack, and an attack eaten by another move's frames
is the game correctly telling you that you were busy. **This one is not an attack.** It is
where the second body stands — and the line between the two bodies is the class's escape
from both the ordinary limits on where she can be and the ordinary limits on what she can
reach. A mechanic the rest of her own kit can lock her out of is a mechanic sitting behind
a timing test. It was one: sending the shadow meant waiting out a swing and then hitting a
single frame, and chaining anything into it was a matter of luck.

**What it does not do** is reach back into a startup or an active frame, or out of a stun.
Recovery is the part that is *over* — the animation finishing, not the decision. A cancel
that reached into the wind-up would let her take a committed swing back after throwing it,
and one that reached into a stun would make the mechanic an escape from being hit, which
is the opponent's reward and the only thing in the game that is meant to take the controls
away.

**And it buys tempo rather than safety**, which is why it needs no charge behind it the way
Rush does. Send shadow costs nineteen frames and the longest recovery it can cut short is
Executioner's twenty-six, so the exchange nets her seven: she is busy for nearly as long
either way, and what changes is that the frames do something. The trade is a recovery for
the mechanic, not frames bought back — a blocked Executioner goes from −12 to about −5 and
is still a punish.

Those two counts are knobs and the relationship is not:
`cutting_a_recovery_short_does_not_rescue_her_from_the_punish` in
`crates/sim/tests/reaver.rs` fails if the cancel ever turns a blocked commitment safe, so a
tuning pass may move either number and cannot lose the rule.

### Guillotine lotus — `Q`
**Startup** fast · **Recovery** short · **Range** at the shadow · **Mechanic** requires
the shadow; keeps it

**Twelve blades erupt from the shadow along curving paths, hang open, and then chase it
home**, cutting on the way out and again on the way back.

- **Each one is a disc.** A shuriken thrown flat: wide in the plane the flower lies in,
  and barely there across it. It was a sphere of nearly half a metre — wider than a
  fighter's own body and reaching from the shins to the chest — which read as a beach
  ball rather than a blade, and was not a shape anybody could do anything about.
- **The flower is flat.** It opens, holds and closes in one horizontal plane at the
  shadow's midriff — chest-high on nobody, waist-high on everybody, and the same height
  for the whole of its life.
- Out fast, on a spiral: each blade leaves on its own twelfth of the circle and keeps
  turning as it reaches, so the twelve of them open like petals rather than spokes.
- They hang at full extension for about two thirds of a second. That is the window a
  victim has to leave.
- **Home on a spiral of its own, turning the other way.** Coming back a blade sweeps
  against the direction it opened in, and further, so it crosses the bearing it started
  from rather than unwinding onto it. It decelerates into the shadow and deals a share of
  what it dealt going out.

**So the flower can be jumped**, and that is the counterplay the shape always implied and
never delivered. A thin slab at waist height is something a fighter can get over; a sphere
from the shins to the chest was not. Leaving sideways used to be the only answer.

**Twelve small blades rather than six big ones, at half the damage each.** The danger is
meant to be the shape it sweeps rather than any one thing landing, and a single blade is
now a scratch — under a third of her own poke. What that buys is a flower that reads as
petals: solid near the shadow, and **gapped at the rim**, where the twelve arms are far
enough apart to stand between. Measured at full extension, where you are standing relative
to a petal is worth the difference between taking nothing and taking a third of what the
middle deals. That is spatial counterplay of the kind the class is supposed to be made of,
and it is why the blades are not simply made wider to close the gaps.

That last one is a hit test as much as a look. A return that retraced its outward arm
would be sweeping ground it had already cut — ground whose occupants were cut once and
have had the whole hold to walk off it — so it could only catch somebody who stepped back
into the same line. Turning past the start means the way home crosses floor the way out
never touched, which is what makes the drag through a crowd the ability's own description
of itself rather than a second helping of the first pass.

**The blades track the shadow's live position every frame.** Recall the shadow with a
lotus open and the twelve of them are dragged the length of the arena after it — a long
scything smear through everything in between. That is the class's biggest turn, and it is
two buttons: `Q`, then right click.

> **Implemented.** The move itself has no volume at all; it places the flower and the
> flower does the hitting, which is why the frame table shows it with no damage of its
> own. It is the one move in the game aimed *at the mechanic* rather than at the crosshair
> or along the body — the player aimed when they placed the shadow. See
> [../aiming.md](../aiming.md).
>
> It briefly erupted **on the Reaver's own body**: the move was declared a swing with a
> reach of zero, so its volume came out where she was standing.
>
> The blades used to leave the shadow's **feet** and arc up and back down over the
> eruption. Two things were wrong with that and only one of them was how it looked: a
> blade at full extension was back at ground level, so the volume spent the end of its
> travel half buried in the floor at exactly the reach where it does the most work. The
> flower is planar now, and the height it lies at is its own knob.
>
> The blades are tested as **swept lines** rather than as points, and that is a hit test
> rather than a flourish: the eruption crosses four metres in seven frames, so a blade
> sampled at a point starts the frame at the shadow and ends it a metre past whoever was
> standing there — missing the one victim the ability is named for.

### The dash to the shadow — `shift` + forward
**Startup** instant · **Range** the leash · **Mechanic** requires the shadow out;
collects it

**With the crosshair on the shadow, a forward dodge becomes the dash to it.** Invulnerable
across the gap, at a constant speed that crosses the whole leash inside one dodge, and
arriving picks the shadow up.

Pointed anywhere else, or thrown in any other direction, it is the ordinary dodge. That is
the design: the class's mobility and the universal defensive option are the same button,
so the Reaver can never be denied either — and choosing between them is a flick of the
mouse rather than a second key.

It replaces two abilities from the original list, **Shadow swap** and **Reascend**, both
of which were "get to the shadow" with a different verb attached. One movement option,
aimed.

#### Three things it did not used to do — added 2026-09-14

All three are the same sentence read properly: **the dash goes to where the second body
is.** Not to the floor under it, not only from the floor, and not only when the floor
between the two of them is flat.

**In the air as well.** A forward *air*dodge pointed at the shadow is the dash, exactly as
the grounded one is. Being off the floor is the commonest reason she is not standing where
she wants to be, and mobility that switched off the moment she jumped was mobility in the
wrong place. It is the airdodge, aimed, so it **spends the airdodge** — one commitment per
airtime is what keeps a jump from becoming flight — but it keeps the dash's own frames
rather than the airdodge's shorter ones, because a tail cut short strands her halfway.

**In three dimensions.** The line she flies is the straight line between the two bodies,
and it goes up and down as readily as along. A shadow standing on a dais is *up there*; the
old dash drove only her feet, so she ran at the side of the thing it was standing on and
stopped. Gravity and the arena are off for the crossing — she is a shadow going to her
shadow — and she arrives on the shadow's own spot rather than near it.

**And only a total obstruction refuses it.** Total means what it says: *no* straight line
from her body to its body, not "a ledge is in the way of her feet", which is every dash
onto anything. Two bodies are upright columns over fixed spots, so every line between them
has the same footprint and they differ only in how they rise — she takes any one of them
that gets through. Refused, she gets the ordinary dodge, which is the other half of the
button rather than a dead press.

Nothing in the blockout is tall enough to refuse her: the platforms and the walls are one
and a half metres and a fighter is one point eight, so she can always see over. **A
structure can.** An Elementalist's stone is exactly a fighter's height, so one raised
squarely on the line leaves no way through — which makes cutting the Reaver's line
something another class can actually do, rather than a rule nothing exercises. See
[../aiming.md](../aiming.md).

#### The carry, and the jump out of it — added 2026-09-14

**Arriving does not stop her.** She was crossing at more than thirty metres a second and
that speed is still under her; the frames after she lands are a slide that decays. That
window is the **carry**, and it is the same length however far she came — a dash that spent
its whole dodge getting there is given the window rather than having the distance swallow
it.

**A jump pressed inside the carry takes the slide up with her.** It is the one thing that
can cut a dodge's tail short, and that is half the reward: the frames she would have spent
standing there being punished are spent in the air going somewhere instead. The other half
is the speed, which decays while the window is open — so pressing early keeps more, and the
tech is a gradient rather than a pass mark.

It is a **press**, not a held button, and it answers only a dash. Every dodge in the game
has a punishable tail, and a jump out of *that* would be a universal escape rather than one
class's tech.

### Executioner — `E`, or `shift` + left click
**Startup** slow · **Recovery** committed · **Range** short · **Mechanic** stronger with
the shadow, which now always applies

Blink upward, then slash down in a long arc. The big commitment, and an overhead — it is
the class's answer to a turtle.

On **`E`** as well as the shared grammar's `shift` + left click. See
[Why the two buttons are swapped](#why-the-two-buttons-are-swapped).

### Deadly mistake
**Startup** fast · **Recovery** long on whiff · **Range** self · **Mechanic** requires the
shadow for the teleport

A brief counter stance. Passive: enemies that attack your shadow bleed. If struck during
the stance, you appear behind the attacker and leave your shadow where you were.

**Unbuilt**, and it needs a button that does not exist yet. Both obvious candidates are
swallowed: right click ignores `shift` and sends the shadow either way, and `shift` + `E`
throws Executioner for the same reason. Undecided, and the first thing that will have to
give if this ability is built.

## Why the two buttons are swapped

This class is the only one where `E` is not the mechanic, and where right click is. It
looks like an exception to the control grammar and it is the opposite — it is sentence six
of that grammar taken seriously:

> **The mouse means *where*.**

**Sending the shadow is the only thing in this kit the crosshair aims.** It is a grounded
cast: you point at a patch of floor and the second body goes and stands on it, and where
you put it is the whole decision the class is made of. That belongs under the hand that is
already doing the pointing.

**Executioner does not care.** It is a swing off the body — the yaw is your facing, the
pitch is the camera's, and nothing about it reads the crosshair as a place. It can live on
a key, and it does.

So the split is not "mechanic on a key, abilities on the mouse". It is **aimed things on
the mouse, unaimed things on the key**, which is what the grammar said in the first place
and what four of the six classes get for free by having a mechanic with nothing to aim.

It was the other way round for a day, and the reason to prefer this one is not on paper:
sending the shadow is a *placement*, and placing something with the hand that is not
holding the mouse means committing to a spot you are about to stop looking at.

## Playing it

Send, act off the line, dash back onto it, send again. The skill is keeping the shadow
somewhere that gives you an escape *and* a threat at once, which are usually different
places — and knowing when the quarter-damage copy at your own shoulder is worth more than
either.

**Where you put the shadow is now also where you can stand.** Sending it up onto a dais and
dashing to it is a way onto high ground that costs a mechanic press rather than a jump
arc — and a jump thrown out of the carry on arrival turns the whole thing into one long
move across and up. That is the class's biggest piece of ground covered, and it is three
inputs: right click, `shift` + forward, space.

The two-button turn: `Q` opens the lotus wherever the shadow is standing, right click drags
it home through everything in between. Since right click cuts a recovery short, the turn
does not have to wait for the lotus to finish coming out — the drag is a decision made
inside the animation rather than after it, which is the same kind of decision the Champion
makes with Rush.

## Open questions

- Does the shadow have collision, or is it purely a marker? Collision makes it
  denial-able, which cuts both ways. **It has none today.**
- **How long should the carry be, and how much of the dash should survive it?** Ten frames
  is the first value and it is a guess. The slide itself is inherited rather than chosen —
  it is what the dodge's decay does to thirty-four metres a second — and it carries her a
  good four metres past the shadow, which on a dais is most of the way to the far edge.
  Nobody has played it.
- **Where does Deadly mistake go?** It is the only ability in the kit with no input.
- The archive's bonus on the first auto after reclaiming the shadow. Reclaiming is a dash
  now, so the trigger exists — nobody has decided whether the bonus should.
- **Twelve blades and a quarter-damage copy is a lot of numbers hitting at once.** Nobody has
  played against it. The per-blade damage is deliberately small for that reason.
