#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != "Darwin" ]]; then echo "Dieses Script benötigt macOS." >&2; exit 1; fi
cargo build --manifest-path apps/desktop/Cargo.toml --release --locked
cargo build --release --locked -p buddy-cli
mkdir -p dist
staging="$(mktemp -d dist/beni-bundle.XXXXXX)"
trap 'rm -rf "$staging"' EXIT
app="$staging/Beni Buddy.app"
mkdir -p "$app/Contents/MacOS"
cp apps/desktop/target/release/beni-desktop-buddy "$app/Contents/MacOS/beni-desktop-buddy"
cp target/release/beni-buddy-cli "$app/Contents/MacOS/beni-buddy-cli"
mkdir -p "$app/Contents/Resources"
cp -R integrations "$app/Contents/Resources/"
cp docs/INTEGRATIONS.md "$app/Contents/Resources/"
cp assets/app-icon/Beni.icns "$app/Contents/Resources/"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Beni Buddy</string>
<key>CFBundleDisplayName</key><string>Beni Buddy</string>
<key>CFBundleIdentifier</key><string>io.witzelfitz.beni-buddy</string>
<key>CFBundleExecutable</key><string>beni-desktop-buddy</string>
<key>CFBundleIconFile</key><string>Beni.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>set-by-release-tool</string>
<key>CFBundleVersion</key><string>set-by-release-tool</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
python3 scripts/release.py stamp-plist "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
# Sign nested executables first, then seal the completed bundle.
codesign --force --sign - --timestamp=none "$app/Contents/MacOS/beni-desktop-buddy"
codesign --force --sign - --timestamp=none "$app/Contents/MacOS/beni-buddy-cli"
codesign --force --sign - --timestamp=none "$app"
codesign --verify --deep --strict --verbose=2 "$app"
output="dist/Beni Buddy.app"
if [[ -e "$output" ]]; then mv "$output" "$staging/previous.app"; fi
if ! mv "$app" "$output"; then
  if [[ -e "$staging/previous.app" ]]; then mv "$staging/previous.app" "$output"; fi
  exit 1
fi
echo "Vorbereitet: $output (ad hoc signiert, nicht notarisiert)."
