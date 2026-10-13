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

## Feste Rust-Toolchain

- PASS: Rust 1.99.0, rustfmt und Clippy aus `rust-toolchain.toml` installiert; lokale aktive Toolchain ist `1.99.0-aarch64-apple-darwin`.
- PASS: Derselbe explizite CI-Installationsbefehl `rustup toolchain install --no-self-update` verwendet die aktive Version aus der Projektdatei; kein Update des beweglichen stable-Kanals.
- PASS: `./scripts/check.sh --desktop` mit der versionierten Toolchain: 47 Rusttests, 11 Integrationstests, Formatierung, Clippy, Asset-Hashes und Debug-Build.

Die Mindestversion 1.95 in den Manifesten bleibt bestehen. Die festgelegte Entwicklungstoolchain ist 1.99.0; eine vollständige Prüfung der Mindestversion ist hier nicht ausgeführt worden.

## App-Icon und Ad-hoc-Signierung

- PASS: Originalatlas und Animationsdaten unverändert; fünf Icon-Dateien mit SHA-256-Provenienz. PNG-Grössen/RGBA, ICNS-Container und sieben ICO-Auflösungen geprüft.
- PASS: `./scripts/check.sh --desktop`: 47 Rusttests, 11 Integrationstests, Formatierung, Clippy ohne Warnungen, Asset-/Icon-Prüfung und Debug-Build.
- PASS: `./scripts/package-macos.sh`: Release-Build; beide Binaries und fertiges Bundle ad hoc signiert; `codesign --verify --deep --strict` erfolgreich.
- PASS: Kopie des Bundles mit absichtlich verändertem Ressourcentext wird bei strenger Signaturprüfung abgelehnt. Das ausgelieferte Bundle wurde nicht verändert.
- PASS: ICNS im Bundle und `CFBundleIconFile` korrekt; macOS `NSWorkspace.icon(forFile:)` liefert das Beni-Icon für den Finder.
- NOT_RUN: öffentliche Gatekeeper-/Notarisierungsabnahme auf einem frischen Gerät, native Windows-Explorer-/Linux-Launcher-Abnahme. Die Windows-Exe-Ressource wird im nativen Windows-CI-Build kompiliert.

Master und Exportformate liegen in `assets/app-icon/`; das Exportskript benötigt macOS mit sips/iconutil und keine Python-Pakete. Die ursprünglichen Beni-Grafiken wurden nicht überschrieben.

## Durchklickbare transparente Flächen

- PASS: `./scripts/check.sh --desktop`: 51 Rusttests (27 Root, 24 Desktop), 11 Integrationstests, Formatierung, Clippy ohne Warnungen, Asset-/Icon-Prüfung und Debug-Build.
- PASS: Eingabegeometrie prüft Sprite-Alpha, Rotation, runde UI-Ecken, versteckte Elemente und sortierte, nicht überlappende Rechtecke bei gebrochener DPI-Skalierung. Sichtbare Blase, Bedienleiste und Einstellungen sind in der Eingabemaske enthalten.
- PASS: native macOS-Prüfung mit eigenem Testfenster direkt hinter Beni: transparente Fläche und eingeklappte Leiste geben Klicks weiter; Beni fängt seinen Klick selbst ab und klappt die Leiste auf/zu; Schliessen einer echten IPC-Meldung bleibt bedienbar. Temporäres Testfenster anschliessend beendet.
- PASS: natives Dragging bei aktiver Durchklick-Funktion; ursprüngliche Fensterposition anschliessend wiederhergestellt.
- PASS: Einstellung am laufenden Mac-Buddy ausgeschaltet und wieder eingeschaltet: im ausgeschalteten Zustand erreicht der Klick das Testfenster nicht, nach erneuter Aktivierung wieder schon.
- PASS: neues macOS-Release-Bundle ad hoc signiert und streng geprüft; lokale Buddy-Instanz aktualisiert.
- NOT_RUN (lokal): Windows-/Linux-Desktop-Abnahme und gemischte DPI. Linux-CI führt einen gezielten nativen Backend-Test mit Xvfb und Weston Headless aus; die lokalen Regeln ersetzen diesen Test nicht.

