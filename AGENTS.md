# Beni Desktop Buddy

Dieses eigenständige Projekt baut einen nativen Desktop-Buddy in Rust. Die Stack-Regeln für «Cards against MMP» im übergeordneten Download-Ordner gehören zu einem anderen Produkt. Für dieses vom Nutzer ausdrücklich beauftragte Rust-Projekt gilt die Entscheidung in `docs/adr/0001-native-rust.md`.

- Produkttexte und Dokumentation: Schweizer Hochdeutsch ohne ß. Code-Bezeichner: Englisch.
- Bestehende Beni-Grafiken und ihre Rechte erhalten. Assets und SHA-256-Provenienz gemeinsam aktualisieren.
- Der Animationskern bleibt unabhängig von Fenster, GPU, Netzwerk und Systemzeit. Zeit und Aktionen explizit übergeben.
- Kein Autostart, Mikrofon, Bildschirmzugriff, Telemetrie oder Fernsteuerungsport ohne konkrete Nutzeranforderung.
- Änderungen am Verhalten mit sinnvollen positiven und negativen Tests prüfen.
- Root: `cargo fmt --all --check`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Desktop: separate Befehle mit `--manifest-path apps/desktop/Cargo.toml`. Eine erfolgreiche Kernprüfung bestätigt keinen GUI-Build.
- macOS ist die erste manuelle Abnahmeplattform. Windows und Linux separat prüfen; Wayland erlaubt nicht dieselben Fensterpositionierungsfunktionen wie macOS/X11.
- Keine direkt auf main gepushten Folgeänderungen, kein Force-Push. Neue Aufgaben auf Branches und per PR.
- Keine Zugangsgeheimnisse in Dateien oder Logs. Ein GitHub-Repository gilt erst nach bestätigtem API-Ergebnis als erstellt.
- Nicht ausgeführte Builds, GUI-Abnahmen und externe Schritte offen dokumentieren.
