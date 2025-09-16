// ElephantNotesV2_Professional.m - Professional markdown editor with enterprise file management
#import <Cocoa/Cocoa.h>
#include "../hybrid_editor/hybrid_editor_core.h"
#include "../file_manager/file_manager.h"
#include "../file_manager/professional_file_manager.h"
#include "../editor/editor_abi.h"

@interface ElephantNotesProfessionalTextView : NSTextView {
    NSInteger _currentLineIndex;
    BOOL _isUpdating;
    TextLines* _textLines;
    NSString* _currentFilePath;
    NSString* _originalContent;
    BOOL _hasUnsavedChanges;
    
    // Professional features
    WorkspaceSession* _workspace;
    VersionHistory* _versionHistory;
    NSTimer* _autoSaveTimer;
    NSTimer* _conflictCheckTimer;
    NSMutableArray* _recentFiles;
    BOOL _sessionRecoveryEnabled;
    NSString* _workspacePath;
}

@property (nonatomic, strong) NSString* currentFilePath;
@property (nonatomic, assign) BOOL hasUnsavedChanges;

// File operations
- (void)newDocument;
- (void)openDocument;
- (void)saveDocument;
- (void)saveDocumentAs;
- (void)updateWindowTitle;

// Professional features
- (void)initializeProfessionalFeatures;
- (void)enableAutoSave;
- (void)disableAutoSave;
- (void)createVersionSnapshot:(NSString*)comment;
- (void)showVersionHistory;
- (void)recoverFromAutoSave;
- (void)checkForConflicts;
- (void)saveWorkspaceSession;
- (void)loadWorkspaceSession;
- (void)showFileStatistics;

@end

@implementation ElephantNotesProfessionalTextView

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        _currentLineIndex = 0;
        _isUpdating = NO;
        _textLines = NULL;
        _currentFilePath = nil;
        _originalContent = @"";
        _hasUnsavedChanges = NO;
        _workspace = NULL;
        _versionHistory = NULL;
        _sessionRecoveryEnabled = YES;
        _workspacePath = [@"~/Documents/ElephantNotes_Workspace" stringByExpandingTildeInPath];
        _recentFiles = [[NSMutableArray alloc] init];
        
        // Configure text view
        [self setFont:[NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]];
        [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [self setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]];
        
        // Initialize professional features
        [self initializeProfessionalFeatures];
        
        // Welcome content
        [self newDocument];
        
        // Real-time updates
        [NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(updateDisplay) userInfo:nil repeats:YES];
        
        // Initial render
        dispatch_async(dispatch_get_main_queue(), ^{
            [self updateDisplay];
        });
    }
    return self;
}

- (void)initializeProfessionalFeatures {
    // Initialize professional file manager
    professional_init();
    
    // Configure professional settings
    ProfessionalConfig config = professional_get_config();
    config.auto_save_enabled = true;
    config.auto_save_interval_ms = 3000; // 3 seconds
    config.version_control_enabled = true;
    config.conflict_detection_enabled = true;
    config.session_recovery_enabled = true;
    config.backup_config.strategy = BACKUP_TIMESTAMPED;
    config.backup_config.max_versions = 20;
    professional_set_config(&config);
    
    // Create workspace if it doesn't exist
    NSFileManager* fileManager = [NSFileManager defaultManager];
    if (![fileManager fileExistsAtPath:_workspacePath]) {
        [fileManager createDirectoryAtPath:_workspacePath withIntermediateDirectories:YES attributes:nil error:nil];
    }
    
    // Initialize workspace session
    _workspace = session_create_workspace([_workspacePath UTF8String]);
    
    // Try to recover previous session
    if (_sessionRecoveryEnabled) {
        [self loadWorkspaceSession];
    }
    
    // Enable auto-save
    [self enableAutoSave];
    
    // Start conflict monitoring
    _conflictCheckTimer = [NSTimer scheduledTimerWithTimeInterval:10.0 
                                                           target:self 
                                                         selector:@selector(checkForConflicts) 
                                                         userInfo:nil 
                                                          repeats:YES];
}

