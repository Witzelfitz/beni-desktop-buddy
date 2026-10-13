#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Preserve the existing development bundle; installer mode is explicit.
exec python3 scripts/package-macos.py bundle "$@"
