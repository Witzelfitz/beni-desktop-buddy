# Abnahmematrix

## Automatische Kernprüfung

| Prüfbereich | Erwartung |
| --- | --- |
| Assets | 44 Posen und 20 Animationen, gültige PNG-Geometrie und Segmentreferenzen |
| Bewegung | Übereinstimmung mit 480 originalen SDK-Referenzposen |
| Zeit | Segmentgrenzen, negative Loopzeit und Loopende korrekt |
| Player | One-shot endet in Idle; neue Aktion unterbricht die alte |
| Pause/Tempo | Keine Zeitfortschreibung während Pause; Tempo korrekt |
| Eingaben | Unbekannte Namen, NaN, unendliche/negative Zeit und ungültiges Tempo ablehnen |
| Atlas | Defekte Daten, Segmente ohne Dauer und Koordinaten ausserhalb des PNG ablehnen |
| Geometrie | Rotation um Bodenanker und Spiegelung korrekt |
| Einstellungen | Ungültige gespeicherte Zahlen sicher bereinigen |
| CLI | Gültige JSON-Ausgabe und klare Fehler-/Exitcodes |

## Manuelle Desktopprüfung

Nativer Start, Transparenz und die sichtbare Bedienleiste wurden auf macOS geprüft. Automatisierte egui-UI-Tests stehen im Prüfbericht; die übrigen manuellen Abnahmen sind offen.

| Prüfung | macOS | Windows | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Nativer Start | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Apple-Silicon-Release-DMG: Notarisierung / Tickets / Gatekeeper | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Release-DMG: Installation unter Programme / sichtbarer Start | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Native Entwicklungs-DMGs: Intel und Apple Silicon, Installation / CLI / Fenster auf macOS 15.7.9 | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Gemeinsamer Release-Kandidat: beide DMGs/Apps signiert, notarisiert, Tickets / Gatekeeper / temporäres Kopieren / Ed25519 | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Finale Release-Kandidaten: native CLI / GUI auf macOS 15.7.9 | PASS (Intel + Apple Silicon) | NOT_RUN | NOT_RUN | NOT_RUN |
| Öffentliches Release: alle acht anonymen Downloads zurückgeladen / bytegleich / authentifiziert | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Transparenz / kein Fensterhintergrund | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Alle Animationen / PNG-Alpha | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Dragging / Klick-Begrüssung | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Pause / Tempo / reduzierte Bewegung | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Systemeinstellung zur Bewegung lesen / Übergehen / Speicherung | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Globalen OS-Schalter im laufenden Buddy ändern | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Sichtbare Idle-Bewegung aus vorhandenen SDK-Animationen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Lokale Erinnerungen: Texteingabe / Planen / Entfernen / Fälligkeit / Neustart | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Fokus-/Pausen-Timer: Start / Unterbrechen / Fortsetzen / Abbrechen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Timer-Abschluss bei verborgenem Buddy / Nicht stören / Verlauf | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Einstellungsfenster: Tray / Buddy, Grösse, Schliessen / erneut öffnen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Einstellungen: native AX-Aktionen und IPC während geöffnetem Fenster | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Menü / eigene Sprechblase | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Bedienleiste: eingeklappter Start / Klick auf Beni / Escape | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Grössenwahl Klein / Mittel / Gross, Neustart und Durchklicken | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Hotkeys: globale Pause, Konflikte, Umbelegen, Freigabe und Speicherung | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Nicht stören: Tray / Menü, echte IPC, TTL und Neustart | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Meldungsverlauf: Denken, Tray / Menü, Leeren, Pause und Neustart | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Transparenter Hintergrund / Avatar / Bedienleiste / Blase anklickbar | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Native Accessibility-API: beschrifteter Beni / Aktionen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Vollständige Screenreader-Bedienung (VoiceOver / NVDA / Orca) | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Tray: Ausblenden / Anzeigen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Tray: Pause / Fortsetzen / Einstellungen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Tray: Zurückholen auf Hauptbildschirm | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Tray: Beenden bei ausgeblendetem Fenster | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Neustart mit gespeicherter Offscreen-Position | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Lange Sprechblase: Scrollen / Schliessen / sichtbare Bedienleiste | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Empfang aus / an / wieder aus | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| CLI → Statusblase / Quelle / Geste | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Status nach Ausblenden / Anzeigen | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Empfangsfreigabe nach Neustart | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Vordergrunddarstellung | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Einstellungen nach Neustart | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Retina / DPI / Monitorwechsel | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Kurze Idle-CPU-/Wakeup-Vergleichsmessung: reduziert / animiert | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Lange Idle-/GPU-/Akkumessung, minimiertes Fenster | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |

Transparente Bereiche der Fensterfläche sind bei verfügbarem Tray standardmässig durchklickbar. Native macOS-Prüfung: Ein eigenes Testfenster direkt hinter Beni erhält Klicks durch transparenten Hintergrund und die eingeklappte Leiste; Klicks auf Beni öffnen die Bedienung, sichtbare Blasen lassen sich schliessen, und Dragging funktioniert weiterhin. Die Einstellung lässt sich im Menü abschalten; vollständige Windows-/Linux-Desktop-Abnahme bleibt offen.

Punkt 1 wurde am 12. Oktober 2026 mit dem echten macOS-Menü, gezielten Fensteraufnahmen und CGWindow-Geometrie geprüft. Für den Neustart wurden ausschliesslich Benis gespeicherte Fensterkoordinaten vorübergehend auf `{99999, 99999}` gesetzt; die gesicherten Einstellungen wurden anschliessend wiederhergestellt. Das 340 × 440-Punkte-Fenster landete sichtbar bei `{1372, 661}` auf dem 1728 × 1117-Punkte-Hauptbildschirm. Ein physisches Abhängen eines Monitors und gemischte DPI sind weiterhin NOT_RUN; die Positionierungsregeln dafür sind automatisch getestet.

Das Statuspaket wurde auf macOS mit der gebündelten und der im Cargo-Binärpfad installierten CLI geprüft. Das neue IPC-Verzeichnis hat Rechte 0700, der Socket 0600. Vor der Freigabe und nach dem Abschalten liefert notify Exitcode 3. Nach Freigabe erscheint die Meldung mit Quelle und passender Geste; der finale Text einer langen Testmeldung ist durch Scrollen erreichbar. Beim Ausblenden wird eine Meldung angenommen und erst nach erneutem Anzeigen sichtbar. Die Freigabe bleibt nach Neustart erhalten. Die reale Registrierung von Claude-Code-Hooks, Kurzbefehlen, systemd-Units und Aufgabenplanung ist NOT_RUN; die Vorlagen werden nicht automatisch eingerichtet.

Die einklappbare Bedienleiste wurde am 12. Oktober 2026 im neu gebauten macOS-Bundle geprüft: eingeklappter Start, Auf-/Zuklappen durch Klick auf Beni, Einstellungen über «Menü» und Schliessen samt Bedienleiste mit Escape bei fokussiertem Fenster. Gezielte Aufnahmen ausschliesslich des Buddy-Fensters bestätigen transparente Pixel an der Stelle der eingeklappten Leiste. Die IPC-Freigabe und bisherigen Einstellungen bleiben nach dem Neustart erhalten. Native Windows-/Linux-Prüfungen dieses Verhaltens bleiben NOT_RUN.

Die Accessibility-Anbindung wurde im neuen macOS-Release-Bundle über die native AX-API ausschliesslich für Benis Fenster geprüft. Benannte Buttons, Nachrichtenfeld, Checkboxen und Statuslabel sind erreichbar. AXPress klappt die Leiste auf/zu, bedient Pause/Weiter und Einstellungen und schliesst eine echte IPC-Statusmeldung. Die globale VoiceOver-Einstellung wurde dabei nicht verändert. Diese API-Prüfung ersetzt keine vollständige Abnahme der gesprochenen Ausgabe und Navigation mit VoiceOver, NVDA oder Orca.

Das separate Einstellungsfenster wurde am 13. Oktober 2026 im ad hoc signierten macOS-Bundle über AX-Aktionen und CGWindow-Geometrie geprüft. Das 520 × 680-Punkte-Fenster (712 Punkte inklusive Titelleiste) lässt sich über Buddy und Menüleistensymbol öffnen, mit Footer oder Titelleiste schliessen und danach erneut öffnen. Verkleinern auf 380 × 360 Punkte sowie die Grössenwahl bei Beni beeinflussen jeweils nur das richtige Fenster. «Immer im Vordergrund» verändert nur Benis Fensterebene. Durchklicken bleibt für den Buddy aktiv; die separaten Checkboxen sind bedienbar. Eine echte IPC-Meldung erscheint bei geöffneten Einstellungen. Zuklappen der Buddy-Leiste lässt das Fenster bestehen. Die ursprünglichen Einstellungen und Benis Position wurden nach der Prüfung wiederhergestellt. Native Windows-/Linux- und vollständige Screenreader-Abnahme bleiben NOT_RUN.