- (void)enableAutoSave {
    if (_autoSaveTimer) {
        [_autoSaveTimer invalidate];
    }
    
    _autoSaveTimer = [NSTimer scheduledTimerWithTimeInterval:3.0 
                                                      target:self 
                                                    selector:@selector(performAutoSave) 
                                                    userInfo:nil 
                                                     repeats:YES];
}

- (void)performAutoSave {
    if (!self.currentFilePath || !self.hasUnsavedChanges) return;
    
    NSString* content = [self string];
    const char* path = [self.currentFilePath UTF8String];
    const char* content_cstr = [content UTF8String];
    
    // Perform auto-save using professional file manager
    FileResult result = auto_save_save_now(path, content_cstr, strlen(content_cstr));
    if (result == FILE_SUCCESS) {
        NSLog(@"📁 Auto-saved: %@", self.currentFilePath);
    }
}

- (void)dealloc {
    if (_autoSaveTimer) {
        [_autoSaveTimer invalidate];
    }
    if (_conflictCheckTimer) {
        [_conflictCheckTimer invalidate];
    }
    if (_workspace) {
        session_free_workspace(_workspace);
    }
    if (_versionHistory) {
        version_free_history(_versionHistory);
    }
    if (_textLines) {
        hybrid_free_text_lines(_textLines);
    }
    professional_cleanup();
}

- (void)newDocument {
    // Save current session before creating new document
    if (self.currentFilePath) {
        [self saveWorkspaceSession];
    }
    
    NSString* welcomeContent = @"# ElephantNotes Professional\n\n✨ **Professional Features:**\n- 🔄 **Auto-save** every 3 seconds\n- 📚 **Version control** with history\n- 🔍 **Conflict detection**\n- 💾 **Session recovery**\n- 📊 **File statistics**\n\n## Getting Started\nStart typing your markdown content...\n\n*This document demonstrates professional markdown editing with enterprise features.*";
    
    [self setString:welcomeContent];
    self.currentFilePath = nil;
    _originalContent = welcomeContent;
    self.hasUnsavedChanges = NO;
    [self updateWindowTitle];
    
    // Create version history for new document
    if (_versionHistory) {
        version_free_history(_versionHistory);
        _versionHistory = NULL;
    }
}

- (void)openDocument {
    NSOpenPanel* openPanel = [NSOpenPanel openPanel];
    [openPanel setCanChooseFiles:YES];
    [openPanel setCanChooseDirectories:NO];
    [openPanel setAllowsMultipleSelection:NO];
    [openPanel setAllowedFileTypes:@[@"md", @"markdown", @"mdown", @"mkd", @"txt"]];
    [openPanel setTitle:@"Open Markdown Document"];
    
    // Set initial directory
    char* docs_dir = file_get_documents_dir();
    if (docs_dir) {
        NSString* docsPath = [NSString stringWithUTF8String:docs_dir];
        [openPanel setDirectoryURL:[NSURL fileURLWithPath:docsPath]];
        file_free_string(docs_dir);
    }
    
    [openPanel beginWithCompletionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* url = [[openPanel URLs] firstObject];
            [self loadFileAtPath:[url path]];
        }
    }];
}

