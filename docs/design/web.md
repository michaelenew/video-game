---
status: decided
decided: 2026-09-14
---

# The browser build

```
./crates/web/build-game.sh      ->  target/web/
```

The same game, compiled to WebAssembly and pointed at a canvas instead of a
window. Not a demo of it, not a cut-down version: the same simulation, the same
renderer, the same controls -- and, since 2026-10-03, the same netcode: two
people with a link play each other (§"Playing a friend from a link").

It exists because of §8 of the [design README](README.md) — *play it against a
person* — and because of what that costs to arrange today. Installing Rust,
waiting out a first Bevy build and finding out your driver is unhappy is a large
thing to ask of somebody doing you a favour. A link is not. Everything below is
downstream of wanting the ask to be a link.

## The two browser things in this repository are unrelated

| | `build-sandbox.sh` | `build-game.sh` |
| --- | --- | --- |
| What it is | Frame data, drawn in 2D | The game |
| What is compiled | `crates/web` — the simulation and about a hundred lines of C ABI | `crates/game` — everything |
| Who it is for | Whoever is tuning a move | Whoever was sent the link |
| Size | Tens of kilobytes | Megabytes |
| Bindings | None. Raw exported functions, integers across the boundary | `wasm-bindgen` |

The sandbox answers *what are this move's frames*. The page answers *what is
this game like*. Neither replaces the other, and the sandbox is deliberately
left alone: it is small and self-contained, and the moment it grows a renderer
it stops being either.

## What a page cannot do

Five things, and every one of them is in a file that exists to hold it rather
than scattered through the crate:
[`platform.rs`](../../crates/game/src/platform.rs) has three,
[`online.rs`](../../crates/game/src/online.rs) has how the peer is reached, and `bake.rs` and
`hub.rs` have the checkout. `crates/game/tests/one_platform.rs` is what makes
that true rather than merely written down — it fails on a `std::env`,
`std::fs`, `std::net`, `std::process` or `std::thread` anywhere else in `game`.
Everything outside those four files is written once and compiled twice.

**There is no command line.** So the query string is one: `?p1=champion&dev` is
`--p1 champion --dev`. Both spellings go through the same parser, names are
matched case-insensitively with `-` and `_` alike, and a unit test asserts that
the same run spelled two ways is the same run. Environment variables come
through the same door, so `?shot_frame=200` is `SHOT_FRAME=200`. Without that,
every knob the game has would have been unreachable from a link, and the ones
worth reaching are exactly the ones you want when somebody reports something.

**There is no settings file.** Sensitivity and field of view go to
`localStorage`, holding character-for-character the text the desktop writes to
`~/.config/arena/settings.conf`. One format, two places to keep it. A shared
link cannot carry one player's sensitivity to another, which is the correct
behaviour and comes for free.

The **trophy record** (world W1) is kept the same way under its own key,
`arena.trophies`, beside `~/.config/arena/trophies.conf` on the desktop -- the
same text in both, so a person can read which creatures they have beaten.

**There is no UDP socket.** So the peer is reached another way: a room on a
public message broker to meet, and a WebRTC data channel to play over. Same
`Driver::Online`, same GGRS session; only the way of meeting differs, and
`online.rs` is where it is chosen. See §"Playing a friend from a link".

**There is no checkout.** The Oven still opens on `F7` and every number in it
still moves live, which is most of what the Oven is for: you can try a tuning
idea and watch it. What the browser cannot do is *keep* one, because baking
writes `crates/sim/src/tuned.rs` and pushes it. So the Bake button and the
animation hub's Save both say so, in a sentence that names the way out, rather
than being hidden or — much worse — appearing to work. The same holds for the
hub: edit a recipe, re-run the solver, watch it on a fighter, all fine; saving
it needs a clone.

