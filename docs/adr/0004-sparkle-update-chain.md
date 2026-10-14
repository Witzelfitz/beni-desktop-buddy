# ADR 0004 · Sparkle und authentifizierte Update-Feeds

Status: angenommen am 14. Oktober 2026. R3 beginnt auf Nutzerwunsch während der ausstehenden Microsoft-Firmenprüfung.

## Entscheidung

macOS behält Rust und egui/eframe. Der Plattformadapter verwendet Sparkles native Updateoberfläche und Installer. R3a prüft das Signaturformat und erzeugt lokale Appcasts; R3b ergänzt Framework-Einbettung und Tray-Aktion. Feed-Veröffentlichung und Update A → B folgen separat.

Der geprüfte SDK-Stand ist Sparkle 2.10.0. URL, Dateigrösse und SHA-256 liegen in `release/sparkle-sdk.json`. Neue Mac-Bundles enthalten das SDK mit seinen Symlinks, Hilfsprogrammen und der Lizenz. Eine kleine Objective-C-Brücke wird beim Mac-Cargo-Build statisch eingebunden und lädt ausschliesslich das Framework im eigenen App-Bundle dynamisch. Das SDK wird beim normalen Rust-Build nicht benötigt. Der Rust-Adapter ist an den Hauptthread gebunden; UI-/Netzwerk-/Installationscode gelangt nicht in den Animationskern.

Der öffentliche Feed bleibt über eine eingecheckte, signierte Bundle-Einstellung ausgeschaltet, bis beide Repositories dessen Veröffentlichung unterstützen. Ein deaktivierter Feed führt zur verständlichen Erklärung statt zu einem HTTP-Fehler. Die Bundle-Mindestversion steigt auf die bereits freigegebene Produktanforderung 15.0; die nativen Rust-/Bridge-Binaries behalten ihr Buildziel 11.0. Sparkles fertiges SDK benötigt 12.0 und darf keine niedrigere Bundle-Mindestanforderung erhalten.

Appcasts werden ausschliesslich aus einem vollständig authentifizierten Release-Kandidaten abgeleitet. Der bestehende Prüfer authentifiziert die Manifestbytes vor dem Interpretieren und prüft Paketbytes, Schlüsselzuordnung und signierte Herkunftsbelege. Buildnummer, Version, Paketadresse, Grösse und Mindestanforderungen werden daraus übernommen. Jeder Mac-Architektur und jedem Kanal gehört eine eigene Feeddatei. Ein fehlendes Ziel wird nicht durch eine andere Architektur ersetzt.

Die bestehende Ed25519-Signatur über die finalen DMG-Bytes ist mit Sparkles `edSignature` kompatibel. Der gemeinsame Release-Schlüssel bleibt erhalten. Zusätzlich wird der exakte XML-Bytestrom signiert; Sparkles eingebetteter Signaturblock folgt dem XML. Nachträgliche XML-Formatierung ist verboten. Die lokale Prüfung authentifiziert die Bytes und vergleicht sie mit der erwarteten Ableitung aus dem Kandidaten. Externe HTML-Release-Notes sind im ersten Feed nicht eingebunden.

Der private Produktionsschlüssel bleibt in 1Password und gelangt nur in den Speicher des bereits vorhandenen Signierhelfers. Die Prüfung ungültiger Eingaben und belegter Ausgabeordner erfolgt vor dem Schlüsselzugriff. Die separate R3a-Probe erzeugt Appcasts in einem neuen Ordner. Ab R3c erzeugt die reguläre Kandidatenvorbereitung Appcasts vor der Manifestsignatur im neuen Kandidaten, ohne zweiten Schlüsselzugriff. Manifest v2 bindet die exakten Feed-Bytes; v1 bleibt für historische Releases unverändert. `export` prüft kopierte Kandidatenbytes und gibt nur öffentliche Dateien frei. Bestehende Kandidaten und veröffentlichte Releases werden nicht verändert.

## Nächste Integrationsschritte

1. R3b implementiert: Sparkle mit Hilfsprogrammen und Lizenz einbetten, von innen nach aussen mit derselben Developer ID signieren. Native Brücke und separate Rust-QA-App samt Notarisierung auf ARM macOS 26.6.2 geprüft; Intel/macOS 15 separat offen. Adapter ausschliesslich auf dem Hauptthread; ungepackte Entwicklungsstarts bleiben möglich.
2. R3b implementiert: Tray «Nach Updates suchen …», anfangs ausgeschaltete automatische Prüfwahl, Einstellungen vor Neustart speichern, IPC kontrolliert schliessen und Timer-/Erinnerungsabfrage. Native Callback-/Rust-UI-Prüfung bestanden; echter Update-Neustart bleibt Schritt 5.
3. R3b konfiguriert: `SUPublicEDKey`, `SURequireSignedFeed`, `SUVerifyUpdateBeforeExtraction` und `SUSignedFeedFailureExpirationInterval=0`. Systemprofiling, JavaScript und automatische Installation ausgeschaltet. Öffentliche Feed-Freigabe bleibt bis Schritt 4 deaktiviert.
4. R3c vorbereitet: Feeddateien und strenge Dateimengenprüfung von Release-Pipeline und Website gemeinsam erweitert. Die Website authentifiziert beide Ziel-Feeds, bevor Downloads oder Updates verfügbar werden; beide nutzen denselben gecachten Stable-Release. Separate Beta-Routen bleiben ohne expliziten Kanalverweis 503. Website-Deployment zuerst. Erst danach eine neue unveränderliche Veröffentlichung und einen Stable-Verweis freigeben. Beta bekommt einen getrennten Verweis; GitHubs Latest-Link darf keinen Beta-Feed bestimmen.
5. Zwei updaterfähige Versionen A → B tatsächlich installieren und Neustart, Einstellungen, defekte Signatur, Downloadabbruch sowie einen nicht schreibbaren Installationsort prüfen. Die heutige Version 0.1.1 hat noch keinen Updater und benötigt einmalig die manuelle Installation des ersten updaterfähigen Bundles.

## Abnahmen und Grenzen

Offline-Appcast-Prüfungen laufen einmal im Linux-CI-Job. Die native Signaturprobe verwendet einen kurzlebigen Testschlüssel, ohne Produktionsschlüsselzugriff. Die zusätzliche Brückenprobe startet ausschliesslich ein temporäres signiertes Testbundle und einen lokalen Fixture-Server, prüft gespeicherte Auswahl und Fehlerdialog und entfernt eigene Test-Präferenzen. Eine separate echte Rust-GUI-Abnahme verwendet einen eigenen Datenordner. Beide lassen Installation und Hooks bestehen und benötigen keine neuen Installer für alle Plattformen. Die tatsächliche Installation A → B ist weiterhin eine eigene Abnahme. Windows- und Linux-Updateadapter bleiben eigene Schritte.

## Referenzen

[Sparkle 2.10.0](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0), [programmgesteuerte Anbindung](https://sparkle-project.org/documentation/programmatic-setup/), [Sicherheit und Einstellungen](https://sparkle-project.org/documentation/customization/), [natives Feed-Signaturformat](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Signing.swift), [Ed25519-Schlüsselimport](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Secret.swift).
