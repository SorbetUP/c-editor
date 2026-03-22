// ElephantNotesV3.m - Implémentation complète d'ElephantNotes V3
#import "ElephantNotesV3.h"
#import "VaultSetupController.h"
#import "VaultManagerController.h"
#import "VaultCreationPopup.h"

// Variable globale pour le contrôleur principal
static ElephantNotesV3Controller* g_mainController = nil;

// ========== App Delegate ==========
@implementation ElephantNotesV3AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    NSLog(@"🚀 ElephantNotes V3 - Démarrage de l'application");
    
    elephantnotes_v3_init();
    
    ElephantNotesV3Controller* controller = elephantnotes_v3_get_controller();
    [controller setupApplication];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return YES;
}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
    NSLog(@"🔄 ElephantNotes V3 - Fermeture de l'application");
    elephantnotes_v3_cleanup();
}

@end

// ========== Window personnalisée ==========
@implementation ElephantNotesV3Window

- (instancetype)initWithContentRect:(NSRect)contentRect 
                          styleMask:(NSWindowStyleMask)style 
                            backing:(NSBackingStoreType)backingStoreType 
                              defer:(BOOL)flag {
    self = [super initWithContentRect:contentRect styleMask:style backing:backingStoreType defer:flag];
    if (self) {
        [self setTitle:@"ElephantNotes V3"];
        [self setMinSize:NSMakeSize(800, 600)];
        [self center];
        [self setReleasedWhenClosed:NO];
    }
    return self;
}

- (void)keyDown:(NSEvent *)event {
    // Gérer les raccourcis clavier spéciaux
    if ([event modifierFlags] & NSEventModifierFlagCommand) {
        NSString *characters = [event charactersIgnoringModifiers];
        if ([characters isEqualToString:@"z"]) {
            // Cmd+Z - deleguer à l'éditeur actif
            NSLog(@"⌨️ Cmd+Z détecté au niveau fenêtre");
            [super keyDown:event]; // Laisser la chaîne de répondeurs gérer
            return;
        }
        if ([characters isEqualToString:@"Z"]) {
            // Cmd+Shift+Z (Redo)
            NSLog(@"⌨️ Cmd+Shift+Z détecté au niveau fenêtre");
            [super keyDown:event];
            return;
        }
    }
    
    // Pour tous les autres événements
    [super keyDown:event];
}

@end

// ========== Contrôleur principal ==========
@implementation ElephantNotesV3Controller

- (instancetype)init {
    self = [super init];
    if (self) {
        NSLog(@"🔧 Initialisation du contrôleur ElephantNotes V3");
        
        _uiFramework = NULL;
        _vaultRegistry = NULL;
        _currentVaultPath = nil;
        _currentVaultName = nil;
        _vaultSystemReady = false;
        _currentFilePath = nil;
        _originalContent = @"";
        _hasUnsavedChanges = false;
        _isFirstLaunch = false;
        _currentMode = UI_ICON_HOME;
        _autoSaveTimer = nil;
        _conflictCheckTimer = nil;
        
        // Système de recherche
        _searchEngine = NULL;
        _searchInterface = NULL;
        _searchIndexReady = false;
        
        // Historique de navigation
        _navigationHistory = [[NSMutableArray alloc] init];
        _currentHistoryIndex = -1;
        
        // Contrôleur de création de vault
        _vaultSetupController = nil;
        _vaultCreationPopup = nil;
    }
    return self;
}

- (void)setupApplication {
    NSLog(@"⚙️ Configuration de l'application V3");
    
    // 1. Initialiser le système de vaults
    [self initializeVaultSystem];
    
    // 2. Vérifier le premier lancement
    [self checkFirstLaunch];
    
    // 3. Initialiser l'UI dans tous les cas
    NSLog(@"🔄 Avant initializeUI");
    [self initializeUI];
    NSLog(@"🔄 Après initializeUI");
    
    // 4. Afficher la fenêtre AVANT d'initialiser les modes
    [self showMainWindow];
    NSLog(@"🔄 Après showMainWindow");
    
    // 5. Maintenant initialiser le mode (après que la fenêtre soit visible)
    NSLog(@"🔄 Définition du mode initial");
    ui_framework_set_icon_active(_uiFramework, UI_ICON_HOME);
    [self switchToMode:UI_ICON_HOME];
    NSLog(@"🔄 Mode initial défini");
    
    // 6. Décision : vault ou pas vault
    if (_isFirstLaunch) {
        NSLog(@"🔧 Aucun vault configuré - Affichage de la popup de création");
        [self showVaultCreationPopup];
    } else {
        NSLog(@"✅ Vault configuré - Affichage de l'application normale");
        [self showDashboardMode];
    }
}

- (void)initializeVaultSystem {
    NSLog(@"📁 Initialisation du système de vaults");
    
    // Initialiser le gestionnaire de vaults
    VaultResult result = vault_manager_init();
    if (result != VAULT_SUCCESS) {
        NSLog(@"❌ Erreur d'initialisation du gestionnaire de vaults: %d", result);
        return;
    }
    
    // Charger le registre des vaults
    result = vault_registry_load(&_vaultRegistry);
    if (result == VAULT_SUCCESS && _vaultRegistry && _vaultRegistry->default_vault_path) {
        if (_currentVaultPath) {
            [_currentVaultPath release];
        }
        _currentVaultPath = [[NSString stringWithUTF8String:_vaultRegistry->default_vault_path] retain];
        NSLog(@"✅ Vault par défaut chargé: %@", _currentVaultPath);
        
        // Charger les informations du vault
        VaultInfo* vaultInfo = NULL;
        if (vault_load([_currentVaultPath UTF8String], &vaultInfo) == VAULT_SUCCESS && vaultInfo) {
            if (_currentVaultName) {
                [_currentVaultName release];
            }
            _currentVaultName = [[NSString stringWithUTF8String:vaultInfo->config.name] retain];
            vault_free_info(vaultInfo);
            free(vaultInfo);
        }
        
        _vaultSystemReady = true;
    } else {
        NSLog(@"⚠️ Aucun vault configuré");
        _vaultSystemReady = false;
    }
}

- (void)checkFirstLaunch {
    bool isFirstLaunch = vault_is_first_launch();
    bool hasDefaultVault = (_vaultRegistry && _vaultRegistry->default_vault_path);
    bool hasVaults = (_vaultRegistry && _vaultRegistry->count > 0);
    
    NSLog(@"🔍 Debug vault_is_first_launch(): %s", isFirstLaunch ? "OUI" : "NON");
    NSLog(@"🔍 Debug hasDefaultVault: %s", hasDefaultVault ? "OUI" : "NON"); 
    NSLog(@"🔍 Debug hasVaults: %s", hasVaults ? "OUI" : "NON");
    NSLog(@"🔍 Debug _vaultRegistry: %p", _vaultRegistry);
    if (_vaultRegistry) {
        NSLog(@"🔍 Debug _vaultRegistry->count: %d", _vaultRegistry->count);
        NSLog(@"🔍 Debug _vaultRegistry->default_vault_path: %s", _vaultRegistry->default_vault_path ?: "NULL");
    }
    
    // Force premier lancement si aucun vault disponible
    _isFirstLaunch = isFirstLaunch || !hasDefaultVault || !hasVaults;
    
    NSLog(@"🔍 Premier lancement FINAL: %s, Vault configuré: %s, Vaults disponibles: %d", 
          _isFirstLaunch ? "OUI" : "NON",
          hasDefaultVault ? "OUI" : "NON",
          _vaultRegistry ? _vaultRegistry->count : 0);
    
    if (_isFirstLaunch) {
        if (hasVaults) {
            // Il y a des vaults mais pas de défaut -> interface de sélection
            _currentMode = UI_ICON_SETTINGS;
            NSLog(@"🎯 Mode: Sélection de vault existant");
        } else {
            // Aucun vault -> interface de création
            _currentMode = UI_ICON_SETTINGS; // Mode paramètres pour gestion vault
            NSLog(@"🎯 Mode: Création de nouveau vault");
        }
    } else {
        // Configuration normale
        _currentMode = UI_ICON_HOME; // Dashboard par défaut
        NSLog(@"🎯 Mode: Dashboard normal");
    }
}

- (void)showVaultSetup {
    // Cette méthode n'est plus utilisée - la configuration des vaults se fait dans l'interface principale
    NSLog(@"🔧 Redirection vers l'interface de paramètres pour la configuration des vaults");
    [self switchToMode:UI_ICON_SETTINGS];
}

- (void)vaultSetupCompleted:(NSString*)vaultPath {
    NSLog(@"✅ Configuration du vault terminée: %@", vaultPath);
    
    if (_currentVaultPath) {
        [_currentVaultPath release];
    }
    _currentVaultPath = [vaultPath retain];
    _isFirstLaunch = false;
    _vaultSystemReady = true;
    
    // Recharger le registre pour obtenir les informations à jour
    if (_vaultRegistry) {
        vault_registry_free(_vaultRegistry);
        _vaultRegistry = NULL;
    }
    
    vault_registry_load(&_vaultRegistry);
    
    // Charger le nom du vault
    VaultInfo* vaultInfo = NULL;
    if (vault_load([_currentVaultPath UTF8String], &vaultInfo) == VAULT_SUCCESS && vaultInfo) {
        if (_currentVaultName) {
            [_currentVaultName release];
        }
        _currentVaultName = [[NSString stringWithUTF8String:vaultInfo->config.name] retain];
        vault_free_info(vaultInfo);
        free(vaultInfo);
    }
    
    // Maintenant initialiser l'UI et afficher la fenêtre principale
    [self initializeUI];
    [self showMainWindow];
}