**A panic has nowhere to print.** There is no terminal, and the default answer
is a blank canvas plus `RuntimeError: unreachable` in a console nobody has open.
A panic hook writes the message onto the page instead. Somebody who followed a
link and hit a driver bug can then say *what* broke without being talked through
the developer tools, which is the difference between a bug report and "it
didn't work".

## Why WebGL 2 and not WebGPU

WebGPU is the better target and is what this should move to eventually. It is
not what a link can rely on today: as of this writing it ships by default in
Chrome and Edge, and is behind a flag or partial elsewhere. A link that works in
one browser is not a link you can send to six people. WebGL 2 has been
everywhere for years and the arena is a few dozen procedural meshes — this
prototype is nowhere near the point where the API is what limits it.

Switching later is a feature flag in `crates/game/Cargo.toml` and nothing else.

## Why single-threaded

Threads in WebAssembly need `SharedArrayBuffer`, which needs the page to be
cross-origin isolated, which needs two response headers (`Cross-Origin-Opener-Policy`
and `Cross-Origin-Embedder-Policy`). GitHub Pages does not let you set response
headers. So `multi_threaded` is off in the browser build and Bevy's schedule
runs on one thread.

That is affordable here for a specific reason: the part of the frame that must
be fast is the simulation, and it is budgeted at a thirty-second of a frame with
no allocation and no floating point — see
[architecture.md](architecture.md) §"The frame budget". What loses the threads is
the renderer's own parallelism, on a scene of a few dozen meshes. If the arena
ever becomes heavy enough for that to bite, the fix is a host that sends the two
headers, not a change to the game.

## Determinism carries over

`sim` has no floating point, no dependencies and no I/O, and none of that changes
when the target does. A fight in the browser is bit-for-bit the fight the desktop
would have played from the same inputs, so anything strange somebody reports from
a link reproduces in a checkout. That is the property that makes a shared build
worth having as a *harness* rather than only as a toy.

It also means the browser build is not a second thing to keep in sync. There is
no wasm-specific gameplay code to drift.

## Size, measured — and why nothing optimises it

The module is the whole cost of a link somebody is not sure they want to click,
so it is worth having the real numbers rather than a feeling about them.

| | Module | Over a gzipping connection | Loads? |
| --- | --- | --- | --- |
| **As the compiler leaves it — what is published** | 23.9 MB | **6.9 MB** | yes |
| `wasm-opt -Oz --all-features` | 17.9 MB | 7.1 MB | **no** |
| `wasm-opt -Oz`, features named explicitly | 20.0 MB | 7.1 MB | yes |

Plus 108 KB of `wasm-bindgen` glue and a 26 KB page.

**`wasm-opt` is not in the build, and the middle row is why.** It shipped once
and every visitor got `CompileError: WebAssembly.instantiate(): invalid value
type 0x0 @+198` after downloading eighteen megabytes to find out.

The mechanism is worth writing down because the mistake is easy to repeat. The
`wasm-release` profile sets `strip = true`, which removes the `target_features`
custom section — the compiler's own record of which post-MVP WebAssembly
features the module uses. Without it `wasm-opt` refuses to touch the module,
naming the first thing it trips over (bulk memory). Passing `--all-features`
makes that error go away, and it is the wrong answer to it: it does not say
"here is what this module uses", it says **"you may use anything you know"** —
so binaryen enabled GC and typed function references and re-encoded the type
section in a form browsers reject. The right set was never in doubt; it was
sitting in the section that got stripped:

```
bulk-memory  bulk-memory-opt  call-indirect-overlong  multivalue
mutable-globals  nontrapping-fptoint  reference-types  sign-ext
```

No `gc`. Naming those explicitly works (third row) — and that is the version to
reinstate if raw size ever matters, because the failure mode is then gone by
construction rather than by having chosen a good binaryen.

