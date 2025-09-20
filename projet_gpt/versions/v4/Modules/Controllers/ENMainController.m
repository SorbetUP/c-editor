//
//  ENMainController.m
//  ElephantNotes V4 - Contrôleur principal de l'application
//

#import "ENMainController.h"
#import "../Tabs/ENDashboardTab.h"
#import "../Tabs/ENEditorTab.h"
#import "../Tabs/ENFilesTab.h"
#import "../Tabs/ENSearchTab.h"
#import "../Tabs/ENToolsTab.h"
#import "../Tabs/ENSettingsTab.h"

@implementation ENMainController

- (instancetype)init {
    self = [super init];
    if (self) {
        _tabs = [[NSMutableDictionary alloc] init];
        _isReady = false;
        NSLog(@"🔧 [ENMainController] Contrôleur principal initialisé");
    }
    return self;
}

- (void)setupWithUIFramework:(UIFramework*)framework {
    if (!framework) {
        NSLog(@"❌ [ENMainController] UIFramework NULL");
        return;
    }
    
    _uiFramework = framework;
    NSLog(@"🖥️ [ENMainController] UIFramework configuré");
    
    // Initialiser les composants dans l'ordre
    [self initializeVaultSystem];
    [self setupSidebar];
    [self setupTabs];
    
    _isReady = true;
    NSLog(@"✅ [ENMainController] Configuration terminée");
}

- (void)initializeVaultSystem {
    NSLog(@"📁 [ENMainController] Initialisation du système de vaults");
    
    // Initialiser le gestionnaire de vaults
    VaultResult result = vault_manager_init();
    if (result != VAULT_SUCCESS) {
        NSLog(@"❌ [ENMainController] Erreur d'initialisation des vaults: %d", result);
        return;
    }
    
    VaultResult registryResult = vault_registry_load(&_vaultRegistry);
    if (registryResult != VAULT_SUCCESS) {
        NSLog(@"❌ [ENMainController] Impossible de charger le registre des vaults: %d", registryResult);
        return;
    }
    
    // Charger le vault par défaut s'il existe
    if (_vaultRegistry->default_vault_path) {
        [self loadVault:[NSString stringWithUTF8String:_vaultRegistry->default_vault_path]];
    } else {
        NSLog(@"⚠️ [ENMainController] Aucun vault par défaut configuré");
    }
    
    NSLog(@"✅ [ENMainController] Système de vaults initialisé");
}

- (void)setupSidebar {
    NSLog(@"🎮 [ENMainController] Configuration de la sidebar");
    
    _sidebar = [[ENSidebar alloc] initWithUIFramework:_uiFramework];
    _sidebar.delegate = self;
    [_sidebar setup];
    
    NSLog(@"✅ [ENMainController] Sidebar configurée");
}

- (void)setupTabs {
    NSLog(@"📑 [ENMainController] Configuration des onglets");
    
    // Ordre optimisé pour un workflow naturel :
    // 1. Dashboard - Vue d'ensemble
    // 2. Search - Accès direct à la recherche
    // 3. Settings - Paramétrage
    
    ENDashboardTab* dashboardTab = [[ENDashboardTab alloc] init];
    [self registerTab:dashboardTab forIconType:UI_ICON_HOME];
    [dashboardTab release];
    
    ENSearchTab* searchTab = [[ENSearchTab alloc] init];
    [self registerTab:searchTab forIconType:UI_ICON_SEARCH];
    [searchTab release];
    
    ENSettingsTab* settingsTab = [[ENSettingsTab alloc] init];
    [self registerTab:settingsTab forIconType:UI_ICON_SETTINGS];
    [settingsTab release];
    
    // Activer l'onglet par défaut (Dashboard)
    [self switchToTab:UI_ICON_HOME];
    
    NSLog(@"✅ [ENMainController] Onglets configurés : Dashboard → Search → Settings");
}

- (void)switchToTab:(UIIconType)iconType {
    NSString* tabName = [self nameForIconType:iconType];
    NSLog(@"🔄 [ENMainController] Basculement vers : %@", tabName);
    
    // Désactiver l'onglet actuel
    if (_currentTab) {
        [_currentTab didBecomeInactive];
    }
    
    // Activer le nouvel onglet
    ENTabBase* newTab = [self tabForIconType:iconType];
    if (newTab) {
        _currentTab = newTab;
        [_currentTab didBecomeActive];
        
        // Mettre à jour la sidebar
        [_sidebar setActiveIcon:iconType];
        
        NSLog(@"✅ [ENMainController] Onglet actif : %@", tabName);
    } else {
        NSLog(@"⚠️ [ENMainController] Onglet non trouvé : %@", tabName);
        
        // Contenu temporaire en attendant l'implémentation des onglets
        NSString* tempContent = [NSString stringWithFormat:
            @"# %@ %@\n\n"
            @"## ElephantNotes V4 - Architecture Modulaire\n\n"
            @"L'onglet **%@** sera bientôt disponible.\n\n"
            @"### 🏗️ Modules implémentés :\n"
            @"- ✅ ENTabBase (Classe de base)\n"
            @"- ✅ ENSidebar (Barre latérale)\n"
            @"- ✅ ENMainController (Contrôleur principal)\n"
            @"- 🔄 EN%@Tab (En cours...)\n\n"
            @"### 📁 Vault actuel :\n"
            @"- **Nom :** %@\n"
            @"- **Chemin :** `%@`\n\n"
            @"---\n\n"
            @"*Interface générée par ENMainController*",
            [self emojiForIconType:iconType], tabName, tabName, tabName,
            _currentVaultName ?: @"Aucun vault",
            _currentVaultPath ?: @"Non configuré"
        ];
        
        ui_framework_set_editor_content(_uiFramework, tempContent);
        [_sidebar setActiveIcon:iconType];
    }
}