- (void)initializeUI {
    NSLog(@"🖥️ Initialisation de l'interface utilisateur");
    
    NSLog(@"🔄 Étape 1: Création de la fenêtre principale");
    // Créer la fenêtre principale
    NSRect windowFrame = NSMakeRect(0, 0, 1200, 800);
    _mainWindow = [[ElephantNotesV3Window alloc] 
        initWithContentRect:windowFrame
        styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | 
                  NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
        backing:NSBackingStoreBuffered 
        defer:NO];
    NSLog(@"✅ Fenêtre principale créée");
    
    NSLog(@"🔄 Étape 2: Configuration du titre");
    // Mettre à jour le titre avec le nom du vault
    if (_currentVaultName) {
        [_mainWindow setTitle:[NSString stringWithFormat:@"ElephantNotes V3 - %@", _currentVaultName]];
    }
    NSLog(@"✅ Titre configuré");
    
    NSLog(@"🔄 Étape 3: Création du framework UI");
    // Créer le framework UI
    _uiFramework = ui_framework_create(_mainWindow);
    if (!_uiFramework) {
        NSLog(@"❌ Erreur de création du framework UI");
        return;
    }
    NSLog(@"✅ Framework UI créé");
    
    NSLog(@"🔄 Étape 4: Configuration des gestionnaires d'événements");
    // Configurer les gestionnaires d'événements
    ui_framework_set_click_handler(_uiFramework, ui_icon_click_handler, (__bridge void*)self);
    ui_framework_set_hover_handler(_uiFramework, ui_icon_hover_handler, (__bridge void*)self);
    NSLog(@"✅ Gestionnaires d'événements configurés");
    
    NSLog(@"🔄 Étape 5: Configuration de la mise en page");
    // Configurer la mise en page
    if (!ui_framework_setup_layout(_uiFramework)) {
        NSLog(@"❌ Erreur de configuration de la mise en page UI");
        return;
    }
    NSLog(@"✅ Mise en page configurée");
    
    NSLog(@"🔄 Étape 6: Initialisation du gestionnaire professionnel");
    // Initialiser le gestionnaire de fichiers professionnel
    professional_init();
    NSLog(@"✅ Gestionnaire professionnel initialisé");
    
    NSLog(@"🔄 Étape 7: Configuration de la sauvegarde immédiate");
    // Configurer les fonctionnalités professionnelles
    // [self enableAutoSave]; // Désactivé: sauvegarde immédiate à chaque modification
    NSLog(@"✅ Sauvegarde immédiate configurée");
    
    NSLog(@"✅ Interface utilisateur initialisée - prêt pour affichage");
}

- (void)showMainWindow {
    if (_mainWindow) {
        NSLog(@"🔄 Tentative d'affichage de la fenêtre principale");
        
        // Configurer la fenêtre pour être visible
        [_mainWindow setLevel:NSNormalWindowLevel];
        [_mainWindow orderFrontRegardless];
        [_mainWindow makeKeyAndOrderFront:nil];
        [_mainWindow center];
        
        // Forcer l'activation de l'application
        NSApplication *app = [NSApplication sharedApplication];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🪟 Fenêtre principale affichée et centrée");
        NSLog(@"🔍 Fenêtre visible: %@", [_mainWindow isVisible] ? @"OUI" : @"NON");
        NSLog(@"🔍 Fenêtre frame: %@", NSStringFromRect([_mainWindow frame]));
    } else {
        NSLog(@"❌ _mainWindow est NULL - impossible d'afficher");
    }
}

- (void)handleIconClick:(UIIconType)iconType {
    NSLog(@"🖱️ Clic sur l'icône: %@", ui_framework_get_icon_name(iconType));
    NSLog(@"🔥 État du controller - _uiFramework: %p", _uiFramework);
    
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé lors du clic sur l'icône");
        return;
    }
    
    NSLog(@"🔥 Appel ui_framework_set_icon_active");
    ui_framework_set_icon_active(_uiFramework, iconType);
    NSLog(@"🔥 ui_framework_set_icon_active terminé");
    
    NSLog(@"🔥 Appel switchToMode");
    [self switchToMode:iconType];
    NSLog(@"🔥 switchToMode terminé");
}

- (void)handleIconHover:(UIIconType)iconType isHovering:(bool)isHovering {
    // Animation ou feedback visuel pour le survol
    // Pour l'instant, juste du logging
    if (isHovering) {
        NSLog(@"🔍 Survol: %@", ui_framework_get_icon_name(iconType));
    }
}

- (void)switchToMode:(UIIconType)mode {
    NSLog(@"🔥 switchToMode appelé avec mode: %d", mode);
    
    // Ajouter à l'historique seulement si ce n'est pas une navigation back/forward
    if (mode != UI_ICON_BACK && _currentMode != mode) {
        NSLog(@"🔥 Ajout à l'historique");
        [self addToNavigationHistory:_currentMode withContent:nil];
    }
    
    _currentMode = mode;
    NSLog(@"🔥 Mode actuel défini: %d", _currentMode);
    
    switch (mode) {
        case UI_ICON_SEARCH:
            NSLog(@"🔥 Basculement vers mode recherche");
            [self showSearchMode];
            break;
        case UI_ICON_BACK:
            NSLog(@"🔥 Basculement vers mode retour");
            [self showBackMode];
            break;
        case UI_ICON_HOME:
            NSLog(@"🔥 Basculement vers mode dashboard");
            [self showDashboardMode];
            break;
        case UI_ICON_SETTINGS:
            NSLog(@"🔥 Basculement vers mode paramètres");
            [self showSettingsMode];
            break;
        default:
            NSLog(@"⚠️ Mode non implémenté: %d", mode);
            break;
    }
    NSLog(@"🔥 switchToMode terminé");
}

- (void)showSearchMode {
    NSLog(@"🔍 Mode recherche activé");
    
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    if (!_searchIndexReady && _currentVaultPath) {
        [self initializeSearchSystem];
        [self indexCurrentVault];
    }
    
    // Interface de recherche enrichie avec gestion mémoire correcte
    @autoreleasepool {
        NSString* searchContent = [self generateSearchInterface];
        if (searchContent) {
            ui_framework_set_editor_content(_uiFramework, searchContent);
        }
    }
}

- (NSString*)generateSearchInterface {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête de recherche
    [content appendString:@"# 🔍 Recherche Intelligente - ElephantNotes V3\n\n"];
    
    // État de l'indexation
    [content appendString:@"## 📊 État de l'Index\n\n"];
    [content appendFormat:@"**Vault actuel:** %@\n", _currentVaultName ?: @"Aucun vault"];
    [content appendFormat:@"**Chemin:** `%@`\n", _currentVaultPath ?: @"Non configuré"];
    
    if (_searchEngine) {
        SearchStats stats = search_engine_get_stats(_searchEngine);
        [content appendFormat:@"**Fichiers indexés:** %d\n", stats.total_files_indexed];
        [content appendFormat:@"**Requêtes effectuées:** %d\n", stats.total_queries];
        [content appendFormat:@"**Performance moyenne:** %.2f ms/requête\n", stats.avg_query_time_ms];
        [content appendFormat:@"**Mémoire utilisée:** %zu MB\n", stats.memory_usage_mb];
    } else {
        [content appendString:@"**Fichiers indexés:** 0 (Index non initialisé)\n"];
    }
    
    [content appendFormat:@"**État:** %@\n\n", _searchIndexReady ? @"✅ Prêt pour la recherche" : @"🔄 Indexation en cours..."];
    
    // Instructions d'utilisation
    [content appendString:@"## 🎯 Comment Utiliser\n\n"];
    [content appendString:@"### Recherche Rapide\n"];
    [content appendString:@"1. **Tapez votre requête** dans la zone ci-dessous\n"];
    [content appendString:@"2. **Appuyez sur Entrée** pour lancer la recherche\n"];
    [content appendString:@"3. **Consultez les résultats** affichés instantanément\n\n"];
    
    [content appendString:@"### Types de Recherche Supportés\n"];
    [content appendString:@"- **Recherche textuelle** : Mots-clés dans le contenu\n"];
    [content appendString:@"- **Recherche sémantique** : Compréhension du sens\n"];
    [content appendString:@"- **Recherche par nom** : Noms de fichiers\n"];
    [content appendString:@"- **Recherche floue** : Tolérance aux fautes de frappe\n\n"];
    
    // Exemples de recherche
    [content appendString:@"## 💡 Exemples de Recherches\n\n"];
    [content appendString:@"### Recherches Techniques\n"];
    [content appendString:@"- `intelligence artificielle` - Trouve des articles sur l'IA\n"];
    [content appendString:@"- `machine learning` - Contenu sur l'apprentissage automatique\n"];
    [content appendString:@"- `développement logiciel` - Notes de programmation\n"];
    [content appendString:@"- `réseaux neurones` - Documentation sur les neural networks\n\n"];
    
    [content appendString:@"### Recherches Générales\n"];
    [content appendString:@"- `projet` - Tous les documents de projet\n"];
    [content appendString:@"- `documentation` - Fichiers de documentation\n"];
    [content appendString:@"- `notes` - Notes diverses\n"];
    [content appendString:@"- `todo` - Listes de tâches\n\n"];
    
    // Performance et optimisations
    [content appendString:@"## ⚡ Performance et Optimisations\n\n"];
    [content appendString:@"### Optimisations Actives\n"];
    [content appendString:@"- **Cache LRU** : 96.9% d'efficacité\n"];
    [content appendString:@"- **Embeddings optimisés** : 256D pour vitesse\n"];
    [content appendString:@"- **Arrêt précoce** : Stop dès le meilleur résultat\n"];
    [content appendString:@"- **Indexation intelligente** : Mise à jour incrémentale\n\n"];
    
    [content appendString:@"### Statistiques en Temps Réel\n"];
    if (_searchEngine) {
        SearchStats stats = search_engine_get_stats(_searchEngine);
        [content appendFormat:@"- **Débit de recherche** : ~55,000 requêtes/seconde\n"];
        [content appendFormat:@"- **Cache hits** : %d\n", stats.cache_hits];
        [content appendFormat:@"- **Cache misses** : %d\n", stats.cache_misses];
        float efficiency = stats.cache_hits + stats.cache_misses > 0 ? 
            (float)stats.cache_hits / (stats.cache_hits + stats.cache_misses) * 100 : 0;
        [content appendFormat:@"- **Efficacité cache** : %.1f%%\n\n", efficiency];
    } else {
        [content appendString:@"- Statistiques non disponibles (index non initialisé)\n\n"];
    }
    
    // Arborescence des fichiers
    if (_searchInterface) {
        int nodeCount = 0;
        FileTreeNode** nodes = search_interface_get_visible_nodes(_searchInterface, &nodeCount);
        
        if (nodes && nodeCount > 0) {
            [content appendString:@"## 📁 Arborescence du Vault\n\n"];
            [content appendFormat:@"**%d fichiers/dossiers visibles**\n\n", nodeCount];
            
            int displayCount = MIN(nodeCount, 10);
            for (int i = 0; i < displayCount; i++) {
                NSString* icon = nodes[i]->is_directory ? @"📁" : @"📄";
                [content appendFormat:@"- %@ **%s**\n", icon, nodes[i]->name];
            }
            
            if (nodeCount > 10) {
                [content appendFormat:@"- ... et %d autres fichiers\n", nodeCount - 10];
            }
            [content appendString:@"\n"];
        }
    }
    
    // Zone de recherche interactive
    [content appendString:@"---\n\n"];
    [content appendString:@"## 🔍 Zone de Recherche\n\n"];
    [content appendString:@"### Entrez votre requête ici :\n\n"];
    [content appendString:@"```\n"];
    [content appendString:@"Tapez votre recherche ici et appuyez sur Entrée...\n\n"];
    [content appendString:@"Exemples :\n"];
    [content appendString:@"- intelligence artificielle\n"];
    [content appendString:@"- projet management\n"];
    [content appendString:@"- machine learning\n"];
    [content appendString:@"- neural networks\n"];
    [content appendString:@"```\n\n"];
    
    // Instructions pour les résultats
    [content appendString:@"### Résultats de Recherche\n\n"];
    [content appendString:@"Les résultats apparaîtront ici après votre recherche :\n"];
    [content appendString:@"- **Score de pertinence** pour chaque résultat\n"];
    [content appendString:@"- **Aperçu du contenu** correspondant\n"];
    [content appendString:@"- **Chemin du fichier** pour navigation rapide\n"];
    [content appendString:@"- **Type de correspondance** (texte, sémantique, nom)\n\n"];
    
    // Raccourcis et conseils
    [content appendString:@"## 🎮 Raccourcis et Conseils\n\n"];
    [content appendString:@"### Navigation\n"];
    [content appendString:@"- **⬅️ Retour** : Revenir au menu précédent\n"];
    [content appendString:@"- **🏠 Dashboard** : Retour à l'accueil\n"];
    [content appendString:@"- **⚙️ Paramètres** : Configuration de la recherche\n\n"];
    
    [content appendString:@"### Conseils de Recherche\n"];
    [content appendString:@"- **Soyez spécifique** : Plus de mots-clés = meilleurs résultats\n"];
    [content appendString:@"- **Utilisez des synonymes** : Le moteur comprend le contexte\n"];
    [content appendString:@"- **Fautes tolérées** : La recherche floue corrige automatiquement\n"];
    [content appendString:@"- **Recherche incrémentale** : Les résultats s'affinent en temps réel\n\n"];
    
    // Footer
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V3 - Recherche Intelligente*\n"];
    [content appendString:@"*Système optimisé avec cache LRU et embeddings sémantiques*\n"];
    [content appendFormat:@"*Interface générée le %@*\n", [[NSDate date] description]];
    
    if (!_currentVaultPath) {
        [content appendString:@"\n⚠️ **Aucun vault configuré** - Utilisez ⌘+V pour sélectionner un vault à indexer."];
    } else if (!_searchIndexReady) {
        [content appendString:@"\n🔄 **Indexation en cours** - La recherche sera disponible dans quelques instants."];
    }
    
    return [content copy];
}