**But the first row is the one to ship, and not only for safety.** Look at the
second column: optimising makes the *compressed* size slightly **worse**.
Compacting code by sharing and reordering leaves gzip less repetition to find,
and the compressed size is what a visitor actually pays on any host that
compresses. So `wasm-opt` buys 4–6 MB of raw size, costs 3% of wire size, and
carries a dependency on which binaryen the machine happens to have. That is a
bad trade for a prototype somebody is following a link to, and dropping it also
takes two and a half minutes off every build.

The build script now checks what it is about to publish with
`WebAssembly.compile` — the same validator the browser runs — and refuses to
write a module that fails. That check, not a better flag, is the actual lesson:
the broken build passed locally because this machine had binaryen 124 and the
runner had 108.

**What is actually in the 23.9 MB**: about 16 MB of code and 6 MB of data, and
almost all of both is Bevy, `wgpu` and the shader compiler. The game's own code,
the Oven's 1,225 knobs and the baked animation tables are a rounding error next
to that. Dropping `bevy_egui` — which would cost the Oven and the animation hub
— is the one obvious cut left, and on the crate sizes it looks like a few
megabytes rather than a step change. If the number ever has to come down a lot,
the thing to attack is Bevy's feature list.

## Publishing

`.github/workflows/pages.yml` runs `build-game.sh` on a push to `main` and hands
`target/web/` to GitHub Pages. It needs Pages switched on for the repository,
with the source set to GitHub Actions — Settings → Pages → Build and deployment
→ Source.

It is a deploy, not a gate: it does not run the test suite, and a red build there
does not block anything. The checks in [CLAUDE.md](../../CLAUDE.md) are still the
only checks, and they still run on a person's machine before a push.

Locally, `python3 -m http.server --directory target/web 8080`. Opening the file
directly will not work — ES modules and `fetch` both need an origin.

## Playing a friend from a link

*Built 2026-10-03.* What a person does:

1. Opens the page and presses **Play with a friend**. The URL gains
   `?room=k3x9qw#key=…` -- six random letters to recognise the room by, and
   twenty-six after the `#` that seal it (§"Sealed rooms") -- and the page
   shows that link with a Copy button.
2. Sends it. Their friend opens it -- in a browser, or on a desktop with
   `game --join '<link>'` (§"A desktop in a browser's room").
3. Both pages say *Online: you are player one / two*, and the match starts from
   the beginning. Whoever opened the room first is player one.

While waiting, the fight runs as training, so the wait is practice. The match
then starts from the world the link describes, not from wherever practice got
to: classes and arena are the link's (`&p1=`, `&p2=`, `&arena=`), and in a
match the training tools that edit the world on one machine -- Tab, Backspace,
pause, stepping, rewind, the class pickers -- are switched off rather than
allowed to desync it. `H` and `T` still work, because they travel on the wire.

### From inside the game: the Esc menu

*Built 2026-10-06.* The same rooms, from the client rather than the page, and
the same on a desktop as in a browser (`crates/game/src/menu.rs`). Escape
steps out of the fight and brings up a menu; a click in the arena goes back.
In it:

- **Create a room**: a fresh name and secret, and the fight being practised --
  both classes, the creature, its temper, the arena -- written into the link
  (`picker::describe`). The match starts from the world *the link* describes,
  built by the same function a friend's client will use (`start_world`), so
  the two agree even where the link cannot say everything about the practice.
- **Join**: paste a friend's link. It is unpacked exactly as `--join` is.
- **The active room**, while there is one: its name, how meeting is going, the
  link with a **Copy link** button, and **Leave**. In a browser the link also
  goes into the address bar, so a reload stays in the room.
- **Progress** -- the creature list with trophies and tempers -- on the right,
  and only while the menu is up.

A room made on a desktop links to the published page, so a friend without the
game clicks it and plays in their browser; `--page <url>` points it at another
deploy. A page releases the mouse itself on Escape without the key reaching the
game, so the game also opens the menu when the page reports its mouse lock gone.

### How the two pages find each other