Die Einstellung ist standardmässig aktiv und wird gespeichert. Ohne Tray oder bei Fehlern bleibt normale Bedienung erhalten. macOS verwendet passive lokale/globale Mausmonitore auf dem Hauptthread und gibt lokale Ereignisse unverändert weiter. Beim Abschalten ist die Positionsauswertung deaktiviert, die bestehenden Monitore werden für eine erneute Aktivierung behalten und bei Programmende entfernt. Es werden keine Tastaturereignisse überwacht oder Eingabedaten gespeichert. Windows verwendet native Fensterregionen; X11 nur die Input Shape und Wayland die kopierte Oberflächen-Eingaberegion. Der Animationskern bleibt davon unabhängig.

## Gespeicherte Grössenwahl · 13. Oktober 2026

- PASS: `./scripts/check.sh --desktop`: 55 Rusttests (27 Root, 28 Desktop), 11 Integrationstests, Formatierung, Clippy ohne Warnungen, Asset-/Icon-Prüfung und Debug-Build.
- PASS: echte UI-Auswahl aller drei Grössen; genau ein Resize-Auftrag pro Änderung, kein fortlaufendes Resize. Text, Quelle, Geste, Pausezustand und Frist einer aktiven Meldung bleiben erhalten. Die ausgewählte Grösse wird im Accessibility-Baum als aktivierter Radio-Button angezeigt.
- PASS: Speicherung aller Grössen über den App-Save-Pfad; vorhandene Animations-/IPC-Einstellungen bleiben erhalten. Fehlende, unbekannte, typfalsche und beschädigte gespeicherte Werte fallen auf «Mittel» zurück.
- PASS: Avatar, Bedienleiste, Blase und Einstellungsmenü bleiben in allen drei Grössen erreichbar und in der Eingabemaske enthalten. Lange Blasen scrollen bis zur letzten Zeile; das Menü lässt sich in allen Grössen scrollen und bedienen.
- PASS: natives macOS-Release-Bundle ad hoc signiert und streng geprüft. Fensterabmessungen nach Auswahl: Klein 300 × 360, Mittel 340 × 440, Gross 420 × 560 Punkte. Beim Vergrössern am Bildschirmrand wurde die Position in den sichtbaren Arbeitsbereich korrigiert.
- PASS: eigenes Testfenster hinter Beni erhält in jeder Grösse Klicks auf transparente Flächen. Testfenster danach entfernt.
- PASS: reale Neustarts mit jeder gespeicherten Grösse; Energie, Tempo, reduzierte Bewegung, Durchklicken und IPC-Freigabe erhalten. Beim dritten schnellen Neustart meldete LaunchServices einmal `-600`; erneutes Öffnen als neue Instanz war erfolgreich. Zum Abschluss wieder «Mittel», Bedienung eingeklappt und vorherige sichtbare Fensterposition wiederhergestellt; echte IPC-Meldung angenommen und angezeigt.
- NOT_RUN: native Windows-/Linux-Desktop-Abnahme der Grössenwahl, physische Monitorwechsel und gemischte DPI. CI prüft diese Branch-Änderungen separat auf allen drei Plattformen.

Die Grössenwahl liegt in der Desktop-App und wird separat unter `buddy-size` gespeichert. Der Animationskern und die Hook-Konfiguration bleiben unverändert. Schrift und Bedienelemente werden nicht mitverkleinert; das Einstellungsmenü passt sich der verfügbaren Fensterfläche an.

## Freiwillige globale Hotkeys · 13. Oktober 2026