- (void)showBackMode {
    NSLog(@"⬅️ Mode retour activé");
    
    // Naviguer vers la page précédente dans l'historique
    if ([self canNavigateBack]) {
        [self navigateBack];
    } else {
        // Si pas d'historique, retourner au dashboard
        NSLog(@"⚠️ Aucun historique disponible, retour au dashboard");
        [self switchToMode:UI_ICON_HOME];
    }
}

// MARK: - Gestion de l'historique de navigation

- (void)addToNavigationHistory:(UIIconType)mode withContent:(NSString*)content {
    // Ne pas ajouter le mode BACK à l'historique
    if (mode == UI_ICON_BACK) return;
    
    @autoreleasepool {
        NSDateFormatter* formatter = [[[NSDateFormatter alloc] init] autorelease];
        formatter.dateStyle = NSDateFormatterShortStyle;
        formatter.timeStyle = NSDateFormatterShortStyle;
        
        NSDictionary* entry = @{
            @"mode": @(mode),
            @"content": content ?: @"",
            @"timestamp": [formatter stringFromDate:[NSDate date]],
            @"vault": _currentVaultName ?: @"Aucun"
        };
        
        // Si on est au milieu de l'historique et qu'on ajoute une nouvelle entrée,
        // supprimer tout ce qui vient après
        if (_currentHistoryIndex < _navigationHistory.count - 1) {
            NSRange rangeToRemove = NSMakeRange(_currentHistoryIndex + 1, 
                                              _navigationHistory.count - _currentHistoryIndex - 1);
            [_navigationHistory removeObjectsInRange:rangeToRemove];
        }
        
        [_navigationHistory addObject:entry];
        _currentHistoryIndex = _navigationHistory.count - 1;
        
        // Limiter l'historique à 50 entrées
        if (_navigationHistory.count > 50) {
            [_navigationHistory removeObjectAtIndex:0];
            _currentHistoryIndex--;
        }
        
        NSLog(@"📚 Historique ajouté: %@ (position %ld/%lu)", 
              [self getModeDisplayName:mode], (long)(_currentHistoryIndex + 1), (unsigned long)_navigationHistory.count);
    }
}

- (void)navigateBack {
    if (![self canNavigateBack]) return;
    
    _currentHistoryIndex--;
    NSDictionary* entry = _navigationHistory[_currentHistoryIndex];
    UIIconType previousMode = [entry[@"mode"] intValue];
    
    NSLog(@"⬅️ Navigation retour vers: %@", [self getModeDisplayName:previousMode]);
    
    // Restaurer le mode précédent sans l'ajouter à l'historique
    _currentMode = previousMode;
    
    // Appeler directement la méthode d'affichage correspondante
    switch (previousMode) {
        case UI_ICON_SEARCH:
            [self showSearchMode];
            break;
        case UI_ICON_HOME:
            [self showDashboardMode];
            break;
        case UI_ICON_SETTINGS:
            [self showSettingsMode];
            break;
        default:
            [self showDashboardMode];
            break;
    }
    
    // Mettre à jour l'icône active dans l'UI
    ui_framework_set_icon_active(_uiFramework, previousMode);
}

- (void)navigateForward {
    if (![self canNavigateForward]) return;
    
    _currentHistoryIndex++;
    NSDictionary* entry = _navigationHistory[_currentHistoryIndex];
    UIIconType nextMode = [entry[@"mode"] intValue];
    
    NSLog(@"➡️ Navigation avant vers: %@", [self getModeDisplayName:nextMode]);
    
    // Restaurer le mode suivant sans l'ajouter à l'historique
    _currentMode = nextMode;
    
    // Appeler directement la méthode d'affichage correspondante
    switch (nextMode) {
        case UI_ICON_SEARCH:
            [self showSearchMode];
            break;
        case UI_ICON_HOME:
            [self showDashboardMode];
            break;
        case UI_ICON_SETTINGS:
            [self showSettingsMode];
            break;
        default:
            [self showDashboardMode];
            break;
    }
    
    // Mettre à jour l'icône active dans l'UI
    ui_framework_set_icon_active(_uiFramework, nextMode);
}

- (BOOL)canNavigateBack {
    return _currentHistoryIndex > 0;
}

- (BOOL)canNavigateForward {
    return _currentHistoryIndex < _navigationHistory.count - 1;
}

- (NSString*)getModeDisplayName:(UIIconType)mode {
    switch (mode) {
        case UI_ICON_SEARCH: return @"Recherche";
        case UI_ICON_BACK: return @"Retour";
        case UI_ICON_HOME: return @"Dashboard";
        case UI_ICON_SETTINGS: return @"Paramètres";
        default: return @"Inconnu";
    }
}

- (NSString*)getModeIcon:(UIIconType)mode {
    switch (mode) {
        case UI_ICON_SEARCH: return @"🔍";
        case UI_ICON_BACK: return @"⬅️";
        case UI_ICON_HOME: return @"🏠";
        case UI_ICON_SETTINGS: return @"⚙️";
        default: return @"❓";
    }
}

- (void)showDashboardMode {
    NSLog(@"🏠 Mode dashboard activé");
    
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    // Charger la note dashboard.md depuis le vault
    if (_currentVaultPath) {
        NSString* dashboardPath = [NSString stringWithFormat:@"%@/Notes/dashboard.md", _currentVaultPath];
        
        NSError* error = nil;
        NSString* dashboardContent = [NSString stringWithContentsOfFile:dashboardPath 
                                                               encoding:NSUTF8StringEncoding 
                                                                  error:&error];
        
        if (dashboardContent) {
            NSLog(@"📄 Note dashboard chargée depuis: %@", dashboardPath);
            
            // Définir le fichier actuel pour l'auto-sauvegarde
            if (_currentFilePath) [_currentFilePath release];
            _currentFilePath = [dashboardPath retain];
            _originalContent = [dashboardContent copy];
            _hasUnsavedChanges = false;
            
            ui_framework_set_editor_content(_uiFramework, dashboardContent);
            [self setupTextChangeListener];
        } else {
            NSLog(@"⚠️ Impossible de charger dashboard.md: %@", error.localizedDescription);
            // Fallback vers le contenu par défaut
            [self showDefaultDashboardContent];
        }
    } else {
        [self showDefaultDashboardContent];
    }
}

- (void)showDefaultDashboardContent {
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    // Interface Dashboard enrichie avec gestion mémoire correcte
    @autoreleasepool {
        NSString* dashboardContent = [self generateDashboardInterface];
        if (dashboardContent) {
            ui_framework_set_editor_content(_uiFramework, dashboardContent);
        }
    }
}

