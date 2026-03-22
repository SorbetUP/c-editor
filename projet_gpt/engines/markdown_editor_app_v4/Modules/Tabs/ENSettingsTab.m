//
//  ENSettingsTab.m
//  ElephantNotes V4 - Module Settings
//

#import "ENSettingsTab.h"

@implementation ENSettingsTab

- (instancetype)init {
    return [super initWithName:@"Settings" icon:@"⚙️"];
}

- (NSString*)generateContent {
    return [self generateSettingsContent];
}

- (NSString*)generateSettingsContent {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# ⚙️ Paramètres - ElephantNotes V4\n\n"];
    
    // Section Configuration Actuelle
    [content appendString:@"## 🔧 Configuration Actuelle\n\n"];
    [content appendFormat:@"**Version:** ElephantNotes V4.0.0 (Architecture Modulaire)\n"];
    [content appendFormat:@"**Vault actuel:** %@\n", self.currentVaultName ?: @"Aucun"];
    [content appendFormat:@"**Chemin vault:** `%@`\n", self.currentVaultPath ?: @"Non configuré"];
    [content appendString:@"**Recherche:** ✅ Module ENSearchTab intégré\n"];
    [content appendString:@"**Auto-sauvegarde:** ✅ Disponible\n\n"];
    
    // Gestion des Vaults
    [content appendString:@"## 📁 Gestion des Vaults\n\n"];
    [content appendString:@"### Actions Disponibles\n"];
    [content appendString:@"- **Gestionnaire de vaults** : ⌘+V\n"];
    [content appendString:@"- **Nouveau vault** : ⌘+Shift+V\n"];
    [content appendString:@"- **Changer de vault** : Sélectionner dans la liste ci-dessous\n\n"];
    
    // Liste des vaults disponibles
    NSArray* availableVaults = [self getAvailableVaults];
    if (availableVaults.count > 0) {
        [content appendString:@"### Vaults Disponibles\n\n"];
        [content appendString:@"| Nom | Chemin | Notes | Statut |\n"];
        [content appendString:@"|-----|--------|--------|--------|\n"];
        
        for (NSDictionary* vault in availableVaults) {
            NSString* status = [vault[@"path"] isEqualToString:self.currentVaultPath] ? @"🟢 Actuel" : @"⚪ Disponible";
            [content appendFormat:@"| **%@** | `%@` | %@ | %@ |\n", 
                vault[@"name"], vault[@"path"], vault[@"noteCount"], status];
        }
        [content appendString:@"\n"];
    } else {
        [content appendString:@"### Aucun vault configuré\n"];
        [content appendString:@"Utilisez ⌘+V pour créer votre premier vault.\n\n"];
    }
    
    // Paramètres d'Apparence
    [content appendString:@"## 🎨 Apparence\n\n"];
    [content appendString:@"### Configuration d'Édition\n"];
    [content appendString:@"- **Police:** Monaco (Police monospace)\n"];
    [content appendString:@"- **Taille de police:** 14pt (Optimale pour la lecture)\n"];
    [content appendString:@"- **Thème:** Clair (Par défaut macOS)\n"];
    [content appendString:@"- **Largeur d'édition:** Responsive\n"];
    [content appendString:@"- **Numérotation lignes:** Activée\n\n"];
    
    [content appendString:@"### Rendu Markdown\n"];
    [content appendString:@"- **Prévisualisation:** En temps réel\n"];
    [content appendString:@"- **Coloration syntaxe:** Activée\n"];
    [content appendString:@"- **Liens automatiques:** Activés\n"];
    [content appendString:@"- **Tableaux:** Support complet\n\n"];
    
    // Architecture V4
    [content appendString:@"## 🏗️ Architecture V4\n\n"];
    [content appendString:@"### Modules Implémentés\n"];
    [content appendString:@"- **✅ ENTabBase** : Classe de base pour les onglets\n"];
    [content appendString:@"- **✅ ENSidebar** : Gestion de la barre latérale\n"];
    [content appendString:@"- **✅ ENMainController** : Contrôleur principal\n"];
    [content appendString:@"- **✅ ENAppDelegate** : Délégué d'application\n"];
    [content appendString:@"- **✅ ENDashboardTab** : Module Dashboard\n"];
    [content appendString:@"- **✅ ENSearchTab** : Module Search (sans interface custom)\n"];
    [content appendString:@"- **✅ ENSettingsTab** : Ce module Settings\n\n"];
    
    [content appendString:@"### Avantages de cette Architecture\n"];
    [content appendString:@"- **Séparation des responsabilités** : Chaque module a un rôle défini\n"];
    [content appendString:@"- **Extensibilité** : Ajouter nouveaux onglets facilement\n"];
    [content appendString:@"- **Maintenabilité** : Fichiers plus petits et focalisés\n"];
    [content appendString:@"- **Interface cohérente** : Tous les onglets utilisent ui_framework_set_editor_content()\n"];
    [content appendString:@"- **Barre latérale préservée** : Plus de problème d'interface masquée\n\n"];
    
    // Fonctionnalités Professionnelles
    [content appendString:@"## 💾 Fonctionnalités Professionnelles\n\n"];
    [content appendString:@"### Sauvegarde Automatique\n"];
    [content appendString:@"- **Auto-sauvegarde:** ✅ Disponible\n"];
    [content appendString:@"- **Sauvegarde sur inactivité:** ✅ Disponible\n"];
    [content appendString:@"- **Détection de modifications:** ✅ En temps réel\n\n"];
    
    [content appendString:@"### Gestion de Version\n"];
    [content appendString:@"- **Contrôle de version:** ✅ Disponible\n"];
    [content appendString:@"- **Snapshots:** Disponibles\n"];
    [content appendString:@"- **Détection de conflits:** ✅ Active\n"];
    [content appendString:@"- **Récupération de session:** ✅ Restauration automatique\n\n"];
    
    // Actions Système
    [content appendString:@"## 🔧 Actions Système\n\n"];
    [content appendString:@"### Maintenance\n"];
    [content appendString:@"- **Réindexer la recherche** : Relancer l'application\n"];
    [content appendString:@"- **Vider le cache** : Redémarrer l'application\n"];
    [content appendString:@"- **Exporter les données** : Copier le dossier vault\n"];
    [content appendString:@"- **Sauvegarde manuelle** : ⌘+S\n\n"];
    
    [content appendString:@"### Raccourcis Clavier\n\n"];
    [content appendString:@"| Raccourci | Action | Description |\n"];
    [content appendString:@"|-----------|--------|-------------|\n"];
    [content appendString:@"| ⌘+N | Nouvelle note | Créer un nouveau document |\n"];
    [content appendString:@"| ⌘+O | Ouvrir | Ouvrir un fichier existant |\n"];
    [content appendString:@"| ⌘+S | Sauvegarder | Sauvegarder le document actuel |\n"];
    [content appendString:@"| ⌘+Shift+S | Sauvegarder sous | Sauvegarder avec nouveau nom |\n"];
    [content appendString:@"| ⌘+V | Gestionnaire vaults | Ouvrir le gestionnaire |\n"];
    [content appendString:@"| ⌘+Shift+V | Nouveau vault | Créer un nouveau vault |\n"];
    [content appendString:@"| ⌘+Z | Annuler | Annuler la dernière action |\n"];
    [content appendString:@"| ⌘+Shift+Z | Rétablir | Rétablir l'action annulée |\n\n"];
    
    // Informations Système
    [content appendString:@"## 📊 Informations Système\n\n"];
    [content appendString:@"### Architecture ElephantNotes V4\n"];
    [content appendString:@"- **Moteur de rendu:** C Engine + UI Framework\n"];
    [content appendString:@"- **Interface:** Architecture modulaire Objective-C\n"];
    [content appendString:@"- **Système de vaults:** JSON + Professional File Manager\n"];
    [content appendString:@"- **Recherche:** Advanced Search + Module ENSearchTab\n"];
    [content appendString:@"- **Performance:** Optimisée pour macOS native\n\n"];
    
    [content appendString:@"### Support Technique\n"];
    [content appendString:@"- **Compatibilité:** macOS 10.15+\n"];
    [content appendString:@"- **Formats supportés:** Markdown (.md, .markdown)\n"];
    [content appendString:@"- **Encodage:** UTF-8\n"];
    [content appendString:@"- **Taille max fichier:** Illimitée (dans limites RAM)\n\n"];
    
    // Zone d'édition pour settings.md
    [content appendString:@"---\n\n"];
    [content appendString:@"## ✏️ Éditer les Paramètres\n\n"];
    [content appendString:@"Pour personnaliser ces paramètres, créez un fichier `settings.md` dans votre vault :\n\n"];
    [content appendString:@"```markdown\n"];
    [content appendString:@"# Mes Paramètres Personnalisés\n\n"];
    [content appendString:@"## Configuration\n"];
    [content appendString:@"- Mon paramètre 1\n"];
    [content appendString:@"- Mon paramètre 2\n\n"];
    [content appendString:@"## Notes\n"];
    [content appendString:@"Ces paramètres remplacent cette interface par défaut.\n"];
    [content appendString:@"```\n\n"];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V4 - Configuration et Paramètres*\n"];
    [content appendFormat:@"*Interface générée le %@*\n", [[NSDate date] description]];
    
    if (!self.currentVaultPath) {
        [content appendString:@"\n⚠️ **Configuration requise** - Créez un vault avec ⌘+V pour commencer."];
    }
    
    return [content copy];
}

- (NSArray*)getAvailableVaults {
    // Version simple qui retourne un exemple pour la démo
    // TODO: Intégrer avec le vrai système de vaults
    if (self.currentVaultPath) {
        return @[
            @{
                @"name": self.currentVaultName ?: @"Vault actuel",
                @"path": self.currentVaultPath,
                @"noteCount": @"5 notes"
            }
        ];
    }
    
    return @[];
}

- (void)didBecomeActive {
    NSLog(@"⚙️ [ENSettingsTab] Settings activé");
    [super didBecomeActive];
}

- (void)didBecomeInactive {
    NSLog(@"⚙️ [ENSettingsTab] Settings désactivé");
    [super didBecomeInactive];
}

@end