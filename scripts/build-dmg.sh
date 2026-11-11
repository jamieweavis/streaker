#!/usr/bin/env bash
# Package one or more macOS `Streaker` binaries into a universal Streaker.app inside a .dmg.
#
# Usage: scripts/build-dmg.sh <version> <output-dir> <binary>...
#
# Run from the repo root (the app icon is read from assets/app/icon.icns). The binaries are merged
# with `lipo`, so pass one per architecture. The app is ad-hoc signed, not notarised.
set -euo pipefail

version="$1"
out_dir="$2"
shift 2

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

app="$work/dmg/Streaker.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

lipo -create "$@" -output "$app/Contents/MacOS/Streaker"
chmod +x "$app/Contents/MacOS/Streaker"
cp assets/app/icon.icns "$app/Contents/Resources/icon.icns"

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Streaker</string>
  <key>CFBundleExecutable</key>
  <string>Streaker</string>
  <key>CFBundleIconFile</key>
  <string>icon</string>
  <key>CFBundleIdentifier</key>
  <string>com.jamieweavis.streaker</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Streaker</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSHumanReadableCopyright</key>
  <string>Copyright © 2018-$(date +%Y) Jamie Weavis</string>
</dict>
</plist>
EOF

codesign --force --sign - "$app"
ln -s /Applications "$work/dmg/Applications"

mkdir -p "$out_dir"
dmg="$out_dir/Streaker-$version-macos-universal.dmg"
hdiutil create -volname Streaker -srcfolder "$work/dmg" -format UDZO -ov "$dmg"
(cd "$out_dir" && shasum -a 256 "$(basename "$dmg")" > "$(basename "$dmg").sha256")
echo "$dmg"
