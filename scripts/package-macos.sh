#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(uname -s)" != "Darwin" ]]; then echo "Dieses Script benötigt macOS." >&2; exit 1; fi
cargo build --manifest-path apps/desktop/Cargo.toml --release --locked
cargo build --release --locked -p buddy-cli
app="dist/Beni Buddy.app"
mkdir -p "$app/Contents/MacOS"
cp apps/desktop/target/release/beni-desktop-buddy "$app/Contents/MacOS/beni-desktop-buddy"
cp target/release/beni-buddy-cli "$app/Contents/MacOS/beni-buddy-cli"
mkdir -p "$app/Contents/Resources"
cp -R integrations "$app/Contents/Resources/"
cp docs/INTEGRATIONS.md "$app/Contents/Resources/"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Beni Buddy</string>
<key>CFBundleDisplayName</key><string>Beni Buddy</string>
<key>CFBundleIdentifier</key><string>io.witzelfitz.beni-buddy</string>
<key>CFBundleExecutable</key><string>beni-desktop-buddy</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
echo "Vorbereitet: $app (lokales, nicht notarisiertes Entwicklungsbundle)."