- (void)loadFileAtPath:(NSString*)path {
    if (!path) return;
    
    // Save current session
    if (self.currentFilePath) {
        [self saveWorkspaceSession];
    }
    
    const char* path_cstr = [path UTF8String];
    FileContent* content = NULL;
    
    // Check for auto-save recovery
    FileContent* autoSaveContent = NULL;
    FileResult autoSaveResult = auto_save_recover(path_cstr, &autoSaveContent);
    
    if (autoSaveResult == FILE_SUCCESS && autoSaveContent) {
        NSAlert* alert = [[NSAlert alloc] init];
        [alert setMessageText:@"Auto-save Recovery"];
        [alert setInformativeText:@"An auto-saved version of this file was found. Would you like to recover it?"];
        [alert addButtonWithTitle:@"Recover Auto-save"];
        [alert addButtonWithTitle:@"Load Original"];
        [alert addButtonWithTitle:@"Cancel"];
        
        NSInteger choice = [alert runModal];
        if (choice == NSAlertFirstButtonReturn) {
            // Use auto-save content
            content = autoSaveContent;
        } else if (choice == NSAlertSecondButtonReturn) {
            // Load original and clean auto-save
            file_free_content(autoSaveContent);
            auto_save_cleanup(path_cstr);
            FileResult result = file_read(path_cstr, &content);
            if (result != FILE_SUCCESS) {
                [self showErrorAlert:@"Error Opening File" message:file_get_error_message(result)];
                return;
            }
        } else {
            // Cancel
            file_free_content(autoSaveContent);
            return;
        }
    } else {
        // No auto-save, load original
        FileResult result = file_read(path_cstr, &content);
        if (result != FILE_SUCCESS) {
            [self showErrorAlert:@"Error Opening File" message:file_get_error_message(result)];
            return;
        }
    }
    
    if (content) {
        NSString* fileContent = [NSString stringWithUTF8String:content->content];
        [self setString:fileContent];
        
        self.currentFilePath = path;
        _originalContent = fileContent;
        self.hasUnsavedChanges = NO;
        [self updateWindowTitle];
        
        // Add to recent files
        [_recentFiles removeObject:path];
        [_recentFiles insertObject:path atIndex:0];
        if ([_recentFiles count] > 10) {
            [_recentFiles removeLastObject];
        }
        
        // Add to workspace session
        session_add_file(_workspace, path_cstr);
        
        // Load version history
        if (_versionHistory) {
            version_free_history(_versionHistory);
        }
        version_create_history(path_cstr, &_versionHistory);
        
        file_free_content(content);
        
        NSLog(@"✅ Loaded file: %@", path);
    }
}

- (void)saveDocument {
    if (self.currentFilePath) {
        [self saveToPath:self.currentFilePath];
    } else {
        [self saveDocumentAs];
    }
}

- (void)saveDocumentAs {
    NSSavePanel* savePanel = [NSSavePanel savePanel];
    [savePanel setAllowedFileTypes:@[@"md"]];
    [savePanel setTitle:@"Save Markdown Document"];
    [savePanel setNameFieldStringValue:@"Untitled.md"];
    
    // Set initial directory
    char* docs_dir = file_get_documents_dir();
    if (docs_dir) {
        NSString* docsPath = [NSString stringWithUTF8String:docs_dir];
        [savePanel setDirectoryURL:[NSURL fileURLWithPath:docsPath]];
        file_free_string(docs_dir);
    }
    
    [savePanel beginWithCompletionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* url = [savePanel URL];
            NSString* path = [url path];
            
            if (![path.pathExtension isEqualToString:@"md"]) {
                path = [path stringByAppendingPathExtension:@"md"];
            }
            
            [self saveToPath:path];
        }
    }];
}

- (void)saveToPath:(NSString*)path {
    if (!path) return;
    
    NSString* content = [self string];
    const char* path_cstr = [path UTF8String];
    const char* content_cstr = [content UTF8String];
    
    // Create version snapshot before saving
    if (_versionHistory) {
        ProfessionalConfig config = professional_get_config();
        version_add_version(_versionHistory, content_cstr, strlen(content_cstr), 
                          config.author_name, "Auto-saved version");
    } else {
        // Create version history for new file
        version_create_history(path_cstr, &_versionHistory);
        if (_versionHistory) {
            ProfessionalConfig config = professional_get_config();
            version_add_version(_versionHistory, content_cstr, strlen(content_cstr), 
                              config.author_name, "Initial version");
        }
    }
    
    // Save with professional backup
    FileResult result = file_save_with_backup(path_cstr, content_cstr, strlen(content_cstr));
    if (result == FILE_SUCCESS) {
        self.currentFilePath = path;
        _originalContent = content;
        self.hasUnsavedChanges = NO;
        [self updateWindowTitle];
        
        // Clean auto-save
        auto_save_cleanup(path_cstr);
        
        // Update workspace session
        session_add_file(_workspace, path_cstr);
        [self saveWorkspaceSession];
        
        NSLog(@"✅ Saved file: %@", path);
        [self showSaveSuccessIndicator];
    } else {
        [self showErrorAlert:@"Error Saving File" message:file_get_error_message(result)];
    }
}

