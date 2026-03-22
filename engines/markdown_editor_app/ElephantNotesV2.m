// ElephantNotesV2.m - Advanced markdown editor with C libraries and file operations
#import <Cocoa/Cocoa.h>
#include "../hybrid_editor/hybrid_editor_core.h"
#include "../file_manager/file_manager.h"
#include "../editor/editor_abi.h"

@interface ElephantNotesTextView : NSTextView {
    NSInteger _currentLineIndex;
    BOOL _isUpdating;
    TextLines* _textLines;
    NSString* _currentFilePath;
    NSString* _originalContent;
    BOOL _hasUnsavedChanges;
}

@property (nonatomic, strong) NSString* currentFilePath;
@property (nonatomic, assign) BOOL hasUnsavedChanges;

- (void)newDocument;
- (void)openDocument;
- (void)saveDocument;
- (void)saveDocumentAs;
- (BOOL)hasUnsavedChanges;
- (void)updateWindowTitle;

@end

@implementation ElephantNotesTextView

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        _currentLineIndex = 0;
        _isUpdating = NO;
        _textLines = NULL;
        _currentFilePath = nil;
        _originalContent = @"";
        _hasUnsavedChanges = NO;
        
        // Configure text view
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]];
        
        // Welcome content
        [self newDocument];
        
        // Real-time updates
        [NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(updateDisplay) userInfo:nil repeats:YES];
        
        // Initial render
        dispatch_async(dispatch_get_main_queue(), ^{
            [self updateDisplay];
        });
    }
    return self;
}

- (void)dealloc {
    if (_textLines) {
        hybrid_free_text_lines(_textLines);
    }
}

- (void)newDocument {
    // Use v1 content for testing
    NSString* welcomeContent = @"# Test Markdown Strict\n\nTests des couleurs :\n- **Gras en vert**\n- *Italique en jaune*\n- ==Surligné en orange==\n\n## Test mixte\nTexte avec **gras**, *italique* et ==surligné== ensemble.";
    
    [self setString:welcomeContent];
    self.currentFilePath = nil;
    _originalContent = welcomeContent;
    self.hasUnsavedChanges = NO;
    [self updateWindowTitle];
}

- (void)openDocument {
    NSOpenPanel* openPanel = [NSOpenPanel openPanel];
    [openPanel setCanChooseFiles:YES];
    [openPanel setCanChooseDirectories:NO];
    [openPanel setAllowsMultipleSelection:NO];
    [openPanel setAllowedFileTypes:@[@"md", @"markdown", @"mdown", @"mkd", @"txt"]];
    [openPanel setTitle:@"Open Markdown Document"];
    
    // Set initial directory to Documents
    char* docs_dir = file_get_documents_dir();
    if (docs_dir) {
        NSString* docsPath = [NSString stringWithUTF8String:docs_dir];
        [openPanel setDirectoryURL:[NSURL fileURLWithPath:docsPath]];
        file_free_string(docs_dir);
    }
    
    [openPanel beginWithCompletionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* url = [[openPanel URLs] firstObject];
            [self loadFileAtPath:[url path]];
        }
    }];
}

- (void)loadFileAtPath:(NSString*)path {
    if (!path) return;
    
    const char* path_cstr = [path UTF8String];
    FileContent* content = NULL;
    
    FileResult result = file_read(path_cstr, &content);
    if (result == FILE_SUCCESS && content) {
        NSString* fileContent = [NSString stringWithUTF8String:content->content];
        [self setString:fileContent];
        
        self.currentFilePath = path;
        _originalContent = fileContent;
        self.hasUnsavedChanges = NO;
        [self updateWindowTitle];
        
        // Add to recent files (implement later)
        
        file_free_content(content);
        
        NSLog(@"✅ Loaded file: %@", path);
    } else {
        NSAlert* alert = [[NSAlert alloc] init];
        [alert setMessageText:@"Error Opening File"];
        [alert setInformativeText:[NSString stringWithFormat:@"Could not open file: %s", file_get_error_message(result)]];
        [alert setAlertStyle:NSAlertStyleWarning];
        [alert runModal];
    }
}

- (void)saveDocument {
    if (self.currentFilePath) {
        [self saveToPath:self.currentFilePath];
    } else {
        [self saveDocumentAs];
    }
}

