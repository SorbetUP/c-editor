#import <Cocoa/Cocoa.h>
#import "../editor/editor_abi.h"

@interface HybridTextView : NSTextView
@property (nonatomic, assign) NSInteger currentLineIndex;
@property (nonatomic, strong) NSMutableArray *lineContents;
@property (nonatomic, strong) NSMutableArray *lineRanges;
@property (nonatomic, strong) NSTimer *updateTimer;
@end

@implementation HybridTextView

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        self.currentLineIndex = 0;
        self.lineContents = [[NSMutableArray alloc] init];
        self.lineRanges = [[NSMutableArray alloc] init];
        
        // Configure like web version
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        
        // Setup initial content
        [self setupInitialContent];
        
        // Setup update timer for frequent updates
        self.updateTimer = [NSTimer scheduledTimerWithTimeInterval:0.05
                                                            target:self
                                                          selector:@selector(updateRendering)
                                                          userInfo:nil
                                                           repeats:YES];
    }
    return self;
}

- (void)dealloc {
    [self.updateTimer invalidate];
}

- (void)setupInitialContent {
    NSArray *initialLines = @[
        @"# Titre",
        @"",
        @"Écrivez votre markdown ici...",
        @"",
        @"Exemple:",
        @"- **Gras**",
        @"- *Italique*",
        @"- ==Surligné==",
        @"- ++Souligné++"
    ];
    
    [self.lineContents addObjectsFromArray:initialLines];
    
    NSString *fullText = [initialLines componentsJoinedByString:@"\n"];
    [self setString:fullText];
    
    [self calculateLineRanges];
    [self updateRendering];
}

- (void)calculateLineRanges {
    [self.lineRanges removeAllObjects];
    
    NSString *text = [self string];
    NSUInteger length = [text length];
    NSUInteger lineStart = 0;
    
    for (NSUInteger i = 0; i <= length; i++) {
        if (i == length || [text characterAtIndex:i] == '\n') {
            NSRange lineRange = NSMakeRange(lineStart, i - lineStart);
            [self.lineRanges addObject:[NSValue valueWithRange:lineRange]];
            lineStart = i + 1;
        }
    }
}

- (NSInteger)getCurrentLineIndex {
    NSRange selectedRange = [self selectedRange];
    
    for (NSInteger i = 0; i < [self.lineRanges count]; i++) {
        NSRange lineRange = [[self.lineRanges objectAtIndex:i] rangeValue];
        if (selectedRange.location >= lineRange.location && 
            selectedRange.location <= NSMaxRange(lineRange)) {
            return i;
        }
    }
    return 0;
}

- (NSString *)getLineContent:(NSInteger)lineIndex {
    if (lineIndex < 0 || lineIndex >= [self.lineRanges count]) return @"";
    
    NSRange lineRange = [[self.lineRanges objectAtIndex:lineIndex] rangeValue];
    return [[self string] substringWithRange:lineRange];
}

- (void)updateRendering {
    NSInteger newCurrentLine = [self getCurrentLineIndex];
    
    // Recalculate line ranges and contents
    [self calculateLineRanges];
    
    // Update line contents from current text
    [self.lineContents removeAllObjects];
    for (NSInteger i = 0; i < [self.lineRanges count]; i++) {
        [self.lineContents addObject:[self getLineContent:i]];
    }
    
    // Always re-render if current line changed
    if (newCurrentLine != self.currentLineIndex) {
        NSLog(@"📍 Cursor moved to line %ld (was %ld)", newCurrentLine, self.currentLineIndex);
        self.currentLineIndex = newCurrentLine;
        [self renderAllLines];
    }
    // Also re-render if line count changed (new lines added/removed)
    else if ([self.lineRanges count] != [[[self textStorage] string] componentsSeparatedByString:@"\n"].count) {
        [self renderAllLines];
    }
}