Bei ausgeblendetem Buddy bleibt das separate Fenster im nativen Mac-Test bedienbar: «Beni spiegeln» lässt sich umschalten und zurücksetzen. Tray «Einstellungen» zeigt Beni wieder und fokussiert das bereits offene Fenster.

Der lokale Timer wurde am 13. Oktober 2026 im signierten macOS-Bundle über ausschliesslich eigene AX-Aktionen geprüft: Fokusdauer auf eine Minute setzen, Pausentimer starten/abbrechen, Fokus starten, Restzeit unterbrechen und wieder fortsetzen. Mit geschlossenen Einstellungen, ausgeblendetem Buddy und aktivem «Nicht stören» endet die reale einminütige Fokuszeit. Der Abschluss erscheint mit Quelle `timer` im Verlauf. Nach Schliessen des Verlaufs und Deaktivieren von «Nicht stören» zeigt Beni die wartende Blase und die vorhandene Geste «Daumen hoch». «Timer zurücksetzen» und Wiederherstellen der Vorgaben 25/5 Minuten wurden geprüft. Ein laufender Pausentimer wird durch App-Beenden beendet und nach Neustart nicht fortgesetzt; nur die Minuten bleiben gespeichert. Vorherige Buddy-Einstellungen und Fensterposition bleiben erhalten. Native Windows-/Linux- und Standby-Abnahme sind NOT_RUN.

Die lokalen Erinnerungen wurden am 13. Oktober 2026 im signierten macOS-Bundle mit ausschliesslich eigener Bedienung geprüft: echte Tastatureingabe im benannten Feld, Planen und Entfernen, Minutenwert setzen. Eine einminütige Erinnerung wurde bei ausgeblendetem Buddy, geschlossenen Einstellungen und «Nicht stören» fällig. Der Text erscheint im Verlauf mit Quelle `erinnerung`; nach Freigabe zeigt Beni die Blase und die ursprüngliche Geste «Idee». Eine weitere geplante Erinnerung und ein ungesendeter Entwurf wurden vor dem Beenden angelegt. Benis Persistenz enthält keinen der Testtexte, nach Neustart sind Feld und Liste leer. Vorherige Einstellungen einschliesslich Timer-Minuten und Fensterposition bleiben erhalten. Native Windows-/Linux-, Standby- und vollständige Screenreader-Abnahme bleiben NOT_RUN.

Am 14. Oktober 2026 wurde der gemeinsame Release-Kandidat 0.1.0/Build 1 lokal aus dem nativen Installer-Lauf `38472531951` erzeugt. Beide Apps und DMGs sind Developer-ID-signiert, von Apple akzeptiert und mit Tickets/Gatekeeper geprüft. Die finalen DMGs wurden schreibgeschützt geöffnet und ihre Apps in temporäre Ordner kopiert: Inhalt, Version, Architektur, ausführbare Dateien und Signaturen bestanden; ARM-CLI nativ mit `validate-assets` gestartet, anschliessend Kopien entfernt und Volumes ausgeworfen. Alle zehn lokalen Dateien sind unverändert authentifiziert. Die acht hochgeladenen Release-Dateien wurden aus dem unveröffentlichten GitHub-Entwurf zurückgeladen und gegen lokale Bytes und Signaturen geprüft. Kein finales GUI-Fenster gestartet und keine Intel-CLI auf ARM ausgeführt; native Entwicklungs-GUI-Abnahmen auf macOS 15.7.9 ersetzen diese offenen finalen Prüfungen nicht. Bestehende Programme-Installation und Hooks unverändert.

