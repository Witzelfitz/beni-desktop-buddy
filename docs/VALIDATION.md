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
| PASS | [GitHub Actions](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38248825723) | Alle sechs Jobs bestanden: Kern/CLI und Desktop auf macOS, Windows und Linux; drei Download-Artefakte erstellt |
| NOT_RUN | Vollständige manuelle Testmatrix | Windows/Linux-Desktop, Monitorwechsel, Langzeitlast und Installer-Abnahme sind offen |

Die Desktop-UI-Tests führen denselben `egui::Context::run_ui`-Pfad aus wie eframe und senden Pointer-Ereignisse. Sie prüfen UI-Aktionen und daraus entstehende native Fensterkommandos; sie ersetzen keine Prüfung des Betriebssystem-/Compositor-Verhaltens.

Die plattformübergreifende CI hat den Implementierungsstand `0bc6d05` geprüft. Der nachfolgende Abschlusscommit aktualisiert ausschliesslich Dokumentation und Vorschau; er verändert keinen Programmcode oder Build-Workflow.

## Punkt 1 · Zuverlässigkeit

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `./scripts/check.sh --desktop` | 13 Kern-/CLI-Tests und 13 Desktoptests, Formatierung, Clippy ohne Warnungen, Asset-Hashes und Debug-Build |
| PASS | Fehlerregressionen | Fehler laufen nach sechs Sekunden ab; erfolgreiche Aktion löscht Fehler; wiederholter Zeichenfehler verlängert die Frist nicht; Dragging, Menü und Beenden bleiben erreichbar |
| PASS | Tray ohne Renderpass | Anzeigen, Ausblenden, Rückholen, Pause/Fortsetzen, Einstellungen und Beenden über echten egui-Logikpfad mit begrenztem Nachrichtenkanal |
| PASS | Positionierungsregeln | Gültige Position beibehalten, entfernten Monitor korrigieren, negative Monitorposition, Teilüberlappung, Hauptbildschirm, kleine/ungültige/fehlende Geometrie |
| PASS | `./scripts/package-macos.sh` | Neuer Release-Build und valides Entwicklungsbundle |
| PASS | Native macOS-Trayprüfung | Ausblenden/Anzeigen, Pause/Fortsetzen (sichtbare «Weiter»-Schaltfläche), Einstellungen, explizites Rückholen und Beenden bei ausgeblendetem Fenster |
| PASS | Neustart mit Offscreen-Position | Gespeicherte Position `{99999, 99999}` wird sichtbar wiederhergestellt; Nutzereinstellungen danach aus Sicherung restauriert |
| NOT_RUN | Physisches Abhängen / gemischte DPI | Automatische Regeln geprüft; reale Mehrmonitor-Abnahme offen |
| NOT_RUN | Windows-/Linux-Desktop-Abnahme | Kein entsprechender lokaler Desktop vorhanden |

Das Tray wird auf dem nativen Hauptthread erzeugt. Ereignisse wecken die App auch ohne sichtbaren Renderpass. macOS-Arbeitsbereiche werden in logischen Cocoa-Punkten inklusive Dock und Menüleiste berücksichtigt. Unter Wayland bleibt Positionierung vom Compositor abhängig. Linux benötigt einen StatusNotifier-Host; ein fehlender Host ist nicht zwingend als Fehler bei der Tray-Erzeugung erkennbar. Bei erkanntem Tray-Fehler bleibt eine normale Titelleiste verfügbar; unter macOS zusätzlich der Dock-Eintrag.

Nächste getrennte Pakete: Sprechblase, lokale IPC-Steuerung und danach optional die Emotionserkennung. Click-through, Grössenwahl und vollständige Plattform-Abnahme bleiben offen. Das lokale macOS-Bundle ist ein Entwicklungsbundle ohne Notarisierung.
