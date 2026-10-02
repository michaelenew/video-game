---
status: the review guide for the bestiary as built
written: 2026-10-01
---

# Reviewing the bestiary — a guide

Ten creatures and the Ridgeback, their arenas, and a rudimentary world (an
arena picker, trophies, tempered rematches) were built while you were away.
This page is the way through all of it at once: how to start each fight, what
it is, where the harness says it landed, what to try first, and **every
question that is waiting for a person**, each with an ID so you can answer it
by ID ("GNAW-1: three tokens", "HORN-C1: yes").

- **IDs ending in a number** (`GNAW-3`) are open questions.
- **IDs with a `C`** (`GNAW-C2`) are **decisions the builders made on your
  behalf** — a health changed from the spec, a rule changed, a test re-pinned
  — collected so you can confirm or overrule them. The full "please confirm"
  list is §5.

The pictures are in the [gallery](gallery/README.md): every creature in its
arena, and one telegraph each. The cast-wide table every number here comes
from is [bestiary.md](bestiary.md) §8; each creature's own document ends with
§12 (its open questions) and §13 (where it landed), which this page condenses
and links rather than replaces.

---

## 1 · How to play any fight

**Desktop**

```
cargo run --release -p game -- --hunt <creature>             # that creature, in its arena
cargo run --release -p game -- --hunt <creature> --p1 <class> # as a class
cargo run --release -p game -- --hunt <creature> --temper 3   # tempered, earned or not
cargo run --release -p game -- --hunt hornback-escort         # the Hornback's crossing
./scripts/dev.sh --hunt <creature>                            # with the dev tools
```

**Browser** (the Pages build, or `./crates/web/build-game.sh` and serve
`target/web`): the query string does what the flags do —
`?hunt=<creature>`, `?hunt=<creature>&p1=<class>`, `?hunt=<creature>&temper=3`,
`?hunt=hornback-escort`, and `&dev` for the dev tools.

`<creature>` is one of `ridgeback`, `gnawers`, `hornback`, `hornback-escort`,
`mireback`, `sandmaw`, `pair`, `broodmother`, `veilstalker`, `mantis`,
`galewing`, `siegeshell`. `<class>` is `champion`, `bulwark`, `reaver`,
`elementalist`, `blood` (Blood mage) or `dual` (Dual mage).

**Keys in a fight**

| Key | Does |
| --- | --- |
| mouse, `WASD`, `space`, `shift` | aim, move (camera-relative), jump, dodge |
| `L` / `M` / `R` click, `Q`, `E` | the class's moves; `E` is the class mechanic |
| `H` | hunt the Ridgeback / back to versus |
| `Shift+H` | the next creature, in its own arena — all eleven, round and back |
| `T` | the same creature at the next temper you have earned (`--temper` offers all four) |
| `Tab` | cycle your class (restarts the fight) |
| `Backspace` | restart |
| `F1` | hitbox, hurtbox and senses overlay |
| `F7` | the Oven: every tuned number, live |
| `P`, `]`, `[` | pause, step forward, step back |

The list at the right of the HUD is the picker: every creature, the trophies
you hold for it, the tempers it offers, and a `>` at the one being hunted.
Beat a creature and its trophy is written at that temper; `T` then offers the
next. Trophies live in `~/.config/arena/trophies.conf` (desktop) or the
browser's local storage.

**The dev species** — the gnats (`--hunt gnats`, the critter machinery with
nothing of its own) and the sentinel (`--hunt sentinel`, one of every hazard
and sense) — are reachable by name and kept out of `Shift+H` and the trophy
list (§6 of this page, `REV-C1`).

**The harness, if you want the numbers yourself:**
`cargo run --release -p hunt --bin fight -- --species <x> --class <c> --repeats 24`
(`--hunters 2` for a pair, `--gamble` for a creature's second plan,
`--arena crossing` for the Hornback's crossing; the Broodmother is
`--bin brood -- --all --balanced`).

---

## 2 · The cast at a glance

Tier is the bestiary's target for a decent player alone (1 ≈ nine in ten won,
5 ≈ one in twenty). **Won** is the scripted hunter, 24 hunts solo, from
bestiary §8: the Champion (the class the plans were written for), then the
best and worst class. **Unans.** is unanswerable hits per class (the contract
says zero; zero everywhere since the polish pass, §7).

| Creature | Arena | Tier | Champion won | Best / worst class | Unans. | Status |
| --- | --- | --- | --- | --- | --- | --- |
| [Ridgeback](monsters.md) | Proving ground | 3 | 14/24, 64 s | Champion 14 / Blood, Dual 0 (Bulwark 1) | 0 | Built; the parity bar |
| [Gnawers](creatures/gnawers.md) | Commons | 1 | 24/24, 35 s | everyone 20+ / Dual 20 | 0 | Built; easy |
| [Hornback](creatures/hornback.md) | Low meadow | 1 | 22/24, 109 s | Elementalist 24 in 36 s / Dual 14 | 0 | Built |
| [Hornback, crossing](creatures/hornback.md) | Crossing | 1 | 11/12, 56 s | Champion, Elem. 11 / Blood 4 | 0 | Built; two hunters lose it |
| [Mireback](creatures/mireback.md) | Mire | 2 | 18/24, 147 s | Elementalist 24 in 41 s / Dual 0 | 0 | Built |
| [Sandmaw](creatures/sandmaw.md) | Pan | 2 | 18/24, 191 s | Bulwark 22 / Blood 0 | 0 | Built |
| [The Pair](creatures/the-pair.md) | Den | 3 | 16/24, 159 s | Elementalist 24 in 46 s / Blood 0 | 0 | Built |
| [Broodmother](creatures/broodmother.md) | Hollows | 3 | 5/24, 147 s | Dual 10 / Reaver, Blood 0 | 0 | Built; short fights |
| [Veilstalker](creatures/veilstalker.md) | Ashwood | 4 | 6/24, 336 s | Elementalist 22 / Blood, Dual 0 | 0 | Built |
| [Mantis](creatures/mantis.md) | Shrine | 5 | 0/24 | Elementalist 24 in 71 s / four classes 0–2 | 0 | Built |
| [Galewing](creatures/galewing.md) | Cliffs | 4 | 4/24 (plan A), 7/24 (ride) | Elementalist 17–18 / Blood, Dual 0 | 0 | Built |
| [Siegeshell](creatures/siegeshell.md) | Last Valley | 5 (for two) | 1/24 alone, 4/12 pair | Reaver pair 12/12 / Bulwark, Blood 0 | 0 | Built |