- PASS: `./scripts/check.sh --desktop`: 65 Rusttests (27 Root, 38 Desktop), 11 Integrationstests, Formatierung, Clippy ohne Warnungen, unveränderte Original-/Icon-Assets und Debug-Build.
- PASS: Eingabeprüfung für Modifikatoren, Funktionstasten, Aliase, doppelte Kombinationen, Steuerzeichen, überlange/ungültige Werte und ausgeschlossene Medientasten. Fehlende oder beschädigte gespeicherte Konfiguration bleibt ausgeschaltet.
- PASS: Backend-Tests mit tatsächlichem Registrierungszustand eines Fake-Backends: Umbelegen, Vertauschen, Konflikt bei der zweiten Kombination, Rücknahme bereits registrierter Kandidaten, Fehler bei Freigabe und Freigabe beim Drop.
- PASS: unbekannte Ereignisse, Key-up und wiederholtes Key-down lösen keine zusätzliche Aktion aus. Veraltete wartende Aktionen werden nach Umbelegen/Abschalten verworfen. Die Aktionen erreichen den Logikpfad auch bei ausgeblendetem Buddy ohne Zeichendurchlauf.
- PASS: Einstellungsgruppe über echte Accessibility-Aktion aufgeklappt; benannte Eingabefelder und Checkbox, ausgeschalteter Startzustand sowie deaktivierte Bedienelemente bei fehlender Plattformunterstützung geprüft.
- PASS: neues macOS-Release-Bundle ad hoc signiert und streng geprüft. Im laufenden Buddy aktiviert, Eingabefeld über fokussierte Tastatureingaben bearbeitet und gespeicherte Aktivierung/Kombinationen nach Neustart wiederhergestellt.
- PASS: finale exklusive macOS-Registrierung gegen einen zweiten Prozess geprüft: konkurrierende Registrierung liefert `eventHotKeyExistsErr` (-9878); Abschalten gibt die Kombination frei. Eine blockierte zweite Kombination aktiviert kein halbes Paar und gibt den ersten Kandidaten frei. Bei gescheitertem Umbelegen bleiben die bisherigen Kombinationen aktiv; nach Freigabe durch den zweiten Prozess gelingt das Umbelegen, und die alte Kombination ist frei.
- PASS: echte globale Pause aus einem eigenen anderen Vordergrundprozess, vier wiederholte Key-down-Ereignisse mit nur einem Umschalten sowie Fortsetzen beim nächsten Tastendruck. Über echtes Tray ausgeblendeten Buddy mit dem globalen Rückhol-Kürzel wieder sichtbar gemacht; Fensterlisten nach Titel und PID gefiltert, damit das Testfenster nicht mitgezählt wird.
- PASS: Testprozesse beendet, vorgeschlagene Kombinationen wiederhergestellt und Hotkeys zum Abschluss ausgeschaltet; Bedienung eingeklappt. Vorhandene Grössenwahl «Klein» und die bisherigen Einstellungen erhalten.
- NOT_RUN (lokal): native Windows-/Linux-Hotkeys, physische Tastaturlayouts, macOS-Spaces-/Vollbild-Abnahme und gemischte DPI. Linux-CI führt zusätzlich eine echte X11-Registrierungs-/Konflikt-/Freigabeprüfung in Xvfb aus.
- OFFEN: Wayland-Portal-Unterstützung. Unter Wayland ist die Aktivierung mit einem sichtbaren Hinweis deaktiviert; XWayland wird in einer Wayland-Sitzung ebenfalls nicht als systemweite Unterstützung angeboten.

Die Bibliothek `global-hotkey` 0.8.0 installiert ihren Prozess-Callback über `OnceCell`; Beni installiert ihn deshalb einmal und ändert anschliessend nur die begrenzte Ereignisroute. Native Registrierungen werden erst nach Aktivierung erstellt. macOS verwendet den Bibliotheks-Eventhandler mit eigenen exklusiven Carbon-Registrierungen (`kEventHotKeyExclusive`, dokumentiert in Apples SDK `CarbonEvents.h`): Die normale nicht exklusive Registrierung würde Konflikte mit anderen Apps nicht melden. Referenzen bleiben bis zur erfolgreichen Freigabe im Besitz des Backends. Bei Freigabefehlern werden Aktionen deaktiviert und ein Neustart empfohlen.

