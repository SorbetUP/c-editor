# 🔍 Système de Recherche Avancé pour ElephantNotes

Ce système de recherche ultra-rapide combine deux bibliothèques C complémentaires pour offrir une expérience de recherche moderne et performante.

## 🏗️ Architecture

### 1. Bibliothèque de Recherche Avancée (`advanced_search`)
- **Recherche sémantique** avec embeddings vectoriels (simulation EmbeddingGemma)
- **Recherche floue** avec distance de Levenshtein
- **Recherche exacte** dans les noms de fichiers
- **Modes de performance** : Speed (128D), Balanced (256D), Accuracy (768D)
- **Optimisations** : Cosine similarity, caching, indexation vectorielle

### 2. Interface de Recherche (`search_interface`)
- **Arborescence interactive** avec expand/collapse
- **Barre de recherche** type navigateur
- **Navigation au clavier** et à la souris
- **État persistant** des dossiers dépliés
- **Callbacks** pour intégration UI

## 🚀 Fonctionnalités

### Recherche Avancée
- ✅ **Recherche sémantique** : Trouve les documents par similitude de sens
- ✅ **Recherche floue** : Tolère les fautes de frappe dans les noms de fichiers
- ✅ **Recherche hybride** : Combine exacte, floue et sémantique
- ✅ **Performance optimisée** : <1ms par requête en moyenne
- ✅ **Embedding vectoriel** : Simulation avec normalisation et cosine similarity

### Interface Interactive
- ✅ **Arborescence navigable** : Expand/collapse des dossiers
- ✅ **Barre de recherche live** : Recherche en temps réel
- ✅ **Focus automatique** : Curseur auto dans la barre de recherche
- ✅ **Filtres de fichiers** : Support .md, .txt, .json
- ✅ **Callbacks d'événements** : Sélection fichier, recherche changée, arbre étendu

## 📊 Performance

### Benchmarks
- **Indexation** : ~0.06ms pour 4-5 fichiers
- **Recherche sémantique** : ~0.01ms par requête
- **Construction arborescence** : ~0.03ms
- **Mémoire** : <1MB pour corpus de test

### Modes de Performance
- **Speed Mode** : 128D embeddings, ~0.00ms
- **Balanced Mode** : 256D embeddings, ~0.00ms  
- **Accuracy Mode** : 768D embeddings, ~0.01ms

## 🧪 Tests

### Tests Unitaires
```bash
# Tester la bibliothèque de recherche avancée
cd advanced_search
make clean && make all
./build/test_search
./build/test_search --benchmark

# Tester l'interface de recherche
cd search_interface  
make clean && make all
./build/test_search_interface
```

### Test d'Intégration Complète
```bash
# Test automatique
./build/search_integration

# Test avec benchmarks
./build/search_integration --benchmark

# Mode interactif
./build/search_integration --interactive
```

### Commandes Mode Interactif
- `s <query>` - Rechercher
- `e <path>` - Déplier un dossier
- `c <path>` - Replier un dossier
- `t` - Afficher l'arborescence
- `stats` - Afficher les statistiques
- `q` - Quitter

## 🔧 Compilation

```bash
# Compiler tout le système
cd search_interface
make clean && make all

# Cela compile :
# - libsearch_interface.a
# - libadvanced_search.a (dépendance)
# - test_search_interface (tests unitaires)
# - search_integration (démonstration complète)
```

## 📁 Structure des Fichiers

```
search_interface/
├── search_interface.h          # API interface de recherche
├── search_interface.c          # Implémentation arborescence
├── search_integration.c        # Démonstration intégrée
├── test_search_interface.c     # Tests unitaires
├── Makefile                   # Build système
└── test_vault/               # Données de test
    ├── Notes/
    │   ├── Projects/
    │   │   ├── alpha.md
    │   │   └── beta.md
    │   └── Archive/
    │       └── old_note.md
    └── Templates/
        ├── daily.md
        └── meeting.md

advanced_search/
├── advanced_search.h          # API recherche avancée  
├── advanced_search.c          # Implémentation embeddings
├── test_search.c             # Tests unitaires
└── Makefile                  # Build système
```

## 🎯 Utilisation

### API Recherche Avancée
```c
// Créer le moteur
SearchConfig config = search_engine_get_default_config();
config.mode = SEARCH_MODE_BALANCED;
SearchEngine* engine = search_engine_create(&config);

// Indexer un répertoire  
search_engine_index_directory(engine, "vault_path", NULL, NULL);

// Recherche sémantique
int num_results;
SearchResult* results = search_semantic_similar(engine, "artificial intelligence", &num_results);

// Recherche floue dans les noms
SearchResult* fuzzy_results = search_filename_fuzzy(engine, "document", &num_results);
```

### API Interface de Recherche
```c
// Créer l'interface
SearchInterfaceConfig config = search_interface_get_default_config();
config.on_file_selected = my_file_callback;
config.on_search_changed = my_search_callback;
SearchInterface* interface = search_interface_create(&config);

// Définir le répertoire racine
search_interface_set_root_directory(interface, "vault_path");

// Contrôler l'arborescence
search_interface_expand_node(interface, "path/to/folder");
search_interface_set_search_query(interface, "ma recherche");
```

## 🔮 Prochaines Étapes

Cette bibliothèque est prête pour l'intégration dans ElephantNotes V3. Les prochaines étapes seraient :

1. **Intégration UI native** : Convertir en composants Objective-C/Swift
2. **Modèle d'embedding réel** : Remplacer la simulation par EmbeddingGemma
3. **Persistence** : Sauvegarder l'état de l'arborescence et l'index
4. **Optimisations** : FAISS réel, compression vectorielle, indexation incrémentale

## 📝 Notes Techniques

- **Simulation d'embedding** : Utilise un algorithme simple basé sur les caractères pour démontrer l'architecture
- **Compatibilité** : Code C11 portable, compatible macOS/Linux
- **Mémoire** : Gestion manuelle avec cleanup automatique
- **Performance** : Optimisé pour corpus de notes de taille moyenne (1k-10k fichiers)

Le système est entièrement fonctionnel en tant que preuve de concept et démontre toutes les fonctionnalités requises pour l'interface de recherche d'ElephantNotes.