Two browsers cannot talk directly until each knows how to reach the other, and
learning that takes one note each way -- an *offer* and an *answer*, in
WebRTC's words. Something has to carry those two notes. Three options were on
the table: copy and paste them by hand, run a small server of our own, or
borrow public infrastructure. **The third, for now**: free public MQTT brokers
(`broker.emqx.io`, `broker.hivemq.com`, `test.mosquitto.org`), which any page
can reach over a WebSocket. A room is a topic; every note goes to all three, so
one being down does not close the room. Google's and Cloudflare's public STUN
servers tell each page what its address looks like from outside its router.
None of them carries a frame of the match: once the data channel is open, the
two browsers talk to each other and nobody else.

### Sealed rooms

*Built 2026-10-05.* A public broker's topics are readable by anyone, and the
two notes a room trades hold each player's network addresses. So the link
carries a secret -- 26 random letters, about 129 bits, after `#key=`, the part
of a link a browser never sends to the server it came from -- and two things
are derived from it and the room's name with SHA-256 under separate labels:
the **topic** the brokers see (a hex string that names nothing), and a key
that seals every note with ChaCha20-Poly1305. A note that does not open is
dropped, so a stranger can neither read the room nor forge a hello into it.
The topic is bound into each seal, so a note lifted from one room does not
open in another, and every note gets a fresh random nonce, so a repeated hello
is not recognisable as a repeat.

It is one more `Board`: `seal::Sealed<B>` wraps any board, so the room never
sees a sealed note and the brokers never see an open one, and nothing else
changed. A link without a key still works, sealed under the room name alone,
which stops somebody watching every topic but not somebody who can guess six
letters.

### A desktop in a browser's room

