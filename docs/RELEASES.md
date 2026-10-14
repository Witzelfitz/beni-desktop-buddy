# Release-Vertrag und Versionierung

R1 stellt ein gemeinsames Datenformat und Offline-Prüfwerkzeuge für die spätere Downloadseite und Updateadapter bereit. Das Format ist [JSON Schema Draft 2020-12](https://json-schema.org/draft/2020-12/json-schema-validation). Es veröffentlicht keine Downloads und implementiert noch keinen Updater. [Die Beispiele](../release/examples/README.md) sind ausdrücklich unveröffentlichte Testdaten.

## Verbindliche Versionsquelle

`Cargo.toml` im Root-Workspace enthält die Produktversion unter `workspace.package.version` und die Buildnummer unter `workspace.metadata.beni-release.build-number`. Kern und CLI erben die Version. Die eigenständigen Desktop-/IPC-Manifeste und beide Cargo-Lockfiles müssen denselben Stand tragen; `scripts/release.py check-versions` prüft dies auf jeder CI-Plattform.

Das Produkt verwendet eine begrenzte [SemVer-Regel](https://semver.org/): Stable `X.Y.Z`, Beta `X.Y.Z-beta.N` mit positivem N. Keine führenden Nullen, kein Build-Metadaten-Suffix und vorerst keine weiteren Kanäle. Git-Tags heissen genau `v` plus Produktversion. Vor 1.0 sind Minor-Versionen für inkompatible Änderungen erlaubt; Patch-Versionen enthalten kompatible Korrekturen. Schema-Version und Produktversion sind getrennt.

Jeder veröffentlichte Build bekommt eine neue, weltweit über Stable/Beta steigende Buildnummer. Nummern und Release-Tags werden nicht wiederverwendet; Dateien einer veröffentlichten Version bleiben unverändert. Die Release-Pipeline muss vor einer Veröffentlichung gegen die vorhandenen Releases prüfen, dass die Nummer neu und grösser ist. R1 prüft lokal gültige Werte und Übereinstimmung, noch keinen entfernten Veröffentlichungskatalog.

Das macOS-Packaging übernimmt die numerische Produktversion in `CFBundleShortVersionString` und die Buildnummer in `CFBundleVersion`, bevor es signiert. Bei `0.2.0-beta.1`, Build 7, werden daraus `0.2.0` und `7`; der Beta-Kanal bleibt im Release-Manifest erhalten. Buildnummern sind vorerst auf 1–9999 begrenzt, passend zum verwendeten einteiligen [Apple-Buildformat](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html). Das bisherige Entwicklungsbundle bleibt bei `0.1.0`, Build 1.

Eine Versionsänderung umfasst Root-, Desktop- und IPC-Manifeste sowie beide mit Cargo aktualisierten Lockfiles. Den Versionscheck zuerst ausführen; im Tag-Workflow zusätzlich `--tag "$GITHUB_REF_NAME"` verwenden. In R2 wird die Freigabe-/Tag-Pipeline ergänzt; die bestehende CI produziert weiter Entwicklungsarchive.

## Manifest v1 und v2

Die Schemas [v1](../release/schema-v1.json) und [v2](../release/schema-v2.json) beschreiben die Struktur. `scripts/release.py validate` prüft zusätzlich Beziehungen zwischen Version, Kanal, URLs und Zielplattformen. Das Schema allein ersetzt diese semantische Prüfung nicht. Unbekannte Felder werden abgelehnt; inkompatible Erweiterungen erhalten eine neue `schema_version`.

| Feld | Bedeutung |
| --- | --- |
| `schema_version` | 1 für bestehende Releases, 2 für Releases mit Mac-Appcasts; unbekannte Versionen ablehnen. |
| `product` | Genau `io.witzelfitz.beni-buddy`. |
| `version`, `build_number` | Produktversion und global steigende Buildnummer. |
| `channel` | `stable` oder `beta`, passend zur Version. Stable darf Beta nicht anzeigen. |
| `release_notes_url` | GitHub-Release-Seite genau derselben Version. |
| `artifacts` | 1–6 tatsächlich verfügbare, einzigartige Plattform-/Architekturpaare. Fehlende Ziele gelten als nicht verfügbar. |
| `platform`, `architecture` | `macos`/`windows`/`linux` und `aarch64`/`x86_64`; keine automatische Wahl einer inkompatiblen Datei. |
| `format` | macOS `dmg`, Windows `exe`, Linux `appimage`. Weitere Paketformate bekommen später einen eigenen Vertrag. |
| `requirements` | Lesbare Anforderungen; macOS/Windows zusätzlich `minimum_os_version`, Linux `minimum_glibc_version`. Angaben erst nach echter Plattformabnahme veröffentlichen. |
| `url` | Versionsgebundene HTTPS-Adresse im öffentlichen Desktop-Repo. |
| `size_bytes`, `sha256` | Tatsächliche Dateigrösse, 1 Byte bis 4 GiB, und SHA-256 als 64 kleine Hexzeichen. |
| `signature` | `algorithm: ed25519`, Schlüsselkennung `beni-update-vN` und Adresse der zum Paket gehörenden `.sig`-Datei. |

Dateinamen: `beni-buddy-{version}-{platform}-{architecture}.{extension}`. Endungen sind `.dmg`, `.exe` beziehungsweise `.AppImage`. Paketadressen sind exakt `https://github.com/Witzelfitz/beni-desktop-buddy/releases/download/v{version}/{dateiname}`; Signaturen liegen daneben unter `{dateiname}.sig`. Der Validator lässt andere Hosts, Credentials, Queryparameter, Fragmente, Pfadtraversierung sowie andere Versionen oder Architekturen nicht zu. GitHub kann den tatsächlichen Download auf seinen Asset-Host weiterleiten; der künftige HTTP-Adapter muss seine Redirect-Regeln ausdrücklich prüfen.

Der Manifest-Reader begrenzt Eingaben auf 256 KiB und lehnt doppelte JSON-Felder ab. Beschreibungen enthalten keine Steuerzeichen. Fehlerausgaben geben keine gelieferten URL-Werte oder Tokens aus. Website und Updater müssen entsprechende Eingabegrenzen beibehalten.

### Authentifizierte Mac-Feeds in v2

V2 verlangt zusätzlich `macos_update_feeds`: pro vorhandenem Mac-Ziel genau ein Eintrag mit `architecture`, `url`, `size_bytes` (höchstens 256 KiB), `sha256` und `key_id`. Die Adresse endet exakt auf `appcast-macos-{architecture}-{channel}.xml` im gleichen versionierten GitHub-Release. Schlüsselkennung und Ziel müssen zum Mac-Paket gehören. Fehlende, doppelte oder fremde Ziele werden abgelehnt. V1 akzeptiert diese neuen Felder ausdrücklich nicht. Neue Prüfer lesen beide Versionen; bestehende öffentliche v1-Dateien werden nicht umgeschrieben.

Die Release-Pipeline signiert Appcasts mit dem bereits geladenen Paketschlüssel, bevor sie das Manifest signiert. Der strenge öffentliche Dateisatz besteht aus `release.json`, `release.json.sig`, jeweils Paket/`.sig`/`.sha256` und genau den im v2-Manifest genannten XML-Dateien. Jede XML-Datei trägt Sparkles eingebetteten Signaturblock; es gibt keine weitere XML-Sidecar-Datei. `candidate.json` und dessen Signatur bleiben lokal. `local-release.py export` erzeugt den passenden Upload-Ordner ohne Schlüsselzugriff.

Website und finale Mac-Abnahme verifizieren Feed-Bytes und Signaturen sowie die Übereinstimmung mit Version, Build, Mindestanforderung, Paketadresse, Grösse und Paketsignatur. Python und Node verwenden dieselben öffentlichen [Testvektoren](../release/examples/sparkle-contract.json). Die Website-Erweiterung muss vor dem ersten v2-Release deployed sein; [Release-Ablauf](UPDATE_RELEASE_RUNBOOK.md).

## Veröffentlichung und Signaturen

Der [lokale Release-Helfer](LOCAL_RELEASE.md) übernimmt gezielt gebaute und nativ geprüfte CI-Installer und erzeugt nach Developer-ID-Signierung/Notarisierung Prüfsummen, Ed25519-Signaturen und `release.json`. Er erzeugt noch keinen öffentlichen Tag oder Release. Erst wenn alle vorgesehenen Zielpakete geprüft und vollständig hochgeladen sind, dürfen Stable-/Beta-Verweise und native Appcasts darauf zeigen. Ein Manifest muss nicht alle sechs Ziele enthalten; die geplante Zielmenge wird ausdrücklich festgelegt, damit ein fehlgeschlagener Build nicht stillschweigend ein Paket aus dem Angebot entfernt.

Der vorgesehene erste Produktionsfeed liegt bei `https://github.com/Witzelfitz/beni-desktop-buddy/releases/latest/download/release.json` für Stable. Er entsteht erst in der Release-Pipeline. Beta braucht einen expliziten, getrennten Kanalverweis; GitHubs Latest-Verweis wird dafür nicht verwendet. Website und Updater benutzen die Metadaten aus derselben Veröffentlichung. [GitHub beschreibt Latest-/Release-Links hier](https://docs.github.com/en/repositories/releasing-projects-on-github/linking-to-releases).

Die `.sig`-Datei enthält die Ed25519-Signatur über die exakten Paketbytes als Base64. Zusätzlich signiert `release.json.sig` die exakten UTF-8-Bytes des Manifests; der lokale Prüfer authentifiziert diese vor dem Interpretieren. Der öffentliche Schlüssel liegt unter `release/keys/`; der private Teil in 1Password. Schlüsselkennungen sind Zuordnungen zu bereits vertrauenswürdigen öffentlichen Schlüsseln; das Manifest darf keine neuen Vertrauensanker einschleusen. R3a bestätigt die Kompatibilität mit Sparkle 2.10.0: dieselbe Paketsignatur wird als `sparkle:edSignature` übernommen, der XML-Feed erhält zusätzlich einen nativen eingebetteten Signaturblock. [Lokale Ableitung und Prüfung](UPDATES.md) ändern weder Kandidaten noch veröffentlichte Releases. R3b bindet den öffentlichen Schlüssel fest in das Mac-Bundle ein. R3c bereitet die Feed-Veröffentlichung vor; WinSparkle bleibt R4.

**R1 prüft Signaturangaben; die lokale Release-Pipeline ergänzt echte kryptografische Paket-/Manifestprüfung.** Sie ermittelt Grösse/SHA-256 erst aus den finalen notarisierten Dateien und prüft global steigende Buildnummern gegen authentifizierte vorhandene Releases. Rust-Updateadapter müssen diese Prüfung noch mit fest eingebundenen Schlüsseln implementieren, Versions-/Buildrückschritte verweigern und vor dem Installieren die Paketintegrität prüfen. R3a prüft die Sparkle-Signaturkompatibilität, R3b ergänzt den Desktop-Adapter und R3c die gemeinsame Veröffentlichungsvorbereitung. Produktionsfreigabe und echtes Update A → B bleiben offen. SHA-256 und HTTPS allein begründen kein Vertrauen in einen neuen Signierschlüssel. Apple-/Windows-Codesignierung sind zusätzliche Plattformprüfungen.

## Lokale Prüfung

Python 3.11 oder neuer genügt für Versionschecks und Plist-Stempeln. Der Schemavalidator verwendet festgelegte Python-Testabhängigkeiten in einer virtuellen Umgebung:

```sh
python3 -m venv .venv-release
.venv-release/bin/python -m pip install -r scripts/requirements-release.txt
python3 scripts/release.py check-versions
.venv-release/bin/python scripts/check-release.py
.venv-release/bin/python scripts/release.py validate release/examples/stable.json --for-build --channel stable --tag v0.1.0
.venv-release/bin/python scripts/release.py validate release/examples/beta.json --channel beta
BENI_RELEASE_PYTHON="$PWD/.venv-release/bin/python" ./scripts/check.sh --desktop
```

Unter Windows die virtuelle Umgebung mit `py -3 -m venv .venv-release` erstellen und `.venv-release\Scripts\python.exe` statt `.venv-release/bin/python` verwenden. `validate` ohne `--for-build` prüft auch historische oder Beispiel-Manifeste. Mit `--for-build` muss das Manifest der aktuellen Cargo-Version/Buildnummer entsprechen. Optionales `--tag` bindet es an den Release-Tag. Erfolg liefert Exitcode 0, Ablehnung 2; das Werkzeug führt keine Netzwerk-, Signier- oder Veröffentlichungsschritte aus.

Die vorhandenen drei CI-Jobs prüfen Versionskonsistenz. Nur der Linux-Job installiert die kleinen Schema-Testabhängigkeiten und führt die plattformunabhängigen Negativ-/Positivtests aus; keine weitere Matrix, keine Modellaufrufe und keine zusätzlichen Installer-Builds.
