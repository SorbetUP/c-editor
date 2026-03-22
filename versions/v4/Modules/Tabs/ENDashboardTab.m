//
//  ENDashboardTab.m
//  ElephantNotes V4 - Module Dashboard
//

#import "ENDashboardTab.h"

@implementation ENDashboardTab

- (instancetype)init {
    return [super initWithName:@"Dashboard" icon:@"🏠"];
}

- (NSString*)generateContent {
    return [self generateDashboardContent];
}

- (NSString*)generateDashboardContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête avec informations vault
    [content appendString:@"# 🏠 Dashboard - ElephantNotes V4\n\n"];
    
    // Section vault actuel
    [content appendString:@"## 📁 Vault Actuel\n"];
    [content appendFormat:@"**Nom:** %@\n", self.currentVaultName ?: @"Aucun vault"];
    [content appendFormat:@"**Chemin:** `%@`\n", self.currentVaultPath ?: @"Non configuré"];
    
    // Statistiques du vault
    if (self.currentVaultPath) {
        NSDictionary* stats = [self getVaultStatistics];
        [content appendFormat:@"**Notes:** %@ fichiers\n", stats[@"noteCount"] ?: @"0"];
        [content appendFormat:@"**Taille:** %@ MB\n", stats[@"totalSize"] ?: @"0.0"];
        [content appendFormat:@"**Dernière modification:** %@\n\n", stats[@"lastModified"] ?: @"Inconnue"];
    } else {
        [content appendString:@"\n"];
    }
    
    // Actions rapides
    [content appendString:@"## 🚀 Actions Rapides\n\n"];
    [content appendString:@"### 📝 Fichiers\n"];
    [content appendString:@"- **Nouvelle note** : ⌘+N\n"];
    [content appendString:@"- **Ouvrir fichier** : ⌘+O\n"];
    [content appendString:@"- **Sauvegarder** : ⌘+S\n"];
    [content appendString:@"- **Sauvegarder sous** : ⌘+Shift+S\n\n"];
    
    [content appendString:@"### 📁 Vaults\n"];
    [content appendString:@"- **Gestionnaire de vaults** : ⌘+V\n"];
    [content appendString:@"- **Nouveau vault** : ⌘+Shift+V\n"];
    [content appendString:@"- **Changer de vault** : Clic sur ⚙️ Paramètres\n\n"];
    
    [content appendString:@"### 🔍 Recherche\n"];
    [content appendString:@"- **Recherche intelligente** : Clic sur 🔍 Recherche\n"];
    [content appendString:@"- **Indexation automatique** : Activée\n"];
    [content appendString:@"- **État de l'index** : ✅ Prêt\n\n"];
    
    // Fichiers récents
    if (self.currentVaultPath) {
        NSArray* recentFiles = [self getRecentFiles:5];
        if (recentFiles.count > 0) {
            [content appendString:@"## 📄 Fichiers Récents\n\n"];
            for (NSDictionary* file in recentFiles) {
                [content appendFormat:@"- **%@** - %@\n", file[@"name"], file[@"date"]];
            }
            [content appendString:@"\n"];
        }
    }
    
    // Navigation
    [content appendString:@"## 🎮 Navigation\n\n"];
    [content appendString:@"| Icône | Fonction | Description |\n"];
    [content appendString:@"|-------|----------|-------------|\n"];
    [content appendString:@"| 🔍 | **Recherche** | Rechercher dans toutes les notes |\n"];
    [content appendString:@"| ⬅️ | **Retour** | Historique et navigation |\n"];
    [content appendString:@"| 🏠 | **Dashboard** | Cette page d'accueil |\n"];
    [content appendString:@"| ⚙️ | **Paramètres** | Configuration et vaults |\n\n"];
    
    // Zone d'édition
    [content appendString:@"## ✏️ Zone d'Édition\n\n"];
    [content appendString:@"```markdown\n"];
    [content appendString:@"# Votre Nouvelle Note\n\n"];
    [content appendString:@"Commencez à écrire ici...\n\n"];
    [content appendString:@"- Point 1\n"];
    [content appendString:@"- Point 2\n"];
    [content appendString:@"- Point 3\n\n"];
    [content appendString:@"**Gras** et *italique* supportés.\n"];
    [content appendString:@"```\n\n"];
    
    // Architecture info
    [content appendString:@"---\n\n"];
    [content appendString:@"## 🏗️ Architecture V4\n\n"];
    [content appendString:@"Cette version utilise une **architecture modulaire** :\n\n"];
    [content appendString:@"- **ENTabBase** : Classe de base pour les onglets\n"];
    [content appendString:@"- **ENSidebar** : Gestion de la barre latérale\n"];
    [content appendString:@"- **ENMainController** : Contrôleur principal\n"];
    [content appendString:@"- **ENDashboardTab** : Ce module Dashboard\n\n"];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Architecture Modulaire*\n"];
    [content appendFormat:@"*Interface générée le %@*\n", [[NSDate date] description]];
    
    if (!self.currentVaultPath) {
        [content appendString:@"\n⚠️ **Aucun vault configuré** - Utilisez ⌘+V pour créer ou sélectionner un vault."];
    }
    
    return [content copy];
}

