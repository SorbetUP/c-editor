#import <Cocoa/Cocoa.h>
#import "../editor/editor_abi.h"

@interface MarkdownEditor : NSTextView {
    NSInteger currentLine;
    NSTimer *timer;
}
@end

@implementation MarkdownEditor

- (instancetype)initWithFrame:(NSRect)frame {
    self = [super initWithFrame:frame];
    if (self) {
        currentLine = 0;
        
        // Setup
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor whiteColor]];
        
        // Content
        [self setString:@"# Test\n\n**Bold** text\n*Italic* text\n==Highlight== text"];
        
        // Timer
        timer = [NSTimer scheduledTimerWithTimeInterval:0.1 
                                                 target:self 
                                               selector:@selector(update) 
                                               userInfo:nil 
                                                repeats:YES];
    }
    return self;
}

- (void)dealloc {
    [timer invalidate];
}

- (NSInteger)lineFromCursor {
    NSRange sel = [self selectedRange];
    NSString *text = [self string];
    NSInteger line = 0;
    NSInteger pos = 0;
    
    for (NSInteger i = 0; i <= [text length]; i++) {
        if (i == [text length] || [text characterAtIndex:i] == '\n') {
            if (sel.location >= pos && sel.location <= i) {
                return line;
            }
            line++;
            pos = i + 1;
        }
    }
    return line;
}

- (NSArray *)lines {
    return [[self string] componentsSeparatedByString:@"\n"];
}

- (NSRange)rangeForLine:(NSInteger)line {
    NSArray *lines = [self lines];
    if (line < 0 || line >= [lines count]) return NSMakeRange(0, 0);
    
    NSInteger pos = 0;
    for (NSInteger i = 0; i < line; i++) {
        pos += [[lines objectAtIndex:i] length] + 1;
    }
    return NSMakeRange(pos, [[lines objectAtIndex:line] length]);
}

- (void)update {
    NSInteger newLine = [self lineFromCursor];
    if (newLine == currentLine) return;
    
    currentLine = newLine;
    [self render];
}

- (void)render {
    NSMutableAttributedString *text = [[self textStorage] mutableCopy];
    NSArray *lines = [self lines];
    
    // Reset
    [text removeAttribute:NSBackgroundColorAttributeName range:NSMakeRange(0, [text length])];
    [text removeAttribute:NSForegroundColorAttributeName range:NSMakeRange(0, [text length])];
    [text removeAttribute:NSFontAttributeName range:NSMakeRange(0, [text length])];
    
    // Base
    NSDictionary *base = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:base range:NSMakeRange(0, [text length])];
    
    // Lines
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSRange range = [self rangeForLine:i];
        if (range.length == 0) continue;
        
        NSString *content = [lines objectAtIndex:i];
        
        if (i == currentLine) {
            // Current line - blue background
            [text addAttribute:NSBackgroundColorAttributeName 
                         value:[NSColor colorWithRed:0.3 green:0.4 blue:1.0 alpha:0.3] 
                         range:range];
        } else {
            // Other lines - render markdown
            [self applyMarkdown:text range:range content:content];
        }
    }
    
    NSRange oldSel = [self selectedRange];
    [[self textStorage] setAttributedString:text];
    [self setSelectedRange:oldSel];
}

- (void)applyMarkdown:(NSMutableAttributedString *)text range:(NSRange)range content:(NSString *)content {
    // Headers
    if ([content hasPrefix:@"# "]) {
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.3 green:0.4 blue:1.0 alpha:1.0],
            NSFontAttributeName: [NSFont boldSystemFontOfSize:20]
        } range:range];
        return;
    }
    
    // Bold **text** - GREEN
    NSRegularExpression *bold = [NSRegularExpression regularExpressionWithPattern:@"\\*\\*([^*]+)\\*\\*" options:0 error:nil];
    NSArray *boldMatches = [bold matchesInString:content options:0 range:NSMakeRange(0, [content length])];
    for (NSTextCheckingResult *match in boldMatches) {
        NSRange mr = NSMakeRange(range.location + match.range.location, match.range.length);
        [text addAttributes:@{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.0 green:0.8 blue:0.0 alpha:1.0], // GREEN
            NSFontAttributeName: [NSFont boldSystemFontOfSize:14]
        } range:mr];
    }
    
    // Italic *text* - YELLOW  
    NSRegularExpression *italic = [NSRegularExpression regularExpressionWithPattern:@"(?<!\\*)\\*([^*]+)\\*(?!\\*)" options:0 error:nil];
    NSArray *italicMatches = [italic matchesInString:content options:0 range:NSMakeRange(0, [content length])];
    for (NSTextCheckingResult *match in italicMatches) {
        NSRange mr = NSMakeRange(range.location + match.range.location, match.range.length);
        [text addAttribute:NSForegroundColorAttributeName 
                     value:[NSColor colorWithRed:1.0 green:1.0 blue:0.0 alpha:1.0] // YELLOW
                     range:mr];
    }
    
    // Highlight ==text== - ORANGE
    NSRegularExpression *highlight = [NSRegularExpression regularExpressionWithPattern:@"==([^=]+)==" options:0 error:nil];
    NSArray *highlightMatches = [highlight matchesInString:content options:0 range:NSMakeRange(0, [content length])];
    for (NSTextCheckingResult *match in highlightMatches) {
        NSRange mr = NSMakeRange(range.location + match.range.location, match.range.length);
        [text addAttributes:@{
            NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.5 blue:0.0 alpha:0.8], // ORANGE
            NSForegroundColorAttributeName: [NSColor blackColor]
        } range:mr];
    }
}

- (void)keyDown:(NSEvent *)event {
    [super keyDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self update];
    });
}

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self update];
    });
}

@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        editor_library_init();
        
        NSRect frame = NSMakeRect(100, 100, 800, 600);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"Editor"];
        [window setBackgroundColor:[NSColor colorWithRed:0.1 green:0.1 blue:0.1 alpha:1.0]];
        
        NSScrollView *scroll = [[NSScrollView alloc] initWithFrame:NSMakeRect(20, 20, frame.size.width - 40, frame.size.height - 40)];
        [scroll setHasVerticalScroller:YES];
        [scroll setAutohidesScrollers:YES];
        [scroll setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        
        MarkdownEditor *editor = [[MarkdownEditor alloc] initWithFrame:[scroll.contentView bounds]];
        [editor setVerticallyResizable:YES];
        [editor setHorizontallyResizable:NO];
        [[editor textContainer] setWidthTracksTextView:YES];
        [[editor textContainer] setContainerSize:NSMakeSize([scroll.contentView bounds].size.width, FLT_MAX)];
        
        [scroll setDocumentView:editor];
        [[window contentView] addSubview:scroll];
        
        [window makeKeyAndOrderFront:nil];
        [window center];
        [window makeFirstResponder:editor];
        
        [app run];
    }
    return 0;
}