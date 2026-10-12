# GitHub vorbereiten und veröffentlichen

Der lokale Repository-Name ist `beni-desktop-buddy`. Als Ziel ist ein neues **privates** Repository vorgesehen; standardmässig unter dem verbundenen persönlichen GitHub-Konto. Es wurde in der Vorbereitungssitzung noch kein Remote erstellt.

Der lokale Branch `main` enthält ausschliesslich den initialen README-Commit. Die Implementierung liegt auf `feat/desktop-buddy-bootstrap`. Das Veröffentlichungsscript erzeugt ein neues privates Remote, pusht die beiden Branches und öffnet einen Pull Request für die Implementierung.

Sobald eine gültige GitHub-Verbindung und Netzwerkzugriff vorhanden sind:

```sh
# Falls nötig, GitHub CLI im eigenen Terminal anmelden.
gh auth login

# Im Root dieses Repository, mit explizitem Owner:
./scripts/publish-repo.sh Witzelfitz
```

Für eine Organisation deren Namen als Argument übergeben. Das Konto braucht die Berechtigung, dort Repositories anzulegen.

Das Script überschreibt keine bestehenden Repositories und veröffentlicht keine uncommitteten Dateien. Es führt keinen Force-Push aus und merged den PR nicht.

## Unterbrochene Veröffentlichung fortsetzen

Wenn das Remote bereits erstellt wurde, den Owner und Remote zuerst prüfen:

```sh
git remote -v
gh repo view OWNER/beni-desktop-buddy --json nameWithOwner,isPrivate,url
```

Anschliessend fehlende Schritte einzeln fortsetzen, ohne ein weiteres Repository anzulegen:

```sh
git push -u origin main
git push -u origin feat/desktop-buddy-bootstrap
gh pr list --repo OWNER/beni-desktop-buddy --head feat/desktop-buddy-bootstrap
gh pr create --repo OWNER/beni-desktop-buddy --base main --head feat/desktop-buddy-bootstrap \
  --title="Beni Desktop Buddy: native Rust-Grundlage" --body-file docs/BOOTSTRAP_PR.md
```

`gh pr create` nur ausführen, wenn noch kein passender PR existiert. Der nächste Arbeitsschritt ist der erste verbundene Desktop-Build mit committedem `apps/desktop/Cargo.lock`, danach die manuelle macOS-Abnahme. Erst mit bestätigtem CI-/Abnahmeergebnis mergen.
