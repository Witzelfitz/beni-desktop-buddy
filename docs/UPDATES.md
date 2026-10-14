# Updatekette · macOS zuerst

Stand 14. Oktober 2026: [R3a/PR #40](https://github.com/Witzelfitz/beni-desktop-buddy/pull/40) ist gemergt. R3b ergänzt Sparkle im Mac-Bundle, die Tray-Aktion und die Neustart-Abfrage. Die öffentlich veröffentlichte Version 0.1.1 hat weiterhin keinen eingebauten Updater; die neuen Quellen sind noch nicht als neue Version veröffentlicht. Die Microsoft-Firmenprüfung blockiert diese macOS-Arbeit nicht.

## Anbindung im Buddy

«Nach Updates suchen …» im Menüleistensymbol öffnet bei freigegebenem Feed Sparkles native Oberfläche. Solange `release/update-policy.json` den öffentlichen Feed noch nicht freigibt, erscheint eine Erklärung im eigenen Einstellungsfenster. Ist der Buddy ausgeblendet, wird er dafür zunächst zurückgeholt. Es erfolgt keine Anfrage an einen noch fehlenden Endpunkt. Bei ungepackten Cargo-Starts oder noch nicht unterstützten Plattformen bleibt Beni bedienbar und erklärt den verfügbaren Downloadweg.

Die Checkbox «Automatisch nach Updates suchen» ist anfangs ausgeschaltet und bis zur Feed-Freigabe deaktiviert. Sparkle speichert die Nutzerauswahl selbst; Beni setzt sie beim Start nicht zurück. Während einer laufenden Prüfung sind die zugehörigen Buttons gesperrt. Automatische Installation, Profiling und JavaScript sind ausgeschaltet.

Vor einem angeforderten Update-Neustart speichert Beni die Einstellungen synchron und beendet erst danach seinen IPC-Empfang. Ein Dateisystemfehler hält den Neustart an; IPC bleibt aktiv und der Nutzer kann später erneut versuchen. Die gespeicherte Empfangsfreigabe bleibt erhalten. Bei laufenden oder pausierten Timern sowie geplanten/wartenden Erinnerungen zeigt das separate Fenster «Später» oder «Update installieren und neu starten». «Später» bleibt auch nach Ende der Aufgaben wirksam, bis der Nutzer den Neustart ausdrücklich wählt. Timer und Erinnerungen selbst sind weiterhin sitzungsgebunden.

Das SDK wird beim Packaging anhand von `release/sparkle-sdk.json` geladen oder über `BENI_SPARKLE_ARCHIVE` als bereits vorhandenes Archiv übernommen. Grösse und Hash müssen vor dem Entpacken stimmen. Framework-Symlinks, Hilfsprogramme und Lizenz bleiben erhalten. Geschachtelte Codesign-Objekte werden von innen nach aussen mit derselben Identität signiert, ihre vorhandenen Entitlements bewahrt. Der signierte Bundle-Inhalt trägt den öffentlichen Verifikationsschlüssel und getrennte HTTPS-Verweise für Architektur/Kanal. Bundle-Mindestversion ist 15.0; das Rust-/Bridge-Buildziel bleibt 11.0.

## Appcasts aus einem lokalen Kandidaten

Mit der Python-Umgebung aus `scripts/requirements-release.txt`:

```sh
python scripts/sparkle.py prepare \
  --candidate dist/release-candidate-NEUE-VERSION \
  --output dist/appcasts-NEUE-VERSION
python scripts/sparkle.py verify \
  --candidate dist/release-candidate-NEUE-VERSION \
  --output dist/appcasts-NEUE-VERSION
```

`prepare` verwendet die bestehende lokale Schlüsselzuordnung in `dist/release-signing.json` und den separaten 1Password-Release-Schlüssel. Erst nach erfolgreicher Kandidatenprüfung wird der Schlüssel angefragt. `verify` braucht nur den eingecheckten öffentlichen Schlüssel. Ein vorhandener Ausgabeordner wird abgelehnt; die Dateien des Kandidaten bleiben unverändert.

Für einen Stable-Kandidaten mit beiden Mac-Paketen entstehen:

- `appcast-macos-aarch64-stable.xml`
- `appcast-macos-x86_64-stable.xml`

Beta-Dateien enden auf `-beta.xml` und enthalten den Sparkle-Kanal `beta`. Es entstehen nur tatsächlich vorhandene Mac-Ziele. Die Feeds enthalten dieselben versionsgebundenen DMG-Adressen und Paketsignaturen wie der authentifizierte Downloadkatalog. Die Sparkle-Buildnummer entspricht `build_number`, die angezeigte Version der Release-Version.

Die Appcasts sind lokale, unveröffentlichte Zusatzdateien. Sie dürfen noch nicht als zusätzliche Assets zu vorhandenen Releases hochgeladen werden: Die aktuelle Website und Release-Abnahme erwarten exakt die bisherigen Paket-/Metadatendateien. Die koordinierte Feed-Veröffentlichung folgt nach Bundle-/Tray-Anbindung. Keine alten Versionen nachträglich ändern.

## Prüfungen

```sh
python scripts/check-sparkle.py
```

Diese Offline-Prüfung braucht kein Modell, keinen Netzwerkzugriff und keine Zugangsdaten. Sie läuft einmal im vorhandenen Linux-CI-Job und im lokalen `scripts/check.sh`.

Für den nativen Kompatibilitätsversuch das SDK aus `release/sparkle-sdk.json` herunterladen und den Archivpfad übergeben:

```sh
python scripts/check-sparkle.py --sparkle-archive /absoluter/pfad/Sparkle-2.10.0.tar.xz
```

Der Helfer prüft Grösse und Hash vor dem Entpacken. Sparkles natives `sign_update` akzeptiert die Python-Paket-/Feed-Signaturen und weist manipulierte Bytes zurück. Zusätzlich wird eine bytegleiche Feed-Signatur mit dem nativen Signierer erzeugt. Dafür kommt ausschliesslich ein neu erzeugter Testschlüssel über private Standardeingabe zum Einsatz; der Produktionsschlüssel bleibt unangetastet. Ohne diesen Parameter ist die native Probe ausdrücklich `NOT_RUN`.

Die Brücke lässt sich separat ohne Rust-Neubau prüfen:

```sh
BENI_SPARKLE_ARCHIVE=/absoluter/pfad/Sparkle-2.10.0.tar.xz \
  python scripts/check-macos-updater.py --developer-id
```

Sie baut ein temporäres Testbundle, prüft Laden/Signaturen, gespeicherte automatische Prüfwahl über echte Neustarts, den verzögerten Installationscallback und einen ungültigen Feed mit eigenem Fehlerdialog. Nur der Test verwendet einen kurzlebigen HTTP-Server auf `127.0.0.1`; produktive Feed-Verweise bleiben festes HTTPS. Test-Präferenzen liegen in frisch erzeugten Domains und werden entfernt. Bestehende Installation und Hooks bleiben unangetastet. Ohne `--developer-id` wird ad hoc signiert.

Für eine getrennte GUI-Abnahme kann `BENI_BUDDY_DATA_DIR` einen absoluten Datenordner angeben. Relative Pfade werden abgelehnt. Der produktive Start verwendet weiterhin den bisherigen Standardpfad. Sparkle-Präferenzen müssen in Testkopien zusätzlich über einen eigenen `SUDefaultsDomain` isoliert werden; dieser muss sich vom Bundle-Identifier der Test-App unterscheiden.

Nächste Schritte: koordinierte Feed-Veröffentlichung in beiden Repositories, neue Bootstrap-Version und echtes Update A → B. Downloadabbruch, nicht schreibbarer Installationsort und finaler Erhalt der Einstellungen nach Installation sind noch offen; siehe [ADR 0004](adr/0004-sparkle-update-chain.md).

Die synchronen Persistenzprüfungen verwenden den tatsächlichen vendorten FileStorage-Quelltext:

```sh
python scripts/check-storage-sync.py
```

Sie prüfen einen vorangegangenen Hintergrundschreibauftrag, den neuen Snapshot und einen Dateisystemfehler. Die Desktop-Prüfung bestätigt mit echtem isoliertem IPC, dass der Empfang bei einem Speicherfehler aktiv bleibt. Der [eframe-Patch](../vendor/eframe/BENI_PATCH.md) dokumentiert die begrenzte API-Erweiterung.
