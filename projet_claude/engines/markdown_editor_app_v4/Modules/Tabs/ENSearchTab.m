//
//  ENSearchTab.m
//  ElephantNotes V4 - Module Search (interface markdown)
//

#import "ENSearchTab.h"

@implementation ENSearchTab

- (instancetype)init {
    self = [super initWithName:@"Search" icon:@"🔍"];
    if (self) {
        _searchIndexReady = false;
        _isSearching = false;
        _searchResults = [[NSArray alloc] init];
        _currentQuery = nil;
    }
    return self;
}

- (NSString*)generateContent {
    return [self generateSearchContent];
}

- (NSString*)generateSearchContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# 🔍 Recherche - ElephantNotes V4\n\n"];
    
    // Si on a des résultats de recherche, les afficher en premier
    if (_currentQuery && [_currentQuery length] > 0) {
        NSString* resultsContent = [self formatSearchResults:_searchResults forQuery:_currentQuery];
        [content appendString:resultsContent];
        [content appendString:@"\n---\n\n"];
    }
    
    // Section de recherche
    [content appendString:@"## 🔎 Rechercher dans vos notes\n\n"];
    
    if (_isSearching) {
        [content appendString:@"**🔄 Recherche en cours...**\n\n"];
    } else if (_currentQuery && [_currentQuery length] > 0) {
        [content appendFormat:@"**Dernière recherche :** `%@`\n\n", _currentQuery];
        [content appendString:@"**Nouvelle recherche :**\n\n"];
    } else {
        [content appendString:@"**Zone de recherche :**\n\n"];
    }
    
    [content appendString:@"```\n"];
    if (_currentQuery && [_currentQuery length] > 0) {
        [content appendFormat:@"%@\n", _currentQuery];
        [content appendString:@"\n# Pour effectuer une nouvelle recherche, modifiez le texte ci-dessus\n"];
        [content appendString:@"# puis sauvegardez (⌘+S) pour lancer la recherche\n"];
    } else {
        [content appendString:@"Tapez votre recherche ici...\n"];
        [content appendString:@"\n# Exemples de recherches :\n"];
        [content appendString:@"# TODO\n"];
        [content appendString:@"# \"phrase exacte\"\n"];
        [content appendString:@"# projet important\n"];
    }
    [content appendString:@"```\n\n"];
    
    // Informations sur la recherche
    if (self.currentVaultPath) {
        [content appendFormat:@"**Vault actuel :** %@\n", self.currentVaultName ?: @"Sans nom"];
        [content appendFormat:@"**Chemin :** `%@`\n", self.currentVaultPath];
        
        // Statistiques de recherche
        if (_searchIndexReady) {
            NSArray* fileList = [self getAllFilesInVault];
            [content appendFormat:@"**Fichiers indexés :** %ld\n", (long)fileList.count];
            [content appendFormat:@"**Index pret :** %@\n\n", _searchIndexReady ? @"✅ Oui" : @"🔄 En cours..."];
        } else {
            [content appendString:@"**Index pret :** ❌ Non initialisé\n\n"];
        }
        
        // Charger la liste des fichiers
        NSArray* fileList = [self getAllFilesInVault];
        if (fileList.count > 0) {
            [content appendString:@"## 📄 Fichiers disponibles\n\n"];
            [content appendString:@"| Fichier | Taille | Dernière modification |\n"];
            [content appendString:@"|---------|--------|-----------------------|\n"];
            
            NSInteger maxFiles = MIN(fileList.count, 20); // Limiter à 20 fichiers
            for (NSInteger i = 0; i < maxFiles; i++) {
                NSDictionary* file = fileList[i];
                [content appendFormat:@"| **%@** | %@ | %@ |\n", 
                    file[@"name"] ?: @"Sans nom",
                    file[@"size"] ?: @"0 B",
                    file[@"date"] ?: @"Inconnue"];
            }
            
            if (fileList.count > 20) {
                [content appendFormat:@"\n*... et %ld autres fichiers*\n", (long)(fileList.count - 20)];
            }
            [content appendString:@"\n"];
        } else {
            [content appendString:@"## 📄 Aucun fichier trouvé\n\n"];
            [content appendString:@"Votre vault ne contient aucune note pour le moment.\n"];
            [content appendString:@"Créez votre première note avec **⌘+N**.\n\n"];
        }
    } else {
        [content appendString:@"⚠️ **Aucun vault configuré**\n\n"];
        [content appendString:@"Vous devez d'abord configurer un vault pour utiliser la recherche.\n"];
        [content appendString:@"Utilisez **⌘+V** pour créer ou sélectionner un vault.\n\n"];
    }
    
    // Instructions d'utilisation
    [content appendString:@"## 🎯 Comment utiliser la recherche\n\n"];
    [content appendString:@"### Types de recherche supportés\n"];
    [content appendString:@"- **Recherche simple :** `mot clé`\n"];
    [content appendString:@"- **Recherche par phrase :** `\"phrase exacte\"`\n"];
    [content appendString:@"- **Recherche par extension :** `*.md`\n"];
    [content appendString:@"- **Recherche dans le contenu :** Toutes les notes sont indexées\n\n"];
    
    [content appendString:@"### Navigation\n"];
    [content appendString:@"- Utilisez la zone de recherche ci-dessus pour commencer\n"];
    [content appendString:@"- Les résultats s'afficheront automatiquement\n"];
    [content appendString:@"- Cliquez sur un fichier pour l'ouvrir\n"];
    [content appendString:@"- Utilisez **⬅️ Retour** pour revenir à l'écran précédent\n\n"];
    
    // Raccourcis clavier
    [content appendString:@"## ⌨️ Raccourcis de recherche\n\n"];
    [content appendString:@"| Raccourci | Action |\n"];
    [content appendString:@"|-----------|--------|\n"];
    [content appendString:@"| ⌘+F | Focus sur la recherche |\n"];
    [content appendString:@"| Échap | Effacer la recherche |\n"];
    [content appendString:@"| ↑/↓ | Naviguer dans les résultats |\n"];
    [content appendString:@"| Entrée | Ouvrir le fichier sélectionné |\n\n"];
    
    // Performance et statistiques  
    if (_searchIndexReady) {
        [content appendString:@"## 📊 Statistiques de performance\n\n"];
        [content appendString:@"- **Mode :** Recherche native simplifiée\n"];
        [content appendString:@"- **Recherche :** Noms de fichiers et contenu\n"];
        [content appendString:@"- **Performance :** Optimisée pour petits vaults\n"];
        [content appendString:@"- **Fonctionnalités :** Recherche textuelle complète\n\n"];
    }
    
    // Zone de saisie
    [content appendString:@"---\n\n"];
    [content appendString:@"## ✏️ Zone de recherche interactive\n\n"];
    [content appendString:@"Utilisez l'éditeur ci-dessous pour taper votre recherche :\n\n"];
    [content appendString:@"```\n"];
    [content appendString:@"🔍 Votre recherche ici...\n\n"];
    [content appendString:@"Exemples :\n"];
    [content appendString:@"- TODO\n"];
    [content appendString:@"- \"note importante\"\n"];
    [content appendString:@"- *.md\n"];
    [content appendString:@"- projet markdown\n"];
    [content appendString:@"```\n\n"];
    
    // Architecture info
    [content appendString:@"---\n\n"];
    [content appendString:@"## 🏗️ Architecture V4 - Module Search\n\n"];
    [content appendString:@"Cette interface de recherche utilise :\n\n"];
    [content appendString:@"- **ENSearchTab** : Module search modulaire\n"];
    [content appendString:@"- **ui_framework_set_editor_content()** : Affichage natif\n"];
    [content appendString:@"- **Pas d'interface personnalisée** : Préserve la sidebar\n"];
    [content appendString:@"- **Contenu markdown** : Interface simple et efficace\n\n"];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Recherche Modulaire*\n"];
    [content appendFormat:@"*Interface générée le %@*\n", [[NSDate date] description]];
    
    return [content copy];
}