- (void)showErrorAlert:(NSString*)title message:(const char*)message {
    NSAlert* alert = [[NSAlert alloc] init];
    [alert setMessageText:title];
    [alert setInformativeText:[NSString stringWithUTF8String:message]];
    [alert setAlertStyle:NSAlertStyleWarning];
    [alert runModal];
}

- (void)showSaveSuccessIndicator {
    NSView* superview = [[self enclosingScrollView] superview];
    NSTextField* indicator = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 20, 200, 30)];
    [indicator setStringValue:@"✅ Saved with version control"];
    [indicator setBackgroundColor:[NSColor colorWithRed:0.0 green:0.7 blue:0.0 alpha:0.8]];
    [indicator setTextColor:[NSColor whiteColor]];
    [indicator setBordered:NO];
    [indicator setEditable:NO];
    [indicator setFont:[NSFont systemFontOfSize:12 weight:NSFontWeightSemibold]];
    [indicator.layer setCornerRadius:5];
    
    [superview addSubview:indicator];
    
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 3 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [NSAnimationContext runAnimationGroup:^(NSAnimationContext *context) {
            context.duration = 0.5;
            indicator.animator.alphaValue = 0.0;
        } completionHandler:^{
            [indicator removeFromSuperview];
        }];
    });
}

- (void)createVersionSnapshot:(NSString*)comment {
    if (!self.currentFilePath || !_versionHistory) return;
    
    NSString* content = [self string];
    const char* content_cstr = [content UTF8String];
    ProfessionalConfig config = professional_get_config();
    
    FileResult result = version_add_version(_versionHistory, content_cstr, strlen(content_cstr), 
                                          config.author_name, [comment UTF8String]);
    
    if (result == FILE_SUCCESS) {
        NSLog(@"📸 Version snapshot created: %@", comment);
        [self showVersionSnapshotIndicator:comment];
    }
}

- (void)showVersionSnapshotIndicator:(NSString*)comment {
    NSView* superview = [[self enclosingScrollView] superview];
    NSTextField* indicator = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 60, 300, 40)];
    [indicator setStringValue:[NSString stringWithFormat:@"📸 Snapshot: %@", comment]];
    [indicator setBackgroundColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.8]];
    [indicator setTextColor:[NSColor whiteColor]];
    [indicator setBordered:NO];
    [indicator setEditable:NO];
    [indicator setFont:[NSFont systemFontOfSize:11 weight:NSFontWeightMedium]];
    [indicator.layer setCornerRadius:5];
    
    [superview addSubview:indicator];
    
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 3 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [NSAnimationContext runAnimationGroup:^(NSAnimationContext *context) {
            context.duration = 0.5;
            indicator.animator.alphaValue = 0.0;
        } completionHandler:^{
            [indicator removeFromSuperview];
        }];
    });
}

- (void)checkForConflicts {
    if (!self.currentFilePath) return;
    
    const char* path = [self.currentFilePath UTF8String];
    FileConflict* conflict = NULL;
    
    FileResult result = conflict_check_file(path, &conflict);
    if (result == FILE_SUCCESS && conflict) {
        NSAlert* alert = [[NSAlert alloc] init];
        [alert setMessageText:@"File Conflict Detected"];
        [alert setInformativeText:@"This file has been modified by another application. What would you like to do?"];
        [alert addButtonWithTitle:@"Reload from Disk"];
        [alert addButtonWithTitle:@"Keep My Changes"];
        [alert addButtonWithTitle:@"Show Differences"];
        
        NSInteger choice = [alert runModal];
        switch (choice) {
            case NSAlertFirstButtonReturn:
                [self loadFileAtPath:self.currentFilePath];
                break;
            case NSAlertSecondButtonReturn:
                // Keep current changes, create backup of disk version
                break;
            case NSAlertThirdButtonReturn:
                // TODO: Show diff view
                break;
        }
        
        conflict_free(conflict);
    }
}

