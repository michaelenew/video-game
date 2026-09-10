#!/usr/bin/env bash
# Two networked clients on one machine, over real UDP.
#
#   ./scripts/p2p-localhost.sh            two windows, if you have a display
#   ./scripts/p2p-localhost.sh --headless two Xvfb displays, screenshots both
#
# For a headless determinism check without graphics, prefer:
#   cargo run -p net --bin p2p_localhost
set -euo pipefail
cd "$(dirname "$0")/.."

PORT_A=47811
PORT_B=47812
cargo build -p game

if [ "${1:-}" = "--headless" ]; then
  export WGPU_BACKEND=vulkan
  Xvfb :98 -screen 0 900x560x24 >/dev/null 2>&1 & XA=$!
  Xvfb :97 -screen 0 900x560x24 >/dev/null 2>&1 & XB=$!
  trap 'kill $XA $XB 2>/dev/null || true' EXIT
  sleep 2
  DISPLAY=:98 DEBUG_OVERLAY=1 DEMO=1 ./target/debug/game --port $PORT_A --peer 127.0.0.1:$PORT_B &
  GA=$!
  DISPLAY=:97 DEMO=1 ./target/debug/game --port $PORT_B --peer 127.0.0.1:$PORT_A &
  GB=$!
  trap 'kill $GA $GB $XA $XB 2>/dev/null || true' EXIT
  sleep "${WAIT:-25}"
  mkdir -p target
  DISPLAY=:98 import -window root target/peer-a.png
  DISPLAY=:97 import -window root target/peer-b.png
  echo "target/peer-a.png target/peer-b.png"
else
  ./target/debug/game --port $PORT_A --peer 127.0.0.1:$PORT_B &
  ./target/debug/game --port $PORT_B --peer 127.0.0.1:$PORT_A
fi
