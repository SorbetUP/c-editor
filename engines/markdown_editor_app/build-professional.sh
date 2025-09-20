#!/bin/bash

# Build ElephantNotes Professional with enterprise file management
APP_NAME="ElephantNotesProfessional"
BUNDLE_NAME="${APP_NAME}.app"

echo "🏢 Building ${APP_NAME} with enterprise-grade file management..."

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
make professional
if [ $? -ne 0 ]; then
    echo "❌ Failed to build file_manager"
    exit 1
fi
cd ../markdown_editor_app

# Build vault_manager
echo "📁 Building vault manager..."
cd ../vault_manager

# Check for json-c
if ! pkg-config --exists json-c; then
    echo "⚠️  json-c not found, installing via Homebrew..."
    if command -v brew >/dev/null 2>&1; then
        brew install json-c
    else
        echo "❌ Please install json-c: brew install json-c"
        exit 1
    fi
fi

make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build vault_manager"
    exit 1
fi
cd ../markdown_editor_app

# Check for required dependencies
echo "🔍 Checking dependencies..."

# Check for OpenSSL (required for hashing)
if ! pkg-config --exists openssl; then
    echo "⚠️  OpenSSL not found, installing via Homebrew..."
    if command -v brew >/dev/null 2>&1; then
        brew install openssl
    else
        echo "❌ Please install OpenSSL: brew install openssl"
        exit 1
    fi
fi

# Get dependency flags
OPENSSL_CFLAGS=$(pkg-config --cflags openssl)
OPENSSL_LIBS=$(pkg-config --libs openssl)
JSONC_CFLAGS=$(pkg-config --cflags json-c)
JSONC_LIBS=$(pkg-config --libs json-c)

# Compile with all libraries including vault system
echo "🔗 Linking ${APP_NAME} with enterprise libraries and vault system..."
clang -framework Cocoa \
      -I../hybrid_editor -I../file_manager -I../editor -I../markdown -I../vault_manager \
      ${OPENSSL_CFLAGS} ${JSONC_CFLAGS} \
      ../hybrid_editor/build/libhybrid_editor.a \
      ../file_manager/build/libprofessional_file_manager.a \
      ../vault_manager/build/libvault_manager.a \
      ../editor/libeditor.a \
      ../markdown/libmarkdown.a \
      ${OPENSSL_LIBS} ${JSONC_LIBS} -lpthread \
      ElephantNotesV2_Professional.m VaultSetupController.m -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ ${APP_NAME} compilation successful"
else
    echo "❌ ${APP_NAME} compilation failed"
    exit 1
fi

# Create Info.plist with professional features
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.elephantnotes.professional.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes Professional</string>
    <key>CFBundleVersion</key>
    <string>2.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>2.1</string>
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
            <key>CFBundleTypeIconFile</key>
            <string>document</string>
        </dict>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>workspace</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>ElephantNotes Workspace</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
    <key>NSAppleScriptEnabled</key>
    <true/>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Edit in ElephantNotes Professional</string>
            </dict>
            <key>NSMessage</key>
            <string>openWithElephantNotes</string>
            <key>NSPortName</key>
            <string>ElephantNotesProfessional</string>
            <key>NSSendTypes</key>
            <array>
                <string>NSStringPboardType</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create README for professional features
cat > "${BUNDLE_NAME}/Contents/Resources/README_Professional.md" << EOF
# ElephantNotes Professional v2.1

## 🏢 Enterprise Features

### Version Control
- **Automatic versioning**: Every save creates a version snapshot
- **Manual snapshots**: Create named versions with ⌘+K
- **Version history**: Browse and restore previous versions
- **Diff comparison**: Compare versions side-by-side

### Auto-Save & Recovery
- **Auto-save**: Automatic saving every 3 seconds
- **Crash recovery**: Recover unsaved changes after app restart
- **Session management**: Restore cursor position and scroll state
- **Workspace sessions**: Save and restore multiple files

### File Monitoring
- **Conflict detection**: Detect external file changes
- **Real-time monitoring**: Monitor files for external modifications
- **Merge conflict resolution**: Handle concurrent edits gracefully

### Professional Shortcuts
- **⌘+K**: Create version snapshot
- **⌘+I**: Show file statistics
- **⌘+H**: Show version history
- **⌘+O**: Open file with recovery options
- **⌘+S**: Save with automatic versioning

### Statistics & Analytics
- Files managed, versions created, auto-saves performed
- Conflict detection and resolution tracking
- Storage usage monitoring
- Active session tracking

### Backup Strategies
- **Simple backups**: .bak files
- **Timestamped backups**: Date/time stamped versions
- **Versioned backups**: Numbered sequence backups
- **Incremental backups**: Only save changes

### Security Features
- **File integrity**: SHA-256 content hashing
- **Backup verification**: Verify backup integrity
- **Permission management**: Respect file permissions
- **Safe operations**: Atomic file operations

## 🚀 Getting Started

1. **Create documents**: Use ⌘+N for new documents
2. **Enable features**: All professional features are enabled by default
3. **Version snapshots**: Press ⌘+K to create named versions
4. **View statistics**: Press ⌘+I to see file management stats
5. **Recovery**: Automatic recovery prompts on file open

## 📁 File Locations

- **Workspace**: ~/Documents/ElephantNotes_Workspace/
- **Backups**: ~/.elephantnotes/backups/
- **Versions**: Stored alongside original files (.v1, .v2, etc.)
- **Auto-saves**: Temporary .autosave files

## 🔧 Configuration

Professional features can be configured programmatically:
- Auto-save interval (default: 3 seconds)
- Maximum versions to keep (default: 20)
- Backup strategy (default: timestamped)
- Conflict detection (default: enabled)

## 🆘 Support

For enterprise support and custom configurations:
- GitHub: https://github.com/SorbetUP/c-editor
- Professional support available for enterprise deployments
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo ""
echo "🏢 Professional Features:"
echo "   ✨ Enterprise-grade version control"
echo "   🔄 Auto-save every 3 seconds with recovery"
echo "   📊 File statistics and analytics"
echo "   🔍 Real-time conflict detection"
echo "   💾 Multiple backup strategies"
echo "   🛡️  SHA-256 file integrity verification"
echo "   📱 Session management and workspace recovery"
echo "   ⚡ Professional keyboard shortcuts"
echo ""
echo "🎯 Professional Shortcuts:"
echo "   ⌘+K  Create version snapshot"
echo "   ⌘+I  Show file statistics"
echo "   ⌘+H  Show version history"
echo ""
echo "🚀 Launch: open ${BUNDLE_NAME}"
echo "📝 Or double-click to start professional editing!"
echo ""
echo "📁 Workspace: ~/Documents/ElephantNotes_Workspace/"
echo "💾 Backups: ~/.elephantnotes/backups/"