- (NSDictionary*)getVaultStatistics {
    if (!self.currentVaultPath) return @{};
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    // Compter les fichiers markdown dans Notes/
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    NSArray* contents = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
    
    NSUInteger noteCount = 0;
    unsigned long long totalSize = 0;
    NSDate* lastModified = [NSDate distantPast];
    
    if (!error && contents) {
        for (NSString* item in contents) {
            if ([[item pathExtension] isEqualToString:@"md"]) {
                noteCount++;
                
                NSString* fullPath = [notesPath stringByAppendingPathComponent:item];
                NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
                
                if (attributes) {
                    totalSize += [attributes[NSFileSize] unsignedLongLongValue];
                    NSDate* modified = attributes[NSFileModificationDate];
                    if ([modified compare:lastModified] == NSOrderedDescending) {
                        lastModified = modified;
                    }
                }
            }
        }
    }
    
    NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
    [formatter setDateStyle:NSDateFormatterMediumStyle];
    [formatter setTimeStyle:NSDateFormatterShortStyle];
    
    return @{
        @"noteCount": @(noteCount).stringValue,
        @"totalSize": [NSString stringWithFormat:@"%.1f", totalSize / (1024.0 * 1024.0)],
        @"lastModified": [formatter stringFromDate:lastModified]
    };
}

- (NSArray*)getRecentFiles:(int)count {
    if (!self.currentVaultPath) return @[];
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    NSArray* contents = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
    
    if (error || !contents) return @[];
    
    NSMutableArray* files = [[[NSMutableArray alloc] init] autorelease];
    
    for (NSString* item in contents) {
        if ([[item pathExtension] isEqualToString:@"md"]) {
            NSString* fullPath = [notesPath stringByAppendingPathComponent:item];
            NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
            
            if (attributes) {
                NSDate* modified = attributes[NSFileModificationDate];
                NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
                [formatter setDateStyle:NSDateFormatterShortStyle];
                [formatter setTimeStyle:NSDateFormatterShortStyle];
                
                [files addObject:@{
                    @"name": [item stringByDeletingPathExtension],
                    @"date": [formatter stringFromDate:modified],
                    @"modificationDate": modified
                }];
            }
        }
    }
    
    // Trier par date de modification (plus récent en premier)
    [files sortUsingComparator:^NSComparisonResult(NSDictionary* file1, NSDictionary* file2) {
        NSDate* date1 = file1[@"modificationDate"];
        NSDate* date2 = file2[@"modificationDate"];
        return [date2 compare:date1];
    }];
    
    // Limiter le nombre de résultats
    NSUInteger maxCount = MIN(files.count, count);
    return [files subarrayWithRange:NSMakeRange(0, maxCount)];
}

- (void)didBecomeActive {
    NSLog(@"🏠 [ENDashboardTab] Dashboard activé");
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"🏠 [ENDashboardTab] Dashboard désactivé");
    [super didBecomeInactive];
}

@end