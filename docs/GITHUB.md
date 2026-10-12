# GitHub-Repository

Das private Repository wurde erstellt: [Witzelfitz/beni-desktop-buddy](https://github.com/Witzelfitz/beni-desktop-buddy).

Der ursprüngliche Branch `main` enthält den initialen README-Commit. Die Implementierung wird auf `feat/desktop-buddy-bootstrap` über einen Pull Request geprüft. Folgeänderungen ebenfalls auf einem Feature-Branch und per PR vornehmen; kein Force-Push.

```sh
git clone https://github.com/Witzelfitz/beni-desktop-buddy.git
cd beni-desktop-buddy
# Solange der initiale PR noch offen ist:
git switch feat/desktop-buddy-bootstrap
./scripts/check.sh --desktop
cargo run --manifest-path apps/desktop/Cargo.toml --locked
```

CI prüft Kern/CLI und Desktop auf macOS, Windows und Linux. Erfolgreiche Desktop-Jobs liefern unter «Actions» Download-Artefakte. Das macOS-Archiv enthält ein `.app`-Bundle; Windows/Linux enthalten die ausführbare Datei. Die vollständige manuelle Plattformprüfung folgt separat.

`scripts/publish-repo.sh` bleibt als Hilfsmittel für die Erstellung eines weiteren neuen privaten Repository aus einem vorbereiteten lokalen Checkout erhalten. Es überschreibt keine bestehenden Repositories und wird für dieses bereits angelegte Remote nicht erneut ausgeführt.
