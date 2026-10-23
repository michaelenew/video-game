#!/usr/bin/env bash
# Run a script under Blender, as a Python module, with no display.
#
#   art/blender.sh art/reaver/build.py [args...]
#
# Blender is the `bpy` package in a virtual environment under target/, which
# `./scripts/setup-tools.sh art` makes, along with MakeHuman's CC0 data.
set -euo pipefail
cd "$(dirname "$0")/.."
PY=target/art/venv/bin/python
if [ ! -x "$PY" ] || [ ! -d target/art/mh ]; then
  echo "Blender or MakeHuman's data is missing: ./scripts/setup-tools.sh art" >&2
  exit 1
fi
exec "$PY" -W ignore "$@"
