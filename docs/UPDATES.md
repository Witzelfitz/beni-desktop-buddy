# Updatekette · macOS zuerst

Stand 14. Oktober 2026: [Version 0.1.2/Build 3](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.2) ist als erstes updaterfähiges Mac-Bundle veröffentlicht. R3a/R3b/R3c und Versions-PR #43 sind gemergt; Website PR #7 ist deployed. Beide Architektur-Feeds und Downloadkatalog sind live und verwenden denselben authentifizierten Release. 0.1.1 und ältere Installationen benötigen einmalig die manuelle Installation von 0.1.2. Die Microsoft-Firmenprüfung blockiert diese macOS-Arbeit nicht.

## Anbindung im Buddy

«Nach Updates suchen …» im Menüleistensymbol öffnet bei freigegebenem Feed Sparkles native Oberfläche. Solange `release/update-policy.json` den öffentlichen Feed noch nicht freigibt, erscheint eine Erklärung im eigenen Einstellungsfenster. Ist der Buddy ausgeblendet, wird er dafür zunächst zurückgeholt. Es erfolgt keine Anfrage an einen noch fehlenden Endpunkt. Bei ungepackten Cargo-Starts oder noch nicht unterstützten Plattformen bleibt Beni bedienbar und erklärt den verfügbaren Downloadweg.

Die Checkbox «Automatisch nach Updates suchen» ist anfangs ausgeschaltet und bis zur Feed-Freigabe deaktiviert. Sparkle speichert die Nutzerauswahl selbst; Beni setzt sie beim Start nicht zurück. Während einer laufenden Prüfung sind die zugehörigen Buttons gesperrt. Automatische Installation, Profiling und JavaScript sind ausgeschaltet.

Vor einem angeforderten Update-Neustart speichert Beni die Einstellungen synchron und beendet erst danach seinen IPC-Empfang. Ein Dateisystemfehler hält den Neustart an; IPC bleibt aktiv und der Nutzer kann später erneut versuchen. Die gespeicherte Empfangsfreigabe bleibt erhalten. Bei laufenden oder pausierten Timern sowie geplanten/wartenden Erinnerungen zeigt das separate Fenster «Später» oder «Update installieren und neu starten». «Später» bleibt auch nach Ende der Aufgaben wirksam, bis der Nutzer den Neustart ausdrücklich wählt. Timer und Erinnerungen selbst sind weiterhin sitzungsgebunden.

Das SDK wird beim Packaging anhand von `release/sparkle-sdk.json` geladen oder über `BENI_SPARKLE_ARCHIVE` als bereits vorhandenes Archiv übernommen. Grösse und Hash müssen vor dem Entpacken stimmen. Framework-Symlinks, Hilfsprogramme und Lizenz bleiben erhalten. Geschachtelte Codesign-Objekte werden von innen nach aussen mit derselben Identität signiert, ihre vorhandenen Entitlements bewahrt. Der signierte Bundle-Inhalt trägt den öffentlichen Verifikationsschlüssel und getrennte HTTPS-Verweise für Architektur/Kanal. Bundle-Mindestversion ist 15.0; das Rust-/Bridge-Buildziel bleibt 11.0.

## Integrierte Vorbereitung für neue Releases

`local-release.py prepare` erzeugt nach den finalen Paketsignaturen beide benötigten Appcasts im selben unveröffentlichten Kandidaten. Dafür verwendet es den bereits geladenen Release-Schlüssel. Das neue Manifest v2 enthält `macos_update_feeds` mit Architektur, versionsgebundener Adresse, Grösse, SHA-256 und bestehender Schlüsselkennung. Kandidatenprüfer und finale Release-Abnahme verifizieren die eingebetteten Feed-Signaturen und vergleichen den XML-Inhalt bytegenau mit dem angekündigten Mac-Paket. Die bisherigen Manifest-v1-Releases bleiben lesbar und unverändert.

