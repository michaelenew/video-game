# Working in this repository

Read [`docs/design/README.md`](docs/design/README.md) first — it is the map. The
design documents are the specification; the code is meant to match them, and
when it does not, one of the two is wrong and both get fixed.

Below are the rules that are load-bearing enough that breaking them has cost
real time more than once. Each has a test that enforces it, named in the rule.

## Aiming: there is one model, and you do not write a second one

**Everything that decides where an ability goes lives in
[`crates/sim/src/aim.rs`](crates/sim/src/aim.rs). Everything that intersects a
ray with a shape lives in `crates/sim/src/math.rs`. Nothing else in the
simulation may do either.** Enforced by `crates/sim/tests/one_aim.rs`.

The player's whole frame of reference is the crosshair, so the model is:

> One raycast, from the **camera** through the crosshair, ignoring anything
> behind the character model. It meets terrain, structures, and the ability's
> own max-range sphere. The first thing it reaches is what the player is
> pointing at, and the ability goes there.

**Bodies are not on that ray** — not other fighters, not the creature. It is
asking which *place* is under the crosshair, and a body is a thing standing in a
place, so the ray goes through it to the geometry behind. What a shot runs into
is a separate question, asked along the ability's own path by
`aim::first_along`. The creature is why: up close it fills the screen, the
reticle lands on its chest three metres up, and with it on the ray every
skillshot came out as a metre-long stub pointed at the sky.

Two kinds of skillshot start with that ray. Three more lines of effect do not —
they are pointed by something the player decided earlier (Racing borrows the
ray for a direction, but its start and its length were decided already). **Five in total, and
every one of them is a function in `aim.rs`:**