- (NSArray*)getAllFilesInVault {
    if (!self.currentVaultPath) return @[];
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSMutableArray* result = [[[NSMutableArray alloc] init] autorelease];
    
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    
    NSDirectoryEnumerator* enumerator = [fileManager enumeratorAtPath:notesPath];
    NSString* fileName;
    
    while ((fileName = [enumerator nextObject])) {
        NSString* fullPath = [notesPath stringByAppendingPathComponent:fileName];
        
        // Ignorer les dossiers et ne garder que les fichiers .md
        BOOL isDirectory;
        if ([fileManager fileExistsAtPath:fullPath isDirectory:&isDirectory] && !isDirectory) {
            if ([[fileName pathExtension] isEqualToString:@"md"]) {
                NSError* error = nil;
                NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:&error];
                
                if (attributes && !error) {
                    NSNumber* fileSize = attributes[NSFileSize];
                    NSDate* modificationDate = attributes[NSFileModificationDate];
                    
                    // Formater la taille
                    NSString* sizeString;
                    NSUInteger size = [fileSize unsignedIntegerValue];
                    if (size < 1024) {
                        sizeString = [NSString stringWithFormat:@"%lu B", (unsigned long)size];
                    } else if (size < 1024 * 1024) {
                        sizeString = [NSString stringWithFormat:@"%.1f KB", size / 1024.0];
                    } else {
                        sizeString = [NSString stringWithFormat:@"%.1f MB", size / (1024.0 * 1024.0)];
                    }
                    
                    // Formater la date
                    NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
                    [formatter setDateStyle:NSDateFormatterShortStyle];
                    [formatter setTimeStyle:NSDateFormatterShortStyle];
                    NSString* dateString = [formatter stringFromDate:modificationDate];
                    
                    [result addObject:@{
                        @"name": [fileName stringByDeletingPathExtension],
                        @"fullPath": fullPath,
                        @"size": sizeString,
                        @"date": dateString,
                        @"modificationDate": modificationDate
                    }];
                }
            }
        }
    }
    
    // Trier par date de modification (plus récent en premier)
    [result sortUsingComparator:^NSComparisonResult(NSDictionary* file1, NSDictionary* file2) {
        NSDate* date1 = file1[@"modificationDate"];
        NSDate* date2 = file2[@"modificationDate"];
        return [date2 compare:date1];  // Ordre décroissant
    }];
    
    return [[result copy] autorelease];
}

