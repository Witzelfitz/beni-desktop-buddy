# Updatekette · macOS zuerst

Stand 14. Oktober 2026: R3a bereitet authentifizierte Sparkle-Appcasts vor. Beni 0.1.1 hat weiterhin keinen eingebauten Updater. Die Microsoft-Firmenprüfung blockiert diese macOS-Arbeit nicht.

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

Nächste Schritte und geplante Sicherheitsvorgaben: [ADR 0004](adr/0004-sparkle-update-chain.md). Echte Installation A → B und die Tray-Bedienung sind noch offen.