- (void)saveDocumentAs {
    NSSavePanel* savePanel = [NSSavePanel savePanel];
    [savePanel setAllowedFileTypes:@[@"md"]];
    [savePanel setTitle:@"Save Markdown Document"];
    [savePanel setNameFieldStringValue:@"Untitled.md"];
    
    // Set initial directory to Documents
    char* docs_dir = file_get_documents_dir();
    if (docs_dir) {
        NSString* docsPath = [NSString stringWithUTF8String:docs_dir];
        [savePanel setDirectoryURL:[NSURL fileURLWithPath:docsPath]];
        file_free_string(docs_dir);
    }
    
    [savePanel beginWithCompletionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* url = [savePanel URL];
            NSString* path = [url path];
            
            // Ensure .md extension
            if (![path.pathExtension isEqualToString:@"md"]) {
                path = [path stringByAppendingPathExtension:@"md"];
            }
            
            [self saveToPath:path];
        }
    }];
}

- (void)saveToPath:(NSString*)path {
    if (!path) return;
    
    NSString* content = [self string];
    const char* path_cstr = [path UTF8String];
    const char* content_cstr = [content UTF8String];
    
    // Use improved save with backup and comparison
    FileResult result = file_save_with_backup(path_cstr, content_cstr, strlen(content_cstr));
    if (result == FILE_SUCCESS) {
        self.currentFilePath = path;
        _originalContent = content;
        self.hasUnsavedChanges = NO;
        [self updateWindowTitle];
        
        NSLog(@"✅ Saved file: %@", path);
        
        // Show brief success indicator
        [self showSaveSuccessIndicator];
    } else {
        NSAlert* alert = [[NSAlert alloc] init];
        [alert setMessageText:@"Error Saving File"];
        [alert setInformativeText:[NSString stringWithFormat:@"Could not save file: %s", file_get_error_message(result)]];
        [alert setAlertStyle:NSAlertStyleWarning];
        [alert runModal];
    }
}

- (void)showSaveSuccessIndicator {
    // Brief visual feedback for successful save
    NSView* superview = [[self enclosingScrollView] superview];
    NSTextField* indicator = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 20, 120, 30)];
    [indicator setStringValue:@"✅ Saved"];
    [indicator setBackgroundColor:[NSColor colorWithRed:0.0 green:0.7 blue:0.0 alpha:0.8]];
    [indicator setTextColor:[NSColor whiteColor]];
    [indicator setBordered:NO];
    [indicator setEditable:NO];
    [indicator setFont:[NSFont systemFontOfSize:12 weight:NSFontWeightSemibold]];
    [indicator.layer setCornerRadius:5];
    
    [superview addSubview:indicator];
    
    // Fade out after 2 seconds
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [NSAnimationContext runAnimationGroup:^(NSAnimationContext *context) {
            context.duration = 0.5;
            indicator.animator.alphaValue = 0.0;
        } completionHandler:^{
            [indicator removeFromSuperview];
        }];
    });
}

- (BOOL)hasUnsavedChanges {
    NSString* currentContent = [self string];
    return file_has_unsaved_changes([_originalContent UTF8String], [currentContent UTF8String]);
}

- (void)updateWindowTitle {
    NSWindow* window = [self window];
    if (!window) return;
    
    NSString* title;
    if (self.currentFilePath) {
        NSString* filename = [self.currentFilePath lastPathComponent];
        title = [self hasUnsavedChanges] ? [NSString stringWithFormat:@"%@ • ElephantNotes v2", filename] : [NSString stringWithFormat:@"%@ - ElephantNotes v2", filename];
    } else {
        title = [self hasUnsavedChanges] ? @"Untitled • ElephantNotes v2" : @"Untitled - ElephantNotes v2";
    }
    
    [window setTitle:title];
    [window setDocumentEdited:[self hasUnsavedChanges]];
}

// Hybrid editor logic using C library
- (void)updateDisplay {
    if (_isUpdating) return;
    
    NSInteger newCurrentLine = [self getCurrentLineUsingCLib];
    
    // Only update if current line changed (for performance)
    if (newCurrentLine == _currentLineIndex) return;
    
    _isUpdating = YES;
    
    // Check for unsaved changes
    BOOL hadChanges = self.hasUnsavedChanges;
    self.hasUnsavedChanges = [self hasUnsavedChanges];
    if (hadChanges != self.hasUnsavedChanges) {
        [self updateWindowTitle];
    }
    
    // Store selection
    NSRange oldSelection = [self selectedRange];
    
    // Apply hybrid formatting with REAL current line tracking
    [self applyHybridFormattingUsingCLib:newCurrentLine];
    
    // Restore selection
    [self setSelectedRange:oldSelection];
    
    _currentLineIndex = newCurrentLine;
    _isUpdating = NO;
}

