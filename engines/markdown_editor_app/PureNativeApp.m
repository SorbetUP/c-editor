#import <Cocoa/Cocoa.h>
#import "../editor/editor_abi.h"

@interface RealTimeHybridTextView : NSTextView {
    NSInteger _currentLineIndex;
    BOOL _isUpdating;
}
@end

@implementation RealTimeHybridTextView

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        _currentLineIndex = 0;
        _isUpdating = NO;
        
        // Configure text view
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]];
        
        // Initial content avec tests de toutes les fonctionnalités
        [self setString:@"# Test Markdown Strict\n\nTests des couleurs :\n- **Gras en vert**\n- *Italique en jaune*\n- ==Surligné en orange==\n- ++Souligné++\n\nTests incomplets :\n- **Gras incomplet\n- *Italique incomplet\n- ==Surligné incomplet\n\n## Test mixte\nTexte avec **gras**, *italique* et ==surligné== ensemble.\n\nListe :\n* Item 1\n+ Item 2\n- Item 3"];
        
        // Start the real-time update
        [NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(updateHybridDisplay) userInfo:nil repeats:YES];
    }
    return self;
}

- (NSInteger)getCurrentLineIndex {
    NSRange selectedRange = [self selectedRange];
    NSString *text = [self string];
    
    NSInteger lineIndex = 0;
    NSUInteger currentPos = 0;
    
    for (NSUInteger i = 0; i <= [text length]; i++) {
        if (i == [text length] || [text characterAtIndex:i] == '\n') {
            if (selectedRange.location >= currentPos && selectedRange.location <= i) {
                return lineIndex;
            }
            lineIndex++;
            currentPos = i + 1;
        }
    }
    
    return MAX(0, lineIndex - 1);
}

- (NSArray *)getLines {
    return [[self string] componentsSeparatedByString:@"\n"];
}

- (NSRange)getRangeForLineIndex:(NSInteger)lineIndex {
    NSArray *lines = [self getLines];
    if (lineIndex < 0 || lineIndex >= [lines count]) {
        return NSMakeRange(0, 0);
    }
    
    NSUInteger pos = 0;
    for (NSInteger i = 0; i < lineIndex; i++) {
        pos += [[lines objectAtIndex:i] length] + 1; // +1 for \n
    }
    
    return NSMakeRange(pos, [[lines objectAtIndex:lineIndex] length]);
}

- (void)updateHybridDisplay {
    if (_isUpdating) return;
    
    NSInteger newCurrentLine = [self getCurrentLineIndex];
    // Désactivé temporairement - pas de mise à jour de ligne courante
    // if (newCurrentLine == _currentLineIndex) return;
    
    _isUpdating = YES;
    
    // Store current selection
    NSRange oldSelection = [self selectedRange];
    
    // Apply formatting to all lines (sans effet de sélection)
    [self applyHybridFormattingForCurrentLine:-1]; // -1 = pas de ligne courante
    
    // Restore selection
    [self setSelectedRange:oldSelection];
    
    _currentLineIndex = newCurrentLine;
    _isUpdating = NO;
}

- (void)applyHybridFormattingForCurrentLine:(NSInteger)currentLine {
    NSMutableAttributedString *text = [[self textStorage] mutableCopy];
    NSArray *lines = [self getLines];
    
    // Apply formatting line by line
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSRange lineRange = [self getRangeForLineIndex:i];
        if (lineRange.length == 0 && i < [lines count] - 1) continue;
        
        NSString *lineContent = [lines objectAtIndex:i];
        
        if (i == currentLine) {
            // Current line: plain markdown with blue background
            [self formatAsCurrentLine:text range:lineRange];
        } else {
            // Other lines: rendered with markdown formatting (cache les caractères)
            [self formatAsRenderedLine:text range:lineRange content:lineContent];
        }
    }
    
    // Apply the formatting
    [[self textStorage] setAttributedString:text];
}

- (void)formatAsCurrentLine:(NSMutableAttributedString *)text range:(NSRange)range {
    // Ligne sélectionnée avec surlignage bien visible
    NSDictionary *attrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.25] // Surlignage bleu plus visible
    };
    
    [text setAttributes:attrs range:range];
}

