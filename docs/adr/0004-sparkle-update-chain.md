# ADR 0004 · Sparkle und authentifizierte Update-Feeds

Status: angenommen am 14. Oktober 2026. R3 beginnt auf Nutzerwunsch während der ausstehenden Microsoft-Firmenprüfung.

## Entscheidung

macOS behält Rust und egui/eframe. Der Plattformadapter verwendet Sparkles native Updateoberfläche und Installer. Der erste Integrationsschritt prüft das Signaturformat und erzeugt lokale Appcasts; Framework-Einbettung, Tray-Aktion und Veröffentlichung folgen separat.

Der geprüfte SDK-Stand ist Sparkle 2.10.0. URL, Dateigrösse und SHA-256 liegen in `release/sparkle-sdk.json`. Das SDK wird noch nicht mit Beni ausgeliefert. Die native Probe entpackt ausschliesslich das gepinnte Archiv und verwendet einen kurzlebigen Testschlüssel; kein Zugang zum Produktionsschlüssel ist nötig.

Appcasts werden ausschliesslich aus einem vollständig authentifizierten Release-Kandidaten abgeleitet. Der bestehende Prüfer authentifiziert die Manifestbytes vor dem Interpretieren und prüft Paketbytes, Schlüsselzuordnung und signierte Herkunftsbelege. Buildnummer, Version, Paketadresse, Grösse und Mindestanforderungen werden daraus übernommen. Jeder Mac-Architektur und jedem Kanal gehört eine eigene Feeddatei. Ein fehlendes Ziel wird nicht durch eine andere Architektur ersetzt.

Die bestehende Ed25519-Signatur über die finalen DMG-Bytes ist mit Sparkles `edSignature` kompatibel. Der gemeinsame Release-Schlüssel bleibt erhalten. Zusätzlich wird der exakte XML-Bytestrom signiert; Sparkles eingebetteter Signaturblock folgt dem XML. Nachträgliche XML-Formatierung ist verboten. Die lokale Prüfung authentifiziert die Bytes und vergleicht sie mit der erwarteten Ableitung aus dem Kandidaten. Externe HTML-Release-Notes sind im ersten Feed nicht eingebunden.

Der private Produktionsschlüssel bleibt in 1Password und gelangt nur in den Speicher des bereits vorhandenen Signierhelfers. Die Prüfung ungültiger Eingaben und belegter Ausgabeordner erfolgt vor dem Schlüsselzugriff. Appcasts entstehen in einem separaten neuen Ordner; Kandidaten und veröffentlichte Releases werden nicht verändert.

## Nächste Integrationsschritte

1. Sparkle mit seinen Hilfsprogrammen und Lizenzen ins Bundle einbetten, von innen nach aussen mit derselben Developer ID signieren und notarisieren. Den Adapter ausschliesslich auf dem Hauptthread betreiben; ungepackte Entwicklungsstarts bleiben möglich.
2. «Nach Updates suchen …» im Menüleistensymbol anbinden. Automatische Prüfungen sind eine Nutzerauswahl und anfangs ausgeschaltet; Sparkles gespeicherte Auswahl nicht bei jedem Start zurücksetzen. Installation und Neustart bleiben bewusst. Laufende Timer und Erinnerungen berücksichtigen.
3. `SUPublicEDKey` fest einbinden, `SURequireSignedFeed` und `SUVerifyUpdateBeforeExtraction` aktivieren. `SUSignedFeedFailureExpirationInterval=0` verhindert den zeitgesteuerten Fallback bei ungültigen Feed-Signaturen. Systemprofiling, JavaScript und automatische Installation bleiben ausgeschaltet.
4. Feeddateien und die strenge Dateimengenprüfung von Release-Pipeline und Website gemeinsam erweitern. Erst danach eine neue unveränderliche Veröffentlichung und einen Stable-Verweis freigeben. Beta bekommt einen getrennten Verweis; GitHubs Latest-Link darf keinen Beta-Feed bestimmen.
5. Zwei updaterfähige Versionen A → B tatsächlich installieren und Neustart, Einstellungen, defekte Signatur, Downloadabbruch sowie einen nicht schreibbaren Installationsort prüfen. Die heutige Version 0.1.1 hat noch keinen Updater und benötigt einmalig die manuelle Installation des ersten updaterfähigen Bundles.

## Abnahmen und Grenzen

Offline-Regressionsprüfungen laufen einmal im Linux-CI-Job. Die native Sparkle-Probe läuft bewusst lokal mit dem gepinnten SDK; sie startet keine App, lädt kein Update, ändert keine Hooks und benötigt keinen Rust-/Installer-Neubau. Ein erfolgreicher Signaturtest bestätigt noch keinen funktionierenden Desktop-Updater. Windows- und Linux-Updateadapter bleiben eigene Schritte.

## Referenzen

[Sparkle 2.10.0](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0), [programmgesteuerte Anbindung](https://sparkle-project.org/documentation/programmatic-setup/), [Sicherheit und Einstellungen](https://sparkle-project.org/documentation/customization/), [natives Feed-Signaturformat](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Signing.swift), [Ed25519-Schlüsselimport](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Secret.swift).