The Bulwark's column moved on 2026-10-01 when the hunter learned his shield
throw, and again when the shield began to strike creatures (`CLASS-5`); his
row per creature is in §3 and in bestiary §8.

**Four things are true across the whole cast**, and each is a question in §4:
the Elementalist's fire pillar wins most fights from nine metres (`CLASS-1`);
the Blood mage wins almost nothing against a creature (`CLASS-2`); the Dual
mage lives or dies on how often windows come (`CLASS-3`). (A fourth, that the
thrown shield touched no creature, is resolved: `CLASS-5`.)

---

## 3 · Each fight

### The Ridgeback — the parity bar

*You can't reach what kills it from the ground, and you can't leave the ground
without earning it: break a foot, climb, ride the buck, topple it.* 13 m long,
5.5 m at the back, eight moves. [monsters.md](monsters.md) §9.

- **Play:** `--hunt` (or `--hunt ridgeback`), `?hunt`. The proving ground.
- **Harness:** Champion 14/24 in 64 s, Reaver 12, Elementalist 6, Bulwark 1;
  Blood mage, Dual mage 0. Zero unanswerable for every class.
- **Try first:** poke a foot, then *watch* — unload only into a recovery you
  can see is long, and stand beside the hind leg rather than behind it. Then
  climb on a stumble and ride the buck.

**Open**
- `RIDGE-1` Is it too lethal? A hunt is over in about a minute, won or lost
  (levers: `think_frames`, `combo_appetite`, the bite's damage).
- `RIDGE-2` Is the windup's tracking (nine tenths; nobody walks out of a tell)
  an animal following you or a homing missile? (`startup_tracking`)
- `RIDGE-3` It often opens with the spray after the 3 s grace — is three
  seconds enough to get your bearings? (`hunt_grace`)
- `RIDGE-4` The Dual mage's float clears the tail from the floor and declines
  the ground game: identity or hole?
- `RIDGE-5` The Bulwark's only routes up are the two lowest (stumbling
  shoulder, toppled barrel). Is that the class's answer here?
- `RIDGE-6` Is the kick's 20-frame tell long enough for a person to find the
  spot it does not reach?
- `RIDGE-7` Is one win in twelve for a decent player a floor or a wall — and
  is the next knob the damage (a quarter of a fighter a hit) or the tells?
- `RIDGE-8` The buck's damage (45 a throw): does a good rider barely feel it?
- `RIDGE-9` The nape (2.4×, on the most violent part of the shake): does
  anybody choose it over the ridge?
- `RIDGE-10` The Blood mage breaks no foot and the Dual mage's punch misses the
  ridge from where the plan works it: the creature's question or the class's?
- `RIDGE-11` Coop: numbers are set for one. Tune a second Ridgeback for two?
- `RIDGE-12` (A4) Does a swing on its back that follows the back through the
  shake feel like aiming, or like the animal moving your sword?

### The Gnawers — tier 1, the pack

*Nothing in the pack can hurt you much; the pack can.* Six knee-high biters
and the Big One. Keep them in front of you, count them, kill the leader and
the rest break. [creatures/gnawers.md](creatures/gnawers.md).

- **Play:** `--hunt gnawers`, `?hunt=gnawers`. The Commons.
- **Harness:** every class 20–24 of 24; Elementalist in 24 s, Champion 35 s,
  Dual mage 87 s. Zero unanswerable.
- **Try first:** watch for a tail going up and a gnawer coming in — the
  28-frame crouch after it is the swing's cue. Then go for the Big One in a
  scatter; see whether the rout at the den reads as cornered animals.

