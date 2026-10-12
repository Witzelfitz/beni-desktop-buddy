# SDK-Referenzposen

` sdk-poses.json ` wurde direkt durch die originale JavaScript-Funktion `poseAt` aus Beni Allrounder 1.0.0 erzeugt, bevor die Rust-Portierung geprüft wurde. Die Fixture wird nicht aus der Rust-Implementierung berechnet.

20 Animationen × 8 Zeitpunkte × 3 Energiewerte ergeben 480 Einträge. Zeitpunkte: −100, 0, 100, 321, 1000, 5000, Loopdauer und Loopdauer + 1 ms. Energie: 0, 0.7 und 1. Geprüft werden Frame, Segmentindex, Spiegelung, Translation, Rotation, Skalierung und Fortschritt.

SHA-256 der ursprünglichen `src/motion.mjs`: `dd268f1c4ed3bd07ccc0a9870d38134a0c783af35eaf50dd23d741b29ca5ac7d`.

Bei bewussten Änderungen am Animationsvertrag die Referenzen aus dem ursprünglichen beziehungsweise gemeinsam aktualisierten SDK neu erzeugen und die Abweichung dokumentieren.
