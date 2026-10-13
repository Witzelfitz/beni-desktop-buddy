#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != "Linux" ]]; then echo "Dieser Check benötigt Linux." >&2; exit 1; fi
xvfb-run -a env BENI_INPUT_BACKEND=x11 cargo test --manifest-path apps/desktop/Cargo.toml --locked native_linux_input_regions -- --ignored --test-threads=1
beni_runtime="$(mktemp -d)"
chmod 700 "$beni_runtime"
env XDG_RUNTIME_DIR="$beni_runtime" weston --backend=headless-backend.so --use-pixman --no-config --socket=beni-input-test --idle-time=0 --log="$beni_runtime/weston.log" &
beni_weston_pid=$!
trap 'kill "$beni_weston_pid" 2>/dev/null || true; wait "$beni_weston_pid" 2>/dev/null || true; rm -rf "$beni_runtime"' EXIT
for ((beni_attempt=0; beni_attempt<50; beni_attempt++)); do
  if [[ -S "$beni_runtime/beni-input-test" ]]; then break; fi
  if ! kill -0 "$beni_weston_pid" 2>/dev/null; then cat "$beni_runtime/weston.log"; exit 1; fi
  sleep 0.1
done
if [[ ! -S "$beni_runtime/beni-input-test" ]]; then cat "$beni_runtime/weston.log"; exit 1; fi
env XDG_RUNTIME_DIR="$beni_runtime" WAYLAND_DISPLAY=beni-input-test BENI_INPUT_BACKEND=wayland cargo test --manifest-path apps/desktop/Cargo.toml --locked native_linux_input_regions -- --ignored --test-threads=1
