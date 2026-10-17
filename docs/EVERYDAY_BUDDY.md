# Beni im Alltag: Desktop und Web

Nutzerentscheidung vom 15. Oktober 2026. [Website-Textvorbereitung: PR #12](https://github.com/Witzelfitz/Beni-Allrounder/pull/12). Beni begleitet Fokus, Erinnerungen und bewusst verbundene Statusquellen. Der Einstieg benötigt keine KI-Einrichtung. Der bestehende lokale Ollama-Chat bleibt freiwillig; eigene Cloud-Keys und zusätzliche Anbieter folgen später. Ein finanzierter Chat ist eine spätere Pilotidee, keine Kostenfreigabe.

## Reihenfolge

1. Menü-Rad mit Fokus, Erinnerung, Verlauf, Ruhe, Reden und Einstellungen. Direkte Fokus-/Erinnerungsansichten sowie ein Zustandsanzeiger im offenen Rad.
2. Verständliche Zustände und gelegentliche Zwischenstände: Start, sinnvoller Fortschritt und Abschluss aktiver Aufgaben. Keine zufälligen Dauerhinweise. Zwischenstände respektieren «Nicht stören», Sichtbarkeit, Gespräch, Verlauf und bestehende Sprechblasenregeln. Pro Ereignis höchstens eine Anzeige; Pause verschiebt den Fortschritt. Dieser Schritt ist noch nicht umgesetzt.
3. Gemeinsamer Auftritt: zwei Wege auf mini.witzelfitz.io, «Beni auf meinem Desktop» und «Beni in meinem Projekt». Studio, Einbettung, OBS und Bibliothek bleiben erhalten. Schweizer Hochdeutsch, kurze Texte und bestehende Gesten auf beiden Wegen. Eine veröffentlichte Desktop-Funktion wird als solche beschrieben, eine Web-Vorschau als Vorschau.
4. Später: begrenzter, freiwilliger Chat-Pilot. Betreiber-Key nur auf einem Server; Zugang, Anfragen und Budget dort begrenzen. Eigene Keys bleiben eine erweiterte Option.

## Menü-Rad

Klick auf Beni öffnet sechs beschriftete Aktionen rund um die Figur. Ein weiterer Klick oder Escape schliesst das Rad. Es benötigt keine neue Fensterposition oder zusätzliche Desktopfläche. Seit 0.1.7 wird die Figur bei geöffnetem Rad passend kleiner gezeichnet, damit keine Taste die Figur überdeckt. Schliessen stellt die ursprüngliche Darstellung wieder her; Fensterposition und -grösse bleiben unverändert. Ziehen verschiebt Beni weiterhin. Hintergrundbereiche bleiben durchklickbar, sichtbare Tasten erhalten eigene Eingaberegionen und Accessibility-Namen. Bei sehr enger Fensterfläche gibt es eine begrenzte scrollbar dargestellte Aktionsfläche.

Seit 0.1.7 verwenden Fokus und Erinnerung [kompakte Eingabekarten](COMPACT_UI.md) mit Fokus auf dem Eingabefeld, Enter-Bestätigung und Escape zum Schliessen ohne Entwurfsverlust. Die vollständigen Einstellungen bleiben separat erreichbar. Fokus starten/unterbrechen/fortsetzen/abbrechen und Erinnerungen planen/entfernen verwenden denselben Zustand wie die Einstellungen. Beide Ansichten zeigen ihre Sitzungsgrenzen ausdrücklich. Erinnerungen und aktive Timer überleben derzeit kein App-Beenden; eine spätere Speicherung ist eine eigene Produkt-/Datenschutzentscheidung.

«Ruhe» schaltet das bestehende «Nicht stören» um. Animation pausieren und Beenden bleiben im Tray, Alt/Option + Q beendet die App bei fokussiertem Fenster. Die bisherige Begrüssungsanimation ist über die Einstellungen auswählbar. Das Rad wird beim Nachlesen vom Verlauf abgelöst. Der Zustand im offenen Rad zeigt den aktiven Timer, sonst geplante/wartende Erinnerungen oder «Bereit». Automatische Zwischenmeldungen folgen erst in Schritt 2.

## Verbindliche Quellen

| Inhalt | Quelle und Übernahme |
| --- | --- |
| Grafiken und Animationsdaten | Bestehender Web-/SDK-Bestand. Kontrollierte, versionierte Übernahme in Rust mit Provenienz und Referenzprüfungen; keine neuen Posen |
| Desktop-Version, Pakete und Plattformen | Öffentlicher, authentifizierter Desktop-Release. Die Website nutzt weiterhin den signierten Katalog und dessen Feeds |
| Begriffe, Persona, Beispiele und Gestenzuordnung | Geplanter kleiner versionierter Katalog im öffentlichen Desktop-Repo; Web übernimmt eine festgelegte Version. Noch nicht implementiert |

Die Rust- und Web-Laufzeiten bleiben getrennt. Keine Laufzeitabhängigkeit der lokalen Kernfunktionen von der Website, kein zusätzlicher Fernsteuerungsport und kein automatischer Upload von Statusmeldungen. Die streng geprüften Release-Metadaten werden für künftige Funktionsangaben nur über einen bewusst versionierten, kompatiblen Vertrag erweitert.

## Gemeinsamer Veröffentlichungsablauf

Für neue Produktfunktionen gilt die vom Nutzer bestätigte Reihenfolge:

1. **Website vorbereiten:** zusammengehörigen Web-PR mit Texten, Demo und Versionszuordnung vorbereiten. Neue Desktopfunktionen bleiben bis zum Release unveröffentlicht beziehungsweise eindeutig als Vorschau markiert. Die bestehende öffentliche Downloadseite beschreibt weiterhin den verfügbaren Installer.
2. **App prüfen und veröffentlichen:** Desktop-PR, geeignete Plattformprüfungen, bewusst gestartete Installer-Builds, Signaturen und native Abnahme. Erst danach Release und authentifizierte Metadaten freigeben.
3. **Website freischalten:** vorbereiteten Web-Stand nach tatsächlicher Verfügbarkeit deployen. Version und Downloads stammen aus demselben Release; Beispiele und Hilfe passen zur freigegebenen Bedienung.
4. **Gemeinsam abnehmen:** Download → Installation → erster Fokus/Erinnerung → Zustandsanzeige → Update. Bestehende Einbettungen und ältere unterstützte App-Versionen bleiben gesondert berücksichtigt.

Website-Deployments starten keine Rust-Builds. Ein gemergter Desktop-PR ist noch kein veröffentlichter Installer. Das Menü-Rad ist seit 0.1.6/Build 7 veröffentlicht. 0.1.7/Build 8 ergänzt kompakte Eingabekarten und mehr Abstand zur Figur (Desktop-PR #67/#68). Website-PR #14 zeigt dieselben echten App-Bilder und die passende Hilfe; Katalog und beide Mac-Feeds sind live geprüft. Automatische Zwischenmeldungen aus Schritt 2 sind weiterhin offen.


Seit 0.1.8/Build 9 hält Beni zwischen gelegentlichen vorhandenen Gesten eine ruhige Originalpose. Regulärer Sekundentakt in Ruhe, 15 fps für automatische Idle-Gesten und 30 fps für aktive Gesten; Eingaben/IPC wecken weiter direkt. Native Menüwerte und macOS-Klickflächen werden nur bei Änderungen geschrieben/kopiert. Desktop-PR #70/#71 und Website-PR #15 sind gemergt; das Update ist veröffentlicht. Katalog und beide Mac-Feeds stimmen überein. Eine bestimmte Akkuersparnis ist nicht gemessen. Der nächste Produktabschnitt bleibt Schritt 2 mit verständlichen Zwischenständen.
