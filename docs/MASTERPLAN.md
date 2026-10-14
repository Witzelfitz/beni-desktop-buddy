# Masterplan für Beni Desktop Buddy

Beni wird als native Rust-App über `https://mini.witzelfitz.io/download` angeboten. Installation, Updates über das Menüleistensymbol und eine freiwillige KI-Einrichtung gehören zum Produkt. Website und Desktop-App bleiben in getrennten Repositories. Dieser Plan ist der verbindliche Arbeitsstand für weitere Sessions; die Umsetzung erfolgt Schritt für Schritt mit eigenen PRs und einer Rückmeldung nach jedem PR.

Stand: 14. Oktober 2026. Desktop-Version [0.1.1/Build 2](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.1) ist für macOS und Linux öffentlich veröffentlicht. Beide finalen Developer-ID-Pakete für Intel und Apple Silicon bestehen native Installation, CLI und GUI auf macOS 15.7.9 sowie Signatur-, Notarisierungs-, Gatekeeper- und Ed25519-Prüfungen. Downloadseite in Beni-Allrounder gemergt, über Coolify deployed und live unter `mini.witzelfitz.io/download` abgenommen. R3 beginnt mit authentifizierten Sparkle-Feeds; Updates über das Menüleistensymbol und KI-Verbindungen folgen.

## Festgelegte Entscheidungen

- `Witzelfitz/beni-desktop-buddy`: Rust-Code, Packaging, Updater, KI-Adapter und Release-Workflows.
- `Witzelfitz/Beni-Allrounder`: Website, neue Route `/download`, Gestaltung und Installationshilfe. Hosting-Dokumentation: Node auf Coolify. Das Website-Repo bleibt eigenständig.
- Der Nutzer erlaubt ein öffentliches Desktop-Repo, wenn dies die Distribution vereinfacht. Bevorzugte Distribution: öffentliche GitHub Releases im Desktop-Repo; damit ist kein zusätzliches Release-Repository erforderlich. Die Website bleibt unabhängig von Rust-Builds.
- Eigenständiger Rust-Code wird unter MIT veröffentlicht. Beni-Figur, Grafiken, App-Icon, importierte Animationsdaten und Referenzdaten behalten ihre gesonderten Rechte; siehe [Rechte und Herkunft](../LICENSE.md).
- Rust und egui/eframe bleiben bestehen. Kein Framework-Wechsel allein für Installer oder Updates.
- Nur vorhandene Beni-Animationen verwenden; keine neuen Sprites oder Bewegungsdaten.
- Nutzer wählen beim ersten Start direkt nach der Installation: ohne KI/später, Ollama lokal, Claude oder OpenAI. Diese Ersteinrichtung gehört zum Installationsablauf und bleibt später in den Einstellungen erreichbar.
- Eigene Nutzer-Keys, kein eingebauter gemeinsamer Cloud-Key. Anbieterkommunikation erfolgt aus der Desktop-App; die Website erhält keine Keys oder Chattexte.
- Anzeigen dürfen nur tatsächlich verfügbare Plattformpakete und funktionierende Anbieter versprechen.

## Arbeitsweise und Fortschritt

Vor jeder Fortsetzung diesen Plan und die Roadmap lesen, offene PRs und den aktuellen Stand prüfen. Den nächsten offenen Schritt umsetzen; erfüllte Abnahmen mit PASS, FAIL, NOT_RUN oder BLOCKED dokumentieren. Fertige Schritte mit PR und konkretem Ergebnis ergänzen. Eine Architekturänderung oder ein grösserer Umfang wird vor der Umsetzung im Plan begründet.

Neue Aufgaben auf Branches, keine direkten Folgeänderungen auf main und kein Force-Push. Pro PR eine Rückmeldung mit Link, Verhalten, Prüfung und offenen Abnahmen. Bereits erteilte Freigaben beachten; keine erneute Bestätigung für autorisierte Routinearbeit verlangen.

