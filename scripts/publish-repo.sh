#!/usr/bin/env bash
# Run only to publish this prepared project to a NEW private GitHub repository.
set -euo pipefail
cd "$(dirname "$0")/.."
owner="${1:-}"
if [[ -z "$owner" ]]; then owner="$(gh api user --jq .login)"; fi
if [[ ! "$owner" =~ ^[A-Za-z0-9][A-Za-z0-9-]*$ ]]; then
  echo "Ungültiger GitHub-Owner." >&2; exit 1
fi
repo="$owner/beni-desktop-buddy"
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Bitte Änderungen zuerst committen; veröffentlicht werden nur geprüfte Commits." >&2; exit 1
fi
if [[ "$(git branch --show-current)" != "feat/desktop-buddy-bootstrap" ]]; then
  echo "Bitte zuerst feat/desktop-buddy-bootstrap auschecken." >&2; exit 1
fi
if git remote get-url origin >/dev/null 2>&1; then
  echo "Ein origin ist bereits gesetzt. Fortsetzung siehe docs/GITHUB.md." >&2; exit 1
fi
gh api user --jq .login >/dev/null
if gh repo view "$repo" --json nameWithOwner >/dev/null 2>&1; then
  echo "$repo existiert schon. Dieses Script überschreibt kein bestehendes Repository." >&2; exit 1
fi
gh repo create "$repo" --private --source=. --remote=origin \
  --description="Mini-Beni als nativer Desktop-Buddy in Rust: Avatar, Animationen und transparentes Desktopfenster."
# main contains only the initial README; implementation is reviewed through a PR.
git push -u origin main
git push -u origin feat/desktop-buddy-bootstrap
gh pr create --repo "$repo" --base main --head feat/desktop-buddy-bootstrap \
  --title="Beni Desktop Buddy: native Rust-Grundlage" --body-file docs/BOOTSTRAP_PR.md
