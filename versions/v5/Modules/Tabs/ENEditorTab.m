//
//  ENEditorTab.m
//  ElephantNotes V4 - Module Editor pour l'édition de fichiers
//

#import "ENEditorTab.h"

@interface ENEditorTab () {
    NSString* _defaultCreationDirectory;
}

- (NSString*)resolvedCreationDirectory;
- (NSString*)generateDefaultNoteFileName;

@end

@implementation ENEditorTab

- (instancetype)init {
    self = [super initWithName:@"Editor" icon:@"📝"];
    if (self) {
        _currentFilePath = nil;
        _currentFileName = nil;
        _currentFileContent = nil;
        _hasUnsavedChanges = NO;
        _isFileLoaded = NO;
        _defaultCreationDirectory = nil;
    }
    return self;
}

- (NSString*)generateContent {
    if (_isFileLoaded && _currentFileName) {
        return [self generateEditorContent];
    } else {
        return [self generateWelcomeContent];
    }
}

- (NSString*)generateEditorContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête avec informations du fichier
    [content appendFormat:@"# 📝 Éditeur - %@\n\n", _currentFileName ?: @"Sans titre"];
    
    // Statut du fichier
    if (_hasUnsavedChanges) {
        [content appendString:@"**⚠️ Modifications non sauvegardées**\n\n"];
    } else {
        [content appendString:@"**✅ Fichier sauvegardé**\n\n"];
    }
    
    // Informations du fichier
    [content appendString:@"## 📄 Informations du fichier\n\n"];
    [content appendFormat:@"**Nom :** %@\n", _currentFileName ?: @"Nouveau fichier"];
    [content appendFormat:@"**Chemin :** `%@`\n", _currentFilePath ?: @"Non sauvegardé"];
    [content appendFormat:@"**Taille :** %ld caractères\n\n", 
        (long)(_currentFileContent ? [_currentFileContent length] : 0)];
    
    // Zone d'édition
    [content appendString:@"## ✏️ Contenu du fichier\n\n"];
    [content appendString:@"```markdown\n"];
    if (_currentFileContent && [_currentFileContent length] > 0) {
        [content appendString:_currentFileContent];
    } else {
        [content appendString:@"# Nouveau Document\n\n"];
        [content appendString:@"Commencez à écrire votre contenu ici...\n\n"];
        [content appendString:@"## Section 1\n\n"];
        [content appendString:@"Votre texte ici.\n\n"];
        [content appendString:@"## Section 2\n\n"];
        [content appendString:@"Autre contenu.\n"];
    }
    [content appendString:@"\n```\n\n"];
    
    // Instructions d'utilisation
    [content appendString:@"## 🎯 Comment utiliser l'éditeur\n\n"];
    [content appendString:@"### Édition\n"];
    [content appendString:@"1. **Modifier le contenu** dans le bloc de code ci-dessus\n"];
    [content appendString:@"2. **Sauvegarder** avec ⌘+S pour enregistrer les modifications\n"];
    [content appendString:@"3. **Le contenu sera automatiquement** extrait et sauvegardé\n\n"];
    
    [content appendString:@"### Actions disponibles\n"];
    [content appendString:@"- **⌘+S** : Sauvegarder le fichier actuel\n"];
    [content appendString:@"- **⌘+Shift+S** : Sauvegarder sous un nouveau nom\n"];
    [content appendString:@"- **⌘+N** : Créer un nouveau fichier\n"];
    [content appendString:@"- **⌘+O** : Ouvrir un fichier existant\n"];
    [content appendString:@"- **⌘+W** : Fermer le fichier actuel\n\n"];
    
    // Aperçu du rendu
    if (_currentFileContent && [_currentFileContent length] > 0) {
        [content appendString:@"---\n\n"];
        [content appendString:@"## 👁️ Aperçu du rendu\n\n"];
        [content appendString:_currentFileContent];
        [content appendString:@"\n"];
    }
    
    // Footer
    [content appendString:@"\n---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Éditeur Modulaire*\n"];
    [content appendFormat:@"*Fichier : %@*\n", _currentFileName ?: @"Nouveau document"];
    
    return [content copy];
}

