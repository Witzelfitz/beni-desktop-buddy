# Schnelle Prüfungen und native Linux-Proben

Die drei schnellen Jobs prüfen Root und Desktop auf Linux, macOS und Windows: Formatierung, Unit-/Integrationsprüfungen, Clippy, Asset-/Release-Policy und Build. Feature-Branches laufen nur über PRs; Pushes nur auf main. Dokumentation bleibt ausgenommen. Release-Installer entstehen weiterhin ausschliesslich in den bewusst gestarteten Installer-Workflows.

## Native Probe getrennt

Ein eigener Linux-Job läuft nach erfolgreichen schnellen Prüfungen, wenn Desktop, native Abhängigkeiten, Assets, Toolchain oder Probe-Werkzeuge geändert wurden. Reine Dokumentations- und Änderungen an den unabhängigen Kern-/Chatverträgen starten ihn nicht. Bei manuellen Läufen ist die Probe aktiviert. Unbekannte Commit-Zuordnung ist ein Fehler; der erste Push ohne Vorgänger behandelt die Probe als notwendig.

Der schnelle Linux-Job exportiert sein bereits gebautes Desktop-Testprogramm. Debugsymbole werden nur in der Kopie entfernt; erwartete Testnamen, ELF-Architektur, SHA-256 und derselbe Workflow-Commit werden geprüft. Das Artefakt lebt einen Tag. Der Probejob installiert nur Laufzeit-/Testserverabhängigkeiten; er kompiliert Rust nicht erneut. Diese Datei ist kein öffentlicher Installer.

X11-Eingaberegionen und tatsächliche Hotkey-Konflikte werden unter einem eigenen Xvfb geprüft. Wayland erhält einen eigenen headless Weston mit Software-GL und privatem Laufzeitordner. Lokaler Einstieg bleibt `bash scripts/check-input-linux.sh` (Linux, Cargo und Testserver erforderlich).

## Retry-Grenze

Nur beim SIGSEGV des eigenen Test-Compositors (-11), fehlender erkannter Protokollverletzung und fehlgeschlagener Probe gibt es einmal einen frischen Versuch. Appfehler bei lebendem Compositor, Protokollfehler, gewöhnliche Timeouts und ein wiederholter Compositor-Ausfall bleiben Fehler. Ein erfolgreicher Wayland-Test verlangt einen weiterhin lebenden Compositor. Prozesse und Laufzeitordner werden anschliessend aufgeräumt.

Die schnellen Ergebnisse bleiben separat sichtbar. Eine fehlgeschlagene notwendige native Probe verhindert die vollständige grüne PR-Abnahme. Finale native Installerprüfungen und Hardware-/Compositor-Abnahmen sind zusätzlich erforderlich; diese Trennung ersetzt keine Release-Prüfung.
