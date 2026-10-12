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
| Transparenz / kein Fensterhintergrund | PASS | NOT_RUN | NOT_RUN | NOT_RUN |
| Alle Animationen / PNG-Alpha | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Dragging / Klick-Begrüssung | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Pause / Tempo / reduzierte Bewegung | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| Menü / eigene Sprechblase | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
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
| Idle-Last und minimiertes Fenster | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |

Transparente Bereiche der Fensterfläche können im MVP weiterhin Mausereignisse blockieren. Das ist ein offener Punkt für Phase 2 und muss bei der ersten Abnahme dokumentiert werden.

Punkt 1 wurde am 12. Oktober 2026 mit dem echten macOS-Menü, gezielten Fensteraufnahmen und CGWindow-Geometrie geprüft. Für den Neustart wurden ausschliesslich Benis gespeicherte Fensterkoordinaten vorübergehend auf `{99999, 99999}` gesetzt; die gesicherten Einstellungen wurden anschliessend wiederhergestellt. Das 340 × 440-Punkte-Fenster landete sichtbar bei `{1372, 661}` auf dem 1728 × 1117-Punkte-Hauptbildschirm. Ein physisches Abhängen eines Monitors und gemischte DPI sind weiterhin NOT_RUN; die Positionierungsregeln dafür sind automatisch getestet.

Das Statuspaket wurde auf macOS mit der gebündelten und der im Cargo-Binärpfad installierten CLI geprüft. Das neue IPC-Verzeichnis hat Rechte 0700, der Socket 0600. Vor der Freigabe und nach dem Abschalten liefert notify Exitcode 3. Nach Freigabe erscheint die Meldung mit Quelle und passender Geste; der finale Text einer langen Testmeldung ist durch Scrollen erreichbar. Beim Ausblenden wird eine Meldung angenommen und erst nach erneutem Anzeigen sichtbar. Die Freigabe bleibt nach Neustart erhalten. Die reale Registrierung von Claude-Code-Hooks, Kurzbefehlen, systemd-Units und Aufgabenplanung ist NOT_RUN; die Vorlagen werden nicht automatisch eingerichtet.