- (NSString*)generateWelcomeContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"# 📝 Éditeur - ElephantNotes V4\n\n"];
    
    [content appendString:@"## 🎉 Bienvenue dans l'éditeur modulaire !\n\n"];
    [content appendString:@"Aucun fichier n'est actuellement ouvert. Choisissez une action :\n\n"];
    
    // Actions rapides
    [content appendString:@"## 🚀 Actions rapides\n\n"];
    [content appendString:@"### 📄 Créer un nouveau fichier\n"];
    [content appendString:@"- Utilisez **⌘+N** pour créer un nouveau document\n"];
    [content appendString:@"- Ou cliquez sur \"Nouveau fichier\" dans le menu\n\n"];
    
    [content appendString:@"### 📂 Ouvrir un fichier existant\n"];
    [content appendString:@"- Utilisez **⌘+O** pour parcourir vos fichiers\n"];
    [content appendString:@"- Ou sélectionnez un fichier dans la liste ci-dessous\n\n"];
    
    // Liste des fichiers récents
    [content appendString:[self generateFileListContent]];
    
    // Conseils d'utilisation
    [content appendString:@"## 💡 Conseils d'utilisation\n\n"];
    [content appendString:@"### Interface modulaire\n"];
    [content appendString:@"- **🏠 Dashboard** : Vue d'ensemble de votre vault\n"];
    [content appendString:@"- **🔍 Recherche** : Trouvez rapidement vos documents\n"];
    [content appendString:@"- **📝 Éditeur** : Créez et modifiez vos notes (cet onglet)\n"];
    [content appendString:@"- **⚙️ Paramètres** : Configuration et gestion des vaults\n\n"];
    
    [content appendString:@"### Markdown natif\n"];
    [content appendString:@"- **Syntaxe simple** : # Titre, **gras**, *italique*\n"];
    [content appendString:@"- **Aperçu en temps réel** : Votre contenu est rendu instantanément\n"];
    [content appendString:@"- **Tableaux, listes, liens** : Support complet du Markdown\n\n"];
    
    [content appendString:@"---\n\n"];
    [content appendString:@"*Prêt à commencer ? Créez votre premier document !*\n"];
    
    return [content copy];
}

- (NSString*)generateFileListContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 📋 Fichiers disponibles\n\n"];
    
    if (!self.currentVaultPath) {
        [content appendString:@"⚠️ **Aucun vault configuré**\n\n"];
        [content appendString:@"Configurez un vault dans les paramètres pour voir vos fichiers.\n\n"];
        return [content copy];
    }
    
    // Chercher les fichiers dans le vault
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    NSArray* files = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
    
    if (error || !files || files.count == 0) {
        [content appendString:@"📭 **Aucun fichier trouvé**\n\n"];
        [content appendString:@"Votre vault est vide. Créez votre première note !\n\n"];
        return [content copy];
    }
    
    // Filtrer et trier les fichiers .md
    NSMutableArray* markdownFiles = [[NSMutableArray alloc] init];
    for (NSString* file in files) {
        if ([[file pathExtension] isEqualToString:@"md"]) {
            NSString* fullPath = [notesPath stringByAppendingPathComponent:file];
            NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
            
            if (attributes) {
                [markdownFiles addObject:@{
                    @"name": [file stringByDeletingPathExtension],
                    @"path": fullPath,
                    @"size": attributes[NSFileSize],
                    @"modified": attributes[NSFileModificationDate]
                }];
            }
        }
    }
    
    // Trier par date de modification
    [markdownFiles sortUsingComparator:^NSComparisonResult(NSDictionary* file1, NSDictionary* file2) {
        NSDate* date1 = file1[@"modified"];
        NSDate* date2 = file2[@"modified"];
        return [date2 compare:date1];
    }];
    
    if (markdownFiles.count > 0) {
        [content appendString:@"| Fichier | Taille | Dernière modification |\n"];
        [content appendString:@"|---------|--------|-----------------------|\n"];
        
        NSInteger maxFiles = MIN(markdownFiles.count, 10);
        for (NSInteger i = 0; i < maxFiles; i++) {
            NSDictionary* file = markdownFiles[i];
            NSString* fileName = file[@"name"];
            NSNumber* fileSize = file[@"size"];
            NSDate* modDate = file[@"modified"];
            
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
            NSString* dateString = [formatter stringFromDate:modDate];
            
            [content appendFormat:@"| **%@** | %@ | %@ |\n", fileName, sizeString, dateString];
        }
        
        if (markdownFiles.count > 10) {
            [content appendFormat:@"\n*... et %ld autres fichiers*\n", (long)(markdownFiles.count - 10)];
        }
    }
    
    [markdownFiles release];
    [content appendString:@"\n"];
    
    return [content copy];
}

