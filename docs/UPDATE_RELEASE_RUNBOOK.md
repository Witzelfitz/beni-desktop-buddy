# Freigabe der Mac-Updatekette

Stand: 14. Oktober 2026. Website-/Pipeline-PRs und Versions-PR #43 sind gemergt. Version A ([0.1.2/Build 3](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.2)) ist für beide Mac-Ziele und Linux öffentlich, beide Mac-Feeds sind live. Schritte 1 und 2 sind durchgeführt; die Anleitung bleibt für B wiederverwendbar. Nutzerinstallation A und manuelle Updateprüfung («up to date») sind bestätigt; Version B (0.1.3/Build 4) ist ebenfalls veröffentlicht; der Nutzer hat das tatsächliche Update A → B bestätigt. 0.1.4/Build 5 mit lokalem Ollama-Chat ist veröffentlicht; 0.1.5/Build 6 ergänzt die Gesprächsblase und JSON-Emotionen und ist ebenfalls über die bestehenden Stable-Feeds verfügbar. Nutzerupdate auf 0.1.5 ist noch nicht bestätigt. 0.1.1 bleibt unverändert. Microsofts Firmenprüfung läuft unabhängig weiter.

## 1 · Beide Repositories vorbereiten

Desktop-Pipeline mit Manifest v2 und Website-Adapter für `/updates/macos/{architecture}/stable.xml` per PR mergen. Zuerst die Website deployen und prüfen: `/api/downloads` bietet weiterhin 0.1.1 an, `/download` lädt, beide neuen Feed-Routen antworten zunächst 503 ohne Installeradresse. Der bisherige v1-Release enthält keine Appcasts. Beta bleibt getrennt und ohne eigenen Verweis ebenfalls nicht verfügbar.

Python und Node prüfen dieselben öffentlichen XML-/Signatur-Testvektoren. Alle vorhandenen und neuen Prüfungen müssen bestehen. Zusätzliche Installer-Matrix oder Cloud-Schlüssel sind dafür nicht erforderlich.

## 2 · Erste updaterfähige Version A

Vorgesehen: 0.1.2/Build 3; vor dem Versions-PR gegen den aktuellen authentifizierten Release-Katalog prüfen. Root-, Desktop- und IPC-Manifeste sowie beide Cargo-Lockfiles gemeinsam aktualisieren. Im selben PR `macos_feed_ready` in `release/update-policy.json` freigeben. Das betrifft erst das neu gebaute Bundle, nicht bestehende Installationen. CI grün abwarten und mergen.

Nur einmal gezielt die beiden Mac-Architekturen bauen:

```sh
gh workflow run macos-installers.yml --ref main \
  -f architecture=both -f mode=development
```

Falls der neue Stable-Release weiterhin Linux anbieten soll, zusätzlich genau den Linux-Job ausführen. Ein alter Linux-Installer darf nicht unter einer neuen Produktversion erscheinen; Windows wartet auf die Herausgebersignatur.

```sh
gh workflow run portable-installers.yml --ref main -f platform=linux
```

Nach erfolgreicher nativer Abnahme den neuen Kandidaten lokal mit dem vorhandenen Developer-ID-Zertifikat, Schlüsselbundprofil `beni-notary` und 1Password-Release-Schlüssel erstellen:

```sh
python scripts/local-release.py prepare \
  --run MAC_RUN_ID --architecture aarch64 x86_64 \
  --minimum-os 15.0 --notary-profile beni-notary \
  --output dist/release-candidate-0.1.2
python scripts/local-release.py verify dist/release-candidate-0.1.2
python scripts/local-release.py export \
  --candidate dist/release-candidate-0.1.2 \
  --output dist/release-upload-0.1.2
```

Für Linux im `prepare`-Befehl zusätzlich `--portable-run LINUX_RUN_ID --portable-platform linux` angeben. Befehle mit der eingerichteten Python-Umgebung aus `scripts/requirements-release.txt` ausführen. Neue Ausgabeordner verwenden. `prepare` erzeugt die final notarisierten Pakete, Paketsignaturen, Appcasts und das v2-Manifest zusammen. `export` kopiert ausschliesslich geprüfte öffentliche Dateien; private Herkunftsbelege bleiben lokal.

