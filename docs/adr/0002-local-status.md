# ADR 0002 · Lokale Statusmeldungen

Status: akzeptiert für das nächste Paket nach PR #2.

Beni empfängt kurze Meldungen von ausdrücklich eingerichteten lokalen Programmen. Der Nutzer aktiviert «Statusmeldungen empfangen» in der App; die Standardeinstellung ist aus. Die Einstellung wird lokal gespeichert. Es gibt keinen TCP-Port, Autostart, System-Notification-Reader oder Zugriff auf Dateien, Bildschirm, Zwischenablage oder Mikrofon durch Meldungen.

## Transport

Ein eigenes `buddy-ipc`-Crate hält Betriebssystem und Transport aus `buddy-core` heraus. `interprocess` verwendet Unix-Sockets auf macOS/Linux und Named Pipes auf Windows. Ein einzelner Hintergrundthread mit Tokio verarbeitet begrenzte Verbindungen; die Oberfläche wird bei angenommenen Meldungen geweckt. Das Abschalten bricht auch eine unvollständige Verbindung ab und räumt den Listener auf.

- macOS: `~/Library/Application Support/io.witzelfitz.beni-buddy/ipc/buddy.sock`.
- Linux: `$XDG_RUNTIME_DIR/beni-buddy/ipc/buddy.sock`; ohne Laufzeitverzeichnis unter dem lokalen Datenverzeichnis, üblicherweise `~/.local/share/beni-buddy/ipc/buddy.sock`.
- Unix: eigenes IPC-Verzeichnis 0700, Socket 0600, Kontrolle der Peer-UID. Eine gehaltene Dateisperre verhindert mehrere Listener. Nur eigene, entsprechend geschützte verwaiste Socket-Dateien dürfen ersetzt werden; reguläre Dateien und Symlinks werden abgelehnt.
- Windows: `\\.\pipe\io.witzelfitz.beni-buddy.<Benutzer-SID>`. Eine geschützte DACL erlaubt nur diesen Benutzer und verweigert Netzwerk-Anmeldungen. Die Bibliothek setzt ausserdem `PIPE_REJECT_REMOTE_CLIENTS` und beim ersten Listener `FILE_FLAG_FIRST_PIPE_INSTANCE`.

## Vertrag v1

Genau eine UTF-8-JSON-Zeile pro Verbindung, einschliesslich Zeilenende höchstens 4096 Bytes:

```json
{"v":1,"text":"Build grün","state":"daumen","source":"shell","priority":"normal","ttl_s":30}
```

`v` und `text` sind Pflichtfelder. `state` ist optional, `source` standardmässig `cli`, `priority` standardmässig `normal`, `ttl_s` standardmässig 30. Unbekannte Felder, Versionen, Animationen oder Prioritäten werden abgelehnt. Aliase werden auf kanonische Animationen normalisiert. Erlaubt sind daumen, jubeln, stopp, traurig, nachdenken, ueberrascht, idee, reden und hallo. Steuerzeichen werden entfernt, Text danach auf 1–280 Unicode-Zeichen geprüft. Quellen enthalten 1–48 ASCII-Zeichen aus Buchstaben, Zahlen und `._-`. TTL: 1–120 Sekunden ab Annahme, einschliesslich Wartezeit und Anzeige.

Die Bestätigung ist `{"v":1,"ok":true}` oder `{"v":1,"ok":false,"error":"…"}`. Erfolg bedeutet angenommen, nicht bereits angezeigt. Bei einem Timeout kann die Meldung bereits angenommen worden sein; die CLI wiederholt sie nicht automatisch. Server-I/O endet nach einer Sekunde, der gesamte CLI-Austausch nach 1,5 Sekunden.

## Warteschlange und Gesten

Höchstens 20 Meldungen, 5 angenommene Meldungen pro Quelle und 20 insgesamt in 10 Sekunden. Die zusätzliche Gesamtgrenze verhindert Umgehen durch wechselnde Quellennamen. Abgelaufene Einträge verschwinden. Hohe Priorität kommt vor normalen wartenden Einträgen; innerhalb einer Priorität bleibt die Reihenfolge erhalten. Die gerade sichtbare Nachricht wird nicht unterbrochen. Ausgeblendet, minimiert, verdeckt oder bei geöffnetem Menü wird die Anzeige zurückgestellt; die TTL läuft weiter.

`buddy-core` erhält Zeit ausdrücklich in monotonen Millisekunden. Explizites `state` hat Vorrang vor dem `RuleClassifier`; andernfalls fällt dieser auf `reden` zurück. Ein späterer Klassifikator kann dasselbe Trait nutzen. In diesem Paket gibt es kein Modell und keine externen Aktionen aus Meldungsinhalt.

Referenzen: [Interprocess](https://docs.rs/interprocess/2.4.4/interprocess/local_socket/index.html), [Windows-Listener-Sicherheit](https://docs.rs/interprocess/2.4.4/interprocess/os/windows/local_socket/trait.ListenerOptionsExt.html), [Tokio](https://docs.rs/tokio/1.53.2/tokio/).
