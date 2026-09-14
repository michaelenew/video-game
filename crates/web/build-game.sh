#!/usr/bin/env bash
# Build the whole game as a web page.
#
#   ./crates/web/build-game.sh   ->  target/web/
#
# Three files and a marker: the page, the wasm-bindgen glue, and the module.
# Serve the directory over HTTP -- ES modules and `fetch` do not work from a
# `file://` URL -- or publish it. `.github/workflows/pages.yml` runs exactly
# this and hands the directory to GitHub Pages.
#
# What comes out is the same build the desktop runs, minus the peer: see
# `crates/game/src/platform.rs` for the four places the two differ, and
# `docs/design/web.md` for why it is the whole game rather than a cut-down one.
set -euo pipefail
cd "$(dirname "$0")/../.."

OUT=target/web
WASM=target/wasm32-unknown-unknown/wasm-release/game.wasm

# ---------------------------------------------------------------------------
# The glue generator has to match the crate it is generating glue for.
#
# A mismatch does not fail the build. It fails in the browser, at load, with a
# message about schema versions that reads like a bug in the game -- so it is
# checked here, where the fix is one line and can be printed.
# ---------------------------------------------------------------------------
want=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/[",]/, "", $3); print $3; exit }' Cargo.lock)
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}' || true)
if [ "$have" != "$want" ]; then
  echo "wasm-bindgen ${have:-(not installed)} but the lock file wants $want." >&2
  echo "  cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

# `wasm-release` is `release` with `opt-level = "z"`, `panic = "abort"` and the
# symbols stripped. Download size is the whole cost of a link somebody is not
# sure they want to click.
cargo build -p game --target wasm32-unknown-unknown --profile wasm-release

rm -rf "$OUT"
mkdir -p "$OUT"
wasm-bindgen --target web --no-typescript --out-name game --out-dir "$OUT" "$WASM"

# ---------------------------------------------------------------------------
# What gets published has to be something a browser will actually accept.
#
# This check is here because the first published build was not. The step it
# replaces ran `wasm-opt -Oz --all-features` to shrink the module;
# `--all-features` tells binaryen it may use everything it knows, GC included,
# and it wrote a type section no browser would parse -- `CompileError: invalid
# value type 0x0 @+198`, on a page that had loaded eighteen megabytes first. It
# passed locally because this machine had a different binaryen from the
# runner's, which is the whole reason a check belongs in the script rather than
# in somebody's memory of having looked once.
#
# `WebAssembly.compile` is the same validator the browser runs, so this is the
# real thing rather than a proxy for it. Node is on every CI runner; when it is
# missing, say so rather than implying the build was checked.
# ---------------------------------------------------------------------------
if command -v node >/dev/null 2>&1; then
  if node --input-type=module -e "
      import { readFile } from 'node:fs/promises';
      await WebAssembly.compile(await readFile('$OUT/game_bg.wasm'));
    "; then
    echo "the module validates"
  else
    echo "REFUSING TO PUBLISH: a browser cannot compile $OUT/game_bg.wasm" >&2
    exit 1
  fi
else
  echo "node not found, so nothing has checked that a browser can load this" >&2
fi

# The controls panel comes from the manual, so the page cannot describe keys the
# game does not have. The build stamp is so a bug report can name a build.
CONTROLS=$(cargo run -q -p manual -- --html)
STAMP="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
git diff --quiet 2>/dev/null || STAMP="$STAMP+dirty"
STAMP="$STAMP · $(date -u +%Y-%m-%d)"

CONTROLS="$CONTROLS" STAMP="$STAMP" python3 - crates/web/game/index.html "$OUT/index.html" <<'PY'
import os, sys
template, out = sys.argv[1], sys.argv[2]
html = open(template).read()
for marker, value in (("__CONTROLS__", os.environ["CONTROLS"]), ("__BUILD__", os.environ["STAMP"])):
    assert marker in html, f"the page is missing the {marker} placeholder"
    html = html.replace(marker, value)
open(out, "w").write(html)
PY

# GitHub Pages runs Jekyll over anything without this, which is slower and has
# opinions about files beginning with an underscore.
touch "$OUT/.nojekyll"

echo
echo "$OUT"
ls -l "$OUT" | awk 'NR > 1 { printf "  %-16s %6d KB\n", $9, ($5 + 1023) / 1024 }'
echo
echo "Try it:  python3 -m http.server --directory $OUT 8080"