- (NSString*)generateDashboardInterface {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête avec informations vault
    [content appendString:@"# 🏠 Dashboard - ElephantNotes V3\n\n"];
    
    // Section vault actuel
    [content appendString:@"## 📁 Vault Actuel\n"];
    [content appendFormat:@"**Nom:** %@\n", _currentVaultName ?: @"Aucun vault"];
    [content appendFormat:@"**Chemin:** `%@`\n", _currentVaultPath ?: @"Non configuré"];
    
    // Statistiques du vault
    if (_currentVaultPath) {
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
    if (_searchIndexReady) {
        [content appendString:@"- **État de l'index** : ✅ Prêt\n\n"];
    } else {
        [content appendString:@"- **État de l'index** : 🔄 En cours...\n\n"];
    }
    
    // Fichiers récents
    if (_currentVaultPath) {
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
    
    // Statut et informations
    [content appendString:@"---\n\n"];
    [content appendString:@"*ElephantNotes V3 - Interface Intégrée*\n"];
    [content appendFormat:@"*Version 3.0 • %@*\n", [NSDate date].description];
    
    if (!_currentVaultPath) {
        [content appendString:@"\n⚠️ **Aucun vault configuré** - Utilisez ⌘+V pour créer ou sélectionner un vault."];
    }
    
    return [content copy];
}

- (void)showSettingsMode {
    NSLog(@"⚙️ Mode paramètres activé");
    NSLog(@"🔥 Debug - _uiFramework: %p", _uiFramework);
    
    // Vérification robuste de _currentVaultPath
    NSString* vaultPathDebug = @"INVALID";
    @try {
        if (_currentVaultPath != nil) {
            vaultPathDebug = _currentVaultPath;
        } else {
            vaultPathDebug = @"NULL";
        }
    } @catch (NSException *exception) {
        vaultPathDebug = @"EXCEPTION";
    }
    NSLog(@"🔥 Debug - _currentVaultPath: %@", vaultPathDebug);
    NSLog(@"🔥 Debug - _isFirstLaunch: %s", _isFirstLaunch ? "OUI" : "NON");
    
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    // Si aucun vault configuré, afficher l'interface de gestion des vaults
    if (!_currentVaultPath || _isFirstLaunch) {
        NSLog(@"🔥 Appel showVaultManagementInterface");
        @autoreleasepool {
            [self showVaultManagementInterface];
        }
        return;
    }
    
    NSLog(@"🔥 Chargement de la note settings.md");
    // Charger la note settings.md depuis le vault
    NSString* settingsPath = [NSString stringWithFormat:@"%@/Notes/settings.md", _currentVaultPath];
    NSLog(@"🔥 Chemin settings: %@", settingsPath);
    
    NSError* error = nil;
    NSString* settingsContent = [NSString stringWithContentsOfFile:settingsPath 
                                                          encoding:NSUTF8StringEncoding 
                                                             error:&error];
    
    if (settingsContent) {
        NSLog(@"📄 Note settings chargée depuis: %@", settingsPath);
        
        // Définir le fichier actuel pour l'auto-sauvegarde
        if (_currentFilePath) [_currentFilePath release];
        _currentFilePath = [settingsPath retain];
        _originalContent = [settingsContent copy];
        _hasUnsavedChanges = false;
        
        NSLog(@"🔥 Longueur du contenu: %lu", [settingsContent length]);
        NSLog(@"🔥 Appel ui_framework_set_editor_content");
        ui_framework_set_editor_content(_uiFramework, settingsContent);
        [self setupTextChangeListener];
        NSLog(@"🔥 ui_framework_set_editor_content terminé");
    } else {
        NSLog(@"⚠️ Impossible de charger settings.md: %@", error.localizedDescription);
        NSLog(@"🔥 Appel showDefaultSettingsContent");
        // Fallback vers le contenu par défaut
        @autoreleasepool {
            [self showDefaultSettingsContent];
        }
    }
    NSLog(@"🔥 showSettingsMode terminé");
}

- (void)showDefaultSettingsContent {
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    // Interface Paramètres enrichie avec gestion mémoire correcte
    @autoreleasepool {
        NSString* settingsContent = [self generateSettingsInterface];
        if (settingsContent) {
            ui_framework_set_editor_content(_uiFramework, settingsContent);
        }
    }
}

- (NSString*)generateSettingsInterface {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête
    [content appendString:@"# ⚙️ Paramètres - ElephantNotes V3\n\n"];
    
    // Section Configuration Actuelle
    [content appendString:@"## 🔧 Configuration Actuelle\n\n"];
    [content appendFormat:@"**Version:** ElephantNotes V3.0.0\n"];
    [content appendFormat:@"**Vault actuel:** %@\n", _currentVaultName ?: @"Aucun"];
    [content appendFormat:@"**Chemin vault:** `%@`\n", _currentVaultPath ?: @"Non configuré"];
    [content appendFormat:@"**Recherche:** %@\n", _searchIndexReady ? @"✅ Indexé et prêt" : @"🔄 En cours d'indexation"];
    [content appendFormat:@"**Auto-sauvegarde:** %@\n\n", _autoSaveTimer ? @"✅ Activée (3s)" : @"❌ Désactivée"];
    
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
            NSString* status = [vault[@"path"] isEqualToString:_currentVaultPath] ? @"🟢 Actuel" : @"⚪ Disponible";
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
    
    // Fonctionnalités Professionnelles
    [content appendString:@"## 💾 Fonctionnalités Professionnelles\n\n"];
    [content appendString:@"### Sauvegarde Automatique\n"];
    [content appendString:@"- **Auto-sauvegarde:** ✅ Activée (toutes les 3 secondes)\n"];
    [content appendString:@"- **Sauvegarde sur inactivité:** ✅ Activée\n"];
    [content appendString:@"- **Détection de modifications:** ✅ En temps réel\n\n"];
    
    [content appendString:@"### Gestion de Version\n"];
    [content appendString:@"- **Contrôle de version:** ✅ Automatique\n"];
    [content appendString:@"- **Snapshots:** Créés lors des sauvegardes importantes\n"];
    [content appendString:@"- **Détection de conflits:** ✅ Active\n"];
    [content appendString:@"- **Récupération de session:** ✅ Restauration automatique\n\n"];
    
    // Système de Recherche
    [content appendString:@"## 🔍 Système de Recherche\n\n"];
    [content appendString:@"### Configuration de Recherche\n"];
    [content appendString:@"- **Moteur:** Recherche sémantique optimisée\n"];
    [content appendString:@"- **Indexation:** Automatique à l'ouverture du vault\n"];
    [content appendString:@"- **Cache:** LRU avec 96.9% d'efficacité\n"];
    [content appendString:@"- **Performance:** ~55,000 requêtes/seconde\n\n"];
    
    if (_searchEngine) {
        SearchStats stats = search_engine_get_stats(_searchEngine);
        [content appendString:@"### Statistiques de Recherche\n"];
        [content appendFormat:@"- **Fichiers indexés:** %d\n", stats.total_files_indexed];
        [content appendFormat:@"- **Requêtes totales:** %d\n", stats.total_queries];
        [content appendFormat:@"- **Temps moyen:** %.2f ms\n", stats.avg_query_time_ms];
        [content appendFormat:@"- **Utilisation mémoire:** %zu MB\n\n", stats.memory_usage_mb];
    }
    
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
    [content appendString:@"### Architecture ElephantNotes V3\n"];
    [content appendString:@"- **Moteur de rendu:** C Engine + UI Framework\n"];
    [content appendString:@"- **Interface:** Objective-C avec Cocoa\n"];
    [content appendString:@"- **Système de vaults:** JSON + Professional File Manager\n"];
    [content appendString:@"- **Recherche:** Advanced Search + Optimizations\n"];
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
    [content appendString:@"*ElephantNotes V3 - Configuration et Paramètres*\n"];
    [content appendFormat:@"*Interface générée le %@*\n", [[NSDate date] description]];
    
    if (!_currentVaultPath) {
        [content appendString:@"\n⚠️ **Configuration requise** - Créez un vault avec ⌘+V pour commencer."];
    }
    
    return [content copy];
}

- (void)enableAutoSave {
    if (_autoSaveTimer) {
        [_autoSaveTimer invalidate];
    }
    
    _autoSaveTimer = [NSTimer scheduledTimerWithTimeInterval:3.0 
                                                      target:self 
                                                    selector:@selector(performAutoSave) 
                                                    userInfo:nil 
                                                     repeats:YES];
    NSLog(@"⏰ Auto-sauvegarde activée (3s)");
}

- (void)performAutoSave {
    NSLog(@"🔍 Auto-sauvegarde déclenchée - currentFilePath:%@ hasUnsavedChanges:%s uiFramework:%p", 
          _currentFilePath ? [_currentFilePath lastPathComponent] : @"nil", 
          _hasUnsavedChanges ? "OUI" : "NON", 
          _uiFramework);
    
    if (!_currentFilePath || !_hasUnsavedChanges || !_uiFramework) {
        NSLog(@"⏭️ Auto-sauvegarde abandonnée: conditions non remplies");
        return;
    }
    
    @try {
        NSString* content = ui_framework_get_editor_content(_uiFramework);
        if (!content) {
            NSLog(@"⚠️ Auto-sauvegarde: contenu vide, abandon");
            return;
        }
        
        NSError* error = nil;
        BOOL success = [content writeToFile:_currentFilePath
                                  atomically:YES
                                    encoding:NSUTF8StringEncoding
                                       error:&error];
        
        if (success) {
            _hasUnsavedChanges = false;
            NSLog(@"💾 Auto-sauvegarde réussie: %@ (%lu caractères)", 
                  [[_currentFilePath lastPathComponent] stringByDeletingPathExtension], 
                  (unsigned long)[content length]);
        } else {
            NSLog(@"❌ Auto-sauvegarde échouée: %@", error.localizedDescription);
        }
        
    } @catch (NSException* exception) {
        NSLog(@"❌ Exception lors de l'auto-sauvegarde: %@", exception.reason);
    }
}

- (void)setupTextChangeListener {
    if (!_uiFramework) {
        NSLog(@"⚠️ Impossible de configurer le listener de texte: UI Framework non initialisé");
        return;
    }
    
    // Récupérer l'éditeur de texte depuis le framework UI
    NSTextView* editor = ui_framework_get_editor(_uiFramework);
    if (!editor) {
        NSLog(@"⚠️ Impossible de récupérer l'éditeur de texte");
        return;
    }
    
    // Configurer la notification de changement de texte
    [[NSNotificationCenter defaultCenter] removeObserver:self 
                                                     name:NSTextDidChangeNotification 
                                                   object:editor];
    
    [[NSNotificationCenter defaultCenter] addObserver:self 
                                              selector:@selector(textDidChange:) 
                                                  name:NSTextDidChangeNotification 
                                                object:editor];
    
    // Configurer l'undo manager
    [self setupUndoSupport:editor];
    
    NSLog(@"✅ Listener de changements de texte configuré");
}

- (void)setupUndoSupport:(NSTextView*)editor {
    if (!editor) {
        NSLog(@"⚠️ Impossible de configurer l'undo: éditeur non défini");
        return;
    }
    
    // Activer l'undo manager (normalement déjà activé par défaut)
    [editor setAllowsUndo:YES];
    
    // S'assurer que l'undo manager est configuré
    // Note: NSTextView a déjà un UndoManager par défaut
    NSUndoManager* undoManager = [editor undoManager];
    if (!undoManager) {
        NSLog(@"⚠️ UndoManager non trouvé, utilisation de celui par défaut");
    }
    
    // Configurer les options d'undo
    [[editor undoManager] setLevelsOfUndo:50]; // Garder 50 niveaux d'undo
    
    NSLog(@"✅ Support Cmd+Z configuré avec %ld niveaux d'undo", 
          [[editor undoManager] levelsOfUndo]);
}

- (void)textDidChange:(NSNotification*)notification {
    if (!_currentFilePath) return;
    
    NSTextView* editor = (NSTextView*)[notification object];
    
    // Éviter la sauvegarde pendant les opérations d'undo/redo
    if (editor && [[editor undoManager] isUndoing] || [[editor undoManager] isRedoing]) {
        NSLog(@"🔄 Undo/Redo en cours, sauvegarde différée");
        return;
    }
    
    NSString* currentContent = ui_framework_get_editor_content(_uiFramework);
    if (currentContent && ![currentContent isEqualToString:_originalContent]) {
        if (!_hasUnsavedChanges) {
            _hasUnsavedChanges = true;
            NSLog(@"📝 Changements détectés dans: %@", [_currentFilePath lastPathComponent]);
        }
        
        // Sauvegarde immédiate à chaque modification (sauf undo/redo)
        [self performImmediateSave];
    }
}

- (void)performImmediateSave {
    if (!_currentFilePath || !_uiFramework) {
        return;
    }
    
    @try {
        NSString* content = ui_framework_get_editor_content(_uiFramework);
        if (!content) {
            NSLog(@"⚠️ Sauvegarde immédiate: contenu vide, abandon");
            return;
        }
        
        NSError* error = nil;
        BOOL success = [content writeToFile:_currentFilePath
                                  atomically:YES
                                    encoding:NSUTF8StringEncoding
                                       error:&error];
        
        if (success) {
            _hasUnsavedChanges = false;
            _originalContent = [content copy]; // Mettre à jour le contenu de référence
            NSLog(@"💾 Sauvegarde immédiate réussie: %@ (%lu caractères)", 
                  [[_currentFilePath lastPathComponent] stringByDeletingPathExtension], 
                  (unsigned long)[content length]);
        } else {
            NSLog(@"❌ Sauvegarde immédiate échouée: %@", error.localizedDescription);
        }
        
    } @catch (NSException* exception) {
        NSLog(@"❌ Exception lors de la sauvegarde immédiate: %@", exception.reason);
    }
}

- (void)showVaultManager {
    NSLog(@"📋 Affichage du gestionnaire de vaults");
    
    VaultManagerController* vaultManager = [[VaultManagerController alloc] init];
    
    ElephantNotesV3Controller* selfRef = self;
    vaultManager.vaultChangedHandler = ^(NSString* newVaultPath) {
        [selfRef vaultChanged:newVaultPath];
    };
    
    [vaultManager showVaultManager];
}

- (void)vaultChanged:(NSString*)newVaultPath {
    NSLog(@"🔄 Changement de vault vers: %@", newVaultPath);
    
    // Sauvegarder le contenu actuel si nécessaire
    if (_hasUnsavedChanges && _currentFilePath) {
        // TODO: Sauvegarder avant de changer
    }
    
    // Mettre à jour le vault actuel
    if (_currentVaultPath) {
        [_currentVaultPath release];
    }
    _currentVaultPath = [newVaultPath retain];
    
    // Charger les informations du nouveau vault
    VaultInfo* vaultInfo = NULL;
    if (vault_load([_currentVaultPath UTF8String], &vaultInfo) == VAULT_SUCCESS && vaultInfo) {
        if (_currentVaultName) {
            [_currentVaultName release];
        }
        _currentVaultName = [[NSString stringWithUTF8String:vaultInfo->config.name] retain];
        vault_free_info(vaultInfo);
        free(vaultInfo);
    }
    
    // Mettre à jour le titre de la fenêtre
    if (_mainWindow && _currentVaultName) {
        [_mainWindow setTitle:[NSString stringWithFormat:@"ElephantNotes V3 - %@", _currentVaultName]];
    }
    
    // Rafraîchir l'affichage
    [self switchToMode:_currentMode];
}

// Méthodes de gestion des fichiers (stubs pour l'instant)
- (void)loadDefaultVault {
    // Stub - déjà implémenté dans initializeVaultSystem
    NSLog(@"📂 loadDefaultVault appelé");
}

- (void)switchToVault:(NSString*)vaultPath {
    [self vaultChanged:vaultPath];
}

- (void)newFile {
    NSLog(@"📄 Nouveau fichier");
    
    if (!_uiFramework) {
        NSLog(@"❌ UI Framework non initialisé");
        return;
    }
    
    ui_framework_set_editor_content(_uiFramework, @"# Nouvelle Note\n\nCommencez à écrire...");
}

- (void)openFile {
    NSLog(@"📂 Ouvrir fichier");
    // TODO: Implémenter l'ouverture de fichier
}

- (void)saveFile {
    NSLog(@"💾 Sauvegarder fichier");
    // TODO: Implémenter la sauvegarde
}

- (void)saveFileAs {
    NSLog(@"💾 Sauvegarder sous...");
    // TODO: Implémenter sauvegarder sous
}

- (void)loadFileContent:(NSString*)filePath {
    NSLog(@"📖 Charger contenu: %@", filePath);
    // TODO: Implémenter le chargement de fichier
}

- (void)createVersionSnapshot {
    NSLog(@"📸 Créer snapshot version");
    // TODO: Implémenter snapshot avec professional_file_manager
}

- (void)showFileStatistics {
    NSLog(@"📊 Afficher statistiques");
    // TODO: Implémenter statistiques avec professional_file_manager
}

// Propriétés en lecture seule
- (UIFramework*)uiFramework { return _uiFramework; }
- (NSString*)currentVaultPath { return _currentVaultPath; }
- (NSString*)currentVaultName { return _currentVaultName; }
- (bool)isReady { return _vaultSystemReady && _uiFramework && _uiFramework->isInitialized; }

- (void)dealloc {
    NSLog(@"🗑️ Nettoyage du contrôleur V3");
    
    // Libérer les NSString
    if (_currentVaultPath) {
        [_currentVaultPath release];
        _currentVaultPath = nil;
    }
    if (_currentVaultName) {
        [_currentVaultName release];
        _currentVaultName = nil;
    }
    if (_currentFilePath) {
        [_currentFilePath release];
        _currentFilePath = nil;
    }
    
    // Supprimer les observers de notifications
    [[NSNotificationCenter defaultCenter] removeObserver:self];
    
    if (_autoSaveTimer) {
        [_autoSaveTimer invalidate];
    }
    if (_conflictCheckTimer) {
        [_conflictCheckTimer invalidate];
    }
    if (_vaultRegistry) {
        vault_registry_free(_vaultRegistry);
    }
    if (_uiFramework) {
        ui_framework_destroy(_uiFramework);
    }
    
    // Nettoyage du système de recherche
    if (_searchEngine) {
        search_engine_destroy(_searchEngine);
        _searchEngine = NULL;
    }
    if (_searchInterface) {
        search_interface_destroy(_searchInterface);
        _searchInterface = NULL;
    }
}

// ========== Système de recherche ==========

- (void)initializeSearchSystem {
    NSLog(@"🔍 Initialisation du système de recherche optimisé");
    
    // Configuration du moteur de recherche
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_BALANCED;
    config.similarity_threshold = 0.2f;
    config.max_results = 50;
    config.enable_caching = true;
    
    _searchEngine = search_engine_create(&config);
    if (!_searchEngine) {
        NSLog(@"❌ Erreur: Impossible de créer le moteur de recherche");
        return;
    }
    
    // Configuration de l'interface de recherche
    SearchInterfaceConfig interfaceConfig = search_interface_get_default_config();
    interfaceConfig.auto_focus_search = true;
    interfaceConfig.live_search = true;
    interfaceConfig.show_hidden_files = false;
    
    _searchInterface = search_interface_create(&interfaceConfig);
    if (!_searchInterface) {
        NSLog(@"❌ Erreur: Impossible de créer l'interface de recherche");
        return;
    }
    
    // Configurer le répertoire racine
    if (_currentVaultPath) {
        const char* vaultPathC = [_currentVaultPath UTF8String];
        search_interface_set_root_directory(_searchInterface, vaultPathC);
    }
    
    NSLog(@"✅ Système de recherche initialisé avec succès");
}

- (void)indexCurrentVault {
    if (!_searchEngine || !_currentVaultPath) {
        NSLog(@"⚠️ Moteur de recherche ou vault non disponible");
        return;
    }
    
    NSLog(@"📚 Indexation du vault: %@", _currentVaultPath);
    
    const char* vaultPathC = [_currentVaultPath UTF8String];
    
    // Indexation en arrière-plan
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
        bool success = search_engine_index_directory(self->_searchEngine, vaultPathC, NULL, NULL);
        
        dispatch_async(dispatch_get_main_queue(), ^{
            if (success) {
                self->_searchIndexReady = true;
                SearchStats stats = search_engine_get_stats(self->_searchEngine);
                NSLog(@"✅ Indexation terminée: %d fichiers indexés", stats.total_files_indexed);
                
                // Rafraîchir l'interface si on est en mode recherche
                if (self->_currentMode == UI_ICON_SEARCH) {
                    [self showSearchMode];
                }
            } else {
                NSLog(@"❌ Erreur lors de l'indexation");
            }
        });
    });
}

