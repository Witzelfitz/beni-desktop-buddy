# Mit Beni reden

Verfügbar ab [0.1.4/Build 5](https://github.com/Witzelfitz/beni-desktop-buddy/releases/tag/v0.1.4) für macOS und Linux. Auf dem Mac über das Menüleistensymbol aktualisieren, unter Linux das neue AppImage manuell ersetzen. Windows-Veröffentlichung wartet ausdrücklich auf die Microsoft-Firmenprüfung und Herausgebersignatur. Chatfenster und lokaler Ollama-Adapter sind auf macOS geprüft; echte Windows-/Linux-Chatgespräche bleiben separat offen. Version 0.1.3 enthält diesen Chat noch nicht. Claude und OpenAI sowie ein allgemeiner Ersteinrichtungsdialog folgen separat.

## Nächste Version: Gespräch direkt bei Beni

Im aktuellen Entwicklungs-PR hängt die Gesprächsblase an der Figur, im selben transparenten Buddy-Fenster. Das separate Chatfenster entfällt. Beni behält möglichst seine Grösse und Position: Die Blase erscheint darüber, bei wenig Platz oben seitlich. Bildschirmwechsel lösen eine erneute Grössenprüfung aus; auf Wayland bestimmt weiterhin der Compositor die absolute Fensterposition. Bei sehr kleinen Bildschirmen kann die Figur während des Gesprächs kleiner dargestellt werden.

Beni antwortet ausdrücklich in sauberem Hochdeutsch mit Schweizer Rechtschreibung, niemals in Dialekt. Eszett wird vor der Anzeige in «ss» umgewandelt; bekannte Dialektgrüsse/-wörter führen zu einer sicheren Fehlermeldung. Diese Wortprüfung ergänzt die Modellvorgabe und ist keine vollständige Spracherkennung oder Grammatikprüfung.

Der lokale Adapter fordert JSON nach Schema an. Erst nach vollständiger Prüfung erscheinen die Antwortabschnitte; rohe JSON-Teile und Modellüberlegungen bleiben unsichtbar. Pro Abschnitt liefert das Modell Text und eine Emotion. Die App ordnet dieser Emotion eine bestehende Geste zu. Abschnitte wechseln nach einer Lesedauer oder über «Weiter»; die letzte Antwort bleibt lesbar. «Gespräch nachlesen» zeigt die vollständigen Gesprächspaare. Warten verwendet «Denken»; Pause und reduzierte Bewegung bleiben wirksam. Gesten starten je Abschnitt einmal und kehren anschliessend zum bisherigen Ablauf zurück.

[JSON-Beispiel](examples/chat-reply.json): Version 1, ein bis sechs Abschnitte, je höchstens 600 Unicode-Zeichen. Zulässige Emotionen:

| Emotion | Bestehende Geste |
| --- | --- |
| neutral | reden |
| friendly | hallo |
| curious | idee |
| happy | jubeln |
| sad | traurig |
| surprised | ueberrascht |
| encouraging | daumen |
| thoughtful | nachdenken |

Unbekannte Emotionen, zusätzliche Felder/Animationsnamen, falsche Version, leere/übergrosse Texte und unvollständiges JSON werden abgewiesen. Wire-JSON ist auf 24 KiB begrenzt. Der Kontext enthält dieselben geprüften JSON-Antworten; dessen 32-KiB-Grenze berücksichtigt diese tatsächlich gesendeten Bytes. Gespräch und Emotionen bleiben in der Sitzung. Sprachausgabe wurde ausdrücklich zurückgestellt; Mikrofonzugriff gehört nicht zu diesem Schritt.

«Verbindung & Kontext» enthält Modellwahl und genaue Vorschau. Nach Modellwahl oder Senden klappt die Einrichtung zu. Im Textfeld beendet Escape zuerst die Eingabe; ein weiteres Escape schliesst die Blase und bricht gegebenenfalls die Anfrage ab. Die normale Fensterposition wird separat von der vorübergehenden Gesprächsfläche gespeichert. Statusmeldungen werden bei offenem Gespräch weiter angenommen und im Verlauf geführt; ihre Blasen warten auf das Schliessen und bleiben an TTL gebunden.

Diese Änderung ist noch nicht in einem neuen öffentlichen Installer enthalten. Die folgenden Angaben beschreiben die bereits veröffentlichte 0.1.4.

## Öffnen und verbinden

1. Beni anklicken: Die Bedienleiste klappt auf. «Reden» öffnet ein eigenes, verschiebbares und skalierbares Fenster. Unter «Mit Beni reden» ist dieselbe Aktion im Menüleistensymbol erreichbar, auch bei ausgeblendetem Buddy.
2. «Lokales Ollama verbinden» wählen. Die Verbindung ist standardmässig aus; ohne diese Auswahl erfolgt keine Ollama-Abfrage.
3. Ein vorhandenes lokales Textmodell wählen. Die Liste nennt Modellname und lokale Dateigrösse. Bild-, Embedding- und als remote/Cloud markierte Modelle werden ausgefiltert. Bei älteren Diensten ohne Fähigkeitsangaben wird kein ungeprüftes Modell freigeschaltet.
4. Nachricht eingeben und «Senden» drücken. Cmd+Enter auf macOS beziehungsweise Ctrl+Enter auf Windows/Linux sendet ebenfalls; Enter fügt eine neue Zeile ein.

Der erste Adapter verwendet ausschliesslich `http://127.0.0.1:11434`, ohne Systemproxy oder HTTP-Weiterleitungen. Ollama muss lokal laufen. Die App installiert weder Ollama noch Modelle und wechselt nicht auf einen Cloud-Dienst. Nach einer Modelländerung im Dienst «Modelle aktualisieren» wählen. Beni prüft Modell-ID, lokalen Status und Chatfähigkeit vor jedem Gesprächsaufruf erneut. Ollama selbst ist ein separater, vom Nutzer verwalteter Dienst; die App kann dessen interne Konfiguration nicht kontrollieren.

## Gespräch und Bedienung

Antworten erscheinen Stück für Stück im Chatfenster. Während Beni wartet, verwendet er die vorhandene «Denken»-Animation; bei sichtbarer Ausgabe «Reden». Animationspause und reduzierte Bewegung bleiben wirksam. Laufende Chats halten die normale Statusanzeige zurück; Statusmeldungen werden weiterhin getrennt angenommen und unterliegen ihren bestehenden TTL-Grenzen.

«Abbrechen» beendet eine laufende Anfrage. Auch das Schliessen des Chatfensters bricht eine laufende Antwort ab, beendet aber Beni nicht. «Chat leeren» entfernt die Gesprächspaare dieser Sitzung. Modellwechsel, Aktualisieren der Modellliste und «Ollama trennen» beginnen einen neuen Kontext. Eine neue Anfrage wartet, bis der bisherige Worker seine Verbindung freigegeben hat.

«Kontext für diese Anfrage» zeigt vor dem Senden genau die begrenzten Nachrichten inklusive Benis Rolle, abgeschlossenen Gesprächspaaren und aktueller Frage. Unvollständige oder abgebrochene Antworten werden nicht in den nächsten Kontext übernommen. Hook-/Statusverlauf, Dateien, Zwischenablage, Bildschirm und Mikrofon sind nicht Bestandteil der Anfrage; Antworten führen keine Systemaktionen aus.

Verbindungsauswahl und Modellname werden gespeichert. Chattexte bleiben ausschliesslich in der aktuellen App-Sitzung; ein App-Neustart beginnt einen leeren Chat. Ein Update-Neustart wird während einer aktiven Anfrage wie bei Timern zur Bestätigung angeboten. Vor dem Neustart werden die Auswahl gespeichert und der Worker abgebrochen.

## Grenzen und Fehler

Die [gemeinsame Grundlage](adr/0005-chat-provider-contract.md) begrenzt Eingabe auf 4 KiB, gesamten Anfragekontext auf 32 KiB und zehn abgeschlossene Gesprächspaare sowie Antworttext auf 16 KiB. Das Ausgabetokenbudget beträgt 512. Grössere Antworten enden mit einer verständlichen Anzeige; Byte- und Ereignisgrenzen werden zusätzlich unabhängig vom Anbieter geprüft.

HTTP-Anfragen laufen in einem Hintergrundworker mit begrenztem Ereigniskanal. Abbruch bleibt auch bei Rückstau wirksam und verwirft verspätete Ereignisse. Der Adapter schliesst wartende Anfragen bei Abbruch und verwendet Connect-/Metadaten-/Gesamtfristen. Wire-Daten, einzelne NDJSON-Zeilen und Modelllisten sind vor der Verarbeitung begrenzt. EOF ohne Abschluss, falsche Rollen, Werkzeugaufrufe, fehlende Pflichtfelder oder ungültige Typen bei bekannten Protokollfeldern und Cloud-Markierungen führen zu sicheren Fehlerkategorien; rohe Providerfehler werden nicht angezeigt oder protokolliert. Nicht benötigte Metadaten und Modellüberlegungen werden ignoriert.

Falls Ollama nicht erreichbar ist, bleibt der Chat bedienbar und zeigt den Verbindungsfehler. Läuft kein geeignetes lokales Chatmodell, kann der Nutzer die Verbindung später erneut prüfen. Es erfolgt kein stiller Modell-Download.

## Prüfung

Offline-Prüfungen verwenden einen eigenen Loopback-Testserver. Geprüft sind echte HTTP-Anfragen, Modellfilter und erneute Modellprüfung, NDJSON mit aufgeteiltem UTF-8, Antwort-/Wire-Grenzen, fehlender Abschluss, Redirect-/Fehlerbehandlung und das tatsächliche Schliessen einer wartenden Verbindung bei Abbruch. Ein begrenzter Worker-Kanal und der Erhalt des Abschlussereignisses werden separat geprüft.

Echte Modellabnahme ist bewusst aus der normalen CI ausgenommen:

```sh
cargo test --manifest-path apps/desktop/Cargo.toml --locked \
  real_local_ollama_answers_without_downloading_a_model -- --ignored
```

Dieser gezielte Test verwendet ein bereits vorhandenes lokales Chatmodell und einen festen, kurzen Testsatz. Er lädt keine Modelle herunter und verwendet keine Cloud-Keys. Native macOS-Abnahme: Klickzugang, eigenes Fenster, lokale Modellwahl, echte Antwort, Mindestgrösse, Modell-Popup schliessen, Abbruch und gespeicherte Modellwahl nach Neustart. Finale 0.1.4-Installer bestehen Installation, CLI, eigenes Fenster und Entfernung auf beiden Mac-Zielen sowie Linux/X11/Wayland. Echte Windows-/Linux-Chatgespräche und tatsächlicher Nutzer-Chat nach dem Update bleiben separat offen; siehe [Testmatrix](TEST_MATRIX.md).

Referenzen: [Ollama Chat](https://docs.ollama.com/api/chat), [Streaming](https://docs.ollama.com/api/streaming), [Modellliste](https://docs.ollama.com/api/tags), [offizieller API-Vertrag für Fähigkeits-/Remote-Angaben](https://github.com/ollama/ollama/blob/main/api/types.go), [reqwest 0.13.5](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html).

Strukturiertes Ollama-Format: [offizielle Dokumentation](https://docs.ollama.com/capabilities/structured-outputs).
