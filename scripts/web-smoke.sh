#!/usr/bin/env bash
# Prove the browser build starts: build it if it is not there, serve it, load
# it in headless Chromium and report.
#
#   ./scripts/web-smoke.sh [screenshot.png]
#
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
