# Windows- und Linux-Installer

Die Installer werden mit dem manuellen Workflow **Windows and Linux installers** gebaut. Er veröffentlicht nichts. Erst ein vollständig erfolgreicher Lauf mit Paket, Prüfsumme, Build-Beleg und nativer Abnahme darf in die lokale Release-Vorbereitung eingehen. Der Workflow läuft nicht bei jedem Push oder PR.

## Windows

Ziel: Windows 11 oder neuer auf Intel/AMD x86_64. Ein Windows-Server-Runner bestätigt Paket und Fensterstart; die vollständige manuelle Windows-11-Abnahme auf einem Endgerät bleibt separat.

NSIS installiert ohne Administratorrechte unter `%LOCALAPPDATA%\Programs\Beni Buddy`. Eintrag und Deinstallation stehen im Startmenü und unter den installierten Apps. Es wird kein Autostart eingerichtet. Vor einer erneuten Installation Beni über sein Menü beenden. Der Installer und die Deinstallation verweigern Änderungen, solange die vorhandene GUI-Datei gesperrt ist. Einstellungen bleiben bei der Deinstallation erhalten, ebenso zusätzliche Dateien im Installationsordner.

Noch **keine Authenticode-Herausgebersignatur**: Windows kann den unbekannten Herausgeber melden oder den Start blockieren. Eine öffentliche unsignierte Vorschau muss deutlich so bezeichnet und bewusst freigegeben werden. Keine Anleitung zum Abschalten von Sicherheitsfunktionen. Siehe [Microsoft-Signierung](WINDOWS_SIGNING.md).

## Linux

Ziel: Intel/AMD x86_64, GLIBC 2.35 oder neuer. Build-Basis Ubuntu 22.04. X11 und Wayland werden mit virtuellen nativen Displays geprüft. Die Fensterverschiebung unter Wayland bleibt vom Compositor abhängig.

Das heruntergeladene `.AppImage` ausführbar machen und starten, beispielsweise:

```sh
chmod +x ./beni-buddy-0.1.1-linux-x86_64.AppImage
./beni-buddy-0.1.1-linux-x86_64.AppImage
```

Falls FUSE auf dem System fehlt, kann der AppImage-Runtime-Modus `APPIMAGE_EXTRACT_AND_RUN=1` verwendet werden. Er extrahiert temporär und führt das Paket aus. Ein Start installiert noch nichts. Die optionale Menüeinbindung benötigt Python 3:

```sh
./beni-buddy-0.1.1-linux-x86_64.AppImage --install-desktop
./beni-buddy-0.1.1-linux-x86_64.AppImage --remove-desktop
```

Die bewusste Installation kopiert das AppImage nach `${XDG_DATA_HOME:-~/.local/share}/beni-buddy/`, legt den eigenen Menüeintrag und das bestehende Beni-Icon an. Entfernen löscht nur belegte, unveränderte Installationsdateien. Einstellungen und andere Dateien bleiben erhalten. Verlinkte Zielpfade oder fremde Dateien werden abgelehnt. `--cli validate-assets` prüft die eingebetteten Assets.

GUI, CLI, bestehende Integrationsvorlagen und Lizenztexte sind enthalten. Die benötigten freien X11-/Wayland-Bibliotheken werden mit Copyright-Dateien und Paket-/Hash-Provenienz gebündelt. GLIBC, Loader und Grafiktreiber kommen vom System. Packaging-Werkzeuge und AppImage-Runtime sind versioniert und vor der Ausführung per SHA-256 geprüft.

## Release-Vertrag

`local-release.py prepare --portable-run RUN --portable-platform linux windows` kann erfolgreiche native main-Artefakte mit den Mac-Paketen zusammenführen. Windows ohne Herausgebersignatur benötigt zusätzlich `--allow-unsigned-windows` nach Nutzerfreigabe. Herkunft, Commit, Produkteingaben, Architektur, Version, Hash und native Fenster-/Deinstallationsbelege werden vor Zugriff auf Signierschlüssel geprüft. Die finalen Pakete und der gemeinsame Katalog erhalten Ed25519-Signaturen. Diese sind von der Windows-Herausgebersignierung unabhängig.

Automatische Updates sind noch nicht enthalten. Ihre Adapter und echte Update-Abnahmen bleiben R3/R4/R5-Folgearbeit.

## Aktuelle Abnahme

- **PASS lokal/macOS:** 111 Rusttests, fmt/clippy beider Workspaces; 9 portable Packaging-Tests, 23 lokale Release-Tests, 28 Mac-Packaging-Tests, 19 Schema-Tests, 11 Integrationsprüfungen; bestehende Assets und Icon-Provenienz unverändert.
- **NOT_RUN:** Native Windows-/Linux-Installer-Workflows, öffentliche Pakete, vollständige manuelle Windows-11-/Linux-Desktopabnahme, Microsoft-Signierung, echte Updates. Nach den Workflows und Veröffentlichung mit konkreten Belegen ergänzen.
