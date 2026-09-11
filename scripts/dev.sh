#!/usr/bin/env bash
# Launch in full development mode: hitbox wireframes and the Oven, both open.
#
# Extra arguments are passed through, so class picks still work:
#   ./scripts/dev.sh --p1 bellator --p2 reaver
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run -p game -- --dev "$@"
