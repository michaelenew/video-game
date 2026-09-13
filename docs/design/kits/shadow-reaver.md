---
status: proposed
decided: 2026-09-09
revised: 2026-09-13
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
Guillotine erupts *at* it, and the mechanic key *is* it — and a copy of either would be
the same ability fired twice from the same place.

Input map in [../controls.md](../controls.md).

## Auto attack

Melee, on left click. Decent damage, thrown constantly, and it is the move the shadow
copies most.

> The archive's *first auto after reclaiming your shadow deals bonus damage scaled by
> dexterity* is unbuilt, and now sits oddly: reclaiming is no longer a moment, it is a
> dash you chose. Parked rather than cut.

## Abilities

### Send shadow — `E`
**Startup** fast · **Recovery** short · **Range** the throw's own reach · **Mechanic**
sends the shadow out, or calls it home

A real move with a wind-up you can be punished during, rather than an instant state flip —
which is what it used to be, and what made the class's whole setup free.

**Out:** the second body races to where the crosshair is pointing and stops there.

**Home:** pressed again, it dashes back through anything between the two of you, damaging
and slowing it. **And it takes an open Guillotine lotus with it**, which is the
combination the kit is built around — see below.

### Guillotine lotus — `Q`
**Startup** fast · **Recovery** short · **Range** at the shadow · **Mechanic** requires
the shadow; keeps it

**Six blades erupt from the shadow along curving paths, hang open, and then chase it
home**, cutting on the way out and again on the way back.

- Out fast, on a curve: each blade leaves on its own sixth of the circle and keeps
  turning, so the six of them open like petals rather than spokes.
- They hang at full extension for about two thirds of a second. That is the window a
  victim has to leave.
- Coming back they decelerate into the shadow, and deal a share of what they dealt going
  out.

**The blades track the shadow's live position every frame.** Recall the shadow with a
lotus open and the six of them are dragged the length of the arena after it — a long
scything smear through everything in between. That is the class's biggest turn, and it is
two buttons: `Q`, then `E`.

> **Implemented.** The move itself has no volume at all; it places the flower and the
> flower does the hitting, which is why the frame table shows it with no damage of its
> own. It is the one move in the game aimed *at the mechanic* rather than at the crosshair
> or along the body — the player aimed when they placed the shadow. See
> [../aiming.md](../aiming.md).
>
> It briefly erupted **on the Reaver's own body**: the move was declared a swing with a
> reach of zero, so its volume came out where she was standing.
>
> The blades are tested as **swept lines** rather than as points, and that is a hit test
> rather than a flourish: the eruption crosses four metres in seven frames, so a blade
> sampled as a ball starts the frame at the shadow's feet and ends it a metre past
> whoever was standing there — missing the one victim the ability is named for.

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

### Executioner — right click, or `shift` + left click
**Startup** slow · **Recovery** committed · **Range** short · **Mechanic** stronger with
the shadow, which now always applies

Blink upward, then slash down in a long arc. The big commitment, and an overhead — it is
the class's answer to a turtle.

On **right click** as well as the shared grammar's `shift` + left click, because right
click is otherwise dead on a class with no shield to raise and the kit has always
described the Reaver as fighting with both hands.

### Deadly mistake
**Startup** fast · **Recovery** long on whiff · **Range** self · **Mechanic** requires the
shadow for the teleport

A brief counter stance. Passive: enemies that attack your shadow bleed. If struck during
the stance, you appear behind the attacker and leave your shadow where you were.

**Unbuilt**, and it now needs a button. `Shift` + right click is the obvious candidate and
has not been decided.

## Playing it

Send, act off the line, dash back onto it, send again. The skill is keeping the shadow
somewhere that gives you an escape *and* a threat at once, which are usually different
places — and knowing when the quarter-damage copy at your own shoulder is worth more than
either.

The two-button turn: `Q` opens the lotus wherever the shadow is standing, `E` drags it
home through everything in between.

## Open questions

- Does the shadow have collision, or is it purely a marker? Collision makes it
  denial-able, which cuts both ways. **It has none today.**
- **Where does Deadly mistake go?** It is the only ability in the kit with no input.
- The archive's bonus on the first auto after reclaiming the shadow. Reclaiming is a dash
  now, so the trigger exists — nobody has decided whether the bonus should.
- **Six blades and a quarter-damage copy is a lot of numbers hitting at once.** Nobody has
  played against it. The per-blade damage is deliberately small for that reason.
