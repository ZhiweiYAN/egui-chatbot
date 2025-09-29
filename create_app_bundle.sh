#!/bin/bash

APP_NAME="egui-chatbot"
BUNDLE_NAME="${APP_NAME}.app"
BUILD_DIR="target/release"
BUNDLE_DIR="target/${BUNDLE_NAME}"

echo "Building release binary..."
cargo build --release

echo "Creating app bundle structure..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

echo "Copying binary..."
cp "$BUILD_DIR/eframe_template" "$BUNDLE_DIR/Contents/MacOS/${APP_NAME}"

echo "Copying icon..."
cp "assets/AppIcon.icns" "$BUNDLE_DIR/Contents/Resources/"

echo "Creating Info.plist..."
cat > "$BUNDLE_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.egui-chatbot</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>egui Chatbot</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
</dict>
</plist>
EOF

echo "Setting executable permissions..."
chmod +x "$BUNDLE_DIR/Contents/MacOS/${APP_NAME}"

echo "App bundle created at: $BUNDLE_DIR"
echo "You can now drag ${BUNDLE_NAME} to your Applications folder!"