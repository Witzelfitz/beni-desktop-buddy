# Roadmap

Der verbindliche nächste Arbeitsabschnitt steht im [Masterplan für Distribution, Updates und KI](MASTERPLAN.md). R0/R1 sind abgeschlossen. R2 ist in PR #25 gemergt; die echte Notarisierung und lokale Apple-Silicon-Installation sind bestanden. Installer-CI, Intel-/Alt-OS-Abnahmen und öffentliche Downloads bleiben offen. Den Plan bei jeder Fortsetzung lesen und nach jedem abgeschlossenen PR aktualisieren.

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
- Erledigt: separates, verschiebbares und skalierbares Einstellungsfenster über das Menüleistensymbol oder die Buddy-Leiste; freie Buddy-Fläche und weiter angezeigte Statusmeldungen.
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

- Erledigt: lokaler Fokus-/Pausen-Timer mit wählbarer Dauer, Unterbrechen/Fortsetzen/Abbrechen, bestehenden Abschlussgesten und Sitzungsverlauf; unabhängig von IPC und Animationspause.
- Erledigt: bis zu zehn lokale Erinnerungen mit eigenem Text, Wartezeit, Entfernen und bestehender «Idee»-Geste; Sitzungsspeicher, Anzeigevorrang und Verlauf. Wiederholungen und Erinnerungen nach App-Neustart bleiben offen.
- Ereignisse wie `thinking`, `speaking`, `success` auf bestehende Animationsnamen abbilden.
- Erledigt: dynamische, scrollbar und schliessbar dargestellte Sprechblasen mit Lesedauer und TTL. Kleine Szenen aus mehreren Gesten bleiben offen.
- Erledigt: standardmässig ausgeschaltete lokale Status-IPC auf macOS/Linux/Windows, CLI notify/run und Integrationsvorlagen; begrenzte Nachrichten, TTL, Prioritäten und Gestenregeln.
- Erledigt: Sitzungsverlauf der letzten 100 angenommenen Statusmeldungen mit Quelle und Alter, auch bei «Nicht stören» und abgelaufener TTL; beim Nachlesen zeigt Beni «Denken».
- Erledigt: dezenter Idle-Ablauf ausschliesslich aus bestehenden SDK-Animationen («Warten», gelegentlich «Denken» und «Drehen»), mit Vorrang für Meldungen, manuelle Bedienung, Pause und Bewegungsreduktion.
- Optional: Embedding-Klassifikator mit eigenem Datensatz und Go/No-Go-Abnahme.

Abnahme: Zeit und Ereignisse sind testbar; Nachrichten sind begrenzt; externe Steuerung ist standardmässig aus.

## 4 · Installation und Updates

- Erledigt in PR #23: gemeinsamer [Release-Vertrag](RELEASES.md) mit Schema, Stable-/Beta-Beispielen, Offline-Prüfung und verbindlicher Cargo-Version/Buildnummer. Öffentliche Installer und echte Updateprüfung bleiben Folgearbeit.
- Gemergt in PR #25: [macOS-DMG und Release-Buildweg](MACOS_INSTALLER.md) für Apple Silicon/Intel, eindeutige Testpakete und verbindliche Apple-Abnahmen. Apple-Silicon-App und DMG tatsächlich notarisiert, Tickets und Gatekeeper geprüft; Nutzerinstallation unter Programme startet. CI-Secrets, Installer-CI, native Intel- und Alt-OS-Abnahme noch offen.
- Aktuelle Folgearbeit: [lokale Release-Pipeline](LOCAL_RELEASE.md) nach [ADR 0003](adr/0003-local-release-signing.md). Native CI-Builds ohne Apple-Schlüssel, lokale Signierung/Notarisierung und Ed25519 für Pakete und Manifest; bewusste Veröffentlichung bleibt ein eigener Schritt. Actions-Signing-Secrets sind für diesen gewählten Ablauf nicht erforderlich.
- Erledigt: Beni-App-Icon (PNG/ICNS/ICO), Windows-Exe-Ressource und Linux-Desktop-Vorlage in den plattformspezifischen Paketen.
- Erledigt: macOS-Entwicklungsbundle ad hoc signieren und streng prüfen; Apple-Silicon-Release mit Developer ID signieren und notarisieren. Windows-Signierung bleibt offen.
- Reproduzierbare Releases aus freigegebenen Tags.
- Optionaler, bewusst aktivierter Autostart.
- Geplant: Installer, Downloadseite, signierte Updates über das Menüleistensymbol und KI-Ersteinrichtung gemäss [Masterplan](MASTERPLAN.md).

Abnahme: Pakete werden auf sauberen Geräten installiert und sauber deinstalliert; Signaturen und Updateverhalten sind geprüft.
