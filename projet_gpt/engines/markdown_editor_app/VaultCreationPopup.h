// VaultCreationPopup.h - Fenêtre popup pour créer un vault
#import <Cocoa/Cocoa.h>
#include "../vault_manager/vault_manager.h"

@class VaultCreationPopup;

@protocol VaultCreationDelegate <NSObject>
- (void)vaultCreationPopup:(VaultCreationPopup*)popup didCompleteWithSuccess:(BOOL)success vaultPath:(NSString*)vaultPath;
@end

@interface VaultCreationPopup : NSWindowController {
    NSTextField* _nameField;
    NSTextField* _locationField;
    NSTextView* _descriptionTextView;
    NSButton* _createSamplesCheckbox;
    NSButton* _browseButton;
    NSButton* _createButton;
    NSButton* _cancelButton;
    NSProgressIndicator* _progressIndicator;
    NSTextField* _statusLabel;
}

@property (nonatomic, assign) id<VaultCreationDelegate> delegate;

- (void)showPopup;

@end