- (void)saveWorkspaceSession {
    if (_workspace) {
        // Update current file session
        if (self.currentFilePath) {
            NSRange selection = [self selectedRange];
            session_update_cursor(_workspace, [self.currentFilePath UTF8String], 
                                (uint32_t)selection.location, 
                                (uint32_t)selection.location, 
                                (uint32_t)NSMaxRange(selection));
        }
        
        session_save_workspace(_workspace);
    }
}

- (void)loadWorkspaceSession {
    // TODO: Implement session loading
}

- (void)disableAutoSave {
    if (_autoSaveTimer) {
        [_autoSaveTimer invalidate];
        _autoSaveTimer = nil;
    }
}

- (void)showVersionHistory {
    if (!_versionHistory || _versionHistory->count == 0) {
        NSAlert* alert = [[NSAlert alloc] init];
        [alert setMessageText:@"No Version History"];
        [alert setInformativeText:@"No versions have been created for this document yet."];
        [alert addButtonWithTitle:@"OK"];
        [alert runModal];
        return;
    }
    
    NSAlert* alert = [[NSAlert alloc] init];
    [alert setMessageText:@"Version History"];
    
    NSMutableString* versions = [[NSMutableString alloc] init];
    for (int i = 0; i < _versionHistory->count; i++) {
        FileVersion* version = &_versionHistory->versions[i];
        [versions appendFormat:@"Version %d: %s - %s\n", 
            version->version_id, 
            version->timestamp, 
            version->comment];
    }
    
    [alert setInformativeText:versions];
    [alert addButtonWithTitle:@"OK"];
    [alert runModal];
}

- (void)recoverFromAutoSave {
    if (!self.currentFilePath) return;
    
    const char* path = [self.currentFilePath UTF8String];
    FileContent* content = NULL;
    
    FileResult result = auto_save_recover(path, &content);
    if (result == FILE_SUCCESS && content) {
        NSString* recoveredContent = [NSString stringWithUTF8String:content->content];
        [self setString:recoveredContent];
        self.hasUnsavedChanges = YES;
        [self updateWindowTitle];
        
        file_free_content(content);
        
        NSLog(@"📄 Recovered from auto-save: %@", self.currentFilePath);
    }
}

- (void)showFileStatistics {
    FileManagerStats stats;
    stats_get_current(&stats);
    
    NSAlert* alert = [[NSAlert alloc] init];
    [alert setMessageText:@"File Manager Statistics"];
    [alert setInformativeText:[NSString stringWithFormat:
        @"Files managed: %d\n"
        @"Versions created: %d\n"
        @"Auto-saves performed: %d\n"
        @"Conflicts detected: %d\n"
        @"Storage used: %.2f MB\n"
        @"Active sessions: %d",
        stats.total_files_managed,
        stats.total_versions_created,
        stats.total_auto_saves,
        stats.conflicts_detected,
        (double)stats.total_storage_used / (1024 * 1024),
        stats.active_sessions
    ]];
    [alert addButtonWithTitle:@"OK"];
    [alert runModal];
}

- (BOOL)hasUnsavedChanges {
    NSString* currentContent = [self string];
    return file_has_unsaved_changes([_originalContent UTF8String], [currentContent UTF8String]);
}

- (void)updateWindowTitle {
    NSWindow* window = [self window];
    if (!window) return;
    
    NSString* title;
    if (self.currentFilePath) {
        NSString* filename = [self.currentFilePath lastPathComponent];
        title = [self hasUnsavedChanges] ? 
            [NSString stringWithFormat:@"%@ • ElephantNotes Professional", filename] : 
            [NSString stringWithFormat:@"%@ - ElephantNotes Professional", filename];
    } else {
        title = [self hasUnsavedChanges] ? 
            @"Untitled • ElephantNotes Professional" : 
            @"Untitled - ElephantNotes Professional";
    }
    
    [window setTitle:title];
    [window setDocumentEdited:[self hasUnsavedChanges]];
}

