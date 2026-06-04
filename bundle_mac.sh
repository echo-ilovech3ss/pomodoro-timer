#!/bin/bash
set -e

APP_NAME="Focus Flow"
APP_BUNDLE="/Applications/${APP_NAME}.app"
INFO_PLIST="${APP_BUNDLE}/Contents/Info.plist"

echo "=== 1. Building Release Binary ==="
cargo build --release

echo "=== 2. Creating Application Bundle Directory ==="
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

echo "=== 3. Copying Binary ==="
cp "target/release/focus-flow" "${APP_BUNDLE}/Contents/MacOS/Focus Flow"
chmod +x "${APP_BUNDLE}/Contents/MacOS/Focus Flow"

echo "=== 4. Generating Retina AppIcon.icns ==="
ICON_SET="logo.iconset"
mkdir -p "$ICON_SET"

# Generate various sizes for retina and standard displays
sips -z 16 16 logo.png --out "${ICON_SET}/icon_16x16.png" > /dev/null 2>&1
sips -z 32 32 logo.png --out "${ICON_SET}/icon_16x16@2x.png" > /dev/null 2>&1
sips -z 32 32 logo.png --out "${ICON_SET}/icon_32x32.png" > /dev/null 2>&1
sips -z 64 64 logo.png --out "${ICON_SET}/icon_32x32@2x.png" > /dev/null 2>&1
sips -z 128 128 logo.png --out "${ICON_SET}/icon_128x128.png" > /dev/null 2>&1
sips -z 256 256 logo.png --out "${ICON_SET}/icon_128x128@2x.png" > /dev/null 2>&1
sips -z 256 256 logo.png --out "${ICON_SET}/icon_256x256.png" > /dev/null 2>&1
sips -z 512 512 logo.png --out "${ICON_SET}/icon_256x256@2x.png" > /dev/null 2>&1
sips -z 512 512 logo.png --out "${ICON_SET}/icon_512x512.png" > /dev/null 2>&1
sips -z 1024 1024 logo.png --out "${ICON_SET}/icon_512x512@2x.png" > /dev/null 2>&1

iconutil -c icns "$ICON_SET"
cp logo.icns "${APP_BUNDLE}/Contents/Resources/AppIcon.icns"

# Clean up temp icon files
rm -rf "$ICON_SET" logo.icns

echo "=== 5. Writing Info.plist ==="
cat <<EOF > "$INFO_PLIST"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Focus Flow</string>
    <key>CFBundleIdentifier</key>
    <string>com.focusflow.pomodoro</string>
    <key>CFBundleName</key>
    <string>Focus Flow</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "=== 6. Updating Launch Services Database ==="
# Force macOS to register the new bundle and icon immediately
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP_BUNDLE"

echo "================================================="
echo "  Success! Focus Flow is installed in Applications!"
echo "  You can launch it from Finder, Launchpad, or Spotlight."
echo "================================================="
