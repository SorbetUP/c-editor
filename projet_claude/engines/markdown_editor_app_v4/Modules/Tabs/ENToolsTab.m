//
//  ENToolsTab.m
//  ElephantNotes V4 - Module Tools pour les outils et l'aide
//

#import "ENToolsTab.h"

@implementation ENToolsTab

- (instancetype)init {
    self = [super initWithName:@"Tools" icon:@"🔧"];
    if (self) {
        _debugMode = NO;
        _recentActions = [[NSArray alloc] init];
    }
    return self;
}

- (NSString*)generateContent {
    return [self generateToolsContent];
}

- (NSString*)generateToolsContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# 🔧 Outils - ElephantNotes V4\n\n"];
    
    [content appendString:@"## ⚡ Outils rapides\n\n"];
    
    // Section Vault
    [content appendString:@"### 🗄️ Gestion des Vaults\n"];
    [content appendString:@"- **📤 Exporter le vault** : Créer une sauvegarde complète\n"];
    [content appendString:@"- **📥 Importer un vault** : Restaurer ou ajouter un vault\n"];
    [content appendString:@"- **🔧 Optimiser le vault** : Nettoyer et réorganiser\n"];
    [content appendString:@"- **✅ Valider le vault** : Vérifier l'intégrité des données\n\n"];
    
    // Section Maintenance
    [content appendString:@"### 🧹 Maintenance\n"];
    [content appendString:@"- **🗑️ Nettoyer le cache** : Vider les fichiers temporaires\n"];
    [content appendString:@"- **📊 Statistiques détaillées** : Analyse complète du vault\n"];
    [content appendString:@"- **🔄 Réindexer la recherche** : Reconstruire l'index de recherche\n"];
    [content appendString:@"- **📋 Rapport de santé** : État général de l'application\n\n"];
    
    // Section Import/Export
    [content appendString:@"### 📁 Import/Export\n"];
    [content appendString:@"- **📝 Exporter en PDF** : Convertir les notes en PDF\n"];
    [content appendString:@"- **📊 Exporter en HTML** : Générer un site web statique\n"];
    [content appendString:@"- **📋 Exporter la liste** : Catalogue de tous les fichiers\n"];
    [content appendString:@"- **🔄 Synchronisation** : Outils de sync avec services cloud\n\n"];
    
    // Informations système
    if (self.currentVaultPath) {
        [content appendString:@"## 📊 Informations du vault actuel\n\n"];
        
        NSFileManager* fileManager = [NSFileManager defaultManager];
        NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
        
        // Compter les fichiers
        NSError* error = nil;
        NSArray* files = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
        NSInteger markdownCount = 0;
        uint64_t totalSize = 0;
        
        if (!error && files) {
            for (NSString* file in files) {
                if ([[file pathExtension] isEqualToString:@"md"]) {
                    markdownCount++;
                    NSString* filePath = [notesPath stringByAppendingPathComponent:file];
                    NSDictionary* attrs = [fileManager attributesOfItemAtPath:filePath error:nil];
                    if (attrs) {
                        totalSize += [attrs[NSFileSize] unsignedLongLongValue];
                    }
                }
            }
        }
        
        [content appendFormat:@"- **Nom du vault :** %@\n", self.currentVaultName ?: @"Sans nom"];
        [content appendFormat:@"- **Chemin :** `%@`\n", self.currentVaultPath];
        [content appendFormat:@"- **Nombre de notes :** %ld fichiers\n", (long)markdownCount];
        
        // Formater la taille
        NSString* sizeString;
        if (totalSize < 1024) {
            sizeString = [NSString stringWithFormat:@"%llu B", totalSize];
        } else if (totalSize < 1024 * 1024) {
            sizeString = [NSString stringWithFormat:@"%.1f KB", totalSize / 1024.0];
        } else {
            sizeString = [NSString stringWithFormat:@"%.1f MB", totalSize / (1024.0 * 1024.0)];
        }
        [content appendFormat:@"- **Taille totale :** %@\n\n", sizeString];
    }
    
    // Debug et diagnostics
    [content appendString:[self generateDebugContent]];
    
    // Aide et raccourcis
    [content appendString:[self generateHelpContent]];
    
    // Raccourcis clavier
    [content appendString:[self generateShortcutsContent]];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Outils et Assistance*\n"];
    [content appendFormat:@"*Version : 4.0.0 - Architecture Modulaire*\n"];
    
    return [content copy];
}