// Hybrid editor logic (same as v2)
- (void)updateDisplay {
    if (_isUpdating) return;
    
    NSInteger newCurrentLine = [self getCurrentLineUsingCLib];
    
    if (newCurrentLine == _currentLineIndex) return;
    
    _isUpdating = YES;
    
    BOOL hadChanges = self.hasUnsavedChanges;
    self.hasUnsavedChanges = [self hasUnsavedChanges];
    if (hadChanges != self.hasUnsavedChanges) {
        [self updateWindowTitle];
    }
    
    NSRange oldSelection = [self selectedRange];
    [self applyHybridFormattingUsingCLib:newCurrentLine];
    [self setSelectedRange:oldSelection];
    
    _currentLineIndex = newCurrentLine;
    _isUpdating = NO;
}

- (NSInteger)getCurrentLineUsingCLib {
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    NSUInteger cursorPos = [self selectedRange].location;
    NSUInteger pos = 0;
    
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSUInteger lineLength = [[lines objectAtIndex:i] length];
        if (cursorPos <= pos + lineLength) {
            return i;
        }
        pos += lineLength + 1;
    }
    
    return [lines count] - 1;
}

- (void)applyHybridFormattingUsingCLib:(NSInteger)currentLine {
    NSMutableAttributedString *text = [[self textStorage] mutableCopy];
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    
    for (NSInteger i = 0; i < [lines count]; i++) {
        NSRange lineRange = [self getRangeForLineIndex:i];
        if (lineRange.length == 0 && i < [lines count] - 1) continue;
        
        NSString *lineContent = [lines objectAtIndex:i];
        
        if (i == currentLine) {
            [self formatAsCurrentLine:text range:lineRange];
        } else {
            [self formatLineUsingCLib:text range:lineRange content:[lineContent UTF8String]];
        }
    }
    
    [[self textStorage] setAttributedString:text];
}

- (NSRange)getRangeForLineIndex:(NSInteger)lineIndex {
    NSArray *lines = [[self string] componentsSeparatedByString:@"\n"];
    if (lineIndex < 0 || lineIndex >= [lines count]) {
        return NSMakeRange(0, 0);
    }
    
    NSUInteger pos = 0;
    for (NSInteger i = 0; i < lineIndex; i++) {
        pos += [[lines objectAtIndex:i] length] + 1;
    }
    
    return NSMakeRange(pos, [[lines objectAtIndex:lineIndex] length]);
}

- (void)formatAsCurrentLine:(NSMutableAttributedString *)text range:(NSRange)range {
    NSDictionary *attrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:0.25]
    };
    [text setAttributes:attrs range:range];
}

- (void)formatLineUsingCLib:(NSMutableAttributedString *)text range:(NSRange)range content:(const char*)content {
    if (range.location + range.length > [text length]) {
        return;
    }
    
    NSString *contentStr = [NSString stringWithUTF8String:content];
    
    const char *html = editor_markdown_to_html(content);
    if (!html) {
        [self applyBasicFormatting:text inRange:range content:contentStr];
        return;
    }
    
    NSString *htmlString = [NSString stringWithUTF8String:html];
    [self applyFormattingAndHideMarkupV1Style:text inRange:range content:contentStr htmlResult:htmlString];
}

- (void)applyBasicFormatting:(NSMutableAttributedString *)text inRange:(NSRange)range content:(NSString *)content {
    NSDictionary *baseAttrs = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttrs range:range];
}

- (void)applyFormattingAndHideMarkupV1Style:(NSMutableAttributedString *)text inRange:(NSRange)baseRange content:(NSString *)content htmlResult:(NSString *)html {
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
    
    // Bold, italic, highlight (same as v2)
    if ([html containsString:@"<strong>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"\\*\\*([^*\\n]+)\\*\\*" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0],
                                NSFontAttributeName: [NSFont fontWithName:@"Monaco-Bold" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightBold]
                            }];
    }
    
    if ([html containsString:@"<em>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"(?<!\\*)\\*([^*\\n]+)\\*(?!\\*)" 
                            attributes:@{
                                NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.85 blue:0.24 alpha:1.0]
                            }];
    }
    
    if ([html containsString:@"<mark>"]) {
        [self applyInlineFormatAndHide:text baseRange:baseRange pattern:@"==([^=\\n]+)==" 
                            attributes:@{
                                NSBackgroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.65 blue:0.0 alpha:0.8],
                                NSForegroundColorAttributeName: [NSColor blackColor]
                            }];
    }
}

