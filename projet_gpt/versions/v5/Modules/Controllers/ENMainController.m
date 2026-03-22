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
#import "../Tabs/ENTablesTab.h"

@implementation ENMainController {
    NSMutableArray<NSNumber*>* _navigationHistory;
    BOOL _handlingBackNavigation;
    UIIconType _currentIcon;
}

- (instancetype)init {
    self = [super init];
    if (self) {
        _tabs = [[NSMutableDictionary alloc] init];
        _navigationHistory = [[NSMutableArray alloc] init];
        _handlingBackNavigation = NO;
        _currentIcon = UI_ICON_HOME;
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

    [[NSNotificationCenter defaultCenter] addObserver:self
                                             selector:@selector(handleMarkdownNoteLink:)
                                                 name:@"ENMarkdownNoteLinkClicked"
                                               object:nil];
    
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
    [self updateBackIconState];
    
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

    ENEditorTab* editorTab = [[ENEditorTab alloc] init];
    [self registerTab:editorTab forIconType:UI_ICON_EDITOR];
    [editorTab release];

    ENTablesTab* tablesTab = [[ENTablesTab alloc] init];
    [self registerTab:tablesTab forIconType:UI_ICON_TABLES];
    [tablesTab release];

    ENSettingsTab* settingsTab = [[ENSettingsTab alloc] init];
    [self registerTab:settingsTab forIconType:UI_ICON_SETTINGS];
    [settingsTab release];
    
    // Activer l'onglet par défaut (Dashboard)
    [self switchToTab:UI_ICON_HOME];
    [self updateBackIconState];
    
    NSLog(@"✅ [ENMainController] Onglets configurés : Dashboard → Search → Editor → Tables → Settings");
}

- (void)updateBackIconState {
    if (!_uiFramework) {
        return;
    }
    bool hasHistory = (_navigationHistory && [_navigationHistory count] > 0);
    ui_framework_set_icon_enabled(_uiFramework, UI_ICON_BACK, hasHistory);
}

- (void)navigateBack {
    if (!_navigationHistory || [_navigationHistory count] == 0) {
        NSLog(@"⚠️ [ENMainController] Aucun historique de navigation");
        [_sidebar setActiveIcon:_currentIcon];
        return;
    }
    UIIconType previousIcon = (UIIconType)[[_navigationHistory lastObject] integerValue];
    [_navigationHistory removeLastObject];
    _handlingBackNavigation = YES;
    [self switchToTab:previousIcon];
    _handlingBackNavigation = NO;
}

- (void)switchToTab:(UIIconType)iconType {
    if (iconType == UI_ICON_BACK) {
        [self navigateBack];
        return;
    }
    
    NSString* tabName = [self nameForIconType:iconType];
    ENTabBase* newTab = [self tabForIconType:iconType];
    
    if (newTab && _currentTab && (_currentIcon == iconType) && !_handlingBackNavigation) {
        [_sidebar setActiveIcon:iconType];
        return;
    }
    
    if (!_handlingBackNavigation && newTab && _currentTab && _currentIcon != iconType) {
        NSNumber* previousIconNumber = @(_currentIcon);
        if (![_navigationHistory count] || ![[_navigationHistory lastObject] isEqualToNumber:previousIconNumber]) {
            [_navigationHistory addObject:previousIconNumber];
            if ([_navigationHistory count] > 25) {
                [_navigationHistory removeObjectAtIndex:0];
            }
        }
    }
    
    NSLog(@"🔄 [ENMainController] Basculement vers : %@", tabName);
    
    if (_currentTab) {
        [_currentTab didBecomeInactive];
    }
    
    if (newTab) {
        _currentTab = newTab;
        _currentIcon = iconType;
        [_currentTab didBecomeActive];
        [_sidebar setActiveIcon:iconType];
        NSLog(@"✅ [ENMainController] Onglet actif : %@", tabName);
    } else {
        NSLog(@"⚠️ [ENMainController] Onglet non trouvé : %@", tabName);
        
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
    
    [self updateBackIconState];
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
        case UI_ICON_TABLES: return @"Tables";
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
        case UI_ICON_TABLES: return @"📊";
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
    [[NSNotificationCenter defaultCenter] removeObserver:self];
    [_sidebar release];
    [_tabs release];
    [_navigationHistory release];
    [_currentVaultPath release];
    [_currentVaultName release];
    [super dealloc];
}

- (void)handleMarkdownNoteLink:(NSNotification*)notification {
    NSString* linkPath = notification.userInfo[@"path"];
    if (!linkPath || [linkPath length] == 0) {
        return;
    }

    if ([linkPath hasPrefix:@"www."]) {
        linkPath = [NSString stringWithFormat:@"https://%@", linkPath];
    }

    if ([linkPath containsString:@"://"] && ![linkPath hasPrefix:@"file://"]) {
        NSURL* url = [NSURL URLWithString:linkPath];
        if (url) {
            [[NSWorkspace sharedWorkspace] openURL:url];
        }
        return;
    }

    ENEditorTab* editorTab = (ENEditorTab*)[self tabForIconType:UI_ICON_EDITOR];
    if (![editorTab isKindOfClass:[ENEditorTab class]]) {
        NSLog(@"⚠️ [ENMainController] Impossible d'ouvrir le lien %@ : onglet éditeur indisponible", linkPath);
        return;
    }

    BOOL opened = [editorTab openNoteLink:linkPath];
    if (opened) {
        [self switchToTab:UI_ICON_EDITOR];
    } else {
        NSBeep();
        NSLog(@"⚠️ [ENMainController] Note introuvable pour le lien %@", linkPath);
    }
}

@end
