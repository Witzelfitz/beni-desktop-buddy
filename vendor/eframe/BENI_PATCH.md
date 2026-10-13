# Lokaler eframe-Patch: Accessibility in zusätzlichen Fenstern

Basis: eframe 0.36.2 aus dem veröffentlichten [Crate-Archiv](https://static.crates.io/crates/eframe/eframe-0.36.2.crate), SHA-256 `55ad41fac6e3149abb4866e93df37dbd33baa95db525c3368c944584e95a5dc2`. Upstream-Commit `49682f8baa058bf49e011035cfbd6e825f88a5ef`, Pfad `crates/eframe`. MIT-/Apache-2.0-Lizenzen liegen bei.

Der einzige geänderte Upstream-Quelltext ist `src/native/glow_integration.rs`. Glow initialisiert im Original nur für den Root-Viewport einen AccessKit-Adapter. Das erzeugt für native Zusatzfenster einen egui-Accessibility-Baum, macht ihn aber nicht für die Betriebssystem-API verfügbar.

Der Patch übergibt den bereits vorhandenen Eventloop-Proxy an den Glutin-Kontext und initialisiert einen Adapter bei der Erstellung jeder neuen egui-winit-Integration. Zusätzliche Fenster werden zuerst unsichtbar erstellt, weil AccessKit zwingend vor der ersten Sichtbarkeit installiert werden muss. Danach gilt die ursprünglich gewünschte Sichtbarkeit. Der Root-Viewport behält seinen bisherigen versteckten Start und den eframe-Pfad zum ersten Anzeigen. Bestehende Adapter werden nicht erneut initialisiert; Neuanlage nach Schliessen erhält einen neuen Adapter.

`apps/desktop/Cargo.toml` verwendet diese lokale Kopie über `[patch.crates-io]`. Alle anderen Bibliotheken und Versionsnummern bleiben unverändert. Cargo.toml.orig und die VCS-Metadaten dokumentieren das veröffentlichte Paket. Der unbenutzte Upstream-Lockfile ist ausgelassen; der Desktop-Lockfile liegt weiterhin in `apps/desktop`.

Vor Entfernung des Patches bei einem Upstream-Update müssen die native Fensterprüfung und die macOS-AX-Aktionen für Root und Einstellungen wiederholt werden: Öffnen, Checkboxen und Grössenwahl bedienen, Schliessen, erneut Öffnen sowie Meldungsempfang. Die Desktoptests prüfen zusätzlich den Child-Viewport, seine unabhängige Eingabefläche und die an Root adressierten Fensterkommandos. Die vollständige gesprochene Screenreader-Bedienung bleibt eine separate manuelle Abnahme.