| Kind | Call | Rule |
| --- | --- | --- |
| Grounded | `aim::grounded_path` | Ground: cast exactly there. Max range: max range on the ground in the mouse's direction. If it travels, it travels from the character to that point. **No structure is ground**: the ray goes through the Elementalist's stones to the floor, for every placement including Raise. A stone under the crosshair must never move where something lands. |
| Skillshot | `aim::skillshot_path` | Ground: that spot raised to the **middle of whatever stands on it** (`aim::standing_middle`, with the height from `aim::stands_at` — a fighter's, unless the crosshair passed through something shorter), because the floor is never the target — bodies are not on the ray, so a ground hit means "there". Anything else — wall, stone (its top too: `aim::sight_for_attack`), body, monster, range sphere — the point of intersection exactly. Straight line from the caster, and that line is its whole reach. |
| Swing | `aim::swing_path` | A body moving: no raycast, reach off the body. Yaw is `facing`; pitch follows the camera, **with a dead zone while standing** — level through the first 45° below the horizon, exact above it, and the leftover past it. The camera sits above the shoulder, so looking at somebody at your own height is looking slightly down at them. In the air there is no shared floor to read that way, so the pitch is followed exactly. Standing, pointed at something **shorter than a fighter** (`aim::stands_at`), it dips to meet it at the same share of its height a level swing meets a fighter at (`aim::stoop`); zero where only fighters stand. A **one-armed** move leaves from that shoulder rather than the chest: `Move::hand`, declared in the table beside the shape, and `aim::across` is the only thing that turns it into a direction. |
| At the mechanic | `aim::mechanic_path` | Where the class mechanic is standing. The player aimed when they placed it. Guillotine lotus only. |
| Racing | `aim::racing_path` | From the stone the Elementalist is holding churning (her own feet if there is none), **flat toward the crosshair's spot on the ground** (the point `grounded_path` would land on), as far as the hold bought. The ray chooses the direction only: the place was aimed when the stone was raised, and the hold is the distance. Fissure, and the Air ball, which rolls the same way from where she raised it (its hold buys size, so only the direction is read). |

The rest of `aim.rs` -- twenty-seven functions as of 2026-10-02 -- are **not**
lines of effect. `aim::sight` and `aim::sight_for_attack` are the one raycast
itself, `aim::origin` and `aim::hand_origin` where a cast leaves the body, and
`aim::first_along` what a path runs into. `aim::pointing_at`
answers *is the crosshair on that thing*, which the Reaver's forward dodge asks
about her shadow. It points nothing anywhere, but it is built from the eye and
the look direction, so it belongs with the rest of them — the alternative is an
angle worked out beside the ability, which is the mistake below wearing a hat.
`aim::pointing_at_disc` asks the same of a disc on the floor, which is the Blood mage's blink
asking about one of her pools; a puddle is not a body's column, so it gets its own shape.
`aim::clear_between` answers *is there any straight line from this body to that
one*, which the same dodge asks second: the dash crosses to wherever the shadow
is unless nothing reaches it. It is ray-against-shape work in service of a
decision about where something goes, so it belongs here rather than beside the
dodge for exactly the same reason. `aim::planted_ahead` answers *where does a
thing go that nobody aimed* — the slab the Elementalist's Landfall drives out of
the floor, a fixed distance along her flat facing, because she is arriving rather
than pointing. Its move still declares a line of effect (a swing, for the slam
itself); this is the second thing the same move puts in the world, and written
beside the ability it would be a facing, a distance and a floor query sitting
next to a move, which is the mistake below in its usual clothes.
`aim::shadow_faces` and `aim::copied_swing` answer *which way is forward for a
copy thrown from somewhere else*: the Reaver's shadow out on the field turns her
swing to the nearest body in reach, keeping her pitch. The copy is still a
swing; this is only its yaw. `aim::look_onto` is the raycast run backwards —
*which pitch puts the crosshair on that point* — for anything that plays the
game rather than referees it: the sparring bot, and the Elementalist rehearsal.
A bot that aimed from its chest would be the mistake below, made by a player.
`aim::look_onto_closely` is the same, settled for as many rounds as a steep look
needs (`critcheck`, and the dev pack's hunter); the bot keeps its six.
`aim::stands_at` answers *how tall is the thing under the crosshair* — the last
body the ray passed through, which never stops it — and `aim::stoop` turns that
into how far below the shoulder a standing swing meets it: what makes a
knee-high critter hittable (bestiary A1, [`docs/design/critters.md`](docs/design/critters.md)).
`aim::line_clear` answers *is anything solid on this straight line*, which a pack
asks when it cuts a ring round a fighter. Critters are on `aim::first_along`'s
list with the creatures (A2). `aim::sight_clear` answers *can something at this
point see that one* -- `line_clear`, and no floor hazard that blocks sight
(smoke) -- which a creature's perception filter asks from its head
(`crate::perception`, bestiary P5). `aim::in_view` and `aim::in_view_of` (A5)
answer *is that point on this fighter's screen and not behind anything*: a cone
round the look, from the eye, then `sight_clear` -- the Veilstalker asks so it
never reveals itself off-screen. `aim::on_screen` is that cone alone, with
nothing asked about what stands in front -- for what the renderer draws over
everything, the floor markers (the Galewing's report). `aim::in_view_from` and
`aim::off_look` ask the same of a look a creature *remembers* -- where you stood
and which way you faced, a glance old -- and how far off it a point is: the
Veilstalker's decloak, and the report's thirds of the screen. A sight test
written beside a creature's brain would be the mistake below with the roles
swapped. `aim::underfoot_up` answers *which way is up for what a fighter stands
on* -- `+y` on the floor, the mounted part's own on a creature -- which is what
`swing_path`'s dead zone is measured against (bestiary A4, the Galewing).
`aim::blink_to` answers *where does a body sent along the floor stop* -- the
Dual mage's blink, short of the first thing its feet meet -- and `aim::settle`
*what does this point stand on*. `aim::standable` asks *is that footing* (not a
course's drop), and `aim::footing_toward` *where nearby is*, if the aimed spot
is not: the Reaver's send scans a little way back toward her and is refused if
nothing is there. **Every eye `aim.rs` starts
from is `camera::eye_under`**: the eye held under a cave's vault, which the
drawn camera starts from too; with no ceiling overhead it is `camera::eye`.

Which one a move is comes from `Move::aim()`, **declared** in the move table so
every move has an answer, and printed in the `aimed` column of
`cargo run -p sim --bin frametable`. Inferring it from what a move leaves behind
is what let Fissure — a skillshot that races along the ground, by its own kit
entry — come out as a bubble seven metres in front of the body.

**The mistake this prevents, which has been made three times:** taking a ray
from the *chest* along the *look angle*. That ray is parallel to the crosshair's
and never converges with it, so the reticle sits on one thing and the ability
goes past it — by more the further away it is. If you find yourself writing
`camera::eye(...)`, `camera::eye_under(...)`, `look_dir()`, or a ray-vs-shape call outside those two
files, stop: the thing you want already exists.

If none of the five fits a new ability, **change `aim.rs`** rather than working
around it. A change there is true of every ability at once, which is the point.
The full specification is [`docs/design/aiming.md`](docs/design/aiming.md).

## The valley is one map: ask the terrain, never walk a table

Since 2026-10-09 the valley is one map of eighteen places
([`docs/design/atlas.md`](docs/design/atlas.md)), indexed in 16 m tiles by
[`crates/sim/src/atlas.rs`](crates/sim/src/atlas.rs). **Any question about
the ground goes through `arena::Terrain`** -- `World::terrain()`, or
`scene.arena` -- and asks for the boxes near something: `near`, `around`,
`along`, or a query that already does (`resolve`, `ground_under`,
`relief_at`). Walking `arena.solids` yourself reads one place's own table and
nothing of its neighbours, so on the map it is wrong at every doorway;
`Terrain::solids()` is every box round the whole place and is for planning,
not for a body or a ray, because it is the expensive one.

Two more things that follow from it:

- **The world runs in the coordinates of the place it is in**, and moves into
  the next place's when the fighters cross (`sim::valley::open`). Every
  position the snapshot keeps is shifted by `World::shift`. **A new field that
  holds a position goes into `shift`**, or the first crossing leaves it a
  reach away; `tests/valley.rs` (`a_change_of_frame_changes_nothing`) is what
  catches it.
- **The renderer draws the map by distance** (`crates/game/src/stream.rs`), off
  one root at `-World::map_origin()`. Anything the renderer remembers between
  frames in world coordinates has to move when the origin does (the camera's
  focus does: `CameraRig::shift`).

`arena::MAX_SOLIDS` (64) now holds only for an arena fought on alone
(`tests/arena.rs`); the town and the reaches have no cap.

**The ground between places is land** (`crate::valley::land`): a height
function, not a floor per place. Ground steeper than `terrain_steepest` is a
wall (`Terrain::on_land`, `too_steep_to_climb`), and whether a box hangs is
`Terrain::hangs` -- above the floor under it -- never `Solid::hangs`, which
assumes the floor is at zero. A reach's road, seams and crags are written in
map coordinates; trees, rocks, crags and cairns are made by
`valley::layout`. Every road and every room is walked end to end by
`tests/valley.rs`: a change to the land that leaves a way unwalkable fails
there.

## Tuning: every magnitude is a knob in the Oven

Feel numbers live in the Oven (`crates/sim/src/oven.rs`), are edited in the
running game with F7, and are baked to `crates/sim/src/tuned.rs`. A number
written straight into the code cannot be tuned or committed from there.
Enforced by `crates/sim/tests/knobs.rs`, which lists the handful of genuine
exemptions and demands a reason for each.

After changing the Oven's shape, run `cargo run -p sim --bin bake_tuning` —
`crates/sim/tests/oven.rs` fails if the committed file is not what the Oven
would write.

## The simulation is deterministic and float-free

`crates/sim` has no floating point at all, no dependencies, and no I/O: rollback
netcode re-simulates past frames and two machines must agree bit for bit. Fixed
point is `Fx`, 16.16. Enforced by `crates/sim/tests/no_floats.rs` and
`determinism.rs`.

Anything that affects gameplay belongs in the `World` snapshot — **including
animation clocks**, because a clip advancing on the renderer's own clock pops
every time a rollback happens.

## A frame costs what it is budgeted, and a frame never allocates

The simulation runs **more than once per picture**: a rollback re-simulates up to
eight frames and saves a snapshot for each, inside the same 16.7 ms that still
has to draw. So per-frame cost arrives multiplied, exactly when the connection
is already struggling. One rollback burst gets a quarter of the frame, one
`advance` a thirty-second of it, one frame of `view` an eighth. Enforced by
`crates/sim/tests/budget.rs`, `view/tests/budget.rs` and `net/tests/budget.rs`.

**Add to the `no floats, no deps, no I/O` list: no allocation.** A simulation
frame, and the whole path from snapshot to screen in `view`, must not touch the
heap — an allocator is a lock with a tail, and it is what turns one frame in a
thousand into a millisecond. That check and the 4 KiB cap on `World` are the
ones that matter: they give the same answer on every machine, whereas the three
timing budgets run at 5–40x the measured cost and will only catch a change of
*kind* — an accidental quadratic, an unbounded scan, a blocking call.

The full reasoning is [`docs/design/architecture.md`](docs/design/architecture.md)
§"The frame budget".

## Feel is a set of properties, not a vibe

`crates/sim/tests/feel.rs` holds the relationships that must survive tuning:
every attack punishable on block, risk scaling with reward, and so on. If one
fails, either it is a bug **or a design decision changed** — and then
`docs/design/feel-log.md` and the relevant kit document need updating too, not
just the assertion. Record what you tried in the feel log, including the things
you reverted.

## The browser build is the same program

`game` compiles for a window and for a canvas, and **the two differ in exactly
five things, each of which lives in a file that exists to hold it.** Where a
run's settings come from (argv, or the query string — `?dev` is `--dev`), where
a player's settings are kept (a file, or local storage) and where a panic can be
read are `crates/game/src/platform.rs`. How the peer is reached (UDP, or a
public broker and WebRTC) is `online.rs`.
Whether there is a checkout to commit a bake to is `bake.rs` and `hub.rs`.
Enforced by `crates/game/tests/one_platform.rs`, which fails on `std::env`,
`std::fs`, `std::net`, `std::process`, `std::thread` or `web_sys` anywhere else
in the crate.

This rule is here for a failure that is *silent in one direction*, rather than
for one that has cost time already. A `std::fs::write` added to a system
compiles for wasm32 perfectly well — `std` is there, the call is there — and
returns an error nobody reads. The feature then works on the desk and quietly
does nothing on the web, and the first report of it comes from somebody who was
sent a link.

Build it with `./crates/web/build-game.sh`; the reasoning is
[`docs/design/web.md`](docs/design/web.md).

## The overlay draws what the hit test uses

`state::hitbox` is the one description of an attack's volume, and the debug
overlay, the browser sandbox and the creature's exchange all read it. An overlay
that can drift from the rule it illustrates is worse than no overlay.

## The art direction is a crate with no engine in it

Colour, skies and palettes live in `crates/look`, which depends on `sim` and
nothing else. The rule is the same one `view` follows for presentation logic: a
thing that is a pure function from a few numbers to a colour should be callable
without starting a game, because that is what makes it cheap to look at.

It is cheap to look at because **there will not be a developer for this game
forever**, so the cost that matters is not the arenas that exist — it is the
next one. `cargo run -p look --example skies` draws every arena's sky in one
picture in under a second; judging the same twenty-three in the game is
twenty-three launches, and what you are judging is a gradient, which does not
need a game to exist.

The loop, in that order: **derive, look, tweak, fold back.** A new arena names
one colour and gets a whole sky from it. Somebody looks at the sheet. What they
change by hand is a tweak. What the tweak turns out to be *generally* true
about is moved into the derivation, so the next arena starts closer and the
tweak is deleted. A tweak that stays a tweak forever is a derivation nobody
wrote down.

There are four modules and they stack: `sky` is what is behind everything,
`palette` what a surface's colour is, `edge` where the accent and the outline
go, and `skies` the one table that gives each arena its sky — from which its
palette follows, so an arena that names one colour has a whole look.

**Judge a palette lit, never as albedo.** `palette::lit` is the renderer's
response curve, measured off a screenshot rather than guessed, and every swatch
on the sheet goes through it. The first version of the palette looked bright and
distinct as raw albedo and arrived in the game as a white wash with the
differences flattened out of it by the tonemapper's knee. A harness that does
not predict the screen is a harness that will be tuned against *instead of* the
screen.

`crates/game/src/sky.rs` and `crates/game/src/shapes.rs` are the Bevy half: a
mesh, a material, a fog component, and the vertex colours that carry the
accent. Nothing in either decides what a colour should be.

**Sound is the same rule, in `crates/sound`.** A sound is a function from a
few numbers the simulation already has -- a blow's weight is its impact
freeze, what it struck and how big that was, a telegraph is exactly as long
as its startup, a footfall is the floor's material -- to a waveform, and
nothing is recorded. `cargo run -p sound --example sheet` renders the whole
voice to WAVs and a sheet of spectrograms in a second; `crates/sound/src/cue.rs`
is the one place the simulation's transitions are read as sounds, and
`crates/game/src/sound.rs` only plays them. If you find yourself deciding what
something sounds like in the game crate, stop: it goes in a patch. The
reasoning is [`docs/design/sound.md`](docs/design/sound.md).

## Tools that are not on every machine

Four things this repository does need tools a fresh machine does not have: the
**browser build** needs `wasm-bindgen` at the lock file's exact version, a
**headless screenshot** needs Xvfb, a software Vulkan driver and ImageMagick,
**loading the built page** (`./scripts/web-smoke.sh`, which is how a change
to the browser build is checked) needs Playwright and its Chromium, and **a
desktop joining a page's room** (`./scripts/room-desktop.sh`, how a change to
`net::native` is checked) needs all of those and mosquitto.
`./scripts/setup-tools.sh web`, `shot`, `browser`, `broker` or `all` installs
them, once, and is the only place the steps are written. A fifth, `desktop`,
is not a tool but the one system library the **desktop build** itself needs
on Linux: ALSA's headers, for the game's sound; a build error naming
`alsa-sys` is that. The Pages workflow runs it; a
cloud environment's setup script should run it too, so a session starts with
them.

**Decide up front, install in the background, check before the step.** The
install is a cargo build of about ninety seconds and an apt run, so it is never
worth waiting on and never worth discovering late:

1. Before starting work, read the task for whether it will end in a browser
   build (anything touching `crates/web`, `platform.rs`, the manual's HTML, or a
   "check it in the browser") or a headless screenshot (anything that has to be
   *seen*: a clip, a material, a HUD change). If it will, start the install
   **now**, in the background, logging to a file:

   ```
   ./scripts/setup-tools.sh web browser > target/setup-tools.log 2>&1 &
   ```

2. Do everything that does not need it.
3. Before the build or the screenshot, read the log. The script's last lines say
   `installed`, `already installed`, or `FAILED:` with the exact command, and it
   exits non-zero if anything is still missing.
4. If it failed, run the build's own type-check instead
   (`cargo check -p game --target wasm32-unknown-unknown`), say in the report
   which command failed and with what, and say plainly that the browser build,
   the page load or the screenshot was **not** run. "This machine does not have
   it" is a report of a failed install, never a reason to skip the step
   silently.

A browser-build change is checked by `./scripts/web-smoke.sh`: it builds, serves
and loads the page in headless Chromium, fails on any console error, failed
request or missing controls entry, and leaves a screenshot. Look at the
screenshot; a page that loads clean and draws nothing has still failed.

## Checks before you push

```
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

These are the only checks there are: they run on your machine, and nothing
runs them for you. The one workflow in `.github/` publishes the browser build to
GitHub Pages and tests nothing, so a green Pages run means the page deployed,
not that the change is good.
