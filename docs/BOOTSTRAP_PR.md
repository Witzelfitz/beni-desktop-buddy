Beni bekommt eine native Rust-Grundlage als animierter Desktop-Buddy. Der bestehende PNG-Atlas mit 44 Posen und 20 Animationsabläufen wird eingebettet; eine unabhängige Engine berechnet die Bewegung. Eine vorbereitete egui/eframe-App zeichnet Beni in einem transparenten, verschiebbaren Fenster und bietet Sprechblase, Animationsauswahl, Pause und lokale Einstellungen.

Kern und CLI sind separat testbar. Die Desktop-App hat einen eigenen Cargo-Workspace, damit GUI-Abhängigkeiten die Headless-Prüfung nicht blockieren. CI prüft beide Workspaces auf macOS, Windows und Linux.

Validierung auf macOS/Apple Silicon: 13 Kern-/CLI-Tests einschliesslich 480 Referenzposen des ursprünglichen SDK, vier Desktop-UI-Tests mit echten egui-Ereignissen, Clippy ohne Warnungen, Asset-Hashes, Formatierung sowie Debug- und Release-Build. Die native App wurde gestartet und ihr transparentes Avatarfenster visuell geprüft. Beide Cargo-Workspaces enthalten committete Lockfiles. Das macOS-App-Bundle wurde gebaut. Details stehen in `docs/VALIDATION.md` und `docs/TEST_MATRIX.md`.

Offene nächste Schritte: vollständige manuelle Plattform-Abnahme, Tray/Rückholen, Click-through, Grössen-/Monitorsteuerung und anschliessend Packaging/Signierung. Autostart und Fernsteuerung sind noch nicht implementiert.
