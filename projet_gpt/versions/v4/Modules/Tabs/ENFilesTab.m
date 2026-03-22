//
//  ENFilesTab.m
//  ElephantNotes V4 - Module Files pour la gestion des fichiers et dossiers
//

#import "ENFilesTab.h"

@implementation ENFilesTab

- (instancetype)init {
    self = [super initWithName:@"Files" icon:@"📁"];
    if (self) {
        _currentDirectory = nil;
        _currentFiles = nil;
        _showHiddenFiles = NO;
    }
    return self;
}

- (NSString*)generateContent {
    return [self generateFilesContent];
}

- (NSString*)generateFilesContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# 📁 Gestionnaire de Fichiers - ElephantNotes V4\n\n"];
    
    // Navigation actuelle
    NSString* currentPath = _currentDirectory ?: self.currentVaultPath;
    if (currentPath) {
        [content appendFormat:@"## 📍 Répertoire actuel\n\n"];
        [content appendFormat:@"**Chemin :** `%@`\n\n", currentPath];
        
        // Boutons de navigation
        [content appendString:@"### 🧭 Navigation\n"];
        [content appendString:@"- **⬆️ Dossier parent** : Remonter d'un niveau\n"];
        [content appendString:@"- **🏠 Racine du vault** : Retour à la racine\n"];
        [content appendString:@"- **🔄 Actualiser** : Recharger la liste des fichiers\n\n"];
        
        // Liste des fichiers et dossiers
        [content appendString:[self generateDirectoryListContent]];
    } else {
        [content appendString:@"⚠️ **Aucun vault configuré**\n\n"];
        [content appendString:@"Configurez un vault dans les paramètres pour accéder aux fichiers.\n\n"];
    }
    
    // Gestion des vaults
    [content appendString:[self generateVaultManagementContent]];
    
    // Actions rapides
    [content appendString:@"## ⚡ Actions rapides\n\n"];
    [content appendString:@"### Fichiers\n"];
    [content appendString:@"- **⌘+N** : Créer une nouvelle note\n"];
    [content appendString:@"- **⌘+Shift+N** : Créer un nouveau dossier\n"];
    [content appendString:@"- **Suppr** : Supprimer le fichier sélectionné\n"];
    [content appendString:@"- **F2** : Renommer le fichier sélectionné\n\n"];
    
    [content appendString:@"### Vaults\n"];
    [content appendString:@"- **⌘+V** : Gestionnaire de vaults\n"];
    [content appendString:@"- **⌘+Shift+V** : Créer un nouveau vault\n"];
    [content appendString:@"- **⌘+O** : Ouvrir un vault existant\n\n"];
    
    // Informations du système de fichiers
    if (currentPath) {
        [content appendString:@"## 📊 Informations\n\n"];
        
        NSFileManager* fileManager = [NSFileManager defaultManager];
        NSError* error = nil;
        NSArray* files = [fileManager contentsOfDirectoryAtPath:currentPath error:&error];
        
        if (!error && files) {
            NSInteger totalFiles = 0;
            NSInteger totalFolders = 0;
            NSInteger markdownFiles = 0;
            uint64_t totalSize = 0;
            
            for (NSString* file in files) {
                if (_showHiddenFiles || ![file hasPrefix:@"."]) {
                    NSString* fullPath = [currentPath stringByAppendingPathComponent:file];
                    BOOL isDirectory;
                    if ([fileManager fileExistsAtPath:fullPath isDirectory:&isDirectory]) {
                        if (isDirectory) {
                            totalFolders++;
                        } else {
                            totalFiles++;
                            if ([[file pathExtension] isEqualToString:@"md"]) {
                                markdownFiles++;
                            }
                            
                            NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
                            if (attributes) {
                                totalSize += [attributes[NSFileSize] unsignedLongLongValue];
                            }
                        }
                    }
                }
            }
            
            [content appendFormat:@"- **Dossiers :** %ld\n", (long)totalFolders];
            [content appendFormat:@"- **Fichiers :** %ld\n", (long)totalFiles];
            [content appendFormat:@"- **Notes Markdown :** %ld\n", (long)markdownFiles];
            
            // Formater la taille totale
            NSString* sizeString;
            if (totalSize < 1024) {
                sizeString = [NSString stringWithFormat:@"%llu B", totalSize];
            } else if (totalSize < 1024 * 1024) {
                sizeString = [NSString stringWithFormat:@"%.1f KB", totalSize / 1024.0];
            } else if (totalSize < 1024 * 1024 * 1024) {
                sizeString = [NSString stringWithFormat:@"%.1f MB", totalSize / (1024.0 * 1024.0)];
            } else {
                sizeString = [NSString stringWithFormat:@"%.1f GB", totalSize / (1024.0 * 1024.0 * 1024.0)];
            }
            [content appendFormat:@"- **Taille totale :** %@\n\n", sizeString];
        }
    }
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Gestionnaire de Fichiers Modulaire*\n"];
    [content appendFormat:@"*Répertoire : %@*\n", currentPath ?: @"Non configuré"];
    
    return [content copy];
}

