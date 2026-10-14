# Entwicklung und Prüfungen

Rustup installiert die in `rust-toolchain.toml` festgelegte Toolchain (Rust 1.99.0, rustfmt und Clippy). Die Manifeste deklarieren Rust 1.95 als Mindestversion. macOS benötigt die Xcode Command Line Tools, Windows die MSVC Build Tools.

## Zwei Workspaces

Der Root-Workspace enthält Kern, Chatvertrag, CLI und IPC. `apps/desktop` hat einen eigenen Workspace und Lockfile für GUI-Abhängigkeiten. Eine bestandene Kernprüfung bestätigt keinen GUI-Build. Beide Lockfiles verwenden `--locked`.

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --manifest-path apps/desktop/Cargo.toml --check
cargo test --manifest-path apps/desktop/Cargo.toml --locked
cargo clippy --manifest-path apps/desktop/Cargo.toml --all-targets --locked -- -D warnings
cargo run --locked -p buddy-cli -- validate-assets
```

Für Debian/Ubuntu beispielsweise:

```sh
sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-dev libxext-dev libxi-dev libxrandr-dev libxcursor-dev libgl1-mesa-dev
```

## Python-Werkzeuge

Die Release-Prüfungen verwenden die Versionen aus `scripts/requirements-release.txt`; native Installer-Workflows verwenden Python 3.13. Die Rust-Anwendung und ihre Chat-/Animationslogik benötigen keine Python-Laufzeit. Die optionale Linux-Menüintegration verwendet Python 3.

```sh
python3.13 -m venv .venv-release
.venv-release/bin/python -m pip install -r scripts/requirements-release.txt
export BENI_RELEASE_PYTHON="$PWD/.venv-release/bin/python"
./scripts/check.sh
./scripts/check.sh --desktop
```

Die Release-Pipeline bleibt in Python. [Prüfeinstieg und Toolchains](TOOLING.md) beschreibt: derselbe Interpreter für alle Prüfungen, automatische Wahl des Projekt-Venv und frühzeitige Prüfung der festgelegten Abhängigkeiten. Eine vollständige Rust-Portierung braucht einen belegten Wartungsvorteil.

## CI und native Abnahme

Feature-Branches werden über PRs geprüft; Push-Prüfungen laufen nur auf main. Dokumentationsänderungen starten keine Rust-Matrix. Caches werden auf main geschrieben und von PRs gelesen. Regelmässige Prüfungen bauen Debug-Binaries; native Installer entstehen nur bei bewusst gestarteten Installer-Workflows. `workflow_dispatch` kann zusätzlich Entwicklungsarchive bauen, diese sind keine veröffentlichten Installer.

Native Eingaberegionen und X11-Hotkeys: `bash scripts/check-input-linux.sh`, mit eigenem Xvfb-/Weston-Testserver. Weitere Plattform-/Hardwareabnahmen stehen in [TEST_MATRIX.md](TEST_MATRIX.md). CPU-/Repaint-Messungen stehen in [PERFORMANCE.md](PERFORMANCE.md).

[CI und native Prüfungen](CI.md) beschreibt die getrennten Jobs, Pfadfilter, Artefaktübergabe und begrenzten Infrastruktur-Retries. Finale Installer-Abnahmen bleiben Voraussetzung für eine Veröffentlichung.

## Architektur und Distribution

- [Architektur](ARCHITECTURE.md) und [native Rust-Entscheidung](adr/0001-native-rust.md)
- [eframe-Patch: Accessibility und synchrones Speichern](../vendor/eframe/BENI_PATCH.md)
- [Release-Vertrag](RELEASES.md), [Mac-Installer](MACOS_INSTALLER.md) und [lokale Signierung](LOCAL_RELEASE.md)
- [Update-Runbook](UPDATE_RELEASE_RUNBOOK.md), [Windows-Signierung](WINDOWS_SIGNING.md) und [Linux-Installer](PORTABLE_INSTALLERS.md)
- [Masterplan](MASTERPLAN.md) und [Rechte/Provenienz](../LICENSE.md)
