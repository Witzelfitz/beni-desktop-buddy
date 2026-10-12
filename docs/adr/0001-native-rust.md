# ADR 0001: Beni als native Rust-App

Status: Für die vorbereitete Grundlage gewählt, Desktop-Abnahme noch offen.

## Auftrag

Der Nutzer möchte ein neues GitHub-Repository und den vorhandenen Beni-Avatar als Desktop-Buddy in einer Rust-Applikation. Die Web-Lernseite bleibt ein eigenes Projekt. Die übergeordneten Download-Ordnerregeln für das Kartenspiel beschreiben einen anderen Auftrag; dessen Node/MongoDB-Stack passt nicht zu dieser ausdrücklichen Rust-Anforderung.

## Entscheidung

Der Animationskern verwendet Rust und die originalen Asset-Daten. egui/eframe 0.36.2 stellt das native transparente Fenster bereit. Der Glow-Renderer vermeidet einen zusätzlich direkt konfigurierten WGPU-Stack für dieses kleine 2D-MVP. Daten und PNG werden beim Kompilieren eingebettet.

Der Root-Workspace enthält Kern und CLI. Die GUI liegt in einem separaten Workspace mit eigenem Lockfile. Damit sind Animationsprüfungen auch ohne GUI-Abhängigkeiten und ohne GPU möglich. CI muss beide Workspaces explizit prüfen.

## Konsequenzen

- Kein Node-Server, WebView oder JavaScript zur Laufzeit.
- Die Bewegung wird gegen berechnete Referenzposen des bisherigen SDK geprüft.
- Die Desktopbibliothek ist exakt gepinnt; ihr Lockfile muss beim ersten verbundenen Build erzeugt und committed werden.
- Transparenz, Vordergrunddarstellung, Monitorwechsel und Dragging brauchen echte Plattformtests. Unter Wayland gelten Einschränkungen für die absolute Positionierung.
- Tray, Click-through, Autostart und globale Hotkeys sind nachgelagerte Funktionen. Die MVP-Oberfläche bietet direkt eine sichtbare Beenden-Aktion.
- Ein Kern-PASS ist kein Desktop-PASS.

Referenzen: [eframe-App-Vertrag](https://docs.rs/eframe/0.36.2/eframe/trait.App.html), [ViewportBuilder](https://docs.rs/egui/0.36.2/egui/viewport/struct.ViewportBuilder.html), [Release 0.36.2](https://github.com/emilk/egui/releases).