Nach PR #31/#32 bestehen am 14. Oktober 2026 auch beide finalen Developer-ID-Pakete die [native Abnahme](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38524476222) auf macOS 15.7.9: enthaltene CLI und eigenes sichtbares Fenster, Installation, Signaturen/Tickets/Gatekeeper und vollständige Entfernung. Die heruntergeladenen Abnahmebelege passen zu den finalen Paket-Hashes. Release 0.1.0/Build 1 ist veröffentlicht; alle acht anonymen HTTPS-Downloads stimmen bytegenau mit dem authentifizierten lokalen Kandidaten überein. Die zuvor offene finale GUI-/Intel-CLI-Abnahme ist damit erledigt. Exakte 15.0-/11.0-Geräte und frische Nutzerbedienung bleiben offen.


## Installer-Ergänzung 0.1.1, 14. Oktober 2026

| Prüfung | macOS Intel/ARM | Windows x86_64 | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Installation / CLI / eigenes sichtbares Fenster bzw. konfigurierte Oberfläche / Deinstallation | PASS, 15.7.9 | PASS, Server 2025 mit Mesa-Testtreiber | PASS, Ubuntu 22.04 | PASS, Ubuntu 22.04 |
| Laufende App vor Überschreiben/Deinstallation schützen | NOT_RUN | PASS | NOT_RUN | NOT_RUN |
| AppImage-Menüeinbindung/-entfernung / zusätzliche Dateien erhalten | NOT_RUN | NOT_RUN | PASS | PASS (gemeinsamer Installer) |
| Finale Publisher-Signatur / native finale Paketabnahme | PASS, Developer ID + Apple-Notarisierung | NOT_RUN | Ed25519 PASS; native Paketbytes unverändert | Ed25519 PASS; native Paketbytes unverändert |
| Öffentliche Downloads / authentifizierter Katalog / echte Live-Browserauswahl | PASS, 0.1.1 | NOT_RUN, wartet auf Microsoft-Signierung | PASS, 0.1.1 | PASS, gleiches AppImage |
| Physische Windows-11-/Linux-Desktop-/GPU-Abnahme / echte Updates | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |

Belege: [Linux](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38536064173), [Windows](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38537805266), [finale Mac-DMGs](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38538384964). Die älteren Zeilen oben halten die früheren manuellen Produktabnahmen fest. Diese Ergänzung bestätigt Installer und native Smoke-Tests, keine vollständige manuelle Bedienung auf Windows/Linux. Elf öffentliche Dateien wurden anonym zurückgeladen und bytegleich/authentifiziert geprüft. Das Linux-AppImage wurde zusätzlich über den echten Downloadbutton der Live-Seite geladen und per Hash verglichen.

## Updatekette R3a, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| Authentifizierter Kandidat → Architektur-/Kanal-Feeds; Release-Felder und Paketsignaturen identisch | PASS, Offline-Fixtures |
| Ableitung für beide Mac-Ziele des echten authentifizierten 0.1.1-Kandidaten | PASS, nur gelesen, keine Produktions-Appcasts signiert oder geschrieben |
| Feed-/Manifest-/Paket-/Herkunftsmanipulation, fremder Schlüssel, falsche Länge, zusätzliche/verlinkte Dateien | PASS, negative Offline-Prüfungen |
| Vorhandene Ausgabe und Kandidat vor Änderungen schützen; ungültige Eingaben vor Schlüsselzugriff ablehnen | PASS, Offline-Fixtures |
| Sparkle 2.10.0 akzeptiert Python-Paket-/Feed-Signaturen, erzeugt bytegleichen Feed und lehnt manipulierte Bytes ab | PASS, native Probe auf dem Entwickler-Mac; nur kurzlebiger Testschlüssel |
| Produktions-Appcasts erzeugen / veröffentlichen | NOT_RUN |
| Framework im signierten Bundle / Tray-Aktion / echtes Update A → B | NOT_RUN |
| Downloadabbruch / nicht schreibbarer Installationsort / Einstellungen nach Update | NOT_RUN |

Native Probe: `scripts/check-sparkle.py --sparkle-archive …`, SDK-Grösse und SHA-256 aus `release/sparkle-sdk.json` vor dem Entpacken geprüft. Keine App gestartet oder ersetzt, keine Hooks verändert. Die Signaturkompatibilität bestätigt noch keine echte Installation.

