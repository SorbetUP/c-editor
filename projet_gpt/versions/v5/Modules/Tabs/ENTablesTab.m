//
//  ENTablesTab.m
//  ElephantNotes V5 - Module Tables pour la création et gestion des tableaux
//

#import "ENTablesTab.h"

@implementation ENTablesTab

- (instancetype)init {
    self = [super initWithName:@"Tables" icon:@"📊"];
    if (self) {
        _currentTableMarkdown = nil;
        _tableRows = 3;
        _tableCols = 3;
        _isEditingTable = NO;
    }
    return self;
}

- (NSString*)generateContent {
    return [self generateTablesContent];
}

- (NSString*)generateTablesContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# 📊 Tables - ElephantNotes V5\n\n"];
    
    // Statut actuel
    if (_isEditingTable && _currentTableMarkdown) {
        [content appendString:@"## ✏️ Éditeur de table actuel\n\n"];
        [content appendFormat:@"**Dimensions :** %ld lignes × %ld colonnes\n\n", (long)_tableRows, (long)_tableCols];
        [content appendString:@"### Aperçu du tableau\n\n"];
        [content appendString:_currentTableMarkdown];
        [content appendString:@"\n\n"];
        
        // Actions d'édition
        [content appendString:@"### 🛠️ Actions d'édition\n"];
        [content appendString:@"- **➕ Ajouter une ligne** : Étendre le tableau verticalement\n"];
        [content appendString:@"- **➕ Ajouter une colonne** : Étendre le tableau horizontalement\n"];
        [content appendString:@"- **❌ Supprimer ligne** : Réduire le nombre de lignes\n"];
        [content appendString:@"- **❌ Supprimer colonne** : Réduire le nombre de colonnes\n"];
        [content appendString:@"- **💾 Enregistrer** : Sauvegarder le tableau dans une note\n"];
        [content appendString:@"- **🔄 Nouveau tableau** : Recommencer avec un tableau vide\n\n"];
    } else {
        [content appendString:@"## 🚀 Nouveau Système de Rendu Ligne par Ligne\n\n"];
        [content appendString:@"**ElephantNotes V5** introduit un système révolutionnaire de rendu de tableaux :\n\n"];
        [content appendString:@"### ✨ Fonctionnalités Avancées\n"];
        [content appendString:@"- **📏 Calcul automatique des largeurs** - Chaque colonne a une largeur optimale\n"];
        [content appendString:@"- **🎯 Cohérence parfaite** - Toutes les lignes partagent les mêmes dimensions\n"];
        [content appendString:@"- **🔄 Rendu ligne par ligne** - Performance optimisée pour grands tableaux\n"];
        [content appendString:@"- **📊 Métadonnées JSON** - Export avec spécifications de colonnes\n\n"];
        
        [content appendString:@"### 🧪 Démonstration - Tableau Simple\n\n"];
        [content appendString:@"| Produit | Prix | Stock |\n"];
        [content appendString:@"|---------|------|-------|\n"];
        [content appendString:@"| MacBook | €2499 | 15 |\n"];
        [content appendString:@"| iPhone | €1199 | 32 |\n"];
        [content appendString:@"| iPad | €899 | 8 |\n\n"];
        
        [content appendString:@"### 🎨 Démonstration - Contenu Variable\n\n"];
        [content appendString:@"| Nom | Description | Responsabilités |\n"];
        [content appendString:@"|-----|-------------|----------------|\n"];
        [content appendString:@"| Alice Martin | Senior Software Engineer | Développement d'applications distribuées, architecture microservices |\n"];
        [content appendString:@"| Bob Wilson | Product Manager | Gestion de roadmap, coordination équipes |\n"];
        [content appendString:@"| Catherine Lee | Data Scientist | Machine learning, analyse prédictive |\n\n"];
        
        [content appendString:@"### 🎯 Démonstration - Alignement\n\n"];
        [content appendString:@"| Gauche | Centre | Droite |\n"];
        [content appendString:@"|:-------|:------:|-------:|\n"];
        [content appendString:@"| Text | Centré | 123.45 |\n"];
        [content appendString:@"| Long texte ici | Court | 9.99 |\n"];
        [content appendString:@"| A | Milieu | 1000.00 |\n\n"];
        
        [content appendString:@"## 🚀 Créateur de tableaux\n\n"];
        [content appendString:@"Créez facilement des tableaux Markdown avec cet outil interactif.\n\n"];
        
        // Boutons de création rapide
        [content appendString:@"### ⚡ Création rapide\n"];
        [content appendString:@"- **📋 Tableau simple (3×3)** : Tableau basique pour débuter\n"];
        [content appendString:@"- **📊 Tableau de données (5×4)** : Idéal pour des statistiques\n"];
        [content appendString:@"- **📝 Liste comparative (4×2)** : Pour comparer des éléments\n"];
        [content appendString:@"- **🎯 Tableau personnalisé** : Choisir les dimensions\n\n"];
    }
    
    // Exemples de tableaux
    [content appendString:[self generateTableExamples]];
    
    // Guide d'utilisation
    [content appendString:[self generateTableHelp]];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V5 - Éditeur de Tables Avancé*\n"];
    [content appendFormat:@"*Support complet des tableaux Markdown avec rendu natif*\n"];
    
    return [content copy];
}

