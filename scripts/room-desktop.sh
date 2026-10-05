#!/usr/bin/env bash
# Prove a desktop can join a page's room: a page in headless Chromium opens a
# sealed room, the desktop build joins it with --join under Xvfb, and the two
# must meet through a real MQTT broker and play. See crates/web/room-desktop.mjs.
#
#   ./scripts/room-desktop.sh [screenshot-prefix]
#
# Needs the page built (./crates/web/build-game.sh; WEB_REBUILD=1 rebuilds),
# and three tools: `./scripts/setup-tools.sh browser shot broker`. The broker is
# mosquitto on this machine rather than a public one, so the check does not
# depend on somebody else's server or on this machine reaching it.
set -euo pipefail
cd "$(dirname "$0")/.."

PREFIX="${1:-target/room-desktop}"
PORT="${PORT:-8080}"
BROKER_PORT="${BROKER_PORT:-9001}"
DISPLAY_NUM="${DISPLAY_NUM:-98}"

for tool in mosquitto Xvfb import node; do
  command -v "$tool" >/dev/null || {
    echo "$tool is missing: ./scripts/setup-tools.sh browser shot broker" >&2
    exit 1
  }
done

if [ ! -f target/web/game_bg.wasm ] || [ "${WEB_REBUILD:-0}" = 1 ]; then
  ./crates/web/build-game.sh
fi
cargo build -p game
mkdir -p "$(dirname "$PREFIX")"

CONF="$(mktemp)"
printf 'listener %s 127.0.0.1\nprotocol websockets\nallow_anonymous true\n' "$BROKER_PORT" > "$CONF"
mosquitto -c "$CONF" >"$PREFIX-broker.log" 2>&1 &
BROKER=$!
python3 -m http.server --directory target/web "$PORT" >/dev/null 2>&1 &
SERVER=$!
Xvfb ":$DISPLAY_NUM" -screen 0 900x560x24 >/dev/null 2>&1 &
XVFB=$!
trap 'kill $BROKER $SERVER $XVFB 2>/dev/null || true; rm -f "$CONF"' EXIT
sleep 2

ROOM="desk$RANDOM"
KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
LINK="http://127.0.0.1:$PORT/?room=$ROOM&broker=ws://127.0.0.1:$BROKER_PORT#key=$KEY"
DISPLAY=":$DISPLAY_NUM" WGPU_BACKEND=vulkan node crates/web/room-desktop.mjs "$LINK" "$PREFIX"
