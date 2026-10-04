# Design — current state

Everything currently decided, proposed, or parked, in one place. This supersedes
[`../archive/`](../archive/README.md), which is 2016–2019 source material kept for reference.

**Start here:** [combat kernel](combat-kernel.md) → [controls](controls.md) →
[ability spec](ability-spec.md) → a class kit. For implementation, see
[architecture](architecture.md); before touching anything that is *pointed at
something*, see [aiming](aiming.md).

**Reviewing the creatures?** [review.md](review.md) is the way through the
bestiary as built: how to start every fight, what each creature is, where the
harness says it landed, what to try first, and every open question and every
decision made on your behalf, each with an ID to answer by. The pictures are
in the [gallery](gallery/README.md).

---

## 1 · Settled

**Frame.** Peer-to-peer, isolated battle arena. Coop against monsters, or versus. 3D third
person, with ability spectacle as an explicit goal. Smash-like — spatial commitment, whiff
punishment, reads — not Tekken-like. The closed arena is deliberate: far less art than an
open world.

**No cooldowns.** Abilities cost **frames** (primary) and **the class mechanic** (secondary).
There is no universal resource bar. **One qualifier, added 2026-09-14:** a move you have just
thrown is locked for 30 frames — *that* move, nothing else, so the question stays "what else
have I got" rather than "do I have anything". See
[combat-kernel.md](combat-kernel.md) §"The repeat lockout" for why that is not the thing this
line rules out.

**Time to kill.** ~60 seconds versus. 1–20 minutes coop, by fight difficulty.

**Defense splits into two verbs.** Dodge is universal and evasive. Block is shield-gated and
positional, covering a facing arc rather than a bubble. Blocking costs **space and a
vulnerable window** — knockback plus stunlock, no chip damage, no guard meter. Parry is the
opening frames of block and rewards with a stagger. Special attacks are the guard breakers.

