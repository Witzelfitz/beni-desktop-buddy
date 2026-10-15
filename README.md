# Beni Desktop Buddy

Dein kleiner Begleiter auf dem Desktop: eine native Rust-App mit vertrauten Beni-Gesten, lokalen Statusmeldungen und Gesprächen direkt an der Figur.

<img src="docs/images/macos-buddy.png" alt="Beni auf dem macOS-Desktop mit eingeklappter Bedienung" width="340">

**[Herunterladen](https://mini.witzelfitz.io/download)** · [Release 0.1.6](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.6) · [Bedienung](docs/USER_GUIDE.md)

## Was Beni mitbringt

- Menü-Rad mit Fokus-/Pausen-Timer, eigenen Erinnerungen, Verlauf und Ruhe; direkt per Klick auf Beni erreichbar.
- Statusmeldungen von eigenen Werkzeugen und Codex-/Claude-Code-Hooks, mit Meldungsverlauf.
- Separate Einstellungen, Grössenwahl und «Nicht stören».
- Bestehende Animationen, reduzierte Bewegung und durchklickbare transparente Flächen.
- Freiwilliger lokaler Ollama-Chat direkt an der Figur, in Hochdeutsch mit Schweizer Rechtschreibung und passenden Gesten.

## Installieren und loslegen

macOS (Apple Silicon/Intel, ab macOS 15) und Linux (x86_64, GLIBC 2.35+, X11/Wayland) sind veröffentlicht. Windows wartet auf die Herausgebersignierung.

Auf dem Mac aktualisierst du über Benis Menüleistensymbol → «Nach Updates suchen …». Unter Linux ersetzt du das AppImage manuell. Die [Downloadseite](https://mini.witzelfitz.io/download) zeigt die passende Installationshilfe.

Beni anklicken → «Reden» → «Lokales Ollama verbinden» → vorhandenes Chatmodell wählen. [Chat und Datenverarbeitung](docs/CHAT.md). Ollama läuft separat; Beni lädt keine Modelle herunter. Claude und OpenAI folgen später.

Statusempfang und globale Hotkeys sind freiwillig. Gesprächs- und Meldungsinhalte bleiben sitzungsgebunden. Keine Telemetrie, Sprachausgabe oder Mikrofon-/Bildschirmaufnahme durch den Buddy.

## Selbst bauen

Rustup und die nativen Build-Werkzeuge installieren. Die Toolchain und beide Lockfiles sind festgelegt. Plattformabhängigkeiten und alle Prüfungen stehen in [DEVELOPMENT.md](docs/DEVELOPMENT.md).

```sh
# Kern, CLI und IPC ohne Desktopfenster prüfen.
cargo test --workspace --locked
cargo run --locked -p buddy-cli -- validate-assets

# Separaten GUI-Workspace starten.
cargo run --manifest-path apps/desktop/Cargo.toml --locked
```

## Mehr erfahren

- [Bedienung](docs/USER_GUIDE.md), [Chat](docs/CHAT.md) und [Statusintegrationen](docs/INTEGRATIONS.md)
- [Entwicklung](docs/DEVELOPMENT.md) und [Architektur](docs/ARCHITECTURE.md)
- [Masterplan und nächste Schritte](docs/MASTERPLAN.md)
- [Ausgeführte Prüfungen und offene Abnahmen](docs/TEST_MATRIX.md)

Der Rust-Code steht unter [MIT](LICENSE-MIT). Beni-Figur, Grafiken und importierte Animationsdaten behalten ihre [gesonderten Rechte](LICENSE.md).
