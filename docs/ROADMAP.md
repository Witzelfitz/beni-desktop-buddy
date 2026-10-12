# Roadmap

## 1 · Desktop-Abnahme und erster Build

- Erledigt: GUI-Abhängigkeiten, committete Lockfiles und erfolgreiche Tests, Clippy und Release-Builds auf macOS, Windows und Linux.
- Erledigt: nativer macOS-Start, Transparenz und PNG-Alpha; egui-UI-Tests für Menü, Nachricht, Pause/Fortsetzen, reduzierte Bewegung, Dragging und Beenden.
- Offen: alle Animationen und Persistenz manuell auf dem Desktop prüfen.
- Monitorwechsel und hohe Pixeldichte prüfen; einen verlorenen Fensterstand wiederherstellen können.
- Windows, Linux/X11 und Wayland separat prüfen und Ergebnisse dokumentieren.

Abnahme: Ein echter Build läuft auf dem Desktop und besteht die manuelle Testmatrix.

## 2 · Ein zuverlässiger Buddy

- Tray-Menü mit Anzeigen, Verbergen, Pause, Einstellungen und Beenden.
- Click-through für transparente Flächen mit einer verlässlichen Rückholmöglichkeit.
- Globale Rückhol-/Pause-Aktion, Benutzerwahl für Hotkeys.
- Grössenwahl und sicheres Begrenzen auf den sichtbaren Arbeitsbereich.
- OS-Einstellung für reduzierte Bewegung berücksichtigen.
- CPU-/GPU-Verbrauch über einen längeren Idle-Lauf messen, Hintergrundlast weiter reduzieren.

Abnahme: Beni lässt sich jederzeit wiederfinden und beenden, blockiert die Arbeit nicht und bleibt im Idle sparsam.

## 3 · Nützliche Reaktionen

- Lokale Erinnerungen und Fokus-/Pausen-Timer.
- Ereignisse wie `thinking`, `speaking`, `success` auf bestehende Animationsnamen abbilden.
- Zeitlich begrenzte Sprechblasen und kleine Szenen aus mehreren Gesten.
- Nutzerfreigegebene lokale IPC-Steuerung für andere Apps.

Abnahme: Zeit und Ereignisse sind testbar; Nachrichten sind begrenzt; externe Steuerung ist standardmässig aus.

## 4 · Installation und Updates

- Beni-App-Icon und plattformspezifische Pakete.
- macOS signieren und notarisieren; Windows-Signierung vorbereiten.
- Reproduzierbare Releases aus freigegebenen Tags.
- Optionaler, bewusst aktivierter Autostart.
- Updatekonzept erst nach den ersten stabilen Builds festlegen.

Abnahme: Pakete werden auf sauberen Geräten installiert und sauber deinstalliert; Signaturen und Updateverhalten sind geprüft.
