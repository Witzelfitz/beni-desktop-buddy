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