`local-release.py export` prüft nochmals die tatsächlich kopierten Bytes und legt einen neuen Upload-Ordner an, ohne die privaten Kandidatenbelege. Es lädt nichts hoch. Auf der Website kommen `/updates/macos/aarch64/stable.xml` und `/updates/macos/x86_64/stable.xml` hinzu. Sie verwenden denselben vollständig authentifizierten Stable-Release wie `/api/downloads` und liefern die signierten XML-Bytes unverändert. Fehlt ein beworbener Feed oder stimmt er nicht, sind weder Update- noch Downloadlinks verfügbar. Ein alter v1-Release bietet weiterhin Downloads, aber noch keinen Update-Feed. Beta bleibt ohne eigenen Kanalverweis nicht verfügbar.

Die Website-Unterstützung muss vor dem ersten v2-Release deployed sein. Danach folgt die neue updaterfähige Version; Der Versions-PR für 0.1.2/Build 3 schaltet `release/update-policy.json` ausschliesslich für das neue Bundle ein. Veröffentlichung erst nach finaler Installer-Abnahme. Der konkrete Ablauf steht im [Release-Ablauf](UPDATE_RELEASE_RUNBOOK.md).

## Separate Appcast-Probe aus einem lokalen Kandidaten

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

Diese separate Probe bleibt für R3a-Abnahmen verfügbar. Für neue Veröffentlichungen die integrierten Dateien aus `local-release.py prepare` und `export` verwenden. Niemals Appcasts nachträglich zu 0.1.1 oder anderen veröffentlichten Releases hochladen.

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

0.1.2 ist final abgenommen und veröffentlicht, beide Stable-Feeds sind live. Nächste Schritte: einmalige Nutzerinstallation A und echtes Update A → B. Downloadabbruch, nicht schreibbarer Installationsort und finaler Erhalt der Einstellungen nach Installation sind noch offen; siehe [ADR 0004](adr/0004-sparkle-update-chain.md).

Die synchronen Persistenzprüfungen verwenden den tatsächlichen vendorten FileStorage-Quelltext:

```sh
python scripts/check-storage-sync.py
```

Sie prüfen einen vorangegangenen Hintergrundschreibauftrag, den neuen Snapshot und einen Dateisystemfehler. Die Desktop-Prüfung bestätigt mit echtem isoliertem IPC, dass der Empfang bei einem Speicherfehler aktiv bleibt. Der [eframe-Patch](../vendor/eframe/BENI_PATCH.md) dokumentiert die begrenzte API-Erweiterung.

## Veröffentlichung 0.1.2

Alle 13 öffentlichen Dateien, beide Feed-Routen und der Live-Katalog stimmen bytegenau mit dem authentifizierten Release-Kandidaten überein. Ein echter Chromium-Download des Apple-Silicon-DMGs von der produktiven Downloadseite hat die erwartete Grösse und SHA-256; macOS-Vorauswahl und Version/Build stimmen. Sparkle 2.10.0 hat den tatsächlichen signierten Feed in einer isolierten Developer-ID-Probe geladen: Signaturprüfung erfolgreich, genau ein passendes Item für Build 3 und 0.1.2; natives Ergebnis «aktuelle Version». Der Probe-Server war ausschliesslich lokal, die produktive App behält ihre feste HTTPS-Adresse.

Finale native Installerabnahme für Intel/Apple Silicon auf macOS 15.7.9 bestanden. Beim ersten Intel-Lauf schlug der Ticketcheck mit Exitcode 68 fehl; lokales DMG, App und kopierte App validierten korrekt. Gezieltes Wiederholen nur des Intel-Jobs bestand mit identischen Paketbytes. Linux 0.1.2 ist ebenfalls nativ unter X11/Wayland abgenommen. Bestehende Programme-Installation und Codex-Hook blieben bis zur Nutzerinstallation bytegleich.

Die Nutzerinstallation und tatsächliche Aktualisierung A → B sind damit noch nicht bestätigt. Anschliessend folgen verschobener Neustart, Einstellungen/Empfang nach Installation, defekte Paketsignatur, Downloadabbruch und nicht schreibbarer Installationsort.
