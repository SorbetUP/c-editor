#import <Cocoa/Cocoa.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

// Import our C engines
#import "editor_abi.h"
#import "markdown.h"

@interface MarkdownEditorViewController : NSViewController <NSTextViewDelegate>

// UI Components - matching HTML structure exactly
@property (strong, nonatomic) NSView *headerView;
@property (strong, nonatomic) NSTextField *titleLabel;
@property (strong, nonatomic) NSView *statusView;
@property (strong, nonatomic) NSView *statusIndicator;
@property (strong, nonatomic) NSTextField *statusLabel;

@property (strong, nonatomic) NSScrollView *scrollView;
@property (strong, nonatomic) NSTextView *textView;

// Buttons - same as HTML
@property (strong, nonatomic) NSButton *importButton;
@property (strong, nonatomic) NSButton *exportButton;
@property (strong, nonatomic) NSButton *clearCacheButton;

// C Engine Integration
@property (assign, nonatomic) BOOL engineReady;
@property (strong, nonatomic) NSMutableArray<NSString *> *lineContents;
@property (assign, nonatomic) NSInteger currentLineIndex;

// Rendering system (simplified for now)

// File operations
- (BOOL)openFile:(NSString *)filePath;
- (void)saveFile:(NSString *)filePath;
- (void)importMarkdownFile;
- (void)exportMarkdownFile;

@end