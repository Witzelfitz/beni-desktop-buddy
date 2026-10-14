#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ $# -gt 1 || ($# -eq 1 && "$1" != "--desktop" && "$1" != "--help") ]]; then
  echo "Usage: scripts/check.sh [--desktop | --help]" >&2
  exit 2
fi
if [[ "${1:-}" == "--help" ]]; then
  echo "Usage: scripts/check.sh [--desktop]"
  echo "Default: core, CLI, assets and release policy. --desktop adds GUI checks."
  echo "Python: BENI_RELEASE_PYTHON, then .venv-release/bin/python, then python3."
  exit 0
fi
check_python="${BENI_RELEASE_PYTHON:-}"
if [[ -z "$check_python" ]]; then
  if [[ -x .venv-release/bin/python ]]; then
    check_python="$PWD/.venv-release/bin/python"
  else
    check_python="python3"
  fi
fi
if ! command -v "$check_python" >/dev/null 2>&1; then
  echo "Python für die Prüfungen fehlt. Siehe docs/TOOLING.md." >&2
  exit 2
fi
"$check_python" scripts/check-environment.py
"$check_python" scripts/check-tooling.py
"$check_python" scripts/release.py check-versions
"$check_python" scripts/check-macos-packaging.py
"$check_python" scripts/check-portable-packaging.py
"$check_python" scripts/check-vendor-tests.py
"$check_python" scripts/check-vendor.py
"$check_python" scripts/check-release.py
"$check_python" scripts/check-local-release.py
"$check_python" scripts/check-sparkle.py
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --locked -p buddy-cli -- validate-assets
"$check_python" scripts/check-integrations.py
"$check_python" scripts/check-assets.py
if [[ "${1:-}" == "--desktop" ]]; then
  cargo fmt --manifest-path apps/desktop/Cargo.toml --check
  cargo test --manifest-path apps/desktop/Cargo.toml --locked
  "$check_python" scripts/check-storage-sync.py
  cargo clippy --manifest-path apps/desktop/Cargo.toml --all-targets --locked -- -D warnings
  cargo build --manifest-path apps/desktop/Cargo.toml --locked
fi
