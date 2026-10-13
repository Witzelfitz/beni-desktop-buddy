# Beni mit lokalen Programmen verbinden

Beni starten und im Menü **«Statusmeldungen empfangen» einschalten**. Die Einstellung ist bei einer neuen Installation aus. Danach können lokale Programme deines Benutzers Meldungen senden; Beni liest keine Mitteilungszentrale aus. Ausblenden oder Minimieren hält die Anzeige zurück. «Animation pausieren» friert nur die Bewegung ein.

**«Nicht stören»** im Menü oder Tray hält Statusanzeigen zurück, während Beni sichtbar bleibt. Der Modus wird gespeichert und schliesst eine laufende Statusblase. Die CLI und Hooks können bei aktiviertem Empfang weiter Meldungen senden; «angenommen» bestätigt die Aufnahme in die begrenzte Warteschlange. Die TTL läuft weiter. Nach dem Ausschalten erscheinen nur noch gültige Meldungen, auch hohe Priorität übergeht «Nicht stören» nicht.

Verpasste Meldungen findest du unter **«Verlauf öffnen»** im Menü oder **«Meldungsverlauf»** im Tray. Dort bleiben die letzten 100 angenommenen Meldungen bis zum Leeren oder Beenden nachlesbar, auch nach Ablauf ihrer TTL. Während des Nachlesens denkt Beni; neue Statusblasen warten, ihre TTL läuft weiter. Der Verlauf bleibt ausschliesslich im Arbeitsspeicher.

## CLI finden

Die Download-Pakete enthalten Desktop-App, `beni-buddy-cli`, Wrapper und Hook-Vorlage. Im macOS-Bundle liegt die CLI unter `Beni Buddy.app/Contents/MacOS/beni-buddy-cli`, die Vorlagen unter `Contents/Resources/integrations`. Unter Windows/Linux liegen CLI und Desktopprogramm nebeneinander.

Aus dem Repository lässt sich die CLI auch bauen oder optional in den Cargo-Binärpfad installieren:

```sh
cargo build --release --locked -p buddy-cli
# Optional, falls du beni-buddy-cli direkt im PATH haben möchtest:
cargo install --path crates/buddy-cli --locked
```

## macOS und Linux

`BENI_CLI` kann auf einen absoluten Pfad zeigen, falls die CLI nicht im PATH liegt. Für das macOS-Bundle beispielsweise:

```sh
export BENI_CLI="/voller/Pfad/Beni Buddy.app/Contents/MacOS/beni-buddy-cli"
"$BENI_CLI" notify "Build grün" --state daumen --source shell
"$BENI_CLI" notify "Prüfe Änderungen" --source shell
"$BENI_CLI" run -- cargo test
bash integrations/beni-run.sh cargo test
```

`run` zeigt den Start und danach Erfolg oder Fehler. Ausgabe, Argumentgrenzen und Exitcode des Programms bleiben erhalten. Eine nicht erreichbare Beni-App verhindert die Ausführung nicht. Übergebe ein Programm und Argumente; Shell-Ausdrücke nur bewusst über eine Shell wie `bash -lc` ausführen.

Für ein Linux-Service kann eine bestehende **User-Unit** optional diese Zeile erhalten (Pfad anpassen):

```ini
ExecStartPost=-/voller/Pfad/beni-buddy-cli notify "Dienst gestartet" --state daumen --source systemd
```

Die Vorlage richtet keinen Dienst und keinen Autostart ein. Die User-Unit muss zum selben Benutzer und Desktop passen. Unter Wayland bestimmt der Compositor die Fensterposition; ein StatusNotifier-Host ist für das Tray nötig.

## Windows / PowerShell

```powershell
$env:BENI_CLI = 'C:\voller Pfad\beni-buddy-cli.exe'
& $env:BENI_CLI notify 'Build grün' --state daumen --source powershell
& $env:BENI_CLI notify 'Prüfe Änderungen' --source powershell
& $env:BENI_CLI run -- cargo test
.\integrations\beni-run.ps1 cargo test
```

