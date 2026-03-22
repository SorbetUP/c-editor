#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import C engines
#import "../editor/editor_abi.h"
#import "../markdown/markdown.h"

// Global variables
NSScrollView *g_editorScrollView = nil;
NSTextView *g_editorTextView = nil;
NSMutableArray *g_editorLines = nil;
NSMutableArray *g_lineContents = nil;
NSInteger g_currentLineIndex = -1;
NSTimer *g_updateTimer = nil;

@interface HybridEditorLine : NSTextView
@property (nonatomic, assign) NSInteger lineIndex;
@property (nonatomic, assign) BOOL isCurrentLine;
@property (nonatomic, strong) NSString *markdownContent;
@end

@implementation HybridEditorLine

- (instancetype)initWithFrame:(NSRect)frame lineIndex:(NSInteger)index {
    self = [super initWithFrame:frame];
    if (self) {
        self.lineIndex = index;
        self.isCurrentLine = NO;
        self.markdownContent = @"";
        
        // Configure like web editor
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setEditable:YES];
        [self setSelectable:YES];
        [self setVerticallyResizable:NO];
        [self setHorizontallyResizable:NO];
        [self setTextContainerInset:NSMakeSize(8, 4)];
        
        // Single line
        [[self textContainer] setWidthTracksTextView:YES];
        [[self textContainer] setContainerSize:NSMakeSize(FLT_MAX, 1.8 * 14)]; // line-height: 1.8
        
        // Configure as single line text view
        
        // Border
        [self setWantsLayer:YES];
        [self layer].borderColor = [NSColor clearColor].CGColor;
        [self layer].borderWidth = 0;
        [self layer].cornerRadius = 3;
    }
    return self;
}

- (void)setAsCurrentLine:(BOOL)current {
    self.isCurrentLine = current;
    
    if (current) {
        // Current line: markdown mode with highlighting
        [self setBackgroundColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.1]]; // rgba(76, 110, 245, 0.1)
        [self layer].borderColor = [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0].CGColor; // #4c6ef5
        [self layer].borderWidth = 3;
        [self showMarkdown];
    } else {
        // Rendered line
        [self setBackgroundColor:[NSColor colorWithRed:1.0 green:1.0 blue:1.0 alpha:0.02]]; // rgba(255, 255, 255, 0.02)
        [self layer].borderColor = [NSColor clearColor].CGColor;
        [self layer].borderWidth = 0;
        [self renderAsHTML];
    }
}

- (void)showMarkdown {
    [self setString:self.markdownContent];
    [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
}

- (void)renderAsHTML {
    if (!self.markdownContent || [self.markdownContent length] == 0) {
        [self setString:@""];
        return;
    }
    
    // Use C engine for rendering
    const char *html_output = editor_markdown_to_html([self.markdownContent UTF8String]);
    if (html_output) {
        NSString *htmlString = [NSString stringWithUTF8String:html_output];
        
        // Convert HTML to attributed string
        NSData *htmlData = [htmlString dataUsingEncoding:NSUTF8StringEncoding];
        if (htmlData) {
            NSDictionary *options = @{
                NSDocumentTypeDocumentAttribute: NSHTMLTextDocumentType,
                NSCharacterEncodingDocumentAttribute: @(NSUTF8StringEncoding)
            };
            
            NSError *error = nil;
            NSAttributedString *attributedString = [[NSAttributedString alloc] 
                initWithData:htmlData 
                options:options 
                documentAttributes:nil 
                error:&error];
            
            if (attributedString && !error) {
                // Apply dark theme colors to the attributed string
                NSMutableAttributedString *darkAttributed = [attributedString mutableCopy];
                [darkAttributed addAttribute:NSForegroundColorAttributeName 
                                       value:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0] 
                                       range:NSMakeRange(0, [darkAttributed length])];
                [darkAttributed addAttribute:NSFontAttributeName 
                                       value:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
                                       range:NSMakeRange(0, [darkAttributed length])];
                
                [[self textStorage] setAttributedString:darkAttributed];
            } else {
                // Fallback to plain text
                [self setString:self.markdownContent];
            }
        }
    } else {
        // C engine failed, show plain text
        [self setString:self.markdownContent];
    }
}

- (void)updateContent:(NSString *)content {
    self.markdownContent = content ?: @"";
    if (self.isCurrentLine) {
        [self showMarkdown];
    } else {
        [self renderAsHTML];
    }
}

@end

