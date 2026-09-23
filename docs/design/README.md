# Design — current state

Everything currently decided, proposed, or parked, in one place. This supersedes
[`../archive/`](../archive/README.md), which is 2016–2019 source material kept for reference.

**Start here:** [combat kernel](combat-kernel.md) → [controls](controls.md) →
[ability spec](ability-spec.md) → a class kit. For implementation, see
[architecture](architecture.md); before touching anything that is *pointed at
something*, see [aiming](aiming.md).

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
the skill ceiling is. Aerials suspend the fall for a per-move number of frames.

**Roster.** Six classes. The Gatekeeper is retired and not backfilled — a missing long-range
poke is a design choice in a closed arena, not a gap.

## 2 · The roster

| Class | Mechanic — what abilities spend | Primary buttons | State |
| --- | --- | --- | --- |
| [Shadow Reaver](kits/shadow-reaver.md) | Shadow position (always placed) | `L` auto · `R` Send shadow · `Q` Guillotine lotus · `E` Executioner | Rebuilt |
| [Elementalist](kits/elementalist.md) | Structure slots (cap 3) | `L` beam auto · `R` Cataclysm · `Q` Fire pillar · `E` Raise · **and the same three, airborne** | Strong |
| [Blood mage](kits/blood-mage.md) | Grey health, and essence pools where she cut somebody | `L` Reaping sweep · `R` Haemorrhage · `M` Bloodletter · `Q` Grasp · `E` Black spike · `shift` on a pool = Blink | **v1 built**, unplayed; [the kit](blood-mage.md) |
| [Dual mage](kits/dual-mage.md) | Meter position — **two bars, proposed** | `L` dark auto (pulls) · `R` light auto (pushes) · `M` Lance, two forms · `Q` Judgement · `E` Sweep | **Core rebuilt**; [v2 proposed](dual-mage-v2.md) |
| [Champion](kits/champion.md) | Rush charge (one, cancels recoveries) | `L`/`M`/`R` = sword/hammer/spear, three hits deep · `space` + weapon = takeoff · `E` Rush | Shaped |
| [Bulwark](kits/bulwark.md) | Shield position | `L` Bash · `shift`+`L` Slam · `Q` Grapple · `R` Guard · `E` Throw/Recall/leap | New |
| ~~Gatekeeper~~ | — | — | Retired |

*Dual mage was Statera. Champion was Bellator, and Shifter before that.*

Each kit is **six abilities plus an auto and the mechanic input** — enough for a real match,
few enough to balance and to read in third person.

## 3 · Documents

| Document | Covers | Status |
| --- | --- | --- |
| [combat-kernel.md](combat-kernel.md) | No cooldowns, TTK, what that breaks | Decided |
| [controls.md](controls.md) | Input grammar, per-class schemes | Proposed |
| [ability-spec.md](ability-spec.md) | The format kits are written in | Proposed |
| [aiming.md](aiming.md) | The one raycast, and the two kinds of skillshot | Decided |
| [defense.md](defense.md) | Dodge, block, parry, guard breaks | Proposed |
| [blood-mage.md](blood-mage.md) | v1 kit: grey health, essence pools, the scythe, the blink | **Built 2026-09-23**, unplayed |
| [dual-mage.md](dual-mage.md) | The two-pole meter, the depth curve and ascension | Decided; v2 proposed |
| [dual-mage-v2.md](dual-mage-v2.md) | v2: two bars, the hill between them, the tiers, wings | **Proposed** |
| [plans/](plans/) | Action plans for implementation threads: [Blood mage v1](plans/blood-mage-v1.md), [Dual mage v2](plans/dual-mage-v2.md) | Briefs |
| [champion.md](champion.md) | Forms, the three-hit chain, and the mid-animation swap | Decided |
| [bulwark.md](bulwark.md) | Why the class exists; shield as volume | Proposed |
| [elementalist.md](elementalist.md) | Structure interaction in versus | Decided |
| [gatekeeper-retirement.md](gatekeeper-retirement.md) | Why it was cut, what was salvaged | Decided |
| [monsters.md](monsters.md) | The Ridgeback: the climb, the ride, the control algorithm, measuring the fight | Proposed, rebuilt |
| [architecture.md](architecture.md) | Rust workspace, determinism, rollback | Decided |
| [web.md](web.md) | The browser build: what a page cannot do, and what it does instead | Decided |
| [animation.md](animation.md) | The skeleton, authoring clips, the hub | Decided |
| [parked.md](parked.md) | Progression and equipment | **Parked** |