- (NSString*)generateDirectoryListContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    NSString* targetPath = _currentDirectory ?: self.currentVaultPath;
    if (!targetPath) {
        return @"";
    }
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    NSArray* files = [fileManager contentsOfDirectoryAtPath:targetPath error:&error];
    
    if (error) {
        [content appendFormat:@"❌ **Erreur lors de la lecture du répertoire :** %@\n\n", error.localizedDescription];
        return [content copy];
    }
    
    if (!files || files.count == 0) {
        [content appendString:@"📭 **Répertoire vide**\n\n"];
        [content appendString:@"Ce dossier ne contient aucun fichier.\n\n"];
        return [content copy];
    }
    
    // Séparer les dossiers et fichiers
    NSMutableArray* directories = [[NSMutableArray alloc] init];
    NSMutableArray* regularFiles = [[NSMutableArray alloc] init];
    
    for (NSString* file in files) {
        if (!_showHiddenFiles && [file hasPrefix:@"."]) {
            continue;
        }
        
        NSString* fullPath = [targetPath stringByAppendingPathComponent:file];
        BOOL isDirectory;
        if ([fileManager fileExistsAtPath:fullPath isDirectory:&isDirectory]) {
            NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
            NSDate* modDate = attributes[NSFileModificationDate];
            NSNumber* size = attributes[NSFileSize];
            
            NSDictionary* fileInfo = @{
                @"name": file,
                @"path": fullPath,
                @"isDirectory": @(isDirectory),
                @"size": size ?: @(0),
                @"modified": modDate ?: [NSDate date]
            };
            
            if (isDirectory) {
                [directories addObject:fileInfo];
            } else {
                [regularFiles addObject:fileInfo];
            }
        }
    }
    
    // Trier par nom
    NSComparator sortComparator = ^NSComparisonResult(NSDictionary* file1, NSDictionary* file2) {
        return [file1[@"name"] localizedCaseInsensitiveCompare:file2[@"name"]];
    };
    
    [directories sortUsingComparator:sortComparator];
    [regularFiles sortUsingComparator:sortComparator];
    
    [content appendString:@"## 📋 Contenu du répertoire\n\n"];
    [content appendString:@"| Type | Nom | Taille | Dernière modification |\n"];
    [content appendString:@"|------|-----|--------|-----------------------|\n"];
    
    // Afficher les dossiers en premier
    for (NSDictionary* dir in directories) {
        NSString* name = dir[@"name"];
        NSDate* modDate = dir[@"modified"];
        
        NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
        [formatter setDateStyle:NSDateFormatterShortStyle];
        [formatter setTimeStyle:NSDateFormatterShortStyle];
        NSString* dateString = [formatter stringFromDate:modDate];
        
        [content appendFormat:@"| 📁 | **%@** | — | %@ |\n", name, dateString];
    }
    
    // Puis les fichiers
    for (NSDictionary* file in regularFiles) {
        NSString* name = file[@"name"];
        NSNumber* fileSize = file[@"size"];
        NSDate* modDate = file[@"modified"];
        
        // Icône selon l'extension
        NSString* icon = @"📄";
        NSString* extension = [[name pathExtension] lowercaseString];
        if ([extension isEqualToString:@"md"] || [extension isEqualToString:@"markdown"]) {
            icon = @"📝";
        } else if ([extension isEqualToString:@"txt"]) {
            icon = @"📃";
        } else if ([extension isEqualToString:@"pdf"]) {
            icon = @"📕";
        } else if ([extension isEqualToString:@"jpg"] || [extension isEqualToString:@"png"] || [extension isEqualToString:@"gif"]) {
            icon = @"🖼️";
        }
        
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
        
        NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
        [formatter setDateStyle:NSDateFormatterShortStyle];
        [formatter setTimeStyle:NSDateFormatterShortStyle];
        NSString* dateString = [formatter stringFromDate:modDate];
        
        [content appendFormat:@"| %@ | **%@** | %@ | %@ |\n", icon, name, sizeString, dateString];
    }
    
    [content appendString:@"\n"];
    
    [directories release];
    [regularFiles release];
    
    return [content copy];
}