**Decided for you**
- `GNAW-C1` The crouch tell is **28 frames**, not 18 (and not 24: a coin toss
  for the Champion's sword).
- `GNAW-C2` Bodies are tougher than proposed (gnawer 160, Big One 500); the
  token rest is 60 frames — the leader-dead rout ended most hunts in 20 s.
- `GNAW-C3` A rout **cornered at the den turns and fights** (4 m); the
  pile-on leaps at one spot in sequence; a six-metre hedge rings the Commons.
- `GNAW-C4` Two moves added beyond the seed: the maul (answered by a jump)
  and the gnaw on a stone's top. Darts only from the front arc (±37°).

**Open**
- `GNAW-1` **Is it too easy?** The hunter keeps most of its health (levers:
  `FrontArc`, the token count, the crouch's 28 frames).
- `GNAW-2` Two tokens: fair or polite? Would three with a longer rest feel
  more like animals?
- `GNAW-3` Is the leader-dead rout too cheap a win?
- `GNAW-4` The Dual mage treed (a pack waiting under her): identity or
  stalemate?
- `GNAW-5` Is 28 frames too readable for a pack (with two at once; with a
  howled three)?
- `GNAW-6` Can fighters walk through gnawers? Ghosts, or right (seven bodies
  cannot wall you in)?
- `GNAW-7` Corpses stay until their slot is needed: the pack count you read,
  or clutter?
- `GNAW-8` Should a crouching fighter be "short" to the aim model
  (`stands_at`), so spears dip for a crouch? (A versus change.)
- `GNAW-9` Does the tail coming in read? It is a warning the design did not
  have.
- `GNAW-10` The hedge makes the Commons a box from some angles — acceptable?
- `GNAW-11` Is the Elementalist's 24 s with nine tenths of her health the
  fight working? Her bolts reach the ring from outside it.

### The Hornback herd — tier 1, the arena as weapon

*The herd is the terrain; the bull is the fight.* Eight cows and a bull. Bait
a charge into something solid; get out of a stampede's lane, or ride it.
[creatures/hornback.md](creatures/hornback.md).

- **Play:** `--hunt hornback`, `?hunt=hornback` (the low meadow);
  `--hunt hornback-escort`, `?hunt=hornback-escort` (the crossing: walk a cart
  along a road the herd migrates over).
- **Harness:** Champion 22/24 in 109 s, Elementalist 24 in 36 s, Blood mage
  21, Bulwark 20, Reaver 15, Dual mage 14. Crossing: 4–11 of 12 by class; two
  Champions 1 of 12. Four classes take one unanswerable hit in 24.
- **Try first:** stand a boulder behind you, wait for the paws, step out of
  the lane on the head-drop, and hit the stunned bull. Then try riding a cow.

**Decided for you**
- `HORN-C1` **Bull health 2000**, not 3500 (at 3500 nobody finished inside
  two minutes). Horns stay 500.
- `HORN-C2` The arena's edge does **not** stun (a charge pulls up short); the
  bank does. The bull interposes 10 m out and charges from 9–15 m.
- `HORN-C3` The bull's guard turns swings, not projectiles.
- `HORN-C4` Cows push only fighters on the floor; a cow's back is mountable.
- `HORN-C5` The crossing: a 1500-health cart at 2 m/s; killing the bull also
  wins it. **The carter's key trophy is not built** — a crossing win writes
  the Hornback's ordinary trophy.

**Open**
- `HORN-1` Should the arena's edge stun? (Built: no; the lever is more
  boulders.)
- `HORN-2` Can the Elementalist raise a stone under the bull (refused, or a
  small topple as her reward)?
- `HORN-3` Is the charge too easy to walk out of (4.7 m of walk against a 2 m
  lane)?
- `HORN-4` Should driving cows off cost anything (a thinner herd next time)?
- `HORN-5` Should the bull be rideable (it would become a light monster)?
- `HORN-6` Does the rally below a quarter make the ending a squeeze or a wall?
- `HORN-7` Is a declinable fight right for a tier-1 teacher?
- `HORN-8` Two seconds of cow ride (`ride_patience` 120 f, ~20 m): trick or
  toy?
- `HORN-9` Two hunters lose the crossing (1 of 12): a plan question first —
  then, should a hunter standing in a lane count as escorting?
- `HORN-10` The Elementalist's 36 s with nine tenths of her health is
  trivial: identity or hole? (See `CLASS-1`.)

### The Mireback — tier 2, the floor

*It does not chase you; it takes the floor away.* A 7 m toad that spreads tar.
Burn the tar — which clears the floor and builds the slag steps up it.
[creatures/mireback.md](creatures/mireback.md).

- **Play:** `--hunt mireback`, `?hunt=mireback`. The Mire.
- **Harness:** Champion 21/24 in 120 s, Reaver 22, Elementalist 24 in 41 s,
  Bulwark 18, Blood mage 5, Dual mage 0. Unanswerable 0–3 by class (each a
  flop out of tar laid after it committed).
- **Try first:** kick a brazier over onto the tar, and watch the flop's ring
  of warnings — does it read before it lands? Then climb the slag to burst a
  wart.

**Decided for you**
- `MIRE-C1` **Health 10 000**, from 11 000.
- `MIRE-C2` Swallow and Gag are never chosen by the brain (the tongue starts
  one); gutted is the stock topple, winded the stock stumble.
- `MIRE-C3` It burns by its **footprint** (up to three pools), not the shared
  rule's centre; fire spreads after its `Spread` frames.
- `MIRE-C4` The flop shoves bodies out sideways and a downward face is no
  surface — opt-in, so the Ridgeback's pins hold (`MIRE-11`).
- `MIRE-C5` Not built: the coat's sheen, a swollen sac, the stomach camera.

**Open**
- `MIRE-1` Is a 40 % walk in tar mud or glue?
- `MIRE-2` Choosing to be swallowed (`--gamble`): a clever read or a cheese?
  (Keep it choosable.)
- `MIRE-3` Does the Elementalist make it tier 1? (She wins every hunt in under
  a minute — `flee` could favour her fire.)
- `MIRE-4` The Dual mage has no fire and no floor (0 of 24; her punch passes
  over the warts from the crown): identity or a duller fight?
- `MIRE-5` Should `JudgementField` ignite tar?
- `MIRE-6` Braziers relight in 15 s: too long or too short?
- `MIRE-7` The stomach camera stays outside: is watching a bulge fun?
- `MIRE-8` Is a full floor a loss condition or a slow one?
- `MIRE-9` Is 45 % threatening oppressive in the hands, or only in the
  harness?
- `MIRE-10` Does the flop — its main damage — read from its ring early enough?
- `MIRE-11` Does the Ridgeback want the flop's two opt-in rules (shove out,
  no surface facing down)?