- (void)renderAllLines {
    NSMutableAttributedString *newText = [[NSMutableAttributedString alloc] init];
    
    for (NSInteger i = 0; i < [self.lineContents count]; i++) {
        NSString *lineContent = [self.lineContents objectAtIndex:i];
        NSAttributedString *lineAttr;
        
        if (i == self.currentLineIndex) {
            // Current line: show as plain markdown
            lineAttr = [self createMarkdownAttributedString:lineContent];
        } else {
            // Other lines: render as HTML
            lineAttr = [self createRenderedAttributedString:lineContent];
        }
        
        [newText appendAttributedString:lineAttr];
        
        // Add newline except for last line
        if (i < [self.lineContents count] - 1) {
            NSAttributedString *newline = [[NSAttributedString alloc] 
                initWithString:@"\n" 
                attributes:@{NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]}];
            [newText appendAttributedString:newline];
        }
    }
    
    // Preserve cursor position
    NSRange oldSelection = [self selectedRange];
    
    [[self textStorage] setAttributedString:newText];
    
    // Restore cursor position
    if (oldSelection.location <= [[self string] length]) {
        [self setSelectedRange:oldSelection];
    }
}

- (NSAttributedString *)createMarkdownAttributedString:(NSString *)markdown {
    NSDictionary *markdownAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.1] // Current line highlight
    };
    
    return [[NSAttributedString alloc] initWithString:markdown attributes:markdownAttrs];
}

- (NSAttributedString *)createRenderedAttributedString:(NSString *)markdown {
    if (!markdown || [markdown length] == 0) {
        return [[NSAttributedString alloc] initWithString:@"" attributes:@{}];
    }
    
    // Use C engine to render
    const char *html = editor_markdown_to_html([markdown UTF8String]);
    NSString *htmlString = html ? [NSString stringWithUTF8String:html] : markdown;
    
    // Convert simple HTML to attributed string
    NSMutableAttributedString *result = [[NSMutableAttributedString alloc] init];
    
    // Parse simple HTML tags and convert to attributes
    [self parseAndApplyHTML:htmlString toAttributedString:result];
    
    return result;
}

