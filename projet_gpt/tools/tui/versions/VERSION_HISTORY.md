# TUI Editor - Historique des Versions

## Version 1.0.0 - "Enhanced Editor" (2024-09-12)

### ✨ Fonctionnalités
- Gestion intelligente du curseur avec C engine
- Sauvegarde de fichiers (Ctrl+S)
- Système d'aide intégré (Ctrl+H)
- Numéros de ligne configurables (Ctrl+L)
- Coloration syntaxique Markdown basique
- Messages de statut avec horodatage
- Gestion intelligente des tabs
- Protection contre buffer overflow
- Confirmation de sortie pour fichiers modifiés

### 🐛 Corrections
- Buffer overflow potentiel corrigé (buffer 65536)
- Gestion propre des signaux (SIGINT/SIGTERM)
- Validation stricte des limites de fichier
- Fuites mémoire potentielles éliminées

### 🎮 Commandes
- Ctrl+S: Sauvegarder
- Ctrl+H: Aide
- Ctrl+L: Basculer numéros de ligne
- Ctrl+Q: Quitter (avec confirmation)
- Ctrl+C: Sortie forcée
- Tab: Insérer espaces

---

## Version 2.0.0 - "Advanced Editor" (En développement)

### 🎯 Nouvelles fonctionnalités prévues
- [ ] Recherche de texte (Ctrl+F)
- [ ] Système Undo/Redo (Ctrl+Z/Ctrl+Y)
- [ ] Sélection de texte (Shift+arrows)
- [ ] Copier/Coller (Ctrl+C/Ctrl+V)
- [ ] Ouverture de fichiers (Ctrl+O)
- [ ] Coloration syntaxique avancée
- [ ] Navigation par mots (Ctrl+Left/Right)
- [ ] Recherche et remplacement (Ctrl+R)
- [ ] Mode insertion/overwrite (Insert)
- [ ] Statut de mode dans la barre de statut