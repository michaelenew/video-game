# video-game

Design notes live in [`docs/`](docs/README.md) — current design in [`docs/design/`](docs/design/README.md),
with the original Google Drive export archived alongside it.

Run it with `cargo run -p game`; `cargo run -p game -- --help` lists everything
you can type. `--hunt` starts a hunt (`--hunt <creature>` for any creature that
is built, in its own arena, and `H` / `Shift+H` switch in game); `--arena range`
is a dev arena with one of everything — see
[`docs/design/arenas.md`](docs/design/arenas.md), and
[`docs/design/species.md`](docs/design/species.md) for adding a creature. `./crates/web/build-game.sh` builds the same thing as a web page —
see [`docs/design/web.md`](docs/design/web.md).