- (void)parseAndApplyHTML:(NSString *)html toAttributedString:(NSMutableAttributedString *)result {
    // Base attributes for rendered lines
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:1.0 blue:1.0 alpha:0.02] // Rendered line background
    };
    
    // Simple HTML parsing for basic tags
    NSString *text = html;
    
    // Remove HTML tags and apply formatting
    if ([text containsString:@"<h1>"]) {
        text = [text stringByReplacingOccurrencesOfString:@"<h1>" withString:@""];
        text = [text stringByReplacingOccurrencesOfString:@"</h1>" withString:@""];
        
        NSMutableDictionary *headerAttrs = [baseAttrs mutableCopy];
        headerAttrs[NSFontAttributeName] = [NSFont fontWithName:@"Monaco" size:24] ?: [NSFont monospacedSystemFontOfSize:24 weight:NSFontWeightBold];
        headerAttrs[NSForegroundColorAttributeName] = [NSColor whiteColor];
        
        [result appendAttributedString:[[NSAttributedString alloc] initWithString:text attributes:headerAttrs]];
        return;
    }
    
    if ([text containsString:@"<h2>"]) {
        text = [text stringByReplacingOccurrencesOfString:@"<h2>" withString:@""];
        text = [text stringByReplacingOccurrencesOfString:@"</h2>" withString:@""];
        
        NSMutableDictionary *headerAttrs = [baseAttrs mutableCopy];
        headerAttrs[NSFontAttributeName] = [NSFont fontWithName:@"Monaco" size:20] ?: [NSFont monospacedSystemFontOfSize:20 weight:NSFontWeightBold];
        headerAttrs[NSForegroundColorAttributeName] = [NSColor whiteColor];
        
        [result appendAttributedString:[[NSAttributedString alloc] initWithString:text attributes:headerAttrs]];
        return;
    }
    
    if ([text containsString:@"<strong>"]) {
        // Handle bold text
        NSMutableAttributedString *temp = [[NSMutableAttributedString alloc] initWithString:text attributes:baseAttrs];
        
        NSRegularExpression *boldRegex = [NSRegularExpression regularExpressionWithPattern:@"<strong>(.*?)</strong>" 
                                                                                   options:0 
                                                                                     error:nil];
        NSArray *matches = [boldRegex matchesInString:text options:0 range:NSMakeRange(0, [text length])];
        
        for (NSTextCheckingResult *match in [matches reverseObjectEnumerator]) {
            NSRange fullRange = [match rangeAtIndex:0];
            NSRange contentRange = [match rangeAtIndex:1];
            NSString *boldText = [text substringWithRange:contentRange];
            
            NSMutableDictionary *boldAttrs = [baseAttrs mutableCopy];
            boldAttrs[NSForegroundColorAttributeName] = [NSColor whiteColor];
            boldAttrs[NSFontAttributeName] = [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold];
            
            [temp replaceCharactersInRange:fullRange withString:boldText];
            [temp setAttributes:boldAttrs range:NSMakeRange(fullRange.location, [boldText length])];
        }
        
        [result appendAttributedString:temp];
        return;
    }
    
    if ([text containsString:@"<em>"]) {
        // Handle italic text
        NSMutableAttributedString *temp = [[NSMutableAttributedString alloc] initWithString:text attributes:baseAttrs];
        
        NSRegularExpression *italicRegex = [NSRegularExpression regularExpressionWithPattern:@"<em>(.*?)</em>" 
                                                                                     options:0 
                                                                                       error:nil];
        NSArray *matches = [italicRegex matchesInString:text options:0 range:NSMakeRange(0, [text length])];
        
        for (NSTextCheckingResult *match in [matches reverseObjectEnumerator]) {
            NSRange fullRange = [match rangeAtIndex:0];
            NSRange contentRange = [match rangeAtIndex:1];
            NSString *italicText = [text substringWithRange:contentRange];
            
            NSMutableDictionary *italicAttrs = [baseAttrs mutableCopy];
            italicAttrs[NSForegroundColorAttributeName] = [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]; // #ffd93d
            
            [temp replaceCharactersInRange:fullRange withString:italicText];
            [temp setAttributes:italicAttrs range:NSMakeRange(fullRange.location, [italicText length])];
        }
        
        [result appendAttributedString:temp];
        return;
    }
    
    if ([text containsString:@"<mark>"]) {
        // Handle highlight text
        NSMutableAttributedString *temp = [[NSMutableAttributedString alloc] initWithString:text attributes:baseAttrs];
        
        NSRegularExpression *markRegex = [NSRegularExpression regularExpressionWithPattern:@"<mark>(.*?)</mark>" 
                                                                                   options:0 
                                                                                     error:nil];
        NSArray *matches = [markRegex matchesInString:text options:0 range:NSMakeRange(0, [text length])];
        
        for (NSTextCheckingResult *match in [matches reverseObjectEnumerator]) {
            NSRange fullRange = [match rangeAtIndex:0];
            NSRange contentRange = [match rangeAtIndex:1];
            NSString *markText = [text substringWithRange:contentRange];
            
            NSMutableDictionary *markAttrs = [baseAttrs mutableCopy];
            markAttrs[NSBackgroundColorAttributeName] = [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]; // #ffd93d
            markAttrs[NSForegroundColorAttributeName] = [NSColor blackColor];
            
            [temp replaceCharactersInRange:fullRange withString:markText];
            [temp setAttributes:markAttrs range:NSMakeRange(fullRange.location, [markText length])];
        }
        
        [result appendAttributedString:temp];
        return;
    }
    
    if ([text containsString:@"<li>"]) {
        // Handle list items
        text = [text stringByReplacingOccurrencesOfString:@"<li>" withString:@"• "];
        text = [text stringByReplacingOccurrencesOfString:@"</li>" withString:@""];
    }
    
    // Default: plain text with base attributes
    [result appendAttributedString:[[NSAttributedString alloc] initWithString:text attributes:baseAttrs]];
}

