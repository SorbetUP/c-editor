#!/bin/bash

# Build fixed hybrid app
APP_NAME="CEditorFixedHybrid"
BUNDLE_NAME="${APP_NAME}.app"

echo "🏗️ Building ${BUNDLE_NAME} with bug fixes..."

# Clean previous build
rm -rf "${BUNDLE_NAME}"

# Create bundle structure
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Compile with all C engines
echo "🔗 Linking with C engines (editor + markdown + cursor)..."
clang -framework Cocoa \
      -framework WebKit \
      -I../editor -I../markdown -I../cursor \
      -L../editor -leditor \
      -L../markdown -lmarkdown \
      -L../cursor -lcursor \
      PerfectHybrid.m -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ Fixed app compilation successful"
else
    echo "❌ Fixed app compilation failed"
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
    <string>C Editor Fixed Hybrid</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "✅ ${BUNDLE_NAME} created successfully with fixes"
echo "🚀 Launching fixed app..."

open "${BUNDLE_NAME}"