## Updatekette R3b, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| SDK-Grösse/Hash, Symlinks, Lizenz und Helper-Signierreihenfolge; feste Schlüssel-/Kanal-/Sicherheitskonfiguration | PASS, 31 Packaging-Regressionsprüfungen |
| Synchrones Speichern mit zwei echten FileStorage-Tests; IPC bei Schreibfehler erhalten | PASS, tatsächlicher Persistenzcode und isolierter IPC-Test |
| Verzögerter Neustart; «Später» bleibt wirksam, ausdrückliche Bestätigung; Zugriff ohne laufende native Instanz | PASS, positive/negative Rust-/UI-Prüfungen |
| Hauptthread-Grenze, Framework laden, gespeicherte automatische Prüfwahl über echte Neustarts, asynchroner Installationscallback genau einmal | PASS, temporäres Developer-ID-Bundle auf ARM macOS 26.6.2 |
| Ungültiger lokaler Feed tatsächlich angefordert, nativer Fehlerdialog, keine Installation | PASS, eigener Loopback-Fixture-Server und Testprozess; keine produktive HTTP-Ausnahme |
| Echte Rust-GUI: neues Tray-Item → unabhängige Einstellungen, Erklärung bei noch fehlendem Feed, native Prüfwahl vorhanden | PASS, separate App/Präferenzdomains und eigener Datenordner, ARM macOS 26.6.2 |
| Updateaktion bei ausgeblendetem Buddy: Zurückholen, unabhängiges Fenster öffnen und schliessen | PASS, echter signierter Rust-Build nach Korrektur |
| Neues Rust-/Sparkle-Bundle: Apple-Annahme, Ticket und Gatekeeper | PASS, lokale Notarisierung der separaten QA-App; kein öffentlicher Installer |
| Bestehende Programme-Installation und Codex-Hook nach GUI-Abnahme bytegleich; nur eigener Testprozess beendet | PASS |
| Neues Intel-/macOS-15-Bundle / produktive Feed-Freigabe / neue öffentliche Version | NOT_RUN |
| Echtes Update A → B, Einstellungen nach Installation, Downloadabbruch, nicht schreibbarer Installationsort | NOT_RUN |

Das normale Packaging enthält das gepinnte SDK und verwendet macOS 15.0 als Bundle-Mindestversion. Der öffentliche Feed bleibt durch `release/update-policy.json` gesperrt, bis Desktop-Pipeline und Website dieselben zusätzlichen Feeddateien unterstützen. Die Brückenprobe benötigt keinen Rust-Build und läuft bewusst lokal; keine zusätzliche automatische Installer-Matrix eingeführt. Die veröffentlichten Dateien von 0.1.1 bleiben unverändert.

## Update-Veröffentlichung R3c, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| Manifest v1 unverändert weiter lesbar; v2 verlangt genau die vorhandenen Mac-Architekturen, passenden Kanal/Version/Schlüssel und begrenzte Feed-Metadaten | PASS, 20 Schema-/Versionsprüfungen |
| Kandidat mit integrierten Appcasts; Export enthält nur öffentliche Dateien, schützt Quellen/belegte Ausgabe und prüft kopierte Bytes | PASS, positive/negative Offline-Prüfungen |
| Finale GitHub-Abnahme lädt und authentifiziert ausgewählten Feed ohne privaten Schlüssel oder neuen Build | PASS, 27 Release-Regressionsprüfungen mit API-Fixtures |
| Fehlender/manipulierter/verlinkter Feed, fremde Signatur, korrekt neu signierter falscher Build und konsistente falsche GitHub-Digests | PASS, negative Prüfungen in Release-/Appcast-Tests |
| Identische öffentliche Python-/Node-Testvektoren für Stable ARM/Intel und Beta ARM, Unicode und XML-Escaping | PASS, beide Repositories; keine privaten Testschlüssel gespeichert |
| Sparkle 2.10.0 akzeptiert Python-Signaturen und erzeugt identische Feed-Bytes | PASS, native Probe mit kurzlebigem Testschlüssel; insgesamt 21 Appcast-Prüfungen inklusive dieser Probe |
| Echter bestehender lokaler 0.1.1-Kandidat mit unveränderten Paketbytes authentifiziert | PASS, nur gelesen, kein Signierzugriff |
| Root/GUI fmt, Tests und Clippy | PASS auf Entwickler-Mac: 29 Root- und 88 Desktop-Tests |
| Website-Code mit realem öffentlichen 0.1.1-Katalog: Downloads verfügbar, noch kein Update-Feed | PASS, anonymer Abruf |
| Website-Deployment / erste öffentliche v2-Feeds / neue Bootstrap-Version | NOT_RUN, eigene PRs und Release-Abnahme folgen |
| Tatsächliche Installation A → B und Installationsfehlerfälle | NOT_RUN; konkreter Ablauf in UPDATE_RELEASE_RUNBOOK.md |

