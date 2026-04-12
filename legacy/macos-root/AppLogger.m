//
//  AppLogger.m
//  ElephantNotes V4 - Système de logging centralisé
//

#import "AppLogger.h"

@interface AppLogger ()
@property (nonatomic, strong) NSString* logBasePath;
@property (nonatomic, strong) NSMutableDictionary* logFileHandles;
@property (nonatomic, strong) dispatch_queue_t logQueue;
@property (nonatomic, assign) LogLevel currentLogLevel;
@property (nonatomic, assign) NSUInteger maxLogSize;
@property (nonatomic, assign) NSInteger maxLogFiles;
@end

@implementation AppLogger

+ (instancetype)sharedLogger {
    static AppLogger *sharedInstance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        sharedInstance = [[AppLogger alloc] init];
    });
    return sharedInstance;
}

- (instancetype)init {
    self = [super init];
    if (self) {
        _logFileHandles = [[NSMutableDictionary alloc] init];
        _logQueue = dispatch_queue_create("com.elephantnotes.logger", DISPATCH_QUEUE_SERIAL);
        _currentLogLevel = LOG_LEVEL_DEBUG;
        _maxLogSize = 10 * 1024 * 1024; // 10MB par fichier
        _maxLogFiles = 5; // Garder 5 fichiers de log par section
    }
    return self;
}

- (void)dealloc {
    // Fermer tous les file handles
    for (NSFileHandle* handle in [_logFileHandles allValues]) {
        [handle closeFile];
    }
    [_logFileHandles release];
    [_logBasePath release];
    [super dealloc];
}

- (void)initializeWithAppPath:(NSString*)appPath {
    self.logBasePath = [appPath stringByAppendingPathComponent:@"Logs"];
    [self createLogDirectories];
    
    // Log de démarrage
    [self logInfo:LOG_SECTION_APP message:@"🚀 AppLogger initialisé"];
    [self logInfo:LOG_SECTION_APP message:[NSString stringWithFormat:@"📁 Logs dans: %@", self.logBasePath]];
}

- (void)createLogDirectories {
    NSFileManager* fm = [NSFileManager defaultManager];
    NSError* error = nil;
    
    // Créer le dossier principal des logs
    if (![fm createDirectoryAtPath:self.logBasePath 
           withIntermediateDirectories:YES 
                            attributes:nil 
                                 error:&error]) {
        NSLog(@"❌ Erreur création dossier logs: %@", error.localizedDescription);
        return;
    }
    
    // Créer un sous-dossier pour chaque section
    NSArray* sections = @[@"App", @"UI", @"Search", @"Vault", @"Editor", @"FileManager"];
    for (NSString* section in sections) {
        NSString* sectionPath = [self.logBasePath stringByAppendingPathComponent:section];
        [fm createDirectoryAtPath:sectionPath 
          withIntermediateDirectories:YES 
                           attributes:nil 
                                error:nil];
    }
}

- (NSString*)sectionToString:(LogSection)section {
    switch (section) {
        case LOG_SECTION_APP: return @"App";
        case LOG_SECTION_UI: return @"UI";
        case LOG_SECTION_SEARCH: return @"Search";
        case LOG_SECTION_VAULT: return @"Vault";
        case LOG_SECTION_EDITOR: return @"Editor";
        case LOG_SECTION_FILE_MANAGER: return @"FileManager";
        default: return @"Unknown";
    }
}

- (NSString*)levelToString:(LogLevel)level {
    switch (level) {
        case LOG_LEVEL_DEBUG: return @"DEBUG";
        case LOG_LEVEL_INFO: return @"INFO";
        case LOG_LEVEL_WARNING: return @"WARN";
        case LOG_LEVEL_ERROR: return @"ERROR";
        case LOG_LEVEL_CRITICAL: return @"CRIT";
        default: return @"UNKNOWN";
    }
}

- (NSString*)levelToEmoji:(LogLevel)level {
    switch (level) {
        case LOG_LEVEL_DEBUG: return @"🔍";
        case LOG_LEVEL_INFO: return @"ℹ️";
        case LOG_LEVEL_WARNING: return @"⚠️";
        case LOG_LEVEL_ERROR: return @"❌";
        case LOG_LEVEL_CRITICAL: return @"🚨";
        default: return @"❓";
    }
}

- (NSString*)getLogFilePath:(LogSection)section {
    NSString* sectionName = [self sectionToString:section];
    NSString* sectionDir = [self.logBasePath stringByAppendingPathComponent:sectionName];
    
    // Format: ElephantNotes_Section_YYYY-MM-DD.log
    NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
    [formatter setDateFormat:@"yyyy-MM-dd"];
    NSString* dateStr = [formatter stringFromDate:[NSDate date]];
    [formatter release];
    
    NSString* filename = [NSString stringWithFormat:@"ElephantNotes_%@_%@.log", sectionName, dateStr];
    return [sectionDir stringByAppendingPathComponent:filename];
}

