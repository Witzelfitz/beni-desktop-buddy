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

## Sprechblase und plattformübergreifende Statusintegration

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `./scripts/check.sh --desktop` | 42 Rusttests: 10 Engine-, 6 Protokoll-/Queue-/Regeltests, 5 CLI-Prozesstests, 6 echte IPC-Tests und 15 Desktoptests; Formatierung, beide Clippy-Prüfungen, Asset-Hashes und Debug-Build |
| PASS | `python3 scripts/check-integrations.py` | Fünf Tests für Hook-Filter, untrusted Input, nicht verfügbare CLI, JSON-Vorlage sowie Exitcodes/Argumentgrenzen des POSIX-Wrappers |
| PASS | Gestenregeln | 52 deutsche/englische Statusbeispiele; explizite Geste hat Vorrang, unbekannte/unerlaubte Animationen abgelehnt |
| PASS | Echter Unix-Transport | Annahme/Ack, Normalisierung, Wecken, Überlastung, falsches/grosses JSON, Idle-Client, Timeout, mehrere Listener, Abschalten/Neustart, sichere Dateirechte und verwaiste Sockets |
| PASS | Windows-Cross-Compile | `cargo check -p buddy-ipc --target x86_64-pc-windows-gnu --locked` sowie Clippy für IPC/CLI und alle Targets; prüft Windows-Code, führt ihn nicht aus |
| PASS | macOS-Release-Bundle | Desktop, CLI, Wrapper, Hook-Vorlage und Integrationsanleitung gebündelt |
| PASS | Native macOS-Sprechblase | Lange Meldung, Scrollen bis «LETZTE ZEILE», Schliessen, Quellenname, sichtbare Geste und weiterhin sichtbare Bedienleiste |
| PASS | Native macOS-IPC | Empfang standardmässig aus; Aktivieren, Abschalten (Exitcode 3), erneutes Aktivieren und Erhalt der Freigabe nach Neustart |
| PASS | Verdeckte Darstellung | Via Tray ausgeblendet, Meldung angenommen, erst nach erneutem Anzeigen zugestellt |
| PASS | Lokale CLI eingerichtet | `cargo install --path crates/buddy-cli --locked`; `beni-buddy-cli` im Cargo-Binärpfad |
| NOT_RUN | Native Windows-/Linux-GUI und echte Integrationsregistrierung | Keine Windows-/Linux-Desktop-Sitzung; bestehende Claude-/OS-Konfigurationen nicht verändert |