- (NSInteger)getCurrentLineUsingCLib {
    // Use v1's reliable approach instead of buggy C library
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    NSUInteger cursorPos = [self selectedRange].location;
    NSUInteger pos = 0;
    
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSUInteger lineLength = [[lines objectAtIndex:i] length];
        if (cursorPos <= pos + lineLength) {
            return i;
        }
        pos += lineLength + 1; // +1 for \n
    }
    
    return [lines count] - 1; // Last line
}

- (void)applyHybridFormattingUsingCLib:(NSInteger)currentLine {
    NSMutableAttributedString *text = [[self textStorage] mutableCopy];
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    
    // Process each line using v1 approach
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSRange lineRange = [self getRangeForLineIndex:i];
        if (lineRange.length == 0 && i < [lines count] - 1) continue;
        
        NSString *lineContent = [lines objectAtIndex:i];
        
        if (i == currentLine) {
            // Current line: show raw markdown (disabled for now)
            [self formatAsCurrentLine:text range:lineRange];
        } else {
            // Other lines: use C library to format and hide markup
            [self formatLineUsingCLib:text range:lineRange content:[lineContent UTF8String]];
        }
    }
    
    [[self textStorage] setAttributedString:text];
}

// Add v1's getRangeForLineIndex method
- (NSRange)getRangeForLineIndex:(NSInteger)lineIndex {
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    if (lineIndex < 0 || lineIndex >= [lines count]) {
        return NSMakeRange(0, 0);
    }
    
    NSUInteger pos = 0;
    for (NSInteger i = 0; i < lineIndex; i++) {
        pos += [[lines objectAtIndex:i] length] + 1; // +1 for \n
    }
    
    return NSMakeRange(pos, [[lines objectAtIndex:lineIndex] length]);
}

- (void)formatAsCurrentLine:(NSMutableAttributedString *)text range:(NSRange)range {
    // Current line styling - blue background, raw markdown visible
    NSDictionary *attrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.25]
    };
    [text setAttributes:attrs range:range];
}

- (void)safeAddAttributes:(NSDictionary*)attrs toText:(NSMutableAttributedString*)text range:(NSRange)range {
    if (range.location + range.length <= [text length]) {
        [text addAttributes:attrs range:range];
    } else {
        NSLog(@"❌ Skipping invalid range: %lu + %lu > %lu", range.location, range.length, [text length]);
    }
}

// v1 Advanced functions for hiding markup
- (void)applyInlineFormatAndHide:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern attributes:(NSDictionary *)attributes {
    if (baseRange.location + baseRange.length > [text length]) return;
    
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    
    for (NSTextCheckingResult *match in matches) {
        if ([match numberOfRanges] < 2) continue;
        
        // Format the content part
        NSRange contentRange = NSMakeRange(baseRange.location + [match rangeAtIndex:1].location, [match rangeAtIndex:1].length);
        if (contentRange.location + contentRange.length <= [text length]) {
            [text addAttributes:attributes range:contentRange];
        }
        
        // Hide markup characters with tiny font and same color as background
        NSRange fullMatch = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        if (fullMatch.location + fullMatch.length <= [text length]) {
            NSRange beforeContent = NSMakeRange(fullMatch.location, contentRange.location - fullMatch.location);
            NSRange afterContent = NSMakeRange(NSMaxRange(contentRange), NSMaxRange(fullMatch) - NSMaxRange(contentRange));
            
            // Make markup invisible but preserve spacing
            NSDictionary *hideAttrs = @{
                NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0], // Same as background
                NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
            };
            
            if (beforeContent.length > 0 && beforeContent.location + beforeContent.length <= [text length]) {
                [text addAttributes:hideAttrs range:beforeContent];
            }
            if (afterContent.length > 0 && afterContent.location + afterContent.length <= [text length]) {
                [text addAttributes:hideAttrs range:afterContent];
            }
        }
    }
}

- (void)hideMarkupInRange:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern {
    if (baseRange.location + baseRange.length > [text length]) return;
    
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    for (NSTextCheckingResult *match in matches) {
        NSRange markupRange = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        if (markupRange.location + markupRange.length <= [text length]) {
            [text addAttributes:@{
                NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0], // Same as background
                NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
            } range:markupRange];
        }
    }
}

