# macOS-Installer vorbereiten und freigeben

R2 baut ein schreibgeschütztes DMG mit Beni-App, bestehendem Icon, «Programme»-Link und kurzer Installationshilfe. Die App enthält CLI, eingecheckte Integrationsvorlagen und die eigenen Lizenztexte. Lokale Hook-Konfigurationen und nicht eingecheckte Dateien werden nicht übernommen. Vor dem Ersetzen einer vorhandenen App Beni beenden; aktive Timer und Erinnerungen enden dabei. Nutzer ziehen die App nach Programme und öffnen sie dort, danach wird das DMG ausgeworfen. Zur Deinstallation Beni beenden und das Bundle löschen; die bisherigen Einstellungen bleiben erhalten.

Apple Silicon (`aarch64`) und Intel (`x86_64`) erhalten getrennte Pakete. Native Builds verwenden vorhandene Cargo-Caches. Cross-Builds benötigen ein zuvor bewusst installiertes Rust-Ziel. `lipo` prüft beide Executables, `otool` deren Mindestversion. Buildziel und Bundleangabe sind macOS 11.0; eine reale Abnahme auf diesem alten System bleibt offen.

## Drei eindeutig getrennte Modi

| Modus | Signierung und Abnahme | Dateiname |
| --- | --- | --- |
| `development` | Ad hoc, kein Apple-Upload | `beni-buddy-{version}-macos-{arch}-development.dmg` |
| `signed-test` | Developer ID, sichere Zeitstempel, Hardened Runtime, keine Notarisierung | `beni-buddy-{version}-macos-{arch}-unnotarized.dmg` |
| `release` | Developer ID, Hardened Runtime, App und DMG akzeptiert/notarisiert, Tickets angehängt und Gatekeeper geprüft | `beni-buddy-{version}-macos-{arch}.dmg` |

Der Installer-Befehl verwendet standardmässig `release`; ohne Notarisierungsprofil bricht er vor dem Build ab. Es gibt keinen stillen Rückfall auf ein Testpaket. Das bisherige `./scripts/package-macos.sh` bleibt ein ad-hoc-signiertes Entwicklungsbundle ohne Apple-Upload. Beide Buildwege beziehen Produktversion und Buildnummer aus Cargo.

```sh
python3 scripts/package-macos.py installer --mode development --architecture aarch64
python3 scripts/package-macos.py installer --mode signed-test --architecture aarch64
python3 scripts/check-macos-installer.py dist/macos/beni-buddy-0.1.0-macos-aarch64-unnotarized.dmg --architecture aarch64 --mode signed-test
```

Eine gültige Developer-ID-Application-Identität wird bei genau einem Treffer automatisch ausgewählt. Bei mehreren Identitäten `--identity` oder `BENI_SIGNING_IDENTITY` setzen. Es werden keine neuen Zertifikate erstellt und keine privaten Schlüssel vom lokalen Mac exportiert.

Der Build signiert Executables vor dem Bundle, verwendet in signierten Modi Hardened Runtime und sichere Zeitstempel und fügt keine zusätzlichen Runtime-Entitlements hinzu. Nach der Signierung werden Signaturen streng geprüft. Ein Release notarisiert zunächst die App, hängt ihr Ticket an und prüft es mit Gatekeeper. Danach folgen DMG, dessen Signierung, Notarisierung, Ticket und Gatekeeper-Abnahme. Nur nach vollständig erfolgreicher Prüfung werden Dateien in `dist/macos/` übernommen; existierende Dateinamen werden nicht überschrieben.

Neben dem DMG entstehen eine `.sha256`-Datei und ein `.build.json`-Beleg mit tatsächlich berechneter Grösse/Prüfsumme, Architektur, Version und Signier-/Notarisierungsstatus. Der Beleg ist **kein `release.json` und kein Update-Feed**. Die Ed25519-Updatesignatur und authentifizierte Feedableitung bleiben vor der ersten öffentlichen Veröffentlichung gemäss [Release-Vertrag](RELEASES.md) einzurichten. Die CI lädt vorerst ausschliesslich befristete Actions-Artefakte hoch; sie erstellt keinen öffentlichen GitHub Release.

## notarytool und einmalige Einrichtung

`notarytool` gehört zu Apples Entwicklungswerkzeugen. Es reicht die bereits signierte Software zur automatischen Prüfung bei Apple ein. Bei Erfolg erzeugt Apple ein Ticket; `stapler` hängt dieses an App/DMG, damit die Bestätigung auch ohne Netzverbindung mitgeliefert wird. [Apple erklärt den Ablauf hier](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).

Die lokale Developer-ID-Identität und das Schlüsselbundprofil `beni-notary` sind eingerichtet. App und Apple-Silicon-DMG wurden am 13. Oktober 2026 im echten Release-Modus von Apple akzeptiert. Auf diesem Mac muss das Profil nicht erneut eingerichtet werden. Für einen weiteren Build-Mac erfolgt die einmalige Einrichtung direkt im Terminal:

```sh
xcrun notarytool store-credentials beni-notary
```

