#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

@interface AppDelegate : NSObject <NSApplicationDelegate, WKNavigationDelegate>

@property (strong, nonatomic) NSWindow *window;
@property (strong, nonatomic) WKWebView *webView;
@property (strong, nonatomic) NSTextField *statusLabel;
@property (strong, nonatomic) NSTask *serverTask;
@property (assign, nonatomic) NSInteger serverPort;

@end
