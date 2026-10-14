# ADR 0006: Gespräch an der Figur und geprüfte JSON-Emotionen

Status: umgesetzt in PR #54; als 0.1.5/Build 6 nach nativer Paketabnahme veröffentlicht.

## Auftrag

Der Nutzer verlangt sauberes Hochdeutsch mit Schweizer Rechtschreibung, ausdrücklich keinen Dialekt. Das Gespräch soll an der Figur hängen und passende unterschiedliche Emotionen verwenden. Die Nachfrage nach hörbarer Ausgabe wurde mit «Vorerst nur Text und passende Gesten» beantwortet.

## Entscheidung

Die Gesprächsblase und Eingabe gehören in dasselbe transparente Root-Fenster wie Beni. Die frühere Entscheidung für einen eigenen Chat-Viewport in ADR 0005 wird damit ersetzt. Einstellungen behalten ihr eigenes Fenster. Eine sichtbare Blase enthält Antwort, Eingabe, Schliessen, Abbrechen und optionalen Gesprächsverlauf; Modellwahl und genaue Kontextvorschau bleiben einklappbar.

Öffnen erweitert die Buddy-Fläche oberhalb der Figur; bei zu wenig Platz erscheint die Blase seitlich. Die Avatarfläche behält möglichst Grösse und Bildschirmanker. Vor jeder Grössen-/Layoutänderung wird die aktuelle native Position und sichtbare Arbeitsfläche gelesen, auch wenn der Buddy seit dem letzten Logikdurchlauf bewegt wurde. Es gibt keine zusätzliche Abfrage der Bildschirmgeometrie je Idle-Frame. Monitoränderungen prüfen die Fläche erneut. Eine separate normale Fensterposition verhindert, dass die vorübergehend erweiterte Chatfläche den nächsten Start verschiebt. Wayland garantiert weiterhin keine absolute Fensterposition; sehr kleine Flächen können die Figur vorübergehend verkleinern.

Der Ollama-Adapter fordert ein JSON-Schema mit Version 1 und ein bis sechs Abschnitten an. Jeder Abschnitt enthält ausschliesslich `text` und eine erlaubte `emotion`; zusätzliche Felder und beliebige Animationsnamen werden abgewiesen. Vor Anzeige wird die gesamte begrenzte Antwort geprüft. Die Anwendung meldet während des Empfangs nur Fortschritt und zeigt «Denken»; unvollständiges JSON und interne Modellüberlegungen erscheinen nicht. Validierte Abschnitte erhalten eine Lesedauer, wechseln automatisch oder über «Weiter» und spielen je Abschnitt eine bestehende Geste einmal. Nach dem letzten Abschnitt bleibt der Text sichtbar, der ursprüngliche Player wird wiederhergestellt. Pause und reduzierte Bewegung bleiben erhalten.

`buddy-chat` verwendet die bereits im Workspace festgelegten serde-/serde_json-Versionen für diesen reinen Datenvertrag; es hat weiterhin keine GUI, keinen Transport und keine Systemzeit. Die neue Emotion-Positivliste enthält acht Werte. Die Zuordnung auf bestehende Animationsnamen ist fest und wird gegen die eingebettete Engine geprüft. Der Animationskern und die Beni-Assets bleiben unverändert.

Hochdeutsch wird in der festen Persona ausdrücklich verlangt. Eszett wird vor Anzeige normalisiert; ein konservativer Wortfilter weist bekannte Dialektgrüsse und -wörter ab. Er ersetzt keine vollständige Sprach-/Grammatikprüfung. Auch der generische Textteil-Vertrag prüft die fertige Antwort; die neue Oberfläche zeigt keine ungeprüften Zwischenstände. Nur vollständige, gültige Gesprächspaare gehen in den folgenden Kontext ein. Dort werden strukturierte Antworten als exakt dieselben kanonischen JSON-Bytes geführt; die 32-KiB-Grenze berücksichtigt diese Bytes statt nur des angezeigten Textes.

## Grenzen und Abnahme

JSON höchstens 24 KiB, pro Abschnitt höchstens 600 Unicode-Zeichen, höchstens sechs Abschnitte. Weitere Anfrage-/Wire-/Zeit-/Ereignisgrenzen gelten weiter. Es werden keine Modell-Downloads, Audioausgabe, Mikrofon-, Datei- oder Bildschirmdaten ergänzt. Normale Statusmeldungen bleiben getrennt; ihre Anzeigen warten während eines offenen Gesprächs und verfallen weiterhin nach TTL.

Positive und negative Tests prüfen Sprache, UTF-8, Version/Felder/Emotionen, Kontextbytes, Ereignis-/Zeitgrenzen, Abbruch und alte Ereignisse, Gestenfolge und Wiederherstellung, Oberfläche/Inputregion sowie Fensteranker und Monitorwechsel. Native macOS-Abnahme verwendet eigene QA-Daten und ein bereits vorhandenes lokales Modell. [Bedienung/JSON-Beispiel](../CHAT.md), [Abnahme](../TEST_MATRIX.md), [Ollama-Schemaformat](https://docs.ollama.com/capabilities/structured-outputs).