Einen GitHub-Release **als Entwurf** mit sämtlichen Exportdateien erstellen. Den genauen Quellcommit als Tag-Ziel verwenden, keine vorhandenen Tags oder Assets ersetzen. Release-Notizen aus einer Datei übergeben. Assetmenge, Grössen, Digests und zurückgeladene Signaturen prüfen. Erst danach die finale Mac-Abnahme starten; sie baut Rust nicht erneut:

```sh
gh workflow run macos-release-acceptance.yml --ref main -f tag=v0.1.2
```

Beide finalen Architekturprüfungen müssen bestehen. Erst dann den Entwurf veröffentlichen und als Latest freigeben. Website-Cache: Erfolg höchstens fünf Minuten, vorherige Nichtverfügbarkeit höchstens 30 Sekunden. Beide öffentlichen XML-Routen anonym zurückladen: Bytes, Signaturen und Zielpakete müssen exakt dem Export entsprechen. `/api/downloads` muss denselben Release anbieten. Der Server signiert oder formatiert keine XML-Datei neu.

## 3 · Installation A und Update B

0.1.1 hat noch keinen eingebauten Updater. A einmal über den neuen Installer installieren. Anschliessend «Nach Updates suchen …» im Menüleistensymbol prüfen: natives Fenster, aktuelle Version, automatische Prüfwahl anfangs aus und nach Nutzerwahl über einen Neustart erhalten.

B ist als 0.1.3/Build 4 nach Katalogprüfung und finaler nativer Abnahme veröffentlicht. Die folgende Regel gilt ebenso für weitere Zielversionen: B muss dieselbe Bundle-ID, einen vertrauenswürdigen Release-Schlüssel, eine passende Developer ID und eine grössere Buildnummer behalten. Dieselbe Kandidaten-/Entwurfs-/Abnahmekette durchführen; keinen Testfehlerfeed als Stable veröffentlichen.

Die tatsächliche Updateabnahme benötigt zwei installierbare Bundles und eine kontrollierte Testinstallation. Testdaten und native Präferenzdomain isolieren; nach dem Sparkle-Neustart ausdrücklich prüfen, dass weiterhin der Testdatenordner verwendet wird. Vor Änderungen an einer produktiven Installation deren Einstellungen sichern und laufende Timer/Erinnerungen berücksichtigen. Bestehende Hooks nicht überschreiben.

| Fall | Nachweis |
| --- | --- |
| A erkennt B für den eigenen Chip | Angezeigte Version/Build und geladene DMG-Bytes stimmen mit dem authentifizierten Release überein. |
| Installation und Neustart | B läuft tatsächlich; Einstellungen, Empfangsfreigabe und vorhandene Hooks sind erhalten. |
| Timer aktiv/pausiert oder Erinnerung geplant | «Später» hält den Neustart an; erst ausdrückliche Zustimmung installiert. |
| Automatische Prüfwahl | Bleibt über Installation und Neustart erhalten; keine automatische Installation. |
| Defekte Feed-/Paketsignatur | Eigene Testquelle; natives Fehlerergebnis, A bleibt startbar und unverändert. |
| Download abbrechen | A bleibt nutzbar, erneute Prüfung möglich. |
| Installationsort nicht schreibbar | Verständliches Ergebnis, kein stiller Datenverlust, A bleibt nutzbar. |
| Schreibfehler bei Einstellungen | Neustart bleibt angehalten, IPC weiterhin aktiv. |

Ergebnisse und Grenzen in `docs/TEST_MATRIX.md` dokumentieren. Native Intel/macOS-15-Abnahme und Tests auf frischen Nutzergeräten ausdrücklich von der Entwickler-Mac-Abnahme unterscheiden. Windows-/Linux-Updateadapter und KI-Einrichtung bleiben die nächsten eigenen Schritte.