Der PowerShell-Wrapper ist optional; der direkte `run`-Befehl braucht keine Script-Ausführungsrichtlinie. Eine bestehende Aufgabe der Windows-Aufgabenplanung kann die CLI oder den Wrapper aufrufen. Für ein sichtbares Beni-Fenster muss sie unter deinem Benutzer in deiner angemeldeten Desktop-Sitzung laufen. Es werden keine Aufgaben angelegt.

## macOS-Kurzbefehle

In einem eigenen Kurzbefehl die Aktion **«Shell-Script ausführen»** mit absolutem CLI-Pfad verwenden:

```sh
"/voller/Pfad/Beni Buddy.app/Contents/MacOS/beni-buddy-cli" notify "Zeit für eine Pause" --state hallo --source shortcuts
```

Für den Status eines GitHub-Workflows, mit eingerichtetem `gh`, beispielsweise:

```sh
"$BENI_CLI" run -- gh run watch RUN_ID --exit-status
```

`gh` verbindet sich mit GitHub; Beni selbst verwendet ausschliesslich lokale IPC.

## Claude Code

Die optionale Vorlage unter `integrations/claude-code/` benötigt Python 3 und die CLI im PATH oder in `BENI_CLI`. Übernimm die `hooks` aus `settings.example.json` in deine bestehende projektbezogene `.claude/settings.json`, ohne andere Einstellungen zu ersetzen. Passe den absoluten Script-Pfad an. Auf Windows verwende `python` statt `python3`, falls dies der installierte Python-Befehl ist, und `/` in Pfaden oder korrekt maskierte Backslashes.

- `Stop` → «Claude Code ist fertig» / daumen.
- `Notification` bei Freigabe oder benötigter Eingabe → «Claude Code braucht dich» / ueberrascht, hohe Priorität.
- Rekursive Stop-Hooks, reine Idle-Hinweise und erfolgreiche Anmeldungen werden ignoriert.

