# Unveröffentlichte Release-Beispiele

`stable.json` und `beta.json` sind ausschliesslich Testdaten für Schema und Validator. URLs, Dateigrössen, SHA-256-Werte und Systemanforderungen sind Beispiele; es gibt dafür keine veröffentlichten Installer oder Signaturen. Die Website und ein künftiger Updater dürfen diese Dateien nicht als Download-Feed verwenden.

Ein Manifest darf nur die tatsächlich gebauten und freigegebenen Plattformen/Architekturen aufführen. Die Beispiele zeigen macOS/Apple Silicon sowie Windows/Linux/x86_64; damit wird noch keine Plattformabnahme behauptet. Details stehen im [Release-Vertrag](../../docs/RELEASES.md).

`sparkle-contract.json` enthält drei öffentliche Signatur-/XML-Testvektoren für Stable/ARM, Stable/Intel und Beta/ARM, inklusive Unicode und XML-Escaping. Ihr zufälliger Testschlüssel wurde verworfen; nur der öffentliche Schlüssel und Signaturen sind gespeichert. Dieselbe Datei liegt im Website-Repo unter `tests/fixtures/sparkle-contract.json`. Beide Testsätze müssen bei einer Änderung weiter dieselben Bytes akzeptieren. Keine Produktionsdownloads.