**Aktueller Schritt: R3, Updatekette auf Nutzerwunsch während der Microsoft-Prüfung.** R3a erzeugt und prüft lokale signierte Sparkle-Appcasts aus den authentifizierten Release-Kandidaten. Der native Versuch mit SDK 2.10.0 bestätigt die bestehende Paket- und neue Feed-Signatur; keine Schlüsselrotation nötig. Als Nächstes: Framework-/Bundle-Einbettung und Tray-Anbindung, danach koordinierte Veröffentlichung und echtes Update A → B. [Ablauf](UPDATES.md), [ADR 0004](adr/0004-sparkle-update-chain.md). Noch kein Updater in 0.1.1 und keine öffentlichen Appcasts.

[Version 0.1.1/Build 2](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.1) ist für macOS (Intel/Apple Silicon) und Linux (x86_64, GLIBC 2.35) veröffentlicht; alle elf anonymen Dateien sind bytegleich/authentifiziert, der echte Linux-Download auf der Live-Seite ist geprüft. Windows-NSIS pro Benutzer ist gebaut und nativ abgenommen, aber noch nicht öffentlich. Microsoft Artifact Signing Basic ist ausdrücklich freigegeben; Konto `witzelfitzbeni`, Ressourcengruppe `beni-release-signing`, Switzerland North, Provider und Identity-Verifier-Rolle sind bestätigt. Die öffentliche Firmenprüfung von Witzelfitz Consulting GmbH ist eingereicht, laut Nutzer «In Progress». Danach folgen Public Trust und OIDC-Signierung. [Anleitung](WINDOWS_SIGNING.md). Installer-Builds bleiben manuell und gezielt; bestehende Releases unverändert.