- (void)initializeSearchEngine {
    if (!self.currentVaultPath) {
        NSLog(@"⚠️ [ENSearchTab] Aucun vault configuré pour initialiser la recherche");
        return;
    }
    
    // Pour l'instant, utiliser une approche simplifiée
    // TODO: Intégrer le vrai moteur de recherche avancée quand il sera prêt
    _searchEngine = NULL; // Pas de moteur pour l'instant
    _searchIndexReady = true; // Utiliser la recherche de fichiers native
    
    NSLog(@"✅ [ENSearchTab] Recherche native initialisée (mode simple)");
}

- (void)didBecomeActive {
    NSLog(@"🔍 [ENSearchTab] Search activé");
    [self initializeSearchEngine];
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"🔍 [ENSearchTab] Search désactivé");
    [super didBecomeInactive];
}

- (void)performSearch:(NSString*)query {
    if (!query || [query length] == 0) {
        [self clearSearch];
        return;
    }
    
    if (!_searchIndexReady) {
        NSLog(@"⚠️ [ENSearchTab] Moteur de recherche non prêt");
        return;
    }
    
    _isSearching = true;
    [_currentQuery release];
    _currentQuery = [query copy];
    
    NSLog(@"🔍 [ENSearchTab] Recherche pour: '%@'", query);
    
    // Effectuer une recherche simple dans les fichiers
    NSArray* allFiles = [self getAllFilesInVault];
    NSMutableArray* searchResults = [[NSMutableArray alloc] init];
    
    for (NSDictionary* file in allFiles) {
        NSString* fileName = file[@"name"];
        NSString* filePath = file[@"fullPath"];
        
        // Recherche dans le nom du fichier
        BOOL matchInName = [fileName localizedCaseInsensitiveContainsString:query];
        
        // Recherche dans le contenu du fichier
        NSString* content = nil;
        BOOL matchInContent = NO;
        NSString* preview = @"";
        float relevanceScore = 0.0f;
        
        if ([filePath length] > 0) {
            NSError* error = nil;
            content = [NSString stringWithContentsOfFile:filePath encoding:NSUTF8StringEncoding error:&error];
            
            if (!error && content) {
                matchInContent = [content localizedCaseInsensitiveContainsString:query];
                
                if (matchInContent) {
                    // Extraire un aperçu autour de la correspondance
                    NSRange matchRange = [content rangeOfString:query options:NSCaseInsensitiveSearch];
                    if (matchRange.location != NSNotFound) {
                        NSInteger start = MAX(0, (NSInteger)matchRange.location - 50);
                        NSInteger length = MIN(100, (NSInteger)[content length] - start);
                        NSRange previewRange = NSMakeRange(start, length);
                        preview = [content substringWithRange:previewRange];
                        
                        // Nettoyer l'aperçu
                        preview = [preview stringByReplacingOccurrencesOfString:@"\n" withString:@" "];
                        preview = [preview stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
                    }
                }
            }
        }
        
        // Calculer le score de pertinence
        if (matchInName && matchInContent) {
            relevanceScore = 1.0f; // Match parfait
        } else if (matchInName) {
            relevanceScore = 0.8f; // Match dans le nom
        } else if (matchInContent) {
            relevanceScore = 0.6f; // Match dans le contenu
        }
        
        // Ajouter aux résultats si on a trouvé quelque chose
        if (matchInName || matchInContent) {
            NSDictionary* resultDict = @{
                @"filePath": filePath ?: @"",
                @"fileName": fileName ?: @"Sans nom",
                @"preview": preview ?: @"",
                @"relevanceScore": @(relevanceScore),
                @"fileSize": file[@"size"] ?: @"0 B",
                @"lastModified": file[@"date"] ?: @"Inconnue",
                @"matchType": matchInName ? @"filename" : @"content",
                @"matchPosition": @(0)
            };
            
            [searchResults addObject:resultDict];
        }
    }
    
    // Trier par score de pertinence
    [searchResults sortUsingComparator:^NSComparisonResult(NSDictionary* result1, NSDictionary* result2) {
        NSNumber* score1 = result1[@"relevanceScore"];
        NSNumber* score2 = result2[@"relevanceScore"];
        return [score2 compare:score1]; // Ordre décroissant
    }];
    
    [_searchResults release];
    _searchResults = [searchResults copy];
    [searchResults release];
    
    NSLog(@"✅ [ENSearchTab] %ld résultats trouvés", (long)[_searchResults count]);
    
    _isSearching = false;
    
    // Actualiser l'affichage
    [self refreshContent];
}

- (void)clearSearch {
    [_currentQuery release];
    _currentQuery = nil;
    
    [_searchResults release];
    _searchResults = [[NSArray alloc] init];
    
    _isSearching = false;
    
    NSLog(@"🧹 [ENSearchTab] Recherche effacée");
    [self refreshContent];
}

- (NSString*)formatSearchResults:(NSArray*)results forQuery:(NSString*)query {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendFormat:@"## 🎯 Résultats pour \"%@\"\n\n", query];
    
    if (results.count == 0) {
        [content appendString:@"**Aucun résultat trouvé.**\n\n"];
        [content appendString:@"### 💡 Suggestions :\n"];
        [content appendString:@"- Vérifiez l'orthographe de votre recherche\n"];
        [content appendString:@"- Essayez des mots-clés plus généraux\n"];
        [content appendString:@"- Utilisez des termes différents\n"];
        [content appendString:@"- Recherchez une phrase exacte avec des guillemets\n\n"];
        return [content copy];
    }
    
    [content appendFormat:@"**%ld résultat%@ trouvé%@**\n\n", 
        (long)results.count, 
        results.count > 1 ? @"s" : @"",
        results.count > 1 ? @"s" : @""];
    
    // Afficher les résultats sous forme de tableau
    [content appendString:@"| Score | Fichier | Aperçu | Type |\n"];
    [content appendString:@"|-------|---------|--------|------|\n"];
    
    for (NSDictionary* result in results) {
        NSString* fileName = result[@"fileName"] ?: @"Sans nom";
        NSString* preview = result[@"preview"] ?: @"";
        NSString* matchType = result[@"matchType"] ?: @"exact";
        NSNumber* score = result[@"relevanceScore"] ?: @(0.0);
        
        // Limiter l'aperçu à 60 caractères
        if ([preview length] > 60) {
            preview = [[preview substringToIndex:57] stringByAppendingString:@"..."];
        }
        
        // Remplacer les pipes dans le contenu pour éviter de casser le tableau
        preview = [preview stringByReplacingOccurrencesOfString:@"|" withString:@"⎪"];
        
        // Formater le score sous forme de pourcentage
        NSString* scorePercent = [NSString stringWithFormat:@"%.0f%%", [score floatValue] * 100];
        
        [content appendFormat:@"| %@ | **%@** | %@ | %@ |\n", 
            scorePercent, fileName, preview, matchType];
    }
    
    [content appendString:@"\n"];
    
    // Instructions d'interaction
    [content appendString:@"### 📖 Comment utiliser ces résultats\n\n"];
    [content appendString:@"1. **Copier le nom** du fichier qui vous intéresse\n"];
    [content appendString:@"2. **Utiliser ⌘+O** pour ouvrir un fichier par son nom\n"];
    [content appendString:@"3. **Affiner votre recherche** si nécessaire\n"];
    [content appendString:@"4. **Utiliser les guillemets** pour une recherche exacte\n\n"];
    
    return [content copy];
}

