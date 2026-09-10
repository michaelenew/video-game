#!/usr/bin/env bash
# Build the browser sandbox: compile the sim to wasm and inline it into a
# single self-contained HTML file.
#
#   ./crates/web/build-sandbox.sh  ->  target/sandbox/frame-lab.html
#
# Self-contained on purpose: no server, no fetch, no CORS. Open the file.
set -euo pipefail
cd "$(dirname "$0")/../.."

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo build -p web --target wasm32-unknown-unknown --profile wasm-release

WASM=target/wasm32-unknown-unknown/wasm-release/web.wasm
OUT=target/sandbox/frame-lab.html
mkdir -p "$(dirname "$OUT")"

python3 - "$WASM" crates/web/sandbox/index.html "$OUT" <<'PY'
import base64, sys
wasm, tpl, out = sys.argv[1], sys.argv[2], sys.argv[3]
html = open(tpl).read()
b64 = base64.b64encode(open(wasm, "rb").read()).decode()
assert "__WASM_B64__" in html, "template is missing the wasm placeholder"
open(out, "w").write(html.replace("__WASM_B64__", b64))
print(f"{out}  ({len(html)//1024} KB template + {len(b64)//1024} KB wasm)")
PY
