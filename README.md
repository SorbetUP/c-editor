# C Editor Core

Un noyau C minimal et portable pour l'édition de documents Markdown avec conversion JSON bidirectionnelle.

## Fonctionnalités

### Blocs supportés (v1)
- **Paragraphe** - Texte libre avec styles inline
- **Titres** - `#` à `######` (détection en début de ligne, conversion automatique en MAJUSCULE à la saisie)
- **Images** - `![alt](url)` avec attributs optionnels `{w=160 h=120 a=0.9 align=left|center|right}`
- **Tableaux** - Format "pipe table" Markdown standard

### Styles inline supportés (v1)
- `**gras**` - Texte en gras
- `*italique*` - Texte en italique  
- `***gras italique***` - Combinaison gras + italique
- `==surligné==` - Surlignage (couleur par défaut)
- `++souligné++` - Soulignement (couleur + gap par défaut)

### Limitations v1
- Pas de liens `[text](url)`
- Pas de code inline `` `code` `` ou blocs de code
- Pas de listes (prévues pour v2)

## Schéma JSON canonique

Les couleurs sont **toujours** stockées sous forme de tableaux de 4 flottants `[r,g,b,a]` avec r,g,b,a ∈ [0.0,1.0].

### Structure du document
```json
{
  "name": "new note",
  "meta": {
    "default": { "fontsize": 11, "font": "Helvetica" },
    "icon": "",
    "updated": 0,
    "created": 0
  },
  "elements": [...]
}
```

### Éléments supportés

**Text**
```json
{
  "type": "text",
  "text": "hello world",
  "align": "left|center|right|justify",
  "font": "Helvetica",
  "font_size": 16,
  "color": [0,0,0,1],
  "bold": true,
  "italic": true,
  "underline": { "color": [0,0,0,0.4], "gap": 7 },
  "highlight": { "color": [1,1,0,0.3] },
  "level": 0
}
```

**Image**
```json
{
  "type": "image", 
  "src": "https://example/img.png",
  "alt": "demo",
  "align": "left|center|right",
  "width": 160,
  "height": 120,
  "alpha": 0.9
}
```

**Table**
```json
{
  "type": "table",
  "grid_color": [0,0,0,0],
  "grid_size": 1,
  "background_color": [1,1,1,1],
  "rows": [
    [ [{"type": "text", "text": "A"}], [{"type": "text", "text": "B"}] ],
    [ [{"type": "text", "text": "1"}], [{"type": "text", "text": "2"}] ]
  ]
}
```

## API C

### Structures principales

```c
typedef struct {
    float r, g, b, a; // [0.0, 1.0]
} RGBA;

typedef enum { 
    ALIGN_LEFT, ALIGN_CENTER, ALIGN_RIGHT, ALIGN_JUSTIFY 
} Align;

typedef struct {
    char *text;
    char *font;
    Align align;
    int font_size;
    RGBA color;
    bool bold, italic;
    bool has_underline, has_highlight;
    RGBA underline_color; 
    int underline_gap;
    RGBA highlight_color;
    int level; // 0=paragraphe, 1-6=titres
} ElementText;

typedef struct {
    char *src, *alt;
    Align align;
    int width, height;
    float alpha;
} ElementImage;

typedef struct {
    size_t rows, cols;
    ElementText ***cells;
    RGBA grid_color, background_color;
    int grid_size;
} ElementTable;

typedef struct {
    char *name;
    char *default_font; 
    int default_fontsize;
    RGBA default_text_color;
    RGBA default_highlight_color;
    RGBA default_underline_color; 
    int default_underline_gap;
    long created, updated;
    Element *elements; 
    size_t elements_len;
} Document;
```

### Fonctions principales

```c
// Initialisation et libération
void editor_init(Document *doc);
void doc_free(Document *doc);

// Saisie caractère par caractère
void editor_feed_char(Document *doc, unsigned codepoint);
void editor_commit_line(Document *doc);

// Conversions
int json_export_markdown(const Document *doc, char **out_md);
int json_import_markdown(const char *md, Document *out_doc);
int json_stringify(const Document *doc, char **out_json);
int json_parse(const char *json, Document *out_doc);
```