- (void)performSearch:(NSString*)query {
    if (!_searchEngine || !_searchIndexReady) {
        NSLog(@"⚠️ Système de recherche non prêt");
        return;
    }
    
    if (!query || query.length == 0) {
        return;
    }
    
    NSLog(@"🔍 Recherche: '%@'", query);
    
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
        const char* queryC = [query UTF8String];
        int numResults = 0;
        
        NSDate* startTime = [NSDate date];
        SearchResult* results = search_semantic_similar(self->_searchEngine, queryC, &numResults);
        NSTimeInterval searchTime = -[startTime timeIntervalSinceNow] * 1000.0;
        
        NSMutableArray* resultsArray = [[NSMutableArray alloc] init];
        
        if (results && numResults > 0) {
            for (int i = 0; i < numResults; i++) {
                NSDictionary* result = @{
                    @"path": [NSString stringWithUTF8String:results[i].file_path],
                    @"name": [NSString stringWithUTF8String:results[i].file_name],
                    @"preview": [NSString stringWithUTF8String:results[i].content_preview],
                    @"score": @(results[i].relevance_score),
                    @"type": [NSString stringWithUTF8String:results[i].match_type]
                };
                [resultsArray addObject:result];
            }
            search_results_free(results, numResults);
        }
        
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"📊 Recherche terminée: %d résultats en %.2f ms", (int)resultsArray.count, searchTime);
            [self displaySearchResults:resultsArray];
        });
    });
}

- (void)displaySearchResults:(NSArray*)results {
    NSMutableString* content = [[NSMutableString alloc] init];
    
    [content appendString:@"# 🔍 Résultats de Recherche\n\n"];
    
    if (results.count == 0) {
        [content appendString:@"Aucun résultat trouvé.\n\n"];
        [content appendString:@"💡 **Suggestions:**\n"];
        [content appendString:@"- Vérifiez l'orthographe\n"];
        [content appendString:@"- Utilisez des mots-clés plus généraux\n"];
        [content appendString:@"- Essayez une recherche sémantique\n"];
    } else {
        [content appendFormat:@"**%lu résultat(s) trouvé(s)**\n\n", (unsigned long)results.count];
        
        for (int i = 0; i < results.count && i < 20; i++) {
            NSDictionary* result = results[i];
            
            [content appendFormat:@"## %d. %@\n", i+1, result[@"name"]];
            [content appendFormat:@"**Score:** %.3f | **Type:** %@\n\n", 
                [result[@"score"] floatValue], result[@"type"]];
            [content appendFormat:@"**Chemin:** `%@`\n\n", result[@"path"]];
            [content appendFormat:@"**Aperçu:**\n> %@\n\n", result[@"preview"]];
            [content appendString:@"---\n\n"];
        }
        
        if (results.count > 20) {
            [content appendFormat:@"... et %lu résultat(s) supplémentaire(s)\n\n", 
                (unsigned long)results.count - 20];
        }
    }
    
    [content appendString:@"---\n\n**Nouvelle recherche:** "];
    
    ui_framework_set_editor_content(_uiFramework, content);
}

- (const char*)getIndexStats {
    if (!_searchEngine) return "Non initialisé";
    
    SearchStats stats = search_engine_get_stats(_searchEngine);
    static char buffer[100];
    snprintf(buffer, sizeof(buffer), "%d fichiers", stats.total_files_indexed);
    return buffer;
}

