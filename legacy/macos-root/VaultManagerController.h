// VaultManagerController.h - Interface de gestion des vaults
#import <Cocoa/Cocoa.h>
#include "../vault_manager/vault_manager.h"

@interface VaultManagerController : NSWindowController <NSTableViewDataSource, NSTableViewDelegate> {
    IBOutlet NSTableView* _vaultsTableView;
    IBOutlet NSButton* _addVaultButton;
    IBOutlet NSButton* _removeVaultButton;
    IBOutlet NSButton* _setDefaultButton;
    IBOutlet NSButton* _openVaultButton;
    IBOutlet NSButton* _closeButton;
    IBOutlet NSTextField* _vaultInfoLabel;
    IBOutlet NSTextField* _vaultPathLabel;
    IBOutlet NSTextField* _vaultStatsLabel;
    
    VaultRegistry* _registry;
    NSMutableArray* _vaultInfos;
    NSInteger _selectedVaultIndex;
}

@property (nonatomic, assign) NSInteger selectedVaultIndex;

// Actions
- (IBAction)addVault:(id)sender;
- (IBAction)removeVault:(id)sender;
- (IBAction)setAsDefault:(id)sender;
- (IBAction)openVaultLocation:(id)sender;
- (IBAction)closeManager:(id)sender;

// Management
- (void)showVaultManager;
- (void)refreshVaultList;
- (void)updateVaultInfo;
- (void)selectVaultAtIndex:(NSInteger)index;

// Completion handler
@property (nonatomic, copy) void (^vaultChangedHandler)(NSString* newVaultPath);

@end