## Compilation

### Prérequis
- GCC ou Clang avec support C11
- Make
- Valgrind (optionnel, pour les tests mémoire)

### Commandes de base

```bash
# Construction de la bibliothèque
make

# Lancer les tests
make test

# Construction avec débogage
make debug

# Tests avec sanitizers
make sanitize

# Tests mémoire avec Valgrind
make valgrind

# Formatage du code
make format

# Nettoyage
make clean
```

## Tests

Le projet inclut des tests complets couvrant :

1. **Fixture 1** - Paragraphe simple avec styles inline
2. **Fixture 2** - Titres avec détection de niveau
3. **Fixture 3** - Image avec attributs
4. **Fixture 4** - Tableaux multi-lignes
5. **Fixture 5** - Surlignage et soulignement
6. **Tests d'intégration** - Round-trip Markdown ↔ JSON
7. **Cas limites** - Entrée malformée, styles non fermés
8. **Tests mémoire** - Aucune fuite avec Valgrind/ASan

### Exécution des tests

```bash
# Tests standard
make test

# Tests avec vérification mémoire
make sanitize && ./run_tests
make valgrind
```

## Règles de conversion

### Saisie utilisateur → JSON
- Caractère par caractère avec `editor_feed_char()`
- `\n` commit la ligne courante
- Titres : `#...#` + espace → niveau détecté, **texte en MAJUSCULE**, bold=true
- Inline : priorité `***` > `**` > `*`, pas d'imbrication croisée
- Marqueurs Markdown **supprimés** du texte final

### JSON → Markdown  
- Titres : préfixe `#...# `, **pas** de forçage uppercase à l'export
- Inline : restauration des marqueurs `**bold**`, `*italic*`, `==highlight==`, `++underline++`
- Images : format `![alt](src){attributs}`
- Tables : format pipe table standard

### Markdown → JSON
- Parcours ligne par ligne avec look-ahead pour les tables
- Détection ordre : table > image > titre > paragraphe
- Normalisation couleurs via `meta.default`

## Architecture

```
src/
├── editor.h/c    - API principale et structures
├── json.h/c      - Parsing/sérialisation JSON
└── markdown.h/c  - Parsing/export Markdown

tests/
└── test_main.c   - Tests unitaires et d'intégration
```

## Cas d'usage

```c
#include "editor.h"

// Création d'un nouveau document
Document doc;
editor_init(&doc);

// Saisie caractère par caractère
const char *input = "# Hello *World*!";
for (size_t i = 0; i < strlen(input); i++) {
    editor_feed_char(&doc, input[i]);
}
editor_commit_line(&doc);

// Export JSON
char *json;
json_stringify(&doc, &json);
printf("JSON: %s\n", json);

// Export Markdown  
char *markdown;
json_export_markdown(&doc, &markdown);
printf("Markdown: %s\n", markdown);

// Nettoyage
doc_free(&doc);
free(json);
free(markdown);
```

## Décisions d'implémentation

### Parsing inline
- Automate à états simple avec pile pour les marqueurs
- Priorité stricte : `***` puis `**` puis `*`
- Pas de regex complexes, tokenization explicite

### Tables
- Détection par motif : ligne + séparateur + données
- Split par `|` avec trim automatique
- Un seul élément `table` par bloc

### Couleurs
- **Toujours** normalisées en float [0.0, 1.0]
- Format JSON canonique : `[r,g,b,a]`
- Aucun support de format 0-255 ou tuples

### Gestion mémoire
- Pas de fuites (testé avec Valgrind)
- Libération explicite avec `doc_free()`
- Allocation dynamique des chaînes

## Limitations connues

1. **v1** : Pas de listes, liens ou blocs de code
2. **Inline** : Pas d'imbrication complexe (volontaire)
3. **Export** : Perte des propriétés couleur/gap pour underline/highlight
4. **Tables** : Pas de fusion de cellules

Ces limitations sont documentées et seront adressées en v2.

## Licence

Domaine public - Aucune restriction d'usage.