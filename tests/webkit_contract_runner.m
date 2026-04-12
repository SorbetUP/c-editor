#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <WebKit/WebKit.h>

@interface ContractRunner : NSObject <WKNavigationDelegate>
@property(nonatomic, strong) WKWebView *webView;
@property(nonatomic, strong) NSWindow *window;
@property(nonatomic, strong) NSURL *url;
@property(nonatomic, assign) NSTimeInterval timeoutSeconds;
@property(nonatomic, assign) BOOL done;
@property(nonatomic, assign) BOOL bootstrapInjected;
@property(nonatomic, assign) int exitCode;
@property(nonatomic, copy) NSString *output;
@property(nonatomic, strong) NSDate *deadline;
- (instancetype)initWithURL:(NSURL *)url timeout:(NSTimeInterval)timeoutSeconds;
- (void)start;
- (void)schedulePoll;
- (void)pollState;
- (NSString *)bootstrapScript;
@end

@implementation ContractRunner

- (instancetype)initWithURL:(NSURL *)url timeout:(NSTimeInterval)timeoutSeconds {
    self = [super init];
    if (self) {
        _url = url;
        _timeoutSeconds = timeoutSeconds;
        _exitCode = 2;
        _output = @"{\"error\":\"runner not started\"}";
    }
    return self;
}

- (void)start {
    WKWebViewConfiguration *configuration = [[WKWebViewConfiguration alloc] init];
    configuration.websiteDataStore = [WKWebsiteDataStore nonPersistentDataStore];
    configuration.preferences.javaScriptEnabled = YES;
    if (@available(macOS 11.0, *)) {
        configuration.defaultWebpagePreferences.allowsContentJavaScript = YES;
    }
    self.webView = [[WKWebView alloc] initWithFrame:NSMakeRect(0, 0, 1280, 900)
                                      configuration:configuration];
    self.webView.navigationDelegate = self;
    self.window = [[NSWindow alloc] initWithContentRect:NSMakeRect(-16000, -16000, 1280, 900)
                                              styleMask:NSWindowStyleMaskBorderless
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    [self.window setReleasedWhenClosed:NO];
    [self.window setContentView:self.webView];
    [self.window orderFront:nil];
    self.deadline = [NSDate dateWithTimeIntervalSinceNow:self.timeoutSeconds];
    [self.webView loadRequest:[NSURLRequest requestWithURL:self.url]];
    [self schedulePoll];
}

- (void)schedulePoll {
    if (self.done) {
        return;
    }

    if ([[NSDate date] compare:self.deadline] != NSOrderedAscending) {
        [self finishWithCode:2 payload:[NSString stringWithFormat:
            @"{\"error\":\"timeout\",\"state\":%@}",
            self.output ?: @"{}"]];
        return;
    }

    __weak typeof(self) weakSelf = self;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.20 * NSEC_PER_SEC)),
                   dispatch_get_main_queue(), ^{
        [weakSelf pollState];
    });
}

- (NSString *)javaScriptProbe {
    return @"(() => {"
            "return JSON.stringify({"
            "status: window.__contractStatus || (document.body ? (document.body.dataset.status || '') : ''),"
            "progress: window.__contractProgress || (document.body ? (document.body.dataset.progress || '') : ''),"
            "result: window.__contractResult || (document.getElementById('result') ? document.getElementById('result').textContent : '')"
            "});"
            "})()";
}

- (NSString *)bootstrapScript {
    NSString *path = [[[NSFileManager defaultManager] currentDirectoryPath]
        stringByAppendingPathComponent:@"tests/webkit_contract_bootstrap.js"];
    NSError *error = nil;
    NSString *script = [NSString stringWithContentsOfFile:path
                                                 encoding:NSUTF8StringEncoding
                                                    error:&error];
    if (!script || error) {
        return [NSString stringWithFormat:
            @"window.__contractStatus='error'; window.__contractProgress='bootstrap-read-failed'; "
             "window.__contractResult=JSON.stringify({error:%@});",
            [[NSString alloc] initWithData:[NSJSONSerialization dataWithJSONObject:@{@"message": error.localizedDescription ?: @"bootstrap script missing"}
                                                                  options:0
                                                                    error:nil]
                                  encoding:NSUTF8StringEncoding]];
    }
    return script;
}

- (void)pollState {
    if (self.done) {
        return;
    }

    if (!self.bootstrapInjected) {
        self.bootstrapInjected = YES;
        [self.webView evaluateJavaScript:[self bootstrapScript] completionHandler:nil];
    }

    __weak typeof(self) weakSelf = self;
    [self.webView evaluateJavaScript:[self javaScriptProbe]
                   completionHandler:^(id value, NSError *error) {
        if (weakSelf.done) {
            return;
        }

        if (error) {
            [weakSelf schedulePoll];
            return;
        }

        if (![value isKindOfClass:[NSString class]]) {
            [weakSelf schedulePoll];
            return;
        }

        NSData *data = [(NSString *)value dataUsingEncoding:NSUTF8StringEncoding];
        NSDictionary *payload = data ? [NSJSONSerialization JSONObjectWithData:data options:0 error:nil] : nil;
        NSString *status = [payload[@"status"] isKindOfClass:[NSString class]] ? payload[@"status"] : @"";
        NSString *result = [payload[@"result"] isKindOfClass:[NSString class]] ? payload[@"result"] : @"";
        weakSelf.output = (NSString *)value;

        if ([status isEqualToString:@"ok"]) {
            [weakSelf finishWithCode:0 payload:result];
            return;
        }

        if ([status isEqualToString:@"error"]) {
            [weakSelf finishWithCode:1 payload:result];
            return;
        }

        [weakSelf schedulePoll];
    }];
}

- (void)finishWithCode:(int)code payload:(NSString *)payload {
    self.done = YES;
    self.exitCode = code;
    self.output = payload ?: @"";
    CFRunLoopStop(CFRunLoopGetMain());
}

- (void)webView:(WKWebView *)webView didFailNavigation:(WKNavigation *)navigation withError:(NSError *)error {
    [self finishWithCode:1 payload:[NSString stringWithFormat:@"{\"error\":\"navigation failed\",\"code\":%ld}",
                                   (long)error.code]];
}

- (void)webView:(WKWebView *)webView didFailProvisionalNavigation:(WKNavigation *)navigation withError:(NSError *)error {
    [self finishWithCode:1 payload:[NSString stringWithFormat:@"{\"error\":\"provisional navigation failed\",\"code\":%ld}",
                                   (long)error.code]];
}

@end

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc < 2) {
            fprintf(stderr, "usage: webkit_contract_runner <url> [timeout_seconds]\n");
            return 2;
        }

        NSString *urlString = [NSString stringWithUTF8String:argv[1]];
        NSURL *url = [NSURL URLWithString:urlString];
        if (!url) {
            fprintf(stderr, "invalid url\n");
            return 2;
        }

        NSTimeInterval timeoutSeconds = 30.0;
        if (argc >= 3) {
            timeoutSeconds = atof(argv[2]);
            if (timeoutSeconds <= 0) {
                timeoutSeconds = 30.0;
            }
        }

        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyProhibited];
        ContractRunner *runner = [[ContractRunner alloc] initWithURL:url timeout:timeoutSeconds];
        [runner start];
        CFRunLoopRun();
        printf("%s\n", runner.output.UTF8String ?: "");
        return runner.exitCode;
    }
}
