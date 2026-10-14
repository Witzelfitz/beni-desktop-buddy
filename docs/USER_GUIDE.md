# Beni bedienen

Die Installation und Updates findest du auf [der Downloadseite](https://mini.witzelfitz.io/download). [Mit Beni reden](CHAT.md) erklärt die Gesprächsblase und Ollama; [Integrationen](INTEGRATIONS.md) erklärt lokale Statusquellen.

| Aktion | Bedienung |
| --- | --- |
| Beni verschieben | Beni mit der linken Maustaste ziehen |
| Bedienleiste auf-/zuklappen | Beni anklicken; beim Start ist die Leiste eingeklappt |
| Bedienung ohne Maus | Bei fokussiertem Buddy-Fenster mit Tab zu Beni oder einem Bedienelement wechseln; Eingabe oder Leertaste aktiviert es |
| Beni anzeigen / ausblenden | «Beni» in der Menüleiste → «Beni anzeigen» / «Beni ausblenden» |
| Verlorenen Beni zurückholen | «Beni» in der Menüleiste → «Beni zurückholen» (Hauptbildschirm) |
| Einstellungen öffnen | «Beni» in der Menüleiste → «Einstellungen»; alternativ Beni anklicken → «Menü» |
| Begrüssen | Bedienleiste aufklappen → «Hallo!» |
| Pause/Fortsetzen | «Pause» / «Weiter» |
| Animation auswählen | «Menü» → Animation |
| Energie, Tempo, Spiegelung | «Menü» |
| Beni-Grösse | «Menü» → «Klein», «Mittel» oder «Gross»; die Auswahl bleibt nach Neustart erhalten |
| Globale Tastenkürzel | «Menü» → «Tastenkürzel» → Kombinationen wählen und «Globale Hotkeys aktivieren» |
| Weniger Bewegung | «Menü» → «Weniger Bewegung» |
| Hintergrund durchklickbar | Standardmässig aktiv; im Menü abschaltbar. Beni, sichtbare Bedienleiste und Sprechblasen bleiben anklickbar |
| Fokus-/Pausen-Timer | «Einstellungen» → «Fokus & Pause»: Minuten wählen, starten, unterbrechen, fortsetzen oder abbrechen |
| Lokale Erinnerungen | «Einstellungen» → «Erinnerungen»: Text und Wartezeit eingeben, planen oder entfernen |
| Eigene Sprechblase | Nachricht im Menü eingeben; lange Texte scrollen; Blase anklicken oder mit × schliessen |
| Status von lokalen Programmen | «Menü» → «Statusmeldungen empfangen», danach CLI `notify` oder `run` |
| Statusanzeigen pausieren | «Menü» oder Tray → «Nicht stören» |
| Verpasste Statusmeldungen lesen | «Menü» → «Verlauf öffnen» oder Tray → «Meldungsverlauf» |
| Vordergrunddarstellung | «Menü» → «Immer im Vordergrund» |
| Bedienleiste schliessen | Escape bei Beni oder Beni erneut anklicken |
| Einstellungen schliessen | «Schliessen», Titelleiste oder Escape im Einstellungsfenster |
| Beenden | «Beenden» im Fenster oder Tray; Alt/Option + Q bei fokussiertem Buddy- oder Einstellungsfenster |

Die Bedienleiste erscheint erst nach einem Klick auf Beni. Ein weiterer Klick oder Escape klappt sie wieder zu; das separate Einstellungsfenster bleibt offen. Benis Grösse und Position bleiben beim Auf-/Zuklappen gleich; aktive Statusmeldungen behalten Text, Quelle, Geste und Anzeigedauer. Ziehen klappt die Leiste nicht auf. Das Tray-Menü ist auch bei eingeklappter Bedienung verfügbar.

Die native Accessibility-Anbindung über [eframes AccessKit-Feature](https://docs.rs/eframe/0.36.2/eframe/#feature-flags) ist aktiviert. Beni wird als beschrifteter Button mit Auf-/Zuklappzustand dargestellt; die Sprechblase hat einen benannten Schliessen-Knopf, und das Nachrichtenfeld einen festen Namen. Tab, Eingabe und Leertaste bedienen auch die eingeklappte Leiste. Eine vollständige Abnahme mit VoiceOver, NVDA und Orca bleibt separat zu prüfen.


«Klein» verwendet eine Zeichenfläche von 300 × 360 Punkten, «Mittel» die bisherige Grösse von 340 × 440 und «Gross» 420 × 560. Schrift und Bedienelemente behalten ihre Grösse. Die Einstellungen haben ein unabhängiges Fenster; seine Grösse hängt nicht von Beni ab. Aktive Meldungen behalten beim Wechsel Text, Quelle, Geste und Anzeigedauer. Fehlende oder beschädigte Grösseneinstellungen fallen auf «Mittel» zurück. Nach einer Vergrösserung wird die Fensterposition erneut geprüft; unter Wayland bestimmt weiterhin der Compositor die Platzierung.

«Einstellungen» im Menüleistensymbol und «Menü» bei Beni öffnen dasselbe verschiebbare, skalierbare Fenster mit normaler Titelleiste. Es startet mit 520 × 680 Punkten, hat eine scrollbar dargestellte Bedienung und einen stets erreichbaren «Schliessen»-Knopf. Es ist vollständig anklickbar, auch wenn transparente Bereiche des Buddys durchklickbar sind. Schliessen beendet nur dieses Fenster. Beni bleibt sichtbar; Statusmeldungen werden während der Einstellungen weiter angezeigt. Escape beendet zuerst eine aktive Eingabe und schliesst danach das Fenster. Alt/Option + Q beendet die ganze App. Zuklappen der Buddy-Leiste schliesst das separate Fenster nicht.

Unter «Fokus & Pause» startet ein lokaler Timer mit frei wählbaren 1–180 Minuten (Vorgabe: 25 Minuten Fokus, 5 Minuten Pause). Es läuft immer eine Phase; die nächste startest du selbst. «Timer unterbrechen» hält die Restzeit, «Timer fortsetzen» zählt weiter, «Timer abbrechen» verwirft den Lauf. Benis Animationspause, Tempo und reduzierte Bewegung beeinflussen den Timer nicht. Die Restzeit ist im Einstellungsfenster auch bei zugeklapptem Abschnitt sichtbar.

Am Ende zeigt Beni eine Meldung mit der vorhandenen Geste «Daumen hoch» beziehungsweise «Idee». Der Timer läuft auch mit geschlossenen Einstellungen oder ausgeblendetem Buddy und benötigt keine IPC-Freigabe. «Nicht stören», Nachlesen, Fehler und laufende Sprechblasen halten die Abschlussanzeige zurück; empfangene IPC-Meldungen haben Vorrang. Nach spätestens zwei Minuten verfällt eine wartende Timer-Blase. Der Abschluss bleibt im Sitzungsverlauf unter Quelle `timer` nachlesbar. Start eines neuen Timers oder «Timer zurücksetzen» entfernt eine noch wartende Timer-Blase. Nur die gewählten Minuten werden gespeichert: App-Beenden stoppt den Lauf, ein Neustart startet keinen Timer.

Unter «Erinnerungen» kannst du Beni eigene Texte nach 1–1440 Minuten anzeigen lassen. Bis zu zehn geplante oder auf Anzeige wartende Erinnerungen sind gleichzeitig möglich. Der Text hat höchstens 240 Zeichen; Steuerzeichen werden entfernt. Die Liste zeigt Restzeit beziehungsweise «Fällig · wartet auf Anzeige». «Entfernen» löscht eine geplante Erinnerung oder ihre wartende Anzeige. Bereits fällige Einträge im Meldungsverlauf bleiben erhalten.

Erinnerungen laufen parallel zum Fokus-Timer, auch bei geschlossenen Einstellungen oder verborgenem Buddy, ohne IPC-Freigabe. Beni verwendet dafür die vorhandene Geste «Idee» und die Quelle `erinnerung`. IPC-Meldungen und Fokus-/Pausen-Abschlüsse haben Vorrang; vorhandene Blasen, Nicht stören, Nachlesen und Fehler halten die Anzeige zurück. Nach zwei Minuten verfällt eine wartende Blase; der Verlauf bleibt nachlesbar. Texte, Entwurf und Termine werden nicht gespeichert. App-Beenden verwirft geplante Erinnerungen; nach Neustart ist die Liste leer.

Das Tray-Menü bietet auch Pause/Fortsetzen und Einstellungen. Gespeicherte Positionen werden beim Start und nach Änderungen der Bildschirme geprüft; vollständig verlorene Fenster werden automatisch zurückgeholt. Auf macOS berücksichtigt dies Menüleiste, Dock und Retina-Punkte. Eine Fehlermeldung verschwindet nach sechs Sekunden oder bei einer erfolgreichen Aktion; bei einem Zeichenfehler bleibt eine bedienbare Ersatzfläche sichtbar.

Unter Linux braucht das Tray einen StatusNotifier-Host (KSNI, beispielsweise in KDE oder über eine passende GNOME-Erweiterung). Unter Wayland bestimmt der Compositor die Fensterposition; das Begrenzen und gezielte Zurückholen auf den Hauptbildschirm ist dort nicht garantiert. Wenn das Erstellen des Tray fehlschlägt, erhält das Fenster eine normale Titelleiste; auf macOS erscheint zusätzlich ein Dock-Eintrag.

Die Sprechblase passt ihre Höhe an, bietet bei langen Texten Scrollen und eine Anzeigedauer nach Lesezeit (4–30 Sekunden, bei Statusmeldungen zusätzlich durch TTL begrenzt). Sie zeigt Text ohne Audioausgabe. Einzelaktionen kehren nach einem Durchlauf zu «warten» zurück. Einstellungen werden lokal gespeichert. Idle zeichnet mit bis zu 15 fps, aktive Gesten mit bis zu 30 fps. Bei Pause und reduzierter Bewegung bleibt nur die Prüfung einmal pro Sekunde; Eingaben und Statusmeldungen wecken Beni direkt, sichtbare Texte bekommen zusätzlich ihren Ablaufzeitpunkt. Ausgeblendete oder verdeckte Fenster starten keine Animationsschleife. Messmethode und Grenzen stehen in [PERFORMANCE.md](PERFORMANCE.md).

«Systemeinstellung beachten» ist standardmässig aktiv: Beni reduziert Bewegung, wenn macOS, Windows oder ein unterstütztes Linux-Desktop-Portal dies vorgeben. Die bisherige Option «Weniger Bewegung» bleibt zusätzlich wirksam und wird durch das System niemals überschrieben. Wer die Systemvorgabe bewusst übergehen möchte, schaltet «Systemeinstellung beachten» aus. Änderungen werden während des Betriebs übernommen; ohne verfügbare Systemvorgabe gilt die manuelle Auswahl. Unter Linux hängt die Erkennung vom Portal und Desktop ab; ältere Portale ohne `reduced-motion` verwenden diesen Rückfall. [Plattformdetails und Abnahme](SYSTEM_MOTION.md).

Im Idle nutzt Beni ausschliesslich die importierten Mini-Beni-Animationen: «Warten» enthält bereits Atmen und Umschauen. Nach jeweils 18 Sekunden ungestörtem Warten wechseln sich die vorhandenen Einzelaktionen «Denken» und «Drehen» ab; danach geht es zurück zu «Warten». Tempo gilt auch für diese Wartezeit. Offene Bedienung, sichtbare Meldungen, Fehler, Pause und Bewegungsreduktion verhindern neue Idle-Gesten. Beim Nachlesen wird der Ablauf angehalten und danach fortgesetzt. Idle-Gesten bleiben bei bis zu 15 fps; es werden weder neue Bilder noch neue Bewegungsdaten erzeugt.

«Nicht stören» ist anfangs ausgeschaltet und wird über Neustarts gespeichert. Der Modus schliesst eine laufende Statusblase und pausiert neue Statusanzeigen aller Prioritäten. Beni bleibt sichtbar und bedienbar; eigene Nachrichten, Animationspause und Fehleranzeigen bleiben unabhängig. Ein sichtbarer Hinweis und der Haken im Tray zeigen den Modus an. Die lokale IPC bleibt bei eingeschaltetem Empfang erreichbar: Meldungen warten in der bestehenden begrenzten Warteschlange, ihre TTL läuft weiter. Nach dem Ausschalten zeigt Beni nur noch gültige Meldungen in der üblichen Reihenfolge. Ein CLI-Erfolg bedeutet weiterhin «angenommen», nicht «bereits angezeigt».

Der Meldungsverlauf zeigt die letzten 100 angenommenen Statusmeldungen sowie Timer-Abschlüsse und fällige Erinnerungen, neueste zuerst, mit Quelle, vergangener Zeit und Kennzeichnung hoher Priorität. Er erfasst auch Meldungen bei verstecktem Beni, bei «Nicht stören» und nach Ablauf ihrer Anzeige-TTL. Während du nachliest, bleibt Beni über dem scrollbar dargestellten Verlauf sichtbar und zeigt «Denken» (`nachdenken`) in einer Schleife. Automatische Statusblasen warten solange; die TTL läuft weiter. Nach Schliessen, Escape oder Zuklappen setzt Beni seine vorherige Animation fort. Pause und reduzierte Bewegung bleiben wirksam. Menü, Begrüssung und eigene Nachrichten beenden die Leseansicht für die gewählte Aktion.

«Leeren» entfernt den Verlauf, einschliesslich noch gepufferter Empfangseinträge. Die bestehende Anzeige-Warteschlange bleibt unabhängig davon. Ausschalten des Empfangs behält den bisherigen Verlauf; nach Beenden von Beni ist er weg. Meldungsinhalte und die Leseansicht werden nicht auf der Festplatte gespeichert. Ungültige oder wegen Limits abgelehnte Meldungen kommen nicht in den Verlauf.

Transparente Flächen leiten Mausklicks an das darunterliegende Fenster weiter. Die Eingabegeometrie folgt der gezeichneten Beni-Silhouette, Sprechblase und Bedienleiste; das eigenständige Einstellungsfenster hat eine normale volle Eingabefläche. Auf macOS stellen passive Mausbewegungsereignisse das Buddy-Fenster vor dem nächsten Klick passend um; Tastaturereignisse werden dafür nicht überwacht. Windows verwendet Fensterregionen, Linux X11 Input Shapes und Wayland Oberflächen-Eingaberegionen. Ohne verfügbares Tray oder bei nativen Fehlern bleibt die normale Fensterbedienung erhalten. Autostart bleibt offen.

Globale Hotkeys sind bei neuen Installationen ausgeschaltet. Die Vorschläge sind `Ctrl+Alt+B` zum Anzeigen und Zurückholen auf den Hauptbildschirm sowie `Ctrl+Alt+P` zum Pausieren/Fortsetzen der Animation. Du kannst die Texte ändern und mit «Hotkeys übernehmen» anwenden; ohne Aktivierung werden nur die Kombinationen gespeichert. Kombinationen verwenden beispielsweise `Ctrl`, `Alt`/`Option`, `Shift`, `Cmd`/`Super` und eine Taste. `Super` bezeichnet die Windows-/Cmd-Taste; Buchstaben verwenden US-Tastenpositionen (Y/Z bei QWERTZ vertauscht). Einzelne Funktionstasten wie `F8` sind ebenfalls möglich. Normale Tasten brauchen Ctrl, Alt/Option oder Cmd/Super; doppelte Kombinationen und Medientasten werden abgelehnt.

Bei einer belegten neuen Kombination bleiben die bisherigen Hotkeys aktiv. Abschalten gibt die Registrierungen frei; ein gehaltenes Kürzel löst nur einmal aus. Auswahl und Aktivierung bleiben nach Neustart erhalten. [global-hotkey 0.8.0](https://docs.rs/global-hotkey/0.8.0/global_hotkey/) liefert die Ereignisse auf macOS, Windows und Linux/X11. Auf macOS fordert Beni zusätzlich exklusive Carbon-Registrierungen an, damit eine bereits von einer anderen App belegte Kombination als Konflikt erkannt wird. Unter Wayland ist die Funktion deaktiviert und im Menü entsprechend beschriftet; auch XWayland in einer Wayland-Sitzung wird nicht als systemweite Unterstützung ausgegeben. Die App empfängt nur Ereignisse ihrer registrierten Kombinationen und protokolliert keine Tastatureingaben.
