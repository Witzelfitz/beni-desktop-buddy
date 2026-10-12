Beni bekommt eine native Rust-Grundlage als animierter Desktop-Buddy. Der bestehende PNG-Atlas mit 44 Posen und 20 Animationsabläufen wird eingebettet; eine unabhängige Engine berechnet die Bewegung. Eine vorbereitete egui/eframe-App zeichnet Beni in einem transparenten, verschiebbaren Fenster und bietet Sprechblase, Animationsauswahl, Pause und lokale Einstellungen.

Kern und CLI sind separat testbar. Die Desktop-App hat einen eigenen Cargo-Workspace, damit GUI-Abhängigkeiten die Headless-Prüfung nicht blockieren. CI prüft beide Workspaces auf macOS, Windows und Linux.

Validierung: 13 Rust-Tests einschliesslich 480 Referenzposen des ursprünglichen SDK, Asset-Hashes, Formatierung und Clippy. Desktop-Build und manuelle GUI-Prüfung sind noch offen: GUI-Abhängigkeiten konnten in der eingeschränkten Vorbereitungssitzung nicht heruntergeladen werden. Der erste verbundene Build muss `apps/desktop/Cargo.lock` erzeugen und committen. Details stehen in `docs/VALIDATION.md` und `docs/TEST_MATRIX.md`.

Offene nächste Schritte: macOS-Abnahme, Tray/Rückholen, Click-through, Grössen-/Monitorsteuerung und anschliessend Packaging/Signierung. Autostart und Fernsteuerung sind noch nicht implementiert.
