#!/bin/bash

# Build native cross-platform hybrid editor
APP_NAME="CEditorNative"
BUNDLE_NAME="${APP_NAME}.app"

echo "🏗️ Building ${BUNDLE_NAME} - Native Cross-Platform..."

# Clean previous build
rm -rf "${BUNDLE_NAME}"

# Create bundle structure
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Compile with C engines
echo "🔗 Linking with C engines..."
clang -framework Cocoa \
      -I../editor -I../markdown \
      -L../editor -leditor \
      -L../markdown -lmarkdown \
      NativeHybridApp.m -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ Native hybrid editor compilation successful"
else
    echo "❌ Native hybrid editor compilation failed"
    exit 1
fi

# Create Info.plist
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.ceditor.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>C Editor Native</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>md</string>
                <string>markdown</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>Markdown Document</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
</dict>
</plist>
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo "🚀 Launching native hybrid editor..."

open "${BUNDLE_NAME}"