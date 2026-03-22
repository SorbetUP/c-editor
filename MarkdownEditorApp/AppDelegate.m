#import "AppDelegate.h"
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
    [self createWindowShell];
    if (![self ensureWebArtifactsReady]) {
        [self updateStatus:@"Build web/WASM impossible"];
    }
    [self startLocalServer];
    [self.window center];
    [self.window makeKeyAndOrderFront:nil];
    [self.window orderFrontRegardless];
    [NSApp activateIgnoringOtherApps:YES];
}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
    [self stopLocalServer];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return YES;
}

// Handle file opening (drag & drop or Open With)
- (BOOL)application:(NSApplication *)sender openFile:(NSString *)filename {
    NSLog(@"📁 Would open file: %@", filename);
    return YES;
}

- (void)createWindowShell {
    NSRect windowFrame = NSMakeRect(100, 100, 1420, 920);
    self.window = [[NSWindow alloc] initWithContentRect:windowFrame
                                              styleMask:(NSWindowStyleMaskTitled |
                                                         NSWindowStyleMaskClosable |
                                                         NSWindowStyleMaskMiniaturizable |
                                                         NSWindowStyleMaskResizable)
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    [self.window setTitle:@"ElephantNote"];
    [self.window setBackgroundColor:[NSColor blackColor]];

    NSView *contentView = self.window.contentView;
    contentView.wantsLayer = YES;
    contentView.layer.backgroundColor = [NSColor blackColor].CGColor;

    NSView *toolbar = [[NSView alloc] init];
    toolbar.translatesAutoresizingMaskIntoConstraints = NO;
    toolbar.wantsLayer = YES;
    toolbar.layer.backgroundColor = [NSColor colorWithWhite:0.03 alpha:1.0].CGColor;
    [contentView addSubview:toolbar];

    NSButton *vaultButton = [self actionButtonWithTitle:@"Vault" action:@selector(chooseVault:)];
    NSButton *browserButton = [self actionButtonWithTitle:@"Navigateur" action:@selector(openInBrowser:)];
    NSButton *reloadButton = [self actionButtonWithTitle:@"Rafraîchir" action:@selector(reloadEditor:)];
    NSStackView *buttonStack = [[NSStackView alloc] initWithFrame:NSZeroRect];
    buttonStack.translatesAutoresizingMaskIntoConstraints = NO;
    buttonStack.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    buttonStack.spacing = 8.0;
    [buttonStack addArrangedSubview:vaultButton];
    [buttonStack addArrangedSubview:browserButton];
    [buttonStack addArrangedSubview:reloadButton];
    [toolbar addSubview:buttonStack];

    self.statusLabel = [[NSTextField alloc] init];
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.statusLabel.editable = NO;
    self.statusLabel.bordered = NO;
    self.statusLabel.backgroundColor = NSColor.clearColor;
    self.statusLabel.textColor = [NSColor colorWithWhite:0.82 alpha:1.0];
    self.statusLabel.font = [NSFont systemFontOfSize:12 weight:NSFontWeightMedium];
    self.statusLabel.stringValue = @"Démarrage...";
    [toolbar addSubview:self.statusLabel];

    WKWebViewConfiguration *configuration = [[WKWebViewConfiguration alloc] init];
    self.webView = [[WKWebView alloc] initWithFrame:NSZeroRect configuration:configuration];
    self.webView.translatesAutoresizingMaskIntoConstraints = NO;
    self.webView.navigationDelegate = self;
    [contentView addSubview:self.webView];

    [NSLayoutConstraint activateConstraints:@[
        [toolbar.topAnchor constraintEqualToAnchor:contentView.topAnchor],
        [toolbar.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [toolbar.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [toolbar.heightAnchor constraintEqualToConstant:56.0],

        [buttonStack.leadingAnchor constraintEqualToAnchor:toolbar.leadingAnchor constant:16.0],
        [buttonStack.centerYAnchor constraintEqualToAnchor:toolbar.centerYAnchor],

        [self.statusLabel.trailingAnchor constraintEqualToAnchor:toolbar.trailingAnchor constant:-16.0],
        [self.statusLabel.centerYAnchor constraintEqualToAnchor:toolbar.centerYAnchor],
        [self.statusLabel.leadingAnchor constraintGreaterThanOrEqualToAnchor:buttonStack.trailingAnchor constant:12.0],

        [self.webView.topAnchor constraintEqualToAnchor:toolbar.bottomAnchor],
        [self.webView.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [self.webView.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [self.webView.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor]
    ]];
}

- (NSButton *)actionButtonWithTitle:(NSString *)title action:(SEL)action {
    NSButton *button = [NSButton buttonWithTitle:title target:self action:action];
    button.bezelStyle = NSBezelStyleRounded;
    button.font = [NSFont systemFontOfSize:12 weight:NSFontWeightSemibold];
    return button;
}

- (NSString *)repositoryRoot {
    NSString *bundlePath = NSBundle.mainBundle.bundlePath;
    return [bundlePath stringByDeletingLastPathComponent];
}

- (NSString *)serverScriptPath {
    return [[self repositoryRoot] stringByAppendingPathComponent:@"tools/local_api/server.py"];
}

- (NSString *)webDocsPath {
    return [[self repositoryRoot] stringByAppendingPathComponent:@"web/site/docs"];
}

- (BOOL)ensureWebArtifactsReady {
    NSFileManager *fileManager = [NSFileManager defaultManager];
    NSString *editorJs = [[self webDocsPath] stringByAppendingPathComponent:@"editor.js"];
    NSString *editorWasm = [[self webDocsPath] stringByAppendingPathComponent:@"editor.wasm"];
    if ([fileManager fileExistsAtPath:editorJs] && [fileManager fileExistsAtPath:editorWasm]) {
        return YES;
    }

    NSString *buildScript = [[self repositoryRoot] stringByAppendingPathComponent:@"scripts/build_github_pages.sh"];
    if (![fileManager isExecutableFileAtPath:buildScript]) {
        return NO;
    }

    NSTask *task = [[NSTask alloc] init];
    task.executableURL = [NSURL fileURLWithPath:@"/bin/bash"];
    task.arguments = @[buildScript];
    task.currentDirectoryURL = [NSURL fileURLWithPath:[self repositoryRoot]];
    NSError *error = nil;
    if (![task launchAndReturnError:&error]) {
        NSLog(@"❌ Build script failed to start: %@", error);
        return NO;
    }
    [task waitUntilExit];
    return task.terminationStatus == 0;
}

- (BOOL)isPortAvailable:(NSInteger)port {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return NO;
    }

    int reuse = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_len = sizeof(addr);
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = htons((uint16_t)port);

    BOOL available = (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) == 0);
    close(fd);
    return available;
}

- (NSInteger)pickServerPort {
    NSInteger startPort = [[NSUserDefaults standardUserDefaults] integerForKey:@"elephantnote.serverPort"];
    if (startPort <= 0) {
        startPort = 8421;
    }

    for (NSInteger port = startPort; port < startPort + 20; port++) {
        if ([self isPortAvailable:port]) {
            return port;
        }
    }
    return startPort;
}

- (void)startLocalServer {
    if (self.serverTask && self.serverTask.isRunning) {
        return;
    }

    NSString *scriptPath = [self serverScriptPath];
    if (![[NSFileManager defaultManager] fileExistsAtPath:scriptPath]) {
        [self updateStatus:@"Serveur local introuvable"];
        return;
    }

    self.serverPort = [self pickServerPort];
    [[NSUserDefaults standardUserDefaults] setInteger:self.serverPort forKey:@"elephantnote.serverPort"];

    NSString *logPath = @"/tmp/elephantnote-desktop.log";
    NSString *command = [NSString stringWithFormat:
                         @"cd '%@' && /usr/bin/python3 '%@' --host 127.0.0.1 --port %ld --repo-root '%@' >> '%@' 2>&1",
                         [self repositoryRoot],
                         scriptPath,
                         (long)self.serverPort,
                         [self repositoryRoot],
                         logPath];

    NSTask *task = [[NSTask alloc] init];
    task.executableURL = [NSURL fileURLWithPath:@"/bin/bash"];
    task.arguments = @[@"-c", command];
    task.currentDirectoryURL = [NSURL fileURLWithPath:[self repositoryRoot]];

    NSError *error = nil;
    if (![task launchAndReturnError:&error]) {
        [self updateStatus:[NSString stringWithFormat:@"Échec serveur: %@", error.localizedDescription]];
        return;
    }

    self.serverTask = task;
    [self updateStatus:[NSString stringWithFormat:@"Serveur local sur :%ld", (long)self.serverPort]];
    [self waitForServerAndLoadWithRetry:0];
}

- (void)stopLocalServer {
    if (self.serverTask && self.serverTask.isRunning) {
        [self.serverTask terminate];
    }
    self.serverTask = nil;
}

- (NSURL *)editorURL {
    return [NSURL URLWithString:[NSString stringWithFormat:@"http://127.0.0.1:%ld/docs/index.html", (long)self.serverPort]];
}

- (NSURL *)healthURL {
    return [NSURL URLWithString:[NSString stringWithFormat:@"http://127.0.0.1:%ld/api/health", (long)self.serverPort]];
}

- (void)waitForServerAndLoadWithRetry:(NSInteger)retry {
    NSURL *url = [self healthURL];
    NSURLSessionDataTask *task = [[NSURLSession sharedSession] dataTaskWithURL:url completionHandler:^(__unused NSData *data, NSURLResponse *response, NSError *error) {
        if (!error && [(NSHTTPURLResponse *)response statusCode] == 200) {
            dispatch_async(dispatch_get_main_queue(), ^{
                [self updateStatus:@"API locale connectée"];
                [self configureVaultIfNeeded];
                [self.webView loadRequest:[NSURLRequest requestWithURL:[self editorURL]]];
            });
            return;
        }

        if (retry >= 40) {
            dispatch_async(dispatch_get_main_queue(), ^{
                [self updateStatus:@"Serveur local non joignable"];
            });
            return;
        }

        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.25 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
            [self waitForServerAndLoadWithRetry:retry + 1];
        });
    }];
    [task resume];
}

- (void)configureVaultIfNeeded {
    NSString *savedVaultPath = [[NSUserDefaults standardUserDefaults] stringForKey:@"elephantnote.vaultPath"];
    if (savedVaultPath.length > 0) {
        [self pushVaultPathToServer:savedVaultPath];
        return;
    }
    [self chooseVault:nil];
}

- (void)pushVaultPathToServer:(NSString *)vaultPath {
    if (vaultPath.length == 0) {
        return;
    }

    NSURL *url = [NSURL URLWithString:[NSString stringWithFormat:@"http://127.0.0.1:%ld/api/vault", (long)self.serverPort]];
    NSMutableURLRequest *request = [NSMutableURLRequest requestWithURL:url];
    request.HTTPMethod = @"PUT";
    request.timeoutInterval = 5.0;
    request.allHTTPHeaderFields = @{@"Content-Type": @"application/json"};
    NSData *body = [NSJSONSerialization dataWithJSONObject:@{@"path": vaultPath} options:0 error:nil];
    request.HTTPBody = body;

    NSURLSessionDataTask *task = [[NSURLSession sharedSession] dataTaskWithRequest:request completionHandler:^(__unused NSData *data, NSURLResponse *response, NSError *error) {
        dispatch_async(dispatch_get_main_queue(), ^{
            if (!error && [(NSHTTPURLResponse *)response statusCode] < 300) {
                [[NSUserDefaults standardUserDefaults] setObject:vaultPath forKey:@"elephantnote.vaultPath"];
                [self updateStatus:[NSString stringWithFormat:@"Vault : %@", vaultPath]];
                return;
            }
            [self updateStatus:@"Impossible de configurer le vault"];
        });
    }];
    [task resume];
}

- (void)chooseVault:(id)sender {
    NSOpenPanel *panel = [NSOpenPanel openPanel];
    panel.canChooseDirectories = YES;
    panel.canChooseFiles = NO;
    panel.canCreateDirectories = YES;
    panel.allowsMultipleSelection = NO;
    panel.title = @"Choisir un dossier pour le vault";
    [panel beginSheetModalForWindow:self.window completionHandler:^(NSModalResponse result) {
        if (result != NSModalResponseOK) {
            return;
        }
        NSURL *selectedURL = panel.URLs.firstObject;
        if (!selectedURL) {
            return;
        }
        [self pushVaultPathToServer:selectedURL.path];
    }];
}

- (void)openInBrowser:(id)sender {
    [[NSWorkspace sharedWorkspace] openURL:[self editorURL]];
}

- (void)reloadEditor:(id)sender {
    [self.webView reload];
}

- (void)updateStatus:(NSString *)status {
    self.statusLabel.stringValue = status ?: @"";
}

- (void)webView:(WKWebView *)webView didFinishNavigation:(WKNavigation *)navigation {
    [self updateStatus:[NSString stringWithFormat:@"ElephantNote prêt sur :%ld", (long)self.serverPort]];
}

@end
