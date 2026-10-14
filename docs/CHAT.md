# Mit Beni reden

Entwicklungsstand: Chatfenster und lokaler Ollama-Adapter sind implementiert und auf macOS geprüft. Noch kein neuer öffentlicher Installer; Version 0.1.3 enthält diesen Chat noch nicht. Claude und OpenAI sowie ein allgemeiner Ersteinrichtungsdialog folgen separat.

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

Dieser gezielte Test verwendet ein bereits vorhandenes lokales Chatmodell und einen festen, kurzen Testsatz. Er lädt keine Modelle herunter und verwendet keine Cloud-Keys. Native macOS-Abnahme: Klickzugang, eigenes Fenster, lokale Modellwahl, echte Antwort, Mindestgrösse, Modell-Popup schliessen, Abbruch und gespeicherte Modellwahl nach Neustart. Native Windows-/Linux-Chatabnahme und neue öffentliche Installer bleiben separat offen.

Referenzen: [Ollama Chat](https://docs.ollama.com/api/chat), [Streaming](https://docs.ollama.com/api/streaming), [Modellliste](https://docs.ollama.com/api/tags), [offizieller API-Vertrag für Fähigkeits-/Remote-Angaben](https://github.com/ollama/ollama/blob/main/api/types.go), [reqwest 0.13.5](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html).
