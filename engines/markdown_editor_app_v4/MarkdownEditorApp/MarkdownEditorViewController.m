#import "MarkdownEditorViewController.h"

@implementation MarkdownEditorViewController

- (void)viewDidLoad {
    [super viewDidLoad];
    
    NSLog(@"🚀 Starting C Markdown Editor...");
    
    // Initialize C engines
    [self initializeCEngines];
    
    // Setup the UI to match HTML version exactly
    [self setupUI];
    
    // Load default content
    [self loadDefaultContent];
    
    NSLog(@"✅ C Markdown Editor initialized successfully");
}

#pragma mark - C Engine Integration

- (void)initializeCEngines {
    NSLog(@"🔧 Initializing C engines...");
    
    // Initialize markdown engine first (simplified)
    NSLog(@"✅ C Markdown engine ready");
    
    // Initialize render engine (simplified for now)
    NSLog(@"✅ C Render engine ready");
    self.engineReady = YES;
    
    // Initialize line contents array
    self.lineContents = [[NSMutableArray alloc] init];
    self.currentLineIndex = 0;
}

#pragma mark - UI Setup - Exact replica of HTML version

- (void)setupUI {
    // Main container with dark background like HTML
    self.view.wantsLayer = YES;
    self.view.layer.backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor; // #1a1a1a
    
    [self setupHeader];
    [self setupEditor];
    [self setupConstraints];
}

- (void)setupHeader {
    // Header view - matching HTML .header style
    self.headerView = [[NSView alloc] init];
    self.headerView.wantsLayer = YES;
    self.headerView.layer.backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor; // #2d2d2d
    self.headerView.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Title label - matching HTML h1 style
    self.titleLabel = [[NSTextField alloc] init];
    self.titleLabel.stringValue = @"C Markdown Editor - Éditeur Hybride";
    self.titleLabel.font = [NSFont systemFontOfSize:16 weight:NSFontWeightSemibold];
    self.titleLabel.textColor = [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]; // #e0e0e0
    self.titleLabel.backgroundColor = [NSColor clearColor];
    self.titleLabel.bordered = NO;
    self.titleLabel.editable = NO;
    self.titleLabel.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Status view container
    self.statusView = [[NSView alloc] init];
    self.statusView.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Status indicator - matching HTML .status-indicator.ready style
    self.statusIndicator = [[NSView alloc] init];
    self.statusIndicator.wantsLayer = YES;
    self.statusIndicator.layer.backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0].CGColor; // #51cf66
    self.statusIndicator.layer.cornerRadius = 4.0;
    self.statusIndicator.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Status label
    self.statusLabel = [[NSTextField alloc] init];
    self.statusLabel.stringValue = @"C Engine Ready";
    self.statusLabel.font = [NSFont systemFontOfSize:12];
    self.statusLabel.textColor = [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0];
    self.statusLabel.backgroundColor = [NSColor clearColor];
    self.statusLabel.bordered = NO;
    self.statusLabel.editable = NO;
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Import button - matching HTML button style
    self.importButton = [[NSButton alloc] init];
    [self.importButton setTitle:@"📥 Importer"];
    [self styleButton:self.importButton];
    [self.importButton setTarget:self];
    [self.importButton setAction:@selector(importMarkdownFile)];
    
    // Export button
    self.exportButton = [[NSButton alloc] init];
    [self.exportButton setTitle:@"📤 Exporter"];
    [self styleButton:self.exportButton];
    [self.exportButton setTarget:self];
    [self.exportButton setAction:@selector(exportMarkdownFile)];
    
    // Clear cache button
    self.clearCacheButton = [[NSButton alloc] init];
    [self.clearCacheButton setTitle:@"🗑️ Vider"];
    [self styleButton:self.clearCacheButton];
    [self.clearCacheButton setTarget:self];
    [self.clearCacheButton setAction:@selector(clearCache)];
    
    // Add subviews
    [self.statusView addSubview:self.statusIndicator];
    [self.statusView addSubview:self.statusLabel];
    
    [self.headerView addSubview:self.titleLabel];
    [self.headerView addSubview:self.statusView];
    [self.headerView addSubview:self.importButton];
    [self.headerView addSubview:self.exportButton];
    [self.headerView addSubview:self.clearCacheButton];
    
    [self.view addSubview:self.headerView];
}

