#!/bin/bash

# Build ElephantNotes V4 - Architecture Modulaire
APP_NAME="ElephantNotesV4"
BUNDLE_NAME="${APP_NAME}.app"

echo "🚀 Building ${APP_NAME} - Architecture Modulaire..."

# Clean previous build
rm -rf "${BUNDLE_NAME}"

# Create bundle structure
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

# Build required C libraries
echo "📚 Building C libraries..."

# Build hybrid_editor_core
cd ../../engines/hybrid_editor
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build hybrid_editor_core"
    exit 1
fi
cd ../../versions/v4

# Build file_manager
cd ../../engines/file_manager
make professional
if [ $? -ne 0 ]; then
    echo "❌ Failed to build file_manager"
    exit 1
fi
cd ../../versions/v4

# Build vault_manager
echo "📁 Building vault manager..."
cd ../../engines/vault_manager

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
cd ../../versions/v4

# Build UI framework
echo "🖥️ Building UI framework..."
cd ../../engines/ui_framework
make static
if [ $? -ne 0 ]; then
    echo "❌ Failed to build UI framework"
    exit 1
fi
cd ../..

# Build search system
echo "🔍 Building search system..."
cd engines/advanced_search
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
cd ../..

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

# Compile with modular architecture
echo "🔗 Linking ${APP_NAME} with modular V4 architecture..."
clang -framework Cocoa \
      -Iengines/hybrid_editor -Iengines/file_manager -Iengines/editor -Iengines/markdown -Iengines/vault_manager -Iengines/ui_framework -Iengines/advanced_search -Iengines/search_interface \
      ${OPENSSL_CFLAGS} ${JSONC_CFLAGS} \
      engines/hybrid_editor/build/libhybrid_editor.a \
      engines/file_manager/build/libprofessional_file_manager.a \
      engines/vault_manager/build/libvault_manager.a \
      engines/ui_framework/build/libui_framework.a \
      engines/advanced_search/build/libadvanced_search.a \
      engines/search_interface/build/libsearch_interface.a \
      engines/editor/libeditor.a \
      engines/markdown/libmarkdown.a \
      ${OPENSSL_LIBS} ${JSONC_LIBS} -lpthread -lm \
      main_v4.m \
      Modules/Controllers/ENAppDelegate.m \
      Modules/Controllers/ENMainController.m \
      Modules/Sidebar/ENSidebar.m \
      Modules/Tabs/ENTabBase.m \
      Modules/Tabs/ENDashboardTab.m \
      Modules/Tabs/ENEditorTab.m \
      Modules/Tabs/ENFilesTab.m \
      Modules/Tabs/ENSearchTab.m \
      Modules/Tabs/ENToolsTab.m \
      Modules/Tabs/ENSettingsTab.m \
      -o "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

if [ $? -eq 0 ]; then
    echo "✅ ${APP_NAME} compilation successful"
else
    echo "❌ ${APP_NAME} compilation failed"
    exit 1
fi

# Create Info.plist for V4
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.elephantnotes.v4.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes V4</string>
    <key>CFBundleVersion</key>
    <string>4.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>4.0</string>
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

# Create README for V4
cat > "${BUNDLE_NAME}/Contents/Resources/README_V4.md" << EOF
# ElephantNotes V4 - Architecture Modulaire

## 🏗️ Nouveautés V4

### Architecture Modulaire
- **Séparation des responsabilités** : Chaque module a un rôle défini
- **Extensibilité** : Ajouter nouveaux onglets facilement  
- **Maintenabilité** : Fichiers plus petits et focalisés
- **Interface cohérente** : Tous les onglets utilisent ui_framework_set_editor_content()

### Modules Implémentés
- **ENTabBase** : Classe de base pour tous les onglets
- **ENSidebar** : Gestion de la barre latérale
- **ENMainController** : Contrôleur principal
- **ENAppDelegate** : Délégué d'application simplifié
- **ENDashboardTab** : Module Dashboard
- **ENSearchTab** : Module Search (interface markdown)
- **ENSettingsTab** : Module Settings

### Résolution du Problème Barre Latérale
- **Interface claire** entre sidebar et contenu
- **Utilisation systématique** de ui_framework_set_editor_content()
- **Pas d'interface personnalisée** qui masque la sidebar
- **Affichage correct** de tous les modes

## 🎯 Raccourcis Clavier

### Navigation
- **🔍 Recherche** : Clic sur l'icône recherche
- **🏠 Dashboard** : Clic sur l'icône dashboard
- **⚙️ Paramètres** : Clic sur l'icône paramètres

### Fichiers
- **⌘+N** : Nouvelle note
- **⌘+O** : Ouvrir fichier
- **⌘+S** : Sauvegarder

### Vaults
- **⌘+V** : Gestionnaire de vaults
- **⌘+Shift+V** : Nouveau vault

## 🔧 Architecture Technique

### Structure des Modules
```
Modules/
├── Sidebar/          # Gestion barre latérale
├── Tabs/            # Onglets modulaires
└── Controllers/     # Contrôleurs principaux
```

### Flux de Fonctionnement
1. ENAppDelegate initialise l'application
2. ENMainController configure les modules
3. ENSidebar gère les événements de navigation
4. Onglets génèrent du contenu markdown
5. Affichage via ui_framework_set_editor_content()

## 📁 Structure des Vaults

Chaque vault contient:
- **Notes/** : Documents Markdown
- **Attachments/** : Pièces jointes
- **Templates/** : Modèles de documents
- **.elephantnotes_vault** : Configuration

## 🛠️ Support Technique

- **Version** : 4.0.0
- **Architecture** : Modulaire Objective-C + C Engine
- **Compatibilité** : macOS 10.15+
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo ""
echo "🚀 ElephantNotes V4 Features:"
echo "   🏗️  Architecture modulaire complète"
echo "   📁 Modules séparés et organisés"
echo "   🔍 Interface de recherche markdown (pas de custom views)"
echo "   🎮 Barre latérale toujours visible"
echo "   ✨ Code maintenable et extensible"
echo "   💾 Intégration complète avec vault system"
echo ""
echo "🎮 Navigation Interface:"
echo "   🔍 Recherche - Interface markdown native"
echo "   🏠 Dashboard - Vue d'ensemble du vault"
echo "   ⚙️ Paramètres - Configuration et vaults"
echo ""
echo "🚀 Launch: open ${BUNDLE_NAME}"
echo "📝 Architecture modulaire prête à utiliser!"
echo ""
echo "📁 Vaults: ~/.elephantnotes/vault_registry.json"
echo "🔧 Configuration automatique au premier lancement"