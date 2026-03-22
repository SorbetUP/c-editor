//
//  AppSettings.h
//  ElephantNotes V4 - Système de settings centralisé
//

#import <Foundation/Foundation.h>

@interface AppSettings : NSObject

+ (instancetype)sharedSettings;

// Initialisation
- (void)initializeWithAppPath:(NSString*)appPath;
- (void)loadSettings;
- (void)saveSettings;

// Vault Settings
@property (nonatomic, retain) NSString* currentVaultPath;
@property (nonatomic, retain) NSString* defaultVaultPath;
@property (nonatomic, retain) NSMutableArray* recentVaults;
@property (nonatomic, assign) BOOL autoLoadLastVault;

// UI Settings
@property (nonatomic, assign) CGFloat sidebarWidth;
@property (nonatomic, assign) BOOL showHiddenFiles;
@property (nonatomic, assign) BOOL autoFocusSearch;
@property (nonatomic, retain) NSString* preferredTheme; // "light", "dark", "auto"

// Editor Settings
@property (nonatomic, retain) NSString* defaultFont;
@property (nonatomic, assign) CGFloat defaultFontSize;
@property (nonatomic, assign) BOOL autoSave;
@property (nonatomic, assign) NSInteger undoLevels;
@property (nonatomic, assign) BOOL showLineNumbers;

// Search Settings
@property (nonatomic, assign) BOOL liveSearch;
@property (nonatomic, assign) NSInteger searchDelayMs;
@property (nonatomic, assign) BOOL caseSensitiveSearch;
@property (nonatomic, retain) NSMutableArray* searchHistory;

// Logging Settings
@property (nonatomic, assign) NSInteger logLevel; // Correspond à LogLevel
@property (nonatomic, assign) NSUInteger maxLogSize;
@property (nonatomic, assign) NSInteger maxLogFiles;
@property (nonatomic, assign) BOOL enableFileLogging;

// Window Settings
@property (nonatomic, assign) NSRect lastWindowFrame;
@property (nonatomic, assign) BOOL rememberWindowPosition;
@property (nonatomic, assign) BOOL startMaximized;

// Advanced Settings
@property (nonatomic, assign) BOOL enableAdvancedSearch;
@property (nonatomic, assign) BOOL enableVersionControl;
@property (nonatomic, assign) NSInteger autoSaveInterval; // en secondes, 0 = immédiat
@property (nonatomic, retain) NSString* backupDirectory;

// Méthodes utilitaires
- (void)addRecentVault:(NSString*)vaultPath;
- (void)removeRecentVault:(NSString*)vaultPath;
- (void)addSearchToHistory:(NSString*)searchTerm;
- (NSArray*)getRecentSearches:(NSInteger)count;

// Reset
- (void)resetToDefaults;
- (void)resetSection:(NSString*)section;

// Import/Export
- (BOOL)exportSettingsToFile:(NSString*)filePath;
- (BOOL)importSettingsFromFile:(NSString*)filePath;

// Validation
- (BOOL)validateSettings;
- (NSArray*)getValidationErrors;

@end