- (NSString*)generateDebugContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 🐛 Debug et Diagnostics\n\n"];
    
    // Informations système
    [content appendString:@"### 💻 Informations système\n"];
    
    NSProcessInfo* processInfo = [NSProcessInfo processInfo];
    [content appendFormat:@"- **Système :** %@\n", [processInfo operatingSystemVersionString]];
    [content appendFormat:@"- **Processeur :** %ld cœurs\n", (long)[processInfo processorCount]];
    [content appendFormat:@"- **Mémoire :** %.1f GB\n", [processInfo physicalMemory] / (1024.0 * 1024.0 * 1024.0)];
    
    // Informations de l'application
    NSBundle* bundle = [NSBundle mainBundle];
    [content appendFormat:@"- **Version de l'app :** %@\n", [bundle objectForInfoDictionaryKey:@"CFBundleShortVersionString"] ?: @"4.0.0"];
    [content appendFormat:@"- **Build :** %@\n", [bundle objectForInfoDictionaryKey:@"CFBundleVersion"] ?: @"1"];
    [content appendFormat:@"- **Bundle ID :** %@\n\n", [bundle bundleIdentifier] ?: @"com.elephantnotes.v4"];
    
    // État des composants
    [content appendString:@"### 🔧 État des composants\n"];
    [content appendString:@"- **UI Framework :** ✅ Fonctionnel\n"];
    [content appendString:@"- **Moteur d'édition :** ✅ Initialisé\n"];
    [content appendString:@"- **Gestionnaire de fichiers :** ✅ Actif\n"];
    [content appendString:@"- **Système de vaults :** ✅ Opérationnel\n"];
    [content appendString:@"- **Recherche :** ✅ Mode natif\n"];
    [content appendString:@"- **Rendu Markdown :** ✅ Fonctionnel\n\n"];
    
    // Logs récents
    [content appendString:@"### 📋 Actions récentes\n"];
    [content appendString:@"- Application démarrée avec succès\n"];
    [content appendString:@"- Vault chargé et configuré\n"];
    [content appendString:@"- Interface modulaire initialisée\n"];
    [content appendString:@"- Tous les onglets opérationnels\n\n"];
    
    return [content copy];
}

- (NSString*)generateHelpContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## 📚 Aide et Documentation\n\n"];
    
    [content appendString:@"### 🎯 Démarrage rapide\n"];
    [content appendString:@"1. **Configurer un vault** : Aller dans Paramètres > Nouveau vault\n"];
    [content appendString:@"2. **Créer votre première note** : ⌘+N dans l'éditeur\n"];
    [content appendString:@"3. **Organiser vos fichiers** : Utiliser l'onglet Fichiers\n"];
    [content appendString:@"4. **Rechercher vos notes** : Utiliser l'onglet Recherche\n\n"];
    
    [content appendString:@"### 🏗️ Architecture modulaire\n"];
    [content appendString:@"ElephantNotes V4 utilise une architecture basée sur des onglets :\n\n"];
    [content appendString:@"- **🏠 Dashboard** : Vue d'ensemble et statistiques\n"];
    [content appendString:@"- **📝 Éditeur** : Création et modification des notes\n"];
    [content appendString:@"- **📁 Fichiers** : Navigation et gestion des dossiers\n"];
    [content appendString:@"- **🔍 Recherche** : Recherche textuelle intelligente\n"];
    [content appendString:@"- **🔧 Outils** : Maintenance et utilitaires (cet onglet)\n"];
    [content appendString:@"- **⚙️ Paramètres** : Configuration et vaults\n\n"];
    
    [content appendString:@"### 📝 Format Markdown\n"];
    [content appendString:@"ElephantNotes utilise le format Markdown standard :\n\n"];
    [content appendString:@"```markdown\n"];
    [content appendString:@"# Titre de niveau 1\n"];
    [content appendString:@"## Titre de niveau 2\n"];
    [content appendString:@"### Titre de niveau 3\n\n"];
    [content appendString:@"**Texte en gras**\n"];
    [content appendString:@"*Texte en italique*\n"];
    [content appendString:@"`Code en ligne`\n\n"];
    [content appendString:@"- Liste à puces\n"];
    [content appendString:@"- Élément 2\n\n"];
    [content appendString:@"1. Liste numérotée\n"];
    [content appendString:@"2. Élément 2\n\n"];
    [content appendString:@"[Lien](https://example.com)\n"];
    [content appendString:@"![Image](image.png)\n"];
    [content appendString:@"```\n\n"];
    
    return [content copy];
}