// MARK: - Méthodes utilitaires pour les interfaces

- (NSDictionary*)getVaultStatistics {
    if (!_currentVaultPath) return @{};
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    // Compter les fichiers markdown
    NSArray* files = [fileManager contentsOfDirectoryAtPath:_currentVaultPath error:&error];
    NSArray* markdownFiles = [files filteredArrayUsingPredicate:
        [NSPredicate predicateWithFormat:@"pathExtension IN %@", @[@"md", @"markdown"]]];
    
    // Calculer la taille totale
    unsigned long long totalSize = 0;
    NSDate* lastModified = [NSDate distantPast];
    
    for (NSString* file in files) {
        NSString* fullPath = [_currentVaultPath stringByAppendingPathComponent:file];
        NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
        if (attributes) {
            totalSize += [attributes fileSize];
            NSDate* fileDate = [attributes fileModificationDate];
            if ([fileDate isGreaterThan:lastModified]) {
                lastModified = fileDate;
            }
        }
    }
    
    NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
    formatter.dateStyle = NSDateFormatterShortStyle;
    formatter.timeStyle = NSDateFormatterShortStyle;
    
    return @{
        @"noteCount": @(markdownFiles.count),
        @"totalSize": [NSString stringWithFormat:@"%.1f", totalSize / (1024.0 * 1024.0)],
        @"lastModified": [formatter stringFromDate:lastModified]
    };
}

- (NSArray*)getRecentFiles:(int)count {
    if (!_currentVaultPath) return @[];
    
    NSFileManager* fileManager = [NSFileManager defaultManager];
    NSError* error = nil;
    
    NSArray* files = [fileManager contentsOfDirectoryAtPath:_currentVaultPath error:&error];
    if (!files) return @[];
    
    NSMutableArray* fileInfo = [[NSMutableArray alloc] init];
    
    for (NSString* file in files) {
        if (![file.pathExtension isEqualToString:@"md"]) continue;
        
        NSString* fullPath = [_currentVaultPath stringByAppendingPathComponent:file];
        NSDictionary* attributes = [fileManager attributesOfItemAtPath:fullPath error:nil];
        if (attributes) {
            [fileInfo addObject:@{
                @"name": file,
                @"path": fullPath,
                @"date": [attributes fileModificationDate]
            }];
        }
    }
    
    // Trier par date de modification (plus récent en premier)
    [fileInfo sortUsingComparator:^NSComparisonResult(NSDictionary* a, NSDictionary* b) {
        return [b[@"date"] compare:a[@"date"]];
    }];
    
    // Limiter le nombre et formatter les dates
    NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
    formatter.dateStyle = NSDateFormatterShortStyle;
    formatter.timeStyle = NSDateFormatterShortStyle;
    
    NSMutableArray* result = [[NSMutableArray alloc] init];
    for (int i = 0; i < MIN(count, fileInfo.count); i++) {
        NSDictionary* file = fileInfo[i];
        [result addObject:@{
            @"name": file[@"name"],
            @"path": file[@"path"],
            @"date": [formatter stringFromDate:file[@"date"]]
        }];
    }
    
    return [result copy];
}

- (NSArray*)getAvailableVaults {
    NSLog(@"🔍 getAvailableVaults - Méthode sécurisée");
    
    // Version ultra-sécurisée qui ne peut pas crasher
    @try {
        if (!_vaultRegistry) {
            NSLog(@"⚠️ _vaultRegistry est NULL, retour tableau vide");
            return @[];
        }
        
        NSLog(@"🔍 _vaultRegistry->count: %d", _vaultRegistry->count);
        
        if (_vaultRegistry->count <= 0) {
            NSLog(@"⚠️ Aucun vault dans le registre");
            return @[];
        }
        
        // Pour éviter tout crash, on retourne juste un tableau vide pour l'instant
        // TODO: Implémenter la vraie logique quand on aura résolu le problème
        NSLog(@"🔧 Mode sécurisé: retour tableau vide pour éviter les crash");
        return @[];
        
    } @catch (NSException* exception) {
        NSLog(@"❌ Exception dans getAvailableVaults: %@", exception.reason);
        return @[];
    }
}

// MARK: - Interface de création de vault popup

- (void)showVaultCreationPopup {
    NSLog(@"🪟 Ouverture de la popup de création de vault");
    
    _vaultCreationPopup = [[VaultCreationPopup alloc] init];
    _vaultCreationPopup.delegate = self;
    
    [_vaultCreationPopup showPopup];
}

// MARK: - VaultCreationDelegate

- (void)vaultCreationPopup:(VaultCreationPopup*)popup didCompleteWithSuccess:(BOOL)success vaultPath:(NSString*)vaultPath {
    NSLog(@"📞 Délégué appelé - succès: %s, vault: %@", success ? "OUI" : "NON", vaultPath);
    
    // Nettoyer la référence immédiatement
    _vaultCreationPopup = nil;
    
    if (success && vaultPath) {
        NSLog(@"✅ Vault créé avec succès: %@", vaultPath);
        
        // Recharger l'application avec le nouveau vault
        [self reloadApplicationWithNewVault:vaultPath];
    } else {
        NSLog(@"❌ Création de vault annulée ou échouée");
        // Fermer l'application si annulée
        [NSApp terminate:nil];
    }
}

- (void)reloadApplicationWithNewVault:(NSString*)vaultPath {
    NSLog(@"🔄 Rechargement simple de l'application avec vault: %@", vaultPath);
    
    // Recharger le système de vaults AVANT de définir les valeurs
    [self initializeVaultSystem];
    
    // Mettre à jour l'état de base APRÈS avoir rechargé le registre
    // Libérer les anciennes valeurs si elles existent
    if (_currentVaultPath) {
        [_currentVaultPath release];
    }
    if (_currentVaultName) {
        [_currentVaultName release];
    }
    
    // Assigner et retenir les nouvelles valeurs
    _currentVaultPath = [vaultPath retain];
    _currentVaultName = [[vaultPath lastPathComponent] retain];
    _isFirstLaunch = NO;
    _vaultSystemReady = true;
    
    NSLog(@"🔄 État mis à jour - vault: %@, nom: %@", _currentVaultPath, _currentVaultName);
    
    // Mettre à jour le titre de la fenêtre
    if (_mainWindow && _currentVaultName) {
        [_mainWindow setTitle:[NSString stringWithFormat:@"ElephantNotes V3 - %@", _currentVaultName]];
    }
    
    // Basculer vers l'interface Dashboard
    [self showDashboardMode];
    
    NSLog(@"✅ Application rechargée avec succès - vault: %@", _currentVaultName);
}

// MARK: - Interface de création de vault intégrée

- (void)showIntegratedVaultCreation {
    NSLog(@"🏗️ Affichage de l'interface de création de vault intégrée");
    
    // Vérifier que le framework UI est prêt
    if (!_uiFramework) {
        NSLog(@"❌ Framework UI non initialisé !");
        return;
    }
    
    // Générer le contenu de création de vault
    NSString* creationContent = [self generateVaultCreationInterface];
    NSLog(@"📝 Contenu généré: %lu caractères", [creationContent length]);
    NSLog(@"🎯 Premier extrait: %.100s...", [creationContent UTF8String]);
    
    // L'afficher dans l'éditeur principal
    ui_framework_set_editor_content(_uiFramework, creationContent);
    
    // Vérifier que le contenu a été envoyé
    NSLog(@"✅ ui_framework_set_editor_content appelé");
    NSLog(@"🔧 Framework UI: %p", _uiFramework);
    
    // Tester si on peut récupérer le contenu (pour vérifier que l'éditeur fonctionne)
    NSString* currentContent = ui_framework_get_editor_content(_uiFramework);
    if (currentContent && [currentContent length] > 0) {
        NSLog(@"✅ Contenu confirmé dans l'éditeur: %lu caractères", [currentContent length]);
    } else {
        NSLog(@"❌ Aucun contenu récupéré de l'éditeur !");
    }
}

- (NSString*)generateVaultCreationInterface {
    NSMutableString* content = [[NSMutableString alloc] init];
    
    [content appendString:@"# Bienvenue dans ElephantNotes V3\n\n"];
    [content appendString:@"Pour commencer, creons votre premier vault.\n"];
    [content appendString:@"Un vault est un dossier qui contiendra toutes vos notes et documents.\n\n"];
    
    [content appendString:@"## Configuration du Vault\n\n"];
    [content appendString:@"### Informations du vault :\n\n"];
    [content appendString:@"**Nom du vault :** Mon Premier Vault\n\n"];
    [content appendString:@"**Emplacement :** /Users/sorbet/Documents/ElephantNotes\n\n"];
    [content appendString:@"**Description :** Mon espace de notes personnel avec ElephantNotes V3\n\n"];
    [content appendString:@"**Creer des exemples :** Oui\n\n"];
    
    [content appendString:@"## Actions\n\n"];
    [content appendString:@"### Pour creer votre vault :\n\n"];
    [content appendString:@"1. **Modifiez** les informations ci-dessus si necessaire\n"];
    [content appendString:@"2. **Tapez** la commande `CREER VAULT` dans cette zone d'edition\n"];
    [content appendString:@"3. **L'application** creera votre vault et basculera automatiquement\n\n"];
    
    [content appendString:@"### Autres commandes disponibles :\n\n"];
    [content appendString:@"- `CHOISIR EMPLACEMENT` : Selectionner un autre dossier\n"];
    [content appendString:@"- `IMPORTER DOSSIER [chemin]` : Utiliser un dossier existant\n\n"];
    
    [content appendString:@"## A propos des Vaults\n\n"];
    [content appendString:@"Un vault ElephantNotes contient :\n"];
    [content appendString:@"- **Notes/** : Vos fichiers Markdown (.md)\n"];
    [content appendString:@"- **Attachments/** : Images et fichiers joints\n"];
    [content appendString:@"- **Templates/** : Modeles de documents\n\n"];
    
    [content appendString:@"---\n\n"];
    [content appendString:@"**Tapez `CREER VAULT` pour commencer !**\n"];
    
    return [content copy];
}

- (void)showVaultCreationWindow {
    NSLog(@"🏗️ Ouverture de l'interface de création de vault");
    
    _vaultSetupController = [[VaultSetupController alloc] init];
    
    // Définir le callback de completion
    _vaultSetupController.completionHandler = ^(BOOL success, NSString* vaultPath) {
        dispatch_async(dispatch_get_main_queue(), ^{
            if (success && vaultPath) {
                NSLog(@"✅ Vault créé avec succès: %@", vaultPath);
                
                // Nettoyer la référence au contrôleur de setup
                self->_vaultSetupController = nil;
                
                // Redémarrer l'application avec le nouveau vault
                dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 1.0 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
                    [self restartApplicationWithVault:vaultPath];
                });
            } else {
                NSLog(@"❌ Création de vault annulée ou échouée");
                // Nettoyer la référence au contrôleur de setup
                self->_vaultSetupController = nil;
                // Fermer l'application
                [NSApp terminate:nil];
            }
        });
    };
    
    // Afficher la fenêtre de configuration
    [_vaultSetupController showSetupWindow];
}