Gespeichert wird ausschliesslich die akzeptierte Hotkey-Konfiguration unter `global-hotkeys`. Tastatureingaben werden nicht protokolliert; Medien-/Event-Tap-Tasten bleiben ausgeschlossen. Animationskern, Statusprotokoll und bestehende Codex-Hook-Konfiguration wurden nicht verändert.

Linux/X11-Korrektur vor dem Merge: Der native CI-Test zeigte, dass `global-hotkey` 0.8.0 `XUngrabKey` puffert und ohne weiteren Request nicht an den Server sendet. Blosses Warten löst das nicht. Das Linux-Backend führt nach der Freigabe einen zweiten, idempotenten Unregister-Aufruf aus: Dessen Keyboard-Mapping-Roundtrip auf derselben Verbindung bestätigt die erste Freigabe am Server. Der native Test verwendet Benis Backend und prüft zusätzlich Rücknahme eines Kandidaten bei Konflikt, Erhalt des bisherigen Paars und Freigabe beim Drop. macOS und Windows verwenden weiterhin ihre bisherigen Freigabepfade.

## Nicht stören · 13. Oktober 2026

- PASS: `./scripts/check.sh --desktop`: 69 Rusttests (27 Root, 42 Desktop), 11 Integrationstests, Formatierung, Clippy ohne Warnungen, Asset-/Icon-Prüfung und Debug-Build.
- PASS: ausgeschalteter Standardzustand, sichere Behandlung beschädigter Speicherung und Speicherung beider Schalterzustände. IPC-Empfang und Animationspause bleiben unabhängig.
- PASS: laufende Remote-Blase wird beim Einschalten geschlossen und die Geste auf «warten» gesetzt; eine vorhandene Animationspause bleibt erhalten. Fehleranzeigen und manuell ausgelöste Nachrichten bleiben bedienbar.
- PASS: Checkbox über Accessibility-Aktion in allen drei Grössen bedient; Modushinweis und Bedienleiste innerhalb des Fensters. Tray-Aktionen funktionieren im Logikpfad auch bei verstecktem Fenster ohne Zeichendurchlauf.
- PASS: echter lokaler Listener nimmt Meldungen beider Prioritäten bei aktivem Modus an. Nach Ablauf einer kurzen TTL und Ausschalten des Modus kommen nur die übrigen gültigen Meldungen in Prioritätsreihenfolge zur Anzeige.
- PASS: natives macOS-Release-Bundle ad hoc signiert und streng geprüft. Echtes Tray schaltet den Modus, Buddy bleibt sichtbar; hohe und normale IPC-Meldung warten. Beim Ausschalten erscheint die gültige Meldung, die abgelaufene bleibt aus. Erneutes Einschalten schliesst die aktive Statusblase.
- PASS: aktiviertes «Nicht stören» nach echtem, geordnetem Neustart wiederhergestellt. Die native Accessibility-Checkbox schaltet den Modus wieder aus. Vorhandene Grössenwahl «Klein», reduzierte Bewegung, Durchklicken, Vordergrund und Empfang bleiben erhalten; Hotkeys bleiben ausgeschaltet. Abschliessende echte IPC-Meldung angezeigt, «Nicht stören» aus, Bedienung eingeklappt.
- NOT_RUN: native Windows-/Linux-Desktop-Abnahme. Die gemeinsame Desktop-Implementierung benötigt keine neue Plattform-API; CI prüft die Änderung separat.

Persistiert wird der Schalter `do-not-disturb`, keine Meldungsinhalte. Die bestehende begrenzte IPC-Warteschlange mit laufender TTL bleibt zuständig; hohe Priorität umgeht den Modus nicht. Animationskern, Statusprotokoll und Codex-Hook-Konfiguration bleiben unverändert.

## Sparsamere GitHub Actions · 13. Oktober 2026

