# Härtung vor weiteren KI-Anbietern

Vom Nutzer am 14. Oktober 2026 angenommen. Umsetzung nacheinander, je ein eigener PR mit Rückmeldung. Produktverhalten und bisherige Grenzen erhalten; Änderungen mit positiven und negativen Prüfungen belegen.

| Schritt | Ergebnis und Abnahme | Stand |
| --- | --- | --- |
| H1 Einstieg | Kurze README, vorhandene Vorschau, Bedienung und Entwicklung in verlinkten Anleitungen; lokale Links und sachliche Angaben prüfen | Vorbereitet: [PR #58](https://github.com/Witzelfitz/beni-desktop-buddy/pull/58) |
| H2 CI | Schnelle Prüfungen von nativen Smoke-Prüfungen trennen, passende Pfadfilter und begrenzte Infrastruktur-Retries; App-/Protokollfehler bleiben Fehler, Release bleibt abnahmegebunden | Vorbereitet: [PR #59](https://github.com/Witzelfitz/beni-desktop-buddy/pull/59) |
| H3 Sprechblasen | Zentrale deterministische Entscheidung für Anzeigevorrang; eigene Warteschlangen/TTL erhalten, Sequenztests für Chat/Timer/Escape/Status ergänzen | Vorbereitet: [PR #60](https://github.com/Witzelfitz/beni-desktop-buddy/pull/60) |
| H4 eframe | Upstream-Basis und tatsächlichen Patch automatisch abgrenzen; Accessibility und synchrones Speichern absichern, Ablösekriterien festhalten | Vorbereitet: [PR #61](https://github.com/Witzelfitz/beni-desktop-buddy/pull/61) |
| H5 Werkzeuge | Python-Pipeline behalten; gemeinsamen Einstieg und Versions-/Abhängigkeitsregeln vereinfachen, keine Portierung nur wegen GitHub-Sprachanteilen | Vorbereitet: [PR #62](https://github.com/Witzelfitz/beni-desktop-buddy/pull/62) |

Stand: alle fünf PRs sind zur Durchsicht vorbereitet, noch nicht gemergt. H5 baut auf H4 auf; #61 vor #62 integrieren. Kern- und separate Desktopprüfungen sowie die isolierte native macOS-Gesprächsprüfung sind erfolgreich. Die CI-Ergebnisse stehen bei den jeweiligen PRs. Die fünf Branches liessen sich in einer isolierten lokalen Integration ohne Konflikte zusammenführen. Dort bestanden die gemeinsamen CI-/Vendor-/Toolingprüfungen, beide Formatprüfungen und die lokalen Dokumentationslinks. Ein öffentlicher Merge oder Release ist damit noch nicht erfolgt.

Eine kurze animierte Vorschau kann aus bestehenden SDK-Animationen entstehen; keine neuen Posen oder Bewegungsdaten. Die vorhandenen Screenshots bleiben für den Einstieg nutzbar. Eine echte Gesprächsdemo und weitere Geräteabnahmen werden getrennt dokumentiert.

Danach: sichere Zugangsdatenverwaltung, Claude und OpenAI jeweils separat. Native Gespräche auf Windows/Linux, physische Monitorwechsel und tatsächliches Nutzerupdate auf 0.1.5 bleiben offen.