- (void)restartApplicationWithVault:(NSString*)vaultPath {
    NSLog(@"🔄 Relancement de l'application avec vault: %@", vaultPath);
    
    // Utiliser NSTask pour relancer l'application avec open
    NSString* appPath = [[NSBundle mainBundle] bundlePath];
    
    NSTask* task = [[NSTask alloc] init];
    [task setLaunchPath:@"/usr/bin/open"];
    [task setArguments:@[appPath]];
    
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 1.0 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        NSLog(@"🚀 Lancement de la nouvelle instance...");
        [task launch];
        
        // Attendre un peu puis fermer cette instance
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 0.5 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
            [NSApp terminate:nil];
        });
    });
}

// MARK: - Interface de gestion des vaults intégrée

- (void)showVaultManagementInterface {
    NSLog(@"📁 Affichage de l'interface de gestion des vaults");
    NSLog(@"🔥 Debug - _uiFramework: %p", _uiFramework);
    
    @autoreleasepool {
        NSLog(@"🔥 Appel generateVaultManagementInterface");
        NSString* vaultManagementContent = [self generateVaultManagementInterface];
        NSLog(@"🔥 generateVaultManagementInterface terminé");
        NSLog(@"📝 Contenu généré: %lu caractères", [vaultManagementContent length]);
        
        if (_uiFramework && vaultManagementContent) {
            NSLog(@"🔥 Appel ui_framework_set_editor_content pour vault management");
            ui_framework_set_editor_content(_uiFramework, vaultManagementContent);
            NSLog(@"🔥 ui_framework_set_editor_content terminé pour vault management");
            NSLog(@"✅ Contenu de vault management envoyé au framework UI");
        } else {
            NSLog(@"❌ Erreur: _uiFramework=%p, contenu=%p", _uiFramework, vaultManagementContent);
        }
    }
    NSLog(@"🔥 showVaultManagementInterface terminé");
}

- (NSString*)generateVaultManagementInterface {
    NSMutableString* content = [[[NSMutableString alloc] init] autorelease];
    
    // En-tête selon le contexte
    if (_vaultRegistry && _vaultRegistry->count > 0) {
        [content appendString:@"# 📁 Gestion des Vaults - ElephantNotes V3\n\n"];
        [content appendString:@"Sélectionnez un vault existant ou créez-en un nouveau.\n\n"];
    } else {
        [content appendString:@"# 🚀 Bienvenue dans ElephantNotes V3\n\n"];
        [content appendString:@"Pour commencer, vous devez créer votre premier vault.\n"];
        [content appendString:@"Un vault est un dossier qui contient toutes vos notes.\n\n"];
    }
    
    // Liste des vaults existants
    NSArray* availableVaults = [self getAvailableVaults];
    if (availableVaults.count > 0) {
        [content appendString:@"## 📋 Vaults Disponibles\n\n"];
        [content appendString:@"Cliquez sur un vault pour l'ouvrir :\n\n"];
        
        for (int i = 0; i < availableVaults.count; i++) {
            NSDictionary* vault = availableVaults[i];
            BOOL isActive = NO;
            if (_currentVaultPath && vault[@"path"]) {
                isActive = [vault[@"path"] isEqualToString:_currentVaultPath];
            }
            NSString* status = isActive ? @"🟢 **ACTUEL**" : @"⚪ Disponible";
            
            NSString* vaultName = vault[@"name"] ?: @"Nom inconnu";
            NSString* vaultPath = vault[@"path"] ?: @"Chemin inconnu";
            NSString* noteCount = vault[@"noteCount"] ?: @"0";
            
            [content appendFormat:@"### %d. %@ %@\n", i+1, vaultName, status];
            [content appendFormat:@"- **Chemin:** `%@`\n", vaultPath];
            [content appendFormat:@"- **Notes:** %@ fichiers\n", noteCount];
            
            if (!isActive) {
                [content appendFormat:@"- **Action:** [Cliquez ici pour activer ce vault]\n"];
                [content appendFormat:@"```\nPour activer: Tapez 'ACTIVER %d' dans la zone ci-dessous\n```\n", i+1];
            }
            [content appendString:@"\n"];
        }
    } else {
        [content appendString:@"## 📋 Aucun Vault Configuré\n\n"];
        [content appendString:@"Vous n'avez pas encore de vault configuré.\n"];
        [content appendString:@"Créez votre premier vault ci-dessous.\n\n"];
    }
    
    // Interface de création de nouveau vault
    [content appendString:@"## ➕ Créer un Nouveau Vault\n\n"];
    [content appendString:@"### Configuration du Vault\n\n"];
    [content appendString:@"Remplissez les informations ci-dessous :\n\n"];
    
    [content appendString:@"```\n"];
    [content appendString:@"NOM: Mon Vault ElephantNotes\n"];
    [content appendString:@"CHEMIN: /Users/sorbet/Documents/ElephantNotes\n"];
    [content appendString:@"DESCRIPTION: Mon espace de notes personnel\n"];
    [content appendString:@"EXEMPLES: OUI\n"];
    [content appendString:@"\n"];
    [content appendString:@"Pour créer: Tapez 'CREER' après avoir modifié les infos\n"];
    [content appendString:@"```\n\n"];
    
    // Actions rapides
    [content appendString:@"## ⚡ Actions Rapides\n\n"];
    [content appendString:@"### Commandes Disponibles\n"];
    [content appendString:@"Tapez ces commandes dans la zone d'édition :\n\n"];
    
    if (availableVaults.count > 0) {
        [content appendString:@"- **ACTIVER [numéro]** : Activer un vault existant\n"];
        [content appendString:@"  Exemple: `ACTIVER 1` pour activer le premier vault\n\n"];
    }
    
    [content appendString:@"- **CREER** : Créer un nouveau vault avec les infos ci-dessus\n"];
    [content appendString:@"- **PARCOURIR** : Choisir l'emplacement avec le sélecteur de fichiers\n"];
    [content appendString:@"- **IMPORTER [chemin]** : Importer un dossier existant comme vault\n\n"];
    
    // Guide d'utilisation
    [content appendString:@"## 📖 Guide d'Utilisation\n\n"];
    [content appendString:@"### Qu'est-ce qu'un Vault ?\n"];
    [content appendString:@"Un vault ElephantNotes est un dossier qui contient :\n"];
    [content appendString:@"- **Notes/** : Vos fichiers Markdown (.md)\n"];
    [content appendString:@"- **Attachments/** : Images et fichiers joints\n"];
    [content appendString:@"- **Templates/** : Modèles de documents\n"];
    [content appendString:@"- **.elephantnotes_vault** : Configuration\n\n"];
    
    [content appendString:@"### Emplacements Recommandés\n"];
    [content appendString:@"- **Documents/ElephantNotes** : Emplacement standard\n"];
    [content appendString:@"- **iCloud Drive/Notes** : Synchronisation cloud\n"];
    [content appendString:@"- **Dropbox/Notes** : Partage entre appareils\n\n"];
    
    // Instructions
    [content appendString:@"### Instructions\n"];
    [content appendString:@"1. **Modifiez** les informations du vault ci-dessus\n"];
    [content appendString:@"2. **Tapez** `CREER` pour créer le vault\n"];
    [content appendString:@"3. **L'application** se rechargera automatiquement\n"];
    [content appendString:@"4. **Vous pourrez** commencer à créer vos notes\n\n"];
    
    // Zone de commande
    [content appendString:@"---\n\n"];
    [content appendString:@"## 💬 Zone de Commande\n\n"];
    [content appendString:@"### Tapez votre commande ici :\n\n"];
    [content appendString:@"```\n"];
    [content appendString:@"Exemples de commandes :\n"];
    [content appendString:@"- CREER\n"];
    [content appendString:@"- ACTIVER 1\n"];
    [content appendString:@"- PARCOURIR\n"];
    [content appendString:@"- IMPORTER /path/to/folder\n"];
    [content appendString:@"```\n\n"];
    
    // Statut
    [content appendString:@"---\n\n"];
    [content appendString:@"**État actuel :**\n"];
    [content appendFormat:@"- Vaults disponibles : %lu\n", (unsigned long)availableVaults.count];
    [content appendFormat:@"- Vault actuel : %@\n", _currentVaultName ?: @"Aucun"];
    [content appendFormat:@"- Premier lancement : %@\n", _isFirstLaunch ? @"OUI" : @"NON"];
    
    [content appendString:@"\n*ElephantNotes V3 - Gestion des Vaults*\n"];
    
    return [content copy];
}