- PASS: `actionlint .github/workflows/ci.yml` und `git diff --check`. Ausschliesslich Workflow und Dokumentation geändert; Rust-Code und Lockfiles unverändert.
- PASS: Zwei laufende Doppelprüfungen nach Inhaltsvergleich gestoppt. Der `main`-Baum `82ee8a3c3cd37912a293a62fc0fdbfe8cc749f55` entspricht dem bereits vollständig grün geprüften Hotkey-Stand; der synthetische Merge-Baum von PR #12 `fbeb6954c8fecc3829158553fff8357505f809ce` entspricht dessen ebenfalls grün geprüftem Branch. Gestoppt: Läufe `38368337270` und `38368386424`.
- Workflow: Feature-Branches laufen nur über PRs auf `main`, Pushes nur auf `main`. Reine Änderungen an Root-README, `docs/` und Issue-/PR-Vorlagen sind ausgenommen. Drei gebündelte Plattformjobs prüfen weiterhin beide Workspaces, Integrationen und Linux-Smoke-Tests; ältere Läufe werden abgebrochen, die Matrix stoppt nach Fehlern, Timeout je Job 25 Minuten.
- Workflow: PRs und normale manuelle Läufe bauen Debug. Release-Builds, Signaturprüfung auf macOS und Archive entstehen auf `main` oder mit dem manuellen Boolean-Input `build_packages`; Artefakte bleiben sieben Tage verfügbar.
- Workflow: Rust Cache v2.9.2 ist auf Commit `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` fixiert. Beide Workspaces sind erfasst. Speichern nur auf `main`, Restore auch in PRs; keine grossen isolierten PR-Caches. Die Repository-Policy erlaubt die Action; zuvor waren keine Caches vorhanden.
- NOT_RUN lokal: erneute Rust-Builds für unveränderten Programmcode. Die neue PR-CI prüft den Workflow einmal auf allen drei Plattformen; deren Ergebnis steht direkt am PR.
- NOT_RUN: gemessene Laufzeiten nach Cache-Aufbau und erneuter manueller Paket-Lauf. Der erste `main`-Lauf ist noch kalt. Einsparungen hängen von Cache-Treffern und Änderungen ab; die Zahl der automatischen Läufe je Feature-Änderung mit offenem PR sinkt von zwei auf einen.

## Meldungsverlauf mit Denken · 13. Oktober 2026

- PASS: `./scripts/check.sh --desktop`: 75 Rusttests (29 Root, 46 Desktop), 11 Integrationstests, Formatierung, Clippy ohne Warnungen, Asset-/Icon-Prüfung und Debug-Build. Nach Korrektur des Pause-Randfalls erneut vollständig geprüft.
- PASS: Empfangspuffer und Desktop-Verlauf halten höchstens 100 Einträge. Echte IPC prüft Normalisierung, leere zweite Abholung, unabhängige Anzeige-Warteschlange, abgelaufene TTL und Ausschluss ungültiger bzw. wegen Rate-Limits abgelehnter Meldungen. Stoppen des Listeners wartet vor der letzten Abholung auf das Thread-Ende.
- PASS: bei verborgenem Buddy und «Nicht stören» eingegangene Meldungen werden im Logikpfad erfasst; auch abgelaufene Einträge bleiben lesbar. Neueste zuerst; neue Meldungen beim Lesen unterbrechen Denken nicht und erscheinen genau einmal im Verlauf.
- PASS: Öffnen über Accessibility-Menüaktion, Layout und Eingabemaske in allen drei Grössen, langer Text mit 280 Zeichen und Quelle mit 48 Zeichen bis zur letzten Zeile scrollbar, benannter Schliessen-Knopf und sichtbarer Avatar über der Liste.
- PASS: `denken` wird über den bestehenden Alias auf `nachdenken` normalisiert und während des Lesens geloopt. Vorherige Animation samt Fortschritt wird danach wiederhergestellt; wiederholtes Öffnen startet die Geste nicht neu. Pause, reduzierte Bewegung, aktive Nachrichtenfristen und manuelle Aktionen bleiben unabhängig. Escape klappt alles zu. Einschalten von «Nicht stören» beendet eine aktive Remote-Blase, während Denken sichtbar bleibt.
- PASS: Leeren umfasst noch nicht abgeholte Empfangseinträge und lässt die Anzeige-Warteschlange bestehen. Ausschalten des Empfangs behält den Verlauf. Der App-Save-Pfad speichert weder Inhalte noch Leseansicht.
- PASS: finales macOS-Release-Bundle ad hoc signiert und streng geprüft. Echtes Tray blendet Beni aus und öffnet den Verlauf wieder; Meldungen beider Prioritäten, einschliesslich abgelaufener TTL, sind sichtbar. Menü öffnet den Verlauf; native Accessibility bestätigt «Denken» über AXHelp. Eingehende Meldungen halten die Animationspause auch nach Schliessen des Verlaufs; der zuvor vorhandene automatische Resume-Randfall ist behoben.
- PASS: natives Leeren, geordneter Neustart und verschwundene Testeinträge der vorherigen Sitzung. Eine danach neu eingegangene echte Codex-Meldung bleibt sichtbar: Ein absolut leerer Verlauf ist bei laufenden Integrationen kein geeignetes Neustartkriterium. Abschliessende frische IPC-Meldung beim Denken erfasst; Ansicht zum Anschauen geöffnet. Grössenwahl «Klein» und bisherige Darstellungs-/Empfangseinstellungen erhalten; «Nicht stören» aus, Hotkeys aus.
- NOT_RUN: native Windows-/Linux-Desktop-Abnahme des Verlaufs. Die gemeinsame Implementierung und der neue Empfangspfad werden in einem einzigen PR-CI-Lauf mit drei Jobs geprüft; die auf main aufgebauten Rust-Caches stehen dafür bereit.

