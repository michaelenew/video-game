---
status: exploration — proposals to choose from, nothing decided
started: 2026-10-09
from: the first playtest
depends: ../kits/champion.md, ../controls.md
---

# 0007 — Every class gets the Champion's grid

## What the playtest said

The Champion is easily the most engaging class, and most of the reason is that its basic
kit holds together. Left, middle and right click are the whole class: a player can play
it, feel the fantasy, and hold their own in a fight with nothing but those three buttons
and the movement keys. On top of that, two rows of the grid give it several ways to move
through the air: the jump attacks (space and a click) and the aerials. Chaining them into
absurd jumps is the most fun anybody had.

The ask: **every class gets a basic kit on left, middle and right click that is the
essence of the class, and each of those three gets an aerial and a jump-attack version,
as the Champion's do.** They should be on theme for the class, and some of them should
change how the class moves. They may lean on the class's own mechanic. The
Elementalist's identity is combining field effects, so some of her air mobility can need
an effect in place, as long as she has a way (not necessarily an easy one) to make that
effect and use it at once. Reworks are welcome; the Elementalist's Updraft and Downdraft
are named as candidates.

This note has several proposals for every empty slot, a recommended set per class that is
meant to hold together the way the Champion's does, and the questions that need a person.

---

## 1 · Why the Champion's grid works

Four things, and every proposal below is checked against them.

1. **The button is the weapon, and the row is the situation.** Left is the sword on the
   floor, in the air, on the way off the floor and in a dash. A player learns three things
   and gets eighteen moves. Nothing has to be chosen from a list, because the situation
   chooses the move.
2. **The three takeoffs are three different reasons to leave the floor.** The rising cut
   is *hit them on the way up*. The uppercut is *take them with you*. The pole drive is
   *just get high*. A player who wants height and a player who wants a launch press
   different buttons.
3. **The air row has a cheap one, a committed one and one that pays you for landing it.**
   The air sword is fast and shoves you the way you are holding. The air hammer is slow and
   drives them into the floor. The air spear kicks you onwards **only if it connects**.
   That last kind is what makes the chaining possible: a hit buys more air, so a good
   player stays up and a sloppy one comes down.
4. **The rows connect into one loop.** Hammer, then uppercut, then space to go higher,
   then air hammer to spike them, then you land on top of them. The loop crosses three
   rows and you never need a key that is not a mouse button or space.

What makes the "absurd jumps" possible is the third point in particular. Call it a
**refuel**: an air move that gives back height or speed, and only gives it for something
the player did well (a hit, a precise aim, a setup made in advance). The game already has
a rule for this, accepted from the jump courses on 2026-10-03: *a movement tool's reach is
paid for in execution (timing, aim, precision), never in waiting, and never free.* Every
refuel below names what pays for it.

## 2 · The rules every proposal keeps

- **One identity per button, across every row.** If left click is the scythe on the floor,
  it is the scythe in the air and on the takeoff.
