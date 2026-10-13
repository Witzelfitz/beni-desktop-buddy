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

## Idle mit vorhandenen Mini-Beni-Gesten · 13. Oktober 2026

- Ursache des Stillstands: gespeichertes `reduced_motion=true` hielt auch «Warten» auf einer festen Pose. Auf ausdrücklichen Wunsch nach sichtbarer Bewegung ist diese manuelle Auswahl lokal jetzt aus. Systembeachtung bleibt an; Grösse, Energie, Tempo, Spiegelung, Vordergrunddarstellung, Empfang, Durchklicken, Hotkeys und «Nicht stören» bleiben erhalten.
- PASS: `./scripts/check.sh --desktop`: 88 Rusttests auf macOS (29 Root, 59 Desktop), 11 Python-Integrationstests, beide Formatierungen und Clippy-Prüfungen, Assets/Icons und Desktop-Debug-Build.
- PASS: alle 480 originalen SDK-Referenzposen stimmen weiterhin überein. PNG, Atlas, Animationsdaten, Aliase und ihre Provenienz sind unverändert. Die drei verwendeten Zustände sind bereits vorhandene Animationen: `warten`, `nachdenken`, `drehen`.
- PASS: deterministischer Desktop-Scheduler mit explizitem Zeitschritt. Jeweils 18 Sekunden ruhiges Warten, dann abwechselnd die bestehenden One-shots mit ihrer originalen Dauer und Pose, danach «Warten». Tempo gilt für die Ruhezeit; grosse Schritte lösen höchstens eine Aktion aus, ungültige Zeitwerte werden abgelehnt.
- PASS: Pause, manuelle/System-Bewegungsreduktion, sichtbare Blasen, Fehler, offene Bedienung und verstecktes Fenster verhindern neue Idle-Gesten. Manuelle und Statusaktionen übernehmen über den vorhandenen Play-Pfad sofort; der Idle-Scheduler wird zurückgesetzt und erzeugt keine Blasen oder Verlaufseinträge.
- PASS: Nachlesen hält den Idle-Scheduler an und stellt Player, Fortschritt, Pause und die Reihenfolge der nächsten vorhandenen Geste wieder her. Idle-eigene Aktionen bleiben bei 15 fps; manuelle/Status-Gesten und das Lesen behalten das bisherige Budget.
- PASS: finales macOS-Release-Bundle/CLI ad hoc signiert und streng geprüft. Sichtbarer nativer Start und gespeicherte Auswahl geprüft; der Nutzer bestätigt «Ja, er bewegt sich». Frische IPC-Meldung angenommen.
- NOT_RUN: vollständigen nativen automatischen Idle-Zyklus über die Accessibility-API verfolgen; Benis AX-Fenster war in dieser Sitzung nicht zuverlässig erreichbar. Auswahl, Reihenfolge, originalgetreue Posen und Unterbrechung sind automatisch geprüft; sichtbare Bewegung ist durch den Nutzer bestätigt.
- NOT_RUN: native Windows-/Linux-Desktop-Abnahme. CI prüft die gemeinsame Implementierung im einzigen PR-Lauf auf drei Buildplattformen.

## Separates Einstellungsfenster · 13. Oktober 2026

Die Bedienelemente liegen in einem eigenen, dekorierten und skalierbaren nativen Fenster statt über dem Avatar. Tray «Einstellungen» und Buddy «Menü» verwenden denselben Viewport. Die App-Einstellungen bleiben in derselben Instanz und im bisherigen Speicher; Statusmeldungen laufen während des offenen Fensters weiter.