// Update line states
void updateLineStates() {
    NSTextView *focusedView = nil;
    NSInteger newCurrentIndex = -1;
    
    // Find which line has focus
    NSWindow *keyWindow = [NSApp keyWindow];
    if (keyWindow) {
        NSResponder *firstResponder = [keyWindow firstResponder];
        if ([firstResponder isKindOfClass:[HybridEditorLine class]]) {
            HybridEditorLine *line = (HybridEditorLine *)firstResponder;
            newCurrentIndex = line.lineIndex;
            focusedView = line;
        }
    }
    
    if (newCurrentIndex == g_currentLineIndex) return; // No change
    
    NSLog(@"Line changed from %ld to %ld", g_currentLineIndex, newCurrentIndex);
    
    // Update previous current line to rendered
    if (g_currentLineIndex >= 0 && g_currentLineIndex < [g_editorLines count]) {
        HybridEditorLine *prevLine = g_editorLines[g_currentLineIndex];
        // Save content from the markdown view
        prevLine.markdownContent = [prevLine string];
        [prevLine setAsCurrentLine:NO];
    }
    
    // Update new current line to markdown
    if (newCurrentIndex >= 0 && newCurrentIndex < [g_editorLines count]) {
        HybridEditorLine *newLine = g_editorLines[newCurrentIndex];
        [newLine setAsCurrentLine:YES];
        g_currentLineIndex = newCurrentIndex;
    }
}

// Handle keyboard events
@interface HybridEditorDelegate : NSObject <NSTextViewDelegate>
@end

@implementation HybridEditorDelegate

- (BOOL)textView:(NSTextView *)textView doCommandBySelector:(SEL)commandSelector {
    if (commandSelector == @selector(insertNewline:)) {
        // Handle Enter key
        NSLog(@"Enter key pressed in line %ld", ((HybridEditorLine *)textView).lineIndex);
        
        HybridEditorLine *currentLine = (HybridEditorLine *)textView;
        NSString *currentContent = [currentLine string];
        NSRange selectedRange = [currentLine selectedRange];
        
        // Split content at cursor
        NSString *beforeCursor = [currentContent substringToIndex:selectedRange.location];
        NSString *afterCursor = [currentContent substringFromIndex:selectedRange.location];
        
        // Update current line with content before cursor
        currentLine.markdownContent = beforeCursor;
        [currentLine setString:beforeCursor];
        
        // Create new line
        NSRect newFrame = currentLine.frame;
        newFrame.origin.y -= (newFrame.size.height + 5); // Move down
        
        HybridEditorLine *newLine = [[HybridEditorLine alloc] 
            initWithFrame:newFrame 
            lineIndex:currentLine.lineIndex + 1];
        newLine.markdownContent = afterCursor;
        
        // Insert into arrays
        [g_editorLines insertObject:newLine atIndex:currentLine.lineIndex + 1];
        [g_lineContents insertObject:afterCursor atIndex:currentLine.lineIndex + 1];
        
        // Update line indices
        for (NSInteger i = currentLine.lineIndex + 1; i < [g_editorLines count]; i++) {
            HybridEditorLine *line = g_editorLines[i];
            line.lineIndex = i;
        }
        
        // Add to scroll view
        [g_editorScrollView.documentView addSubview:newLine];
        
        // Focus new line
        dispatch_async(dispatch_get_main_queue(), ^{
            [[newLine window] makeFirstResponder:newLine];
            [newLine setSelectedRange:NSMakeRange(0, 0)];
        });
        
        return YES; // Handled
    } else if (commandSelector == @selector(deleteBackward:)) {
        // Handle Backspace key
        HybridEditorLine *currentLine = (HybridEditorLine *)textView;
        NSRange selectedRange = [currentLine selectedRange];
        
        if (selectedRange.location == 0 && currentLine.lineIndex > 0) {
            // At start of line, merge with previous line
            NSLog(@"Backspace at line start, merging with previous line");
            
            HybridEditorLine *prevLine = g_editorLines[currentLine.lineIndex - 1];
            NSString *mergedContent = [prevLine.markdownContent stringByAppendingString:currentLine.markdownContent];
            NSUInteger cursorPos = [prevLine.markdownContent length];
            
            // Update previous line
            prevLine.markdownContent = mergedContent;
            if (prevLine.isCurrentLine) {
                [prevLine setString:mergedContent];
            }
            
            // Remove current line
            [currentLine removeFromSuperview];
            [g_editorLines removeObjectAtIndex:currentLine.lineIndex];
            [g_lineContents removeObjectAtIndex:currentLine.lineIndex];
            
            // Update line indices
            for (NSInteger i = currentLine.lineIndex; i < [g_editorLines count]; i++) {
                HybridEditorLine *line = g_editorLines[i];
                line.lineIndex = i;
            }
            
            // Focus previous line and position cursor
            dispatch_async(dispatch_get_main_queue(), ^{
                [[prevLine window] makeFirstResponder:prevLine];
                [prevLine setSelectedRange:NSMakeRange(cursorPos, 0)];
            });
            
            return YES; // Handled
        }
    }
    
    return NO; // Not handled
}

- (void)textDidChange:(NSNotification *)notification {
    HybridEditorLine *line = notification.object;
    if (line.isCurrentLine) {
        // Update stored content for current line
        line.markdownContent = [line string];
        g_lineContents[line.lineIndex] = line.markdownContent;
    }
}