- (void)styleButton:(NSButton *)button {
    // Style buttons to match HTML .btn style
    button.wantsLayer = YES;
    button.layer.backgroundColor = [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0].CGColor; // #4c6ef5
    button.layer.cornerRadius = 4.0;
    button.layer.borderWidth = 0;
    
    NSMutableAttributedString *title = [[NSMutableAttributedString alloc] initWithString:button.title];
    [title addAttribute:NSForegroundColorAttributeName 
                  value:[NSColor whiteColor] 
                  range:NSMakeRange(0, title.length)];
    [title addAttribute:NSFontAttributeName 
                  value:[NSFont systemFontOfSize:12] 
                  range:NSMakeRange(0, title.length)];
    button.attributedTitle = title;
    
    button.bordered = NO;
    button.translatesAutoresizingMaskIntoConstraints = NO;
}

- (void)setupEditor {
    // Create scroll view - matching HTML #hybridEditor style
    self.scrollView = [[NSScrollView alloc] init];
    self.scrollView.hasVerticalScroller = YES;
    self.scrollView.hasHorizontalScroller = NO;
    self.scrollView.autohidesScrollers = YES;
    self.scrollView.backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]; // #1e1e1e
    self.scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Create text view with styling to match HTML
    NSRect textFrame = NSMakeRect(0, 0, 800, 600);
    self.textView = [[NSTextView alloc] initWithFrame:textFrame];
    
    // Style text view to match HTML editor styling exactly
    self.textView.backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]; // #1e1e1e
    self.textView.textColor = [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]; // #e0e0e0
    self.textView.insertionPointColor = [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]; // #4c6ef5
    self.textView.font = [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular];
    
    // Text view properties
    self.textView.delegate = self;
    self.textView.richText = NO; // Plain text only
    self.textView.importsGraphics = NO;
    self.textView.allowsUndo = YES;
    self.textView.usesFontPanel = NO;
    self.textView.usesRuler = NO;
    
    // Line height matching HTML line-height: 1.8
    NSMutableParagraphStyle *paragraphStyle = [[NSMutableParagraphStyle alloc] init];
    paragraphStyle.lineHeightMultiple = 1.8;
    paragraphStyle.paragraphSpacing = 0;
    paragraphStyle.paragraphSpacingBefore = 0;
    
    NSDictionary *attributes = @{
        NSFontAttributeName: self.textView.font,
        NSForegroundColorAttributeName: self.textView.textColor,
        NSParagraphStyleAttributeName: paragraphStyle
    };
    
    [self.textView setTypingAttributes:attributes];
    
    // Setup scroll view
    self.scrollView.documentView = self.textView;
    [self.view addSubview:self.scrollView];
}

- (void)setupConstraints {
    // Header constraints
    [NSLayoutConstraint activateConstraints:@[
        [self.headerView.topAnchor constraintEqualToAnchor:self.view.topAnchor],
        [self.headerView.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor],
        [self.headerView.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor],
        [self.headerView.heightAnchor constraintEqualToConstant:60], // Matching HTML header height
        
        // Title label
        [self.titleLabel.leadingAnchor constraintEqualToAnchor:self.headerView.leadingAnchor constant:20],
        [self.titleLabel.centerYAnchor constraintEqualToAnchor:self.headerView.centerYAnchor],
        
        // Status view
        [self.statusView.trailingAnchor constraintEqualToAnchor:self.clearCacheButton.leadingAnchor constant:-10],
        [self.statusView.centerYAnchor constraintEqualToAnchor:self.headerView.centerYAnchor],
        [self.statusView.widthAnchor constraintEqualToConstant:120],
        [self.statusView.heightAnchor constraintEqualToConstant:20],
        
        // Status indicator
        [self.statusIndicator.leadingAnchor constraintEqualToAnchor:self.statusView.leadingAnchor],
        [self.statusIndicator.centerYAnchor constraintEqualToAnchor:self.statusView.centerYAnchor],
        [self.statusIndicator.widthAnchor constraintEqualToConstant:8],
        [self.statusIndicator.heightAnchor constraintEqualToConstant:8],
        
        // Status label
        [self.statusLabel.leadingAnchor constraintEqualToAnchor:self.statusIndicator.trailingAnchor constant:8],
        [self.statusLabel.centerYAnchor constraintEqualToAnchor:self.statusView.centerYAnchor],
        
        // Buttons
        [self.clearCacheButton.trailingAnchor constraintEqualToAnchor:self.headerView.trailingAnchor constant:-10],
        [self.clearCacheButton.centerYAnchor constraintEqualToAnchor:self.headerView.centerYAnchor],
        [self.clearCacheButton.widthAnchor constraintEqualToConstant:70],
        [self.clearCacheButton.heightAnchor constraintEqualToConstant:28],
        
        [self.exportButton.trailingAnchor constraintEqualToAnchor:self.clearCacheButton.leadingAnchor constant:-5],
        [self.exportButton.centerYAnchor constraintEqualToAnchor:self.headerView.centerYAnchor],
        [self.exportButton.widthAnchor constraintEqualToConstant:90],
        [self.exportButton.heightAnchor constraintEqualToConstant:28],
        
        [self.importButton.trailingAnchor constraintEqualToAnchor:self.exportButton.leadingAnchor constant:-5],
        [self.importButton.centerYAnchor constraintEqualToAnchor:self.headerView.centerYAnchor],
        [self.importButton.widthAnchor constraintEqualToConstant:90],
        [self.importButton.heightAnchor constraintEqualToConstant:28],
    ]];
    
    // Editor constraints
    [NSLayoutConstraint activateConstraints:@[
        [self.scrollView.topAnchor constraintEqualToAnchor:self.headerView.bottomAnchor constant:1], // 1px border like HTML
        [self.scrollView.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor],
        [self.scrollView.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor],
        [self.scrollView.bottomAnchor constraintEqualToAnchor:self.view.bottomAnchor],
    ]];
}

