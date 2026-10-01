# video-game

Design notes live in [`docs/`](docs/README.md) — current design in [`docs/design/`](docs/design/README.md),
with the original Google Drive export archived alongside it.

Run it with `cargo run -p game`; `cargo run -p game -- --help` lists everything
you can type. `--hunt` starts a hunt (`--hunt <creature>` for any creature that
is built, in its own arena, and `H` / `Shift+H` switch in game); `--arena range`
is a dev arena with one of everything. Beating a creature records its trophy
(kept beside your settings, or in the browser's storage) and offers it again
**tempered** — the same creature, cleverer — which `T` steps through and the
list on the right of the screen shows; `--temper <n>` starts at any temper
regardless ([`docs/design/world.md`](docs/design/world.md) §6). See
[`docs/design/arenas.md`](docs/design/arenas.md), and
[`docs/design/species.md`](docs/design/species.md) for adding a creature. `./crates/web/build-game.sh` builds the same thing as a web page —
see [`docs/design/web.md`](docs/design/web.md).