*Built 2026-10-05.* `game --join '<link>'` opens the link a friend's page made:
`platform.rs` unpacks its query and its `#` into the same settings a page
reads from its own address -- room, key, classes, arena -- with any flag typed
beside it winning. Then `net::native` builds the same room from different
parts: the same brokers over a TLS WebSocket (a thread per broker, because
connecting blocks), the same sealed notes, and the same data channel from
[str0m](https://github.com/algesten/str0m), a WebRTC library that does no I/O
itself -- it is handed packets and the time, so the room's once-a-frame polling
drives it with no async runtime. A desktop asks a STUN server for its own
outside address with a twenty-byte question of its own (RFC 5389), since str0m
leaves gathering to its caller. Either side may make the room or the offer.

The two must be the same build: `crates/game/build.rs` stamps a desktop build
with its commit, the way `build-game.sh` stamps the page, so a desktop built
from the commit the page was deployed from (the page's footer names it) is
the page's build.

### Built in three seams, so any one can be swapped

`crates/net/src/meet.rs`:

| Seam | What it is | Today | Slots in later |
| --- | --- | --- | --- |
| `Board` | Where strangers leave notes under a room name | Public MQTT brokers (`browser::public`, `native::public`); any one broker (`?broker=wss://…`); two tabs of one browser (`browser::tabs`, `?board=tabs`); the UDP port itself on a desktop (`direct`); any of these sealed (`seal::Sealed`) | Our own signalling server; Nostr relays; copy and paste |
| `Line` | The direct connection, opened by trading an offer and an answer | WebRTC data channel, unordered and never resent: a page's own (`browser::RtcLine`), or str0m on a desktop (`native::RtcLine`); UDP (`direct::UdpLine`) | A TURN relay |
| `Rendezvous` | The whole meeting, polled once a frame until it hands GGRS a socket | `Room<B: Board, L: Line>` | Matchmaking; a lobby |

`Room` is the protocol and it is four lines of text, which is why a board can be
anything that carries text. Every note is repeated until what it asked for has
happened, because a public broker delivers at most once. A hello carries the
**terms** -- the build stamp and the checksum of the starting world, which
covers both classes, the arena and every tuned number -- and a room never
answers a hello with different terms: two pages from different deploys, or one
with a different `?p1=`, are told so rather than left to desync. The peer with
the lower random id makes the offer, and the offer says who is player one, so
the seats are decided once, by one side, and cannot come out differently on the
two machines. Offers are numbered: a page only speaks from its frame loop, a
frame can take seconds, and an offerer that gave up and started over must never
be handed the answer to its previous offer, which describes a connection that
no longer exists. (The first two-tab run found exactly that.) `crates/net/tests/meet.rs` runs the whole protocol and a real
GGRS match in memory, with notes lost at random.

The desktop's `--port`/`--peer` is the same `Room` over a board and a line that
are both the one UDP port. That also fixed a bug: seats used to come from
ordering the two addresses, and each machine knew its own only as
`127.0.0.1`, so on two real machines both claimed player one.

### When a room will not form: F10

*Built 2026-10-06*, after the first real try failed on both a page and a
desktop with "Could not reach any meeting point", and nothing anywhere said
why. A room is a chain -- look up a broker, connect, TLS, WebSocket, MQTT,
subscribe, hear a hello, open the line -- and that sentence is the end of it,
so every link in it now leaves a trace.

**F10 switches dev mode on mid-session** (`--dev`, without relaunching), and
in dev mode the Esc menu lists the meeting's own account of itself,
`Rendezvous::report`, with a *Copy details* button:

- **Each meeting point, step by step, with times.** `Board::report`, from a
  `meet::Trace` each board keeps. A desktop can say everything: the address a
  broker's name looked up to, which address it connected to or why not, the
  TLS and WebSocket handshake (with the HTTP status if a server answered), and
  the broker's MQTT answer with its refusal code spelled out. A page can say
  much less, by design -- a browser does not tell a page why a connection
  failed, so that a page cannot probe a network -- but it does give the close
  code, and 1006 before the socket ever opened means refused, blocked,
  unreachable or a bad certificate. The browser's own console line beside it
  names the real reason.
- **The sealed room**: its topic, and how many notes on it would not open,
  which is the answer when two clients hear each other and never meet (two
  links with different secrets).
- **The room**: this client's id, hellos posted, notes read, the friend's id
  and when they were last heard, and the stage of the offer and answer.
- **The direct line**: on a desktop the local and outside addresses and the
  ICE state; in a page the connection, ICE, gathering and channel states.

When a meeting fails, the same lines are printed to the console -- the
browser's, or the terminal -- whether dev mode is on or not, because once it
has failed it is too late to switch anything on for the same story.

The first trace this was built to read turned up a bug before it ran:
**a desktop could never reach a `wss://` broker.** tungstenite's TLS uses
rustls, rustls 0.23 will not connect until a process names its cryptography,
and nothing did, so each broker's thread panicked inside the handshake -- and
a panicked thread never marked its broker down, so the room said "Reaching the
meeting point…" for ever. `net` now builds rustls with aws-lc (which str0m
already builds) and `native::install_crypto` names it, so a second backend
arriving with some later dependency cannot make it ambiguous again; a broker
thread that panics anyway reports itself down with the panic's message. `native::tests::a_tls_handshake_fails_with_a_reason_rather_than_a_panic`
fails without the fix. The desktop tests had only ever used a local `ws://`
broker, without TLS, which is why none of them saw it.

### Playing online should feel like playing locally

*2026-10-06*, from the first match between two real people, which found three
things.

**Input delay is none.** GGRS held each player's own input back two frames,
the usual starting point for a fighting game, and against the local game that
was felt at once -- which defeats the reason for rollback, whose job is to make
the network invisible. `net::p2p::INPUT_DELAY` is now zero: a press acts on
the next frame, and everything the far side does is predicted and corrected.
The cost is longer rollbacks (half the round trip, up to the session's
eight-frame window), which the frame budget was set for. `delay=` (`--delay`)
gives a frame or two back, on one side only, for a link where the other
fighter visibly jitters.

**The side that is ahead slows down.** Two clocks never agree, and the faster
one drifts ahead until it hits the prediction window and stops dead -- felt as
lag. GGRS measures the gap and leaves closing it to the game: `online::pace`
holds back one tick in ten on whichever side is ahead.

**Class changes travel on the wire.** Tab and the class pickers used to rebuild
the world on one machine, so a match refused them and you were stuck with the
link's classes. They are now `Travel::class(seat, class)` in your input, like
the creature picker's H: `World::advance` restarts the same fight with the new
class on the same frame on both machines, and both players changing on one
frame both count. Against a person you change only your own.

**So does everything else one keyboard does to the fight.** The rule, from
the same session: *online is two people at one keyboard, with one of them on
another machine.* Backspace, P and `]` are `Travel::RESTART`, `PAUSE` and
`STEP`, acted on by `World::advance` on both machines on the same frame;
`World::paused` is in the snapshot, and a paused frame changes nothing but the
frame count the rollback session owns. Only `[` stays training's -- it steps
back through a history this machine alone keeps.

**H and N used to drop your friend.** Not a desync in the simulation (a page
and a desktop agree bit for bit through every creature and course, run as
wasm32 and x86_64 alike): the first frame of an arena a page has not drawn
before compiles its shaders, on the page's one thread, for seconds. GGRS
counted two seconds of silence as a disconnect, so the side that pressed the
key said "your friend disconnected" and played on with blank inputs for them,
which the other side saw as a desync. `net::p2p` waits fifteen seconds now,
the screen says when your friend has gone quiet, and `online::pace` closes a
gap of more than three frames every other frame rather than one in ten.
`crates/net/tests/trips_online.rs` freezes one side for three and a half
seconds after a trip, and plays every trip and control from either side.

**And the camera of whoever joined.** It read the yaw of a second player on
one keyboard, which nothing turns online, so player two's fighter turned and
the camera did not. The camera now always reads the mouse, and a match starts
with the mouse facing the way your fighter stands.

### What it does not do yet

- **Some network pairs will not connect.** Mobile data and strict office or
  school networks often refuse direct connections. That needs a TURN relay,
  which carries the match and costs money to run. The page says so in a
  sentence rather than hanging.
- **The brokers still see who is talking.** A sealed room hides what the notes
  say, not that two network addresses connected to a broker at the same time.
  And the link is the key: whoever has it can join.
- **Rooms are two players.** A third visitor waits until one of the two leaves.
- **A desktop and a browser on one home network: expected to work, not yet
  seen to.** Chrome hides a page's local address behind a `.local` name that a
  desktop cannot look up, so the connection rests on the page reaching the
  desktop's local address, which the desktop does publish. (The smoke test
  turns the hiding off; two real machines on one network are the check.)
- **The link chooses the classes you start with.** Each of you can change
  your own after that, with Tab or your class picker (see below).
- **Public brokers promise nothing.** Swapping them for one we run is one
  constant, `meet::BROKERS`, and `?broker=` (`--broker`) tries one without a rebuild.
- **A tab in the background stops.** Browsers pause a hidden tab's frames, and
  the meeting and the match both run on frames. Waiting with the tab hidden is
  fine -- the room is still there when you come back -- but hiding it mid-match
  drops the connection.

## What this build cannot tell you

Worth being clear, because a link is easy to over-read:

- **Less about netplay than it looks.** Two tabs on one machine
  (`crates/web/room-smoke.mjs`) prove the pages meet and play, but with no
  latency to roll back over. Real latency is two people on two networks.
- **Nothing conclusive about performance.** The browser is slower and
  single-threaded. A frame that misses here may be fine natively.
- **Only as much about the versus match as you bring a friend for.** Alone it
  is one player on a dummy that stands still, blocks, or attacks on a
  seventy-frame cadence: *does this class feel like anything*, not *is this
  matchup fair*.
