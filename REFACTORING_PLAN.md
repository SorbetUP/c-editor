# Plan de Refactorisation - ElephantNotes V4

## 🎯 Objectif
Refaire l'application avec une architecture modulaire propre pour :
- Séparer les responsabilités 
- Faciliter la maintenance et le debug
- Résoudre les problèmes de barre latérale masquée
- Rendre le code plus lisible et extensible

## 📁 Structure Modulaire

```
Modules/
├── Sidebar/
│   ├── ENSidebar.h
│   └── ENSidebar.m
├── Tabs/
│   ├── ENTabBase.h          # Classe de base
│   ├── ENTabBase.m
│   ├── ENDashboardTab.h
│   ├── ENDashboardTab.m
│   ├── ENSearchTab.h
│   ├── ENSearchTab.m
│   ├── ENSettingsTab.h
│   └── ENSettingsTab.m
└── Controllers/
    ├── ENMainController.h
    ├── ENMainController.m
    ├── ENAppDelegate.h
    └── ENAppDelegate.m
```

## 🏗️ Architecture des Composants

### 1. ENSidebar (Module Barre Latérale)
**Responsabilités :**
- Gestion de l'affichage de la barre latérale
- Gestion des clics sur les icônes
- Animation des états actifs/inactifs
- Interface avec UIFramework

**Interface :**
```objc
@interface ENSidebar : NSObject
@property (nonatomic, weak) id<ENSidebarDelegate> delegate;
- (void)setActiveIcon:(UIIconType)iconType;
- (void)setup;
@end
```

### 2. ENTabBase (Classe de Base)
**Responsabilités :**
- Interface commune pour tous les onglets
- Gestion du cycle de vie des onglets
- Intégration avec ui_framework_set_editor_content()
- État partagé (vault, framework, etc.)

**Interface :**
```objc
@interface ENTabBase : NSObject
- (NSString*)generateContent;
- (void)didBecomeActive;
- (void)didBecomeInactive;
- (void)displayContent;
@end
```

### 3. Modules d'Onglets

#### ENDashboardTab
- Génère l'interface Dashboard en markdown
- Statistiques du vault
- Fichiers récents
- Actions rapides

#### ENSearchTab  
- Interface de recherche en markdown
- Liste des fichiers du vault
- Instructions d'utilisation
- Statistiques de performance

#### ENSettingsTab
- Configuration de l'application
- Gestion des vaults
- Paramètres utilisateur
- Informations système

### 4. ENMainController (Contrôleur Principal)
**Responsabilités :**
- Coordination entre sidebar et onglets
- Gestion des changements d'onglets
- Interface avec le système de vaults
- Gestion des événements

### 5. ENAppDelegate (Délégué Application)
**Responsabilités :**
- Initialisation de l'application
- Configuration de la fenêtre
- Gestion du cycle de vie

## 🔄 Flux de Fonctionnement

1. **Démarrage :**
   - ENAppDelegate initialise l'application
   - ENMainController configure les modules
   - ENSidebar s'initialise avec UIFramework
   - Onglet Dashboard activé par défaut

2. **Changement d'onglet :**
   - Utilisateur clique sur icône sidebar
   - ENSidebar notifie ENMainController
   - ENMainController désactive onglet actuel
   - ENMainController active nouvel onglet
   - Nouvel onglet génère son contenu
   - Contenu affiché via ui_framework_set_editor_content()

3. **Gestion du contenu :**
   - Chaque onglet utilise generateContent()
   - Contenu en markdown formaté
   - Affichage via ui_framework_set_editor_content()
   - Barre latérale toujours visible

## ✅ Plan d'Implémentation

### Phase 1: Structure de Base
- [x] Créer structure de dossiers
- [ ] ENTabBase.h/.m
- [ ] ENSidebar.h/.m  
- [ ] ENMainController.h/.m
- [ ] ENAppDelegate.h/.m

### Phase 2: Modules d'Onglets
- [ ] ENDashboardTab.h/.m
- [ ] ENSearchTab.h/.m
- [ ] ENSettingsTab.h/.m

### Phase 3: Intégration
- [ ] Modifier build script
- [ ] Tester compilation
- [ ] Résoudre les dépendances

### Phase 4: Migration
- [ ] Migrer logique existante vers modules
- [ ] Tester fonctionnalités
- [ ] Supprimer ancien code

### Phase 5: Tests et Validation
- [ ] Test changement d'onglets
- [ ] Test barre latérale visible
- [ ] Test fonctionnalités vault
- [ ] Validation complète

## 🎨 Avantages de cette Architecture

1. **Séparation des Responsabilités :**
   - Chaque module a un rôle défini
   - Code plus facile à comprendre
   - Debug plus simple

2. **Extensibilité :**
   - Ajouter nouveaux onglets facilement
   - Modifier comportement sidebar indépendamment
   - Réutilisabilité des composants

3. **Maintenabilité :**
   - Fichiers plus petits et focalisés
   - Tests unitaires possibles
   - Refactorisation locale sans impact global

4. **Résolution du Problème Barre Latérale :**
   - Interface claire entre sidebar et contenu
   - Utilisation systématique de ui_framework_set_editor_content()
   - Pas d'interface personnalisée qui masque la sidebar

## 🚀 Prochaines Étapes

1. Implémenter ENTabBase.m
2. Implémenter ENSidebar.h/.m
3. Implémenter ENMainController.h/.m
4. Créer les modules d'onglets un par un
5. Intégrer et tester