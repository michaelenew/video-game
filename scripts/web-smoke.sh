#!/usr/bin/env bash
# Prove the browser build starts: build it if it is not there, serve it, load
# it in headless Chromium and report. Then prove two pages can play each
# other: two tabs open one room and must meet (`crates/web/room-smoke.mjs`).
#
#   ./scripts/web-smoke.sh [screenshot.png]
#
# Then the Esc menu: one tab creates a room from it, another joins from it
# (`crates/web/menu-smoke.mjs`). Then a room that cannot form must say why, in
# the console and with F10 in the menu (`crates/web/diag-smoke.mjs`).
#
# WEB_ROOM=0 skips the room, menu and diagnosis checks, which load the game
# five times more.
# WEB_REBUILD=1 rebuilds even when target/web exists. WEB_QUERY=hunt=gnawers
# loads the page asking for something, so a creature can be checked reachable. The build needs
# `./scripts/setup-tools.sh web` and the browser needs `... browser`; both are
# idempotent and a session that knows it will end here starts them first, in
# the background (see CLAUDE.md, "Tools that are not on every machine").
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="${1:-target/web-smoke.png}"
PORT="${PORT:-8080}"

if [ ! -f target/web/game_bg.wasm ] || [ "${WEB_REBUILD:-0}" = 1 ]; then
  ./crates/web/build-game.sh
fi

python3 -m http.server --directory target/web "$PORT" >/dev/null 2>&1 &
SERVER=$!
trap 'kill $SERVER 2>/dev/null || true' EXIT
sleep 1

node crates/web/smoke.mjs "http://127.0.0.1:$PORT/${WEB_QUERY:+?$WEB_QUERY}" "$OUT"

if [ "${WEB_ROOM:-1}" != 0 ]; then
  node crates/web/room-smoke.mjs "http://127.0.0.1:$PORT/${WEB_QUERY:+?$WEB_QUERY}" "${OUT%.png}-room"
  node crates/web/menu-smoke.mjs "http://127.0.0.1:$PORT/${WEB_QUERY:+?$WEB_QUERY}" "${OUT%.png}-menu"
  node crates/web/diag-smoke.mjs "http://127.0.0.1:$PORT/${WEB_QUERY:+?$WEB_QUERY}" "${OUT%.png}-diag"
fi