Der Animationskern, das Nachrichtenformat und die Codex-Hook-Konfiguration bleiben unverändert. Eine begrenzte zusätzliche Empfangsliste im IPC-Adapter liefert nur angenommene, normalisierte Meldungen mit monotonem Empfangszeitpunkt; sie wird regelmässig in den Sitzungsverlauf übernommen. Meldungen werden weder persistiert noch automatisch wiederholt. Statuszustellungen erhalten jetzt einen bestehenden Pausezustand, statt ihn durch `Player::play` aufzuheben.

## Sparsamere statische Darstellung · 13. Oktober 2026

- PASS: Kernabschnitt von `./scripts/check.sh --desktop`: 29 Rusttests, 11 Integrationstests, Root-Formatierung und Clippy, Asset-/Icon-Provenienz. Desktop separat nach Korrektur der Test-API geprüft: 50 Rusttests, Desktop-Formatierung und Clippy ohne Warnungen, Release-Build.
- PASS: echte egui-Ausgabe nach mehreren stabilisierten Frames bestätigt Sekundentakt für Pause und reduzierte Bewegung sowie unveränderte 15-/30-fps-Budgets. Ablaufzeitpunkte von Nachrichten und Fehlern, keine schnelle Schleife für bereits abgelaufene Texte oder unsichtbare Fenster, Gestenende trotz reduziertem Takt geprüft.
- PASS: echte IPC-Meldung nach stabilisiertem pausiertem UI fordert unmittelbar Repaint mit Verzögerung null an; der Logikpfad zeigt den neuen Text und behält die Pause.
- PASS: `scripts/measure-process-macos.c` mit `clang -O2 -Wall -Wextra -Werror` gebaut. Ungültige und zu grosse Argumente, fehlender Prozess und CPU-Zeit eines realen, begrenzt beschäftigten Testprozesses geprüft (99,28 % eines Kerns über 1,01 Sekunden).
- PASS: `./scripts/package-macos.sh` baut die native App und CLI, signiert beide sowie das Bundle ad hoc und prüft die Signatur streng. Sichtbarer nativer Start während der Vergleichsmessungen über ausschliesslich eigene Fenstergeometrie bestätigt.
- PASS: Vorher-/Nachher-Messung derselben sichtbaren Release-App auf einem Apple M3 Max, macOS 26.6.2, zwei Messfenster à 30 Sekunden pro Modus. «Klein», Tempo 0,98, Energie 0,70, «Nicht stören» an, Empfang an, eingeklappte Bedienung. Methode, Einzelwerte und Grenzen in [PERFORMANCE.md](PERFORMANCE.md).
- PASS: abschliessend Originaleinstellungen wiederhergestellt, finale App sichtbar gestartet und frische IPC-Meldung angenommen. Empfang bleibt an, «Nicht stören» aus, «Klein» und «Weniger Bewegung» erhalten.
- NOT_RUN: neue native Bedienabnahme von Pause, Tray-Ausblenden und minimiertem Fenster. Der Zugriff auf Benis native Accessibility-Fenster war in dieser Sitzung nicht zuverlässig möglich; die entsprechende automatische UI-/Logikprüfung besteht. Die bisherigen nativen Abnahmen sind im vorherigen Abschnitt dokumentiert.
- NOT_RUN: lange Idle-, GPU- und Akkumessung sowie native Windows-/Linux-Leistungsmessung. CI-Ergebnisse für die drei Buildplattformen stehen am PR.