Keine neuen Installer für diese Vorbereitung gebaut; keine Produktionsschlüssel gelesen, keine installierte App oder Hooks geändert. Öffentliche Version bleibt 0.1.1, Feed-Freigabe im Bundle bleibt aus.

## Bootstrap-Veröffentlichung 0.1.2, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| Versions-PR #43: Root-/Desktop-Versionen und Lockfiles synchron, Build 3 neu, drei Plattformprüfungen | PASS |
| Neue native Mac-Entwicklungsinstaller Intel/ARM samt GUI | PASS, Run 38569173327 |
| Linux 0.1.2: CLI, X11/Wayland, Menüintegration/-entfernung | PASS, Run 38569177728, Ubuntu 22.04 |
| Lokale Developer-ID-Signierung, Apple-Notarisierung, App-/DMG-Tickets und Gatekeeper beider Mac-Pakete | PASS |
| Finale signierte Intel-/ARM-Pakete: temporäre Installation, CLI, eigenes sichtbares Fenster, Entfernung | PASS, Run 38571146659 auf macOS 15.7.9; Intel-Ticketcheck beim ersten Versuch FAIL/Exitcode 68, gezieltes Wiederholen mit unveränderten Dateien PASS |
| Sparkle liest tatsächlichen signierten 0.1.2-Feed, akzeptiert Signatur und genau ein passendes Item/Build/Version/Adresse/Grösse; natives Ergebnis «aktuelle Version» | PASS, isolierte Developer-ID-Probe auf Entwickler-Mac mit unverändertem Feed und lokalem Fixture-Server, keine bestehende App ersetzt |
| Release 0.1.2: genau 13 öffentliche Dateien, Upload-Digests, alle anonymen Bytes/SHA-256/Signaturen | PASS, Release-ID 412741277, Tag auf Quellcommit cf11fe0bc26fb087eca526a6d92144163fba4bfe |
| Live `/api/downloads` und beide Mac-Stable-Feeds: v2/0.1.2/Build 3, unveränderte XML-Bytes | PASS |
| Echter Browser: macOS-Vorauswahl, Version 0.1.2, Apple-Silicon-DMG tatsächlich geladen und Hash/Grösse identisch | PASS, Chromium mit realer Live-API; keine Browserfehler |
| Bisherige Programme-Installation und Codex-Hook vor Nutzerinstallation unverändert | PASS, SHA-256-Vergleich |
| Nutzerinstallation 0.1.2 und echtes Update A → B | NOT_RUN, Nutzer wird anschliessend durchgeführt |
| Update-Downloadabbruch, defekte Paketsignatur und nicht schreibbarer Installationsort | NOT_RUN |

Windows wartet weiter auf Microsofts Firmenprüfung. Kein neuer Windows-Installer gebaut; Installer-/CI-Builds gezielt einmal je benötigtem Ziel, ausschliesslich der fehlgeschlagene Intel-Abnahmejob ohne Rust-Build wiederholt.

## Nutzerinstallation 0.1.2, 14. Oktober 2026

PASS: Nutzer bestätigt die manuelle Installation und den Start. `/Applications/Beni Buddy.app` meldet 0.1.2/Build 3, freigegebenen Apple-Silicon-Stable-Feed und die unveränderte Produkt-Bundle-ID; der Prozess läuft aus dieser Programme-App.

NOT_RUN: vom Nutzer bestätigtes Ergebnis «aktuelle Version» über die Tray-Aktion, Erhalt seiner Einstellungen und tatsächliches Update A → B. Diese Schritte folgen in der begleiteten Abnahme.

## Nutzerprüfung des Mac-Updaters, 14. Oktober 2026

PASS: Nutzer öffnet «Nach Updates suchen …» in der installierten 0.1.2 und bestätigt «up to date». Native App, HTTPS-Feed und Oberfläche funktionieren in seiner tatsächlichen Installation zusammen.

NOT_RUN: Installation A → B; dafür wird 0.1.3/Build 4 vorbereitet. Neue kleine UI-Ergänzung: installierte Version im Updatebereich sichtbar machen.