- (NSFileHandle*)getFileHandle:(LogSection)section {
    NSString* key = [self sectionToString:section];
    NSFileHandle* handle = [_logFileHandles objectForKey:key];
    
    if (!handle) {
        NSString* logPath = [self getLogFilePath:section];
        
        // Créer le fichier s'il n'existe pas
        NSFileManager* fm = [NSFileManager defaultManager];
        if (![fm fileExistsAtPath:logPath]) {
            [@"" writeToFile:logPath atomically:YES encoding:NSUTF8StringEncoding error:nil];
        }
        
        handle = [NSFileHandle fileHandleForWritingAtPath:logPath];
        if (handle) {
            [handle seekToEndOfFile];
            [_logFileHandles setObject:handle forKey:key];
        }
    }
    
    return handle;
}

- (void)log:(LogLevel)level section:(LogSection)section message:(NSString*)message {
    [self log:level section:section subsection:nil message:message];
}

- (void)log:(LogLevel)level section:(LogSection)section subsection:(NSString*)subsection message:(NSString*)message {
    if (level < self.currentLogLevel) {
        return; // Niveau de log trop bas
    }
    
    dispatch_async(self.logQueue, ^{
        @autoreleasepool {
            // Format du timestamp
            NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
            [formatter setDateFormat:@"yyyy-MM-dd HH:mm:ss.SSS"];
            NSString* timestamp = [formatter stringFromDate:[NSDate date]];
            [formatter release];
            
            // Format du message
            NSString* sectionStr = [self sectionToString:section];
            NSString* levelStr = [self levelToString:level];
            NSString* emoji = [self levelToEmoji:level];
            
            NSString* logLine;
            if (subsection) {
                logLine = [NSString stringWithFormat:@"[%@] %@ %@/%@ - %@ %@\n", 
                          timestamp, emoji, sectionStr, subsection, levelStr, message];
            } else {
                logLine = [NSString stringWithFormat:@"[%@] %@ %@ - %@ %@\n", 
                          timestamp, emoji, sectionStr, levelStr, message];
            }
            
            // Écrire dans le fichier
            NSFileHandle* handle = [self getFileHandle:section];
            if (handle) {
                NSData* data = [logLine dataUsingEncoding:NSUTF8StringEncoding];
                [handle writeData:data];
                [handle synchronizeFile];
                
                // Vérifier la taille du fichier et faire la rotation si nécessaire
                [self checkAndRotateLog:section];
            }
            
            // Aussi écrire dans NSLog pour le debug
            NSLog(@"%@", [logLine stringByTrimmingCharactersInSet:[NSCharacterSet newlineCharacterSet]]);
        }
    });
}

- (void)checkAndRotateLog:(LogSection)section {
    NSString* logPath = [self getLogFilePath:section];
    NSFileManager* fm = [NSFileManager defaultManager];
    
    NSDictionary* attrs = [fm attributesOfItemAtPath:logPath error:nil];
    if (attrs) {
        NSNumber* fileSize = [attrs objectForKey:NSFileSize];
        if ([fileSize unsignedIntegerValue] > self.maxLogSize) {
            [self rotateLog:section];
        }
    }
}

- (void)rotateLog:(LogSection)section {
    NSString* key = [self sectionToString:section];
    
    // Fermer le file handle actuel
    NSFileHandle* handle = [_logFileHandles objectForKey:key];
    if (handle) {
        [handle closeFile];
        [_logFileHandles removeObjectForKey:key];
    }
    
    // Renommer le fichier actuel avec un timestamp
    NSString* currentPath = [self getLogFilePath:section];
    NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
    [formatter setDateFormat:@"yyyy-MM-dd_HH-mm-ss"];
    NSString* timestampStr = [formatter stringFromDate:[NSDate date]];
    [formatter release];
    
    NSString* rotatedPath = [currentPath stringByReplacingOccurrencesOfString:@".log" 
                                                                   withString:[NSString stringWithFormat:@"_%@.log", timestampStr]];
    
    NSFileManager* fm = [NSFileManager defaultManager];
    [fm moveItemAtPath:currentPath toPath:rotatedPath error:nil];
    
    // Nettoyer les anciens logs
    [self cleanOldLogs:section];
}