- (void)formatAsRenderedLine:(NSMutableAttributedString *)text range:(NSRange)range content:(NSString *)content {
    // Simple approach: format with C engine detection but keep original spacing
    [self renderWithCEngineKeepSpacing:text inRange:range content:content];
}

- (void)renderWithCEngineKeepSpacing:(NSMutableAttributedString *)text inRange:(NSRange)baseRange content:(NSString *)content {
    // Use C engine to detect what formatting to apply
    const char *html = editor_markdown_to_html([content UTF8String]);
    if (!html) {
        [self applyBasicFormatting:text inRange:baseRange content:content];
        return;
    }
    
    NSString *htmlString = [NSString stringWithUTF8String:html];
    
    // Apply formatting but hide markup characters instead of removing them
    [self applyFormattingAndHideMarkup:text inRange:baseRange content:content htmlResult:htmlString];
}

- (void)applyFormattingAndHideMarkup:(NSMutableAttributedString *)text inRange:(NSRange)baseRange content:(NSString *)content htmlResult:(NSString *)html {
    // Base styling
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:baseRange];
    
    // Headers
    if ([html containsString:@"<h1>"]) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:20] ?: [NSFont monospacedSystemFontOfSize:20 weight:NSFontWeightBold]
        } range:baseRange];
        [self hideMarkupInRange:text baseRange:baseRange pattern:@"^# "];
        return;
    }
    
    if ([html containsString:@"<h2>"]) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:18] ?: [NSFont monospacedSystemFontOfSize:18 weight:NSFontWeightBold]
        } range:baseRange];
        [self hideMarkupInRange:text baseRange:baseRange pattern:@"^## "];
        return;
    }
    
    // Bold
    if ([html containsString:@"<strong>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"\\*\\*([^*\\n]+)\\*\\*" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0],
                                NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold]
                            }];
    }
    
    // Italic
    if ([html containsString:@"<em>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"(?<!\\*)\\*([^*\\n]+)\\*(?!\\*)" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]
                            }];
    }
    
    // Highlight
    if ([html containsString:@"<mark>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"==([^=\\n]+)==" 
                            attributes:@{
                                NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.65 blue:0.0 alpha:0.8],
                                NSForegroundColorAttributeName: [NSColor blackColor]
                            }];
    }
}

- (void)applyInlineFormatAndHide:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern attributes:(NSDictionary *)attributes {
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    
    for (NSTextCheckingResult *match in matches) {
        // Format the content part
        NSRange contentRange = NSMakeRange(baseRange.location + [match rangeAtIndex:1].location, [match rangeAtIndex:1].length);
        [text addAttributes:attributes range:contentRange];
        
        // Hide markup characters with tiny font
        NSRange fullMatch = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        NSRange beforeContent = NSMakeRange(fullMatch.location, contentRange.location - fullMatch.location);
        NSRange afterContent = NSMakeRange(NSMaxRange(contentRange), NSMaxRange(fullMatch) - NSMaxRange(contentRange));
        
        // Make markup very small and transparent
        NSDictionary *hideAttrs = @{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0], // Same as background
            NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
        };
        
        if (beforeContent.length > 0) {
            [text addAttributes:hideAttrs range:beforeContent];
        }
        if (afterContent.length > 0) {
            [text addAttributes:hideAttrs range:afterContent];
        }
    }
}

- (void)hideMarkupInRange:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern {
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    for (NSTextCheckingResult *match in matches) {
        NSRange markupRange = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0], // Same as background
            NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
        } range:markupRange];
    }
}



- (void)applyBasicFormatting:(NSMutableAttributedString *)text inRange:(NSRange)baseRange content:(NSString *)content {
    // Fallback formatting when C engine is unavailable
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:baseRange];
    
    // Headers 
    if ([content hasPrefix:@"# "] && [content length] > 2) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:20] ?: [NSFont monospacedSystemFontOfSize:20 weight:NSFontWeightBold]
        } range:baseRange];
        return;
    }
    
    if ([content hasPrefix:@"## "] && [content length] > 3) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:18] ?: [NSFont monospacedSystemFontOfSize:18 weight:NSFontWeightBold]
        } range:baseRange];
        return;
    }
    
    // Basic regex formatting without hiding
    [self applyRegexFormatting:text inRange:baseRange pattern:@"\\*\\*([^*\\n]+)\\*\\*" 
                    attributes:@{
                        NSForegroundColorAttributeName: [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0],
                        NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold]
                    }];
    
    [self applyRegexFormatting:text inRange:baseRange pattern:@"(?<!\\*)\\*([^*\\n]+)\\*(?!\\*)" 
                    attributes:@{
                        NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]
                    }];
    
    [self applyRegexFormatting:text inRange:baseRange pattern:@"==([^=\\n]+)==" 
                    attributes:@{
                        NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.65 blue:0.0 alpha:0.8],
                        NSForegroundColorAttributeName: [NSColor blackColor]
                    }];
}



