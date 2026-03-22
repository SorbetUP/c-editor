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