- (void)applyInlineFormatAndHide:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern attributes:(NSDictionary *)attributes {
    if (baseRange.location + baseRange.length > [text length]) return;
    
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    
    for (NSTextCheckingResult *match in matches) {
        if ([match numberOfRanges] < 2) continue;
        
        NSRange contentRange = NSMakeRange(baseRange.location + [match rangeAtIndex:1].location, [match rangeAtIndex:1].length);
        if (contentRange.location + contentRange.length <= [text length]) {
            [text addAttributes:attributes range:contentRange];
        }
        
        NSRange fullMatch = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        if (fullMatch.location + fullMatch.length <= [text length]) {
            NSRange beforeContent = NSMakeRange(fullMatch.location, contentRange.location - fullMatch.location);
            NSRange afterContent = NSMakeRange(NSMaxRange(contentRange), NSMaxRange(fullMatch) - NSMaxRange(contentRange));
            
            NSDictionary *hideAttrs = @{
                NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0],
                NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
            };
            
            if (beforeContent.length > 0 && beforeContent.location + beforeContent.length <= [text length]) {
                [text addAttributes:hideAttrs range:beforeContent];
            }
            if (afterContent.length > 0 && afterContent.location + afterContent.length <= [text length]) {
                [text addAttributes:hideAttrs range:afterContent];
            }
        }
    }
}

- (void)hideMarkupInRange:(NSMutableAttributedString *)text baseRange:(NSRange)baseRange pattern:(NSString *)pattern {
    if (baseRange.location + baseRange.length > [text length]) return;
    
    NSString *lineText = [text.string substringWithRange:baseRange];
    NSError *error = nil;
    NSRegularExpression *regex = [NSRegularExpression regularExpressionWithPattern:pattern options:0 error:&error];
    if (error) return;
    
    NSArray *matches = [regex matchesInString:lineText options:0 range:NSMakeRange(0, [lineText length])];
    for (NSTextCheckingResult *match in matches) {
        NSRange markupRange = NSMakeRange(baseRange.location + match.range.location, match.range.length);
        if (markupRange.location + markupRange.length <= [text length]) {
            [text addAttributes:@{
                NSForegroundColorAttributeName: [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0],
                NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:1] ?: [NSFont monospacedSystemFontOfSize:1 weight:NSFontWeightRegular]
            } range:markupRange];
        }
    }
}

