# ADR 0003 · Native CI-Builds, lokale Release-Signierung

Status: angenommen am 13. Oktober 2026, nach Nutzerwahl «Vorerst lokal signieren».

## Entscheidung

GitHub Actions baut und prüft macOS-Entwicklungsinstaller nativ pro ausgewählter Architektur. Developer-ID-Schlüssel und Apple-Notarisierungsprofil bleiben auf dem Entwickler-Mac. Ein lokaler Release-Helfer übernimmt ausschliesslich erfolgreich geprüfte Artefakte des eigenen Installer-Workflows auf main, vergleicht die Produkteingaben mit dem lokalen Git-Stand und signiert/notarisiert daraus neue Release-DMGs. Dafür werden nur die bereits geprüften Executables wiederverwendet; Ressourcen und Bundle werden aus den lokalen eingecheckten Dateien neu aufgebaut.

Ein separater Ed25519-Schlüssel signiert die exakten finalen Paketbytes und `release.json`. Sein privater Teil liegt in einem eigenen 1Password-Eintrag; der öffentliche Teil liegt im Repository. Das Manifest bleibt v1; seine zusätzliche Signatur heisst `release.json.sig`. Die Schlüsselkennung im Manifest ist kein neuer Vertrauensanker. Der Prüfhelfer verwendet ausschliesslich den bereits lokal vertrauenswürdigen öffentlichen Schlüssel.

Der erste Ablauf bereitet einen lokalen, vollständig geprüften Release-Kandidaten vor. Er erzeugt keine öffentlichen Tags, Releases oder Latest-Verweise. Die bewusste Veröffentlichung folgt nach Review und der Mindestversions-Abnahme. Sparkle und sein Appcast sowie die Einbindung der Verifikationsschlüssel in die Rust-App bleiben R3; diese Pipeline implementiert noch keinen Desktop-Updater.

## Abnahmen und Grenzen

- Nur erfolgreich abgeschlossene native Jobs und unverfallene Artefakte aus dem eigenen Workflow; fremde Repositories, PR-/Feature-Branch-Builds und abweichende Produkteingaben werden abgelehnt.
- Archivpfade, Grössen, Hashes, Build-Belege, Architektur und Bundle-Version vor dem Signieren prüfen. Keine ausführbaren Inhalte aus fremden Artefakten starten.
- Kein zweiter lokaler Rust-Build für bereits nativ geprüfte Executables. Ressourcen, Version und Rechte werden beim lokalen Bundlebau erneut geprüft.
- Beide gewünschten Architekturen müssen vollständig vorliegen; kein stilles Entfernen eines fehlgeschlagenen Ziels.
- Neue Pakete entstehen erst nach Apple-Annahme, Tickets und Gatekeeper-Prüfung. Finale Grösse/Prüfsumme nach Stapling ermitteln, danach erst Ed25519 signieren.
- Kandidatenverzeichnis erst nach vollständiger Prüfung übernehmen; keine bestehenden Pakete überschreiben. Auch Manifest, Signaturen und fremde Schlüssel müssen negative Prüfungen bestehen.
- Geheimnisse nur im Speicher und über private Standardeingabe an 1Password; keine Secrets in Argumenten, temporären Dateien, Git oder Logs.
- macOS 11.0 ist das Buildziel. Veröffentlichte Mindestanforderungen werden separat anhand der tatsächlich geprüften Systeme festgelegt.
- Die vorbereitete CI-Signierung mit Actions-Secrets bleibt optional für später; keine Zertifikatexporte oder Apple-Secrets für den lokalen Ablauf nötig.

## Referenzen

[GitHub-Artefakte](https://docs.github.com/en/rest/actions/artifacts), [Ed25519-API](https://cryptography.io/en/latest/hazmat/primitives/asymmetric/ed25519/), [sensible Werte über 1Password-JSON-Vorlagen](https://developer.1password.com/docs/cli/item-create/).