- PASS: 93 lokale Rusttests (29 Root, 64 Desktop), 11 Python-Integrationstests, Formatierungen, Clippy mit `-D warnings`, Asset-/Icon-Provenienz, Desktop-Debug und signiertes macOS-Release-Bundle.
- PASS: eigene native Viewport-Fläche und Eingabebereich; Root-Geometrie bleibt unverändert. Scrollbare Einstellungen und erreichbarer Footer bei 520 × 680 sowie 380 × 360 Punkten. Fensterbau ist unabhängig von Buddy-Grösse, Transparenz und Durchklicken.
- PASS: Checkboxen und Nachrichtenfeld im Child-Accessibility-Baum; Aktionen auf Grössenwahl und Vordergrund richten Fensterkommandos an Root. Schliessen beendet nur die Einstellungen. Escape beendet zuerst eine fokussierte Eingabe und schliesst beim nächsten Druck das Fenster. Neue IPC-Meldung wird bei offenen Einstellungen angezeigt; Zuklappen der Buddy-Leiste schliesst die Einstellungen nicht.
- PASS: echte macOS-AX-Aktionen für beide Fenster: Öffnen über Buddy und Tray, Grössenwahl Gross → Klein ohne Änderung der Einstellungen-Geometrie, Vordergrund aus/an nur für Beni, separates Verkleinern/Wiederherstellen, Footer- und Titelleistenschliessen und erneutes Öffnen. Echte CLI-Meldung während geöffnetem Fenster sichtbar. Einstellungen und Buddy-Position anschliessend wiederhergestellt.
- PASS: bei über Tray ausgeblendetem Buddy bleibt das separate Fenster bedienbar; Spiegelung aus/an und Rückkehr über Tray «Einstellungen» geprüft.
- PASS: zusätzlicher Glow-Viewport braucht einen eigenen nativen AccessKit-Adapter. Der unveränderte Bibliothekspfad lieferte nur die OS-Titelleiste; der dokumentierte [lokale Patch](../vendor/eframe/BENI_PATCH.md) initialisiert den Adapter für jedes neue Fenster vor dem Anzeigen. Alle übrigen Upstream-Quelltexte, Versionen und Abhängigkeiten bleiben erhalten.
- NOT_RUN: vollständige VoiceOver-/NVDA-/Orca-Bedienung, native Windows-/Linux-Fensterabnahme, visuelle Bewertung durch den Nutzer. Plattformbuilds und gemeinsame UI-Prüfungen werden im einzigen PR-CI-Lauf geprüft.

## Lokaler Fokus-/Pausen-Timer · 13. Oktober 2026

«Fokus & Pause» im separaten Einstellungsfenster bietet eine einzelne, ausdrücklich gestartete Fokus- oder Pausenphase. Dauer 1–180 Minuten, standardmässig 25/5; Unterbrechen, Fortsetzen und Abbrechen halten Timer und Avatar-Pause auseinander. Abschluss über bestehende SDK-Gesten und denselben Anzeige-/Verlaufspfad. Kern, IPC-Vertrag, Hooks, Assets, Abhängigkeiten und CI bleiben unverändert.

- PASS: `./scripts/check.sh --desktop`: 102 lokale Rusttests (29 Root, 73 Desktop), 11 Python-Integrationstests, beide Formatierungen und Clippy-Prüfungen, Assets/Icons und Desktop-Debug-Build. Alle 480 originalen Referenzposen unverändert.
- PASS: explizite monotone Zeit, exakte Grenzen und einmaliger Abschluss für beide Phasen, verbleibende Zeit bei Unterbrechung, Abbruch/Neustart ohne alte Anzeige, verspätete Verarbeitung und 120-Sekunden-Frist. Konfiguration wird pro Feld geprüft; beschädigte Werte fallen zurück, laufende Timer werden nicht gespeichert.
- PASS: Child-Viewport-AccessKit-Aktionen für Start/Unterbrechen/Fortsetzen/Abbrechen beider Phasen. Avatar-Pause bleibt erhalten. Abschluss ohne IPC, mit reduzierter Bewegung und im Logic-Pfad ohne UI. Bestehendes statisches Sekundenbudget bleibt erhalten.
- PASS: verpasste Timer-Abschlüsse bleiben im begrenzten Sitzungsverlauf. Anzeige wartet bei Nicht stören, verborgenem Fenster, Lesen, Fehlern und bestehender Blase; IPC hat Vorrang. Alte Timer-Blase verfällt, Verlauf bleibt. Keine wiederholte Abschlussmeldung.
- PASS: finales macOS-Release-Bundle ad hoc signiert und streng geprüft. Reale einminütige Fokuszeit samt verstecktem Buddy und Nicht stören; Verlauf, spätere Blase und Originalgeste «Daumen hoch» über native AX-Aktionen bestätigt. Pausentimer gestartet/abgebrochen, Restzeit unterbrochen/fortgesetzt, Vorgaben wiederhergestellt.
- PASS: native App mit laufendem Pausentimer beendet und neu gestartet: Timer aus, 25/5-Minuten-Vorgabe gespeichert. Ursprüngliche Einstellungen und Position beibehalten.
- NOT_RUN: native Windows-/Linux-Abnahme, Standby-Verhalten und vollständige Screenreader-Ausgabe. Plattformbuilds und gemeinsame Tests werden im einzigen PR-CI-Lauf geprüft.