- (void)applyRegexFormatting:(NSMutableAttributedString *)text 
                     inRange:(NSRange)baseRange 
                     pattern:(NSString *)pattern 
                  attributes:(NSDictionary *)attributes {
    
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    
    if (error) {
        return;
    }
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    
    // Appliquer les matches en ordre inverse pour éviter les conflits de range
    for (NSTextCheckingResult *match in [matches reverseObjectEnumerator]) {
        NSRange matchRange = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        if (NSMaxRange(matchRange) <= NSMaxRange(baseRange)) { // Vérification de sécurité
            [text addAttributes:attributes range:matchRange];
        }
    }
}

// Override events to trigger updates
- (void)keyDown:(NSEvent *)event {
    [super keyDown:event];
    // Immediate update after key press
    dispatch_async(dispatch_get_main_queue(), ^{
        [self updateHybridDisplay];
    });
}

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    // Immediate update after click
    dispatch_async(dispatch_get_main_queue(), ^{
        [self updateHybridDisplay];
    });
}

@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C engine
        editor_library_init();
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1000, 700);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"ElephantNotes"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Header
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, frame.size.height - 60, frame.size.width, 60)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [headerView setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 20, 400, 20)];
        [titleLabel setStringValue:@"ElephantNotes"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:16 weight:NSFontWeightSemibold]];
        [headerView addSubview:titleLabel];
        
        NSView *statusIndicator = [[NSView alloc] initWithFrame:NSMakeRect(frame.size.width - 200, 25, 8, 8)];
        [statusIndicator setWantsLayer:YES];
        [statusIndicator layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.4 alpha:1.0].CGColor;
        [statusIndicator layer].cornerRadius = 4;
        [statusIndicator setAutoresizingMask:NSViewMinXMargin];
        [headerView addSubview:statusIndicator];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(frame.size.width - 180, 20, 150, 15)];
        [statusLabel setStringValue:@"Markdown Editor Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [statusLabel setAutoresizingMask:NSViewMinXMargin];
        [headerView addSubview:statusLabel];
        
        // Pane header
        NSView *paneHeader = [[NSView alloc] initWithFrame:NSMakeRect(0, frame.size.height - 110, frame.size.width, 50)];
        [paneHeader setWantsLayer:YES];
        [paneHeader layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [paneHeader setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *paneTitle = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 700, 20)];
        [paneTitle setStringValue:@"ElephantNotes - Éditeur Markdown Hybride - Ligne courante en Markdown, autres rendues"];
        [paneTitle setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneTitle setBackgroundColor:[NSColor clearColor]];
        [paneTitle setBordered:NO];
        [paneTitle setEditable:NO];
        [paneTitle setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneHeader addSubview:paneTitle];
        
        // Create scroll view
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(20, 20, frame.size.width - 40, frame.size.height - 150)];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setHasHorizontalScroller:NO];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setBorderType:NSLineBorder];
        [scrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Create the hybrid text view
        RealTimeHybridTextView *textView = [[RealTimeHybridTextView alloc] initWithFrame:[scrollView.contentView bounds]];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        [[textView textContainer] setWidthTracksTextView:YES];
        [[textView textContainer] setContainerSize:NSMakeSize([scrollView.contentView bounds].size.width, FLT_MAX)];
        
        [scrollView setDocumentView:textView];
        
        // Add views to window
        [[window contentView] addSubview:headerView];
        [[window contentView] addSubview:paneHeader];
        [[window contentView] addSubview:scrollView];
        
        // Show window and focus
        [window makeKeyAndOrderFront:nil];
        [window center];
        [window makeFirstResponder:textView];
        
        
        [app run];
    }
    return 0;
}