## Veröffentlichung B und Fehlerproben, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| 0.1.3/Build 4, PR #47, Versionsanzeige im Updatebereich; Root-/GUI fmt, 29/88 Tests, Clippy und drei Plattformprüfungen | PASS |
| Native Mac-Entwicklungsinstaller beide Chips und Linux CLI/X11/Wayland | PASS, Runs 38579115673/38579119594 |
| Lokale Developer-ID-Signierung, Notarisierung/Tickets/Gatekeeper | PASS, beide Mac-Ziele |
| Finale native Mac-Installer Installation/CLI/GUI/Entfernung | PASS, Run 38580839616, beide Chips macOS 15.7.9 |
| Genau 13 öffentliche Dateien, Upload-Digests, alle anonymen Bytes/SHA/Signaturen | PASS, Release-ID 412815822; Tag-Quellcommit 5fe85e8efe17a56238eb544f2c2aef20fd6d4f76 |
| Live-Katalog und beide Stable-Feeds bytegleich zum Kandidaten, 0.1.3/Build 4 | PASS |
| Native SDK-Probe: Build 4 angeboten und ohne Download abgebrochen | PASS, eigene Domain, kurzlebiger Testschlüssel und kontrollierter Loopback-Server |
| Downloadabbruch nach 65536 real empfangenen Bytes | PASS, keine Installationsfreigabe, Test-Executable unverändert |
| Beschädigtes DMG mit korrekter angekündigter Länge | PASS, SDK verweigert vor dem Entpacken: Fehler 4005 mit zugrunde liegendem EdDSA-Validierungsfehler 3002; keine Installation |
| App auf schreibgeschütztem DMG | PASS, SDK-Fehler 1003 vor Download; kein Nachweis für andere Administrator-/ACL-Fälle |
| Einstellungen vor Nutzerupdate geschützt lokal gesichert | PASS, eigener ignorierter Backup-Ordner 0700, Dateien 0600 |
| Tatsächliche Nutzerinstallation 0.1.2 → 0.1.3 samt Neustart, Einstellungen/IPC und Timerabfrage | NOT_RUN, nächster begleiteter Schritt |

Die negativen SDK-Proben laufen ausschliesslich in eigenen temporären Testbundles, ohne private Produktionsschlüssel. Das Paket war eine kontrolliert beschädigte 0.1.2-Fixture, als Testziel Build 4 angekündigt; kein fehlerhafter Stable-Feed veröffentlicht. Produktionsapp behält HTTPS und unveränderte Vertrauensanker.

## Tatsächliches Nutzerupdate und Chat-Grundlage, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| Nutzer aktualisiert 0.1.2 → 0.1.3 über die angebotene Updatekette | PASS, Nutzer bestätigt «update ist auf 0.1.3 durch» |
| Programme-App Version/Build/Bundle-ID und laufender Prozess | PASS, 0.1.3/Build 4, unveränderte Bundle-ID |
| Gespeicherte Buddy-Einstellungen, Grösse, Empfang, Click-through, Nicht stören und Bewegungsregel nach Update | PASS, Werte gegenüber geschützter 0.1.2-Sicherung unverändert; keine Inhalte ausgegeben |
| Codex-Hook nach Update unverändert | PASS, SHA-256-Vergleich mit vorheriger Sicherung |
| Gebündelte 0.1.3-CLI erreicht bestehenden Statusdienst | PASS, Testmeldung mit Quelle update-check angenommen; sichtbare Geste nicht separat abgenommen |
| Tatsächlicher Ablauf «Später» → spätere ausdrückliche Neustartfreigabe mit pausiertem Timer | NOT_RUN, pausierter Timer bestätigt, Ablauf der Abfrage nicht separat bestätigt |
| Weitere Administrator-/ACL-Fälle und neue Zielgeräte | NOT_RUN |
| R7a: Provider-Vertrag, Session und Fake-Worker | PASS, 13 neue Offline-Tests einschliesslich Abbruch, Fristen, Grenzen, Kontext und redigierten Diagnosen |
| Root-/Desktop-fmt, Tests und Clippy | PASS lokal auf macOS, 42 Root- und 88 Desktoptests |
| Release-Schema/Versionskonsistenz und lokale Release-Regressionsprüfungen | PASS, 20 + 27 Offline-Prüfungen; neue Chat-Version in Root-Lockfile berücksichtigt |
| Ollama-Voraussetzung auf Entwickler-Mac | PASS, CLI vorhanden, lokaler Dienst erreichbar, vier Modelle; keine Modellanfrage gestellt |
| Neue Windows-/Linux-Prüfungen für R7a | NOT_RUN lokal, bestehende PR-CI folgt; keine zusätzlichen Installer-Jobs |
| Echte Modellanfragen, Einrichtungs-/Chatfenster, Zugangsdatenverwaltung und Chat-Animationen | NOT_RUN, folgende eigene Umsetzungsschritte |