Das Werkzeug fragt interaktiv nach dem Zugang. Für den Apple-ID-Weg benötigt man die zum Developer-Team gehörende Apple-ID, dessen Team-ID und ein app-spezifisches Passwort. Alternativ kann ein passender App-Store-Connect-Team-API-Key verwendet werden. Die Zugangsdaten bleiben im lokalen Schlüsselbund; der Profilname `beni-notary` ist nur die Bezeichnung dieses Eintrags. Passwörter und private Keys gehören nicht in Chat, Git oder Dokumentation. [Apple beschreibt eigene Workflows und Schlüsselbund-Zugang](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow).

Danach wird die echte Prüfung mit dem vorbereiteten Befehl ausgeführt:

```sh
BENI_NOTARY_PROFILE=beni-notary python3 scripts/package-macos.py installer --architecture aarch64
```

Ein Apple-Upload kann mehrere Minuten dauern. Die Wartezeit ist begrenzt; ein Timeout oder ein nicht akzeptiertes Paket wird nicht als freigegeben ausgegeben. Kein Gatekeeper-Schalter wird verändert.

## Gezielte CI für beide Architekturen

Der Workflow **macOS installers** läuft bei einem bewusst gesetzten `v*`-Tag oder manuell. Normale Branch-Pushes und PRs starten ihn nicht. Ein manueller Lauf baut standardmässig nur ein Apple-Silicon-Entwicklungspaket; Intel oder beide Architekturen sind wählbar. Die [nativen Runner](https://github.com/actions/runner-images) sind `macos-15` und `macos-15-intel`, mit Cache und 60 Minuten Zeitlimit. Die normalen drei PR-Prüfungen enthalten nur die schnellen, simulierten Packaging-Tests.

Der Release-Modus benötigt einen zur Cargo-Version passenden v-Tag auf einem bereits in main enthaltenen Commit und fünf zuvor eingerichtete Actions-Secrets:

- `BENI_MACOS_CERTIFICATE_P12`: Base64 des bewusst exportierten Developer-ID-Zertifikats samt privatem Schlüssel.
- `BENI_MACOS_CERTIFICATE_PASSWORD`: Exportpasswort.
- `BENI_APPLE_API_KEY`: Inhalt des passenden Team-API-Keys (`.p8`).
- `BENI_APPLE_API_KEY_ID`: Key-ID.
- `BENI_APPLE_API_ISSUER`: Issuer-ID.

Diese Secrets sind derzeit nicht eingerichtet. `macos-ci-signing.py` verweigert Änderungen am lokalen Nutzer-Schlüsselbund und läuft nur im Actions-Kontext. Auf dem Runner legt es private temporäre Dateien und einen eigenen Schlüsselbund an, importiert die Identität und validiert/speichert den Notarisierungszugang. Anschliessend werden Suchliste und Schlüsselbund auch bei Fehlern aufgeräumt. Rohes Signier-/Notary-Tooloutput wird nicht in CI-Logs ausgegeben. Secrets werden nicht in Build-Belege oder Installer kopiert.

Der native Abnahmeschritt prüft Paketbytes, Signaturen, Inhalt, temporäre Installation, Bundle-Version und CLI-Start. Nur auf dem frischen Runner startet `--launch` die kopierte App und bestätigt ein eigenes sichtbares Fenster anhand ihrer Prozess-ID; keine Screenshots oder Accessibility-Freigaben. Danach beendet er nur diese Testinstanz, wirft das Testvolume aus und entfernt den Testordner. Der lokale Check startet keine zweite Beni-Instanz und verändert keine vorhandene Installation oder Benutzereinstellungen.

## Aktueller Abnahmestand

PASS: 23 portable Packaging-/Credential-/Receipt-Tests; lokale Root-/Desktop-Prüfungen und drei grüne PR-Plattformjobs; Apple-Silicon-DMG mit Developer-ID-Signaturen, Zeitstempeln und Hardened Runtime. Echte Apple-Notarisierung von App und DMG, angehängte/validierte Tickets und Gatekeeper-Prüfung. Erneute schreibgeschützte DMG-Abnahme mit temporärem Kopieren, Version-/Signatur-/Ticket-/Gatekeeper-Prüfung, CLI-Start und Entfernen. Der Nutzer hat die alte App beendet, die neue App aus dem DMG nach Programme kopiert und ihren sichtbaren Start bestätigt. Laufender Prozess unter `/Applications/Beni Buddy.app`, Version 0.1.0/Build 1, gültige Signatur, Ticket und Gatekeeper-Zulassung geprüft. Kompatibles Ad-hoc-Entwicklungsbundle und kompilierter Swift-Fensterprüfer bestanden.

NOT_RUN: neue Installer-CI, nativer Intel-Installer-/Fenstertest, vollständige Bedienabnahme auf einem frischen Gerät und altes macOS 11.0. CI-Secrets sind nicht eingerichtet. Das lokale notarisierte DMG ist noch kein öffentlicher Download. Diese Schritte müssen vor dem Anschluss von W1 abgeschlossen beziehungsweise die angebotenen Plattformanforderungen entsprechend eingegrenzt werden. Authentifizierte Release-/Update-Metadaten sind ebenfalls noch offen.
