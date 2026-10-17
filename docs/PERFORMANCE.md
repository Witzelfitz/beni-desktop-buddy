# Ressourcenmessung

## Zeichenbudget

Seit 0.1.8: Die normale Warteschleife hält ihre Originalpose und fordert nur eine Aktualisierung pro Sekunde an. Nach jeweils 30 Sekunden ungestörter Ruhe folgt eine vorhandene SDK-Idle-Geste mit bis zu 15 Bildern pro Sekunde; Tempo skaliert auch diese Wartezeit. Manuell oder durch Meldungen ausgelöste Gesten und «Denken» im Meldungsverlauf behalten bis zu 30 Bilder pro Sekunde. Bei Pause oder «Weniger Bewegung» ist die Pose statisch: Es bleibt eine Aktualisierung pro Sekunde für Fensterposition, Verlaufsalter und das Ende einzelner Gesten. Sichtbare Blasen und Fehler planen zusätzlich ihren Ablaufzeitpunkt ein. Eingaben, IPC, Tray und Hotkeys können unabhängig davon sofort neu zeichnen lassen; egui darf auch für Hover, Texteingabe und Scrollen zusätzliche Bilder anfordern.

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


## Vergleich · 17. Oktober 2026, Entwicklung nach 0.1.7

Code des optimierten Laufs: `eaa4c5b885923da0332bb2310f5972b727fda9e1`. Basis: unveränderte Programmlogik des veröffentlichten 0.1.7/Build 8. Apple M3 Max, macOS 26.6.2, Netzbetrieb. Zwei nacheinander gestartete minimale, ad hoc signierte Testbundles mit identischer Bundle-Konfiguration, ohne Sparkle; eigene RON-Dateien unter `/private/tmp/beni-idle-benchmark`. Die Programme-App und ihre Hooks bleiben unverändert. Klein (300 × 360 Punkte), Tempo 1,0, Energie 0,7, Bewegung erlaubt, System-Bewegungsvorgabe für den Vergleich aus, Durchklicken und Vordergrund an, Empfang aus, «Nicht stören» an. Zwölf Sekunden Vorlauf, danach drei Messfenster à zehn Sekunden. Sichtbares eigenes Fenster an jeder Messgrenze geprüft. Keine Builds oder CPU-Profiler während der Messfenster; der Desktop wurde währenddessen weiter benutzt.

| Zustand | Fenster | Dauer, s | CPU, % eines Kerns | Interrupt-Wakeups/s | Speicher am Ende, MiB |
| --- | --- | --- | --- | --- | --- |
| 0.1.7 | 1 | 10.005 | 6.8165 | 19.890 | 303.16 |
| 0.1.7 | 2 | 10.005 | 9.8595 | 32.284 | 273.09 |
| 0.1.7 | 3 | 10.005 | 6.1278 | 19.390 | 273.09 |
| Optimierter Idle | 1 | 10.005 | 5.2096 | 27.086 | 303.69 |
| Optimierter Idle | 2 | 10.005 | 2.5712 | 20.290 | 303.94 |
| Optimierter Idle | 3 | 10.004 | 3.3750 | 26.489 | 303.97 |

In diesem kurzen interaktiven Lauf beträgt der mittlere CPU-Wert 7.60 % gegenüber 3.72 %. Die Interrupt-Wakeups liegen bei 23.85/s gegenüber 24.62/s; dafür ist **keine Einsparung belegt**. Eingaben und passive Mausereignisse wecken die native App unabhängig vom Animationsbudget. Die starken Schwankungen innerhalb und zwischen den Versuchen erlauben keine allgemeine Prozentangabe für CPU-, GPU- oder Akkuersparnis. Auch eine Speicherersparnis ist nicht belegt.

Der erste Versuch mit zwei 30-Sekunden-Fenstern pro Version wird nicht als gültiger Paarvergleich verwendet: Am Ende des optimierten Laufs war dessen Fenster nicht mehr auf dem sichtbaren Schreibtisch. Ein vorheriger Versuch ohne vollständigen Bundle-Rahmen konnte ebenfalls kein sichtbares Fenster bestätigen und wurde vor der Messung beendet. Diese Werte fliessen nicht in die Tabelle ein.

Die deterministischen UI-Tests bestätigen unabhängig davon: originale Pose und Klickfläche bleiben über die Ruhephase unverändert; reguläre Zeichenanforderung einmal pro Sekunde statt bisher ungefähr 15-mal. Nach 30 Sekunden läuft eine vorhandene Geste mit 15 fps, aktive Nutzer-/Statusgesten behalten 30 fps. Chat/Verlauf und Pause behalten ihre eigene Wiedergabe. Tray-Werte werden nur bei Abweichung geschrieben; unveränderte macOS-Klickflächen werden nicht erneut kopiert.

Native App-Starts und sichtbare Testfenster: **PASS**. Lange ruhige Messung ohne Mausbedienung, GPU-/Akkumessung, manuelle vollständige GUI-Abnahme und Windows-/Linux-Hardware: **NOT_RUN**. Die neue Installer-Abnahme ist unten separat dokumentiert. Der vorhandene gemeinsame Sekundentakt für Fenster-Rückholung, Timer, Erinnerungen und Systempräferenzen bleibt bewusst erhalten. Nächster Messschritt ist der getrennte Vergleich ohne Mausbewegung und während normaler Mausbedienung.


Der oben gemessene sparsamere Idle ist als 0.1.8/Build 9 veröffentlicht. Pakete und native Release-Abnahme sind in [TEST_MATRIX.md](TEST_MATRIX.md) dokumentiert. Die offenen GPU-/Akku-/Langzeitmessungen werden durch die Veröffentlichung nicht als bestanden gewertet.