Die CI führt zusätzlich native IPC-/CLI-Tests und Wrapperprüfungen auf macOS, Linux und Windows aus und erstellt Archive mit Desktop, CLI und Vorlagen. Der aktuelle Status steht bei den [PR-/Branch-Prüfungen](https://github.com/Witzelfitz/beni-desktop-buddy/actions?query=branch%3Afeat%2Flocal-status-integrations). Auf Windows entfallen die beiden Unix-Dateirechte-Tests; die Named-Pipe-Tests sowie der PowerShell-Wrapper laufen dort tatsächlich auf dem Windows-Runner.

Die neuen Sprechblasentests prüfen die Erreichbarkeit der letzten Textzeile, das Schliessen und die Fenstergrenzen bei maximalem Quellennamen. Ein weiterer Test sendet über einen echten lokalen Listener, führt den Logikpfad bei verstecktem Fenster aus und prüft anschliessend Text, Quelle, Geste und Abschalten. Meldungsinhalte und Warteschlange werden nicht persistiert. Die lokale macOS-Instanz hat für die Einrichtung Statusmeldungen aktiviert; neue Installationen bleiben standardmässig ausgeschaltet.

## Codex-Statusadapter

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `python3 scripts/check-integrations.py` | 11 Tests einschliesslich vier Codex-Ereignissen, sicheren Argumenten, Eingabe-/Antwortschutz, getrennten Sitzungsquellen, grossen/defekten Payloads, optionaler CLI, leerem JSON und erhaltenden/idempotenten Installer-Aufrufen |
| PASS | Installierte Codex-Version | Lokales CLI und laufender App-Server 0.162.1; Hooks-Feature aktiv |
| PASS | Lokale Registrierung | Vier Benutzer-Hooks in `~/.codex/hooks.json`, Adapter unter `~/.codex/hooks/beni-status.py`; Konfiguration und Script mit Dateirechten 0600 |
| PASS | Tatsächlicher App-Server `hooks/list` | Vier Definitionen erkannt, keine Warnungen/Fehler; jede Definition hat `trustStatus: untrusted` |
| PASS | Direkter macOS-End-to-end-Test | Installiertes Python-Script mit synthetischem Stop-Payload liefert `{}`; Beni zeigt «Codex hat geantwortet» mit Quellenkennung und Daumen-Geste |
| NOT_RUN | Automatische Codex-Turn-Ereignisse | Benötigen die Nutzerprüfung der vier Definitionen über `/hooks`; kein Trust-Bypass verwendet |
| NOT_RUN | Codex-Laufzeit auf Windows/Linux und Cloud-Orchestrierung | Portable Hook-/Installer-Tests laufen in CI; reale Codex-Integration dort nicht ausgeführt |

Die direkten Script-Tests bestätigen den Weg Script → CLI → Beni; sie belegen noch keine automatische Ausführung durch Codex. Ein Probeaufruf des bestehenden Daemon-Proxys lief in einen Timeout. Der anschliessende read-only `hooks/list`-Aufruf über einen temporären stdio-App-Server bestätigte die Registrierung. Der laufende Daemon wurde nicht neu gestartet und es wurde keine zusätzliche Modellsitzung erzeugt.

## Einklappbare Bedienleiste

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `./scripts/check.sh --desktop` | 44 Rusttests (27 Root, 17 Desktop), 11 Integrationstests, Formatierung, Asset-Hashes, beide Clippy-Prüfungen und Debug-Build |
| PASS | UI-Ereignisse | Start eingeklappt; Avatar-Klick klappt auf/zu, ohne Text, Quelle, Geste oder Frist einer aktiven Meldung zu ersetzen; Avatar-Geometrie bleibt gleich |
| PASS | Bedienregressionen | Escape schliesst Einstellungen und Leiste; Dragging klappt die Leiste nicht auf; bei Zeichenfehlern öffnet der Ersatzavatar die Bedienung weiterhin |
| PASS | `./scripts/package-macos.sh` | Release-Build und valides Bundle; lokale laufende Instanz aktualisiert |
| PASS | Native macOS-Prüfung | Start eingeklappt, Avatar-Klick auf/zu, Einstellungen öffnen und Escape im fokussierten Fenster; eigene Fensteraufnahmen bestätigen transparenten Bereich statt Leiste |
| PASS | Status nach Aktualisierung | Installierte CLI liefert Annahme; Quellenname und Statusblase erscheinen weiterhin bei eingeklappter Bedienung |
| NOT_RUN | Native Windows-/Linux-GUI | Derselbe egui-Code; lokale Desktop-Abnahme auf diesen Plattformen weiterhin offen |

Die Bedienleiste behält ihren Platz im transparenten Fenster, auch wenn sie eingeklappt ist. Dadurch verändert das Auf-/Zuklappen weder Benis Grösse noch seine Position. Der offene Zustand wird nicht gespeichert: Nach einem Neustart ist die Leiste wieder eingeklappt. Transparente Fensterbereiche sind weiterhin nicht durchklickbar.

## Native Accessibility-Anbindung

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `./scripts/check.sh --desktop` | 47 Rusttests (27 Root, 20 Desktop), 11 Integrationstests, Formatierung, Asset-Hashes, beide Clippy-Prüfungen und Debug-Build |
| PASS | Accessibility-Baum und Aktionen | Beni als benannter Button mit Auf-/Zuklappzustand und Focus/Click; Bedienelemente erscheinen erst beim Aufklappen; benannter Schliessen-Knopf schliesst die Blase ohne Pointer-Ereignisse |
| PASS | Tastaturereignisse | Tab fokussiert Beni bei eingeklappter Leiste; Eingabe und Leertaste klappen auf/zu und erzeugen keine Drag-Anforderung |
| PASS | Einstellungen | Benannte Checkboxen und Nachrichtenfeld mit TextInput-Rolle, festem Namen und aktuellem Eingabewert |
| PASS | `./scripts/package-macos.sh` | Release-Build und valides Bundle mit nativer AccessKit-Anbindung; lokale Instanz aktualisiert |
| PASS | Native macOS-AX-API | Benis eigener Fensterbaum enthält benannte Buttons, Textfeld, Checkboxen und Statuslabel; AXPress öffnet/schliesst die Leiste, bedient Pause/Weiter und Einstellungen und schliesst eine echte IPC-Meldung |
| NOT_RUN | Vollständige VoiceOver-/NVDA-/Orca-Abnahme | Keine globale Screenreader-Einstellung verändert; gesprochene Ausgabe und vollständige Navigation separat prüfen |
| NOT_RUN | Native Windows-/Linux-Accessibility-Abnahme | Die Plattformadapter sind im Desktop-Lockfile erfasst; die reale Desktop-Abnahme bleibt offen |

Aktiviert wurde ausschliesslich das native `accesskit`-Feature von eframe. Rust-Version und Packaging-Signierung bleiben getrennte Folgeschritte. Die Klickfläche der Sprechblase ist ein Maus-Kürzel; der benannte Schliessen-Button übernimmt Tastatur- und Accessibility-Zugriff, damit der Blasenhintergrund keinen unbeschrifteten Tab-Stopp erzeugt.