- **A takeoff is space and a click together, from the floor, once per trip** (the
  Champion's rule from 2026-10-05). In the air, holding space with a click is that trip's
  takeoff if it has not been spent. The one exception is the Dual mage, whose space press
  in the air is already her second jump (§5).
- **Each class's three takeoffs aim for the three reasons**: one hits going up, one takes
  them along or sets something up, one is height. Not every class fills all three the same
  way, but no class should have three takeoffs that are all height.
- **Each class gets at least one refuel, and it uses the class's own mechanic.** The
  Champion's refuel is a hit. The others are paid in what each class already spends:
  stones and fire, blood, the balance of two bars, the shadow, the shield's weight.
- **The existing air rules still hold.** An aerial holds gravity off for a set number of
  frames (its *hang*), and each hang in one trip is worth less than the last, so nobody
  can stay up forever by alternating moves. A hang is shorter than the move that carries
  it. A committed move gets no shove from the held direction. Every attack is punishable
  if it is blocked.
- **Every move points somewhere through `aim.rs`.** Where a proposal needs a kind of aim
  that does not exist yet, it says so (§8), and the change goes into `aim.rs`, never
  beside the ability.

---

## 3 · Elementalist

**Where she stands today.** On foot, left click is the Bolt (a beam), right click is
Cataclysm, and middle is Cinder spray. Since 2026-10-02 the Bolt and Cataclysm **pass
through people**, on purpose: they set off what she built (stones, fire) rather than
hitting a body. In the air she already has a row: Air bolt on left, Gale on right, and
Cinder on middle. Updraft and Downdraft are on `F`, Tremor is on `R`, and she has **no
takeoffs**. Her big movement is the stone jump: jump while a stone is still rising under
her, about 26 m, or 45 m off two stones raised a couple of frames apart.

**The gap this leaves, measured against the playtest's bar.** With only the three clicks
and movement, the one thing she has that hurts a person on foot is Cinder spray. The
2026-10-02 decision was right for her terrain game, so this note does not undo it. It
proposes instead that **her clicks-only game lives in the air**: a takeoff gets her up
quickly, and her air row hits people. On foot she builds, and in the air she shoots.

**The rework: the button is the element's verb.** Left is wind, middle is fire, right is
breaking. Updraft and Downdraft come off `F` and go into the grid, where the playtest
suggested they belong.

### Recommended grid

| | Left: wind | Middle: fire | Right: break |
| --- | --- | --- | --- |
| **On foot** | Bolt (as built) | Cinder spray (as built) | Cataclysm (as built) |
| **In the air** | Air bolt (as built) | **Flare dash** (new) | Gale, and **Downdraft** when aimed at the floor beneath her |
| **Leaving the floor** | **Updraft**, and **Thermal** if she is standing in fire | **Fire fountain** (new) | **Shatter leap**, off a stone (new) |

### The takeoffs

**Left: Updraft, with Thermal when she stands in fire.** *Recommended.* Updraft as built
(a gust under her that lifts her and anyone beside her) moves from `F` to space and left
click. The new part is effect-gated: if she takes off **inside her own fire** (a pillar's
base, an ember cloud, a burning stone's flames), it is a **Thermal**, hot air rising. It
lifts her roughly twice as high, and anyone it lifts beside her is lit. This is the "make
the effect, then use it" the playtest asked for:

> Cinder spray at her own feet (middle click on the floor beside her) → space and left
> click while the embers hang → Thermal.

That is two buttons, back to back, and the embers last two and a half seconds. It is not
hard to do, but it is a choice made in advance, and it leaves her standing in her own
fire for the moment between them.

*Alternative, Gust kick:* a rising Bolt that kicks any stone it meets **upwards** rather
than along. It sets up the "meteor" that a lofted stone already becomes, but it moves her
nowhere, which is why it is not the recommendation.

**Middle: Fire fountain.** *Recommended.* She leaves the floor on a short column of flame
that hits all around her. It is her **hit on the way up**, and the one takeoff that hurts a
person with no setup, which is what makes her clicks-only game exist. It gives little
height (about a short hop more than a jump) and leaves an ember cloud where she took off.
That cloud is fire for her Air bolts to fly through (a bolt that crosses fire is lit and
does more), and fire for her **next** Thermal when she lands back in it.

*Alternative, Blast:* the parked Blast idea from v2, used as a takeoff. A cone of fire
aimed at the floor pushes her up like a rocket and kicks any stone near her feet. The
v2 document calls it "the one most likely to become the class if its numbers are
generous", which is the risk. It does no damage up close, so her clicks-only game would
still be missing.

**Right: Shatter leap.** *Recommended.* Standing on or beside one of her stones, space and
right click breaks it under her: Cataclysm at her own feet. The stone bursts outwards as
debris (which hits people, as Cataclysm's debris does), and the burst throws her up and
forwards. A **lit** stone bursts as burning debris and an ember cloud, and throws her
further. The height sits between Updraft and a stone jump. A stone jump needs exact
timing on a stone that is still rising. This needs a stone that is already out and
**spends** it, which is the price: one of her three stones is gone.

To make it and use it at once: raise a stone beside her with `E`, step against it, then
space and right click. With no stone in reach, it is an ordinary jump and a Cataclysm
aimed at the crosshair.

*Alternative, Tremor:* move the built Tremor (a quake under her own feet that carries her
about 3 m and leaves a stone) from `R` to space and right click. This is the cautious
option. It costs no stone, it makes one, and it is already built. It is not the
recommendation because it does not use the stone she already has, and because a
telegraphed self-launch is a slow way to leave the floor.

### The aerials

**Left: Air bolt, as built.** The playtest said the projectiles are good as aerials, and
this keeps them. *An option to consider:* the Air bolt kicks her a little the opposite way
to where she aims it, so a bolt aimed straight down is a small extra lift. That would make
it a weak refuel she pays for with aim, and it competes with the Flare dash below, so it is
offered rather than recommended.

**Middle: Flare dash.** *Recommended over the built Cinder.* The cinder bursts at her hand
and the burst kicks her the way she is holding, about an airdodge's distance. It leaves a
line of hanging embers behind her. This is the "flare" the v2 document already talks about
(fleeing in the air, pop one behind you), made into movement. It is also a setup: turn
round and fire an Air bolt back through the line, and the bolt is lit. It is a refuel paid
for in setup. It does not give height, only distance, and it uses up the hang like any
other aerial.

*Alternative:* keep the built Cinder in the air (a cloud that hangs where it bursts).

**Right: Gale, and Downdraft when aimed at the floor beneath her.** *Recommended.* The Gale
starts small and grows as it travels, so it is weak up close. A Gale aimed at the floor a
few metres below her therefore has no use as a shot, and it can become the **Downdraft**
without stealing anything. This is the same kind of overload as the Champion's pole vault,
where a spear put into the floor vaults and a spear pointed at somebody stabs. The
Downdraft keeps what it does today: it drives her down, and on landing it makes a ring of
air, or a ring of fire if she lands in fire. The combination gets a second use, because
Fire fountain and Flare dash both leave fire on the floor to land in.

*Alternative:* the Downdraft becomes the Air bolt aimed straight down. It is rejected
because aiming an Air bolt down at somebody below is a normal thing to do, so that
overload would steal a real shot.

### What it plays like

> **Cinder** at her feet → **space and left click**: a Thermal lifts her and the fighter
> beside her, and he is burning → **Air bolt** at him, lit, on the way up → **Flare dash**
> away, leaving embers → **Gale aimed at the floor**: a Downdraft into the Fire fountain's
> embers → a ring of fire on landing.

Every step of that is one of the three mouse buttons or space, and two of the steps are
effects she made seconds earlier. The stone jump stays as it is, the class's biggest
movement, and it is still the one that needs exact timing.

**Freed keys.** `F` is empty now: Hover (the parked "hang in the air to line up a shot")
or Blast could go there. `R` keeps Tremor unless Tremor becomes the right-click takeoff.

---

## 4 · Blood mage

**Where she stands today.** Left is the Reaping sweep (the scythe, which grows with grey
health), middle is the Bloodletter (a blade thrown seven metres out that comes back), and
right is Haemorrhage (a bolt that makes the victim bleed). There is no air row and no
takeoff. In the air she throws her floor moves. Her only movement is the blink to a pool
of blood (a dodge with the crosshair on a pool) and the Grasp's haul.

**Her rules, which every proposal keeps.** Every move costs a share of her red health.
Pools spill only where she hits somebody **on the floor**. A hit on somebody in the air
spills nothing. A pool can heal her (by putting a move through it) or be a door (by
blinking to it), but not both. Her own kit document warned that vertical movement would be
"the second jump the one class with no movement has not earned". The answer here is that
she earns it the way she earns everything: **in blood**.

**What makes an aerial feel like a spell.** None of these is a weapon swung harder. Each
one is blood doing something blood should not do: rising, raining, pulling her along a
thread, holding somebody up in the air.

### Recommended grid

| | Left: the scythe | Middle: the Bloodletter | Right: Haemorrhage |
| --- | --- | --- | --- |
| **On foot** | Reaping sweep | Bloodletter | Haemorrhage |
| **In the air** | **Crescent** | **Blood thread** | **Red rain** |
| **Leaving the floor** | **Hook vault** | **Letting** (**Fountain** over a pool) | **Marionette** |

### The takeoffs

**Middle: Letting, and Fountain over a pool.** *Recommended, and her headline.* She draws
the Bloodletter across her own arm and the blood comes out of the floor under her as a jet
that throws her up. It costs a share of her red health, like everything she does. That
share is larger than a cast's, and the height is good: more than a full jump, less than a
stone jump. Anyone standing in the jet is cut.

**Standing on a pool, it is the Fountain.** The jet drinks the pool, and the pool becomes
height instead of health: a much bigger launch, and it costs her almost nothing. That is
"a pool is a heal or a door" with a third answer, a door upwards. The pool was made by an
earlier hit, so the setup is real, and spending the pool on height means it does not heal
her.

*Alternative, Ascent on the blade:* she throws the Bloodletter straight up and is pulled
up after it as it climbs. This is the same as the air Blood thread below, but vertical.
It is less on theme, because it does not spend her blood.

**Left: Hook vault.** *Recommended.* A rising scythe stroke that hooks whatever is in front
of her, a person, a stone or the lip of a wall, and swings her **over it**, landing behind.
Against a person it is a cross-up, an attack that lands from the side they were not
guarding. Against a wall it is a climb. It is the scythe used the way the Champion uses
the pole vault. If it whiffs, it is a rising arc and a short hop.

*Alternative, Harvest moon:* a full vertical circle swept up the front, which pops the
victim up. A plain launcher, simpler to build, and less of a scythe.

**Right: Marionette.** *Recommended.* Haemorrhage is fired as she rises. On a hit, the
victim's own blood hauls them up off the floor alongside her, as if they were a puppet on
strings, and they bleed. This is the *take them with you* takeoff, the counterpart of the
Champion's uppercut. A juggle (keeping somebody in the air with hit after hit) is the one
kind of play she has never had.

It runs straight into her rule that a hit in the air spills nothing, so a juggle would
starve her of pools. The proposal that goes with it: **a hit on somebody in the air keeps
its spill, and drops it where they land.** Her kit document already lists this as an open
question ("whether a pool should spill when the victim lands rather than not at all").
This note says yes, because the air game below needs it.

*Alternative, Arterial rise:* the bolt fired into the floor at her feet, and she rides the
spray up. It is too close to Letting, which is why it is not the recommendation.

### The aerials

**Left: Crescent.** *Recommended.* In the air, the scythe's arc goes out as a thin crescent
of blood that travels a few metres, its size set by her grey health like the sweep's. It
is a short spell, not a longer swing. It shoves her the way she is holding (it is the
cheap aerial), and the scythe drinks any pool the crescent passes over, as the sweep does.

*Alternative, Falling reap:* the sweep rolled into the vertical, cutting down across the
target in front of her, as the Champion's air sword does. It is more of a weapon and less
of a spell.

**Middle: Blood thread.** *Recommended, and her refuel.* In the air she throws the
Bloodletter. While it is still flying out, pressing middle click again **pulls her along a
thread of blood to it**. This is the top-ranked unbuilt movement idea in her own kit
document ("ride the Bloodletter home"). It costs the throw's health and the precision of a
second press. She gets distance in whatever direction she aimed, which can be upwards.
Aimed at a person, she arrives on top of them as the blade turns for home. That pass
drinks pools on the way back, as the blade does today.

*Alternative, Orbit:* the blade circles her instead of flying out, slowing her fall and
cutting anyone close. It is defensive and floaty, and it does not move her.

**Right: Red rain.** *Recommended.* Fired from the air, Haemorrhage bursts over the spot on
the floor under the crosshair and **rains**. Everyone under the rain bleeds, and bleeding
is what spills pools. It is her way of making pools from the air, and it reads as the most
spell-like thing she does. It is also where the Fountain's next pool comes from.

*Alternative, Red comet:* she becomes the spell. She dives along the aim as a streak of
blood and bursts on the floor. This is her plunge, the counterpart of the air hammer, and
it would be a strong second choice if Red rain does not read well.

### What it plays like

> **Sweep** someone into a pool → stand on it, **space and middle click**: a Fountain
> throws her high → **Red rain** over him while she is up there, so the floor around him
> fills with pools → **Blood thread** to the edge of the rain → **blink** down into the
> freshest pool.

Every movement in that chain is paid in blood, either hers or somebody else's. That is the
class.

---

## 5 · Dual mage

**Where she stands today.** Left is the dark auto (it pulls the victim and steps her in),
right is the light auto (it shoves the victim and steps her back), and middle is the Lance,
whose form depends on which force she is carrying: light bursts at the end of the line,
dark tethers whatever it crosses. Her two bars, Dark and Light, unlock things as the lower
of the two rises: a blink at half, a second jump and a slow fall at three quarters, and the
wings (ascension) near the top. Her jump is already the highest in the game. In the air she
throws her floor moves, which hang the longest of any class. She has no takeoffs.

**The rework: on the floor she moves them, and in the air she moves herself.** On the floor
she is planted, so a pull drags them in and a push shoves them away. In the air nothing is
holding her, and she is the lighter body, so **the same force moves her instead.** This is
the same rule the Champion's grid follows (the button is the force and the row is the
situation), and it is physics a player already knows: push off a wall while floating and
it is you that moves.

### Recommended grid

| | Left: dark (pull) | Middle: the Lance | Right: light (push) |
| --- | --- | --- | --- |
| **On foot** | Dark auto | Lance, by carried form | Light auto |
| **In the air** | **Dark wing**: a hit pulls *her* to them | Light: **Recoil**. Dark: **Reel** | **Light wing**: a hit pushes *her* off them |
| **Leaving the floor** | **Undertow** | Light: **Sunspear**. Dark: **Anchor** | **Daybreak** |

### The takeoffs

**Left: Undertow.** *Recommended.* A rising dark punch. On a hit, the pull drags the victim
up into the air with her: *take them with you*, the counterpart of the uppercut. It feeds
her dark bar like any auto.

*Alternative, Eclipse rise:* a pull aimed at the floor, so the jump is lower and faster,
and anyone near her is dragged in under her. A defensive "come here" on the way up.

**Right: Daybreak.** *Recommended.* A rising light palm that pops the victim **up and
away**. This is the *hit on the way up*. The light push keeps them out of reach of their
own answer, and it sets up the light wing (below), which needs something to push off.

*Alternative, Sunburst:* the push goes into the floor instead, so she gets a high jump and
everyone round her is shoved out. This is the escape takeoff, and it fits if the
recommended set turns out to have too little height.

**Middle: the Lance from the floor, by carried form.** *Recommended.* Light, the
**Sunspear**: she fires the Lance into the floor at her feet as she leaves it, and the burst
at the end of the line throws her up. Height, and the burst hurts anyone round her
takeoff. Dark, the **Anchor**: she fires the Lance up at the crosshair. If it tethers
something (a person, a creature, a stone, a wall), she is reeled up to it. That is a
grappling hook, and it goes wherever she aimed.

How strong both of these are follows her carried bar, as her power already does. A
mage who has kept her bars in balance jumps further, so the height is paid for by how she
has played.

### The aerials

**Left and right: the wings.** *Recommended.* Same punches, same frames, same hang. The
difference is who moves:

- **Dark wing.** On a hit, she is pulled to the victim, not the other way round. Chained
  dark hits close the gap on somebody in the air.
- **Light wing.** On a hit, she is pushed off the victim. **It also pushes off solid
  ground**: a wall, one of the Elementalist's stones, the Bulwark's planted shield. So it
  works as a wall jump.

That makes her refuel a **hit or a wall**, paid for by aim. The longest hangs in the game
are already hers, and the rule that each hang in a trip is worth less than the last
already stops this becoming a hover.

*Alternative, Plummet and Lift:* the dark auto in the air pulls her down (a fast fall) and
the light auto lifts her a little. This is simpler, and it does not need a hit, which is
why it is weaker as design. *Second alternative:* keep the autos as built.

**Middle: the Lance in the air, by carried form.** *Recommended.* Light, **Recoil**: firing
it pushes her backwards along the line, away from where she aimed. Aimed down, that is a
lift. Dark, **Reel**: if it tethers something, she is pulled to it instead of pulling it.
On a person it is a dive onto them. On a stone or a wall it is a grappling hook in the air.

### Space in the air: the one exception to the takeoff rule

Her space press in the air is already the second jump (when the lower bar holds three
quarters) and, while ascended, a wing beat. So **"hold space with a click in the air" is
not her takeoff.** Instead: *a click on the same press as her second jump is a second
takeoff.* A Dual mage who has earned the second jump has earned a second launch with it.
While ascended, every wing beat with a click is a takeoff, and the falling-off rule for
hangs is what keeps that from being infinite. Whether that is too much is a question for
play (CORE-5).

### What it plays like

> **Dark, light, dark** on the floor to feed both bars → **space and left click**: the
> Undertow takes him up with her → **dark wing** to stay on him → **light wing** off him
> and into the stone behind, pushing off it → **dark Lance** reels her back in → down
> together.

---

## 6 · Shadow Reaver

**Where she stands today.** Left is Slash and right is the shadow (send it, call it home,
drag the lotus back). **Middle click is empty.** The Executioner, her heavy overhead cut,
is on `E`. Deadly mistake, her counter stance, has **no input at all**, and her documents
call that an open problem. In the air she throws her floor moves. Her movement is the
best in the game: a dash to the shadow, and a jump out of the end of that dash that
carries about 50 m.

**The rework: the Executioner moves to middle click, and Deadly mistake gets `E`.** Left is
the quick cut, middle is the heavy cut, and right is the other body. That is the
Champion's sword and hammer with a shadow beside them, and it gives her counter stance a
key at last.

### Recommended grid

| | Left: Slash | Middle: Executioner | Right: the shadow |
| --- | --- | --- | --- |
| **On foot** | Slash | Executioner (moved from `E`) | Send / recall |
| **In the air** | **Kite cut** | **Guillotine drop** | **Swap** |
| **Leaving the floor** | **Moonsault** | **Gallows** | **Hang the shadow** |

### The takeoffs

**Middle: Gallows.** *Recommended.* Her earliest source material describes the
Executioner as "blink upward, then slash down". It was never built that way. As a
takeoff, that is exactly what it becomes: she vanishes upwards a few metres, hangs for a
beat, and the overhead comes down. It is a takeoff that is also an attack from above, and
reading which way she will come down is the defender's problem. It is the *hit* takeoff,
the hit that lands on the way back down.

*Alternative, Rising execution:* an ordinary rising cut that launches. Simpler, and it
throws away the source material.

**Left: Moonsault.** *Recommended.* A back flip that cuts upwards in front of her as she
goes up and back. It is an escape that punishes whoever was pressing her. Every swing she
throws, her shadow throws too, a moment later, so a shadow waiting out in the field does
its own Moonsault there. That turns it into an anti-air (an attack that catches somebody
jumping) from wherever the shadow is.

*Alternative, Rising cut:* the same cut going up and forwards. It is a plain launcher.

**Right: Hang the shadow.** *Recommended, and her big one.* Space and right click sends the
shadow to the crosshair's point **in the air** instead of on the floor, and it waits there
for a moment before falling. Her dash to the shadow then goes upwards, and the jump at the
end of the dash leaves from a point in the sky. It turns her biggest movement into a
vertical one, and it is paid in aim: she has to place the shadow, and then dash and press
jump in the last six frames, as she already does.

This is also the biggest balance risk in this note. A jump out of the dash keeps all of the
dash's speed. Leaving from the sky could carry her across most of a reach. The knob is how
much of the dash's speed a jump keeps when it leaves from a hung shadow.

*Alternative, Shadow step:* the shadow goes under her feet and she jumps off it, a higher
jump that leaves the shadow behind at her takeoff spot. It is cheaper and safer, and her
next dash goes back to where she started.

### The aerials

**Left: Kite cut.** *Recommended.* The Slash, rolled into a vertical arc. On a hit against a
**marked** target (one her shadow has already cut), it **gives her airdodge back**, and her
airdodge, aimed at the shadow, is the dash. So hitting a marked enemy in the air lets her
dash again. That is a refuel paid for with her own loop: send, mark, cut.

*Alternative:* a plain vertical cut that shoves her the way she is holding, like the air
sword.

**Middle: Guillotine drop.** *Recommended.* The Executioner, falling. She drops straight
down blade first. Against somebody already in the air, it drives them into the floor (a
spike, as the air hammer does). It is the committed aerial and the end of the trip. Thrown
out of Gallows, it is the second half of the source material's "up, then down".

*Alternative, Drop to the shadow:* the plunge goes along the line to wherever her shadow
stands, not straight down. That makes it a diagonal dive she aims by placing the shadow.

**Right: Swap.** *Recommended.* In the air, with the shadow out and waiting, right click
does not call it home. **She and the shadow trade places**: she is on the floor where it
was, and it is in the air where she was. Anyone who jumped to catch her is now swinging at
a shadow. It is movement, a feint and a question every opponent has to keep asking: which
one is her?

*Alternative:* keep the built send and recall in the air.

### What it plays like

> **Send** the shadow beside him → **Slash**, so the shadow marks him with its copy →
> **space and middle click**: Gallows, up and down onto him → he jumps out of it → **Kite
> cut** on the marked target, and her airdodge comes back → **dash** to the shadow → **Swap**
> out as he lands.

---

## 7 · Bulwark

**Where he stands today.** Left is Bash, middle is Slam, and right (held) is Guard, whose
first four frames are a parry. The shield stores the weight of what it blocks, and Slam
and the throw spend it. His jump is the lowest in the game, 2.7 m. In the air, Slam waits
for the floor and does more damage the faster he lands. That is already an aerial, and a
good one. He has no takeoffs. **Every move he has needs the shield in his hand.**

**What he wants from the air is different.** He is the heaviest class and should stay the
heaviest. His air game should be short and violent, about **being thrown**, not floating:
the shield as a springboard, a sail and a battery.

### Recommended grid

| | Left: Bash | Middle: Slam | Right: Guard |
| --- | --- | --- | --- |
| **On foot** | Bash | Slam | Guard / parry |
| **In the air** | **Rebound** | Slam (as built) | **Sail** |
| **Leaving the floor** | **Battering ram** | **Unload** | **Shield step** |

### The takeoffs

**Middle: Unload.** *Recommended, and his headline.* He drives the shield into the floor at
his feet and **the stored weight throws him upwards**, and the more weight it holds, the
higher he goes. The battery gives back what it took. It sets up a real choice every time
the shield is loaded: spend the weight on Slam damage now, or on height now and on the
speed Slam does damage with later. With an empty shield, it is a stomp and barely a hop.

*Alternative, Quake hop:* a stomp that knocks down everyone round him, for almost no
height. A pure attack, and he stays heavy.

**Left: Battering ram.** *Recommended.* A low, flat leap forward, shield first, three or
four metres. On a hit, he carries the victim along with him. His jump is short, so his
takeoff should go **along** the ground, not up. It is the *take them with you* takeoff,
pointed the way he wants to push them.

*Alternative, Rising bash:* the shield punched upwards at somebody above him, a plain
anti-air.

**Right: Shield step.** *Recommended.* He plants the shield on the floor in front of him and
springs off its top. The shield's height depends on its weight, so the jump does too.
The shield stays behind as a wall, and while it is planted he has no moves. That is the
price, and the answer is `E`: called home while he is in the air, it flies back to his hand
and hits whoever it passes on the way. Plant, step, recall, Slam is a loop, and each step
of it is a decision.

*Alternative, Rising guard:* he jumps with the guard raised. It is a safe way to go up over
somebody, and it is the obvious home for the ally-cover stance his documents ask about: a
guard held over a teammate below.

### The aerials

**Left: Rebound.** *Recommended, and his refuel.* A Bash in the air that meets **anything
solid** (a body, a wall, a stone, his own planted shield) throws him back off it. It is his
wall jump, and his only way to stay up. It is paid for by being next to something: a
heavy man in the air bounces off things rather than floating over them.

*Alternative:* a plain air Bash that shoves him the way he is holding.

**Middle: Slam, as built.** It is the class's air hammer already, and it gets bigger with
both of the takeoffs above: Unload and Shield step put him high enough for the fall to
count.

**Right: Sail.** *Recommended.* Guard held in the air turns the shield into a sail. He falls
slower and steers better, and **a hit he blocks in the air carries him with it**: he rides
the blow and stores its weight, as any block does. An Elementalist's Gale or a light wing
becomes something he can surf on. It is defensive, and it is movement that his opponent
pays for.

*Alternative, Umbrella:* the guard angled down and held over an ally below. This is the
ally-cover stance, in the air.

### What it plays like

> **Guard** through a string and load the shield → **space and middle click**: Unload, and
> he goes up three times his jump → **Sail** across the gap, riding whatever they throw at
> him → **Rebound** off the stone behind them → **Slam** from height onto the one who
> threw it.

---

## 8 · What this needs in the code

Not built, not costed. Listed so the choices above can be weighed against it.

1. **The takeoff window has to belong to every class.** Today it lives inside the
   Champion's mechanic (`Mechanic::Forms { takeoff }`, `state::arm_takeoff`), as does the
   rule about pressing space within a few frames of a click. It would move onto the
   player, with the window's knobs in the Oven shared by every class or set per class.
2. **Each class's move choice has to know where its feet are.** The Elementalist's already
   does. The Blood mage's, Dual mage's, Reaver's and Bulwark's choose the same move on the
   floor and off it.
3. **New kinds of aim, which belong in `aim.rs`:**
   - the Reaver's hung shadow, which is sent to a point in the air, not to the floor;
   - Drop to the shadow, if chosen, which aims at the mechanic, a kind of aim only the
     lotus uses today;
   - the Gale aimed at the floor beneath her, which needs a test for "the crosshair meets
     the floor this close below me" (a cousin of the pole vault's test);
   - the Dual mage's Reel and Anchor, and the Blood mage's Hook vault, which need the
     first solid thing along a line, not just the first body.
4. **Pulling a body along a line.** The Blood thread, the Reel and the Anchor all pull the
   caster to a point. The Grasp's haul to a wall already does this, so it is a precedent
   rather than a new primitive.
5. **A hit in the air keeps its spill** (Blood mage), which is a rule change to her
   pools, and every essence test has to agree with it.
6. **The rows of tests that hold the Champion's grid up need versions for every class.**
   Each class's takeoffs and aerials must be punishable on block, have hangs shorter than
   the move, and give no shove on committed moves. A new property: *each refuel only pays
   on the thing that pays for it* (a hit, a stone, a pool, weight).
7. **Clips.** Every new move needs an authored animation. The recommended grids hold about
   thirty new moves, counting the Dual mage's two-form Lance as two. Three slots reuse a
   move that exists: Updraft, Downdraft and the Executioner, plus Tremor if it is chosen.

**A suggested order.** Item 1 first, because everything depends on it. Then the
**Elementalist**: three of her six slots are moves she already has, re-bound, so it is the
cheapest whole grid and it answers the playtest's own suggestion. Then the **Reaver**: the
Executioner move is a rebinding, and Gallows and Guillotine drop are close to existing
moves. Then the **Dual mage**: the wings are a change of who moves, not new moves. Then
the **Bulwark**. The **Blood mage** comes last, because she needs the most new machinery
(the pool rule, the thread, the hook).

---

## 9 · Questions for a person

Answer by ID.

| ID | Question | This note's lean |
| --- | --- | --- |
| **CORE-1** | Is "the button is the identity, the row is the situation, and space and a click is a takeoff" now the rule for every class? `controls.md` lists airborne attacks and space as a modifier as open questions; this would settle both. | Yes |
| **CORE-2** | Reaver: the Executioner moves to middle click and Deadly mistake goes to `E`? | Yes |
| **CORE-3** | Elementalist: Updraft becomes space and left click, Downdraft becomes the Gale aimed at the floor, and `F` is freed. On space and right click: Shatter leap (spends a stone) or Tremor (built, makes a stone)? | Re-bind both; Shatter leap |
| **CORE-4** | Elementalist: is it acceptable that her clicks-only game is in the air (takeoffs and the air row), since her standing clicks pass through people on purpose? | Yes, with Fire fountain as the takeoff that hurts |
| **CORE-5** | Dual mage: the rule that in the air she moves herself, and a second takeoff on her second jump. Is a takeoff on every wing beat while ascended too much? | Yes and yes; play the wing beats |
| **CORE-6** | Blood mage: a hit on somebody in the air keeps its spill and drops it where they land? Without it, Marionette and the juggle starve her. | Yes |
| **CORE-7** | Bulwark: Shield step leaves him with no moves until he calls the shield home. Is that a price, or a trap? | A price; play it |
| **CORE-8** | Reaver: should a jump out of a dash to a hung shadow keep all of the dash's speed? | Less; play to find out how much less |
| **CORE-9** | Every class gets at least one refuel. Does the Champion still feel like the class that owns the air, or does the roster need one class that clearly does? | It should still be his: his refuels are the cheapest |

## 10 · What this note does not touch

- The Champion. His grid is the blueprint and is unchanged.
- The Rush row, which has no counterpart on any other class and does not need one. A
  class's mechanic key (`E`) is that class's own row.
- `Q` on every class.
- Numbers. Every height and distance above is a comparison ("more than a full jump, less
  than a stone jump"), because no number survives the first play of a new move. Each move
  gets its knobs in the Oven when it is built.
