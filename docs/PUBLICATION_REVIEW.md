# Vorbereitung der öffentlichen Freigabe

Stand: 13. Oktober 2026. Geprüfter Implementierungsstand: main `bfc2ca8`, einschliesslich der lokal verfügbaren Git-Referenzen. Die öffentliche Freigabe betrifft ausschliesslich `Witzelfitz/beni-desktop-buddy`; Beni-Allrounder bleibt getrennt.

Der Nutzer erlaubt ein öffentliches Desktop-Repo zur Vereinfachung der Downloads und hat MIT für den Rust-Code mit gesonderten Rechten für Beni-Grafiken gewählt. [Rechte und Herkunft](../LICENSE.md) beschreibt die Abgrenzung; [MASTERPLAN](MASTERPLAN.md) hält die anschliessenden Release-Schritte fest.

## Ausgeführte Prüfung

- PASS: gezielte redigierte Musterprüfung über 292 verfügbare Text-Blobs im Git-Verlauf; keine Treffer für private Schlüssel, GitHub-/Provider-Token und AWS-Zugangsschlüssel. Lokal verfügbare Historie: 50 Commits einschliesslich Merge-/Referenzhistorie.
- PASS: Gitleaks 8.30.1 über alle verfügbaren Git-Referenzen, mit vollständiger Redigierung der Ausgabe. 35 Änderungscommits, rund 2,12 MB; keine erkannten Leaks. Der macOS-arm64-Download wurde gegen die veröffentlichte SHA-256-Prüfsumme verifiziert.
- PASS: alle 61 zum Prüfzeitpunkt aufgeführten abgeschlossenen Actions-Runs: Logs heruntergeladen und mit Gitleaks geprüft. Rund 18,66 MB; keine erkannten Leaks. Keine dieser Run-Logabfragen war unverfügbar.
- PASS: zwei eingecheckte macOS-Abbildungen visuell geprüft: eigener Beni und eigene Teststatusmeldung, keine fremden Desktop-Inhalte sichtbar.
- PASS: keine Actions-Secrets im Desktop-Repo konfiguriert. Workflow-Pakete enthalten App/CLI, Integrationsvorlagen, Dokumentation und Icons; in den geprüften eingebetteten Rust-Dateien werden Projekt-Assets und normale Cargo-Pfade verwendet.
- PASS: ursprüngliche Lizenzentscheidung ersetzt durch den vom Nutzer gewählten MIT-Geltungsbereich; Beni-Assets, SDK-Referenzdaten und Drittanbieter-Lizenzen bleiben gesondert.

Ein musterbasierter Scan kann unbekannte Geheimnisformate oder andere vertrauliche Inhalte übersehen. Die Prüfung ist keine Garantie vollständiger Geheimnisfreiheit. Die vorhandenen Actions-Artefakte wurden nicht alle einzeln entpackt; Git-Verlauf, Workflow, Logs und eingecheckte Abbildungen wurden geprüft.

## Auswirkung der Sichtbarkeitsänderung

Quellcode, enthaltene Grafiken, Git-Verlauf, PRs sowie Actions-Historie und Logs werden öffentlich zugänglich. Vorhandene Kopien und Forks lassen sich durch eine spätere Rückkehr zu privat nicht zurückholen. Die gesonderten Rechte an Beni bleiben bestehen; Sichtbarkeit und Open-Source-Lizenz sind getrennte Entscheidungen.

Die tatsächliche GitHub-Sichtbarkeit muss nach dem Umschalten per API bestätigt werden. Eine öffentliche Freigabe stellt noch keinen Installer bereit. Release-Vertrag, signierte Pakete und Update-Feeds folgen gemäss Masterplan; die aktuellen Actions-Archive bleiben Entwicklungsartefakte.

Referenzen: [Gitleaks 8.30.1](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1), [GitHub-Sichtbarkeit](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility), [GitHub-Lizenzen](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository).