**Control grammar — ⚠️ shifted 2026-09-11 and again 2026-09-16, no longer settled.** Now:
*click = attack, **shift = dodge and nothing else**, WASD = move, space = jump, `Q` = the
class special, `E` = the class mechanic, mouse = where.* Shift used to be the attack modifier
as well, told apart from the dodge by whether a click happened to be held — a key whose meaning
depended on the rest of your hand. It is one verb now, which strands the committed move on
three classes until each is given a new home; see
[controls.md](controls.md#shift-is-one-verb-now-2026-09-16). `E` is usually an instant state
change; on three classes the mechanic is an ability instead, because pressing it is not free —
the Blood mage's since 2026-09-12 and the Dual mage's since 2026-09-13, both because their
mechanic has no state to toggle, and the Reaver's because moving a second body across the
arena takes frames and does damage. Hers then went to **right click**, because it is aimed
and the mouse means where, which makes the Reaver the one class where `E` carries something
that is not the mechanic. Was: *click = attack, shift =
ability, WASD = move, space = move more, shift beats WASD.* The last two did not survive
contact with the sandbox. **Space now always jumps** — a vertical takeoff and nothing else —
and **shift plus a direction dodges**. Space plus a direction used to dodge, which meant
pressing jump while moving, which is most of the time, did not jump.

That moved dodge onto shift, where the ability modifier already lived, so the parts of the
grammar that depended on "shift beats WASD" are **open again** rather than settled — and the
ability modifier has since left rather than learn to share. See
[controls.md](controls.md) §Open and §4 below.

**Movement.** Space jumps, and holding it goes higher. Floaty on purpose — a full hop is
around a second, because verticality is part of the positioning game and a beginner needs time
in the air to use it. Classes differ in the air before they differ anywhere else: jump height,
gravity, fall speed and steering all vary. Air control is Quake's — holding forward buys
nothing, strafing across your motion turns you, and turning the camera while strafing is where
the skill ceiling is. Aerials suspend the fall for a per-move number of frames, and **each one
after the first in a single airtime is worth less** — a class with two interchangeable pokes
could otherwise alternate them and never come down.

**The jump is the floor of the movement system, not the whole of it — ⚠️ retuned 2026-09-17.**
Every class has one thing it does with the ground, and a jump good enough to go most of the
way to that technique without it makes the technique a flourish. So the takeoff came down by a
twelfth, which is a third off every apex: full hops now run 2.7 m on the Bulwark to 6.0 m on
the Dual mage, where they ran 4.1 to 8.7. The hold's sustain was stiffened at the same time,
because at its old strength and length a held jump read as an elevator with gravity switching
on at the top. See [feel-log.md](feel-log.md); it changes the Ridgeback's climb, which
`cargo run -p sim --bin beastcheck` now prints as a spread rather than one number.

**A movement tool's reach is paid for in execution — timing, aim, precision — never in
waiting, and never free.** Accepted 2026-10-03 from the jump courses
([courses.md](courses.md) §0): a tool that reaches far with no timing, or reaches far after
standing still long enough, is the thing to fix.

**Roster.** Six classes. The Gatekeeper is retired and not backfilled — a missing long-range
poke is a design choice in a closed arena, not a gap.

## 2 · The roster

| Class | Mechanic — what abilities spend | Primary buttons | State |
| --- | --- | --- | --- |
| [Shadow Reaver](kits/shadow-reaver.md) | Shadow position (always placed) — **a tally on the target, built** | `L` auto · `R` Send shadow · `Q` Guillotine lotus · `E` Executioner | v2 built, unplayed ([v2](shadow-reaver-v2.md)) |
| [Elementalist](kits/elementalist.md) | Structure slots (cap 3) — **and fire on a stone lights it** | `L` beam auto · `R` Cataclysm · `M` Cinder spray · `M5` Quake · `Q` Fire pillar, held: Strike · `E` Raise, held: Fissure · `F` Updraft/Downdraft · `R` Tremor · **and the air row on every one** | Strong; **v2 built, unplayed** ([v2](elementalist-v2.md)) |
| [Blood mage](kits/blood-mage.md) | Grey health, and essence pools where she cut somebody | `L` Reaping sweep · `R` Haemorrhage · `M` Bloodletter · `Q` Grasp · `E` Black spike · `shift` on a pool = Blink | **v1 built**, unplayed; [the kit](blood-mage.md) |
| [Dual mage](kits/dual-mage.md) | Two bars, Dark and Light, and the hill between them | `L` dark auto (pulls) · `R` light auto (pushes) · `M` Lance, two forms · `Q` Judgement · `E` Sweep · **the lower bar unlocks a blink, a second jump, wings** | **Rebuilt on two bars**, unplayed |
| [Champion](kits/champion.md) | Rush charge (one, cancels recoveries) | `L`/`M`/`R` = sword/hammer/spear, three hits deep · `space` + weapon = takeoff · `E` Rush | Shaped |
| [Bulwark](kits/bulwark.md) | Shield position — **and its weight**: stored when blocked, spent by Slam | `L` Bash · `M` Slam · `Q` Grapple · `R` Guard · `E` Throw/Recall/leap | New; [v2 built](bulwark-v2.md), unplayed |
| ~~Gatekeeper~~ | — | — | Retired |

*Dual mage was Statera. Champion was Bellator, and Shifter before that.*

Each kit is **six abilities plus an auto and the mechanic input** — enough for a real match,
few enough to balance and to read in third person.

### The creatures

Eleven, all built (2026-10-01), each in its own arena, on five tiers of difficulty
([bestiary.md](bestiary.md); where each landed for every class is its §8). Start any of them
with `--hunt <name>` (`?hunt=<name>` in the browser); `Shift+H` steps through all eleven in
game, `T` tempers the one you are hunting, and the HUD's list shows your trophies.
[review.md](review.md) has each one's command, numbers and open questions.

| Creature | Tier | Arena | `--hunt` | One line |
| --- | --- | --- | --- | --- |
| [Gnawers](creatures/gnawers.md) | 1 | the Commons | `gnawers` | A knee-high pack and its Big One: keep them in front, kill the leader |
| [Hornback herd](creatures/hornback.md) | 1 | the low meadow; the crossing | `hornback`, `hornback-escort` | A herd and a bull: bait the charge into a rock; escort a cart through the migration |
| [Sandmaw](creatures/sandmaw.md) | 2 | the Pan | `sandmaw` | A worm that hunts by ear: make noise where you want it, stand still to vanish |
| [Mireback](creatures/mireback.md) | 2 | the Mire | `mireback` | A toad that takes the floor away: burn its tar |
| [Ridgeback](monsters.md) | 3 | the proving ground | `ridgeback` | Break a foot, climb, ride the buck, topple it |
| [The Pair](creatures/the-pair.md) | 3 | the Den | `pair` | Two cats; one is always behind you |
| [Broodmother](creatures/broodmother.md) | 3 | the Hollows | `broodmother` | A spider and her clock of sacs: pop them in the slam's window |
| [Galewing](creatures/galewing.md) | 4 | the Cliffs | `galewing` | A raptor out of reach: bring it down, or ride it up |
| [Veilstalker](creatures/veilstalker.md) | 4 | the Ashwood | `veilstalker` | The animal you never see: read what it touches |
| [Mantis](creatures/mantis.md) | 5 | the Shrine | `mantis` | A duellist that guards, parries and reads what you repeat |
| [Siegeshell](creatures/siegeshell.md) | 5, for two | the Last Valley | `siegeshell` | A walking hill; break its anchors before it reaches the wall |

Two dev species, the gnats (a pack with nothing of its own) and the sentinel (one of every
hazard and sense), are reached by `--hunt gnats` / `--hunt sentinel` and kept out of the cycle
and the trophy list. The range (`--arena range`) is the dev arena with one of everything.

## 3 · Documents

| Document | Covers | Status |
| --- | --- | --- |
| [combat-kernel.md](combat-kernel.md) | No cooldowns, TTK, what that breaks | Decided |
| [controls.md](controls.md) | Input grammar, per-class schemes | Proposed |
| [ability-spec.md](ability-spec.md) | The format kits are written in | Proposed |
| [aiming.md](aiming.md) | The one raycast, and the two kinds of skillshot | Decided |
| [defense.md](defense.md) | Dodge, block, parry, guard breaks | Proposed |
| [blood-mage.md](blood-mage.md) | v1 kit: grey health, essence pools, the scythe, the blink | **Built 2026-09-23**, unplayed |
| [dual-mage.md](dual-mage.md) | Two bars, the hill between them, the depth curve, the tiers, ascension and the wings; the single bar under "Was" | **Built 2026-09-23**, unplayed |
| [dual-mage-v2.md](dual-mage-v2.md) | The proposal the two bars were built from | Built; folded into dual-mage.md |
| [plans/](plans/) | Action plans for implementation threads: [Blood mage v1](plans/blood-mage-v1.md) (built), [Dual mage v2](plans/dual-mage-v2.md) (built), [Shadow Reaver v2](plans/shadow-reaver-v2.md) (built, M4's knob pass waiting on play), [Bulwark v2](plans/bulwark-v2.md), [Elementalist v2](plans/elementalist-v2.md) (built, M6's play pass waiting on a person), [Gnawers](plans/gnawers.md) (built; tuning toward the time band open), [Mireback](plans/mireback.md) (built; the coat's sheen and the threat share open), [Sandmaw](plans/sandmaw.md) (built; the threat and walk-up bands open), [Hornback herd](plans/hornback.md) (built; the mages and the two-hunter crossing open), [The Pair](plans/the-pair.md) (built; the threat band and the Blood and Dual mage hunts open), [Veilstalker](plans/veilstalker.md) (built; the way-in band and the Elementalist's fire open), [Mantis](plans/mantis.md) (built; the Bulwark's three-in-four and the fight length open), [Galewing](plans/galewing.md) (built; the out-of-reach share and the classes past the Champion open), [Siegeshell](plans/siegeshell.md) (built; the solo rate and the fight length open) | Briefs |
| [champion.md](champion.md) | Forms, the three-hit chain, and the mid-animation swap | Decided |
| [bulwark.md](bulwark.md) | Why the class exists; shield as volume | Proposed; v2 proposed |
| [bulwark-v2.md](bulwark-v2.md) | v2: the shield is a battery — weight, Slam on `M`, the planted wall | **Built 2026-09-23**, unplayed — weight, Slam on `M`, the loaded throw, the planted wall, the health table |
| [shadow-reaver-v2.md](shadow-reaver-v2.md) | v2: the shadow aims itself, marks, and the cash-in on arrival | **Built, unplayed** |
| [elementalist.md](elementalist.md) | Structure interaction in versus | Decided |
| [elementalist-v2.md](elementalist-v2.md) | v2: four new inputs, hold-to-charge on both placement buttons, Updraft and Downdraft on her body, Cinder spray, Quake and Tremor, lit stones, the dodge through a stone | **Built 2026-09-30**, unplayed; `cargo run -p sim --bin elemental` prints its numbers |
| [gatekeeper-retirement.md](gatekeeper-retirement.md) | Why it was cut, what was salvaged | Decided |
| [monsters.md](monsters.md) | The Ridgeback: the climb, the ride, the control algorithm, measuring the fight | Proposed, rebuilt; **hunts since 2026-09-25**: eight moves, a threat at every range, unplayed |
| [review.md](review.md) | **The review guide**: every fight's command, numbers, what to try first, every open question and every decision made on the owner's behalf, by ID; the [gallery](gallery/README.md) of every creature, arena and telegraph | **Written 2026-10-01** for the owner's review |
| [bestiary.md](bestiary.md) | The cast after the Ridgeback: the contract every creature is held to, eleven creatures on five tiers, the shared machinery (P1–P8) and the aiming changes (A1–A5) they need, and the build order; §8 **where the cast landed**, every creature for every class | **Proposed 2026-09-30; the cast accepted as the first mix**; P1, P2 and P8 built 2026-10-01; §8 measured 2026-10-01, the scripted hunter playing all six classes (`hunt::class`) |
| [species.md](species.md) | A creature is a table: the `Species` every creature is declared as, its knobs, two creatures in the world, and **the recipe for adding one** | **Built 2026-10-01** (bestiary P1 and P8); the Ridgeback pinned bit for bit |
| [critters.md](critters.md) | Small bodies and packs: the 48-byte critter (a box on its feet), the pack brain (glance, ring, tokens, leader, morale, spawning, a monster's pack, standing on a creature), the aim change that makes a knee-high body hittable, `critcheck`, and **the recipe for a pack creature** | **Built 2026-10-01** (bestiary P3, A1, A2); the Ridgeback and versus pinned bit for bit |
| [hazards.md](hazards.md) | The hunt's shared machinery: the lore (one region of the snapshot each fight lays out), floor hazards, perception and hearing, the one fall rule, defended things, creatures against solids, the eye under a ceiling, lengths across the valley, the dev sentinel, and **the recipe for a creature's share of it** | **Built 2026-10-01** (bestiary P4, P5, P6's falls, P7, A5); the Ridgeback and versus pinned bit for bit |
| [arenas.md](arenas.md) | An arena is a table: bounds, solids (ceilings included), floor materials, spawns; the picker (`--hunt <creature>`, `--arena`, `H` / `Shift+H` on the wire); **the recipe for adding one** | **Built 2026-10-01** (bestiary P2, world W0); the proving ground pinned bit for bit |
| [creatures/](creatures/) | One design per creature: [Gnawers](creatures/gnawers.md) (**built 2026-10-01**: `--hunt gnawers`, the Commons; §13 is where it landed), [Hornback herd](creatures/hornback.md) (**built 2026-10-01**: `--hunt hornback`, the low meadow, and `--hunt hornback-escort`, the crossing; §13 is where it landed), [Sandmaw](creatures/sandmaw.md) (**built 2026-10-01**: `--hunt sandmaw`, the Pan; §13 is where it landed), [Mireback](creatures/mireback.md) (**built 2026-10-01**: `--hunt mireback`, the Mire; §13 is where it landed), [The Pair](creatures/the-pair.md) (**built 2026-10-01**: `--hunt pair`, the Den; §13 is where it landed), [Broodmother](creatures/broodmother.md) (**built 2026-10-01**: `--hunt broodmother`, the Hollows; §13 is where it landed), [Galewing](creatures/galewing.md) (**built 2026-10-01**: `--hunt galewing`, the Cliffs; §13 is where it landed), [Veilstalker](creatures/veilstalker.md) (**built 2026-10-01**: `--hunt veilstalker`, the Ashwood; §13 is where it landed), [Mantis](creatures/mantis.md) (**built 2026-10-01**: `--hunt mantis`, the Shrine; §13 is where it landed), [Siegeshell](creatures/siegeshell.md) (**built 2026-10-01**: `--hunt siegeshell`, the Last Valley; §13 is where it landed) | All ten built: the Gnawers, the Hornback herd, the Mireback, the Sandmaw, the Pair, the Broodmother, the Galewing, the Veilstalker, the Mantis and the Siegeshell |
| [courses.md](courses.md) | **Jump courses**: the movement envelope of five classes played in the lab (`--bin envelope`), six floating-island courses (two easy, two hard, two barely possible; `--arena climb`, `N` steps through them), every hop measured for every class with a timing and a distance margin (`--bin courses`), and which tools trivialise what | **Built 2026-10-03**, unplayed; diagnosis, nothing decided |
| [world.md](world.md) | For now: separate arenas picked from the dev harness, with trophies and tempered rematches. Later: a valley of places joined by trails | **Decided in part, 2026-09-30**; W0 (the picker), W1 (trophies) and W2 (tempers) built 2026-10-01 |
| [architecture.md](architecture.md) | Rust workspace, determinism, rollback | Decided |
| [web.md](web.md) | The browser build: what a page cannot do, and what it does instead | Decided |
| [animation.md](animation.md) | The skeleton, authoring clips, the hub | Decided |
| [sparring.md](sparring.md) | The sparring bot: late eyes, imperfect hands, plans chosen by chance, a personality per match | **Built 2026-09-26**, unplayed |
| [parked.md](parked.md) | Progression and equipment | **Parked** |

## 4 · Open

Nothing here blocks a prototype.

**The creatures' questions are in [review.md](review.md)**, grouped by creature with an ID each
(`GNAW-1`, `HORN-C1` …), and the cross-cutting ones there under `CAST`, `WORLD`, `CLASS` and
`MACH`. The four that cut across the whole cast: the Elementalist's fire pillar wins most fights
from nine metres (`CLASS-1`); the Blood mage wins almost nothing against a creature
(`CLASS-2`); tier 5 is built for two (`CAST-1`); and the thrown shield does not touch a
creature (`CLASS-5`).

| Item | Question |
| --- | --- |
| **Class names, across the board** | ⚠️ **Newly open.** *Bellator* became **Champion** on 2026-09-11. The old name was accurate — Latin for a combatant, and the class descends from the old cities' duelling champions — but it was the only Latin name on a roster of plain English ones and read as belonging to a different game. That is a reason to look at all six rather than one. Bulwark, Elementalist, Blood mage and Dual mage are *descriptions*; Shadow Reaver and Champion are *titles*. Worth deciding which register the roster is in before any of them reach a player |
| **Arena size and shape** | Determines whether a space-denying class can corner anyone, and whether block pushback has teeth. ⚠️ **Since 2026-09-26: the walls are lower than a jump** (1.5 m against a 2.7 m lowest hop), so a fighter can leave the arena — the sparring bot found it. See [sparring.md](sparring.md) |
| **Frame counts and damage** | Absent everywhere on purpose. Needs a prototype, not a guess |
| **The repeat lockout's number** | ⚠️ **Newly open.** 30 frames is a first guess. Which abilities want a multiplier and which way is the other half, and both need somebody to play it — the knobs are in the Oven under `Offence` and in each move's `Repeat lockout (%)`. Whether a reactivation wants gating at all (`Move::reactivate`, zero everywhere) is the third |
| **`M` and `LR` reliability** | `M` carries the Dual mage's Lance since 2026-09-16 — a cast she throws every exchange rather than a finisher, which sharpens the question. `LR` is still unspent. Both are the slowest inputs on most mice |
| **Move + heavy attack** | ⚠️ **Half-answered, 2026-09-16.** The collision is gone: shift is only a dodge now, so holding a direction and clicking throws the attack. What is left open is **where the heavies go** — three classes have a committed move with no button at all until each is given one, one kit at a time. See [controls.md](controls.md#shift-is-one-verb-now-2026-09-16) |
| **How many inputs the hands have** | ⚠️ **Explored 2026-09-29**, not decided. Counted by finger and by what a press costs rather than by key, the immediate ability inputs are five today, nine with `F`, `R` and the two mouse side buttons, fourteen with alt as a modifier that has no verb of its own. The hand is not the limit; readability is. See [exploration/0001](exploration/0001_control_budget.md) |
| **Differentiating move+attack** | ⚠️ **Open, and tighter still.** Dodge moving onto shift ended "shift beats WASD", and shift ceasing to modify clicks on 2026-09-16 took the modifier away entirely. Directional attacks (`w`/`a`/`d`/`s` + click) and the third click are what is left |
| **Aerials** | **Settled on two classes, open on four.** Airborne attacks should be *variants of their grounded counterparts* rather than a separate move list — same identity, different frame data. That is how the Champion is built (the button is the weapon and the row of its grid is the situation) and, since 2026-09-14, how the Elementalist is: left click is still the cheap shot, right click is still the committed one, `E` is still earth, and the row is where her feet are. A claim about one class was a coincidence; two is a pattern, and the four that are left are now behind rather than undecided |
| **Attack strings** | ⚠️ **Newly open, 2026-09-14.** The Champion's ground attacks now chain three hits deep, and the chain is a *hit confirm* — a connected link cancels its own tail, a blocked one does not. Whether that is a Champion mechanic or the shape every class's offence should take is not decided, and it is the sort of thing that has to be one or the other |
| **`space` as a modifier** | ⚠️ **Newly open, 2026-09-14, and wider since 2026-09-15.** `space` plus a weapon is a takeoff on the Champion — the first time the jump button has modified anything. It is a whole row of options every class could have, or a precedent that should not spread. It now has a second, different form: `space` pressed *during* the hammer's finisher banks a leap that is spent when the move lands, which is the only place in the game a button is read while you are not free to act. See [controls.md](controls.md) §"Open questions" |
| **A move that carries the body** | ⚠️ **Newly open, 2026-09-15.** `Move::step` is a distance an attack moves you down its own locked facing, added to whatever the stick asks for and finished by the frame its hitbox appears. Six of the Champion's nine chain links use it and nothing else in the roster does. It is the cleanest answer yet to "what makes two attacks at the same range feel different" — *where you are standing when it is over* — and it is available to every class for free. Whether the other five should reach for it, or whether it is the Champion's texture, is not decided |
| **Neutral shift** | Shift with no direction and no click does nothing. A spot dodge in place is the obvious candidate |
| **Double jump** | **Answered for one class, 2026-09-23.** The Dual mage jumps once more in the air while the lower of her two bars holds three quarters, and on every press while she is ascending — earned by the mechanic rather than given to the roster. For everyone else space while airborne still does nothing, and the airdodge is the only air commitment. Whether a second jump wants to spread is now a question with an example to play |
| Dual mage | **Built on two bars, 2026-09-23, and nothing of it played.** The open questions are in [dual-mage.md](dual-mage.md): the calm is pulled two ways between a fast climb and a fragile plateau; whether the band's cadence (one cast stays level, two do not) is right; whether the burn is a consequence or a footnote at a tenth of what it was; whether the refund should count hits or damage; the names of the two beings; whether the float (her feet leave the floor and she walks faster, from 2026-09-17) belongs on the three-quarter tier where it was rehomed or at half with the blink. Still open from before: whether a four-to-one spread on the depth curve is right, and whether "the form is the arm you last punched with" is findable. The rest of her movement is independent of the bars — a 1.3 m step on every auto, forward on the dark one and backward on the light one, and the longest aerial hang in the game — and the numbers to watch there are the step and whether the light auto's backward version feels like being pushed around by your own attack. The play scripts for the four human checkpoints are in the feel log for that date |
| Bulwark | ⚠️ **v2 is built and unplayed** — [bulwark-v2.md](bulwark-v2.md): the shield stores what it blocks, Slam and the throw spend it, and a planted shield is a wall sized by it. Measured end to end ([feel log](feel-log.md)); nobody has played it. If it does not answer "why a shield alone" once played, the fallback is a different class. Open from the build: a Bulwark standing at a foot with the guard up deals twice the damage and dies in half the time, to the unblockable rear-and-slam. Also: possibly a seventh slot for a dedicated ally-cover stance |
| Champion | Whether the mid-animation swap costs Rush — and, since the chain, whether it is still worth building at all. Also: how long a string should survive without a hit (26 frames is a guess), and whether swapping weapons mid-string should flow faster than repeating one at all. **Since 2026-09-15**, five more, all of them in [feel-log.md](feel-log.md): whether the sword's step is too much free pressure, whether the spear's sixteen-frame second hit reads as a two-part move from across the arena, whether "jump into the finisher" occurs to anybody without being taught, whether +14 on hit is too much, and whether the spinning finisher's knockback fights the chain it ends. **Since 2026-09-26** the weapons are told apart by weight as well as shape — an impact freeze on every blow (the hammer sticks, the sword barely catches), knockback you can see, and a slower, harder-hitting hammer — and the new questions are whether a three-second hammer string is too slow and whether thirteen frames of freeze on Earthbreaker feels like weight or like lag; see [feel-log.md](feel-log.md) |
| Shadow Reaver | ⚠️ **v2 is built and unplayed** — [shadow-reaver-v2.md](shadow-reaver-v2.md): the shadow marks from range and any hit of hers cashes the marks. Every number in the tally is a first guess, and the strike out of the carry was added in the building and wants a person's word. Still open: whether the shadow has collision. And **where Deadly mistake goes** — it is the only ability in the kit with no input, and both obvious modifiers are already swallowed. Since 2026-09-17 the leash is twice the throw rather than a third longer, so a placed shadow keeps its place long enough to be a decision; whether eighteen metres of slack is too much room is the new question. Since 2026-09-23 the dash stops dead on the shadow. **Since 2026-10-04 the dash jump keeps all of the dash's speed again** (it kept a fifth from 2026-09-23, which made it an ordinary jump), bled per frame of the carry so the first frame goes furthest, and a press in the dash's last six frames is kept for the arrival: about 50 m past a shadow sent 9 m |
| Elementalist | ⚠️ **v2 is built and unplayed, 2026-09-30** — [elementalist-v2.md](elementalist-v2.md), plan in [plans/elementalist-v2.md](plans/elementalist-v2.md). The four new inputs, the two charges, Updraft/Downdraft, Cinder spray, Quake/Tremor, lit stones and the break-through are in; Blast, Hover and the fire Trail wait on a word. **The plan's stop condition is the first thing to test**: if a full Strike does not feel earned at the nine metres it costs, that is a new document rather than a knob. The five numbers to play first are in [kits/elementalist.md](kits/elementalist.md) §"Open questions". The **double structure jump** — two stones two or three frames apart, jumped while both are still erupting — is the most interesting thing the class does, and since 2026-09-18 it is specified and tested rather than merely allowed. **⚠️ Its height is open and the stone knobs are not the way to move it.** They were tuned down on 2026-09-17 and put straight back: the chain is a *resonance* between how fast she rises and how fast the stone grows, so three frames on the rise halved the double while barely touching the single, and raising her jump makes the single **fall**. Everything there is discontinuous and some of it is non-monotonic — [feel-log.md](feel-log.md) has the map. What the cast-wide jump nerf leaves, with the stones untouched, is a double at 44.8 m against its old 58.8 and a single at 25.9 against 17.5. Structure cap of three is a readability guess, not a balance one — and **Landfall is a second way to spend it**, so it is under more pressure than when the guess was made. Stones are solid and standable, and Raise now places one where the crosshair is; the mobility that implies waits on moves that launch them. Her air row is built and none of its numbers have been played: the three to watch are in [kits/elementalist.md](kits/elementalist.md) §"Open questions" |
| Blood mage | ⚠️ **The v1 rebuild is built and nobody has played it.** [kits/blood-mage.md](kits/blood-mage.md) §"Open questions" carries the proposal's questions with what the build found beside each; [plans/blood-mage-v1.md](plans/blood-mage-v1.md) has the four play scripts (C1–C4) that answer them. The numbers to watch first: the grey fade (12 a second), the reach at full grey (×1.5), and the pool drain (10 a second), which is the counterplay knob. Two things the build settled on its own and the person should overrule if wrong: the `leech` column stays in the move table because the Dual mage's dark arm reads it, and a hit on somebody in the air spills no pool. Her movement: the pool blink is built and always on; beside it are two live Oven flags from 2026-09-17 — the Grasp hauling her to a wall, structure or the creature it caught instead of a person (on), and her dodge as a flat blink (off) — see [kits/blood-mage.md](kits/blood-mage.md) §Movement |

## 5 · Parked — not slated for initial implementation

**Progression and equipment.** See [parked.md](parked.md). The classes are the product; a
build system has nothing to modify until they exist.

The leading proposal is **offensive items as auto-attack modifiers** — mild buffs that change
playstyle rather than power, letting a class spec toward offence, defence or utility without
changing identity, with unlocks forming the early ramp. Its open risk is that some modifiers
may be too central to be optional.

Two conclusions there should only be reopened deliberately: vertical character power is
corrosive to a skill-gates-content thesis, and competitive versus is incompatible with
character progression.

## 6 · Implementation

Rust, eight crates, simulation as a pure function. See
[architecture.md](architecture.md). All six classes have their mechanic and at
least three exemplar moves -- nineteen on the Champion, seven on the
Elementalist, six on the Dual mage, five on the Blood mage and four on the
Shadow Reaver -- there is a monster to fight and
climb, peer-to-peer rollback play works over real UDP, and the test suite covers
determinism, combat relationships, aiming, the ride, the camera, kinematics,
animation and the frame budget.

```
crates/sim    Deterministic simulation. Zero deps, no floating point.
crates/net    Rollback session (GGRS) + headless soak.
crates/view   Interpolation, the follow camera, posing. No engine dependency.
crates/game   Bevy app. Rendering only.
crates/anim   Animation factory: recipes, the solver, contact sheets. See animation.md.
crates/hunt   A scripted hunter (a plan per creature, a class layer for every class) and the report that judges it; the sparring bot.
crates/manual Every command, key and flag. No dependencies, so help is instant.
crates/web    The browser: the playable page, and the frame-data tool.
```

**Requires Rust 1.85+** (Bevy 0.16's MSRV). `rustup update` if Cargo complains about
`edition2024` -- that error names the symptom, not the cause.

**Working on it:** `./scripts/dev.sh` — hitbox wireframes, the Oven, and a class picker beside each health bar. Extra
arguments pass through, so `./scripts/dev.sh --p1 champion` works.

**What can I type?** `./scripts/help.sh`, or `cargo run -p game -- --help`. Every command,
key, flag and environment variable, and the browser build's controls panel is generated from
the same tables.

**Run it:** `cargo run -p game` — 3D arena, standins, HUD with live frame data, debug
overlay on F1 (hitbox and hurtbox wireframes, guard arcs), local two-player, training dummy on 1-4, a sparring bot on 5-7 ([sparring.md](sparring.md)). Click to capture the mouse, Escape
to release. `DEMO=1` scripts player one and `DEBUG_OVERLAY=1` starts with the overlay on;
`./scripts/screenshot.sh` renders headlessly (`./scripts/setup-tools.sh shot` installs what it needs; `web` does the same for the browser build).

**Controls are camera-relative.** The mouse aims; `W` is away from the camera, not along a
world axis; attacks go where you look. Facing locks the moment a move starts, so you commit
to a direction when you commit to the move. See [controls.md](controls.md).

**Camera settings** save to `~/.config/arena/settings.conf` as you change them: sensitivity on
`-` / `=` and field of view on `F3` / `F4`. Set `ARENA_SETTINGS` to keep separate settings per
person on a shared machine. **Camera distance is not one of them** — it is the framing
sphere's radius, the radius decides where the eye is, and the eye is where the aiming ray
starts, so it is tuned in the Oven under **Camera** rather than set per player. See
[architecture.md](architecture.md).

**Send it to somebody:** `./crates/web/build-game.sh` compiles the whole thing to
WebAssembly and writes `target/web`, which is what GitHub Pages serves — the same
simulation and the same renderer, on a canvas, one player against the training
dummy. A browser cannot open a UDP socket, so peer-to-peer stays on the desktop;
the query string does what the flags do, so `?p1=champion&dev` is
`--p1 champion --dev`. See [web.md](web.md).

**Play someone:** `game --port 47811 --peer <their-ip>:47812`. Rollback netcode, no server.
`./scripts/p2p-localhost.sh` runs both ends locally;
`cargo run -p net --bin p2p_localhost` checks two peers stay in sync over real UDP.

**Pick classes:** `game --p1 champion --p2 elementalist`, or Tab to cycle in-game.

**Tune frame data:** `cargo run -p sim --bin frametable` prints every move's on-block and
on-hit advantage, and the repeat lockout beside them. `cargo run -p sim --bin essence` is the
Blood mage's instrument: the pool each move leaves, what each drinks, the grey bar over a
scripted exchange, and the scythe's reach at each level of grey. `./crates/web/build-sandbox.sh` writes a self-contained HTML file with
hitbox overlays and frame stepping.

**Tune it while it runs:** **F7** opens the Oven — every tuned number in the game, grouped by
family and searchable, adjusting live. The bake button writes them to `crates/sim/src/tuned.rs`
and pushes on the current branch, so a tuning session ends as a reviewable diff.
`cargo run -p sim --bin bake_tuning` does the same without launching the game.

**Animation:** `cargo run -p anim --bin bake` regenerates the fighters' baked clips from the
recipes in `crates/anim/src/clips/`. F2 toggles baked playback in-game. The Ridgeback has its
own rig and its own bake -- `cargo run -p anim --bin bake_beast`, recipes in
`crates/anim/src/beast/` -- because its parts are simulation geometry rather than
presentation. See [animation.md](animation.md) §"The creature has its own rig".

**The creature's geometry:** `cargo run -p sim --bin beastcheck` prints how high everything you
can stand on is, in every state that lowers one, against how high a fighter can actually jump.
The climb is a geometry problem and this is the geometry. `--species <name>` for any creature:
it reads everything off the species table.

**Small bodies:** `cargo run -p sim --bin critcheck` stands a critter at 1, 2, 3 and 5 m in
front of every class, puts the crosshair on it, presses every move, and prints where it touched
beside where the same move touches a fighter. `--species <pack> --kind <name>` for any pack. See
[critters.md](critters.md). `cargo run -p sim --bin bake_tuning -- --set <id>=<value>` sets a
knob from a terminal and bakes.

## 7 · The feel harness

Tuning happens in fragments over a long time, so the results have to outlive the
session that found them.

- **[feel-log.md](feel-log.md)** — what was changed, why, and what it actually felt
  like. Reverted experiments are the most valuable entries; keep them.
- **`crates/sim/src/tuning.rs`** — every feel number in one place, each with a
  comment saying where it came from.
- **`crates/sim/tests/feel.rs`** — pins the *relationships* that must hold no matter
  how the numbers move: every attack is punishable on block, every class can beat a
  turtle, a parry pays for itself, the dodge outruns a walk. These found six real
  design gaps on the day they were written.

A number is a guess until someone plays against it. A relationship is a design
decision, and belongs in a test.

## 8 · Next

1. **Review the bestiary, and play it.** Eleven creatures, their arenas, trophies and tempers
   are built and nobody has played them: [review.md](review.md) is the way through, one command
   per fight and every open question by ID. Answering the `C` items (decisions made on your
   behalf) first unblocks the most: health changed from six documents, rules changed in every
   fight, the dev species out of the cycle.
2. **The class-against-creature findings** (`CLASS-1` to `CLASS-5`): the fire pillar, the Blood
   mage's kit, the Dual mage's tempo, the Reaver's Mantis, the thrown shield. Each is a decision
   about a class or about every creature at once, and the harness cannot make it.
3. **Play it against a person.** Everything else is downstream of that — and the creatures need
   it twice over: every fight here was tuned by a scripted hunter with a quarter-second
   reaction. The browser build exists to make the asking cheap: a link instead of a clone,
   [web.md](web.md), and `?hunt=<creature>` opens any fight.
4. Answer the open questions in [feel-log.md](feel-log.md) — the flagged one is whether the
   4-frame parry window is findable by a human.
5. **The world, past W2**: a menu for the picker, the hunter's notes, and — only if wanted —
   the valley ([world.md](world.md) §2–§3).
6. Fill out the kits beyond three moves per class.
7. **Arena size and shape** for versus.