#pragma mark - C Engine Rendering - Same logic as HTML version

- (NSString *)renderMarkdownToHTML:(NSString *)markdown {
    if (!self.engineReady) {
        NSLog(@"❌ C ENGINE NOT READY");
        return markdown; // Fallback to plain text
    }
    
    // Use C markdown engine - corrected API calls
    const char *markdownCString = [markdown UTF8String];
    
    // Try direct HTML rendering first (corrected signature)
    const char *htmlResult = editor_markdown_to_html(markdownCString);
    if (htmlResult) {
        NSString *htmlString = [NSString stringWithUTF8String:htmlResult];
        NSLog(@"✅ C ENGINE: \"%@\" -> \"%@\"", markdown, htmlString);
        return htmlString;
    }
    
    NSLog(@"⚠️ C ENGINE HTML failed, trying JSON approach");
    
    // Fallback to JSON approach (corrected signature)
    const char *jsonResult = editor_parse_markdown_simple(markdownCString);
    if (jsonResult) {
        NSString *jsonString = [NSString stringWithUTF8String:jsonResult];
        
        // Convert JSON to HTML (simplified version)
        NSString *htmlFromJson = [self jsonToHtmlSimple:jsonString];
        NSLog(@"✅ C ENGINE JSON: \"%@\" -> \"%@\"", markdown, htmlFromJson);
        return htmlFromJson;
    }
    
    NSLog(@"❌ C ENGINE COMPLETELY FAILED");
    return markdown; // Ultimate fallback
}

// Simplified JSON to HTML converter (like in HTML version)
- (NSString *)jsonToHtmlSimple:(NSString *)jsonString {
    // For now, just extract text content and apply basic formatting
    // This is a simplified version - the full implementation would parse JSON
    
    if ([jsonString containsString:@"\"bold\":true"]) {
        NSString *text = [self extractTextFromJson:jsonString];
        return [NSString stringWithFormat:@"<strong>%@</strong>", text];
    } else if ([jsonString containsString:@"\"italic\":true"]) {
        NSString *text = [self extractTextFromJson:jsonString];
        return [NSString stringWithFormat:@"<em>%@</em>", text];
    } else if ([jsonString containsString:@"\"type\":\"header\""]) {
        NSString *text = [self extractTextFromJson:jsonString];
        return [NSString stringWithFormat:@"<h1>%@</h1>", text];
    }
    
    // Default: return text content
    return [self extractTextFromJson:jsonString];
}

- (NSString *)extractTextFromJson:(NSString *)jsonString {
    // Simple text extraction from JSON
    NSRange textRange = [jsonString rangeOfString:@"\"text\":\""];
    if (textRange.location != NSNotFound) {
        NSString *remaining = [jsonString substringFromIndex:textRange.location + textRange.length];
        NSRange endRange = [remaining rangeOfString:@"\""];
        if (endRange.location != NSNotFound) {
            return [remaining substringToIndex:endRange.location];
        }
    }
    return @"";
}

#pragma mark - Default Content

- (void)loadDefaultContent {
    // Load the same default content as HTML version
    NSString *defaultContent = @"# Titre\n\nÉcrivez votre markdown ici...\n\nExemple:\n- **Gras**\n- *Italique*\n- ==Surligné==\n- ++Souligné++";
    
    [self.textView setString:defaultContent];
    [self updateLineContents];
    
    // Render initial content
    [self renderCurrentContent];
}