- (NSString*)generateVaultManagementContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 🗄️ Gestion des Vaults\n\n"];
    
    [content appendString:@"### Vault actuel\n"];
    if (self.currentVaultPath && self.currentVaultName) {
        [content appendFormat:@"- **Nom :** %@\n", self.currentVaultName];
        [content appendFormat:@"- **Chemin :** `%@`\n", self.currentVaultPath];
        [content appendString:@"- **Statut :** ✅ Actif\n\n"];
    } else {
        [content appendString:@"- **Statut :** ❌ Aucun vault configuré\n\n"];
    }
    
    [content appendString:@"### Actions disponibles\n"];
    [content appendString:@"- **Créer un nouveau vault** : Organise tes notes dans un nouveau coffre\n"];
    [content appendString:@"- **Ouvrir un vault existant** : Charge un vault déjà créé\n"];
    [content appendString:@"- **Changer de vault** : Bascule entre différents coffres\n"];
    [content appendString:@"- **Paramètres du vault** : Configure les options avancées\n\n"];
    
    [content appendString:@"### Structure recommandée\n"];
    [content appendString:@"```\n"];
    [content appendString:@"MonVault/\n"];
    [content appendString:@"├── Notes/           # Tes documents markdown\n"];
    [content appendString:@"├── Attachments/     # Images et fichiers joints\n"];
    [content appendString:@"├── Templates/       # Modèles de documents\n"];
    [content appendString:@"└── Archive/         # Anciennes notes\n"];
    [content appendString:@"```\n\n"];
    
    return [content copy];
}

- (void)navigateToDirectory:(NSString*)path {
    if (!path || ![[NSFileManager defaultManager] fileExistsAtPath:path]) {
        NSLog(@"❌ [ENFilesTab] Chemin invalide: %@", path);
        return;
    }
    
    [_currentDirectory release];
    _currentDirectory = [path copy];
    
    NSLog(@"📁 [ENFilesTab] Navigation vers: %@", path);
    [self refreshContent];
}

- (void)navigateUp {
    NSString* currentPath = _currentDirectory ?: self.currentVaultPath;
    if (!currentPath) return;
    
    NSString* parentPath = [currentPath stringByDeletingLastPathComponent];
    if (parentPath && ![parentPath isEqualToString:currentPath]) {
        [self navigateToDirectory:parentPath];
    }
}

- (void)refreshCurrentDirectory {
    NSLog(@"🔄 [ENFilesTab] Actualisation du répertoire");
    [self refreshContent];
}

- (void)didBecomeActive {
    NSLog(@"📁 [ENFilesTab] Files activé");
    
    // Initialiser le répertoire actuel au vault si pas déjà défini
    if (!_currentDirectory && self.currentVaultPath) {
        [_currentDirectory release];
        _currentDirectory = [self.currentVaultPath copy];
    }
    
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"📁 [ENFilesTab] Files désactivé");
    [super didBecomeInactive];
}

- (void)createNewFolder:(NSString*)folderName {
    if (!folderName || folderName.length == 0) {
        NSLog(@"❌ [ENFilesTab] Nom de dossier invalide");
        return;
    }
    
    NSString* targetPath = _currentDirectory ?: self.currentVaultPath;
    if (!targetPath) {
        NSLog(@"❌ [ENFilesTab] Aucun répertoire cible");
        return;
    }
    
    NSString* newFolderPath = [targetPath stringByAppendingPathComponent:folderName];
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    if ([fileManager createDirectoryAtPath:newFolderPath withIntermediateDirectories:NO attributes:nil error:&error]) {
        NSLog(@"✅ [ENFilesTab] Dossier créé: %@", newFolderPath);
        [self refreshContent];
    } else {
        NSLog(@"❌ [ENFilesTab] Erreur création dossier: %@", error.localizedDescription);
    }
}

- (void)deleteFile:(NSString*)filePath {
    if (!filePath || ![[NSFileManager defaultManager] fileExistsAtPath:filePath]) {
        NSLog(@"❌ [ENFilesTab] Fichier inexistant: %@", filePath);
        return;
    }
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    if ([fileManager removeItemAtPath:filePath error:&error]) {
        NSLog(@"✅ [ENFilesTab] Fichier supprimé: %@", filePath);
        [self refreshContent];
    } else {
        NSLog(@"❌ [ENFilesTab] Erreur suppression: %@", error.localizedDescription);
    }
}

- (void)renameFile:(NSString*)oldPath toName:(NSString*)newName {
    if (!oldPath || !newName || newName.length == 0) {
        NSLog(@"❌ [ENFilesTab] Paramètres invalides pour renommage");
        return;
    }
    
    NSString* parentDir = [oldPath stringByDeletingLastPathComponent];
    NSString* newPath = [parentDir stringByAppendingPathComponent:newName];
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    if ([fileManager moveItemAtPath:oldPath toPath:newPath error:&error]) {
        NSLog(@"✅ [ENFilesTab] Fichier renommé: %@ -> %@", oldPath, newPath);
        [self refreshContent];
    } else {
        NSLog(@"❌ [ENFilesTab] Erreur renommage: %@", error.localizedDescription);
    }
}

- (void)dealloc {
    [_currentDirectory release];
    [_currentFiles release];
    [super dealloc];
}

@end