// Override events for immediate cursor tracking
- (void)keyDown:(NSEvent *)event {
    if (event.keyCode == 36) { // Enter key
        [self handleEnterKey];
    } else if (event.keyCode == 51) { // Backspace key
        [self handleBackspaceKey:event];
    } else {
        [super keyDown:event];
    }
    
    // Immediate update for cursor movement
    [self performSelector:@selector(updateRendering) withObject:nil afterDelay:0.01];
}

// Override selection change to detect cursor movement
- (void)setSelectedRange:(NSRange)charRange {
    [super setSelectedRange:charRange];
    // Trigger immediate update when selection changes
    [self performSelector:@selector(updateRendering) withObject:nil afterDelay:0.01];
}

// Mouse click detection
- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    // Immediate update after mouse click
    [self performSelector:@selector(updateRendering) withObject:nil afterDelay:0.01];
}

// Arrow key detection
- (void)moveUp:(id)sender {
    [super moveUp:sender];
    [self updateRendering];
}

- (void)moveDown:(id)sender {
    [super moveDown:sender];
    [self updateRendering];
}

- (void)moveLeft:(id)sender {
    [super moveLeft:sender];
    [self updateRendering];
}

- (void)moveRight:(id)sender {
    [super moveRight:sender];
    [self updateRendering];
}

- (void)handleEnterKey {
    // Standard enter behavior, but trigger update
    [self insertNewline:nil];
}

- (void)handleBackspaceKey:(NSEvent *)event {
    // Standard backspace behavior, but trigger update
    [super keyDown:event];
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
        
        [window setTitle:@"C Editor - Native Hybrid"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]]; // #1a1a1a
        
        // Create header
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, frame.size.height - 60, frame.size.width, 60)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor; // #2d2d2d
        
        // Title
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 20, 300, 20)];
        [titleLabel setStringValue:@"C Editor - Native Hybrid"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:16 weight:NSFontWeightSemibold]];
        [headerView addSubview:titleLabel];
        
        // Status
        NSView *statusIndicator = [[NSView alloc] initWithFrame:NSMakeRect(frame.size.width - 200, 25, 8, 8)];
        [statusIndicator setWantsLayer:YES];
        [statusIndicator layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.4 alpha:1.0].CGColor; // #51cf66
        [statusIndicator layer].cornerRadius = 4;
        [headerView addSubview:statusIndicator];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(frame.size.width - 180, 20, 150, 15)];
        [statusLabel setStringValue:@"C Engine Native Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [headerView addSubview:statusLabel];
        
        // Pane header
        NSView *paneHeader = [[NSView alloc] initWithFrame:NSMakeRect(0, frame.size.height - 110, frame.size.width, 50)];
        [paneHeader setWantsLayer:YES];
        [paneHeader layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor; // #2d2d2d
        
        NSTextField *paneTitle = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 600, 20)];
        [paneTitle setStringValue:@"Native Hybrid Editor - Ligne courante en Markdown, autres lignes rendues"];
        [paneTitle setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneTitle setBackgroundColor:[NSColor clearColor]];
        [paneTitle setBordered:NO];
        [paneTitle setEditable:NO];
        [paneTitle setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneHeader addSubview:paneTitle];
        
        // Create scroll view for editor
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(20, 20, frame.size.width - 40, frame.size.height - 150)];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setHasHorizontalScroller:NO];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setBorderType:NSLineBorder];
        [scrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        
        // Create hybrid text view
        HybridTextView *textView = [[HybridTextView alloc] initWithFrame:[scrollView.contentView bounds]];
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
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        
        // Focus text view
        [window makeFirstResponder:textView];
        
        NSLog(@"🚀 Native hybrid editor launched");
        
        [app run];
    }
    return 0;
}