Chat-Grundlage ohne neue externe Rust-Abhängigkeiten; kein HTTP-Aufruf oder Modell-Download. Keine Installer-Matrix oder neue Veröffentlichung für diesen Entwicklungsschritt.

## Klickzugang und lokaler Ollama-Chat, 14. Oktober 2026

| Prüfung | Ergebnis |
| --- | --- |
| Grundlage PR #49 | PASS, drei Plattformprüfungen; gemergt |
| Klick auf Beni → benannter Chatbutton, eigene native Oberfläche, gleicher Avatar bei allen Grössen | PASS, positive UI-Prüfung und echter macOS-Klickzugang |
| Modellliste: nur vorhandene lokale Completion-Modelle, keine Bild-/Embedding-/Cloud-Modelle; erneute ID-/Statusprüfung vor Chatdaten | PASS, eigener HTTP-Testserver; lokal zwei von vier Modellen angeboten |
| Streaming mit geteilter UTF-8-Folge, interne Überlegungen ausgeblendet, Länge/Verbrauch/Abschluss | PASS, Offline-Decoder und echter lokaler Rust-Aufruf |
| Abbruch bei wartendem HTTP-Body schliesst die Verbindung; begrenzter Kanal und vollständiges Leeren bei Worker-Ende | PASS, echter Socket und positive/negative Worker-Prüfungen |
| Ungültige/übergrosse Metadaten oder Antwort, fehlender Abschluss, Tools, Redirects, rohe Fehlermeldungen | PASS, negative Offline-Prüfungen |
| Bestehendes Modell gemma3:4b, echte native Chatantwort | PASS, eigene QA-Instanz und eigener Speicherpfad; kein Modell heruntergeladen |
| Chat-Mindestgrösse: sechs Eingabe-/Footer-Bedienelemente innerhalb des eigenen Fensters | PASS, tatsächliche AX-Geometrie auf macOS |
| Modellwahl nach Neustart erhalten; Gesprächstexte nicht im App-Speicher | PASS, native Speicherung/Neustart und unabhängige Persistenzprüfungen |
| Modell-Popup nach Auswahl geschlossen; laufende Anfrage über Oberfläche abbrechen | PASS, aktuelle native QA-Instanz |
| Denken/Reden aus Originalassets, Pause und Bewegungsreduktion erhalten, keine Übernahme in Statusverlauf | PASS, positive Zustands-/UI-Prüfungen; native Sichtprüfung beider Gesten separat offen |
| Neues öffentliches Paket, tatsächlicher Nutzer-Chat nach Update, native Windows-/Linux-Chatabnahme | NOT_RUN |

Der echte Ollama-Test ist in der regulären CI ignoriert und wird gezielt lokal ausgeführt. Der Animationskern und importierte Assets bleiben unverändert. Kein Zugriff auf Produktionsschlüssel, keine Änderung der installierten Programme-App oder Hook-Definitionen; native QA-Prozesse sauber beendet. Regelmässige Plattform-CI bleibt bei drei Jobs, keine Installer-Matrix für diesen PR.

Lokale Schlussprüfung: PASS, Root-/Desktop-fmt und Clippy, 42 Roottests und 102 Desktoptests; gezielter ignorierter Ollama-Test zusätzlich bestanden. Release-/Versionsprüfungen: 20 + 27 Tests bestanden. Desktop-Debug-Build bestanden. PR-CI für diese Änderung folgt; keine erneuten Installer-Builds.

## Vorbereitung 0.1.4/Build 5

PASS: Chat-PR #50 mit drei grünen Plattformprüfungen gemergt. Neue Produktversion/Buildnummer auf eigenem Versionsbranch; keine veröffentlichten Dateien ersetzt. Native neue Installer, Notarisierung, Release-Entwurf und Feed-Veröffentlichung folgen.
