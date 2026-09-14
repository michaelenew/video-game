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
renderer, the same controls, one player.

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
[`online.rs`](../../crates/game/src/online.rs) has the peer, and `bake.rs` and
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

**There is no peer.** A page cannot open a UDP socket, so `Driver` has one
variant on wasm and `net` is not a dependency of the browser build at all. This
was the known cost going in and it is an acceptable one: the browser is for
*seeing* the game, and the training dummy on `1`–`4` is enough to feel a class
out. Two players on one keyboard still works — dummy mode `4` hands player two
the second key set — so a browser can seat two people on a sofa, just not two
people in two cities.

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

## Getting the second player back

Rollback does not care what carries the inputs. GGRS takes a transport, the wire
format is two bytes per player per frame, and the browser has WebRTC data
channels, which are UDP-like and can be unordered and unreliable — which is what
rollback wants. What it needs that the desktop build does not is a signalling
server for the handshake, which is a small always-on service and therefore a
different kind of commitment from "no server".

Not now, and not blocking anything: the open question this build exists to
answer is whether the fight is any good, and one person and a dummy answers a
surprising amount of it.

## What this build cannot tell you

Worth being clear, because a link is easy to over-read:

- **Nothing about netplay.** No rollback, no prediction, no desync. Those are
  tested by `net`'s SyncTest and by two desktops.
- **Nothing conclusive about performance.** The browser is slower and
  single-threaded. A frame that misses here may be fine natively.
- **Nothing about the versus match.** One player, on a dummy that stands still,
  blocks, or attacks on a seventy-frame cadence. It answers *does this class feel
  like anything*, not *is this matchup fair*.
