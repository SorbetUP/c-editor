#import "SimpleViewController.h"

@implementation SimpleViewController

- (void)viewDidLoad {
    [super viewDidLoad];
    
    NSLog(@"🎯 SimpleViewController loading...");
    
    // Create a dark background view
    self.view.wantsLayer = YES;
    self.view.layer.backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor;
    
    // Create scroll view
    self.scrollView = [[NSScrollView alloc] init];
    self.scrollView.hasVerticalScroller = YES;
    self.scrollView.hasHorizontalScroller = NO;
    self.scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    
    // Create text view
    self.textView = [[NSTextView alloc] init];
    self.textView.backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0];
    self.textView.textColor = [NSColor whiteColor];
    self.textView.font = [NSFont fontWithName:@"Monaco" size:14];
    self.textView.string = @"# Test\n\nCeci est un **test** de l'éditeur markdown.\n\n- Item 1\n- Item 2\n- *Italique*";
    
    // Setup scroll view
    self.scrollView.documentView = self.textView;
    [self.view addSubview:self.scrollView];
    
    // Simple constraints
    [NSLayoutConstraint activateConstraints:@[
        [self.scrollView.topAnchor constraintEqualToAnchor:self.view.topAnchor constant:20],
        [self.scrollView.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:20],
        [self.scrollView.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-20],
        [self.scrollView.bottomAnchor constraintEqualToAnchor:self.view.bottomAnchor constant:-20],
    ]];
    
    NSLog(@"✅ SimpleViewController loaded successfully");
}

@end