- (void)loadFile:(NSString*)filePath {
    if (!filePath) return;
    
    NSError* error = nil;
    NSString* content = [NSString stringWithContentsOfFile:filePath encoding:NSUTF8StringEncoding error:&error];
    
    if (error) {
        NSLog(@"❌ [ENEditorTab] Erreur lors du chargement du fichier: %@", error.localizedDescription);
        return;
    }
    
    [_currentFilePath release];
    _currentFilePath = [filePath copy];

    [_currentFileName release];
    _currentFileName = [[[filePath lastPathComponent] stringByDeletingPathExtension] copy];
    
    [_currentFileContent release];
    _currentFileContent = [content copy];
    
    _hasUnsavedChanges = NO;
    _isFileLoaded = YES;

    NSString* parentDirectory = [_currentFilePath stringByDeletingLastPathComponent];
    if (parentDirectory) {
        [self setDefaultCreationDirectory:parentDirectory];
    }

    NSLog(@"✅ [ENEditorTab] Fichier chargé: %@", _currentFileName);
    [[NSNotificationCenter defaultCenter] postNotificationName:@"ENNoteOpened"
                                                        object:self
                                                      userInfo:@{ @"path": filePath }];
    [self refreshContent];
}

- (void)createNewFile {
    [self prepareNewFileInDirectory:_defaultCreationDirectory];
}

- (void)setDefaultCreationDirectory:(NSString*)directory {
    [_defaultCreationDirectory release];
    _defaultCreationDirectory = [directory copy];
}

