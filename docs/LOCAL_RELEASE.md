# Lokale macOS-Release-Pipeline

Nach [ADR 0003](adr/0003-local-release-signing.md) bleiben Apple-Zugang und Developer-ID-Schlüssel auf dem Entwickler-Mac. GitHub Actions baut Entwicklungsinstaller nativ und prüft dort die temporäre Installation, CLI und ein sichtbares eigenes Fenster. Der lokale Helfer verwendet diese bereits geprüften Executables und baut daraus frisch signierte/notarisierte Release-DMGs. Kein doppelter lokaler Rust-Build und keine Apple-Secrets in Actions nötig.

## Native Pakete vorbereiten

Der Installer-Workflow wird bewusst manuell gestartet. PRs, normale Pushes und Tags starten keine zusätzlichen Installer-Jobs. Ohne private Schlüssel sind die App und ihre Executables ad hoc signiert; das Entwicklungs-DMG selbst bleibt unsigniert. Produktions-DMGs müssen weiterhin Developer-ID-signiert und von Apple akzeptiert sein. [Apple beschreibt die Developer-ID-Signierung von Disk Images](https://developer.apple.com/library/archive/technotes/tn2206/).

```sh
gh workflow run macos-installers.yml --ref main -f architecture=both -f mode=development
gh run list --workflow macos-installers.yml
```

Für eine gezielte Wiederholung nur `aarch64` oder `x86_64` wählen. Der Workflow erzeugt pro Architektur ein sieben Tage gültiges Artefakt mit genau vier Dateien: Entwicklungs-DMG, SHA-256, Build-Beleg und `.native.json`-Abnahmebeleg. Dieser entsteht erst nach erfolgreichem Fensterstart und Entfernen der temporären Installation. Er hält Paket-Hash, Git-Commit und tatsächlich getestete macOS-Version fest. Fehlgeschlagene Läufe erzeugen keinen freigegebenen Abnahmebeleg.

## Signierschlüssel und Python-Umgebung

```sh
python3 -m venv .venv-release
.venv-release/bin/python -m pip install -r scripts/requirements-release.txt
.venv-release/bin/python scripts/local-release.py init-key
```

`init-key` erzeugt einmalig einen separaten Ed25519-Schlüssel. Der private 32-Byte-Schlüssel wird im Speicher erzeugt und als JSON über Standardeingabe an `op item create` übergeben, nicht als Kommandoargument und nicht über eine temporäre Datei. Er liegt danach im 1Password-Eintrag **Beni Buddy Release Signing (beni-update-v1)** im standardmässigen persönlichen Tresor. Für spätere Signierung liest der Helfer nur dessen Passwortfeld. 1Password muss entsperrt und die CLI verbunden sein; gegebenenfalls den Zugriff im 1Password-Dialog bestätigen. [1Password empfiehlt JSON-Vorlagen für sensible Werte](https://developer.1password.com/docs/cli/item-create/).

Nur der öffentliche Schlüssel liegt unter `release/keys/beni-update-v1.json`. `dist/release-signing.json` enthält lokal die nicht geheime Zuordnung zu seiner 1Password-Eintrags-ID. Der Helfer ersetzt weder vorhandene Schlüsseldateien noch diese Zuordnung. Den 1Password-Eintrag behalten und gemäss persönlicher Backup-Regel sichern. Kein privater Schlüssel wird im Repo, einem Installer oder einem CI-Artefakt abgelegt.

## Kandidaten erzeugen und prüfen

```sh
# RUN_ID durch den erfolgreichen nativen main-Lauf ersetzen.
BENI_NOTARY_PROFILE=beni-notary .venv-release/bin/python scripts/local-release.py prepare \
  --run RUN_ID --minimum-os 15.0 --output dist/release-candidate
.venv-release/bin/python scripts/local-release.py verify dist/release-candidate
```

Wenn die beiden Architekturen aus getrennten main-Läufen stammen, `--run` zweimal angeben. Standardmässig sind beide erforderlich. Ein bewusst einzelnes Ziel kann mit `--architecture aarch64` oder `--architecture x86_64` vorbereitet werden; ein fehlendes Ziel wird nicht stillschweigend ausgelassen.

Die angegebene Mindestversion ist eine Freigabeentscheidung, keine automatische Behauptung, dass das Buildziel macOS 11.0 manuell geprüft wurde. Der erste Kandidat verlangt macOS 15 oder neuer; die nativen Belege nennen zusätzlich den tatsächlich verwendeten 15.x-Runner. Eine exakte Abnahme von 15.0 und älteren Versionen auf eigenen Geräten bleibt separat zu dokumentieren.

Vor Signierung prüft der Helfer:

- eigener erfolgreicher Installer-Workflow auf main, native Architektur und bestandener Installations-/Fensterbeleg;
- Git-Commit in `origin/main` und identische Git-Produkteingaben, auch bei zwischenzeitlichen reinen Packaging-/Dokumentationsänderungen;
- genau die erwarteten regulären Archivdateien, GitHub-Archivdigest, Paketbytes, Build-Beleg und native Abnahme;
- lokale unveränderte Produkteingaben, übereinstimmende Version/Buildnummer und bereits vertrauenswürdigen öffentlichen Schlüssel;
- vorhandene GitHub Releases: keine wiederverwendete Version und global grössere Buildnummer. Vorhandene historische Manifeste müssen authentifiziert sein; unbekannte Schlüsselwechsel oder unprüfbare Alt-Releases werden abgelehnt. Der erste Helfer prüft höchstens 99 vorhandene Releases und verweigert einen unvollständigen Katalog.

Die Produkteingaben umfassen Cargo-Manifeste/-Lockfiles, Toolchain, App-/Crate-/Vendor-Quellen, Assets, Integrationsvorlagen und mitgelieferte Lizenz-/Integrationsdokumente. Es werden alle gewünschten nativen Eingaben vor dem ersten Apple-Upload geprüft. Nur die geprüften Executables werden übernommen; Icon, Vorlagen, Lizenzen und Bundle-Plist stammen erneut aus den eingecheckten lokalen Dateien.

Danach werden App und DMG mit Developer ID signiert, von Apple akzeptiert, gestapelt und mit Gatekeeper geprüft. Finale Bytes werden mit [Ed25519](https://cryptography.io/en/latest/hazmat/primitives/asymmetric/ed25519/) signiert. Das lokale Werkzeug begrenzt jedes Paket auf 128 MiB. Die finalen Dateien entstehen erst nach vollständiger Prüfung; vorhandene Kandidaten werden nicht überschrieben und eine fehlgeschlagene Übernahme wird zurückgerollt.

Der Kandidat enthält pro Architektur das DMG, `.sha256` und `.sig`, dazu `release.json`, `release.json.sig` sowie den signierten lokalen Prüfbeleg `candidate.json`/`.sig`. Signaturen sind Base64 über die exakten Datei-Bytes. Der Prüfer authentifiziert das Manifest, bevor er Version, URLs oder Zielpakete interpretiert, und prüft dann alle Paketsignaturen, Grössen und Hashes. Als Vertrauensanker dient ausschliesslich der lokale öffentliche Schlüssel; kein Download darf einen neuen Schlüssel setzen.

## Bewusste Veröffentlichung

`prepare` und `verify` erstellen keine Git-Tags, GitHub Releases oder Latest-Verweise. Die im Kandidaten enthaltenen URLs sind vorgesehene versionsgebundene Adressen, noch keine verfügbaren Downloads. Der nächste Veröffentlichungsschritt ist ein eigener Review: Mindestanforderungen, Release-Notizen, echte Zielmenge und Quellenstand bestätigen, vollständige Pakete/Signaturen/Metadaten in einen zunächst nicht öffentlichen Release hochladen und erst nach Prüfung gemeinsam freigeben. `candidate.json` ist ein lokaler Prüfbeleg und wird nicht als Website-/Update-Feed angeboten.

Die Website W1 folgt erst mit diesem echten Download. Rust-Verifikation, Sparkle-Appcast und Updateinstallation gehören zu R3. Der öffentliche Schlüssel ist jetzt vorbereitet, aber noch nicht als laufender Desktop-Updater eingebunden. Ein späterer Schlüsselwechsel benötigt eigene Übergangsregeln; der aktuelle Helfer vertraut nicht automatisch einem anderen Schlüssel. Die optionale CI-Signierung bleibt vorbereitet; ein manueller Release-CI-Lauf prüft fehlende Secret-Formate vor teuren Builds.

## Geprüfter Kandidat vom 14. Oktober 2026

`dist/release-candidate-0.1.0` enthält den echten gemeinsamen Kandidaten 0.1.0/Build 1 aus dem nativen main-Lauf `38472531951`. Beide Apps und DMGs sind von Apple akzeptiert, Developer-ID-signiert, mit Tickets versehen und durch Gatekeeper sowie erneute temporäre Kopierinstallation geprüft. Alle zehn lokalen Dateien sind authentifiziert. Die finale ARM-CLI wurde nativ ausgeführt; die finale Intel-CLI und finale GUI beider Pakete bleiben separat offen. Der ältere notarisierte ARM-Installer unter `dist/macos/` und die installierte Programme-App bleiben unverändert.

Der [Release-Entwurf](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/untagged-24e439a84876b5dba873) enthält acht Dateien: beide DMGs jeweils mit `.sha256` und `.sig`, dazu `release.json` und `.sig`. Alle Assets wurden zurückgeladen und bytegleich sowie kryptografisch geprüft. `candidate.json` und seine Signatur bleiben ausschliesslich lokal. Der Entwurf ist nicht öffentlich freigegeben und reserviert `v0.1.0`; eine erneute `prepare`-Ausführung für dieselbe Version wird darum bewusst abgelehnt. Zum Fortsetzen den vorhandenen Kandidaten prüfen und den geprüften Entwurf verwenden, keine neuen Schlüssel erzeugen und keine vorhandenen Pakete überschreiben.

## Finale native Abnahme ohne Neubau

Der ausschliesslich manuelle Workflow **macOS release acceptance** prüft einen vorhandenen Release oder Entwurf auf `macos-15` und `macos-15-intel`. Er lädt Manifest und Paketdateien mit dem nur lesenden GitHub-Token, authentifiziert das Manifest vor dessen Auswertung mit dem eingecheckten öffentlichen Schlüssel und prüft Paketbytes, Signatur und Prüfsumme. `download-release` verlangt die eingecheckte Version, vollständige Release-Dateien, hochgeladene Assets und passende GitHub-Digests; vorhandene Zielordner werden nicht überschrieben. Der für den bestehenden Installer-Prüfer erzeugte Build-Beleg ist aus dem authentifizierten Manifest abgeleitet und kein nativer Abnahmebeleg.

```sh
gh workflow run macos-release-acceptance.yml --ref main -f tag=v0.1.0
```

Danach prüft der bestehende native Installer-Helfer die tatsächlichen Developer-ID-Signaturen, Runtime, Zeitstempel, Tickets, Gatekeeper, temporäre Installation, native CLI und ein eigenes sichtbares Fenster. Nur nach Erfolg und Entfernen der Testinstanz entsteht der `.native.json`-Abnahmebeleg mit finalem Paket-Hash und getesteter OS-Version. Beide Belege werden 30 Tage aufbewahrt. Kein Rust-Build, keine Signierung und keine Apple-Secrets; nur zwei gezielt gestartete Abnahmejobs. PRs, Pushes und Tags lösen diesen Workflow nicht aus. Die lokale Beni-Installation bleibt unabhängig davon.
