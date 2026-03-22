#!/bin/bash

# Build example integration using hybrid_editor_core
APP_NAME="ModernElephantNotes"
BUNDLE_NAME="${APP_NAME}.app"

echo "🔧 Building ${APP_NAME} with hybrid_editor_core C library..."

# Clean previous build
rm -rf "${BUNDLE_NAME}"

# Create bundle structure
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Build C library first
echo "📚 Building hybrid_editor_core library..."
make static

if [ $? -ne 0 ]; then
    echo "❌ Failed to build hybrid_editor_core library"
    exit 1
fi

# Compile with all engines and hybrid library
echo "🔗 Linking with C engines and hybrid_editor_core..."
clang -framework Cocoa \
      -I. -I../editor -I../markdown \
      -L. -L../editor -L../markdown -Lbuild \
      -leditor -lmarkdown -lhybrid_editor \
      example_integration.m -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ ${APP_NAME} compilation successful"
else
    echo "❌ ${APP_NAME} compilation failed"
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
    <string>com.elephantnotes.modern.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>Modern ElephantNotes</string>
    <key>CFBundleVersion</key>
    <string>2.0</string>
    <key>CFBundleShortVersionString</key>
    <string>2.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo "🚀 Built with hybrid_editor_core C library for maximum portability!"
echo "📦 Launch with: open ${BUNDLE_NAME}"