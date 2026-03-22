#!/bin/bash

# Build ElephantNotes v2 with C libraries
APP_NAME="ElephantNotesV2"
BUNDLE_NAME="${APP_NAME}.app"

echo "🐘 Building ${APP_NAME} with C libraries..."

# Clean previous build
rm -rf "${BUNDLE_NAME}"

# Create bundle structure
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Build required C libraries
echo "📚 Building C libraries..."

# Build hybrid_editor_core
cd ../hybrid_editor
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build hybrid_editor_core"
    exit 1
fi
cd ../markdown_editor_app

# Build file_manager
cd ../file_manager
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build file_manager"
    exit 1
fi
cd ../markdown_editor_app

# Compile with all libraries
echo "🔗 Linking ElephantNotes v2 with all C libraries..."
clang -framework Cocoa \
      -I../hybrid_editor -I../file_manager -I../editor -I../markdown \
      ../hybrid_editor/build/libhybrid_editor.a \
      ../file_manager/build/libfile_manager.a \
      ../editor/libeditor.a \
      ../markdown/libmarkdown.a \
      ElephantNotesV2.m -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

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
    <string>com.elephantnotes.v2.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes v2</string>
    <key>CFBundleVersion</key>
    <string>2.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>2.0</string>
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
                <string>mdown</string>
                <string>mkd</string>
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
echo "🎯 Features:"
echo "   - Hybrid markdown editing with C core"
echo "   - File operations (Open/Save/Save As)"
echo "   - Automatic backups"
echo "   - Keyboard shortcuts (⌘+O, ⌘+S, ⌘+Shift+S, ⌘+N)"
echo "   - Unsaved changes tracking"
echo "   - Cross-platform C libraries"
echo ""
echo "🚀 Launch: open ${BUNDLE_NAME}"
echo "📝 Or double-click to open!"