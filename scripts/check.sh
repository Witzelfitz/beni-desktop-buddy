#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/release.py check-versions
python3 scripts/check-macos-packaging.py
if [[ -n "${BENI_RELEASE_PYTHON:-}" ]]; then
  "$BENI_RELEASE_PYTHON" scripts/check-release.py
  "$BENI_RELEASE_PYTHON" scripts/check-local-release.py
  "$BENI_RELEASE_PYTHON" scripts/check-sparkle.py
else
  python3 scripts/check-release.py
  python3 scripts/check-local-release.py
  python3 scripts/check-sparkle.py
fi
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --locked -p buddy-cli -- validate-assets
python3 scripts/check-integrations.py
python3 scripts/check-assets.py
if [[ "${1:-}" == "--desktop" ]]; then
  cargo fmt --manifest-path apps/desktop/Cargo.toml --check
  cargo test --manifest-path apps/desktop/Cargo.toml --locked
  cargo clippy --manifest-path apps/desktop/Cargo.toml --all-targets --locked -- -D warnings
  cargo build --manifest-path apps/desktop/Cargo.toml --locked
fi
