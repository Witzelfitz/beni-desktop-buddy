# Ressourcenmessung

## Zeichenbudget

Entwicklung nach 0.1.7: Die normale Warteschleife hält ihre Originalpose und fordert nur eine Aktualisierung pro Sekunde an. Nach jeweils 30 Sekunden ungestörter Ruhe folgt eine vorhandene SDK-Idle-Geste mit bis zu 15 Bildern pro Sekunde; Tempo skaliert auch diese Wartezeit. Der öffentliche Installer 0.1.7 verwendet noch das bisherige animierte Warten. Manuell oder durch Meldungen ausgelöste Gesten und «Denken» im Meldungsverlauf behalten bis zu 30 Bilder pro Sekunde. Bei Pause oder «Weniger Bewegung» ist die Pose statisch: Es bleibt eine Aktualisierung pro Sekunde für Fensterposition, Verlaufsalter und das Ende einzelner Gesten. Sichtbare Blasen und Fehler planen zusätzlich ihren Ablaufzeitpunkt ein. Eingaben, IPC, Tray und Hotkeys können unabhängig davon sofort neu zeichnen lassen; egui darf auch für Hover, Texteingabe und Scrollen zusätzliche Bilder anfordern.

Ausgeblendete, minimierte und als verdeckt gemeldete Fenster erhalten nur den Sekundentakt für die App-Logik. eframe führt bei unsichtbaren Fenstern ohne sichtbares Zusatzfenster keine Zeichenphase aus. Geöffnete Einstellungen bleiben auch bei ausgeblendetem Buddy bedienbar; der Immediate-Viewport erhält dafür weiter UI-Durchläufe. Die App beendet diesen Takt bewusst nicht, damit sie nach Monitorwechseln wieder erreichbar bleibt.

## Wiederholen auf macOS

Die Hilfsdatei ist ein manuell gestartetes Messwerkzeug und gehört nicht zur Anwendung. Sie liest ausschliesslich den angegebenen Prozess; Beni selbst sammelt und versendet keine Leistungsdaten.

```sh
clang -O2 -Wall -Wextra -Werror scripts/measure-process-macos.c -o /tmp/beni-process-usage
pgrep -x beni-desktop-buddy
# Die ausgegebene Prozess-ID einsetzen: zwei Messfenster à 30 Sekunden.
/tmp/beni-process-usage 12345 30 2 > /tmp/beni-usage.csv
```

- Release-Bundle verwenden, auf dem Bildschirm sichtbar lassen, die gleiche Fenstergrösse und dasselbe Tempo wählen. Menü und Bedienleiste zuklappen.
- «Nicht stören» für eine ruhige Basismessung einschalten; Empfang kann aktiviert bleiben. Nach dem Start mindestens zwölf Sekunden warten, bis die Begrüssungsblase weg ist.
- «Weniger Bewegung», Pause, normal animiertes «warten» und Ausblenden getrennt messen. Pause hält die aktuelle Pose fest; vorher immer denselben Zustand wählen.
- Während der Messfenster nicht kompilieren, profilen oder die Oberfläche automatisiert abfragen. Den Mac wach halten. Einstellungen anschliessend zurücksetzen.
- Für längere Messungen beispielsweise `30 30` verwenden (15 Minuten). Dieses Dokument bestätigt nur tatsächlich ausgeführte Messungen.

`cpu_pct_one_core` ist CPU-Zeit geteilt durch reale Dauer: 100 % bedeutet einen vollständig beschäftigten CPU-Kern. Die Nutzer- und Systemzeit von `proc_pid_rusage` werden wie in [Apples XNU-Referenztest](https://github.com/apple-oss-distributions/xnu/blob/main/tests/recount/recount_perf_tests.c) von Mach-Ticks in Nanosekunden umgerechnet. `wakeups_per_second` zählt Interrupt-Wakeups des Prozesses, keine Bilder oder alle möglichen Aufweckereignisse. `footprint_mib` ist der physische Speicherbedarf am Ende des Messfensters, kein Durchschnitt oder Spitzenwert. Das Werkzeug verwirft einen Lauf bei Prozesswechsel oder einer um mehr als eine Sekunde verlängerten Messung, etwa durch Ruhezustand.

CPU-Zeit und Interrupt-Wakeups messen weder GPU-Auslastung noch Energieverbrauch. Weniger Zeichenanforderungen allein belegen keine bestimmte GPU-Einsparung. Ein einzelner Mac und kurze Messfenster ersetzen keine Langzeitprüfung, keine Akkumessung und keine native Windows-/Linux-Abnahme.

## Vergleich · 13. Oktober 2026

Apple M3 Max, macOS 26.6.2, Retina, Release-Bundle. Vorher: `main` mit PR #14 (`a1d410d`), nachher: dieser PR. «Klein» (300 × 360 Punkte), Tempo 0,98, Energie 0,70, gespiegelt aus, immer im Vordergrund, Durchklicken an, Empfang an, Hotkeys aus. «Nicht stören» während der Messungen an; keine offene Blase, Bedienleiste und Verlauf geschlossen. Pro Zustand ein frischer App-Start, zwölf Sekunden Vorlauf und zwei aufeinanderfolgende Messfenster. Keine Builds oder UI-Automation während dieser Fenster. Anschliessend wurden die ursprünglichen gespeicherten Einstellungen wiederhergestellt.

| Zustand | Stand | Fenster | Dauer, s | CPU, % eines Kerns | Interrupt-Wakeups/s | Speicher am Ende, MiB |
| --- | --- | --- | --- | --- | --- | --- |
| Weniger Bewegung | vorher | 1 | 30,002 | 0,9958 | 5,200 | 302,47 |
| Weniger Bewegung | vorher | 2 | 30,010 | 1,1216 | 5,265 | 273,45 |
| Weniger Bewegung | nachher | 1 | 30,010 | 0,4026 | 2,099 | 302,74 |
| Weniger Bewegung | nachher | 2 | 30,004 | 0,4153 | 2,066 | 302,74 |
| Animiertes Warten | vorher | 1 | 30,010 | 4,6895 | 19,893 | 273,30 |
| Animiertes Warten | vorher | 2 | 30,000 | 4,6434 | 20,400 | 273,34 |
| Animiertes Warten | nachher | 1 | 30,010 | 5,2341 | 19,827 | 272,58 |
| Animiertes Warten | nachher | 2 | 30,007 | 5,1796 | 20,395 | 272,61 |

Mit reduzierter Bewegung fällt das Mittel der CPU-Messungen von 1,0587 % auf 0,4090 % (rund 61 % weniger), die Interrupt-Wakeups von 5,23 auf 2,08 pro Sekunde (rund 60 % weniger). Das unveränderte animierte Warten liegt in diesen Läufen bei 4,67 % bzw. 5,21 % CPU und rund 20 Wakeups pro Sekunde; dafür ist keine CPU-Einsparung belegt. Pause nutzt denselben getesteten Zeichenpfad, wurde nativ hier aber nicht separat gemessen.

Der Speicherbedarf ist bei reduzierter Bewegung nachher am Ende des zweiten Fensters rund 29 MiB höher. Ursache und längerfristige Entwicklung sind mit diesen Messungen nicht geklärt; eine Speicheroptimierung ist damit nicht belegt. GPU-Auslastung, Akkulaufzeit, ein langer Idle-Lauf und native Windows-/Linux-Werte bleiben **NOT_RUN**.
