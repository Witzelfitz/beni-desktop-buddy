# Prüfbericht · 12. Oktober 2026

Ausführungsumgebung: macOS auf Apple Silicon, Rust/Cargo 1.99.0. GitHub- und Crates.io-Zugriff sind verfügbar. Beide Cargo-Workspaces enthalten committete Lockfiles.

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `./scripts/check.sh --desktop` | Kern-/CLI-Tests, Formatierung, Clippy, Asset-Hashes sowie Desktop-Test/Clippy/Debug-Build |
| PASS | `cargo test --workspace --locked` | 10 Kern- und 3 CLI-Prozesstests; darin 480 Referenzposen des Original-SDK |
| PASS | `cargo test --manifest-path apps/desktop/Cargo.toml --locked` | Vier egui-UI-Tests: Bedienelemente im Fenster, Menü/Nachricht/reduzierte Bewegung, Pause/Fortsetzen/Beenden und native Drag-Anforderung |
| PASS | Clippy für beide Workspaces, alle Targets, `-D warnings` | Keine Warnungen |
| PASS | `python3 scripts/check-assets.py` | SHA-256 der vier übernommenen Dateien stimmen; 44 Posen und 20 Animationen |
| PASS | `./scripts/package-macos.sh` | Release-Binary und `dist/Beni Buddy.app` erstellt; Info.plist validiert |
| PASS | Nativer macOS-Start | Transparentes 340 × 440-Punkte-Fenster mit Beni und sichtbarer Bedienleiste; CGWindow-Abfrage und gezielter Fenster-Screenshot |
| PASS | PNG-Alpha der Fensteraufnahme | RGBA-Aufnahme; Hintergrundpixel ausserhalb der UI haben Alpha 0 |
| PASS | GitHub-API | Privates Repository `Witzelfitz/beni-desktop-buddy` erstellt und per API bestätigt |
| PENDING | GitHub Actions | Workflow vorbereitet; Resultate am Pull Request prüfen |
| NOT_RUN | Vollständige manuelle Testmatrix | Windows/Linux-Desktop, Monitorwechsel, Langzeitlast und Installer-Abnahme sind offen |

Die Desktop-UI-Tests führen denselben `egui::Context::run_ui`-Pfad aus wie eframe und senden Pointer-Ereignisse. Sie prüfen UI-Aktionen und daraus entstehende native Fensterkommandos; sie ersetzen keine Prüfung des Betriebssystem-/Compositor-Verhaltens.

## Nächster Entwicklungsschritt

Tray und verlässliche Rückholmöglichkeit, Click-through, Grössen-/Monitorsteuerung sowie die vollständige Plattform-Abnahme. Das lokale macOS-Bundle ist ein Entwicklungsbundle ohne Notarisierung.
