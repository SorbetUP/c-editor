#!/bin/bash

# Build ElephantNotes Professional with Vault Management System
APP_NAME="ElephantNotesProfessional"
BUNDLE_NAME="${APP_NAME}.app"

echo "🏢📁 Building ${APP_NAME} with Vault Management System..."

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

# Check for required dependencies
echo "🔍 Checking dependencies..."

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

make clean && make static
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

# Compile with all libraries and vault system
echo "🔗 Linking ${APP_NAME} with Vault Management System..."
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

# Create Info.plist with vault features
cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.elephantnotes.vault.${APP_NAME}</string>
    <key>CFBundleName</key>
    <string>ElephantNotes Professional</string>
    <key>CFBundleVersion</key>
    <string>2.2.0</string>
    <key>CFBundleShortVersionString</key>
    <string>2.2</string>
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
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>ElephantNotes Vault URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>elephantnotes</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create README for vault features
cat > "${BUNDLE_NAME}/Contents/Resources/README_Vault.md" << EOF
# ElephantNotes Professional v2.2 - Vault System

## 📁 Système de Vaults

### Qu'est-ce qu'un vault ?
Un **vault** est un dossier racine qui contient toutes vos notes et documents. Il s'agit de votre espace de travail principal organisé et sécurisé.

### 🚀 Configuration au premier démarrage
Au premier lancement, ElephantNotes vous guide pour :
1. **Choisir un nom** pour votre vault
2. **Sélectionner l'emplacement** où le stocker
3. **Créer la structure** de base automatiquement
4. **Ajouter des notes d'exemple** (optionnel)

### 📂 Structure d'un vault
```
Mon Vault/
├── Notes/              # Vos notes markdown
├── Attachments/        # Fichiers joints (images, PDFs)
├── Templates/          # Modèles réutilisables
└── .workspace/         # Configuration privée
```

### ✨ Fonctionnalités du vault

#### Organisation intelligente
- **Dossier Notes/** : Contient vos fichiers markdown
- **Dossier Attachments/** : Pour vos images et documents
- **Dossier Templates/** : Modèles de notes prêts à utiliser

#### Gestion professionnelle
- 🔄 **Auto-sauvegarde** toutes les 3 secondes
- 📚 **Contrôle de version** automatique pour chaque fichier
- 🔍 **Détection de conflits** en temps réel
- 💾 **Session recovery** après crash
- ⌘+K **Instantanés manuels** avec commentaires

#### Interface adaptée
- **Ouverture de fichiers** : Direct dans le vault
- **Sauvegarde** : Propose automatiquement le dossier Notes/
- **Titre de fenêtre** : Affiche le nom du vault actif
- **Navigation** : Optimisée pour la structure du vault

### 🎯 Raccourcis spécialisés
- **⌘+O** : Ouvrir un fichier (dossier Notes/ par défaut)
- **⌘+S** : Sauvegarder (avec versioning automatique)
- **⌘+Shift+S** : Sauvegarder sous... (dossier Notes/ par défaut)
- **⌘+N** : Nouvelle note
- **⌘+K** : Créer un instantané de version
- **⌘+I** : Statistiques du vault et du fichier
- **⌘+H** : Historique des versions

### 📊 Avantages du système de vault

#### Productivité
- **Organisation claire** : Structure prédéfinie
- **Accès rapide** : Navigation optimisée
- **Templates** : Démarrage rapide avec des modèles

#### Sécurité
- **Versioning automatique** : Jamais de perte de données
- **Backups intelligents** : Copies horodatées
- **Intégrité** : Vérification SHA-256

#### Collaboration (future)
- **Partage** : Structure standardisée
- **Synchronisation** : Préparé pour le cloud
- **Conflits** : Détection et résolution automatique

### 🔧 Configuration avancée
Les vaults sont stockés dans `~/.elephantnotes/vault_registry.json` avec :
- Liste des vaults disponibles
- Vault par défaut
- Préférences utilisateur
- Derniers accès

### 💡 Bonnes pratiques
1. **Un vault par projet** ou thématique
2. **Structure cohérente** : utilisez les dossiers prédéfinis
3. **Templates** : créez vos modèles récurrents
4. **Versioning** : utilisez ⌘+K pour les étapes importantes
5. **Organisation** : sous-dossiers dans Notes/ selon vos besoins

## 🆘 Support
- Interface de configuration intuitive au premier démarrage
- Messages d'erreur explicites en français
- Validation automatique des emplacements
- Suggestions intelligentes pour les noms

Le système de vault transforme ElephantNotes en un véritable environnement de travail professionnel pour vos notes et documents.
EOF

echo "✅ ${BUNDLE_NAME} created successfully with Vault System"
echo ""
echo "🏢📁 Vault Management Features:"
echo "   🎯 Configuration guidée au premier démarrage"
echo "   📂 Structure organisée automatiquement"
echo "   🔄 Integration avec le système professionnel"
echo "   💾 Gestion centralisée des notes"
echo "   ⚡ Navigation optimisée dans le vault"
echo "   🛡️  Versioning et backup dans le vault"
echo ""
echo "🎯 Vault Features:"
echo "   Premier lancement : Configuration automatique"
echo "   Organisation : Notes/, Attachments/, Templates/"
echo "   Navigation : Dossiers vault par défaut"
echo "   Titre : Affiche le nom du vault actif"
echo ""
echo "🚀 Launch: open ${BUNDLE_NAME}"
echo "📝 Au premier démarrage, vous choisirez votre vault !"
echo ""
echo "📁 Vaults configurés dans : ~/.elephantnotes/"
EOF