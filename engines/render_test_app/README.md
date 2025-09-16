# 🚀 Moteur de Rendu C - Application de Test macOS

Cette application macOS native démontre l'intégration du moteur de rendu C cross-platform dans une interface utilisateur moderne.

## ✨ Fonctionnalités

### 🎯 **Interface Web-like Native**
- Rendu DOM-like sans technologies web lourdes
- Layout automatique avec containers et éléments enfants
- Gestion des couleurs, polices, et espacement
- Interface moderne avec sections Header, Content, Stats

### 🖥️ **Intégration macOS Avancée**
- **Cocoa Framework** pour l'interface native
- **Core Graphics** pour le rendu haute qualité
- **CVDisplayLink** pour animation fluide (60 FPS)
- Support du redimensionnement de fenêtre
- Gestion des événements souris

### 🔧 **Démonstration du Moteur**
- **87KB** seulement pour toute la librairie
- Détection automatique de plateforme (macOS)
- Backend Core Graphics activé automatiquement
- Fallback software intégré
- Gestion mémoire optimisée

## 🚀 Lancement Rapide

```bash
# Option 1: Lancer directement
open RenderTestApp.app

# Option 2: Depuis le terminal
./RenderTestApp

# Option 3: Construire et lancer
make run
```

## 🛠️ Compilation

```bash
# Construction simple
make

# Construction + bundle macOS
make bundle

# Nettoyage complet
make clean-all

# Statut du build
make status
```

## 🎮 Utilisation

1. **Lancer l'app** - Une fenêtre de 800x600 s'ouvre
2. **Observer l'interface** - Layout web-like avec sections colorées
3. **Cliquer dans la fenêtre** - Les événements sont loggés dans la console
4. **Redimensionner** - Le layout s'adapte automatiquement
5. **Fermer** - L'app se termine proprement avec cleanup

## 📊 Contenu de Démonstration

### **Header (Bleu)**
- Titre avec emoji et police en gras
- Arrière-plan couleur `#2D7BFB`
- Texte blanc avec padding

### **Section Content (Blanc)**
- Texte de bienvenue
- Liste des fonctionnalités avec emojis
- Police système avec différentes tailles

### **Section Stats (Gris clair)**
- Statistiques du moteur en temps réel
- Police monospace `Menlo`
- Informations techniques

### **Bouton (Vert)**
- Message de succès
- Style button-like
- Arrière-plan `#28A745`

## 🔍 Architecture Technique

### **Structure des Fichiers**
```
render_test_app/
├── main.m              # Application Objective-C principale
├── Makefile           # Système de build
├── README.md          # Cette documentation
├── RenderTestApp      # Exécutable
└── RenderTestApp.app/ # Bundle macOS
    ├── Contents/
    │   ├── MacOS/RenderTestApp
    │   └── Info.plist
```

### **Classes Principales**

#### **RenderView : NSView**
- Intégration du moteur de rendu C
- Gestion du contexte de rendu
- Rendu via `drawRect:`
- Animation avec `CVDisplayLink`

#### **AppDelegate : NSObject**
- Configuration de la fenêtre
- Gestion du cycle de vie
- Setup de l'interface

### **Flux de Rendu**
1. **Initialisation** - Création du contexte render
2. **Contenu** - Construction de l'arbre DOM-like
3. **Layout** - Calcul des positions automatique
4. **Rendu** - Dessin via Core Graphics
5. **Affichage** - Présentation dans NSView

## 🎯 Intégration du Moteur C

### **API Utilisée**
```c
// Création du contexte
render_context_t* ctx = render_engine_create_context(
    RENDER_BACKEND_FRAMEBUFFER, 800, 600
);

// Création d'éléments
render_element_t* header = render_engine_create_element(
    RENDER_ELEMENT_BOX, "header"
);

// Stylisation
header->style.background_color = (render_color_t){45, 123, 251, 255};
header->style.padding = (render_rect_t){15, 15, 15, 15};

// Hiérarchie
render_engine_add_child(root, header);

// Rendu
render_engine_render(ctx);
```

### **Bridging Objective-C ↔ C**
- Headers C importés dans `.m`
- Structures C utilisées directement
- Gestion mémoire hybride (ARC + manuel)
- Conversion des événements Cocoa → C

## 🔧 Personnalisation

### **Modifier le Contenu**
Éditer la méthode `createDemoContent` dans `main.m` pour :
- Ajouter de nouveaux éléments
- Changer les couleurs et styles  
- Modifier le texte et les polices
- Ajuster le layout

### **Ajouter des Interactions**
Étendre `mouseDown:` pour :
- Hit testing avec le moteur
- Animations
- Changement d'état des éléments

### **Optimiser les Performances**
- Activer/désactiver le `CVDisplayLink`
- Ajuster la fréquence de rendu
- Optimiser les redraws

## 🚀 Prochaines Étapes

### **Fonctionnalités Avancées**
- [ ] Parsing HTML/CSS réel
- [ ] Animations CSS-like
- [ ] Support des images
- [ ] Scrolling et viewport
- [ ] Formulaires interactifs

### **Optimisations**
- [ ] Rendu différentiel
- [ ] Cache de layout
- [ ] GPU acceleration
- [ ] Multi-threading

### **Cross-Platform**
- [ ] Port vers iOS
- [ ] Version Linux (GTK)
- [ ] Support Windows
- [ ] Version web (WASM)

---

## 📄 Logs de Test

Les logs de l'application montrent :
```
✅ Render context created: 800x600
✅ Demo content created with 12 elements  
✅ Application launched successfully
🎯 Click anywhere in the window to test interaction
```

**Status** : ✅ **Fonctionnel** - Le moteur de rendu C est parfaitement intégré dans une app macOS native !

---

*Construit avec le moteur de rendu C cross-platform (87KB) - Une alternative légère aux technologies web lourdes*