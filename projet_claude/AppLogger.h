//
//  AppLogger.h
//  ElephantNotes V4 - Système de logging centralisé
//

#import <Foundation/Foundation.h>

typedef enum {
    LOG_LEVEL_DEBUG = 0,
    LOG_LEVEL_INFO = 1,
    LOG_LEVEL_WARNING = 2,
    LOG_LEVEL_ERROR = 3,
    LOG_LEVEL_CRITICAL = 4
} LogLevel;

typedef enum {
    LOG_SECTION_APP = 0,
    LOG_SECTION_UI = 1,
    LOG_SECTION_SEARCH = 2,
    LOG_SECTION_VAULT = 3,
    LOG_SECTION_EDITOR = 4,
    LOG_SECTION_FILE_MANAGER = 5
} LogSection;

@interface AppLogger : NSObject

+ (instancetype)sharedLogger;

// Initialisation
- (void)initializeWithAppPath:(NSString*)appPath;
- (void)createLogDirectories;

// Logging principal
- (void)log:(LogLevel)level section:(LogSection)section message:(NSString*)message;
- (void)logDebug:(LogSection)section message:(NSString*)message;
- (void)logInfo:(LogSection)section message:(NSString*)message;
- (void)logWarning:(LogSection)section message:(NSString*)message;
- (void)logError:(LogSection)section message:(NSString*)message;
- (void)logCritical:(LogSection)section message:(NSString*)message;

// Logging avec sous-sections
- (void)log:(LogLevel)level section:(LogSection)section subsection:(NSString*)subsection message:(NSString*)message;

// Logging de crashes et exceptions
- (void)logException:(NSException*)exception section:(LogSection)section context:(NSString*)context;
- (void)logCrash:(NSString*)crashInfo section:(LogSection)section;

// Gestion des fichiers de log
- (void)rotateLogs;
- (void)clearOldLogs;
- (NSString*)getLogPath:(LogSection)section;
- (NSArray*)getRecentLogs:(LogSection)section count:(NSInteger)count;

// Configuration
- (void)setLogLevel:(LogLevel)level;
- (void)setMaxLogSize:(NSUInteger)maxSize;
- (void)setMaxLogFiles:(NSInteger)maxFiles;

@end

// Utiliser des appels directs plutôt que des macros :
// [[AppLogger sharedLogger] logInfo:LOG_SECTION_APP message:@"Mon message"];
// [[AppLogger sharedLogger] log:LOG_LEVEL_INFO section:LOG_SECTION_APP subsection:@"SubSection" message:@"Mon message"];

// Fonctions helper pour formatage
void LogMessageInfo(LogSection section, NSString* format, ...);
void LogMessageDebug(LogSection section, NSString* format, ...);
void LogMessageWarning(LogSection section, NSString* format, ...);
void LogMessageError(LogSection section, NSString* format, ...);
void LogMessageCritical(LogSection section, NSString* format, ...);

void LogMessageInfoSub(LogSection section, NSString* subsection, NSString* format, ...);
void LogMessageDebugSub(LogSection section, NSString* subsection, NSString* format, ...);
void LogMessageWarningSub(LogSection section, NSString* subsection, NSString* format, ...);
void LogMessageErrorSub(LogSection section, NSString* subsection, NSString* format, ...);