@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C engines
        editor_library_init();
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1000, 700);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Editor - Éditeur Hybride"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]]; // #1a1a1a
        
        // Create header
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, frame.size.height - 60, frame.size.width, 60)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor; // #2d2d2d
        
        // Title
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 20, 300, 20)];
        [titleLabel setStringValue:@"C Editor - Éditeur Hybride"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:16 weight:NSFontWeightSemibold]];
        [headerView addSubview:titleLabel];
        
        // Status indicator
        NSView *statusIndicator = [[NSView alloc] initWithFrame:NSMakeRect(frame.size.width - 200, 25, 8, 8)];
        [statusIndicator setWantsLayer:YES];
        [statusIndicator layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.4 alpha:1.0].CGColor; // #51cf66
        [statusIndicator layer].cornerRadius = 4;
        [headerView addSubview:statusIndicator];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(frame.size.width - 180, 20, 100, 15)];
        [statusLabel setStringValue:@"C Engine Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [headerView addSubview:statusLabel];
        
        // Import/Export buttons
        NSButton *importBtn = [[NSButton alloc] initWithFrame:NSMakeRect(frame.size.width - 350, 20, 80, 24)];
        [importBtn setTitle:@"📂 Import"];
        [importBtn setBezelStyle:NSBezelStyleRounded];
        [importBtn setFont:[NSFont systemFontOfSize:12]];
        [headerView addSubview:importBtn];
        
        NSButton *exportBtn = [[NSButton alloc] initWithFrame:NSMakeRect(frame.size.width - 260, 20, 80, 24)];
        [exportBtn setTitle:@"📄 Export"];
        [exportBtn setBezelStyle:NSBezelStyleRounded];
        [exportBtn setFont:[NSFont systemFontOfSize:12]];
        [headerView addSubview:exportBtn];
        
        // Main content area
        NSView *contentView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, frame.size.width, frame.size.height - 60)];
        [contentView setWantsLayer:YES];
        [contentView layer].backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor;
        
        // Pane header
        NSView *paneHeader = [[NSView alloc] initWithFrame:NSMakeRect(0, contentView.frame.size.height - 50, contentView.frame.size.width, 50)];
        [paneHeader setWantsLayer:YES];
        [paneHeader layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        
        NSTextField *paneTitle = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 600, 20)];
        [paneTitle setStringValue:@"Éditeur Hybride - Ligne courante en Markdown, autres lignes rendues"];
        [paneTitle setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneTitle setBackgroundColor:[NSColor clearColor]];
        [paneTitle setBordered:NO];
        [paneTitle setEditable:NO];
        [paneTitle setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneHeader addSubview:paneTitle];
        
        // Editor container
        NSRect editorRect = NSMakeRect(20, 20, contentView.frame.size.width - 40, contentView.frame.size.height - 90);
        g_editorScrollView = [[NSScrollView alloc] initWithFrame:editorRect];
        [g_editorScrollView setHasVerticalScroller:YES];
        [g_editorScrollView setHasHorizontalScroller:NO];
        [g_editorScrollView setAutohidesScrollers:YES];
        [g_editorScrollView setBorderType:NSLineBorder];
        [g_editorScrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        
        // Document view for lines
        NSView *documentView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, editorRect.size.width - 20, 2000)];
        [g_editorScrollView setDocumentView:documentView];
        
        // Initialize arrays
        g_editorLines = [[NSMutableArray alloc] init];
        g_lineContents = [[NSMutableArray alloc] init];
        
        // Create initial lines (like in web version)
        NSArray *initialContent = @[
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
        
        HybridEditorDelegate *delegate = [[HybridEditorDelegate alloc] init];
        
        CGFloat yPosition = documentView.frame.size.height - 30;
        for (NSInteger i = 0; i < [initialContent count]; i++) {
            NSRect lineFrame = NSMakeRect(10, yPosition - (i * 30), editorRect.size.width - 40, 25);
            HybridEditorLine *line = [[HybridEditorLine alloc] initWithFrame:lineFrame lineIndex:i];
            line.markdownContent = initialContent[i];
            [line setDelegate:delegate];
            
            [g_editorLines addObject:line];
            [g_lineContents addObject:initialContent[i]];
            [documentView addSubview:line];
        }
        
        // Set first line as current
        g_currentLineIndex = 0;
        HybridEditorLine *firstLine = g_editorLines[0];
        [firstLine setAsCurrentLine:YES];
        
        // Render other lines as HTML
        for (NSInteger i = 1; i < [g_editorLines count]; i++) {
            HybridEditorLine *line = g_editorLines[i];
            [line setAsCurrentLine:NO];
        }
        
        [contentView addSubview:paneHeader];
        [contentView addSubview:g_editorScrollView];
        
        // Add views to window
        [[window contentView] addSubview:headerView];
        [[window contentView] addSubview:contentView];
        
        // Set up timer for updating line states
        g_updateTimer = [NSTimer scheduledTimerWithTimeInterval:0.1
                                                         target:[NSValue valueWithPointer:&updateLineStates]
                                                       selector:@selector(invoke)
                                                       userInfo:nil
                                                        repeats:YES];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        
        // Focus first line
        dispatch_async(dispatch_get_main_queue(), ^{
            [window makeFirstResponder:firstLine];
        });
        
        [app run];
    }
    return 0;
}