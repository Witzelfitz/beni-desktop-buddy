# Härtung vor weiteren KI-Anbietern

Vom Nutzer am 14. Oktober 2026 angenommen. Umsetzung nacheinander, je ein eigener PR mit Rückmeldung. Produktverhalten und bisherige Grenzen erhalten; Änderungen mit positiven und negativen Prüfungen belegen.

| Schritt | Ergebnis und Abnahme | Stand |
| --- | --- | --- |
| H1 Einstieg | Kurze README, vorhandene Vorschau, Bedienung und Entwicklung in verlinkten Anleitungen; lokale Links und sachliche Angaben prüfen | Gemergt: [PR #58](https://github.com/Witzelfitz/beni-desktop-buddy/pull/58) |
| H2 CI | Schnelle Prüfungen von nativen Smoke-Prüfungen trennen, passende Pfadfilter und begrenzte Infrastruktur-Retries; App-/Protokollfehler bleiben Fehler, Release bleibt abnahmegebunden | Gemergt: [PR #59](https://github.com/Witzelfitz/beni-desktop-buddy/pull/59) |
| H3 Sprechblasen | Zentrale deterministische Entscheidung für Anzeigevorrang; eigene Warteschlangen/TTL erhalten, Sequenztests für Chat/Timer/Escape/Status ergänzen | Gemergt: [PR #60](https://github.com/Witzelfitz/beni-desktop-buddy/pull/60) |
| H4 eframe | Upstream-Basis und tatsächlichen Patch automatisch abgrenzen; Accessibility und synchrones Speichern absichern, Ablösekriterien festhalten | Gemergt: [PR #61](https://github.com/Witzelfitz/beni-desktop-buddy/pull/61) |
| H5 Werkzeuge | Python-Pipeline behalten; gemeinsamen Einstieg und Versions-/Abhängigkeitsregeln vereinfachen, keine Portierung nur wegen GitHub-Sprachanteilen | Gemergt: [PR #62](https://github.com/Witzelfitz/beni-desktop-buddy/pull/62) |

Stand: alle fünf PRs wurden am 14. Oktober 2026 in main gemergt, H4 vor H5. Die lokale Kopie ist synchronisiert. Der gemeinsame Prüfeinstieg mit `--desktop` besteht auf dem integrierten Stand: 47 Roottests, 110 Desktoptests, fmt, Clippy und GUI-Build sowie Packaging-, Release-, Vendor-, Tooling- und Speicherprüfungen. CI-Policytests und actionlint bestehen zusätzlich. Die isolierte native macOS-Gesprächsprüfung aus H3 besteht; ein echter Ollama-Lauf und neue Installer-Abnahmen wurden für diese Härtungen nicht wiederholt.

Die Branches liessen sich zuvor in einer isolierten lokalen Integration ohne Konflikte zusammenführen. Die abschliessende main-CI auf macOS, Windows und Linux sowie die separaten nativen Linux-Proben bestehen; [Prüfbericht](TEST_MATRIX.md). Diese Härtungen sind Quellcodeänderungen nach dem öffentlichen Release 0.1.5; eine neue installierbare Version wurde damit noch nicht veröffentlicht.

Eine kurze animierte Vorschau kann aus bestehenden SDK-Animationen entstehen; keine neuen Posen oder Bewegungsdaten. Die vorhandenen Screenshots bleiben für den Einstieg nutzbar. Eine echte Gesprächsdemo und weitere Geräteabnahmen werden getrennt dokumentiert.

Danach: sichere Zugangsdatenverwaltung, Claude und OpenAI jeweils separat. Native Gespräche auf Windows/Linux, physische Monitorwechsel und tatsächliches Nutzerupdate auf 0.1.5 bleiben offen.