### The Sandmaw — tier 2, noise

*It is only there when it chooses to be — so make it choose.* An 11 m worm
that hunts by ear. Make noise where you want it to surface; stand still to
vanish; beach it to climb its back.
[creatures/sandmaw.md](creatures/sandmaw.md).

- **Play:** `--hunt sandmaw`, `?hunt=sandmaw`. The Pan.
- **Harness:** Champion 18/24 in 191 s, Bulwark 22, Elementalist 11 (338 s),
  Reaver 10, Dual mage 3, Blood mage 0. Zero unanswerable; zero bites while
  quiet.
- **Try first:** stand still and watch the wake look for you; then walk to
  bring it up where you want; hit the throat in a stand, and press `Q` on a
  gulp if it swallows you.

**Decided for you**
- `SAND-C1` **Health 13 000**, from 5 200 (a stand is eight seconds of free
  throat).
- `SAND-C2` The undertow is **2.4 m** in radius, from 3 — the largest that
  lets every class jump out (after the floor-lift fix).
- `SAND-C3` The spit is 6 active frames, from 10 (a dodge could not cover 10);
  the swallow's tell 30 frames, from 24, and a blow anywhere in the open ring
  gags it.
- `SAND-C4` Struck past `interrupt_strain` in a stand **beaches** it (one rule
  for the stand); strain bleeds 1 %/frame, the interrupt is 700.
- `SAND-C5` The Bulwark's raised shield stops the spit. Not built: the
  sand-step dodge, a camera for the swallowed.

**Open**
- `SAND-1` Is standing still too strong? (`silence_patience`, `feel_radius`)
- `SAND-2` Is walking the right thing to be loud (only 3 m/s is quiet)?
- `SAND-3` The Dual mage: the class the worm loses, or a class for whom the
  fight is not happening?
- `SAND-4` The Bulwark's undertow (his hop carries 2.5 m from the centre now):
  close the hole by the radius, or accept it?
- `SAND-5` The Elementalist's stone beach on every rise: identity or a solved
  puzzle?
- `SAND-6` Drawing each heard noise: keep the rings, or only the glance?
- `SAND-7` Is the swallow's `Q`-on-a-gulp readable (the one timed press in
  any creature fight)?
- `SAND-8` Is a stand "threatening" (it can act at once) or a window (every
  move from it is a 20–30-frame tell)? The threat and walk-up bands turn on it.
- `SAND-9` Does the swallow's 4 m mouth read as the thing to hit?

### The Pair — tier 3, awareness

*Either one alone is fair; together, one is always behind you.* Two 1.8 m
cats. Never commit to one while you can't see the other; split them.
[creatures/the-pair.md](creatures/the-pair.md).

- **Play:** `--hunt pair`, `?hunt=pair`. The Den.
- **Harness:** Champion 16/24 in 159 s, Elementalist 24 in 46 s, Bulwark 23,
  Reaver 7, Dual mage 4, Blood mage 0. Zero unanswerable by both clauses in
  all 144 hunts.
- **Try first:** keep both cats on screen; read the tail before you dodge a
  coil; dodge *toward* a pounce and *across* an ambush lane; wait out the twin
  pounce until both leave the ground.

**Decided for you**
- `PAIR-C1` **Health 2400 a cat** (3600 with two hunters).
- `PAIR-C2` The pounce is 30/14/50 for 110 at 1.4 m (from 24/20 for 150 at
  1.6), the flick at frame 13; the dive and twin pounce retimed so a dodge's
  ten invulnerable frames can clear them.
- `PAIR-C3` The ambush lane runs 2.5 m past where it saw you, for 120 (from
  170); nothing that hurts is thrown at a memory; sight is from the head.
- `PAIR-C4` A feint needs its mate round you by 0.17 turns (not "behind and
  free"); a leaping cat has a hurtbox and no body.
- `PAIR-C5` Not built: the torn-ear trophy's banner, the snarl and roar (no
  audio), Cat's step.

**Open**
- `PAIR-1` Is the windup glint a crutch? Play with and without.
- `PAIR-2` Is the tail flick readable at speed? (Moves to frame 6 if feints
  are bitten more than half the time after ten minutes.)
- `PAIR-3` Does the Blood mage's blink break the fight?
- `PAIR-4` Is the Elementalist too comfortable? (She wins every hunt in 46 s.)
- `PAIR-5` Enrage by kill order: flat (kill both close together), or scaled?
- `PAIR-6` Coop's `hit_draws`: give the Ridgeback it too?
- `PAIR-7` Tell the cats apart by more than colour (one heavier, one faster)?
- `PAIR-8` Threatening is 78–81 % against 45, because one of two cats is
  nearly always free: is that the right measure of this fight?

### The Broodmother — tier 3, target priority

*Every second on the little ones she makes more; every second ignoring them,
they eat you.* A 6 m spider with egg sacs on her back; the screech always
leads to the slam that lays her sacs on the floor — the window to pop them.
[creatures/broodmother.md](creatures/broodmother.md).

