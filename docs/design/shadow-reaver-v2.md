---
status: proposed — v2 of the damage pattern, nothing built
decided: 2026-09-23
supersedes: nothing outright; adds to kits/shadow-reaver.md, whose moves and shadow survive
sources: kits/shadow-reaver.md, docs/archive/combat-design/shadow-reaver-skills.md, this design thread
---

# Shadow Reaver — v2: the shadow keeps a tally, and arriving cashes it

**Identity.** An assassin with a second body. She works you from mid range with something
that is not her, and while it works you it is counting. Then she crosses the gap in one
invulnerable line, arrives at the shadow, and the first cut she lands spends everything it
counted. Then she is gone, because staying is the one thing the class does badly.

The shadow is the best thing in the kit and nothing here touches what it is: a second body
that is never absent, a dash to it that is the class's movement, a lotus that erupts from it
and a recall that drags through everything. What is missing is the **moment** — the frame
where setup becomes explosion — and a reason to leave melee once she is there. This proposal
adds both, and both come off the shadow.

**The action plan for building this is [plans/shadow-reaver-v2.md](plans/shadow-reaver-v2.md).**

## What is wrong today

Three things, and they are one thing.

**The copies miss at range.** The shadow's swing is her aim path translated to where it
stands, with her yaw. A copy thrown from six metres away is a quarter-damage melee arc
pointed wherever her shoulders happen to be, and it lands only if somebody is standing at
exactly her offset from it. The utility of the shadow at range is real and it is unusable.

**There is no burst.** Holding the shadow is a flat quarter on top of everything, sending it
trades that for reach, and the biggest turn is the lotus drag. Nothing in the kit is the
assassin's strike: a hit that is large *because of what came before it*. So the damage
pattern the class is meant to have — dart, set up, get in, explode, get out — has a dart and
a set-up and no explosion.

**There is no reason to leave.** With the same health as everyone and a copy at her shoulder
worth a quarter more on every swing, standing in melee is the correct play. A glass cannon
with nothing that pays for leaving is a cannon.

## The mechanic

### The shadow aims itself

**Out on the field, the shadow turns to face the nearest body inside the copied move's reach
before the copy comes out.** It keeps her swing — the pitch she committed to, the shape, the
frames — and throws it at whoever is there instead of at nobody. Attending at her heel it
keeps her yaw, as now, because at her heel it is copying her and not fighting on its own.

This is a decision about where an ability goes, so it lives in `aim.rs` as one function and
nowhere else. It is the tactical fix and it is the smallest thing in this document, but the
tally below is worthless without it.

### The tally

**Every hit the shadow lands from the field puts a mark on the target.** The copy of a swing,
a lotus blade on the way out, the flower on the way home, the recall's cut — each is one mark
per victim per event, up to a cap. Marks fade one at a time on a clock, so a target who gets
away from the shadow slowly cleans himself.

Marks are drawn **on the victim**, as pips both players can read. That is the counterplay
made visible: a fighter watching his own marks climb knows what is coming and where from.

The attending shadow marks nothing. It is at her shoulder; there is nothing being set up.

### The cash-in

**Arriving at the shadow by dash collects it and opens a window. Her first swing that lands
inside the window spends every mark on the target.**

- Damage is multiplied by one plus the marks times a knob, so a full tally is a large
  multiple of the swing that spends it.
- **At a full tally the cash-in also staggers.** That is the utility, and it is the reward
  for executing the whole pattern rather than a second branch of the kit: a hard stop behind
  a hard condition, which is the rule [ability-spec.md](ability-spec.md) sets for staggers.
  The condition here is that she set up from range, crossed under invulnerability, and hit
  inside the window.
- **Which swing is the player's choice.** Slash cashes fast and comparatively safe.
  Executioner cashes greedy: slower, punishable, and at a full tally the largest single hit
  in the game. The archive's "first auto after reclaiming the shadow deals bonus damage" is
  this, given something to be proportional to and a choice of what to spend it on.
- **Only the dash opens the window.** Recalling the shadow brings it home, and the leash
  brings it home, and neither is *her going there*. The assassin's verb is the crossing.

### Why she leaves

After the cash-in the target is clean and the shadow is at her shoulder marking nothing.
Staying in melee is her own swings plus a small copy, against a body with more health than
hers. Sending the shadow away and dashing to it is the exit, it is invulnerable, and it
starts the next tally. The rhythm falls out of the numbers: **send, mark, cross, cash,
send.**

