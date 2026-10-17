# Roadmap

Der verbindliche nächste Arbeitsabschnitt steht im [Masterplan für Distribution, Updates und KI](MASTERPLAN.md). Version 0.1.8/Build 9 ist für beide Macs und Linux veröffentlicht; Downloadseite und beide signierten Mac-Feeds sind live. Die Gesprächsblase hängt an Beni und verwendet Hochdeutsch mit Schweizer Rechtschreibung sowie geprüfte JSON-Emotionen mit vorhandenen Gesten. Audio bleibt zurückgestellt. Der Nutzer kann über das Menüleistensymbol aktualisieren; die tatsächliche Installation/Gesprächsabnahme folgt. Windows wartet weiterhin auf Firmenprüfung und Herausgebersignatur; zusätzliche Zielgeräte-/Updatefälle bleiben offen.

Die [Härtungen H1–H5](HARDENING.md) aus dem Review sind in PR #58–#62 gemergt: Dokumentation, getrennte native CI-Proben, zentrale Sprechblasenregeln, begrenzter eframe-Patch und gemeinsamer Prüfeinstieg. Das Menü-Rad ist seit 0.1.6 veröffentlicht; 0.1.7 ergänzt kompakte Fokus-/Erinnerungskarten und die optische Korrektur. Website-PR #14 zeigt echte App-Bilder. 0.1.8 ergänzt längere Idle-Ruhephasen und vermeidet redundante native Updates. Eine bestimmte Akkuersparnis ist noch nicht gemessen. Als Nächstes folgen verständliche Zustände und gelegentliche Zwischenstände. Eigene Cloud-Keys sind zurückgestellt; [gemeinsamer Produkt- und Veröffentlichungsplan](EVERYDAY_BUDDY.md). Die tatsächliche Updateinstallation und Alltagserprobung auf dem Nutzergerät bleiben separat offen.

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

- Erledigt in PR #23: gemeinsamer [Release-Vertrag](RELEASES.md) mit Schema, Stable-/Beta-Beispielen, Offline-Prüfung und verbindlicher Cargo-Version/Buildnummer. Öffentliche Installer sind inzwischen unter R2/R5 veröffentlicht; die tatsächliche Nutzerinstallation A → B ist inzwischen bestätigt.
- Gemergt in PR #25: [macOS-DMG und Release-Buildweg](MACOS_INSTALLER.md), erstes Apple-Silicon-DMG tatsächlich notarisiert, Tickets/Gatekeeper und Nutzerinstallation bestanden. PR #28 korrigiert die Intel-Signierreihenfolge; beide nativen Entwicklungsinstaller auf macOS 15.7.9 geprüft.
- Gemergt in PR #27: [lokale Release-Pipeline](LOCAL_RELEASE.md) nach [ADR 0003](adr/0003-local-release-signing.md). Native CI ohne Apple-Schlüssel, lokale Signierung/Notarisierung und Ed25519 für Pakete/Manifest. Echter gemeinsamer Kandidat 0.1.0/Build 1 für Intel und Apple Silicon erstellt und geprüft, Release-Entwurf mit acht zurückgeladenen/authentifizierten Assets vorhanden. Öffentliches Release 0.1.0 inzwischen freigegeben und alle anonymen Downloads geprüft. Actions-Signing-Secrets nicht erforderlich.
- Erledigt: Beni-App-Icon (PNG/ICNS/ICO), Windows-Exe-Ressource und Linux-Desktop-Vorlage in den plattformspezifischen Paketen.
- Erledigt: macOS-Entwicklungsbundle ad hoc signieren und streng prüfen; Apple-Silicon-Release mit Developer ID signieren und notarisieren. Windows-Signierung bleibt offen.
- R3a/R3b gemergt: Sparkle im Bundle, Tray-Updateaktion, automatische Prüfwahl und kontrollierter Neustart. R3c und Bootstrap 0.1.2 veröffentlicht: v2-Manifest, integrierte signierte Feeds und eigener Website-Adapter sind live. [Release-Ablauf](UPDATE_RELEASE_RUNBOOK.md); Nutzerinstallation A und tatsächliches Update A → B sind bestätigt. Der konkrete verschobene Neustart bleibt als zusätzliche manuelle Abnahme offen.
- Reproduzierbare Releases aus freigegebenen Tags.
- Optionaler, bewusst aktivierter Autostart.
- Mac-/Linux-Installer und Downloadseite veröffentlicht; Mac-Updateprüfung ab 0.1.2 verfügbar. Windows-Signierung, Linux-Updateadapter und KI-Ersteinrichtung bleiben offen gemäss [Masterplan](MASTERPLAN.md).

Abnahme: Pakete werden auf sauberen Geräten installiert und sauber deinstalliert; Signaturen und Updateverhalten sind geprüft.

## 5 · Gespräche mit Beni

- Erledigt in PR #54 und 0.1.5 veröffentlicht: Gesprächsblase an der Figur, Hochdeutsch mit Schweizer Rechtschreibung, JSON-Antwortabschnitte und bekannte Emotionen/Gesten. Nutzer wünscht vorerst Text; Audio bleibt später.

- Gemergt: gemeinsamer Chatvertrag und begrenzte Sitzung in `buddy-chat`, R7a/PR #49, siehe [ADR 0005](adr/0005-chat-provider-contract.md).
- Gemergt in PR #50, seit 0.1.4 verfügbar und in 0.1.5 durch die Blase aus PR #54 ergänzt: Beni anklicken → «Reden», freiwillige Ollama-Verbindung und vorhandenes geeignetes Textmodell. Native macOS-Chatabnahme bestanden; echte Chatabnahme auf Windows/Linux separat offen. [Chat-Anleitung](CHAT.md).
- Sichere Zugangsdatenverwaltung vor Claude-/OpenAI-Verbindungen; jeder Anbieter mit eigenem PR.
- Bestehende Gesten «Denken» und «Reden», keine neuen Animationen und keine Systemwerkzeuge.

Abnahme: Hintergrundanfragen bleiben abbrechbar, Gesprächskontext ist sichtbar und begrenzt, Keys liegen ausserhalb des gewöhnlichen App-Speichers.