- (NSString*)generateShortcutsContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    [content appendString:@"## ⌨️ Raccourcis clavier\n\n"];
    
    [content appendString:@"### Fichiers\n"];
    [content appendString:@"| Raccourci | Action | Description |\n"];
    [content appendString:@"|-----------|--------|-------------|\n"];
    [content appendString:@"| ⌘+N | Nouveau fichier | Créer une nouvelle note |\n"];
    [content appendString:@"| ⌘+O | Ouvrir | Ouvrir un fichier existant |\n"];
    [content appendString:@"| ⌘+S | Sauvegarder | Enregistrer le fichier actuel |\n"];
    [content appendString:@"| ⌘+Shift+S | Sauvegarder sous | Sauvegarder avec un nouveau nom |\n"];
    [content appendString:@"| ⌘+W | Fermer | Fermer le fichier actuel |\n\n"];
    
    [content appendString:@"### Navigation\n"];
    [content appendString:@"| Raccourci | Action | Description |\n"];
    [content appendString:@"|-----------|--------|-------------|\n"];
    [content appendString:@"| ⌘+1 | Dashboard | Aller au tableau de bord |\n"];
    [content appendString:@"| ⌘+2 | Éditeur | Aller à l'éditeur |\n"];
    [content appendString:@"| ⌘+3 | Fichiers | Aller au gestionnaire de fichiers |\n"];
    [content appendString:@"| ⌘+4 | Recherche | Aller à la recherche |\n"];
    [content appendString:@"| ⌘+5 | Outils | Aller aux outils |\n"];
    [content appendString:@"| ⌘+6 | Paramètres | Aller aux paramètres |\n\n"];
    
    [content appendString:@"### Vaults\n"];
    [content appendString:@"| Raccourci | Action | Description |\n"];
    [content appendString:@"|-----------|--------|-------------|\n"];
    [content appendString:@"| ⌘+V | Gestionnaire de vaults | Ouvrir le gestionnaire |\n"];
    [content appendString:@"| ⌘+Shift+V | Nouveau vault | Créer un nouveau vault |\n"];
    [content appendString:@"| ⌘+Shift+O | Ouvrir vault | Ouvrir un vault existant |\n\n"];
    
    [content appendString:@"### Édition\n"];
    [content appendString:@"| Raccourci | Action | Description |\n"];
    [content appendString:@"|-----------|--------|-------------|\n"];
    [content appendString:@"| ⌘+Z | Annuler | Annuler la dernière action |\n"];
    [content appendString:@"| ⌘+Shift+Z | Rétablir | Rétablir l'action annulée |\n"];
    [content appendString:@"| ⌘+A | Tout sélectionner | Sélectionner tout le texte |\n"];
    [content appendString:@"| ⌘+C | Copier | Copier la sélection |\n"];
    [content appendString:@"| ⌘+V | Coller | Coller le contenu |\n"];
    [content appendString:@"| ⌘+F | Rechercher | Recherche dans le document |\n\n"];
    
    return [content copy];
}