## Lokale Erinnerungen · 13. Oktober 2026

Eigene Texte in einem separaten Abschnitt der Einstellungen planen und entfernen. Der begrenzte Desktop-Scheduler tickt mit expliziter monotoner Zeit im bestehenden Logic-Pfad. Erinnerungen laufen parallel zum Fokus-Timer, benötigen keine IPC-Freigabe und verwenden ausschliesslich die bestehende Geste «Idee».

- PASS: `./scripts/check.sh --desktop`: 111 lokale Rusttests (29 Root, 82 Desktop), 11 Python-Integrationstests, beide Formatierungen und Clippy mit `-D warnings`, Assets/Icons und Desktop-Debug-Build. Alle 480 SDK-Referenzposen unverändert.
- PASS: 1–240 Zeichen nach Entfernen von Steuerzeichen, Unicode-Länge, Wartezeit 1–1440 Minuten, höchstens zehn geplante oder wartende Einträge. Sortierung nach Deadline und Erstellreihenfolge, einmalige Fälligkeit, Entfernen geplanter/wartender Einträge, verspätete Verarbeitung, Ablauf nach 120 Sekunden und erneute Kapazitätsfreigabe.
- PASS: bestehende Blasen, Nicht stören, verstecktes Fenster, Verlauf und Fehler halten die Anzeige zurück. Reihenfolge IPC → Fokus/Pause → Erinnerungen und einmalige Anzeigen. Fokus-Timer und Avatar-Pause bleiben unabhängig. Fällige Einträge bleiben im begrenzten Verlauf, selbst nach verfallener Blase.
- PASS: Native-Viewport-UI mit benanntem Texteingabefeld, Fokus/Tastatur sowie Planen-/Entfernen-Aktionen. Zehn lange Einträge bei 380 × 360 Punkten lassen den Footer erreichbar. Die bestehenden eingebetteten Tests scrollen zu den tiefer liegenden Einstellungen; ihre Bewegungs-/Hit-Map-Prüfungen bleiben erhalten.
- PASS: finales macOS-Release-Bundle ad hoc signiert und streng geprüft. Echte einminütige Erinnerung bei verborgenem Buddy, geschlossenen Einstellungen und Nicht stören; Text im Verlauf sowie anschliessende Blase und «Idee»-Geste über native eigene Aktionen bestätigt.
- PASS: Beenden mit geplantem Termin und ungesendetem Entwurf: keine Testtexte in Benis vollständiger Persistenz, nach Neustart leeres Feld und leere Liste. Ursprüngliche Einstellungen, Timer-Minuten und Fensterposition beibehalten. Kein zusätzlicher Thread oder Zeichenloop.
- NOT_RUN: native Windows-/Linux-Abnahme, Standby-Verhalten und vollständige gesprochene Screenreader-Bedienung. Plattformbuilds und gemeinsame Tests werden im einzigen PR-CI-Lauf geprüft.

## R1 · Release-Vertrag und Versionierung · 13. Oktober 2026

- PASS: `BENI_RELEASE_PYTHON=/private/tmp/beni-release-venv/bin/python ./scripts/check.sh --desktop`: 111 Rusttests (29 Root, 82 Desktop), 11 Hook-/Integrationstests, beide Formatierungen und Clippy mit `-D warnings`, unveränderte Asset-/Icon-Provenienz sowie Desktop-Debug-Build. Python-Testabhängigkeiten isoliert installiert; keine neuen Rust-Abhängigkeiten.
- PASS: nach Ergänzung der Versionsregressionen 19 Release-Tests mit gültigen Stable-/Beta-Beispielen und einzelnen Plattformen. Schema selbst geprüft. Negative Fälle für Version/Kanal/Tag, Build-/Grössen-/Hashgrenzen, fehlende und zusätzliche Felder, doppelte Ziele, unpassende Plattformformate/Anforderungen, fremde URLs/Versionen, fehlende oder falsch zugeordnete Signaturangaben, doppelte JSON-Felder, kaputte und übergrosse Eingaben sowie zurückhaltende Fehlerausgabe.
- PASS: Workspace-/Desktop-/IPC-Versionen und beide Lockfiles stimmen überein; Core/CLI erben die Version. Manipulierte Manifeste, fehlende Vererbung und abweichende Lockfiles werden abgelehnt. Isolierte Beta-Plist mit Build 7 erhält `0.2.0`/`7`, behält Bundle-ID und LSUIElement.
- PASS: `./scripts/package-macos.sh` erzeugt das Release-Entwicklungsbundle; Plist valide, Executables/Bundle ad hoc signiert und streng geprüft. Fertiges Bundle enthält `CFBundleShortVersionString=0.1.0`, `CFBundleVersion=1` und die bisherige Bundle-ID. Kein Neustart der laufenden Nutzer-App erforderlich.
- NOT_RUN: veröffentlichte Installer, echte Paket-/Manifest-Signaturprüfung, Updateinstallation und native Windows-/Linux-Installer. R1 implementiert das Format und prüft Angaben offline; Beispieldateien sind keine verfügbaren Downloads. Die bestehenden drei Plattformjobs übernehmen die PR-Prüfung, Schema-Tests laufen nur unter Linux.