// Keyboard shortcuts with professional features
- (void)keyDown:(NSEvent *)event {
    NSString* characters = [event charactersIgnoringModifiers];
    NSEventModifierFlags modifiers = [event modifierFlags];
    
    if (modifiers & NSEventModifierFlagCommand) {
        if ([characters isEqualToString:@"o"]) {
            [self openDocument];
            return;
        } else if ([characters isEqualToString:@"s"]) {
            if (modifiers & NSEventModifierFlagShift) {
                [self saveDocumentAs];
            } else {
                [self saveDocument];
            }
            return;
        } else if ([characters isEqualToString:@"n"]) {
            [self newDocument];
            return;
        } else if ([characters isEqualToString:@"k"]) {
            // ⌘+K: Create version snapshot
            NSAlert* alert = [[NSAlert alloc] init];
            [alert setMessageText:@"Create Version Snapshot"];
            [alert setInformativeText:@"Enter a comment for this version:"];
            NSTextField* input = [[NSTextField alloc] initWithFrame:NSMakeRect(0, 0, 300, 24)];
            [input setStringValue:@"Manual snapshot"];
            [alert setAccessoryView:input];
            [alert addButtonWithTitle:@"Create"];
            [alert addButtonWithTitle:@"Cancel"];
            
            if ([alert runModal] == NSAlertFirstButtonReturn) {
                [self createVersionSnapshot:[input stringValue]];
            }
            return;
        } else if ([characters isEqualToString:@"i"]) {
            // ⌘+I: Show file statistics
            [self showFileStatistics];
            return;
        }
    }
    
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

// Main application
int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C libraries
        editor_library_init();
        file_manager_init();
        
        // Configure hybrid editor
        HybridConfig config = hybrid_get_config();
        config.strict_markdown = true;
        config.enable_bold = true;
        config.enable_italic = true;
        config.enable_highlight = true;
        config.enable_headers = true;
        config.enable_lists = true;
        hybrid_set_config(&config);
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1400, 900);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"ElephantNotes Professional"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create menu bar with professional features
        NSMenu *mainMenu = [[NSMenu alloc] init];
        
        // File menu
        NSMenuItem *fileMenuItem = [[NSMenuItem alloc] init];
        NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
        
        NSMenuItem *newItem = [[NSMenuItem alloc] initWithTitle:@"New" action:@selector(newDocument) keyEquivalent:@"n"];
        NSMenuItem *openItem = [[NSMenuItem alloc] initWithTitle:@"Open..." action:@selector(openDocument) keyEquivalent:@"o"];
        NSMenuItem *saveItem = [[NSMenuItem alloc] initWithTitle:@"Save" action:@selector(saveDocument) keyEquivalent:@"s"];
        NSMenuItem *saveAsItem = [[NSMenuItem alloc] initWithTitle:@"Save As..." action:@selector(saveDocumentAs) keyEquivalent:@"S"];
        
        [fileMenu addItem:newItem];
        [fileMenu addItem:openItem];
        [fileMenu addItem:[NSMenuItem separatorItem]];
        [fileMenu addItem:saveItem];
        [fileMenu addItem:saveAsItem];
        
        // Professional menu
        NSMenuItem *proMenuItem = [[NSMenuItem alloc] init];
        NSMenu *proMenu = [[NSMenu alloc] initWithTitle:@"Professional"];
        
        NSMenuItem *snapshotItem = [[NSMenuItem alloc] initWithTitle:@"Create Version Snapshot..." action:@selector(createVersionSnapshot:) keyEquivalent:@"k"];
        NSMenuItem *statsItem = [[NSMenuItem alloc] initWithTitle:@"File Statistics..." action:@selector(showFileStatistics) keyEquivalent:@"i"];
        NSMenuItem *historyItem = [[NSMenuItem alloc] initWithTitle:@"Show Version History..." action:@selector(showVersionHistory) keyEquivalent:@"h"];
        
        [proMenu addItem:snapshotItem];
        [proMenu addItem:statsItem];
        [proMenu addItem:historyItem];
        
        [fileMenuItem setSubmenu:fileMenu];
        [proMenuItem setSubmenu:proMenu];
        [mainMenu addItem:fileMenuItem];
        [mainMenu addItem:proMenuItem];
        [app setMainMenu:mainMenu];
        
        // Create scroll view
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(0, 0, frame.size.width, frame.size.height)];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
        [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Create the professional text view
        ElephantNotesProfessionalTextView *textView = [[ElephantNotesProfessionalTextView alloc] initWithFrame:[scrollView.contentView bounds]];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        [[textView textContainer] setWidthTracksTextView:YES];
        [[textView textContainer] setContainerSize:NSMakeSize([scrollView.contentView bounds].size.width, FLT_MAX)];
        
        // Set targets for menu items
        [newItem setTarget:textView];
        [openItem setTarget:textView];
        [saveItem setTarget:textView];
        [saveAsItem setTarget:textView];
        [snapshotItem setTarget:textView];
        [statsItem setTarget:textView];
        [historyItem setTarget:textView];
        
        [scrollView setDocumentView:textView];
        [[window contentView] addSubview:scrollView];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        [window makeFirstResponder:textView];
        
        [app run];
        
        // Cleanup
        file_manager_cleanup();
        professional_cleanup();
    }
    return 0;
}