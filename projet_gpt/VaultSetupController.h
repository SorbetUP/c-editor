// VaultSetupController.h - Interface de configuration du vault au premier démarrage
#import <Cocoa/Cocoa.h>
#include "../vault_manager/vault_manager.h"

@interface VaultSetupController : NSWindowController {
    IBOutlet NSTextField* _welcomeLabel;
    IBOutlet NSTextField* _vaultNameField;
    IBOutlet NSTextField* _vaultLocationField;
    IBOutlet NSButton* _browseButton;
    IBOutlet NSTextView* _descriptionTextView;
    IBOutlet NSPopUpButton* _templatePopup;
    IBOutlet NSButton* _createSamplesCheckbox;
    IBOutlet NSButton* _createButton;
    IBOutlet NSButton* _cancelButton;
    IBOutlet NSProgressIndicator* _progressIndicator;
    IBOutlet NSTextField* _statusLabel;
    
    NSString* _selectedVaultPath;
    BOOL _setupComplete;
}

@property (nonatomic, strong) NSString* selectedVaultPath;
@property (nonatomic, assign) BOOL setupComplete;

// Actions
- (IBAction)browseForLocation:(id)sender;
- (IBAction)createVault:(id)sender;
- (IBAction)cancelSetup:(id)sender;
- (IBAction)vaultNameChanged:(id)sender;

// Setup methods
- (void)showSetupWindow;
- (void)setupUI;
- (BOOL)validateInput;
- (void)updateSuggestedName;

// Completion handler
@property (nonatomic, copy) void (^completionHandler)(BOOL success, NSString* vaultPath);

@end