## R2 · macOS-DMG und Release-Buildweg · 13. Oktober 2026

- PASS: R1/PR #23 gemergt; main-CI einschliesslich bisherigem Release-Packaging auf macOS, Windows und Linux grün.
- PASS: lokale Root-/Desktop-Prüfungen: 111 Rusttests, 19 Release-Tests, 11 Hook-/Integrationstests, Formatierungen, beide Clippy-Prüfungen mit `-D warnings`, unveränderte Assets/Icons und Desktop-Debug-Build. Nach Ergänzung der Packaging-/Receipt-Grenzen 23 portable Tests separat bestanden.
- PASS: reales `signed-test`-DMG für Apple Silicon. Executables und App mit Developer ID, Hardened Runtime und sicheren Zeitstempeln signiert, DMG Developer-ID-signiert; Signaturen streng geprüft. Exakte Architektur und Mach-O-Buildziel geprüft. SHA-256 und Grösse aus dem fertigen Image berechnet, Beleg kennzeichnet `notarized=false`.
- PASS: reales DMG schreibgeschützt geöffnet, Programme-Link und Installationshilfe geprüft, App in eigenen temporären Testordner kopiert, Version/Build/Systemziel sowie Signaturen von App und beiden Executables geprüft; enthaltene CLI führt Asset-Validierung aus. Testvolume ausgeworfen und Testinstallation entfernt. Kein Start/Beenden der laufenden Nutzer-App und keine Änderung an ihren Einstellungen.
- PASS: kompatibler Wrapper erstellt neues ad-hoc-signiertes Entwicklungsbundle in einem eigenen Prüfpfad. Swift-Prüfer für eigene Fenster-Metadaten kompiliert. YAML beider Workflows und Shell-Syntax geprüft; Dokumentverweise und `git diff --check` bestanden.
- PASS simuliert: Freigabe ohne Profil/Developer ID verweigert, Mehrdeutigkeit bei Signieridentitäten, Architektur-/OS-Abweichungen, Runtime-/Zeitstempelpflicht, Apple-Status nur `Accepted`, Stapler/Gatekeeper als Pflichtpfad, Release-Ausgabe nur nach allen erfolgreichen Schritten. Testnamen eindeutig, Pakete unveränderlich; Teilübernahme wird zurückgerollt. CI-Zugangsdaten nur temporär, private Dateirechte auf Unix, Suchliste/Wegwerfen auch bei Fehlern; keine lokale Nutzer-Keychain verändert.
- PASS: [PR #25](https://github.com/Witzelfitz/beni-desktop-buddy/pull/25) nach drei grünen PR-Plattformjobs gemergt. Lokales Schlüsselbundprofil `beni-notary` von Apple validiert; keine Zugangsdaten in Git oder Logs.
- PASS: echtes Release-DMG für Apple Silicon 0.1.0/Build 1. App-ZIP und DMG von Apple akzeptiert, beide Tickets angehängt und validiert; Gatekeeper und strenge Signaturprüfung bestanden. Build-Beleg kennzeichnet `notarized=true`; Grösse und SHA-256 beziehen sich auf das fertige notarisierte Image.
- PASS: `check-macos-installer.py --architecture aarch64 --mode release` prüft das DMG erneut schreibgeschützt und die daraus temporär kopierte App einschliesslich Version, Signaturen, Tickets, Gatekeeper und CLI-Asset-Validierung. Testvolume und Testinstallation anschliessend entfernt.
- PASS: Nutzer führt die echte Installation aus: alten Beni beenden, App aus dem DMG nach Programme kopieren, dort starten. Sichtbarer Start vom Nutzer bestätigt. Prozesspfad unter `/Applications/Beni Buddy.app`, Version 0.1.0/Build 1, gültige Signatur, Ticket und Gatekeeper-Zulassung anschliessend geprüft. Installer-Volume nach dem Repo-Umzug ausgeworfen; die installierte App bleibt geöffnet.
- NOT_RUN: keine Actions-Secrets vorhanden. Neue Installer-CI samt nativem Intel-/Fenster-Starttest, vollständige Abnahme auf einem frischen Gerät und altes macOS 11.0 noch offen. Kein öffentlicher Release, keine Ed25519-Updateprüfung und kein Update-Feed. Einrichtung und vollständige Abnahme stehen in [MACOS_INSTALLER.md](MACOS_INSTALLER.md).

## Lokaler Repo-Umzug · 13. Oktober 2026

- PASS: gesamtes Repo von `~/Downloads/beni-scrollytelling/beni-desktop-buddy` nach `~/GitHub/beni-desktop-buddy` verschoben. Git-Verlauf, lokale Build-Caches und DMGs erhalten; der alte Repo-Ordner ist entfernt, Remote bleibt `Witzelfitz/beni-desktop-buddy`.
- PASS: vier Codex-Hook-Befehle verwenden den bestehenden Adapter unter `~/.codex/hooks/beni-status.py`; Adapter und CLI unter `~/.cargo/bin` sind unabhängig vom Repo. Geprüfte lokale Codex-/Claude-Einstellungen, Shell-Startdateien und LaunchAgent-Definitionen enthalten keinen alten Repo-Pfad; vorhandene Hook-Definitionen unverändert.
- PASS: installierte CLI sendet nach dem Umzug eine echte Meldung an die laufende App und erhält «Meldung angenommen». Der bestehende Codex-Adapter sendet mit synthetischem Stop-Payload und seiner regulären CLI-Auflösung über `~/.cargo/bin`; der echte Notify-Prozess bestätigt Annahme mit Exitcode 0. Dies prüft den direkten Hook-Pfad, keine neue automatische Ereignisregistrierung. 11 Hook-/Integrationstests, 23 Packaging-Tests und Versionsprüfung im neuen Ordner bestanden.

## Lokale Release-Pipeline und native Installer-Belege · 13. Oktober 2026

- PASS: Nutzerwahl «Vorerst lokal signieren»; [ADR 0003](adr/0003-local-release-signing.md). Developer-ID-Schlüssel und Apple-Profil bleiben auf dem Mac. Separater Ed25519-Schlüssel in 1Password erzeugt und Rücklesen gegen den öffentlichen Schlüssel geprüft; keine privaten Werte in Argumenten, Dateien oder Logs. Nur der öffentliche Teil ist eingecheckt.
- PASS: 18 neue lokale Release-Tests mit echten Ed25519-Signaturen. Fremde/fehlgeschlagene CI-Quellen, andere Produkteingaben, falsche Archivdigests, Traversierung, Links, doppelte Dateien, manipulierte native Belege, Paket-/Manifest-/Prüfbeleg-Manipulation, unpassende Keys, globale Buildnummern, Versionswiederverwendung, fehlende Ziele vor Signierung und Rückrollen der Übernahme geprüft.
- PASS: 27 Packaging-Tests einschliesslich Pflichtsignatur von Produktions-DMGs, bewusst unsignierter Entwicklungscontainer mit weiter signierter App, festen/redaktierten Signierdiagnosen, schnellem optionalem CI-Credential-Preflight und verweigerten falschen nativen Abnahmebelegen.
- PASS: komplette lokale Root-/Desktop-Prüfungen: 111 Rusttests, 19 Release-Schema-/Versionstests, 11 Hooktests, Formatierungen, Clippy, Assets/Icons und Desktop-Build. Neue schnelle Signaturtests nur im bestehenden Linux-CI-Job, keine zusätzliche PR-Matrix und keine Tag-triggernden Installer-Jobs.
- FAIL: [erster gezielter Intel-Run](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38467512409). Rust-Release-Build läuft; Packaging-Codesign endet mit Exitcode 1, ohne signiertes Ergebnis. Der damalige allgemeine Fehler benannte keine Signierstufe. Der überarbeitete Build trennt unsignierte Entwicklungs-DMGs und Developer-ID-Distribution und gibt feste Fehlerkategorien/Stufen aus.
- NOT_RUN: native Wiederholung auf Intel/Apple Silicon mit `.native.json`-Belegen und echter Kandidat über den neuen Helfer. Exakte macOS-15.0-/macOS-11.0-Geräteabnahme, öffentliche Veröffentlichung, Sparkle-Appcast und Rust-Updater bleiben separat offen. Die App unter Programme wurde für diese Arbeit nicht beendet oder ersetzt.

## Intel-Signierreihenfolge · 13. Oktober 2026

- FAIL: [zweiter Intel-Lauf](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38470639800) grenzt den Fehler auf das Signieren eines Executables ein: «Code object is not signed at all». Die Trennung des Entwicklungs-DMGs war keine Behebung dieser Ursache.
- PASS: Ursache lokal mit zwei eigenen, per Clang für x86_64 erzeugten und ausdrücklich unsignierten Mach-O-Proben reproduziert. Beim zuerst signierten Hauptprogramm prüft `codesign` das umgebende Bundle und scheitert am noch unsignierten beigelegten CLI-Executable. Die gleiche Probe funktioniert mit CLI → Hauptprogramm → Bundle, jeweils strenge Prüfung. Keine Proben gestartet; eigener temporärer Ordner anschliessend entfernt.
- PASS: Packaging verwendet jetzt verbindlich diese Reihenfolge auch bei vorgebauten nativen CI-Executables. 28 portable Packaging-Tests, darunter die Regression mit einem initial unsignierten Helfer; 18 Release-/Signaturtests und Diff-Prüfung bestanden. Keine Rust-/Asset-/Versionsänderung und kein Neustart der installierten App.
- NOT_RUN: erneute native Intel-Installer-/Fenster-Abnahme und anschliessender gemeinsamer Release-Kandidat; die lokale Mach-O-Probe ersetzt keinen nativen GUI-Test.

## R2 · Native Abnahme und lokale Release-Freigabe · 14. Oktober 2026

- PASS: PR #27 und PR #28 gemergt; ihre PR- und main-Prüfungen auf macOS/Windows/Linux grün. 111 Rusttests, 19 Schema-/Versionstests, 11 Hooktests, zuletzt 28 Packaging- und 18 kryptografische Release-Tests.
- PASS: eigene unsignierte Intel-Mach-O-Probe auch Developer-ID-signiert; Helfer, Hauptprogramm und Bundle mit Hardened Runtime und sicheren Zeitstempeln streng geprüft. Probe nicht ausgeführt und entfernt.
- PASS: [gezielter nativer Installer-Workflow](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38472531951), main `ecbdc09`, beide Jobs erfolgreich. Intel und Apple Silicon jeweils auf macOS 15.7.9: echtes Entwicklungs-DMG, temporäre Installation, enthaltene CLI und eigenes sichtbares Fenster. Testprozess, Volume und Testordner entfernt; `.native.json` bestätigt Commit, Paket-Hash, Version und Abnahme.
- PASS: beide unverfallenen nativen Artefakte lokal heruntergeladen. Archivdigests gegen GitHub geprüft, genau vier reguläre Dateien, finale Entwicklungs-Paketbytes/Build-Belege und native Installations-/Fensterbelege passen zu Commit/Ziel/Version.
- PASS: echter neuer Ed25519-Schlüssel signiert eine temporäre Kopie des bestehenden notarisierten Apple-Silicon-DMGs; Verifikation erfolgreich, gezielt veränderte Bytes verweigert. Original unangetastet, Probe entfernt.
- OFFEN: tatsächliche gemeinsame Kandidatenerzeugung erreicht vor der ersten Signierung beim 1Password-Schlüssellesen das Zeitlimit. Quelle/Produkteingaben und beide nativen Installer sind bereits bestätigt. Nutzer um Entsperren/Bestätigen des CLI-Zugriffs gebeten; keine unvollständigen Kandidaten übernommen und keine zusätzlichen Apple-Uploads durchgeführt.
- NOT_RUN: gemeinsamer echter Release-Kandidat, finale native GUI-Abnahme seiner Developer-ID-Pakete, exakte macOS-15.0-/11.0-Abnahme, öffentliche Veröffentlichung und Rust-Updater/Sparkle. Aktuelle App unter Programme bleibt geöffnet und Hooks unverändert.