Der Sekundentakt für Positionsprüfung und Logik bleibt erhalten. Normale Animationen werden nicht verlangsamt. Das C-Messwerkzeug wird weder in die App eingebaut noch automatisch durch CI ausgeführt.

## Systemeinstellung für reduzierte Bewegung · 13. Oktober 2026

- PASS: `./scripts/check.sh --desktop`: 82 Rusttests auf macOS (29 Root, 53 Desktop), 11 Python-Integrationstests, beide Formatierungen und Clippy-Prüfungen, Assets/Icons und Desktop-Debug-Build.
- PASS: Ableitung der effektiven Pose und des Zeichenbudgets aus manueller und Systemvorgabe. Systemwechsel und Übergehen verändern weder Animation, Fortschritt, Pause noch Nachrichtenfristen. Manuelle Reduktion bleibt bei nicht verfügbarer Quelle erhalten.
- PASS: neuer benannter Accessibility-Schalter in allen drei Fenstergrössen; Bedienung per UI-Klick, unverändert pausierter Meldungsverlauf, Migration bestehender Einstellungen, beschädigter Speicherwert, getrennte Speicherung ohne Persistenz des Systemwerts.
- PASS: Linux-Portalcode samt privatem D-Bus-Test in einem temporären macOS-Harness kompiliert. Der Testdienst liefert 0/1, unbekannte Zahlen, falschen Typ, fehlenden Schlüssel und verschwindet; Testausführung auf Linux erfolgt in der PR-CI. Kein Zugriff auf einen echten Desktop-Bus in diesen Tests.
- PASS: echter Rust-Adapter auf dem macOS-Hauptthread liest `Some(false)`; direkter AppKit-Aufruf bestätigt dieselbe OS-Vorgabe. Das finale Release-Bundle und die CLI wurden ad hoc signiert und streng geprüft.
- PASS nativ: sichtbarer Start, IPC-Meldung angenommen, «Systemeinstellung beachten» im Buddy-Menü standardmässig aktiv und per AXPress aus/an bedienbar; Statusanzeige wechselt korrekt. «Weniger Bewegung» bleibt dabei erhalten und lässt sich separat bedienen. Abschliessend beide Werte wiederhergestellt und Bedienung zugeklappt.
- PASS nativ: geordneter Neustart speichert `follow-system-motion=true`. Vorherige Werte für Animationseinstellungen, Grösse, Durchklicken, Empfang, Hotkeys und «Nicht stören» unverändert. Nach erneutem Start zeigt das Menü die richtige Systemvorgabe und gespeicherte Auswahl.
- NOT_RUN: globalen Bewegungs-Schalter des OS während der laufenden App umschalten; native Windows-/Linux-Desktop-Abnahme. Details, Abfrageintervalle und Portal-Rückfall in [SYSTEM_MOTION.md](SYSTEM_MOTION.md).

Nur der Desktop liest Systemvorgaben. Der Kern, Statusprotokoll, Hooks und Assets bleiben unverändert. `zbus` 5.19.0 war bereits im Desktop-Lockfile vorhanden und wird jetzt auf Linux zusätzlich direkt verwendet; unter macOS/Windows sind nur vorhandene Bindings erweitert.
