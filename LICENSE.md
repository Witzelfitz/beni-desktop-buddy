# Rechte und Herkunft

## Rust-Code

Der eigenständige Rust-Quellcode in `crates/` und `apps/desktop/`, einschliesslich Rust-Tests und Rust-Buildcode, steht unter der [MIT-Lizenz](LICENSE-MIT), Copyright 2026 Witzelfitz. Diese Freigabe umfasst keine Beni-Grafiken, Figur, App-Icons, importierten Animationsdaten oder SDK-Referenzdaten.

## Beni-Grafiken und Ursprungsdaten

Die Beni-Grafiken und die importierten Animationsdaten stammen aus «Beni Allrounder 1.0.0», Witzelfitz. Bestehende Rechte an Figur, Grafiken und Ursprungsdaten bleiben erhalten. `assets/beni/provenance.json` dokumentiert die unveränderten importierten Dateien mit SHA-256.

Das neue App-Icon unter `assets/app-icon/` wurde mit dem eingebauten Imagegen-Werkzeug auf Basis der bestehenden Beni-Figur gestaltet. Die Rechte an Beni bleiben erhalten. `assets/app-icon/provenance.json` enthält Vorlage, Prompt, Exportverfahren und SHA-256 der erzeugten und abgeleiteten Dateien. Der originale Animationsatlas wurde dafür nicht verändert.

Der Rust-Animationskern überträgt die Bewegungsberechnung aus dem bestehenden Avatar-SDK. Der Referenztest stammt aus der ursprünglichen Funktion `src/motion.mjs`.

Die Rechte an Beni-Figur, Grafiken, App-Icon und importierten Ursprungsdaten bleiben bei ihren Rechteinhabern. Diese Dateien werden durch die MIT-Lizenz des Rust-Codes nicht neu lizenziert. Dazu gehören `assets/beni/`, `assets/app-icon/`, die Beni-Abbildungen in `docs/images/` und `crates/buddy-core/tests/fixtures/sdk-poses.json`.

## Abhängigkeiten und weitere Dateien

Abhängigkeiten und die lokale Kopie in `vendor/eframe/` behalten ihre eigenen Lizenzen. Deren MIT-/Apache-2.0-Texte liegen bei. Für weitere Dateien wird durch die Rust-Lizenz keine zusätzliche pauschale Freigabe ausgesprochen.
