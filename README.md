# Beni Desktop Buddy

Mini-Beni als nativer Desktop-Buddy in Rust: ein animierter Begleiter auf dem Desktop, mit transparentem Fenster, Gesten und Sprechblasen.

Dieses Repository startet mit einer Rust-Grundlage. Die erste Desktop-Abnahmeplattform ist macOS; Windows und Linux sind im Build-Workflow vorgesehen.

## Aktueller Stand

- **Implementiert und getestet:** unabhängiger Rust-Animationskern, 44 Posen, 20 Animationen, SDK-kompatible Bewegung, Aliase, One-shot → Idle, Pause, Tempo, Spiegelung, reduzierte Bewegung und Asset-Validierung.
- **Auf macOS gebaut und gestartet:** Desktop-App mit egui/eframe, transparentem Fenster, Verschieben per Ziehen, Sprechblase, Animationsmenü, Einstellungen und lokalem Speichern der Einstellungen/Fensterposition.
- **Vorbereitet:** CI mit Download-Artefakten für macOS, Windows und Linux; macOS-App-Bundle; Roadmap und manuelle Abnahmeliste.
- **GitHub:** [Witzelfitz/beni-desktop-buddy](https://github.com/Witzelfitz/beni-desktop-buddy), privat.

Kein WebView und kein JavaScript zur Laufzeit: Der bestehende Beni-Animationsansatz wird in Rust berechnet und aus dem originalen PNG-Atlas gezeichnet. Grafik und Daten sind im Binary eingebettet.

## Start

Rust stable (mindestens 1.95), Cargo und die üblichen nativen Build-Werkzeuge installieren. Unter macOS benötigt der Build die Xcode Command Line Tools, unter Windows die MSVC Build Tools.

Im Repository:

```sh
# Animationskern prüfen – benötigt kein Desktopfenster.
cargo test --workspace --locked
cargo run --locked -p buddy-cli -- validate-assets
cargo run --locked -p buddy-cli -- states
cargo run --locked -p buddy-cli -- sample wave 321

# Desktop starten. Beim ersten Mal werden die GUI-Abhängigkeiten geladen.
cargo run --manifest-path apps/desktop/Cargo.toml --locked
```

Beide Cargo-Workspaces enthalten ein committedes Lockfile. Builds und Tests verwenden `--locked`, damit dieselben Abhängigkeiten geladen werden.

Unter Debian/Ubuntu braucht die GUI beispielsweise:

```sh
sudo apt-get install libxkbcommon-dev libwayland-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev libgl1-mesa-dev
```

Linux-Desktop-/Compositor-Verhalten muss separat abgenommen werden. Ein erfolgreicher Build belegt noch keine Transparenz oder Vordergrunddarstellung.

## Bedienung

| Aktion | Bedienung |
| --- | --- |
| Beni verschieben | Beni mit der linken Maustaste ziehen |
| Begrüssen | Beni oder «Hoi!» anklicken |
| Pause/Fortsetzen | «Pause» / «Weiter» |
| Animation auswählen | «Menü» → Animation |
| Energie, Tempo, Spiegelung | «Menü» |
| Weniger Bewegung | «Menü» → «Weniger Bewegung» |
| Eigene Sprechblase | Nachricht im Menü eingeben und anzeigen |
| Vordergrunddarstellung | «Menü» → «Immer im Vordergrund» |
| Menü schliessen | Escape |
| Beenden | «Beenden» oder Alt/Option + Q bei fokussiertem Buddy-Fenster |

Vier Desktop-UI-Tests prüfen die tatsächlichen egui-Ereignisse für Menü, Pause/Fortsetzen, Nachricht, reduzierte Bewegung, Dragging und Beenden. Fenster-/Compositor-Verhalten auf Windows/Linux sowie die vollständige manuelle Abnahme bleiben separat zu prüfen.

Die Sprechblase zeigt Text, ohne Audioausgabe. Einzelaktionen kehren nach einem Durchlauf zu «warten» zurück. Einstellungen werden lokal gespeichert. Idle zeichnet mit bis zu 15 fps, aktive Gesten mit bis zu 30 fps; Pause und reduzierte Bewegung mit bis zu 4 fps.

Das MVP hat noch kein Tray-Menü, keine globalen Hotkeys, keinen Autostart und keine Click-through-Flächen. Diese Funktionen folgen erst nach der Desktop-Abnahme.

## Entwicklung

```sh
./scripts/check.sh
./scripts/check.sh --desktop
./scripts/package-macos.sh
```

Das macOS-Script erstellt nach erfolgreichem Build `dist/Beni Buddy.app`. Dieses Entwicklungsbundle ist noch nicht signiert/notarisiert. Es führt keinen Autostart oder OS-Installation aus.

Der Kern liegt im Root-Workspace. Die Desktop-App bildet wegen ihrer eigenen GUI-Abhängigkeiten einen separaten Cargo-Workspace. Root-Tests bauen deshalb ausdrücklich **nicht** die Desktop-App. Der GitHub-Workflow enthält einen zusätzlichen Desktop-Job auf allen drei Betriebssystemen. Erfolgreiche Builds stellen Download-Artefakte bereit: ein `.app`-Bundle für macOS und ausführbare Dateien für Windows/Linux, jeweils in einem `.tar.gz`-Archiv.

## Struktur

```text
assets/beni/                  Originalatlas, Animationen, Aliase und SHA-256-Provenienz
crates/buddy-core/            Deterministische Rust-Engine und Tests
crates/buddy-cli/             Headless-CLI und Prozess-Integrationstests
apps/desktop/                Native egui/eframe-App
.github/workflows/ci.yml      Kern- und Desktopchecks auf drei Plattformen
scripts/                     Prüfungen, GitHub-Veröffentlichung und macOS-Packaging
docs/                       Architektur, Roadmap, GitHub-Anleitung, Abnahme, Prüfbericht
```

- [Architektur](docs/ARCHITECTURE.md)
- [Entscheidung für native Rust-GUI](docs/adr/0001-native-rust.md)
- [Roadmap mit konkreten Arbeitspaketen](docs/ROADMAP.md)
- [GitHub-Erstellung und PR](docs/GITHUB.md)
- [Manuelle Desktop-Abnahme](docs/TEST_MATRIX.md)
- [Tatsächlich ausgeführte Prüfungen](docs/VALIDATION.md)
- [Rechte und Asset-Herkunft](LICENSE.md)

Technische Referenzen: [eframe 0.36.2](https://docs.rs/eframe/0.36.2/eframe/), [transparentes Fenster](https://docs.rs/egui/0.36.2/egui/viewport/struct.ViewportBuilder.html#method.with_transparent), [native Fensteraktionen](https://docs.rs/egui/0.36.2/egui/viewport/enum.ViewportCommand.html).