- (NSString*)generateTableExamples {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 📝 Exemples de tableaux\n\n"];
    
    // Exemple 1 : Tableau simple
    [content appendString:@"### 1. Tableau de base\n"];
    [content appendString:@"| Nom | Age | Ville |\n"];
    [content appendString:@"|-----|-----|-------|\n"];
    [content appendString:@"| Alice | 25 | Paris |\n"];
    [content appendString:@"| Bob | 30 | Lyon |\n"];
    [content appendString:@"| Claire | 28 | Nice |\n\n"];
    
    // Exemple 2 : Tableau avec alignement
    [content appendString:@"### 2. Tableau avec alignement\n"];
    [content appendString:@"| Produit | Prix | Stock | Statut |\n"];
    [content appendString:@"|:--------|-----:|:-----:|:------:|\n"];
    [content appendString:@"| MacBook Pro | 2499€ | 12 | ✅ Disponible |\n"];
    [content appendString:@"| iPad Air | 699€ | 25 | ✅ Disponible |\n"];
    [content appendString:@"| iPhone 15 | 959€ | 0 | ❌ Rupture |\n\n"];
    
    // Exemple 3 : Tableau de données
    [content appendString:@"### 3. Tableau de performance\n"];
    [content appendString:@"| Métrique | Q1 2024 | Q2 2024 | Q3 2024 | Évolution |\n"];
    [content appendString:@"|----------|---------|---------|---------|:---------:|\n"];
    [content appendString:@"| Ventes | 125K€ | 156K€ | 198K€ | 📈 +58% |\n"];
    [content appendString:@"| Clients | 450 | 567 | 723 | 📈 +61% |\n"];
    [content appendString:@"| Satisfaction | 4.2/5 | 4.5/5 | 4.7/5 | 📈 +12% |\n\n"];
    
    return [content copy];
}

- (NSString*)generateTableHelp {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 📚 Guide des tableaux Markdown\n\n"];
    
    // Syntaxe de base
    [content appendString:@"### 🔤 Syntaxe de base\n"];
    [content appendString:@"```markdown\n"];
    [content appendString:@"| Colonne 1 | Colonne 2 | Colonne 3 |\n"];
    [content appendString:@"|-----------|-----------|-----------|  ← Ligne de séparation obligatoire\n"];
    [content appendString:@"| Valeur 1  | Valeur 2  | Valeur 3  |\n"];
    [content appendString:@"| Valeur 4  | Valeur 5  | Valeur 6  |\n"];
    [content appendString:@"```\n\n"];
    
    // Alignement
    [content appendString:@"### ↔️ Alignement des colonnes\n"];
    [content appendString:@"| Alignement | Syntaxe | Exemple |\n"];
    [content appendString:@"|------------|---------|----------|\n"];
    [content appendString:@"| Gauche (défaut) | `\\|:---\\|` | Texte normal |\n"];
    [content appendString:@"| Droite | `\\|---:\\|` | 123,45€ |\n"];
    [content appendString:@"| Centré | `\\|:---:\\|` | ✅ Statut |\n\n"];
    
    // Conseils d'utilisation
    [content appendString:@"### 💡 Conseils d'utilisation\n"];
    [content appendString:@"- **Espacement** : Les espaces autour des `|` améliorent la lisibilité\n"];
    [content appendString:@"- **Émojis** : Utilisez des émojis pour rendre vos tableaux plus visuels\n"];
    [content appendString:@"- **Largeur** : Pas besoin d'aligner visuellement, le rendu s'adapte\n"];
    [content appendString:@"- **Contenu** : Évitez les retours à la ligne dans les cellules\n\n"];
    
    // Raccourcis clavier
    [content appendString:@"### ⌨️ Raccourcis clavier (futur)\n"];
    [content appendString:@"| Raccourci | Action |\n"];
    [content appendString:@"|-----------|--------|\n"];
    [content appendString:@"| Tab | Aller à la cellule suivante |\n"];
    [content appendString:@"| Shift+Tab | Aller à la cellule précédente |\n"];
    [content appendString:@"| Ctrl+Entrée | Ajouter une nouvelle ligne |\n"];
    [content appendString:@"| Ctrl+Shift+↑/↓ | Déplacer la ligne |\n\n"];
    
    return [content copy];
}

