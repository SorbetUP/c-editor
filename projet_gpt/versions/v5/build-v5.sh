#!/bin/bash

set -e

APP_NAME="ElephantNotesV5"
BUNDLE_NAME="${APP_NAME}.app"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)

cd "${SCRIPT_DIR}"

echo "🚀 Building ${APP_NAME} - Architecture Modulaire..."

rm -rf "${BUNDLE_NAME}"
mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

build_module() {
  local path="$1"
  local target="$2"
  pushd "${REPO_ROOT}/${path}" >/dev/null
  if ! make "${target}"; then
    echo "❌ Failed to build ${path} (${target})"
    exit 1
  fi
  popd >/dev/null
}

echo "📚 Building C libraries..."
build_module "engines/hybrid_editor" static
build_module "engines/file_manager" professional

echo "📁 Building vault manager..."
if ! pkg-config --exists json-c; then
  echo "⚠️  json-c not found, installing via Homebrew..."
  if command -v brew >/dev/null 2>&1; then
    brew install json-c
  else
    echo "❌ Please install json-c: brew install json-c"
    exit 1
  fi
fi
build_module "engines/vault_manager" static

echo "🖥️ Building UI framework..."
build_module "engines/ui_framework" static

echo "🔍 Building search system..."
build_module "engines/advanced_search" static
build_module "engines/search_interface" static

echo "🔍 Checking dependencies..."
if ! pkg-config --exists openssl; then
  echo "⚠️  OpenSSL not found, installing via Homebrew..."
  if command -v brew >/dev/null 2>&1; then
    brew install openssl
  else
    echo "❌ Please install OpenSSL: brew install openssl"
    exit 1
  fi
fi

OPENSSL_CFLAGS=$(pkg-config --cflags openssl)
OPENSSL_LIBS=$(pkg-config --libs openssl)
JSONC_CFLAGS=$(pkg-config --cflags json-c)
JSONC_LIBS=$(pkg-config --libs json-c)

APP_BINARY="${SCRIPT_DIR}/${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

echo "🔗 Linking ${APP_NAME} with modular V5 architecture..."
clang -framework Cocoa       -I"${REPO_ROOT}/engines/hybrid_editor"       -I"${REPO_ROOT}/engines/file_manager"       -I"${REPO_ROOT}/engines/editor"       -I"${REPO_ROOT}/engines/markdown"       -I"${REPO_ROOT}/engines/vault_manager"       -I"${REPO_ROOT}/engines/ui_framework"       -I"${REPO_ROOT}/engines/advanced_search"       -I"${REPO_ROOT}/engines/search_interface"       ${OPENSSL_CFLAGS} ${JSONC_CFLAGS}       "${REPO_ROOT}/engines/hybrid_editor/build/libhybrid_editor.a"       "${REPO_ROOT}/engines/file_manager/build/libprofessional_file_manager.a"       "${REPO_ROOT}/engines/vault_manager/build/libvault_manager.a"       "${REPO_ROOT}/engines/ui_framework/build/libui_framework.a"       "${REPO_ROOT}/engines/advanced_search/build/libadvanced_search.a"       "${REPO_ROOT}/engines/search_interface/build/libsearch_interface.a"       "${REPO_ROOT}/engines/editor/libeditor.a"       "${REPO_ROOT}/engines/markdown/libmarkdown.a"       ${OPENSSL_LIBS} ${JSONC_LIBS} -lpthread -lm       "${SCRIPT_DIR}/main_v5.m"       "${SCRIPT_DIR}/Modules/Controllers/ENAppDelegate.m"       "${SCRIPT_DIR}/Modules/Controllers/ENMainController.m"       "${SCRIPT_DIR}/Modules/Sidebar/ENSidebar.m"       "${SCRIPT_DIR}/Modules/Tabs/ENTabBase.m"       "${SCRIPT_DIR}/Modules/Tabs/ENDashboardTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENEditorTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENFilesTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENSearchTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENToolsTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENSettingsTab.m"       "${SCRIPT_DIR}/Modules/Tabs/ENTablesTab.m"       -o "${APP_BINARY}"

echo "✅ ${APP_NAME} compilation successful"

cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.elephantnotes.v5.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes V5</string>
    <key>CFBundleVersion</key>
    <string>5.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>5.0</string>
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

cat > "${BUNDLE_NAME}/Contents/Resources/README_V5.md" << 'EOF'
# ElephantNotes V5 - Architecture Modulaire

## 🏗️ Nouveautés V5

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
- **Pas d'interface personnalisée** qui masque la barre
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

- **Version** : 5.0.0
- **Architecture** : Modulaire Objective-C + C Engine
- **Compatibilité** : macOS 10.15+
EOF

echo "✅ ${BUNDLE_NAME} created successfully"
echo ""
echo "🚀 ElephantNotes V5 Features:"
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
