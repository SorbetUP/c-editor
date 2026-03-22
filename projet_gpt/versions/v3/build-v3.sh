#!/bin/bash

# Build ElephantNotes V3 - Interface intégrée avec système de vaults
APP_NAME="ElephantNotesV3"
BUNDLE_NAME="${APP_NAME}.app"

echo "🚀 Building ${APP_NAME} - Interface intégrée complète..."

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
cd ../markdown_editor_app_v4

# Build file_manager
cd ../file_manager
make professional
if [ $? -ne 0 ]; then
    echo "❌ Failed to build file_manager"
    exit 1
fi
cd ../markdown_editor_app_v4

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
cd ../markdown_editor_app_v4

# Build UI framework
echo "🖥️ Building UI framework..."
cd ../ui_framework
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build UI framework"
    exit 1
fi
cd ../markdown_editor_app_v4

# Build search system
echo "🔍 Building search system..."
cd ../advanced_search
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build advanced_search"
    exit 1
fi
cd ../search_interface
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build search_interface"
    exit 1
fi
cd ../markdown_editor_app_v4

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

# Compile with all libraries including search system
echo "🔗 Linking ${APP_NAME} with complete V3 architecture + search..."
clang -framework Cocoa \
      -I../hybrid_editor -I../file_manager -I../editor -I../markdown -I../vault_manager -I../ui_framework -I../advanced_search -I../search_interface \
      ${OPENSSL_CFLAGS} ${JSONC_CFLAGS} \
      ../hybrid_editor/build/libhybrid_editor.a \
      ../file_manager/build/libprofessional_file_manager.a \
      ../vault_manager/build/libvault_manager.a \
      ../ui_framework/build/libui_framework.a \
      ../advanced_search/build/libadvanced_search.a \
      ../search_interface/build/libsearch_interface.a \
      ../editor/libeditor.a \
      ../markdown/libmarkdown.a \
      ${OPENSSL_LIBS} ${JSONC_LIBS} -lpthread -lm \
      main_v3.m ElephantNotesV3.m VaultSetupController.m VaultManagerController.m VaultCreationPopup.m AppLogger.m AppSettings.m MenuStandardizer.m \
      -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ ${APP_NAME} compilation successful"
else
    echo "❌ ${APP_NAME} compilation failed"
    exit 1
fi

# Create Info.plist for V3
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.elephantnotes.v3.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes V3</string>
    <key>CFBundleVersion</key>
    <string>3.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>3.0</string>
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
                <string>vault</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>ElephantNotes Vault</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
    <key>NSAppleScriptEnabled</key>
    <true/>
</dict>
</plist>
EOF

# Create README for V3
cat > "${BUNDLE_NAME}/Contents/Resources/README_V3.md" << EOF
# ElephantNotes V3 - Interface Intégrée

## 🚀 Nouveautés V3

### Interface Complète
- **Barre latérale avec icônes**: Navigation intuitive
- **Éditeur intégré**: Markdown professionnel en superposition
- **Système de vaults**: Gestion complète des espaces de travail

### Icônes de Navigation
- **🔍 Recherche**: Rechercher dans toutes les notes
- **⬅️ Retour**: Navigation et historique
- **🏠 Dashboard**: Vue d'ensemble et accueil
- **⚙️ Paramètres**: Configuration et préférences

### Système de Vaults Intégré
- **Premier lancement**: Configuration automatique du vault
- **Multi-vaults**: Basculer entre différents espaces
- **Gestion complète**: Créer, supprimer, configurer les vaults

### Architecture V3
- **Framework UI modulaire**: Components réutilisables
- **Intégration C/Objective-C**: Performance optimale
- **Système de plugins**: Extensibilité future

## 🎯 Raccourcis Clavier

### Navigation
- **⌘+V**: Gestionnaire de vaults
- **⌘+Shift+V**: Nouveau vault

### Fichiers
- **⌘+N**: Nouvelle note
- **⌘+O**: Ouvrir fichier
- **⌘+S**: Sauvegarder

### Édition
- **⌘+Z**: Annuler
- **⌘+Shift+Z**: Rétablir
- **⌘+A**: Tout sélectionner

## 🔧 Fonctionnalités Professionnelles

- **Auto-sauvegarde**: Toutes les 3 secondes
- **Contrôle de version**: Versions automatiques
- **Détection de conflits**: Gestion intelligente
- **Récupération de session**: Restauration automatique

## 📁 Structure des Vaults

Chaque vault contient:
- **Notes/**: Documents Markdown
- **Attachments/**: Pièces jointes
- **Templates/**: Modèles de documents
- **.elephantnotes_vault**: Configuration

## 🛠️ Support Technique

Pour le support et les mises à jour:
- **Version**: 3.0.0
- **Architecture**: UI Framework + Vault System
- **Compatibilité**: macOS 10.15+
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo ""
echo "🚀 ElephantNotes V3 Features:"
echo "   🖥️  Interface intégrée avec barre latérale"
echo "   📁 Système de vaults complet"
echo "   🔍 Navigation par icônes (Recherche, Retour, Dashboard, Paramètres)"
echo "   ✨ Éditeur professionnel en superposition"
echo "   💾 Fonctionnalités professionnelles intégrées"
echo "   🎯 Premier lancement avec configuration automatique"
echo ""
echo "🎮 Navigation Interface:"
echo "   🔍 Recherche - Rechercher dans les notes"
echo "   ⬅️  Retour - Navigation et historique"
echo "   🏠 Dashboard - Vue d'ensemble du vault"
echo "   ⚙️  Paramètres - Configuration et vaults"
echo ""
echo "🚀 Launch: open ${BUNDLE_NAME}"
echo "📝 Interface complète prête à utiliser!"
echo ""
echo "📁 Vaults: ~/.elephantnotes/vault_registry.json"
echo "🔧 Configuration automatique au premier lancement"