- (void)createNewTable:(NSInteger)rows columns:(NSInteger)cols {
    _tableRows = rows;
    _tableCols = cols;
    _isEditingTable = YES;
    
    NSMutableString* tableMarkdown = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête du tableau
    [tableMarkdown appendString:@"|"];
    for (NSInteger col = 1; col <= cols; col++) {
        [tableMarkdown appendFormat:@" Colonne %ld |", (long)col];
    }
    [tableMarkdown appendString:@"\n"];
    
    // Ligne de séparation
    [tableMarkdown appendString:@"|"];
    for (NSInteger col = 0; col < cols; col++) {
        [tableMarkdown appendString:@"-----------|"];
    }
    [tableMarkdown appendString:@"\n"];
    
    // Lignes de données
    for (NSInteger row = 1; row < rows; row++) {
        [tableMarkdown appendString:@"|"];
        for (NSInteger col = 1; col <= cols; col++) {
            [tableMarkdown appendFormat:@" Cellule %ld,%ld |", (long)row, (long)col];
        }
        [tableMarkdown appendString:@"\n"];
    }
    
    [_currentTableMarkdown release];
    _currentTableMarkdown = [tableMarkdown copy];
    
    NSLog(@"📊 [ENTablesTab] Nouveau tableau créé : %ldx%ld", (long)rows, (long)cols);
    [self refreshContent];
}

- (void)addTableRow {
    if (!_isEditingTable) return;
    
    _tableRows++;
    
    // Ajouter une nouvelle ligne au markdown existant
    NSMutableString* updatedMarkdown = [[[NSMutableString alloc] initWithString:_currentTableMarkdown] autorelease];
    
    [updatedMarkdown appendString:@"|"];
    for (NSInteger col = 1; col <= _tableCols; col++) {
        [updatedMarkdown appendFormat:@" Nouvelle %ld |", (long)col];
    }
    [updatedMarkdown appendString:@"\n"];
    
    [_currentTableMarkdown release];
    _currentTableMarkdown = [updatedMarkdown copy];
    
    NSLog(@"📊 [ENTablesTab] Ligne ajoutée - Nouveau tableau : %ldx%ld", (long)_tableRows, (long)_tableCols);
    [self refreshContent];
}

- (void)addTableColumn {
    if (!_isEditingTable) return;
    
    _tableCols++;
    
    // Recréer le tableau avec une colonne supplémentaire
    [self createNewTable:_tableRows columns:_tableCols];
    
    NSLog(@"📊 [ENTablesTab] Colonne ajoutée - Nouveau tableau : %ldx%ld", (long)_tableRows, (long)_tableCols);
}

- (void)removeTableRow:(NSInteger)row {
    if (!_isEditingTable || _tableRows <= 2) return; // Garder au moins l'en-tête + 1 ligne
    
    _tableRows--;
    
    // Recréer le tableau avec une ligne en moins
    [self createNewTable:_tableRows columns:_tableCols];
    
    NSLog(@"📊 [ENTablesTab] Ligne supprimée - Nouveau tableau : %ldx%ld", (long)_tableRows, (long)_tableCols);
}

- (void)removeTableColumn:(NSInteger)col {
    if (!_isEditingTable || _tableCols <= 1) return; // Garder au moins 1 colonne
    
    _tableCols--;
    
    // Recréer le tableau avec une colonne en moins
    [self createNewTable:_tableRows columns:_tableCols];
    
    NSLog(@"📊 [ENTablesTab] Colonne supprimée - Nouveau tableau : %ldx%ld", (long)_tableRows, (long)_tableCols);
}

- (NSString*)parseTableFromMarkdown:(NSString*)markdown {
    // Analyse simplifiée d'un tableau Markdown
    if (!markdown || markdown.length == 0) return nil;
    
    NSArray* lines = [markdown componentsSeparatedByString:@"\n"];
    NSMutableArray* tableRows = [[[NSMutableArray alloc] init] autorelease];
    
    for (NSString* line in lines) {
        NSString* trimmedLine = [line stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
        if ([trimmedLine hasPrefix:@"|"] && [trimmedLine hasSuffix:@"|"]) {
            [tableRows addObject:trimmedLine];
        }
    }
    
    return tableRows.count > 0 ? [tableRows componentsJoinedByString:@"\n"] : nil;
}

- (NSString*)formatTableMarkdown:(NSArray*)rows {
    if (!rows || rows.count == 0) return @"";
    
    return [rows componentsJoinedByString:@"\n"];
}

- (void)didBecomeActive {
    NSLog(@"📊 [ENTablesTab] Tables activé");
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"📊 [ENTablesTab] Tables désactivé");
    [super didBecomeInactive];
}

- (void)dealloc {
    [_currentTableMarkdown release];
    [super dealloc];
}

@end