- **Play:** `--hunt broodmother`, `?hunt=broodmother`. The Hollows.
- **Harness (balanced plan):** Champion 7/24 in 143 s, Dual mage 10,
  Elementalist 1 (but 22 of 24 ignoring the brood), Bulwark 0 (his leap
  takes him to the sacs: 43 pops in 129 slam windows), Reaver and Blood mage 0. Unanswerable 0–3 by class (broodling bites from off screen).
- **Try first:** the screech, then get to a sac in the slam's window and pop
  it; see whether the colours (pale to red) tell you which sac is next.

**Decided for you**
- `BROOD-C1` **A sac is 250 at ×2** (two Champion hits), not 150 and a
  three-hit chain; her hide ×0.45, legs ×0.7 for a longer fight.
- `BROOD-C2` A pop in the slam does not flinch her out of it; no broodling
  starts a windup while she lies in the slam; with her dead, the brood die.
- `BROOD-C3` **§12's first question answered by a test, `aim.rs` unchanged**:
  sacs are not on the crosshair's ray; the straight line from the caster still
  meets the sac.
- `BROOD-C4` The slam does not crush her own brood (question 6, not built).

**Open**
- `BROOD-1` (answered by `BROOD-C3`; confirm) Should a sac be on the ray?
- `BROOD-2` Is the burst sac growing back too cruel for a first hunt?
- `BROOD-3` Does the Dual mage's float need more than the web-down?
- `BROOD-4` Is the screech → slam chain too predictable (a script)?
- `BROOD-5` Is the enraged fight easier or harder than the one before it?
- `BROOD-6` Should the slam crush her own brood?
- `BROOD-7` The Reaver's copy turning to broodlings: identity or hole?
- `BROOD-8` The Elementalist wins by ignoring the brood (22 of 24): the
  dilemma does not hold for her. A mother who leaves fire, brood that come for
  whoever burns her, or accept?
- `BROOD-9` Won fights last 2.5 min, not 3–6; is that short?

### The Veilstalker — tier 4, what you can't see

*You never see it; you see what it touches.* A 4 m animal under a veil:
footprints, breath, ripples, the shimmer before a strike, and the paint your
hits leave. [creatures/veilstalker.md](creatures/veilstalker.md).

- **Play:** `--hunt veilstalker`, `?hunt=veilstalker`. The Ashwood.
- **Harness:** Champion 6/24 in 336 s, Elementalist 22 in 208 s, Bulwark 9,
  Reaver 6, Blood mage and Dual mage 0. Zero blind hits, zero unanswerable.
- **Try first:** follow the three-toed prints; hold still on a decloak with
  no fresh prints under it (a mimic); walk out of the spear's lane; tip a
  brazier across its trail.

**Decided for you**
- `VEIL-C1` **Health 7000** (11 200 for two), not 4500, and **the hits
  heavier** (lunge 260, spear 190, rake 140+140, pounce 220, quill 70); a
  region mottles at 800.
- `VEIL-C2` At most three strikes an engagement; it will not strike on bare
  ground; the stalk is 90–150 f at 11 m.
- `VEIL-C3` "The glance of the look" is the fighter's facing; the view gate
  is "plainly in view" with margins.
- `VEIL-C4` The shimmer is a translucent silhouette (the stated fallback), not
  a refraction. Not built: Veilstep, the mottled-pelt trophy, prints in water.

**Open**
- `VEIL-1` Off-screen decloaks are never allowed: eerie, or a rule a player
  spots in two minutes and plays around?