- (void)handleContentSave:(NSString*)content {
    // Override pour déclencher une recherche quand l'utilisateur sauvegarde
    if (!content || [content length] == 0) {
        [self clearSearch];
        return;
    }
    
    // Extraire la requête de recherche du contenu markdown
    NSString* searchQuery = [self extractSearchQueryFromContent:content];
    
    if (searchQuery && [searchQuery length] > 0) {
        NSLog(@"🔎 [ENSearchTab] Recherche déclenchée via sauvegarde: '%@'", searchQuery);
        [self performSearch:searchQuery];
    } else {
        NSLog(@"ℹ️ [ENSearchTab] Aucune requête trouvée dans le contenu sauvegardé");
    }
    
    // Appeler la méthode parent
    [super handleContentSave:content];
}

- (NSString*)extractSearchQueryFromContent:(NSString*)content {
    // Chercher la première ligne non-commentée dans un bloc de code
    NSArray* lines = [content componentsSeparatedByString:@"\n"];
    bool inCodeBlock = false;
    
    for (NSString* line in lines) {
        NSString* trimmedLine = [line stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
        
        // Détecter le début/fin d'un bloc de code
        if ([trimmedLine hasPrefix:@"```"]) {
            inCodeBlock = !inCodeBlock;
            continue;
        }
        
        // Si on est dans un bloc de code et que la ligne n'est pas un commentaire
        if (inCodeBlock && [trimmedLine length] > 0 && ![trimmedLine hasPrefix:@"#"]) {
            // C'est notre requête de recherche
            return trimmedLine;
        }
    }
    
    return nil;
}

- (void)dealloc {
    // Note: _searchEngine est NULL dans cette version simplifiée
    [_currentQuery release];
    [_searchResults release];
    [super dealloc];
}

@end