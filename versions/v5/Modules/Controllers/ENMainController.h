//
//  ENMainController.h
//  ElephantNotes V4 - Contrôleur principal de l'application
//

#import <Cocoa/Cocoa.h>
#include "ui_framework.h"
#include "vault_manager.h"
#include "professional_file_manager.h"
#include "advanced_search.h"

#import "../Sidebar/ENSidebar.h"
#import "../Tabs/ENTabBase.h"

@interface ENMainController : NSObject <ENSidebarDelegate, ENTabDelegate>

// Composants principaux
@property (nonatomic, strong) ENSidebar* sidebar;
@property (nonatomic, strong) NSMutableDictionary* tabs;
@property (nonatomic, strong) ENTabBase* currentTab;
@property (nonatomic, assign) UIFramework* uiFramework;

// Système de vaults
@property (nonatomic, copy) NSString* currentVaultPath;
@property (nonatomic, copy) NSString* currentVaultName;
@property (nonatomic, assign) VaultRegistry* vaultRegistry;

// État de l'application
@property (nonatomic, assign) bool isReady;

// Initialisation
- (instancetype)init;
- (void)setupWithUIFramework:(UIFramework*)framework;
- (void)initializeVaultSystem;
- (void)setupTabs;
- (void)setupSidebar;

// Gestion des onglets
- (void)switchToTab:(UIIconType)iconType;
- (void)registerTab:(ENTabBase*)tab forIconType:(UIIconType)iconType;
- (ENTabBase*)tabForIconType:(UIIconType)iconType;

// Gestion des vaults
- (void)loadVault:(NSString*)vaultPath;
- (void)updateTabsWithVaultInfo;

// Utilitaires
- (NSString*)nameForIconType:(UIIconType)iconType;

@end