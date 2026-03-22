// ElephantNotesV3.h - ElephantNotes Version 3 avec interface intégrée
// Intègre le système de vaults + barre latérale + éditeur professionnel

#import <Cocoa/Cocoa.h>
#include "../ui_framework/ui_framework.h"
#include "../vault_manager/vault_manager.h"
#include "../file_manager/professional_file_manager.h"
#include "../search_interface/search_interface.h"
#include "../advanced_search/advanced_search.h"
#import "VaultCreationPopup.h"

// Forward declarations
@class VaultSetupController;

@interface ElephantNotesV3AppDelegate : NSObject <NSApplicationDelegate>
@end

@interface ElephantNotesV3Window : NSWindow
@end

@interface ElephantNotesV3Controller : NSObject <VaultCreationDelegate> {
    // Composants principaux
    UIFramework* _uiFramework;
    ElephantNotesV3Window* _mainWindow;
    
    // Système de vaults
    VaultRegistry* _vaultRegistry;
    NSString* _currentVaultPath;
    NSString* _currentVaultName;
    bool _vaultSystemReady;
    
    // Système de recherche optimisé
    SearchEngine* _searchEngine;
    SearchInterface* _searchInterface;
    bool _searchIndexReady;
    
    // État de l'application
    NSString* _currentFilePath;
    NSString* _originalContent;
    bool _hasUnsavedChanges;
    bool _isFirstLaunch;
    
    // Composants de l'interface
    UIIconType _currentMode;
    
    // Timers pour les fonctionnalités professionnelles
    NSTimer* _autoSaveTimer;
    NSTimer* _conflictCheckTimer;
    
    // Historique de navigation
    NSMutableArray* _navigationHistory;
    NSInteger _currentHistoryIndex;
    
    // Contrôleur de création de vault
    VaultSetupController* _vaultSetupController;
    VaultCreationPopup* _vaultCreationPopup;
}

// Propriétés publiques
@property (nonatomic, readonly) UIFramework* uiFramework;
@property (nonatomic, readonly) NSString* currentVaultPath;
@property (nonatomic, readonly) NSString* currentVaultName;
@property (nonatomic, readonly) bool isReady;

// Initialisation
- (instancetype)init;
- (void)setupApplication;
- (void)initializeVaultSystem;
- (void)initializeUI;
- (void)showMainWindow;

// Gestion des vaults
- (void)checkFirstLaunch;
- (void)showVaultSetup;
- (void)loadDefaultVault;
- (void)switchToVault:(NSString*)vaultPath;
- (void)showVaultManager;

// Gestion de l'interface
- (void)handleIconClick:(UIIconType)iconType;
- (void)handleIconHover:(UIIconType)iconType isHovering:(bool)isHovering;
- (void)switchToMode:(UIIconType)mode;

// Modes d'interface
- (void)showSearchMode;
- (void)showBackMode;
- (void)showDashboardMode;
- (void)showDefaultDashboardContent;
- (void)showSettingsMode;
- (void)showDefaultSettingsContent;

// Système de recherche
- (void)initializeSearchSystem;
- (void)indexCurrentVault;
- (void)performSearch:(NSString*)query;
- (void)displaySearchResults:(NSArray*)results;

// Historique de navigation
- (void)addToNavigationHistory:(UIIconType)mode withContent:(NSString*)content;
- (void)navigateBack;
- (void)navigateForward;
- (BOOL)canNavigateBack;
- (BOOL)canNavigateForward;

// Gestion des fichiers
- (void)newFile;
- (void)openFile;
- (void)saveFile;
- (void)saveFileAs;
- (void)loadFileContent:(NSString*)filePath;

// Fonctionnalités professionnelles
- (void)enableAutoSave;
- (void)performAutoSave;
- (void)createVersionSnapshot;
- (void)showFileStatistics;

// Callbacks pour les contrôleurs de vaults
- (void)vaultSetupCompleted:(NSString*)vaultPath;
- (void)vaultChanged:(NSString*)newVaultPath;

// Gestion des commandes de vault
- (void)processVaultCommand:(NSString*)command;

// Interface de création de vault
- (void)showVaultCreationPopup;
- (void)reloadApplicationWithNewVault:(NSString*)vaultPath;
- (void)showIntegratedVaultCreation;
- (void)showVaultCreationWindow;
- (void)restartApplicationWithVault:(NSString*)vaultPath;

@end

// Fonctions C pour l'intégration
void elephantnotes_v3_init(void);
void elephantnotes_v3_cleanup(void);
ElephantNotesV3Controller* elephantnotes_v3_get_controller(void);

// Callbacks pour l'UI Framework
void ui_icon_click_handler(UIIconType iconType, void* userData);
void ui_icon_hover_handler(UIIconType iconType, bool isHovering, void* userData);