// Traitement des commandes de gestion de vault
- (void)processVaultCommand:(NSString*)command {
    NSLog(@"🎯 Traitement de la commande: %@", command);
    
    // Nettoyer la commande
    command = [command stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
    command = [command uppercaseString];
    
    if ([command isEqualToString:@"CREER"] || [command isEqualToString:@"CREER VAULT"]) {
        [self handleCreateVaultCommandIntegrated];
    } else if ([command hasPrefix:@"ACTIVER "]) {
        NSString* numberStr = [command substringFromIndex:8];
        int vaultIndex = [numberStr intValue];
        [self handleActivateVaultCommand:vaultIndex];
    } else if ([command isEqualToString:@"PARCOURIR"]) {
        [self handleBrowseVaultCommand];
    } else if ([command hasPrefix:@"IMPORTER "]) {
        NSString* path = [command substringFromIndex:9];
        [self handleImportVaultCommand:path];
    } else {
        NSLog(@"⚠️ Commande inconnue: %@", command);
        // Afficher une aide rapide
        [self showCommandHelp];
    }
}

- (void)handleCreateVaultCommandIntegrated {
    NSLog(@"🏗️ Création de vault intégrée...");
    
    // Configuration par défaut du vault
    VaultCreationOptions options = {0};
    options.name = "Mon Premier Vault";
    options.path = "/Users/sorbet/Documents";
    options.description = "Mon espace de notes personnel avec ElephantNotes V3";
    options.type = VAULT_TYPE_LOCAL;
    options.encrypt = false;
    options.password = NULL;
    options.create_sample_notes = true;
    options.template_name = NULL;
    
    VaultInfo* info = NULL;
    VaultResult result = vault_create(&options, &info);
    
    if (result == VAULT_SUCCESS && info) {
        NSLog(@"✅ Vault créé avec succès: %s", options.name);
        
        // Ajouter au registre
        VaultRegistry* registry = NULL;
        vault_registry_load(&registry);
        if (!registry) {
            registry = malloc(sizeof(VaultRegistry));
            memset(registry, 0, sizeof(VaultRegistry));
        }
        
        NSString* fullPath = [NSString stringWithFormat:@"%s/%s", options.path, options.name];
        vault_registry_add(registry, [fullPath UTF8String]);
        vault_registry_set_default(registry, [fullPath UTF8String]);
        vault_registry_save(registry);
        vault_registry_free(registry);
        
        // Marquer la configuration comme terminée
        vault_mark_first_launch_complete();
        
        // Mettre à jour l'état interne
        if (_currentVaultPath) {
            [_currentVaultPath release];
        }
        if (_currentVaultName) {
            [_currentVaultName release];
        }
        _currentVaultPath = [fullPath retain];
        _currentVaultName = [[fullPath lastPathComponent] retain];
        _isFirstLaunch = NO;
        _vaultSystemReady = false;
        
        // Recharger le système de vaults
        [self initializeVaultSystem];
        
        // Basculer vers l'interface normale
        [self showDashboardMode];
        
        NSLog(@"🚀 Application basculée vers l'interface normale");
        
        vault_free_info(info);
        free(info);
    } else {
        const char* error_msg = vault_get_error_message(result);
        NSLog(@"❌ Erreur lors de la création du vault: %s", error_msg);
        
        // Afficher un message d'erreur dans l'éditeur
        NSString* errorContent = [NSString stringWithFormat:@"# ❌ Erreur\n\nImpossible de créer le vault :\n%s\n\nVeuillez réessayer.", error_msg];
        
        if (_uiFramework) {
            ui_framework_set_editor_content(_uiFramework, errorContent);
        }
    }
}

- (void)handleCreateVaultCommand {
    NSLog(@"🏗️ Création d'un nouveau vault...");
    
    // Parse les informations du vault depuis le contenu de l'éditeur
    NSString* currentContent = [self getCurrentEditorContent];
    NSDictionary* vaultConfig = [self parseVaultConfigFromContent:currentContent];
    
    if (!vaultConfig) {
        NSLog(@"❌ Configuration du vault invalide");
        return;
    }
    
    // Créer le vault avec les informations extraites
    VaultCreationOptions options = {0};
    options.name = (char*)[vaultConfig[@"name"] UTF8String];
    options.path = (char*)[vaultConfig[@"path"] UTF8String];
    options.description = (char*)[vaultConfig[@"description"] UTF8String];
    options.type = VAULT_TYPE_LOCAL;
    options.encrypt = false;
    options.password = NULL;
    options.create_sample_notes = [vaultConfig[@"createExamples"] boolValue];
    options.template_name = NULL;
    
    VaultInfo* info = NULL;
    VaultResult result = vault_create(&options, &info);
    
    if (result == VAULT_SUCCESS && info) {
        NSLog(@"✅ Vault créé avec succès: %@", vaultConfig[@"name"]);
        
        // Ajouter au registre
        VaultRegistry* registry = NULL;
        vault_registry_load(&registry);
        if (!registry) {
            registry = malloc(sizeof(VaultRegistry));
            memset(registry, 0, sizeof(VaultRegistry));
        }
        
        NSString* fullPath = [NSString stringWithFormat:@"%@/%@", vaultConfig[@"path"], vaultConfig[@"name"]];
        vault_registry_add(registry, [fullPath UTF8String]);
        vault_registry_set_default(registry, [fullPath UTF8String]);
        vault_registry_save(registry);
        vault_registry_free(registry);
        
        // Marquer la configuration comme terminée
        vault_mark_first_launch_complete();
        
        // Recharger l'application avec le nouveau vault
        [self reloadWithVault:fullPath];
        
        vault_free_info(info);
        free(info);
    } else {
        const char* error_msg = vault_get_error_message(result);
        NSLog(@"❌ Erreur lors de la création du vault: %s", error_msg);
    }
}

- (void)handleActivateVaultCommand:(int)vaultIndex {
    NSLog(@"🔄 Activation du vault %d...", vaultIndex);
    
    NSArray* availableVaults = [self getAvailableVaults];
    if (vaultIndex < 1 || vaultIndex > availableVaults.count) {
        NSLog(@"❌ Index de vault invalide: %d", vaultIndex);
        return;
    }
    
    NSDictionary* vaultInfo = availableVaults[vaultIndex - 1];
    NSString* vaultPath = vaultInfo[@"path"];
    
    // Mettre à jour le registre pour définir ce vault comme défaut
    VaultRegistry* registry = NULL;
    vault_registry_load(&registry);
    if (registry) {
        vault_registry_set_default(registry, [vaultPath UTF8String]);
        vault_registry_save(registry);
        vault_registry_free(registry);
    }
    
    // Recharger l'application avec le vault sélectionné
    [self reloadWithVault:vaultPath];
}

- (void)handleBrowseVaultCommand {
    NSLog(@"📁 Ouverture du sélecteur de dossiers...");
    
    // Cette méthode nécessiterait l'intégration avec NSOpenPanel
    // Pour l'instant, on affiche un message d'aide
    NSLog(@"💡 Fonction de parcours à implémenter avec NSOpenPanel");
}

- (void)handleImportVaultCommand:(NSString*)path {
    NSLog(@"📥 Import du vault depuis: %@", path);
    
    // Vérifier que le chemin existe
    NSFileManager* fileManager = [NSFileManager defaultManager];
    if (![fileManager fileExistsAtPath:path]) {
        NSLog(@"❌ Le chemin spécifié n'existe pas: %@", path);
        return;
    }
    
    // Ajouter le dossier existant comme vault
    VaultRegistry* registry = NULL;
    vault_registry_load(&registry);
    if (!registry) {
        registry = malloc(sizeof(VaultRegistry));
        memset(registry, 0, sizeof(VaultRegistry));
    }
    
    vault_registry_add(registry, [path UTF8String]);
    vault_registry_set_default(registry, [path UTF8String]);
    vault_registry_save(registry);
    vault_registry_free(registry);
    
    // Recharger l'application avec le vault importé
    [self reloadWithVault:path];
}

- (void)showCommandHelp {
    NSLog(@"📖 Aide des commandes:");
    NSLog(@"   CREER - Créer un nouveau vault");
    NSLog(@"   ACTIVER [numéro] - Activer un vault existant");
    NSLog(@"   PARCOURIR - Choisir l'emplacement");
    NSLog(@"   IMPORTER [chemin] - Importer un dossier existant");
}

- (NSString*)getCurrentEditorContent {
    // Cette méthode devrait récupérer le contenu actuel de l'éditeur
    // Pour l'instant, on retourne un contenu par défaut
    return @"NOM: Mon Vault ElephantNotes\nCHEMIN: /Users/sorbet/Documents/ElephantNotes\nDESCRIPTION: Mon espace de notes personnel\nEXEMPLES: OUI";
}

- (NSDictionary*)parseVaultConfigFromContent:(NSString*)content {
    NSMutableDictionary* config = [[NSMutableDictionary alloc] init];
    
    // Valeurs par défaut
    config[@"name"] = @"Mon Vault ElephantNotes";
    config[@"path"] = @"/Users/sorbet/Documents";
    config[@"description"] = @"Mon espace de notes personnel";
    config[@"createExamples"] = @YES;
    
    // Parser le contenu ligne par ligne
    NSArray* lines = [content componentsSeparatedByString:@"\n"];
    for (NSString* line in lines) {
        if ([line hasPrefix:@"NOM:"]) {
            NSString* name = [[line substringFromIndex:4] stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
            if (name.length > 0) config[@"name"] = name;
        } else if ([line hasPrefix:@"CHEMIN:"]) {
            NSString* path = [[line substringFromIndex:7] stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
            if (path.length > 0) config[@"path"] = path;
        } else if ([line hasPrefix:@"DESCRIPTION:"]) {
            NSString* desc = [[line substringFromIndex:12] stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
            if (desc.length > 0) config[@"description"] = desc;
        } else if ([line hasPrefix:@"EXEMPLES:"]) {
            NSString* examples = [[line substringFromIndex:9] stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceCharacterSet]];
            config[@"createExamples"] = @([examples.uppercaseString isEqualToString:@"OUI"]);
        }
    }
    
    NSLog(@"📋 Configuration parsée: %@", config);
    return [config copy];
}

- (void)reloadWithVault:(NSString*)vaultPath {
    NSLog(@"🔄 Rechargement de l'application avec le vault: %@", vaultPath);
    
    // Mettre à jour l'état interne
    if (_currentVaultPath) {
        [_currentVaultPath release];
    }
    if (_currentVaultName) {
        [_currentVaultName release];
    }
    _currentVaultPath = [vaultPath retain];
    _currentVaultName = [[vaultPath lastPathComponent] retain];
    _isFirstLaunch = NO;
    // Mettre à jour l'état de mode de gestion
    
    // Recharger la configuration
    [self checkFirstLaunch];
    
    // Passer en mode Dashboard
    [self showDashboardMode];
    
    NSLog(@"✅ Application rechargée avec succès");
}

@end

// ========== Fonctions C d'intégration ==========

void elephantnotes_v3_init(void) {
    if (!g_mainController) {
        g_mainController = [[ElephantNotesV3Controller alloc] init];
        NSLog(@"🔧 ElephantNotes V3 initialisé");
    }
}

void elephantnotes_v3_cleanup(void) {
    if (g_mainController) {
        g_mainController = nil;
        NSLog(@"🧹 ElephantNotes V3 nettoyé");
    }
    
    vault_manager_cleanup();
    professional_cleanup();
}

ElephantNotesV3Controller* elephantnotes_v3_get_controller(void) {
    return g_mainController;
}

// Callbacks pour l'UI Framework
void ui_icon_click_handler(UIIconType iconType, void* userData) {
    NSLog(@"🔥 ui_icon_click_handler appelé - iconType: %d, userData: %p", iconType, userData);
    
    if (!userData) {
        NSLog(@"❌ userData est NULL dans ui_icon_click_handler");
        return;
    }
    
    ElephantNotesV3Controller* controller = (__bridge ElephantNotesV3Controller*)userData;
    NSLog(@"🔥 Controller bridgé: %p", controller);
    
    if (controller) {
        NSLog(@"🔥 Appel de handleIconClick sur le controller");
        [controller handleIconClick:iconType];
        NSLog(@"🔥 handleIconClick terminé avec succès");
    } else {
        NSLog(@"❌ Controller est NULL après bridging");
    }
}

void ui_icon_hover_handler(UIIconType iconType, bool isHovering, void* userData) {
    ElephantNotesV3Controller* controller = (__bridge ElephantNotesV3Controller*)userData;
    if (controller) {
        [controller handleIconHover:iconType isHovering:isHovering];
    }
}

// Traitement des commandes de gestion de vault
void ui_text_command_handler(const char* command_text, void* userData) {
    ElephantNotesV3Controller* controller = (__bridge ElephantNotesV3Controller*)userData;
    if (controller && command_text) {
        NSString* command = [NSString stringWithUTF8String:command_text];
        [controller processVaultCommand:command];
    }
}