- (void)registerTab:(ENTabBase*)tab forIconType:(UIIconType)iconType {
    if (!tab) {
        NSLog(@"❌ [ENMainController] Tentative d'enregistrement d'un onglet NULL");
        return;
    }
    
    // Configurer l'onglet
    tab.uiFramework = _uiFramework;
    tab.currentVaultPath = _currentVaultPath;
    tab.currentVaultName = _currentVaultName;
    tab.delegate = self;
    
    // Enregistrer l'onglet
    NSNumber* iconKey = @(iconType);
    [_tabs setObject:tab forKey:iconKey];
    
    NSLog(@"📝 [ENMainController] Onglet enregistré : %@", [self nameForIconType:iconType]);
}

- (ENTabBase*)tabForIconType:(UIIconType)iconType {
    NSNumber* iconKey = @(iconType);
    return [_tabs objectForKey:iconKey];
}

- (void)loadVault:(NSString*)vaultPath {
    if (!vaultPath) return;
    
    NSLog(@"📂 [ENMainController] Chargement du vault : %@", vaultPath);
    
    [_currentVaultPath release];
    _currentVaultPath = [vaultPath copy];
    
    [_currentVaultName release];
    _currentVaultName = [[vaultPath lastPathComponent] copy];
    
    [self updateTabsWithVaultInfo];
    
    NSLog(@"✅ [ENMainController] Vault chargé : %@", _currentVaultName);
}

- (void)updateTabsWithVaultInfo {
    // Mettre à jour tous les onglets avec les nouvelles informations de vault
    for (ENTabBase* tab in [_tabs allValues]) {
        tab.currentVaultPath = _currentVaultPath;
        tab.currentVaultName = _currentVaultName;
    }
    
    // Rafraîchir l'onglet actuel
    if (_currentTab) {
        [_currentTab refreshContent];
    }
}

- (NSString*)nameForIconType:(UIIconType)iconType {
    switch (iconType) {
        case UI_ICON_HOME: return @"Dashboard";
        case UI_ICON_FILES: return @"Files";
        case UI_ICON_EDITOR: return @"Editor";
        case UI_ICON_SEARCH: return @"Search";
        case UI_ICON_TOOLS: return @"Tools";
        case UI_ICON_SETTINGS: return @"Settings";
        case UI_ICON_BACK: return @"Back";
        default: return @"Unknown";
    }
}

- (NSString*)emojiForIconType:(UIIconType)iconType {
    switch (iconType) {
        case UI_ICON_HOME: return @"🏠";
        case UI_ICON_FILES: return @"📁";
        case UI_ICON_EDITOR: return @"📝";
        case UI_ICON_SEARCH: return @"🔍";
        case UI_ICON_TOOLS: return @"🔧";
        case UI_ICON_SETTINGS: return @"⚙️";
        case UI_ICON_BACK: return @"⬅️";
        default: return @"❓";
    }
}

#pragma mark - ENSidebarDelegate

- (void)sidebar:(id)sender didClickIcon:(UIIconType)iconType {
    NSLog(@"🎯 [ENMainController] Clic reçu de la sidebar : %@", [self nameForIconType:iconType]);
    [self switchToTab:iconType];
}

- (void)sidebar:(id)sender didHoverIcon:(UIIconType)iconType isHovering:(bool)isHovering {
    // Gestion des événements de survol si nécessaire
}

#pragma mark - ENTabDelegate

- (void)tabDidChange:(id)sender {
    NSLog(@"🔄 [ENMainController] Onglet modifié");
}

- (void)tabNeedsRefresh:(id)sender {
    NSLog(@"🔄 [ENMainController] Rafraîchissement demandé par un onglet");
    if ([sender isEqual:_currentTab]) {
        [_currentTab refreshContent];
    }
}

- (void)tabContentDidSave:(id)sender withContent:(NSString*)content {
    NSLog(@"💾 [ENMainController] Contenu sauvegardé par un onglet");
    
    // Transmettre l'événement de sauvegarde à l'onglet concerné
    if ([sender isKindOfClass:[ENTabBase class]]) {
        ENTabBase* tab = (ENTabBase*)sender;
        [tab handleContentSave:content];
    }
}

- (void)dealloc {
    [_sidebar release];
    [_tabs release];
    [_currentVaultPath release];
    [_currentVaultName release];
    [super dealloc];
}

@end