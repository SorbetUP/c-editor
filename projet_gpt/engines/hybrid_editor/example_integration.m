// example_integration.m - Example of using hybrid_editor_core in ElephantNotes
#import <Cocoa/Cocoa.h>
#include "hybrid_editor_core.h"
#include "../editor/editor_abi.h"

@interface ModernHybridTextView : NSTextView {
    NSInteger _currentLineIndex;
    BOOL _isUpdating;
    TextLines* _textLines;
}
@end

@implementation ModernHybridTextView

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        _currentLineIndex = 0;
        _isUpdating = NO;
        _textLines = NULL;
        
        // Configure text view
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]];
        
        // Test content
        [self setString:@"# Modern Hybrid Editor\n\nUsing **hybrid_editor_core** C library:\n- *Italic* detection\n- ==Highlight== support\n- **Bold** formatting\n\n## Features\n- Platform-agnostic C core\n- Reusable across all platforms\n- Optimized markdown parsing"];
        
        // Real-time updates
        [NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(updateDisplay) userInfo:nil repeats:YES];
    }
    return self;
}

- (void)dealloc {
    if (_textLines) {
        hybrid_free_text_lines(_textLines);
    }
}

- (void)updateDisplay {
    if (_isUpdating) return;
    
    NSInteger newCurrentLine = [self getCurrentLineUsingCLib];
    if (newCurrentLine == _currentLineIndex) return;
    
    _isUpdating = YES;
    
    // Store selection
    NSRange oldSelection = [self selectedRange];
    
    // Apply hybrid formatting using C library
    [self applyHybridFormattingUsingCLib:newCurrentLine];
    
    // Restore selection
    [self setSelectedRange:oldSelection];
    
    _currentLineIndex = newCurrentLine;
    _isUpdating = NO;
}

- (NSInteger)getCurrentLineUsingCLib {
    // Update text lines if needed
    if (_textLines) {
        hybrid_free_text_lines(_textLines);
    }
    
    const char* text_cstr = [[self string] UTF8String];
    _textLines = hybrid_parse_text(text_cstr);
    
    if (!_textLines) return 0;
    
    NSRange selectedRange = [self selectedRange];
    return hybrid_get_line_at_cursor(_textLines, (int)selectedRange.location);
}

- (void)applyHybridFormattingUsingCLib:(NSInteger)currentLine {
    if (!_textLines) return;
    
    NSMutableAttributedString *text = [[self textStorage] mutableCopy];
    
    // Process each line
    for (int i = 0; i < _textLines->line_count; i++) {
        LineInfo* lineInfo = &_textLines->lines[i];
        NSRange lineRange = NSMakeRange(lineInfo->char_start, lineInfo->length);
        
        // Get line content
        char* lineContent = hybrid_get_line_content([[self string] UTF8String], i);
        if (!lineContent) continue;
        
        if (i == currentLine) {
            // Current line: show raw markdown
            [self formatAsCurrentLine:text range:lineRange];
        } else {
            // Other lines: use C library to format and hide markup
            [self formatLineUsingCLib:text range:lineRange content:lineContent];
        }
        
        free(lineContent);
    }
    
    [[self textStorage] setAttributedString:text];
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

- (void)formatLineUsingCLib:(NSMutableAttributedString *)text range:(NSRange)range content:(const char*)content {
    // Base styling
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:range];
    
    // Use C library to detect format
    MarkdownFormat format = hybrid_detect_line_format(content);
    
    // Apply detected formatting
    if (format & MD_FORMAT_HEADER1) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:20] ?: [NSFont monospacedSystemFontOfSize:20 weight:NSFontWeightBold]
        } range:range];
        [self hideMarkupUsingCLib:text range:range content:content pattern:"^# "];
        return;
    }
    
    if (format & MD_FORMAT_HEADER2) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0],
            NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:18] ?: [NSFont monospacedSystemFontOfSize:18 weight:NSFontWeightBold]
        } range:range];
        [self hideMarkupUsingCLib:text range:range content:content pattern:"^## "];
        return;
    }
    
    // Use C library for detailed analysis
    LineFormats* formats = hybrid_analyze_markdown_line(content);
    if (formats) {
        [self applyDetailedFormattingUsingCLib:text baseRange:range formats:formats];
        hybrid_free_line_formats(formats);
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

// Override events to trigger updates
- (void)keyDown:(NSEvent *)event {
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

// Example usage in main application
int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize both C libraries
        editor_library_init();
        
        // Configure hybrid editor
        HybridConfig config = hybrid_get_config();
        config.strict_markdown = true;
        config.enable_bold = true;
        config.enable_italic = true;
        config.enable_highlight = true;
        hybrid_set_config(&config);
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1000, 700);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"ElephantNotes with C Library"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create scroll view
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(20, 20, frame.size.width - 40, frame.size.height - 40)];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Create the modern hybrid text view using C library
        ModernHybridTextView *textView = [[ModernHybridTextView alloc] initWithFrame:[scrollView.contentView bounds]];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        [[textView textContainer] setWidthTracksTextView:YES];
        [[textView textContainer] setContainerSize:NSMakeSize([scrollView.contentView bounds].size.width, FLT_MAX)];
        
        [scrollView setDocumentView:textView];
        [[window contentView] addSubview:scrollView];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        [window makeFirstResponder:textView];
        
        [app run];
    }
    return 0;
}