- (void)cleanOldLogs:(LogSection)section {
    NSString* sectionName = [self sectionToString:section];
    NSString* sectionDir = [self.logBasePath stringByAppendingPathComponent:sectionName];
    
    NSFileManager* fm = [NSFileManager defaultManager];
    NSArray* files = [fm contentsOfDirectoryAtPath:sectionDir error:nil];
    
    // Filtrer les fichiers de log et les trier par date
    NSMutableArray* logFiles = [NSMutableArray array];
    for (NSString* file in files) {
        if ([file hasPrefix:@"ElephantNotes_"] && [file hasSuffix:@".log"]) {
            NSString* filePath = [sectionDir stringByAppendingPathComponent:file];
            NSDictionary* attrs = [fm attributesOfItemAtPath:filePath error:nil];
            if (attrs) {
                NSDate* modDate = [attrs objectForKey:NSFileModificationDate];
                [logFiles addObject:@{@"path": filePath, @"date": modDate}];
            }
        }
    }
    
    // Trier par date (plus récent en premier)
    [logFiles sortUsingComparator:^NSComparisonResult(NSDictionary* obj1, NSDictionary* obj2) {
        NSDate* date1 = [obj1 objectForKey:@"date"];
        NSDate* date2 = [obj2 objectForKey:@"date"];
        return [date2 compare:date1];
    }];
    
    // Supprimer les fichiers en excès
    if ([logFiles count] > self.maxLogFiles) {
        for (NSInteger i = self.maxLogFiles; i < [logFiles count]; i++) {
            NSString* pathToDelete = [[logFiles objectAtIndex:i] objectForKey:@"path"];
            [fm removeItemAtPath:pathToDelete error:nil];
        }
    }
}

// Méthodes de convenance
- (void)logDebug:(LogSection)section message:(NSString*)message {
    [self log:LOG_LEVEL_DEBUG section:section message:message];
}

- (void)logInfo:(LogSection)section message:(NSString*)message {
    [self log:LOG_LEVEL_INFO section:section message:message];
}

- (void)logWarning:(LogSection)section message:(NSString*)message {
    [self log:LOG_LEVEL_WARNING section:section message:message];
}

- (void)logError:(LogSection)section message:(NSString*)message {
    [self log:LOG_LEVEL_ERROR section:section message:message];
}

- (void)logCritical:(LogSection)section message:(NSString*)message {
    [self log:LOG_LEVEL_CRITICAL section:section message:message];
}

- (void)logException:(NSException*)exception section:(LogSection)section context:(NSString*)context {
    NSString* message = [NSString stringWithFormat:@"EXCEPTION in %@: %@ - %@\nStack: %@", 
                        context, exception.name, exception.reason, exception.callStackSymbols];
    [self log:LOG_LEVEL_CRITICAL section:section message:message];
}

- (void)logCrash:(NSString*)crashInfo section:(LogSection)section {
    NSString* message = [NSString stringWithFormat:@"CRASH: %@", crashInfo];
    [self log:LOG_LEVEL_CRITICAL section:section message:message];
}

// Getters
- (NSString*)getLogPath:(LogSection)section {
    return [self getLogFilePath:section];
}

- (NSArray*)getRecentLogs:(LogSection)section count:(NSInteger)count {
    NSString* logPath = [self getLogFilePath:section];
    NSString* content = [NSString stringWithContentsOfFile:logPath encoding:NSUTF8StringEncoding error:nil];
    if (!content) return @[];
    
    NSArray* lines = [content componentsSeparatedByString:@"\n"];
    NSInteger startIndex = MAX(0, [lines count] - count);
    NSRange range = NSMakeRange(startIndex, [lines count] - startIndex);
    return [lines subarrayWithRange:range];
}

// Setters de configuration
- (void)setLogLevel:(LogLevel)level {
    self.currentLogLevel = level;
}

- (void)setMaxLogSize:(NSUInteger)maxSize {
    self.maxLogSize = maxSize;
}

- (void)setMaxLogFiles:(NSInteger)maxFiles {
    self.maxLogFiles = maxFiles;
}

- (void)rotateLogs {
    for (LogSection section = LOG_SECTION_APP; section <= LOG_SECTION_FILE_MANAGER; section++) {
        [self rotateLog:section];
    }
}

- (void)clearOldLogs {
    for (LogSection section = LOG_SECTION_APP; section <= LOG_SECTION_FILE_MANAGER; section++) {
        [self cleanOldLogs:section];
    }
}

@end

// Fonctions helper pour éviter les problèmes de macros
void LogMessageInfo(LogSection section, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] logInfo:section message:message];
    [message release];
}

void LogMessageDebug(LogSection section, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] logDebug:section message:message];
    [message release];
}

void LogMessageWarning(LogSection section, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] logWarning:section message:message];
    [message release];
}

void LogMessageError(LogSection section, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] logError:section message:message];
    [message release];
}

void LogMessageCritical(LogSection section, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] logCritical:section message:message];
    [message release];
}

void LogMessageInfoSub(LogSection section, NSString* subsection, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] log:LOG_LEVEL_INFO section:section subsection:subsection message:message];
    [message release];
}

void LogMessageDebugSub(LogSection section, NSString* subsection, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] log:LOG_LEVEL_DEBUG section:section subsection:subsection message:message];
    [message release];
}

void LogMessageWarningSub(LogSection section, NSString* subsection, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] log:LOG_LEVEL_WARNING section:section subsection:subsection message:message];
    [message release];
}

void LogMessageErrorSub(LogSection section, NSString* subsection, NSString* format, ...) {
    va_list args;
    va_start(args, format);
    NSString* message = [[NSString alloc] initWithFormat:format arguments:args];
    va_end(args);
    [[AppLogger sharedLogger] log:LOG_LEVEL_ERROR section:section subsection:subsection message:message];
    [message release];
}