- (NSString*)resolvedCreationDirectory {
    if (_defaultCreationDirectory && [_defaultCreationDirectory length] > 0) {
        return _defaultCreationDirectory;
    }
    if (self.currentVaultPath) {
        return [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    }
    return nil;
}

- (NSString*)generateDefaultNoteFileName {
    NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
    [formatter setDateFormat:@"yyyyMMdd-HHmmss"]; // format unique
    NSString* timestamp = [formatter stringFromDate:[NSDate date]];
    return [NSString stringWithFormat:@"note-%@.md", timestamp ?: @"nouveau"];
}

- (void)prepareNewFileInDirectory:(NSString*)directory {
    NSString* targetDirectory = directory ? directory : [self resolvedCreationDirectory];

    if (targetDirectory) {
        BOOL isDir = NO;
        if (![[NSFileManager defaultManager] fileExistsAtPath:targetDirectory isDirectory:&isDir] || !isDir) {
            [[NSFileManager defaultManager] createDirectoryAtPath:targetDirectory
                                      withIntermediateDirectories:YES
                                                       attributes:nil
                                                            error:nil];
        }
    }

    [self setDefaultCreationDirectory:targetDirectory];

    [_currentFilePath release];
    _currentFilePath = nil;

    [_currentFileName release];
    NSString* fileName = [self generateDefaultNoteFileName];
    _currentFileName = [[fileName stringByDeletingPathExtension] copy];

    [_currentFileContent release];
    _currentFileContent = [@"# Nouveau Document\n\nCommencez à écrire votre contenu ici...\n" copy];

    if (_defaultCreationDirectory) {
        NSString* generatedPath = [_defaultCreationDirectory stringByAppendingPathComponent:fileName];
        _currentFilePath = [generatedPath copy];
    }

    _hasUnsavedChanges = YES;
    _isFileLoaded = YES;

    if (_currentFilePath) {
        [[NSNotificationCenter defaultCenter] postNotificationName:@"ENNoteOpened"
                                                            object:self
                                                          userInfo:@{ @"path": _currentFilePath }];
    }

    NSLog(@"📝 [ENEditorTab] Nouveau fichier prêt dans %@", _defaultCreationDirectory ?: @"le vault par défaut");
    [self refreshContent];
}

- (BOOL)openNoteLink:(NSString*)linkPath {
    if (!linkPath || [linkPath length] == 0) {
        return NO;
    }

    NSString* trimmedLink = [linkPath stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
    if ([trimmedLink length] == 0) {
        return NO;
    }

    NSString* resolvedPath = nil;
    if ([trimmedLink hasPrefix:@"file://"]) {
        NSURL* url = [NSURL URLWithString:trimmedLink];
        resolvedPath = [url path];
    } else if ([trimmedLink hasPrefix:@"/"] || [trimmedLink hasPrefix:@"~"]) {
        resolvedPath = [trimmedLink stringByExpandingTildeInPath];
    } else {
        NSString* baseDirectory = nil;
        if (_currentFilePath) {
            baseDirectory = [_currentFilePath stringByDeletingLastPathComponent];
        } else if (_defaultCreationDirectory) {
            baseDirectory = _defaultCreationDirectory;
        } else if (self.currentVaultPath) {
            baseDirectory = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
        }

        if (baseDirectory) {
            resolvedPath = [baseDirectory stringByAppendingPathComponent:trimmedLink];
        } else {
            resolvedPath = trimmedLink;
        }
    }

    resolvedPath = [resolvedPath stringByStandardizingPath];

    NSFileManager* fileManager = [NSFileManager defaultManager];
    if (![fileManager fileExistsAtPath:resolvedPath]) {
        if ([[resolvedPath pathExtension] length] == 0) {
            NSString* candidate = [resolvedPath stringByAppendingPathExtension:@"md"];
            if ([fileManager fileExistsAtPath:candidate]) {
                resolvedPath = candidate;
            }
        }
    }

    if (![fileManager fileExistsAtPath:resolvedPath]) {
        return NO;
    }

    [self loadFile:resolvedPath];
    return YES;
}

- (void)saveCurrentFile {
    if (!_isFileLoaded || !_currentFileContent) {
        NSLog(@"⚠️ [ENEditorTab] Aucun fichier à sauvegarder");
        return;
    }
    
    NSString* savePath = _currentFilePath;
    
    // Si pas de chemin, utiliser le répertoire de création par défaut
    if (!savePath) {
        NSString* notesDir = [self resolvedCreationDirectory];
        if (notesDir) {
            BOOL isDir = NO;
            if (![[NSFileManager defaultManager] fileExistsAtPath:notesDir isDirectory:&isDir] || !isDir) {
                [[NSFileManager defaultManager] createDirectoryAtPath:notesDir
                                          withIntermediateDirectories:YES
                                                           attributes:nil
                                                                error:nil];
            }
            NSString* fileName = _currentFileName ?: @"nouveau-document";
            savePath = [notesDir stringByAppendingPathComponent:[fileName stringByAppendingPathExtension:@"md"]];
        }
    }
    
    if (!savePath) {
        NSLog(@"❌ [ENEditorTab] Impossible de déterminer le chemin de sauvegarde");
        return;
    }
    
    NSError* error = nil;
    BOOL success = [_currentFileContent writeToFile:savePath 
                                        atomically:YES 
                                          encoding:NSUTF8StringEncoding 
                                             error:&error];
    
    if (success) {
        [_currentFilePath release];
        _currentFilePath = [savePath copy];
        _hasUnsavedChanges = NO;
        
        NSLog(@"✅ [ENEditorTab] Fichier sauvegardé: %@", savePath);
        [[NSNotificationCenter defaultCenter] postNotificationName:@"ENNoteOpened"
                                                            object:self
                                                          userInfo:@{ @"path": savePath }];
        [self refreshContent];
    } else {
        NSLog(@"❌ [ENEditorTab] Erreur de sauvegarde: %@", error.localizedDescription);
    }
}

- (void)updateFileContent:(NSString*)content {
    if (!content) return;
    
    [_currentFileContent release];
    _currentFileContent = [content copy];
    _hasUnsavedChanges = YES;
    
    NSLog(@"📝 [ENEditorTab] Contenu mis à jour (%ld caractères)", (long)[content length]);
}

- (void)handleContentSave:(NSString*)content {
    // Extraire le contenu markdown du bloc de code
    NSString* markdownContent = [self extractMarkdownFromContent:content];
    
    if (markdownContent && [markdownContent length] > 0) {
        [self updateFileContent:markdownContent];
        [self saveCurrentFile];
    }
    
    [super handleContentSave:content];
}

- (NSString*)extractMarkdownFromContent:(NSString*)content {
    // Chercher le contenu dans le bloc de code markdown
    NSArray* lines = [content componentsSeparatedByString:@"\n"];
    NSMutableString* extractedContent = [[[NSMutableString alloc] init] autorelease];
    bool inMarkdownBlock = false;
    bool foundMarkdownBlock = false;
    
    for (NSString* line in lines) {
        NSString* trimmedLine = [line stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
        
        if ([trimmedLine isEqualToString:@"```markdown"]) {
            inMarkdownBlock = true;
            foundMarkdownBlock = true;
            continue;
        }
        
        if ([trimmedLine hasPrefix:@"```"] && inMarkdownBlock) {
            break;
        }
        
        if (inMarkdownBlock) {
            [extractedContent appendString:line];
            [extractedContent appendString:@"\n"];
        }
    }
    
    if (foundMarkdownBlock && [extractedContent length] > 0) {
        // Retirer le dernier \n
        return [extractedContent substringToIndex:[extractedContent length] - 1];
    }
    
    return nil;
}

- (void)saveAsNewFile:(NSString*)filePath {
    if (!_isFileLoaded || !_currentFileContent || !filePath) {
        NSLog(@"⚠️ [ENEditorTab] Paramètres invalides pour sauvegarder sous");
        return;
    }
    
    NSError* error = nil;
    BOOL success = [_currentFileContent writeToFile:filePath 
                                        atomically:YES 
                                          encoding:NSUTF8StringEncoding 
                                             error:&error];
    
    if (success) {
        [_currentFilePath release];
        _currentFilePath = [filePath copy];
        
        [_currentFileName release];
        _currentFileName = [[[filePath lastPathComponent] stringByDeletingPathExtension] copy];

        _hasUnsavedChanges = NO;
        
        NSLog(@"✅ [ENEditorTab] Fichier sauvegardé sous: %@", filePath);
        [[NSNotificationCenter defaultCenter] postNotificationName:@"ENNoteOpened"
                                                            object:self
                                                          userInfo:@{ @"path": filePath }];
        [self refreshContent];
    } else {
        NSLog(@"❌ [ENEditorTab] Erreur lors de la sauvegarde: %@", error.localizedDescription);
    }
}

- (void)closeCurrentFile {
    if (_hasUnsavedChanges) {
        NSLog(@"⚠️ [ENEditorTab] Attention: modifications non sauvegardées");
        // Dans une vraie application, on afficherait une alerte
    }
    
    [_currentFilePath release];
    _currentFilePath = nil;
    
    [_currentFileName release];
    _currentFileName = nil;
    
    [_currentFileContent release];
    _currentFileContent = nil;
    
    _hasUnsavedChanges = NO;
    _isFileLoaded = NO;
    
    NSLog(@"📝 [ENEditorTab] Fichier fermé");
    [self refreshContent];
}

- (void)didBecomeActive {
    if (!_isFileLoaded) {
        [self prepareNewFileInDirectory:nil];
    }
    NSLog(@"📝 [ENEditorTab] Editor activé");
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"📝 [ENEditorTab] Editor désactivé");
    [super didBecomeInactive];
}

- (void)dealloc {
    [_currentFilePath release];
    [_currentFileName release];
    [_currentFileContent release];
    [_defaultCreationDirectory release];
    [super dealloc];
}

@end