## 4 · Open

Nothing here blocks a prototype.

| Item | Question |
| --- | --- |
| **Class names, across the board** | ⚠️ **Newly open.** *Bellator* became **Champion** on 2026-09-11. The old name was accurate — Latin for a combatant, and the class descends from the old cities' duelling champions — but it was the only Latin name on a roster of plain English ones and read as belonging to a different game. That is a reason to look at all six rather than one. Bulwark, Elementalist, Blood mage and Dual mage are *descriptions*; Shadow Reaver and Champion are *titles*. Worth deciding which register the roster is in before any of them reach a player |
| **Arena size and shape** | Determines whether a space-denying class can corner anyone, and whether block pushback has teeth |
| **Frame counts and damage** | Absent everywhere on purpose. Needs a prototype, not a guess |
| **The repeat lockout's number** | ⚠️ **Newly open.** 30 frames is a first guess. Which abilities want a multiplier and which way is the other half, and both need somebody to play it — the knobs are in the Oven under `Offence` and in each move's `Repeat lockout (%)`. Whether a reactivation wants gating at all (`Move::reactivate`, zero everywhere) is the third |
| **`M` and `LR` reliability** | `M` carries the Dual mage's Lance since 2026-09-16 — a cast she throws every exchange rather than a finisher, which sharpens the question. `LR` is still unspent. Both are the slowest inputs on most mice |
| **Move + heavy attack** | ⚠️ **Half-answered, 2026-09-16.** The collision is gone: shift is only a dodge now, so holding a direction and clicking throws the attack. What is left open is **where the heavies go** — three classes have a committed move with no button at all until each is given one, one kit at a time. See [controls.md](controls.md#shift-is-one-verb-now-2026-09-16) |
| **Differentiating move+attack** | ⚠️ **Open, and tighter still.** Dodge moving onto shift ended "shift beats WASD", and shift ceasing to modify clicks on 2026-09-16 took the modifier away entirely. Directional attacks (`w`/`a`/`d`/`s` + click) and the third click are what is left |
| **Aerials** | **Settled on two classes, open on four.** Airborne attacks should be *variants of their grounded counterparts* rather than a separate move list — same identity, different frame data. That is how the Champion is built (the button is the weapon and the row of its grid is the situation) and, since 2026-09-14, how the Elementalist is: left click is still the cheap shot, right click is still the committed one, `E` is still earth, and the row is where her feet are. A claim about one class was a coincidence; two is a pattern, and the four that are left are now behind rather than undecided |
| **Attack strings** | ⚠️ **Newly open, 2026-09-14.** The Champion's ground attacks now chain three hits deep, and the chain is a *hit confirm* — a connected link cancels its own tail, a blocked one does not. Whether that is a Champion mechanic or the shape every class's offence should take is not decided, and it is the sort of thing that has to be one or the other |
| **`space` as a modifier** | ⚠️ **Newly open, 2026-09-14, and wider since 2026-09-15.** `space` plus a weapon is a takeoff on the Champion — the first time the jump button has modified anything. It is a whole row of options every class could have, or a precedent that should not spread. It now has a second, different form: `space` pressed *during* the hammer's finisher banks a leap that is spent when the move lands, which is the only place in the game a button is read while you are not free to act. See [controls.md](controls.md) §"Open questions" |
| **A move that carries the body** | ⚠️ **Newly open, 2026-09-15.** `Move::step` is a distance an attack moves you down its own locked facing, added to whatever the stick asks for and finished by the frame its hitbox appears. Six of the Champion's nine chain links use it and nothing else in the roster does. It is the cleanest answer yet to "what makes two attacks at the same range feel different" — *where you are standing when it is over* — and it is available to every class for free. Whether the other five should reach for it, or whether it is the Champion's texture, is not decided |
| **Neutral shift** | Shift with no direction and no click does nothing. A spot dodge in place is the obvious candidate |
| **Double jump** | Space while airborne does nothing. The airdodge is currently the only air commitment. ⚠️ **Sharper since 2026-09-14:** `E` off the floor used to raise a stone, which made a stone under your own feet a sort of second jump for one class. It is Landfall now, so the Elementalist has lost the only thing in the game that was answering this question by accident |
| Dual mage | ⚠️ **A v2 of the mechanic is proposed** — [dual-mage-v2.md](dual-mage-v2.md): two bars and a runaway between them, which answers the depth-curve questions below by replacing the curve's axis. Until it is built: naming the two forces. Ascension drain, refund, threshold and stun numbers. **Since 2026-09-16** the core is rebuilt — depth scales everything, the autos pull and push, Lance is two moves on middle click and Judgement earns its status through power — so the open questions moved with it: whether a four-to-one spread between the centre and the edge is the right one, whether "the form is the arm you last punched with" is findable by anybody who was not told, and whether a Judgement at full depth is too much of a health bar. See [kits/dual-mage.md](kits/dual-mage.md) |
| Bulwark | Possibly a seventh slot for a dedicated ally-cover stance |
| Champion | Whether the mid-animation swap costs Rush — and, since the chain, whether it is still worth building at all. Also: how long a string should survive without a hit (26 frames is a guess), and whether swapping weapons mid-string should flow faster than repeating one at all. **Since 2026-09-15**, five more, all of them in [feel-log.md](feel-log.md): whether the sword's step is too much free pressure, whether the spear's sixteen-frame second hit reads as a two-part move from across the arena, whether "jump into the finisher" occurs to anybody without being taught, whether +14 on hit is too much, and whether the spinning finisher's knockback fights the chain it ends |
| Shadow Reaver | Whether the shadow has collision. And **where Deadly mistake goes** — it is the only ability in the kit with no input, and both obvious modifiers are already swallowed |
| Elementalist | Structure cap of three is a readability guess, not a balance one — and **Landfall is a second way to spend it**, so it is under more pressure than when the guess was made. Stones are solid and standable, and Raise now places one where the crosshair is; the mobility that implies waits on moves that launch them. Her air row is built and none of its numbers have been played: the three to watch are in [kits/elementalist.md](kits/elementalist.md) §"Open questions" |
| Blood mage | ⚠️ **The v1 rebuild is built and nobody has played it.** [kits/blood-mage.md](kits/blood-mage.md) §"Open questions" carries the proposal's questions with what the build found beside each; [plans/blood-mage-v1.md](plans/blood-mage-v1.md) has the four play scripts (C1–C4) that answer them. The numbers to watch first: the grey fade (12 a second), the reach at full grey (×1.5), and the pool drain (10 a second), which is the counterplay knob. Two things the build settled on its own and the person should overrule if wrong: the `leech` column stays in the move table because the Dual mage's dark arm reads it, and a hit on somebody in the air spills no pool |

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
crates/hunt   A scripted hunter, and the report that judges the fight it plays.
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
overlay on F1 (hitbox and hurtbox wireframes, guard arcs), local two-player, training dummy on 1-4. Click to capture the mouse, Escape
to release. `DEMO=1` scripts player one and `DEBUG_OVERLAY=1` starts with the overlay on;
`./scripts/screenshot.sh` renders headlessly.

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
The climb is a geometry problem and this is the geometry.

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

1. **Play it against a person.** Everything else is downstream of that — and the
   Ridgeback needs it twice over. The browser build exists to make the asking
   cheap: a link instead of a clone, [web.md](web.md). The climb now costs something to reach, and
   whether the reward is worth the trip, whether anyone finds the tail hop, and
   whether the ground game reads as a phase or as a toll are not things the
   harness can answer. See [monsters.md](monsters.md) §9.
2. Answer the open questions in [feel-log.md](feel-log.md) — the flagged one is
   whether the 4-frame parry window is findable by a human.
3. Fill out the kits beyond three moves per class.
4. **Arena size and shape.**
