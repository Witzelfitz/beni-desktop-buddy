# Prüfbericht · 12. Oktober 2026

Ausführungsumgebung: macOS auf Apple Silicon, Rust/Cargo 1.99.0. GitHub- und Crates.io-Netzwerkzugriff der Sitzung ist eingeschränkt.

| Status | Prüfung | Ergebnis |
| --- | --- | --- |
| PASS | `cargo test --workspace --offline --locked` | 10 Kern- und 3 CLI-Prozesstests bestanden; darin 480 Referenzposen aus dem Original-SDK |
| PASS | `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | Keine Warnungen |
| PASS | `cargo fmt --all --check` | Root-Workspace formatiert |
| PASS | `cargo fmt --manifest-path apps/desktop/Cargo.toml --check` | Desktop-Quellen formatiert; kein Typ-/Buildnachweis |
| PASS | `cargo run --offline --locked -p buddy-cli -- validate-assets` | 44 Posen, 20 Animationen, 2048 × 1850 PNG-Atlas |
| PASS | `cargo run --offline --locked -p buddy-cli -- sample wave 321` | JSON-Pose ausgegeben |
| PASS | `python3 scripts/check-assets.py` | SHA-256 der vier übernommenen Dateien stimmen |
| PASS | `bash -n scripts/check.sh scripts/publish-repo.sh scripts/package-macos.sh` | Shell-Syntax gültig |
| BLOCKED | `cargo check --manifest-path apps/desktop/Cargo.toml --offline` | eframe ist im lokalen Cargo-Cache nicht vorhanden |
| BLOCKED | `gh api user` und `gh api --method POST /user/repos ...` | Keine Verbindung zur GitHub-API; kein Remote bestätigt |
| BLOCKED | Erweiterter GitHub-Zugriff | Automatische Freigabeprüfung lehnt erweiterten Zugriff ab; Sandbox-Freigaben sind in dieser Sitzung deaktiviert |
| NOT_RUN | Desktop-Build, GUI-Start und manuelle Testmatrix | GUI-Abhängigkeiten fehlen; keine tatsächliche Desktop-Darstellung geprüft |
| NOT_RUN | GitHub Actions und macOS-App-Bundle | Kein entferntes Repository und kein Desktop-Build vorhanden |

Bei der ersten Testausführung wurden zwei erwartete Asset-Zählwerte von 19/456 auf die tatsächlich vorhandenen 20 Animationen/480 Referenzen korrigiert. Danach bestehen sämtliche 13 Tests.

## Nächster konkreter Schritt

Mit verbundenem GitHub-Zugang das vorbereitete private Repository veröffentlichen. Auf einem Rechner mit Crates.io-Zugriff den Desktop-Build ausführen, das dabei erzeugte `apps/desktop/Cargo.lock` committen und die macOS-Abnahme dokumentieren. Ein erfolgreich getesteter Animationskern ist kein Nachweis für eine laufende GUI.