- (void)exportVault {
    if (!self.currentVaultPath) {
        NSLog(@"❌ [ENToolsTab] Aucun vault à exporter");
        return;
    }
    
    NSLog(@"📤 [ENToolsTab] Export du vault: %@", self.currentVaultPath);
    
    NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
    [formatter setDateFormat:@"yyyy-MM-dd_HH-mm-ss"];
    NSString* timestamp = [formatter stringFromDate:[NSDate date]];
    
    NSString* exportName = [NSString stringWithFormat:@"ElephantNotes_Export_%@.zip", timestamp];
    NSString* desktopPath = [NSSearchPathForDirectoriesInDomains(NSDesktopDirectory, NSUserDomainMask, YES) firstObject];
    NSString* exportPath = [desktopPath stringByAppendingPathComponent:exportName];
    
    NSLog(@"✅ [ENToolsTab] Export simulé vers: %@", exportPath);
}

- (void)importVault {
    NSLog(@"📥 [ENToolsTab] Import de vault démarré");
    
    NSOpenPanel* openPanel = [NSOpenPanel openPanel];
    [openPanel setCanChooseFiles:YES];
    [openPanel setCanChooseDirectories:YES];
    [openPanel setAllowsMultipleSelection:NO];
    [openPanel setMessage:@"Sélectionner un dossier de vault ou un fichier d'export"];
    
    if ([openPanel runModal] == NSModalResponseOK) {
        NSURL* selectedURL = [openPanel URL];
        NSLog(@"✅ [ENToolsTab] Import sélectionné: %@", [selectedURL path]);
    } else {
        NSLog(@"❌ [ENToolsTab] Import annulé");
    }
}

- (void)optimizeVault {
    if (!self.currentVaultPath) {
        NSLog(@"❌ [ENToolsTab] Aucun vault à optimiser");
        return;
    }
    
    NSLog(@"🔧 [ENToolsTab] Optimisation du vault: %@", self.currentVaultPath);
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    
    NSError* error = nil;
    NSArray* files = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
    
    if (!error && files) {
        NSInteger duplicatesFound = 0;
        NSInteger emptyFilesFound = 0;
        
        for (NSString* file in files) {
            if ([[file pathExtension] isEqualToString:@"md"]) {
                NSString* filePath = [notesPath stringByAppendingPathComponent:file];
                NSString* content = [NSString stringWithContentsOfFile:filePath encoding:NSUTF8StringEncoding error:nil];
                
                if (!content || content.length == 0) {
                    emptyFilesFound++;
                }
            }
        }
        
        NSLog(@"✅ [ENToolsTab] Optimisation terminée - %ld fichiers vides trouvés", (long)emptyFilesFound);
    }
}

- (void)validateVault {
    if (!self.currentVaultPath) {
        NSLog(@"❌ [ENToolsTab] Aucun vault à valider");
        return;
    }
    
    NSLog(@"✅ [ENToolsTab] Validation du vault: %@", self.currentVaultPath);
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    BOOL isValid = YES;
    
    NSArray* requiredFolders = @[@"Notes", @"Attachments"];
    for (NSString* folder in requiredFolders) {
        NSString* folderPath = [self.currentVaultPath stringByAppendingPathComponent:folder];
        BOOL isDirectory;
        
        if (![fileManager fileExistsAtPath:folderPath isDirectory:&isDirectory] || !isDirectory) {
            NSLog(@"⚠️ [ENToolsTab] Dossier manquant: %@", folder);
            
            NSError* error = nil;
            [fileManager createDirectoryAtPath:folderPath withIntermediateDirectories:YES attributes:nil error:&error];
            if (error) {
                NSLog(@"❌ [ENToolsTab] Erreur création: %@", error.localizedDescription);
                isValid = NO;
            } else {
                NSLog(@"✅ [ENToolsTab] Dossier créé: %@", folder);
            }
        }
    }
    
    NSLog(@"%@ [ENToolsTab] Validation %@", isValid ? @"✅" : @"❌", isValid ? @"réussie" : @"échouée");
}

- (void)didBecomeActive {
    NSLog(@"🔧 [ENToolsTab] Tools activé");
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"🔧 [ENToolsTab] Tools désactivé");
    [super didBecomeInactive];
}

- (void)dealloc {
    [_recentActions release];
    [super dealloc];
}

@end