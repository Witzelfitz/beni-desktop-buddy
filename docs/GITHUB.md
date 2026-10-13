# GitHub-Repository

Das Desktop-Repository: [Witzelfitz/beni-desktop-buddy](https://github.com/Witzelfitz/beni-desktop-buddy).

Die Projektgrundlage wird über [Pull Request #1](https://github.com/Witzelfitz/beni-desktop-buddy/pull/1) mit bestandener plattformübergreifender CI auf `main` übernommen. Folgeänderungen ebenfalls auf einem Feature-Branch und per PR vornehmen; kein Force-Push.

```sh
git clone https://github.com/Witzelfitz/beni-desktop-buddy.git
cd beni-desktop-buddy
./scripts/check.sh --desktop
cargo run --manifest-path apps/desktop/Cargo.toml --locked
```

[Die erste CI-Prüfung](https://github.com/Witzelfitz/beni-desktop-buddy/actions/runs/38248825723) für Kern/CLI und Desktop ist auf macOS, Windows und Linux bestanden. Erfolgreiche Desktop-Jobs liefern unter «Actions» Download-Artefakte. Das macOS-Archiv enthält ein `.app`-Bundle; Windows/Linux enthalten die ausführbare Datei. Die vollständige manuelle Plattformprüfung folgt separat.

`scripts/publish-repo.sh` bleibt als Hilfsmittel für die Erstellung eines weiteren neuen privaten Repository aus einem vorbereiteten lokalen Checkout erhalten. Es überschreibt keine bestehenden Repositories und wird für dieses bereits angelegte Remote nicht erneut ausgeführt.

## Distribution und öffentlicher Zugriff

Der [Masterplan](MASTERPLAN.md) legt öffentliche GitHub Releases als bevorzugte Downloadquelle fest. Der Nutzer hat die öffentliche Freigabe des Desktop-Repositories zur Vereinfachung der Distribution erlaubt und MIT für den Rust-Code mit gesonderten Beni-Rechten gewählt. Das Desktop-Repo ist seit dem 13. Oktober 2026 öffentlich; GitHub-API und anonymer Zugriff bestätigen die Freigabe. Die vorausgehende Prüfung ist im [Prüfbericht](PUBLICATION_REVIEW.md) dokumentiert. Das getrennte Website-Repository Beni-Allrounder bleibt privat.

Actions-Artefakte bleiben Entwicklungsdownloads. Öffentliche Installer und Update-Feeds folgen erst mit den entsprechenden Schritten des Masterplans.