- `VEIL-2` Glancing the look (your camera's facing, a glance old): keep, or
  gate on the body's facing?
- `VEIL-3` Does a shimmer at 9 m/s read on a laptop screen?
- `VEIL-4` Is the Elementalist's fight too easy? (22 of 24.)
- `VEIL-5` Does the Champion enjoy it (it leaves after two hits against a
  class built on three)?
- `VEIL-6` Should fighters leave footprints?
- `VEIL-7` Six seconds of paint, 800 to mottle: first guesses.
- `VEIL-8` The way-in window reads 6–8 % against ~20: really walk-up?

### The Mantis — tier 5, fundamentals

*It fights like a player: it blocks, it parries, and it punishes what you
repeat.* A 3 m duellist with two blades; notches on its blades mark the move
it is ready for. [creatures/mantis.md](creatures/mantis.md).

- **Play:** `--hunt mantis`, `?hunt=mantis`. The Shrine.
- **Harness:** solo, Elementalist 24 in 71 s, Bulwark 18, Dual mage 2,
  Champion, Reaver, Blood mage 0 (the Champion won 5 of 48 when built; 0 of 48
  since the cast's merges). As a pair, Bulwark 11 of 12, Champion 4.
- **Try first:** don't repeat one move into its guard (watch the notches);
  break its guard inside its hold with an unblockable; hit its recoveries;
  go round the side whose blade you broke. Don't dodge the coil early.

**Decided for you**
- `MANT-C1` **Health 7000 solo, 8400 for two** (not 4000/6800); damage ×0.83
  on every scythe.
- `MANT-C2` The guard has a longest hold (120 f) and a rest after (40 f); a
  move seen too late to parry raises the guard on 35 % of commitments.
- `MANT-C3` The habit remembers a Champion's *weapon*, not his chain link.
- `MANT-C4` The court is square with filled corners, walled at 15 m, not round.

**Open**
- `MANT-1` Keep the habit memory (it reads what you repeat)? (Also `CAST-2`.)
- `MANT-2` The second slash's marker shows where and not when: acceptable?
- `MANT-3` Should the counter (12 f) be reactable (16 f)?
- `MANT-4` Should a fighter's parry stagger a creature?
- `MANT-5` Is the Reaver too comfortable? (Measured: she wins none.)
- `MANT-6` The Dual mage has no guard breaker — is `Q` the guard breaker on
  every class?
- `MANT-7` Fight length: 3–4 min solo against the tier table's 8–20 — change
  the table for duellists, or raise health?
- `MANT-8` Does the coil read as fair ("don't blink"), or as dodging being
  broken?
- `MANT-9` **The Bulwark is the easiest class, not the hardest** (his Slam on a
  whiff is worth twice anybody's). Lower a Slam's damage to it, accept it, or
  wait?

### The Galewing — tier 4, the sky

*It lives where you can't reach — make it come down, then choose whether to go
up with it.* An 18 m raptor. Wing-strike it on a low pass; ride it into the
sky and get off in time. [creatures/galewing.md](creatures/galewing.md).

- **Play:** `--hunt galewing`, `?hunt=galewing`. The Cliffs.
- **Harness:** plan A / plan B (the ride, `--gamble`): Champion 4 / 7,
  Elementalist 17 / 18, Reaver 9 / 11, Bulwark 0 / 1, Blood and Dual 0. Zero
  unanswerable, zero unseen tells. Coop 0 of 12.
- **Try first:** dodge the Stoop at the hit and hit its wings while it is down;
  crouch under the talon pass; when it gathers itself to lift, ride it, brace
  through the roll, and step off at the low swoop.

**Decided for you**
- `GALE-C1` Falls are the built rule — free to 9 m, 25 a metre (the document
  said 7.5 m); a throw at 20 m costs 275.
- `GALE-C2` The plateau is a 12 m solid (the game's floor is at zero).
- `GALE-C3` It decides once per approach which move the approach is for; it
  dwells 2.5 s on the floor after a Stoop; it perches with one wing broken; a
  crash cannot follow a crash.
- `GALE-C4` The volley is unguardable (question 5 not built).

**Open**
- `GALE-1` Does the Dual mage's boarding in the air make the crash pointless
  for her?
- `GALE-2` **Two thirds out of reach** for the melee classes (a third was the
  premise): a waiting room? The levers moved it little.
- `GALE-3` Crouching under the talons with 20 cm to spare: skill or hitbox
  trick?
- `GALE-4` Does a level horizon through the roll feel like riding a bird?
- `GALE-5` Should the Bulwark's guard cover a volley from above?
- `GALE-6` Is 275 for a throw at 20 m the right price (fall damage is
  global)?
- `GALE-7` Gravity and the grip test upside down: leave it out?
- `GALE-8` Coop loses (0 of 12): the second hunter runs the same plan. A plan
  for two, or a fight for one?

### The Siegeshell — tier 5, built for two

*It isn't fighting you; it is walking to the wall, and you are the only thing
that can stop it.* A forty-metre shell on six tower legs. Break two ankles on
one side, climb the stair, break the three anchors before the beam breaches
the wall twice. [creatures/siegeshell.md](creatures/siegeshell.md).

- **Play:** `--hunt siegeshell`, `?hunt=siegeshell`; a friend over
  `--port/--peer` for the pair it is built for. The Last Valley.
- **Harness:** a pair — Reaver 12/12, Dual mage 9, Elementalist 6, Champion 5,
  Bulwark and Blood mage 0. Alone — Reaver 3, Elementalist 2, everyone else
  0. Unanswerable: parasites' bites from off screen (Champion 1 alone, 5 as a
  pair).
- **Try first:** jump the footfall ring on the beat; break two ankles on one
  side and climb the broken legs; brace through the shrug, jump the shiver.

**Decided for you**
- `SIEGE-C1` The valley is **300 m**, not 240.
- `SIEGE-C2` **Health is the anchors'**: 30 000 that only an anchor breaking
  takes; anchors **7000**, not 2400.
- `SIEGE-C3` A buckle cannot stumble it again for 15 s (`StumbleRest`).
- `SIEGE-C4` The stair is a ramp, every footing inside the Bulwark's hop; the
  flanks hinge under the far side.
- `SIEGE-C5` **Aim A3 built**: the top of a part you could stand on, seen from
  above, is a place (it also applies to the Ridgeback — `SIEGE-4`). The camera
  passes through it.

**Open**
- `SIEGE-1` The fail state: should the town keep the damage?
- `SIEGE-2` The Elementalist's structure jump clears the crown from the floor:
  keep it, or cap a stone launch near a creature?
- `SIEGE-3` The Dual mage's wings reach the standing rim: identity?
- `SIEGE-4` The top-face aim rule (A3) changes the Ridgeback too: acceptable,
  or riders only?
- `SIEGE-5` Is 0.8 m/s visibly walking?
- `SIEGE-6` A jump every three seconds for ten minutes: rhythm or chore?
- `SIEGE-7` Solo: a floor or a wall? (Four classes win none alone.)
- `SIEGE-8` A shield, a pool and a shadow that live on a part: build the one
  mechanism for three classes, or none?
- `SIEGE-9` The Reaver wins 12 of 12 as a pair; the Bulwark and the Blood mage
  win nothing in any company.

---

## 4 · Across the cast

**The bestiary** ([bestiary.md](bestiary.md) §6–§7)
- `CAST-1` **Tier 5 is built for two.** The Siegeshell and the Mantis are
  tuned so a decent player alone mostly loses. Right ceiling, or should every
  creature be soloable?
- `CAST-2` **The Mantis reads habits** (it counters a repeat sooner). Keep it
  or cut it? (Same as `MANT-1`.)
- `CAST-3` **Three new rides** (the herd, the Galewing, the Siegeshell): is
  the ride the Ridgeback's thing or the game's?
- `CAST-4` The tier table's lengths: duellists (the Mantis) want a shorter
  band than 8–20 min; probably the table moves.

**The world** ([world.md](world.md) §8)
- `WORLD-1` Tempering — three tempers on glance, lead and decisiveness: is
  "the same creature, cleverer" the ceiling you want? Temper III is one win in
  forty for the scripted hunter; fair or only long?
- `WORLD-2` The Siegeshell's breach: does a town carry the scar (only matters
  once there is a town)?
- `WORLD-3` The picker is a key cycle and a text list on the HUD, not a menu.
  Enough for now?

**The classes against creatures** ([bestiary.md](bestiary.md) §8, "Across the
cast")
- `CLASS-1` **The Elementalist's fire pillar decides most fights.** No creature
  steps out of a pillar, so its whole burn lands from nine metres: 24 of 24
  against five creatures. Creatures learn fire, the pillar's burn on a
  creature comes down, or it is her identity?
- `CLASS-2` **Is the Blood mage's kit meant to fight creatures?** She wins only
  where a stun holds something over a spike (Gnawers, Hornback). "The creature
  does not bleed" is what costs her most.
- `CLASS-3` **The Dual mage lives on tempo**: right against creatures that
  leave few windows? Her punch also passes over what is at her feet.
- `CLASS-4` **The Reaver's Mantis**: her shadow beside its guard is played and
  wins nothing.
- `CLASS-5` **Resolved 2026-10-01: the thrown shield strikes creatures and
  critters.** Thrown, the first body it meets takes the throw (with its
  weight) and it plants there empty; recalled, each body it passes through
  takes the recall once and it comes home -- the versus rule, unchanged.
  Through the swing's own path (`state::shield_hitbox`, drawn by the overlay;
  `part_under` and `Monster::take_blow`; `Body::touched_by`, the pack's guard
  and `pack::struck`), so guards, weak points, breakable parts and presence are
  each creature's own; `crates/sim/tests/thrown_shield.rs` states it. The
  Bulwark's harness: the Hornback 19 to 20, the crossing 7 to 8 of 12, the
  Pair 21 to 23, the Broodmother 1 to 0, the Galewing's plan B 2 to 1, the rest
  unchanged; no pin moved. **Still open for a person:** a guard reads the blow
  as coming from where the shield was thrown or planted (it was the shield's
  own position, which the Mantis read as "from under its belly" and never
  guarded); a loaded throw knocks a critter down only if the blow knocked it
  out of what it was doing; and the Siegeshell's climb (a shield that plants
  *on* a part) is not built -- it needs P1.
- `CLASS-6` The thrown shield has no frames at all (an instant mechanic),
  unlike the Reaver's send: should it pay the same price? (kits/bulwark.md.)

**The shared machinery** ([species.md](species.md) §6, [critters.md](critters.md)
§7, [hazards.md](hazards.md) §8, [arenas.md](arenas.md) §5)
- `MACH-1` The Guillotine passes over a 0.6 m body: lower `lotus_height`
  (changes versus), or let it be the Big One's tool?
- `MACH-2` Lunges run through critters (fighters pass through them): right?
- `MACH-3` The Ridgeback does not collide with solids (turning it on is a new
  fight): keep?
- `MACH-4` Hazards reach a creature by its centre (the Mireback opts into its
  footprint): keep the shared rule?
- `MACH-5` A Reaver's shadow starts at the versus mark in a hunt and eases
  over: fix it (moves her pin)?
- `MACH-6` `aim::look_onto` keeps six rounds (the bot's pins); raise it?

**Decided for you, cast-wide**
- `CAST-C1` **One fall rule**: free to 9 m from the footing, 25 a metre (the
  documents asked 7.5 m, and 30 and 18 a metre).
- `CAST-C2` Critters are a **box** on their feet, not a capsule; a swing meets
  a short body at the share of its height a level swing meets a fighter.
- `CAST-C3` Common knobs share one range each across species; breaking any
  breakable part stumbles a creature.
- `CAST-C4` **Health was raised on six creatures** from their documents — the
  bull 3500 → 2000 (down), the Mireback 11 000 → 10 000 (down), the Sandmaw
  5200 → 13 000, the Pair 2400 a cat, the Veilstalker 4500 → 7000, the Mantis
  4000 → 7000, the Siegeshell's anchors 2400 → 7000 — each to bring the
  scripted hunter's fight into its tier's length.
- `CAST-C5` **A temper is four shares** of a creature's own glance, lead,
  decisiveness and strain fall, so no species declares anything (first values
  made temper I nearly as hard as III and were changed).
- `CAST-C6` **The scripted hunter plays all six classes** (`hunt::class`), and
  its pin was moved deliberately for the Dual mage (2026-10-01) and added for
  the Reaver, the Elementalist and the Blood mage; on 2026-10-01 the Bulwark's
  learned the shield throw (`REV-C2`).
- `CAST-C7` Aim A3 and A4: the top of a standable part is a place, and a swing
  on a back is level with the back. Both apply to the Ridgeback too.

---

## 5 · Please confirm

Every `C` above, in one list, so a single reply can carry them:
`REV-C1`, `REV-C2`, `POL-C1`–`POL-C6`, `CAST-C1`–`CAST-C7`, `GNAW-C1`–`C4`, `HORN-C1`–`C5`,
`MIRE-C1`–`C5`, `SAND-C1`–`C5`, `PAIR-C1`–`C5`, `BROOD-C1`–`C4`,
`VEIL-C1`–`C4`, `MANT-C1`–`C4`, `GALE-C1`–`C4`, `SIEGE-C1`–`C5`.

## 6 · Decided while preparing this review

- `REV-C1` **The dev species leave the cycle.** The gnats (id 11) and the
  sentinel (id 12) were in `Shift+H` and the trophy list, by the choice
  recorded in critters.md §7 and hazards.md §8. They are kept out of both now
  (`SpeciesId::is_dev`, `species::shown`) and reached by `--hunt gnats` and
  `--hunt sentinel`; a dev species is listed while it is being hunted.
- `REV-C2` **The Bulwark throws his shield in the scripted hunter**: on a
  window five to eleven metres off he throws it at the work, leaps to it, and
  Slams out of the leap; a shield left planted is recalled. See the feel log
  of 2026-10-01 and bestiary §8 for what it did to his numbers.
- **Fixed: `Shift+H` stopped at the Gnawers.** The cycle stepped from
  `w.monster()`, and a pack creature (the Gnawers, the herd) has none, so from
  the Gnawers it went back to the Ridgeback and the nine after them were never
  offered. It steps from what is hunted now; `picker`'s
  `the_list_and_the_cycle_hold_every_creature_and_no_dev_species` walks the
  whole cycle.
- **Checked, 2026-10-01**: `?hunt=<x>` for all twelve fights loads clean in
  headless Chromium (`WEB_QUERY=hunt=<x> ./scripts/web-smoke.sh`: no console
  error, no failed request, the right arena and the `>` on the right creature),
  and every fight's desktop capture is in the [gallery](gallery/README.md). The
  Sandmaw and the Veilstalker show no creature in their arena shots, by design.


## 7 · Polished after the review was written

*2026-10-01, [plans/polish-fights.md](plans/polish-fights.md).* A pass over the
defects the builds left open. What it answers or moves above, by ID, and what
it decided for you.

**Answered or moved**
- **Unanswerable hits are zero for every creature, every class, solo and in
  pairs** (the §2 column's 0–1, 0–3 and 0–5). A pack's bite begun off your
  screen now lands only through a marker under you for a reaction
  (`POL-C1`); the Mireback's own count was measuring the wrong tar (`POL-C3`).
- `CLASS-1` **Looked into, not changed**: no creature fails its own document
  against the pillar. Only the Mireback (flee) and the Veilstalker (panic)
  have a rule about fire, and both follow it; the rest stand in it because
  nothing says they should not. The choice is set out in bestiary §8, "The
  fire pillar, looked into".
- `GNAW-1` Health is not the lever for a short fight: tried at three sizes,
  each bought five to fourteen seconds and cost the Blood mage and the Dual
  mage most of their wins (gnawers.md §13). Not changed.
- `CAST-1` The Siegeshell alone is now one in twenty-four across the classes
  (`POL-C4`); the Mantis is unchanged.
- `CAST-C4` The Mireback is back at its document's 11 000 (`POL-C5`).

**Decided for you**
- `POL-C1` **A pack's referee** (critters.md §2): a windup begun off a
  fighter's screen lands on them only if its marker has been under them for
  fifteen frames, and a committed body keeps its target through the glance.
  Every pack: the Gnawers, the herd, the brood, the parasites. Their wins did
  not move.
- `POL-C2` **The four windows are each body's own, and of the frames they are
  about** (bestiary §8; the-pair.md, galewing.md, veilstalker.md §13). The
  Pair's are each cat's (the old min-of-two is kept as "threatening,
  together"); the Galewing's leave out the frames it is out of reach and do
  not count its lift as a threat; the Veilstalker's leave out its unseen
  stalk; the Sandmaw's target is amended, not its measure. Every other
  creature prints the same windows as before, to the frame.
- `POL-C3` **The Mireback's "flop from fresh tar"** counts tar that was not on
  the floor at the commit and was under you the frame before the crash -- not
  the crash's own ring, and not old tar you walked into.
- `POL-C4` **The Siegeshell alone has anchors of 5 900** (7 000 for a pair):
  six wins in 144 solo hunts, the Champion one in 24.
- `POL-C5` **The Mireback pauses 60 frames between moves** (from 40), with
  11 000 health: threatening 35–41 % from 45, the Champion 18 of 24 in 147 s.
- `POL-C6` **The Galewing's out of reach** counts a bird you could walk to as
  in reach: 49–59 % for the melee classes. Every creature lever that took it
  lower cost the ride its wager (plan B 7 of 24 to 1), so none was kept.

**New, for a person**
- `POL-1` The Galewing's coop loss (0–1 of 12) is the scripted pair dropping
  out of their jump under the Wing buffet, not the bird: a hunter fix, or is
  a buffet that catches a falling jumper right?
- `POL-2` The Broodmother's pops (a quarter a slam window) and her short
  fights come from a balanced Champion that spends no time at the sacs: the
  hunter's to fix before the creature is judged.
- `POL-3` The Siegeshell's Reaver pair (12 of 12) and its floor's tenth
  threatening are unchanged: the first is her §7, the second the price of a
  ground half meant to be won. Accept both?
- `POL-4` The Mantis's Bulwark still wins 18 of 24 by Slam into its
  recoveries; its habit reads his Bash and is never right. `MANT-9` stands.
