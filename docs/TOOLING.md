# Prüfeinstieg und Toolchains

Rust bleibt die Anwendungssprache. Python unterstützt Release-Manifeste, Signaturen, Installerprüfungen und Grafikprovenienz. Diese Werkzeuge laufen ausserhalb der Anwendung; eine Python-Umgebung ist keine Voraussetzung für Benis Chat oder die Rust-CLI. Die optionale Linux-Menüleistenintegration hat ihren eigenen dokumentierten Python-Bedarf.

Wir behalten die vorhandenen, getesteten Werkzeuge. Ein GitHub-Sprachanteil belegt weder Laufzeitkosten noch einen Wartungsfehler. Ein vollständiger `cargo xtask`-Port wird erst geplant, wenn er einen konkreten Aufwand reduziert und Schema-, Signatur- und Packagingprüfungen gleichwertig ersetzt. Eine zweite parallele Implementierung dieser Regeln wird vermieden.

## Einmal einrichten

Rust ist durch `rust-toolchain.toml` fixiert. Für die Release-Prüfungen empfiehlt sich dieselbe Python-Version wie in der Release-CI, derzeit 3.13; mindestens 3.11 ist erforderlich. Die Paketversionen stehen vollständig in `scripts/requirements-release.txt`.

```sh
python3.13 -m venv .venv-release
.venv-release/bin/python -m pip install -r scripts/requirements-release.txt
```

Der Prüfeinstieg installiert nichts selbst und ändert keine globale Python-Umgebung.

```sh
scripts/check.sh            # Kern, CLI, Assets und Release-Regeln
scripts/check.sh --desktop  # zusätzlich separate GUI-Prüfungen und Build
scripts/check.sh --help
```

Alle Python-Prüfungen verwenden denselben Interpreter: zuerst `BENI_RELEASE_PYTHON`, dann eine vorhandene ausführbare `.venv-release/bin/python`, sonst `python3`. Die Vorprüfung bestätigt Version und exakt festgelegte Pakete, bevor Cargo startet. Fehlende oder abweichende Pakete, ein ungültiger Interpreter und unbekannte Argumente führen zum Abbruch. Ein vorhandenes eigenes Release-Venv kann über einen absoluten Pfad angegeben werden:

```sh
BENI_RELEASE_PYTHON=/absoluter/pfad/bin/python scripts/check.sh --desktop
```

`python3 scripts/check-tooling.py` prüft Interpreterwahl, frühzeitigen Abbruch und die Trennung von Kern- und GUI-Befehlen mit isolierten Ersatzprogrammen; dabei werden keine Builds gestartet. Native GUI-Abnahmen, echte Modelltests und Linux-Compositor-Prüfungen bleiben separate, ausdrücklich dokumentierte Schritte. Ein erfolgreicher Kernlauf bestätigt weiterhin keinen GUI-Build.
