# ADR 0005: Gemeinsame Chat-Grundlage vor den Anbieteradaptern

Status: Grundlage und Ollama-Adapter implementiert. Die Entscheidung für einen eigenen Chat-Viewport wird im aktuellen Entwicklungs-PR durch [ADR 0006](0006-attached-conversation-emotions.md) ersetzt; Cloud-Zugangsdaten folgen separat.

## Auftrag

Nach dem vom Nutzer bestätigten Update 0.1.2 → 0.1.3 beginnt die KI-Phase des [Masterplans](../MASTERPLAN.md). Ollama wird zuerst nutzbar; danach folgen Claude und OpenAI. Alle verwenden eine gemeinsame Oberfläche, freiwillige Einrichtung und eigene Verbindungen. Der Animationskern bleibt netzwerkfrei.

## Entscheidung

Das neue Root-Workspace-Crate `buddy-chat` enthält den anbieterunabhängigen Anfrage-/Antwortvertrag und eine begrenzte Gesprächssitzung. Es hängt weder vom Animationskern noch von GUI, Netzwerkbibliothek oder Betriebssystem ab und benötigt keine zusätzlichen externen Abhängigkeiten. Die Desktop-App bindet es beim folgenden Chat-/Ollama-PR ein. Dieser erste PR erzeugt noch keine neuen Menüeinträge oder Anbieterauswahl ohne funktionierenden Adapter.

`ChatProvider::stream` übersetzt später Antworten eines Hintergrundadapters in Textteile, Abschluss mit Grund/Verbrauch oder eine sichere Fehlerkategorie. Anbieter können tokenweise antworten; für Ollama berücksichtigt der spätere Decoder das dokumentierte [Chatformat](https://docs.ollama.com/api/chat) und [NDJSON-Streaming](https://docs.ollama.com/api/streaming). Rohprotokolle gehören in den jeweiligen Adapter. Interne Modellüberlegungen, Werkzeugaufrufe oder beliebige Animationsnamen sind kein Teil dieses sichtbaren Antwortvertrags.

`ChatSession` validiert explizit gesendete Eingaben und erstellt den Kontext aus fester Beni-Persona und vollständig abgeschlossenen Gesprächspaaren. Fehlerhafte oder abgebrochene Antworten bleiben ausserhalb des nächsten Kontextes. Es gibt nur eine aktive Anfrage je Sitzung. Eine fortlaufende Anfrage-ID verhindert, dass ein später alter Worker einen neuen Chat verändert. Leeren und Verwerfen der Sitzung signalisieren Abbruch; die IDs werden beim Leeren nicht wiederverwendet.

### Grenzen

- Eingabe: höchstens 4 KiB UTF-8, auch vor dem Abschneiden äusserer Leerzeichen. Leere Eingaben und Steuerzeichen ausser Zeilenumbruch/Tab werden abgewiesen.
- Kontext inklusive Persona und aktueller Frage: höchstens 32 KiB UTF-8; höchstens zehn abgeschlossene Gesprächspaare. Alte Paare werden als Ganzes weggelassen. Bytes sind keine Tokens; ein anbieterabhängiger Kontext-/Tokencheck bleibt Aufgabe des Adapters.
- Antwort: höchstens 16 KiB UTF-8, höchstens 1 KiB pro Textteil und 4096 Ereignisse pro Anfrage. Keine still abgeschnittenen UTF-8-Zeichen; eine Grenzverletzung beendet die Anfrage mit sicherer Fehlerkategorie.
- Ausgabetokenbudget: 512, später in die Anbieteroptionen übersetzen. Der Abschluss unterscheidet reguläres Ende von einem Anbieter-Tokenlimit.
- Gesamtdauer: 120 Sekunden. Nach dem ersten sichtbaren Text höchstens 30 Sekunden ohne weiteren Text. Leere Ereignisse verlängern diese Frist nicht. Zeit wird explizit übergeben.

Die Sitzung speichert Gespräche nur im Arbeitsspeicher, ohne Serialisierung oder Dateispeicherung. Debug-Ausgaben enthalten Grössen, Rollen und sichere Zustände; keine Modellnamen oder Gesprächsinhalte. `ChatError` hat keine freien Providertexte, URLs oder Zugangsdaten.

## Konsequenzen für die nächsten PRs

1. Eigener lesbarer Chat-Viewport und ein begrenzter, abbrechbarer Worker-Kanal. Die GUI pollt ohne Blockieren und prüft Fristen auch dann, wenn kein Ereignis ankommt. Eine Cancellation-Flag allein stoppt keinen blockierenden HTTP-Aufruf: der Transport braucht eigene Connect-/Read-/Gesamtfristen und muss die Verbindung bei Abbruch schliessen. EOF ohne Abschluss wird als Protokollfehler gemeldet.
2. R6/R8 zusammen als nutzbarer erster Ablauf: «Ohne KI / später» oder ein tatsächlich erreichbares lokales Ollama mit geeignetem vorhandenem Chatmodell. Keine automatische Modellinstallation und kein Cloud-Fallback. Ein neues Ollama-Modell wird nicht schon durch eine Installation dieser Grundlage heruntergeladen.
3. Beim Wechsel von Verbindung oder Modell laufende Anfrage abbrechen und einen neuen Kontext beginnen. Die Oberfläche zeigt vor dem Senden den verwendeten Kontext. Hook-/Statusverlauf wird nicht automatisch in Chatnachrichten kopiert.
4. Animationen steuert die Anwendung anhand des Zustands: Warten auf Antwort → vorhandenes «Denken», sichtbare Ausgabe → vorhandenes «Reden». Pause und Bewegungsreduktion haben weiterhin Vorrang. Antworten lösen keine Systemaktionen aus.
5. Sichere Zugangsdatenverwaltung vor einem Cloud-Adapter: OS-Schlüsselbund bzw. Sitzungsspeicher, keine Keys im eframe-Speicher oder in Fehlertexten. Danach separate Claude-/OpenAI-PRs und echte gezielte Abnahmen. Keine bezahlten Modellaufrufe in der CI.

## Abnahme dieses PRs

Positive und negative Offline-Tests prüfen Streaming, vollständige Kontextpaare, Unicode-/Bytegrenzen, alte Ereignisse, Abbruch, Fristen, leere/fehlerhafte Antworten, Kontextlöschung und redigierte Diagnostik. Ein Fake-Provider nutzt den tatsächlichen Vertrag in einem eigenen Thread mit begrenztem Channel. Das bestätigt die Schnittstelle, noch keine transportseitige Abbruchgarantie.

Echte Modellanfragen, Zugangsdatenverwaltung, Einrichtungsdialog, Chatfenster und Animationsanbindung sind NOT_RUN beziehungsweise noch nicht implementiert. Die Website bezeichnet KI weiterhin als geplant; die laufende Programme-App und veröffentlichte Release-Dateien werden durch diesen Entwicklungs-PR nicht verändert.