### Glass

**A per-class health table**, the way jump, gravity and fall are already per class. The
Reaver at roughly three quarters of the Champion. The Bulwark above him. Weight is the most
legible difference a character can have and health is the second; a class whose whole
pattern is not being there should be the one that cannot afford to be.

**The attending copy's share is halved.** It is the one thing in the kit that argues for
standing still with the shadow at her heel, and with the tally in place it has to argue
less. Whether it goes entirely is a play question.

## What stays

Everything in [kits/shadow-reaver.md](kits/shadow-reaver.md): Send shadow on right click
with its recovery cancel and its remembered press, the lotus and the drag, the dash in three
dimensions with the carry and the jump out of it, Executioner on `E`. The four states of
the shadow. The leash. No new button and no new move.

Deadly mistake is still unbound and this document does not bind it.

## Why this shape

**The object pays a fourth time.** The shadow already pays movement (the dash), damage (the
copies, the lotus) and, thinly, utility (the recall's slow). The tally makes it pay *burst*,
and the burst is what the assassin fantasy is. Nothing was added to the field to get it: the
marks are a count of what the shadow already does, and the cash-in is a multiplier on a
swing she already has.

**Setup and payoff are the same object seen twice.** Where she put the shadow is where the
marks come from and where she will arrive. A player thinking about shadow placement is
already thinking about the entry, the tally and the exit, because they are one decision.

**The utility is the reward, not a fork.** A stagger that only a perfect pattern produces
gives the pattern something to be perfect *for*, without splitting the kit into a damage
route and a control route that compete for the same buttons.

## What it costs to build

- **`aim::shadow_faces`** — one function: given the shadow's position, the copied move's
  reach and the bodies in the scene, the yaw to throw the copy on. Called from
  `shadow::echo_body` in place of the eased facing when the shadow is out. `one_aim.rs`
  keeps its rule that nothing else in the simulation decides where a thing goes.
- **Marks on the victim.** `Player` gains a mark count and a fade clock. Every shadow damage
  site — the echo strike, the lotus's two passes, the recall's cut — adds one, once per
  victim per event. Two integers in the snapshot.
- **The window.** The dash's arrival already exists as the carry; the cash-in window is a
  frame count started on the same frame. Her hit sites read it: if open, and the victim has
  marks, multiply, clear, and at the cap stagger.
- **The health table.** A `Health` family in the Oven with one multiplier per class, read
  where `max_health` is today. The Bulwark proposal wants the same table; whichever lands
  first adds it.
- **Pips.** The HUD draws marks over the marked body; the overlay draws them as well.

**Tests that change.** `reaver.rs` gains: *a copy from the field turns to whoever is in
reach*; *every shadow hit marks once per event*; *marks fade*; *only the dash opens the
window*; *a cash-in at the cap staggers and clears*; *the attending copy marks nothing*.
`feel.rs` gains a relationship worth pinning for the whole roster: **a class's largest hit is
gated behind something the opponent could see coming** — for the Reaver, marks on his own
body. And `preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution` gets a sibling
for the cash-in: at the cap it should be the biggest hit in the game and still not a round
in one press.

## Open questions

- **The cap and the fade.** Five marks and a fade of a second or two each is a first guess.
  Too low a cap and the burst is a bonus; too high and the shadow never fills it before the
  victim walks away. The fade decides whether stalling cleans you, which it should.
- **Should a lotus fill the tally by itself?** Twelve blades out and twelve back is a lot of
  events. Marking once per pass keeps the flower to two marks; marking per blade caps it
  instantly and makes the lotus the whole setup. Written as once per pass.
- **How long is the window?** The carry is ten frames. A window the same length is a real
  execution test; one twice as long is a comfortable one. Play both.
- **Does the attending copy survive at all?** Halved here. If sticking in melee is still
  correct after the tally exists, it goes.
- **Should the creature take marks?** Yes in principle — the shadow works a Ridgeback's flank
  from range and she crosses to cash on a leg — but a body that big may need its own cap.
- **Deadly mistake** still has no button. The counter-stance is a natural fit for a class
  that now wants to be *near* the target only briefly, and it is still not answered here.