| Schritt | Repository | Status | Ergebnis und Abnahme |
| --- | --- | --- | --- |
| R0 | Desktop | Erledigt | [PR #21](https://github.com/Witzelfitz/beni-desktop-buddy/pull/21): Masterplan und MIT-Abgrenzung; drei Plattformprüfungen grün. Git-Verlauf und 61 Actions-Logs geprüft, keine erkannten Leaks. Öffentliche Sichtbarkeit per API und anonymem HTTP-Zugriff bestätigt; Beni-Allrounder bleibt privat. [Prüfbericht](PUBLICATION_REVIEW.md). |
| R1 | Desktop | Erledigt | [PR #23](https://github.com/Witzelfitz/beni-desktop-buddy/pull/23) gemergt; PR-/main-CI auf drei Plattformen grün. [Release-Vertrag](RELEASES.md), JSON-Schema, Stable-/Beta-Beispiele, Offline-Validator, 19 Tests und verbindliche Cargo-Version/Buildnummer. Noch keine veröffentlichten Installer, Feed oder kryptografische Updateprüfung. |
| R2 | Desktop | Erledigt für macOS 15+; exakte 15.0-/frische Nutzergeräte-Abnahme offen | PR #25/#27/#28: Installer und lokale Signierung. PR #30/#31/#32: echter Kandidat und finale Abnahme. [Finale native Intel-/Apple-Silicon-Installer](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38524476222) auf macOS 15.7.9: Installation, CLI, sichtbare GUI und Entfernung bestanden; Belege stimmen mit finalen Paket-Hashes überein. [0.1.0/Build 1 veröffentlicht](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.0); alle acht anonymen Downloads bytegleich und authentifiziert. Developer ID, Apple-Notarisierung, Tickets und Gatekeeper bestanden. |
| W1 | Beni-Allrounder | Erledigt | [PR #1](https://github.com/Witzelfitz/Beni-Allrounder/pull/1) und Metadaten-/Abnahmeergänzung PR #2: `/download` mit bestehenden Assets, OS-Vorauswahl, manueller Plattform-/Chip-Wahl, Version, Grösse, Anforderungen und Installationshilfe. Node authentifiziert ausschliesslich den öffentlichen Stable-Katalog. 51 Tests, zwölf relevante Tests auf Node 22.16.0 und Desktop/Mobil bestanden. Coolify-Deployment und Live-Domain geprüft, echter Browser-Download bytegleich zum signierten Original. Safari/Firefox separat offen. |
| R3 | Desktop | R3a vorbereitet und geprüft; Desktop-Anbindung offen | [Appcast-Helfer](UPDATES.md), 15 Offline-Regressionsprüfungen und native Sparkle-2.10.0-Signaturkompatibilität bestanden. Architektur-/Kanaltrennung, Paket-/Manifest-/Feed-Manipulation und kein Überschreiben geprüft. Nächste Schritte: Framework-/Tray-Anbindung, gemeinsame Veröffentlichung, echtes Update A → B sowie Downloadabbruch und nicht schreibbarer Installationsort. |
| R4 | Desktop | Installer geprüft; Signierung in Einrichtung | PR #34–#37: NSIS pro Benutzer für Windows 11 x86_64. [Nativer Windows-Lauf](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38537805266): Installation, CLI, eigenes Fenster, Schutz der laufenden App und Deinstallation bestanden; virtueller Server-2025-Runner mit isoliertem Mesa-Testtreiber. Microsoft-Firmenprüfung eingereicht, «In Progress»; Public Trust, OIDC und öffentliche signierte Windows-Ausgabe folgen nach Freigabe; Hardware-/Windows-11- und echte Update-Abnahme offen. |
| R5 | Desktop | Installer veröffentlicht; Updates offen | PR #34–#37: AppImage mit bewusster Menüeinbindung, belegter Deinstallation, Bibliothekslizenzen und GLIBC-2.35-Prüfung. [Ubuntu-22.04-Abnahme](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38536064173): CLI, X11-/Wayland-Oberfläche, Menüinstallation/-entfernung bestanden. 0.1.1 öffentlich, Ed25519-authentifizierter Katalog, OS-/CPU-Auswahl und echter Browser-Download auf mini.witzelfitz.io geprüft. Updateadapter und weitere manuelle Desktop-/Distributionsabnahmen offen. |
| R6 | Desktop | Offen | Gemeinsame Ersteinrichtung: ohne KI/später und bereits implementierte Anbieter. Auswahl später ändern; mehrere Verbindungen speichern, pro Chat einen Anbieter auswählen. Updates bewahren Auswahl und Keys. |
| R7 | Desktop | Offen | Gemeinsame Chat-Grundlage und sichere Zugangsdaten: eigener Rust-Bereich für Netzwerk/Streaming, Abbruch, Timeouts, begrenzten Verlauf und Antwort-/Tokenlimits. Animationskern bleibt netzwerkfrei. Eigener lesbarer Chat-Viewport. |
| R8 | Desktop | Offen | Ollama zuerst: lokalen Dienst erkennen, vorhandenes geeignetes Chatmodell wählen, Verbindung testen. Modell-Downloads erst nach Auswahl und Anzeige von Grösse/Anforderungen. Kein stiller Wechsel auf Cloud-Modelle. |
| R9 | Desktop | Offen | Claude-Adapter über Messages API mit eigenem API-Key, Modellwahl, Streaming und klaren Fehlern/Kostenhinweisen. |
| R10 | Desktop | Offen | OpenAI-Adapter über Responses API mit eigenem API-Key, Modellwahl, Streaming und klaren Fehlern/Kostenhinweisen. |
| R11 | Beide | Offen | Zusammenhängende Abnahme Download → Installation → Einrichtung → Chat → Update → Deinstallation; neue und aktualisierte Installation auf echten Zielplattformen. Website-Aussagen mit tatsächlich verfügbaren Funktionen abgleichen. |

R6/R7 definieren die Ersteinrichtung und Schnittstellen; R8–R10 ergänzen die Anbieter jeweils in eigenen PRs. Eine stabile Veröffentlichung bietet erst die bereits fertig geprüften Auswahlmöglichkeiten an. Für R3 ist ein signiertes, installierbares Paket aus R2 Voraussetzung. W1 kann mit dessen tatsächlich veröffentlichtem Download starten.

## Release und Updatevertrag

Die Desktop-Release-Pipeline ist die einzige Quelle für Version, Pakete und Metadaten. Veröffentlichte Dateien liegen unter versionsgebundenen Adressen; ein freigegebener stabiler Verweis bezeichnet die aktuelle Version. Alle Pakete, Signaturen und Metadaten einer Version werden zuerst vollständig bereitgestellt, dann wird der stabile Verweis aktualisiert. Ein teilweiser Build darf kein Update ankündigen.

Downloadseite und Updater verwenden dieselbe Veröffentlichung. Plattformadapter können daraus die benötigten Sparkle-/WinSparkle-Appcasts oder AppImage-Informationen erhalten. Stable und Beta bleiben getrennt.

Die Downloadseite erkennt nach Nutzerwunsch macOS, Windows oder Linux anhand verfügbarer Browserangaben und wählt das passende Betriebssystem sichtbar vor. Ein hervorgehobener Button nennt das konkrete Ziel, beispielsweise «Für macOS herunterladen». Alle Plattformen bleiben manuell erreichbar; eine Nutzerwahl wird durch spätere Erkennungsantworten nicht überschrieben. Architektur nur bei zuverlässigen Angaben automatisch auswählen; andernfalls eine verständliche Auswahl wie «Apple Silicon»/«Intel» anbieten. Fehlende oder widersprüchliche Angaben führen zur Auswahl, nicht zu einer geratenen Datei. Auf iOS/Android wird kein Desktop-Paket automatisch ausgewählt. Erkennung startet keinen automatischen Download und bietet nur tatsächlich veröffentlichte, kompatible Pakete an; fehlt ein Ziel, wird dessen Verfügbarkeit erklärt. Abnahme: macOS/Windows/Linux, beide Mac-Architekturen, Mobilgeräte, unbekannte/gesperrte Browserangaben, fehlendes Paket sowie manuelle Auswahl vor Abschluss einer asynchronen Erkennung. [Browserangaben und ihre Verfügbarkeit](https://developer.mozilla.org/en-US/docs/Web/API/NavigatorUAData/getHighEntropyValues).

Signaturen sichern die Herkunft; SHA-256 prüft zusätzlich die Integrität. Private Signierschlüssel bleiben ausserhalb des Website-Hosts und werden gesichert; die App enthält nur Verifikationsschlüssel. Zertifikat-/Schlüsselwechsel, Konfigurationsmigration und Wiederherstellung der bisherigen Version gehören zur Updateabnahme.

Manuelle Updateprüfung ist über das Menüleistensymbol erreichbar. Automatische Prüfungen sind eine gespeicherte Nutzerauswahl. Installation und Neustart werden verständlich angeboten; bei laufenden Timern oder geplanten Erinnerungen kann der Nutzer verschieben. Bestehende Einstellungen und KI-Zugangsdaten bleiben erhalten. Für später über Paketmanager installierte Linux-Pakete wird kein eigener Updater in Systemdateien geschrieben.

Aufwendige Installer-Builds, Signierung und Veröffentlichung künftig bei freigegebenen Release-Tags. PRs und main behalten die benötigten Entwicklungsprüfungen mit Cache und ohne Doppeltrigger; Website-Änderungen lösen keine Rust-Builds aus. Eine Veröffentlichung bleibt eine eigene bewusste Release-Aktion.

## KI und Ersteinrichtung

Ein gemeinsamer Provider-Vertrag liefert Textteile, Abschluss, Verbrauch und Fehler. Ollama, Claude und OpenAI laufen in eigenen Adaptern ausserhalb des Animationskerns. Die Oberfläche bleibt reaktionsfähig und unterstützt Abbruch. Während einer Anfrage zeigt Beni «Denken», bei Ausgabe «Reden» und gegebenenfalls eine geprüfte Abschlussgeste. Nur bekannte Animationen sind erlaubt; Pause und Bewegungsreduktion gelten weiterhin.

Persönliche Keys werden im macOS-Schlüsselbund, der Windows-Anmeldeinformationsverwaltung oder einem verfügbaren Linux-Secret-Service gespeichert. Fehlt ein sicherer Speicher, gibt es Sitzungsspeicherung statt Klartextdateien. Verbindung testen, Key ersetzen/löschen und Anbieter wechseln sind Teil der Oberfläche. Logs und Fehlertexte müssen Zugangsdaten aussparen.

Cloud-Nutzung und mögliche API-Kosten werden vor der Verbindung erklärt. Ein Testgespräch sendet nur den ausdrücklich eingegebenen Text und den angezeigten Chatkontext. Der vorhandene Hook-/Statusverlauf wird nicht automatisch an Anbieter übertragen. Datei-, Bildschirm-, Mikrofon- und Werkzeugzugriff gehören nicht zu diesem ersten Chat-Schritt.

Ollama-Installation und Modelle sind getrennte Komponenten. Vorhandene Installationen bevorzugen; Installation oder Modell-Download bewusst anbieten. Modellgrösse, Hardwarebedarf und jeweilige Modelllizenz vor einem Angebot prüfen. Ein Embedding-Modell für Gestenklassifikation ersetzt kein generatives Chatmodell.

Anbietertests in der CI verwenden simulierte Antworten. Echte Cloud-Anfragen und lokale Modelltests laufen gezielt und werden separat dokumentiert; keine bezahlten Modellaufrufe bei jedem Push.

## Bereits fertige Grundlage

Native Rust-Engine mit 480 SDK-Referenzposen, Avatar/Idle aus bestehenden Assets, Tray, separate Einstellungen, Fenster-Rückholung, durchklickbare Fläche, Accessibility, Grössenwahl, Hotkeys, reduzierte Bewegung, begrenzte lokale Status-IPC und CLI/Hook-Vorlagen, Nicht stören, Sitzungsverlauf, Fokus-/Pausen-Timer und lokale Erinnerungen. Letzte Implementierung: PR #20, main `bfc2ca8`.

Zum Zeitpunkt dieses Plans sind 111 lokale Rusttests (29 Root, 82 Desktop) und 11 Python-Integrationstests bestanden. Die letzte main-CI mit Release-Packaging ist auf macOS, Windows und Linux grün. Die native Abnahme liegt hauptsächlich für macOS vor; vollständige Windows-/Linux-, Standby- und Screenreader-Prüfungen bleiben eigene Abnahmen.

## Voraussetzungen für öffentliche Installer

- MIT-Abgrenzung und vorhandene Rechte an Beni beibehalten.
- Git-Verlauf, Actions-Logs und öffentliche Inhalte prüfen; keine Zugangsdaten veröffentlichen. Eine Musterprüfung ist keine Garantie für vollständige Geheimnisfreiheit.
- Lokale Developer-ID-Signieridentität und Team-Zuordnung sind durch die echte Apple-Notarisierung bestätigt; `beni-notary` funktioniert. Nutzerentscheidung: vorerst lokal signieren, private Ed25519-Schlüssel in 1Password. Die optional vorbereitete CI-Signierung benötigt weiterhin eigene Secrets, ist aber keine Voraussetzung dieses lokalen Ablaufs.
- Windows-Signierung und Linux-Paketbasis einrichten. Verfügbare Architekturen und Mindestversionen ausdrücklich benennen.
- Actions-Buildarchive sind Entwicklungsartefakte. Finale notarisierte DMGs für Intel und Apple Silicon sowie signierte Metadaten sind als Release 0.1.0 öffentlich verfügbar und nativ abgenommen. Update-Feeds und KI-Adapter folgen.

Diese Voraussetzungen verhindern den Beginn von R1 nicht. Öffentliche Installationspakete werden erst nach den jeweiligen Plattformabnahmen angeboten.

## Lokale Ablage

Das eigenständige Desktop-Repo liegt seit 13. Oktober 2026 unter `~/GitHub/beni-desktop-buddy`; der vorherige Unterordner in `~/Downloads/beni-scrollytelling` wurde vollständig verschoben, einschliesslich Git-Verlauf, Build-Caches und lokalen DMGs. Die GitHub-Verknüpfung bleibt unverändert. Die installierte App unter `/Applications/Beni Buddy.app` und die bestehenden Codex-Hooks unter `~/.codex/hooks/` sind davon unabhängig. Die vier Hook-Befehle bleiben unverändert, damit keine erneute Vertrauensprüfung durch eine geänderte Definition erforderlich wird. Die Codex-Sitzung wurde inzwischen direkt im neuen Repo wiederaufgenommen.

## Referenzen

[GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases), [Repository-Sichtbarkeit](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility), [Apple-Distribution](https://developer.apple.com/documentation/technologyoverviews/distribution), [Sparkle](https://sparkle-project.org/documentation/), [WinSparkle](https://winsparkle.org/guides/getting-started/), [AppImage-Updates](https://docs.appimage.org/packaging-guide/optional/updates.html).

[Ollama Chat](https://docs.ollama.com/api/chat), [Ollama FAQ](https://docs.ollama.com/faq), [Claude API](https://platform.claude.com/docs/en/api/overview), [OpenAI API](https://developers.openai.com/api/reference/overview).
