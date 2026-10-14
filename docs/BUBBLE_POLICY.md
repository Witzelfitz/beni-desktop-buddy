# Sprechblasen und Gesten

`apps/desktop/src/bubble_policy.rs` entscheidet zentral und ohne Seiteneffekte, ob automatische Meldungen oder Gesprächsgesten zugelassen sind. `app.rs` übergibt Sichtbarkeit, «Nicht stören», offene Ansichten, Fehler und die anhand der expliziten Zeit berechnete Textgültigkeit. Einstellungen und Animationspause blockieren keine Meldungen.

| Situation | IPC | Timer / Erinnerung | Gesprächsgeste |
| --- | --- | --- | --- |
| Frei | Ja | Ja, nach IPC | Nein |
| Begrüssung / eigener Text | Darf ersetzen | Wartet | Nein |
| Aktive Statusmeldung | Wartet | Wartet | Nein |
| Gespräch offen | Wartet | Wartet | Ja, sobald kein transienter Text aktiv ist |
| Verlauf offen / Fehler aktiv | Wartet | Wartet | Nein |
| Versteckt / nicht sichtbar | Wartet | Wartet | Nein, wenn versteckt |
| «Nicht stören» | Wartet | Wartet | Weiterhin möglich |

Die Besitzerpriorität ist Fehler → Verlauf → Gespräch → gültiger Status → eigener Text → frei. Sie beschreibt die Freigabe für Meldungen und Gesten; die Gesprächsoberfläche und ein Fehlerhinweis können gleichzeitig sichtbar sein. Für Gesprächsgesten gilt die bisherige Prüfung «nicht versteckt»; die Meldungszustellung berücksichtigt zusätzlich die vom Fenster gemeldete Sichtbarkeit.

Nach Freigabe konsumiert `deliver_status` höchstens eine Meldung: zuerst IPC (mit dessen bestehender Prioritätswarteschlange), dann Timer, dann Erinnerungen. Ablaufzeit, Mengenbegrenzung und Abbruch bleiben in den zuständigen Modulen. Eine abgelaufene IPC-Meldung blockiert keine lokale Meldung. Der Verlauf bewahrt Meldungen unabhängig von ihrer aktuellen Anzeigefähigkeit.

Beim Schliessen des Gesprächs wird die Anfrage abgebrochen und der vorige Animationszustand unter Erhalt der aktuellen Pause wiederhergestellt. Eine gültige Statusmeldung wird erst nach Ablauf oder bewusstem Schliessen ersetzt. Die bestehenden TTL- und Datenschutzregeln ändern sich nicht.

Die Prüfungen decken alle 128 Kombinationen der Entscheidungsmerkmale, die Besitzerpriorität und den Ablauf «Gespräch denkt → Timer und Erinnerung werden fällig → IPC kommt → Escape → IPC → Timer → Erinnerung» ab. Bestehende Tests prüfen zusätzlich Fehler, Unsichtbarkeit, stille Phasen, Verlauf, Textablauf und Einstellungen. Eine echte Plattformabnahme ersetzt dies nicht.