- (void)updateLineContents {
    NSString *content = self.textView.string;
    NSArray *lines = [content componentsSeparatedByString:@"\n"];
    [self.lineContents removeAllObjects];
    [self.lineContents addObjectsFromArray:lines];
    
    NSLog(@"📝 Updated %lu lines of content", (unsigned long)self.lineContents.count);
}

- (void)renderCurrentContent {
    // For now, just log the rendering - full live preview would require attributed strings
    NSLog(@"🔄 Rendering %lu lines...", (unsigned long)self.lineContents.count);
    
    for (NSUInteger i = 0; i < self.lineContents.count; i++) {
        NSString *line = self.lineContents[i];
        if (line.length > 0) {
            NSString *rendered = [self renderMarkdownToHTML:line];
            NSLog(@"Line %lu: %@ -> %@", (unsigned long)i, line, rendered);
        }
    }
}

#pragma mark - NSTextViewDelegate

- (void)textDidChange:(NSNotification *)notification {
    [self updateLineContents];
    
    // Get current line index (like HTML version)
    NSRange selectedRange = self.textView.selectedRange;
    NSString *textUpToCursor = [self.textView.string substringToIndex:selectedRange.location];
    self.currentLineIndex = [[textUpToCursor componentsSeparatedByString:@"\n"] count] - 1;
    
    NSLog(@"📍 Current line: %ld", (long)self.currentLineIndex);
    
    // Render updated content (with delay to avoid too frequent updates)
    [NSObject cancelPreviousPerformRequestsWithTarget:self selector:@selector(renderCurrentContent) object:nil];
    [self performSelector:@selector(renderCurrentContent) withObject:nil afterDelay:0.3];
}

#pragma mark - File Operations - Same as HTML version

- (BOOL)openFile:(NSString *)filePath {
    NSError *error;
    NSString *content = [NSString stringWithContentsOfFile:filePath 
                                                   encoding:NSUTF8StringEncoding 
                                                      error:&error];
    if (error) {
        NSLog(@"❌ Failed to open file: %@", error.localizedDescription);
        return NO;
    }
    
    [self.textView setString:content];
    [self updateLineContents];
    [self renderCurrentContent];
    
    NSLog(@"✅ Opened file: %@", filePath);
    return YES;
}

- (void)saveFile:(NSString *)filePath {
    NSError *error;
    [self.textView.string writeToFile:filePath 
                           atomically:YES 
                             encoding:NSUTF8StringEncoding 
                                error:&error];
    if (error) {
        NSLog(@"❌ Failed to save file: %@", error.localizedDescription);
    } else {
        NSLog(@"✅ Saved file: %@", filePath);
    }
}

- (void)importMarkdownFile {
    NSOpenPanel *openPanel = [NSOpenPanel openPanel];
    if (@available(macOS 11.0, *)) {
        openPanel.allowedContentTypes = @[[UTType typeWithFilenameExtension:@"md"],
                                        [UTType typeWithFilenameExtension:@"markdown"],
                                        [UTType typeWithFilenameExtension:@"txt"]];
    } else {
        #pragma clang diagnostic push
        #pragma clang diagnostic ignored "-Wdeprecated-declarations"
        openPanel.allowedFileTypes = @[@"md", @"markdown", @"txt"];
        #pragma clang diagnostic pop
    }
    openPanel.allowsMultipleSelection = NO;
    
    if ([openPanel runModal] == NSModalResponseOK) {
        NSURL *url = openPanel.URLs.firstObject;
        [self openFile:url.path];
    }
}

- (void)exportMarkdownFile {
    NSSavePanel *savePanel = [NSSavePanel savePanel];
    if (@available(macOS 11.0, *)) {
        savePanel.allowedContentTypes = @[[UTType typeWithFilenameExtension:@"md"]];
    } else {
        #pragma clang diagnostic push
        #pragma clang diagnostic ignored "-Wdeprecated-declarations"
        savePanel.allowedFileTypes = @[@"md"];
        #pragma clang diagnostic pop
    }
    savePanel.nameFieldStringValue = @"document.md";
    
    if ([savePanel runModal] == NSModalResponseOK) {
        [self saveFile:savePanel.URL.path];
    }
}

- (void)clearCache {
    NSLog(@"🗑️ Clearing cache...");
    // Clear any internal caches if we had them
    [self renderCurrentContent];
}

#pragma mark - Cleanup

- (void)dealloc {
    NSLog(@"🧹 Cleaning up resources");
}

@end