- (void)formatLineUsingCLib:(NSMutableAttributedString *)text range:(NSRange)range content:(const char*)content {
    // Validate range to prevent crashes
    if (range.location + range.length > [text length]) {
        NSLog(@"❌ Invalid range: %lu + %lu > %lu", range.location, range.length, [text length]);
        return;
    }
    
    NSString *contentStr = [NSString stringWithUTF8String:content];
    
    // Use C engine to detect formatting like v1
    const char *html = editor_markdown_to_html(content);
    if (!html) {
        [self applyBasicFormatting:text inRange:range content:contentStr];
        return;
    }
    
    NSString *htmlString = [NSString stringWithUTF8String:html];
    
    // Apply v1-style formatting with markup hiding
    [self applyFormattingAndHideMarkupV1Style:text inRange:range content:contentStr htmlResult:htmlString];
}

- (void)applyBasicFormatting:(NSMutableAttributedString *)text inRange:(NSRange)range content:(NSString *)content {
    // Base styling like v1
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:range];
}

- (void)applyFormattingAndHideMarkupV1Style:(NSMutableAttributedString *)text inRange:(NSRange)baseRange content:(NSString *)content htmlResult:(NSString *)html {
    // Base styling like v1
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:baseRange];
    
    // Headers (same logic as v1)
    if ([html containsString:@"<h1>"]) {
        [self safeAddAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:20] ?: [NSFont monospacedSystemFontOfSize:20 weight:NSFontWeightBold]
        } toText:text range:baseRange];
        [self hideMarkupInRange:text baseRange:baseRange pattern:@"^# "];
        return;
    }
    
    if ([html containsString:@"<h2>"]) {
        [self safeAddAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:18] ?: [NSFont monospacedSystemFontOfSize:18 weight:NSFontWeightBold]
        } toText:text range:baseRange];
        [self hideMarkupInRange:text baseRange:baseRange pattern:@"^## "];
        return;
    }
    
    if ([html containsString:@"<h3>"]) {
        [self safeAddAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:16] ?: [NSFont monospacedSystemFontOfSize:16 weight:NSFontWeightBold]
        } toText:text range:baseRange];
        [self hideMarkupInRange:text baseRange:baseRange pattern:@"^### "];
        return;
    }
    
    // Bold (exact v1 implementation - NO return)
    if ([html containsString:@"<strong>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"\\*\\*([^*\\n]+)\\*\\*" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0],
                                NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold]
                            }];
    }
    
    // Italic (exact v1 implementation - NO return)
    if ([html containsString:@"<em>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"(?<!\\*)\\*([^*\\n]+)\\*(?!\\*)" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]
                            }];
    }
    
    // Highlight (exact v1 implementation - NO return)
    if ([html containsString:@"<mark>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"==([^=\\n]+)==" 
                            attributes:@{
                                NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.65 blue:0.0 alpha:0.8],
                                NSForegroundColorAttributeName: [NSColor blackColor]
                            }];
    }
}

- (void)applyDetailedFormattingUsingCLib:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange formats:(LineFormats*)formats {
    for (int i = 0; i < formats->format_count; i++) {
        FormatInfo* info = &formats->formats[i];
        
        // Calculate actual ranges
        NSRange contentRange = NSMakeRange(baseRange.location + info->content_range.start, 
                                         info->content_range.end - info->content_range.start);
        NSRange fullRange = NSMakeRange(baseRange.location + info->range.start,
                                      info->range.end - info->range.start);
        
        // Apply formatting based on type
        if (info->format == MD_FORMAT_BOLD) {
            [text addAttributes:@{
                NSForegroundColorAttributeName: [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0],
                NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold]
            } range:contentRange];
        } else if (info->format == MD_FORMAT_ITALIC) {
            [text addAttribute:NSForegroundColorAttributeName 
                         value:[NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]
                         range:contentRange];
        } else if (info->format == MD_FORMAT_HIGHLIGHT) {
            [text addAttributes:@{
                NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.65 blue:0.0 alpha:0.8],
                NSForegroundColorAttributeName: [NSColor blackColor]
            } range:contentRange];
        }
        
        // Hide markup characters
        [self hideMarkupCharacters:text fullRange:fullRange contentRange:contentRange];
    }
}