Der Hook überträgt feste Statussätze, keine Prompts, Transkripte oder ursprünglichen Meldungstexte. Eine nicht verfügbare CLI oder ein Timeout stoppt die Coding-Sitzung nicht. Vorlage und Ereignisnamen wurden anhand der [offiziellen Claude-Code-Hook-Referenz](https://code.claude.com/docs/en/hooks) geprüft; die Registrierung in deiner realen Claude-Code-Sitzung bleibt ein eigener Einrichtungsschritt.

## Codex-Sitzungen

Für lokale Codex-Sitzungen gibt es einen separaten Adapter unter `integrations/codex/`. Er nutzt die offiziell unterstützten Lifecycle-Hooks; er liest keine Transkripte und benötigt keinen OpenAI-API-Key. Python 3 und Beni müssen lokal verfügbar sein.

| Ereignis | Beni |
| --- | --- |
| Neue Anfrage (`UserPromptSubmit`) | «Codex arbeitet an deiner Anfrage» / nachdenken |
| Antwort abgeschlossen (`Stop`) | «Codex hat geantwortet» / daumen |
| Freigabeanfrage (`PermissionRequest`) | «Codex braucht deine Freigabe» / ueberrascht, hohe Priorität |
| Unterbrechung (`Interrupt`) | «Codex wurde unterbrochen» / stopp |

Eine Quelle wie `codex-e3c59fd106` unterscheidet Sitzungen, ohne deren ID oder Chatinhalt weiterzugeben. Stop bedeutet Antwortende, nicht automatisch erfolgreiche Tests. Rekursive Stop-Hooks werden übersprungen. Der Hook gibt ausschliesslich `{}` zurück: keine Freigabeentscheidung, keine Fortsetzungsanweisung und keinen zusätzlichen Modellkontext. Eine nicht erreichbare Beni-App stoppt die Sitzung nicht.

### Einrichten

macOS/Linux, im Repository:

```sh
python3 integrations/codex/install-hook.py
```

Windows, im Repository:

```powershell
$env:BENI_CLI = 'C:\voller Pfad\beni-buddy-cli.exe'
python integrations/codex/install-hook.py
```

Der Installer kopiert den Adapter nach `$CODEX_HOME/hooks/beni-status.py` (standardmässig `~/.codex/hooks/beni-status.py`) und ergänzt die vier Ereignisse in `hooks.json`. Bestehende JSON-Hooks bleiben erhalten; bei Änderungen wird eine Sicherung angelegt. Mehrmaliges Ausführen erzeugt keine doppelten Einträge. Die konfigurierte Python-Datei muss dauerhaft vorhanden sein; mit `--python /absoluter/Pfad/python3` kannst du etwa einen stabilen Homebrew-Pfad wählen. Falls du inline `[hooks]` in `config.toml` verwendest, ergänze die Definitionen dort statt eine zweite Darstellung in derselben Ebene zu verwenden.

**Letzter Schritt in Codex:** Im interaktiven CLI `/hooks` öffnen und die vier Beni-Definitionen prüfen und vertrauen. Codex verlangt dies für neue nicht verwaltete Hooks und überspringt sie bis dahin. Danach eine neue lokale Sitzung starten oder die bestehende Sitzung neu laden. Der Installer verändert keine Trust-Datenbank und verwendet keinen Bypass.

Die Beispieldefinition in `hooks.example.json` lässt sich auch manuell mit angepassten absoluten Pfaden übernehmen. Der Adapter sucht die CLI über `BENI_CLI`, danach im PATH und schliesslich unter `~/.cargo/bin`, damit er auch bei eingeschränktem GUI-PATH funktioniert. Beni muss laufen und «Statusmeldungen empfangen» eingeschaltet sein.

Diese lokale Einrichtung gilt nicht automatisch für cloud-orchestrierte Work-/Codex-Sitzungen. Dort hängen Hooks vom jeweiligen Laufzeitmodus und gegebenenfalls verwalteten Enterprise-Einstellungen ab. Referenz: [offizielle OpenAI-Hook-Dokumentation](https://learn.chatgpt.com/docs/hooks).

## Optionen und Fehlercodes

`notify` unterstützt `--state`, `--source`, `--priority normal|high` und `--ttl 1..120`. Ohne `--state` wählen einfache deutsche/englische Regeln die Geste. Unsichere Zuordnung fällt auf reden zurück. Die Warteschlange hält höchstens 20 Nachrichten und priorisiert hohe Meldungen, ohne die gerade sichtbare zu unterbrechen. TTL gilt ab Annahme; alte Meldungen werden verworfen. Text: höchstens 280 Unicode-Zeichen.

| Exitcode von notify | Bedeutung |
| --- | --- |
| 0 | Angenommen, möglicherweise noch in der Warteschlange |
| 2 | Ungültige Argumente oder Meldung |
| 3 | Beni nicht erreichbar oder Endpunkt nicht sicher |
| 4 | Vom Listener abgelehnt, beispielsweise Überlastung |
| 5 | Timeout; die Meldung kann bereits angenommen worden sein |

Für Tests und eigene CLI-Umgebungen gibt es `--endpoint` oder `BENI_IPC_ENDPOINT`; die GUI verwendet weiterhin ihren festen Endpunkt. `run` reicht den Exitcode des gestarteten Programms weiter, bei nicht startbarem Programm 127. `BENI_CLI` wählt nur die CLI für die Wrapper und den Hook.

Transport und Grenzen: [ADR 0002](adr/0002-local-status.md). Die Meldungen werden nicht in einen Verlauf oder eine Datei geschrieben. Integrationen führen nur das ausdrücklich übergebene Programm aus; Meldungen können selbst keine Systemaktionen auslösen.
