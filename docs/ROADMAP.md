# Roadmap

## 1 · Desktop-Abnahme und erster Build

- Erledigt: GUI-Abhängigkeiten, committete Lockfiles und erfolgreiche Tests, Clippy und Release-Builds auf macOS, Windows und Linux.
- Erledigt: nativer macOS-Start, Transparenz und PNG-Alpha; egui-UI-Tests für Menü, Nachricht, Pause/Fortsetzen, reduzierte Bewegung, Dragging und Beenden.
- Offen: alle Animationen und Persistenz manuell auf dem Desktop prüfen.
- Erledigt: verlorene Fensterposition beim Start und nach Monitorwechsel korrigieren, Rückholen auf den Hauptbildschirm. Physische Monitorwechsel und gemischte Pixeldichten bleiben manuell zu prüfen.
- Windows, Linux/X11 und Wayland separat prüfen und Ergebnisse dokumentieren.

Abnahme: Ein echter Build läuft auf dem Desktop und besteht die manuelle Testmatrix.

## 2 · Ein zuverlässiger Buddy

- Erledigt: Tray-Menü mit Anzeigen, Zurückholen, Verbergen, Pause, Einstellungen und Beenden.
- Erledigt: zeitlich begrenzte Fehleranzeige und weiter bedienbare Ersatzfläche bei Zeichenfehlern.
- Erledigt: Bedienleiste startet eingeklappt und lässt sich per Klick auf Beni oder Escape wieder zuklappen, ohne laufende Statusmeldungen zu ersetzen.
- Erledigt: native Accessibility-Anbindung, beschrifteter Avatar und Tastaturbedienung. Vollständige VoiceOver-/NVDA-/Orca-Abnahme bleibt offen.
- Erledigt: durchklickbare transparente Flächen anhand der Beni-Silhouette und sichtbaren UI; im Menü abschaltbar, mit Tray-Rückholmöglichkeit und bedienbarem Fehler-Fallback.
- Erledigt: freiwillig aktivierte, gespeicherte globale Rückhol-/Pause-Hotkeys mit frei wählbaren Kombinationen, Eingabe-/Konfliktprüfung und Wiederholungsschutz auf macOS/Windows/X11. Wayland-Portal-Unterstützung bleibt offen.
- Erledigt: gespeicherte Grössenwahl «Klein», «Mittel», «Gross» mit lesbarer Bedienung, angepasstem Einstellungsmenü und erneuter Positionsprüfung nach Grössenänderungen.
- Erledigt: gespeichertes «Nicht stören» im Menü und Tray; Statusanzeigen aller Prioritäten pausieren, während TTL und Empfang weiterlaufen.
- Erledigt: Systemeinstellung für reduzierte Bewegung auf macOS/Windows und über Linux-Desktop-Portale berücksichtigen, mit gespeicherter Ausnahmemöglichkeit und unveränderter manueller Auswahl. Native Windows-/Linux-Abnahme und ein realer Wechsel der globalen OS-Einstellung bleiben offen.
- Erledigt: unnötige Zeichenanforderungen bei Pause und reduzierter Bewegung von vier auf eine pro Sekunde reduzieren, mit separaten Textfristen und direktem IPC-Wecken; kurze CPU-/Wakeup-Vergleichsmessung auf macOS dokumentieren.
- Offen: CPU-/GPU-Verbrauch über einen längeren Idle-Lauf messen, Akkuverbrauch und native Windows-/Linux-Last prüfen.

Abnahme: Beni lässt sich jederzeit wiederfinden und beenden, blockiert die Arbeit nicht und bleibt im Idle sparsam.

## 3 · Nützliche Reaktionen

- Lokale Erinnerungen und Fokus-/Pausen-Timer.
- Ereignisse wie `thinking`, `speaking`, `success` auf bestehende Animationsnamen abbilden.
- Erledigt: dynamische, scrollbar und schliessbar dargestellte Sprechblasen mit Lesedauer und TTL. Kleine Szenen aus mehreren Gesten bleiben offen.
- Erledigt: standardmässig ausgeschaltete lokale Status-IPC auf macOS/Linux/Windows, CLI notify/run und Integrationsvorlagen; begrenzte Nachrichten, TTL, Prioritäten und Gestenregeln.
- Erledigt: Sitzungsverlauf der letzten 100 angenommenen Statusmeldungen mit Quelle und Alter, auch bei «Nicht stören» und abgelaufener TTL; beim Nachlesen zeigt Beni «Denken».
- Optional: Embedding-Klassifikator mit eigenem Datensatz und Go/No-Go-Abnahme.

Abnahme: Zeit und Ereignisse sind testbar; Nachrichten sind begrenzt; externe Steuerung ist standardmässig aus.

## 4 · Installation und Updates

- Erledigt: Beni-App-Icon (PNG/ICNS/ICO), Windows-Exe-Ressource und Linux-Desktop-Vorlage in den plattformspezifischen Paketen.
- Erledigt: macOS-Entwicklungsbundle ad hoc signieren und streng prüfen. Developer-ID-Signierung und Notarisierung sowie Windows-Signierung bleiben offen.
- Reproduzierbare Releases aus freigegebenen Tags.
- Optionaler, bewusst aktivierter Autostart.
- Updatekonzept erst nach den ersten stabilen Builds festlegen.

Abnahme: Pakete werden auf sauberen Geräten installiert und sauber deinstalliert; Signaturen und Updateverhalten sind geprüft.