- (void)hideMarkupCharacters:(NSMutableAttributedString *)text fullRange:(NSRange)fullRange contentRange:(NSRange)contentRange {
    // Hide opening markup
    if (contentRange.location > fullRange.location) {
        NSRange openMarkup = NSMakeRange(fullRange.location, contentRange.location - fullRange.location);
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
        } range:openMarkup];
    }
    
    // Hide closing markup  
    NSUInteger contentEnd = NSMaxRange(contentRange);
    NSUInteger fullEnd = NSMaxRange(fullRange);
    if (fullEnd > contentEnd) {
        NSRange closeMarkup = NSMakeRange(contentEnd, fullEnd - contentEnd);
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
        } range:closeMarkup];
    }
}

- (void)hideMarkupUsingCLib:(NSMutableAttributedString *)text range:(NSRange)range content:(const char*)content pattern:(const char*)pattern {
    // Simple pattern hiding for headers
    NSString* contentStr = [NSString stringWithUTF8String:content];
    NSString* patternStr = [NSString stringWithUTF8String:pattern];
    
    if ([contentStr hasPrefix:patternStr]) {
        NSRange markupRange = NSMakeRange(range.location, [patternStr length]);
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
        } range:markupRange];
    }
}

// Handle keyboard shortcuts
- (void)keyDown:(NSEvent *)event {
    NSString* characters = [event charactersIgnoringModifiers];
    NSEventModifierFlags modifiers = [event modifierFlags];
    
    if (modifiers & NSEventModifierFlagCommand) {
        if ([characters isEqualToString:@"o"]) {
            [self openDocument];
            return;
        } else if ([characters isEqualToString:@"s"]) {
            if (modifiers & NSEventModifierFlagShift) {
                [self saveDocumentAs];
            } else {
                [self saveDocument];
            }
            return;
        } else if ([characters isEqualToString:@"n"]) {
            [self newDocument];
            return;
        }
    }
    
    [super keyDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self updateDisplay];
    });
}

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self updateDisplay];
    });
}

@end

// Main application
int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C libraries
        editor_library_init();
        file_manager_init();
        
        // Configure hybrid editor
        HybridConfig config = hybrid_get_config();
        config.strict_markdown = true;
        config.enable_bold = true;
        config.enable_italic = true;
        config.enable_highlight = true;
        config.enable_headers = true;
        config.enable_lists = true;
        hybrid_set_config(&config);
        
        // Configure file manager
        FileManagerConfig fileConfig = file_manager_get_config();
        fileConfig.auto_backup = true;
        fileConfig.max_file_size = 50 * 1024 * 1024; // 50MB
        file_manager_set_config(&fileConfig);
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1200, 800);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"ElephantNotes v2.0"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create menu bar
        NSMenu *mainMenu = [[NSMenu alloc] init];
        
        // File menu
        NSMenuItem *fileMenuItem = [[NSMenuItem alloc] init];
        NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
        
        NSMenuItem *newItem = [[NSMenuItem alloc] initWithTitle:@"New" action:@selector(newDocument) keyEquivalent:@"n"];
        NSMenuItem *openItem = [[NSMenuItem alloc] initWithTitle:@"Open..." action:@selector(openDocument) keyEquivalent:@"o"];
        NSMenuItem *saveItem = [[NSMenuItem alloc] initWithTitle:@"Save" action:@selector(saveDocument) keyEquivalent:@"s"];
        NSMenuItem *saveAsItem = [[NSMenuItem alloc] initWithTitle:@"Save As..." action:@selector(saveDocumentAs) keyEquivalent:@"S"];
        
        [fileMenu addItem:newItem];
        [fileMenu addItem:openItem];
        [fileMenu addItem:[NSMenuItem separatorItem]];
        [fileMenu addItem:saveItem];
        [fileMenu addItem:saveAsItem];
        
        [fileMenuItem setSubmenu:fileMenu];
        [mainMenu addItem:fileMenuItem];
        [app setMainMenu:mainMenu];
        
        // Create scroll view
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(0, 0, frame.size.width, frame.size.height)];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Create the ElephantNotes text view
        ElephantNotesTextView *textView = [[ElephantNotesTextView alloc] initWithFrame:[scrollView.contentView bounds]];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        [[textView textContainer] setWidthTracksTextView:YES];
        [[textView textContainer] setContainerSize:NSMakeSize([scrollView.contentView bounds].size.width, FLT_MAX)];
        
        // Set targets for menu items
        [newItem setTarget:textView];
        [openItem setTarget:textView];
        [saveItem setTarget:textView];
        [saveAsItem setTarget:textView];
        
        [scrollView setDocumentView:textView];
        [[window contentView] addSubview:scrollView];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        [window makeFirstResponder:textView];
        
        [app run];
        
        // Cleanup
        file_manager_cleanup();
    }
    return 0;
}