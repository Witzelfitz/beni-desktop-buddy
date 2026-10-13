# Systemeinstellung für Bewegung

Beni reduziert die Bewegung, wenn entweder «Weniger Bewegung» manuell aktiviert ist oder «Systemeinstellung beachten» aktiviert ist und das System reduzierte Bewegung meldet. Die Systemvorgabe ändert weder die gespeicherte manuelle Einstellung noch Pause, Animationsfortschritt, Nachrichtenfristen oder den Meldungsverlauf.

Der neue Schalter ist standardmässig an, auch bei bestehenden Installationen. Er wird getrennt als `follow-system-motion` gespeichert. Ein beschädigter gespeicherter Wert fällt auf «an» zurück. Die erkannte OS-Vorgabe wird nicht persistiert. Wird die Abfrage nicht unterstützt oder schlägt sie fehl, gilt die manuelle Auswahl; das Menü zeigt den aktuellen Status. Zum bewussten Übergehen einer Systemvorgabe den neuen Schalter ausschalten.

| Plattform | Quelle | Aktualisierung |
| --- | --- | --- |
| macOS | [`NSWorkspace.accessibilityDisplayShouldReduceMotion`](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducemotion) | Lesen auf dem Hauptthread, höchstens einmal pro Sekunde |
| Windows | [`SystemParametersInfoW`, `SPI_GETCLIENTAREAANIMATION`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow) | Lesen höchstens einmal pro Sekunde; deaktivierte Animationen bedeuten reduzierte Bewegung |
| Linux, X11/Wayland | [Desktop-Portal Settings](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html), `ReadOne("org.freedesktop.appearance", "reduced-motion")` | Ein Hintergrundthread liest alle fünf Sekunden; Methodenaufrufe mit einer Sekunde Timeout, geänderte Werte wecken die Oberfläche |

Unter Linux bedeutet der standardisierte Wert `1` reduzierte Bewegung, `0` keine Präferenz. Andere numerische Werte gelten ebenfalls als keine Präferenz. Fehlende Dienste, nicht unterstützte Schlüssel und falsche Datentypen gelten als nicht verfügbar. Es werden ausschliesslich diese lesenden Settings-Aufrufe über den lokalen Session-Bus verwendet. Keine GNOME-/KDE-Konfigurationsdateien werden verändert, keine globalen Systemeinstellungen umgeschaltet. Der Hintergrundthread erhält beim Ende der App ein Stoppsignal; D-Bus-Verbindung und Abfragen blockieren den GUI-Thread nicht.

Der Animationskern bleibt unverändert und erhält nur eine abgeleitete Kopie seiner `Settings`. Die effektive Reduktion steuert sowohl die Pose als auch das sparsame Zeichenbudget aus [PERFORMANCE.md](PERFORMANCE.md). Der bestehende Sekundentakt der Desktop-Logik bleibt erhalten; die OS-Abfrage startet keine zusätzliche schnelle Zeichen- oder Polling-Schleife.

## Prüfung

- PASS lokal auf macOS: UI-Tests für Aktivieren/Übergehen, alle Fenstergrössen, benannter Accessibility-Schalter, Pause beim Lesen und unveränderte Nachrichtenfristen. Gespeicherte manuelle Auswahl und Systemausnahme bleiben getrennt; unbekannte bzw. beschädigte Werte haben einen definierten Rückfall.
- PASS: beide Zustände der Systemvorgabe verändern Pose und Zeichenbudget, ohne Animation oder Fortschritt zurückzusetzen; manuelle Reduktion bleibt bei ausgeschalteter Systembeachtung und ausgefallener Quelle aktiv.
- PASS nativ auf macOS: derselbe Rust-Adapter liest `Some(false)`, bestätigt durch den direkten AppKit-Aufruf und die Statusanzeige im Buddy-Menü. Der neue Schalter lässt sich über die native Accessibility-API ausschalten und wieder einschalten; die manuelle Reduktion bleibt erhalten. Geordneter Neustart bestätigt Speicherung und alle bisherigen Darstellungs-/Empfangseinstellungen.
- Linux: ein automatischer Test verwendet einen eigenen D-Bus-Daemon ohne Dienstaktivierung. Ein Fake-Portal prüft echte `ReadOne`-Aufrufe, Änderungen 0/1, unbekannte numerische Werte, fehlende Schlüssel, falschen Typ und verschwundenen Dienst. Ausführung auf Linux erfolgt in der PR-CI; auf macOS wurde der gemeinsame Portal-/Testcode separat erfolgreich kompiliert, der D-Bus-Test nicht ausgeführt.
- NOT_RUN: globalen Bewegungs-Schalter des Betriebssystems während des laufenden Buddy verändern; native Windows-/Linux-Desktop-Abnahme. Der macOS-Systemwert wurde nur gelesen, keine globale Einstellung verändert.
