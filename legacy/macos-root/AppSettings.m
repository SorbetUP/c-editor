//
//  AppSettings.m
//  ElephantNotes V4 - Système de settings centralisé
//

#import "AppSettings.h"
#import "AppLogger.h"

@interface AppSettings ()
@property (nonatomic, retain) NSString* settingsFilePath;
@property (nonatomic, retain) NSMutableDictionary* settings;
@end

@implementation AppSettings

+ (instancetype)sharedSettings {
    static AppSettings *sharedInstance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        sharedInstance = [[AppSettings alloc] init];
    });
    return sharedInstance;
}

- (instancetype)init {
    self = [super init];
    if (self) {
        _settings = [[NSMutableDictionary alloc] init];
        [self setDefaults];
    }
    return self;
}

- (void)dealloc {
    [_settingsFilePath release];
    [_settings release];
    [_currentVaultPath release];
    [_defaultVaultPath release];
    [_recentVaults release];
    [_preferredTheme release];
    [_defaultFont release];
    [_searchHistory release];
    [_backupDirectory release];
    [super dealloc];
}

- (void)initializeWithAppPath:(NSString*)appPath {
    // Créer le dossier de configuration
    NSString* configDir = [appPath stringByAppendingPathComponent:@"Config"];
    NSFileManager* fm = [NSFileManager defaultManager];
    [fm createDirectoryAtPath:configDir withIntermediateDirectories:YES attributes:nil error:nil];
    
    self.settingsFilePath = [configDir stringByAppendingPathComponent:@"settings.plist"];
    
    LogMessageInfo(LOG_SECTION_APP, @"📋 AppSettings initialisé avec: %@", self.settingsFilePath);
    
    [self loadSettings];
}

- (void)setDefaults {
    // Vault Settings
    self.currentVaultPath = nil;
    self.defaultVaultPath = nil;
    self.recentVaults = [NSMutableArray array];
    self.autoLoadLastVault = YES;
    
    // UI Settings
    self.sidebarWidth = 60.0;
    self.showHiddenFiles = NO;
    self.autoFocusSearch = YES;
    self.preferredTheme = @"auto";
    
    // Editor Settings
    self.defaultFont = @"Helvetica";
    self.defaultFontSize = 14.0;
    self.autoSave = YES;
    self.undoLevels = 50;
    self.showLineNumbers = NO;
    
    // Search Settings
    self.liveSearch = YES;
    self.searchDelayMs = 300;
    self.caseSensitiveSearch = NO;
    self.searchHistory = [NSMutableArray array];
    
    // Logging Settings
    self.logLevel = 0; // DEBUG
    self.maxLogSize = 10 * 1024 * 1024; // 10MB
    self.maxLogFiles = 5;
    self.enableFileLogging = YES;
    
    // Window Settings
    self.lastWindowFrame = NSMakeRect(100, 100, 1200, 800);
    self.rememberWindowPosition = YES;
    self.startMaximized = NO;
    
    // Advanced Settings
    self.enableAdvancedSearch = YES;
    self.enableVersionControl = NO;
    self.autoSaveInterval = 0; // Immédiat
    self.backupDirectory = nil;
    
    LogMessageDebug(LOG_SECTION_APP, @"🔧 Settings par défaut définis");
}

- (void)loadSettings {
    NSFileManager* fm = [NSFileManager defaultManager];
    if (![fm fileExistsAtPath:self.settingsFilePath]) {
        LogMessageInfo(LOG_SECTION_APP, @"📋 Aucun fichier de settings existant, utilisation des défauts");
        [self saveSettings]; // Créer le fichier avec les défauts
        return;
    }
    
    NSDictionary* loadedSettings = [NSDictionary dictionaryWithContentsOfFile:self.settingsFilePath];
    if (!loadedSettings) {
        LogMessageError(LOG_SECTION_APP, @"❌ Impossible de charger les settings depuis: %@", self.settingsFilePath);
        return;
    }
    
    // Vault Settings
    self.currentVaultPath = [loadedSettings objectForKey:@"currentVaultPath"];
    self.defaultVaultPath = [loadedSettings objectForKey:@"defaultVaultPath"];
    self.recentVaults = [[loadedSettings objectForKey:@"recentVaults"] mutableCopy] ?: [NSMutableArray array];
    self.autoLoadLastVault = [[loadedSettings objectForKey:@"autoLoadLastVault"] boolValue];
    
    // UI Settings
    NSNumber* sidebarWidth = [loadedSettings objectForKey:@"sidebarWidth"];
    if (sidebarWidth) self.sidebarWidth = [sidebarWidth floatValue];
    
    NSNumber* showHiddenFiles = [loadedSettings objectForKey:@"showHiddenFiles"];
    if (showHiddenFiles) self.showHiddenFiles = [showHiddenFiles boolValue];
    
    NSNumber* autoFocusSearch = [loadedSettings objectForKey:@"autoFocusSearch"];
    if (autoFocusSearch) self.autoFocusSearch = [autoFocusSearch boolValue];
    
    NSString* theme = [loadedSettings objectForKey:@"preferredTheme"];
    if (theme) self.preferredTheme = theme;
    
    // Editor Settings
    NSString* font = [loadedSettings objectForKey:@"defaultFont"];
    if (font) self.defaultFont = font;
    
    NSNumber* fontSize = [loadedSettings objectForKey:@"defaultFontSize"];
    if (fontSize) self.defaultFontSize = [fontSize floatValue];
    
    NSNumber* autoSave = [loadedSettings objectForKey:@"autoSave"];
    if (autoSave) self.autoSave = [autoSave boolValue];
    
    NSNumber* undoLevels = [loadedSettings objectForKey:@"undoLevels"];
    if (undoLevels) self.undoLevels = [undoLevels integerValue];
    
    NSNumber* showLineNumbers = [loadedSettings objectForKey:@"showLineNumbers"];
    if (showLineNumbers) self.showLineNumbers = [showLineNumbers boolValue];
    
    // Search Settings
    NSNumber* liveSearch = [loadedSettings objectForKey:@"liveSearch"];
    if (liveSearch) self.liveSearch = [liveSearch boolValue];
    
    NSNumber* searchDelay = [loadedSettings objectForKey:@"searchDelayMs"];
    if (searchDelay) self.searchDelayMs = [searchDelay integerValue];
    
    NSNumber* caseSensitive = [loadedSettings objectForKey:@"caseSensitiveSearch"];
    if (caseSensitive) self.caseSensitiveSearch = [caseSensitive boolValue];
    
    self.searchHistory = [[loadedSettings objectForKey:@"searchHistory"] mutableCopy] ?: [NSMutableArray array];
    
    // Logging Settings
    NSNumber* logLevel = [loadedSettings objectForKey:@"logLevel"];
    if (logLevel) self.logLevel = [logLevel integerValue];
    
    NSNumber* maxLogSize = [loadedSettings objectForKey:@"maxLogSize"];
    if (maxLogSize) self.maxLogSize = [maxLogSize unsignedIntegerValue];
    
    NSNumber* maxLogFiles = [loadedSettings objectForKey:@"maxLogFiles"];
    if (maxLogFiles) self.maxLogFiles = [maxLogFiles integerValue];
    
    NSNumber* enableFileLogging = [loadedSettings objectForKey:@"enableFileLogging"];
    if (enableFileLogging) self.enableFileLogging = [enableFileLogging boolValue];
    
    // Window Settings
    NSString* windowFrame = [loadedSettings objectForKey:@"lastWindowFrame"];
    if (windowFrame) self.lastWindowFrame = NSRectFromString(windowFrame);
    
    NSNumber* rememberPosition = [loadedSettings objectForKey:@"rememberWindowPosition"];
    if (rememberPosition) self.rememberWindowPosition = [rememberPosition boolValue];
    
    NSNumber* startMaximized = [loadedSettings objectForKey:@"startMaximized"];
    if (startMaximized) self.startMaximized = [startMaximized boolValue];
    
    // Advanced Settings
    NSNumber* enableAdvancedSearch = [loadedSettings objectForKey:@"enableAdvancedSearch"];
    if (enableAdvancedSearch) self.enableAdvancedSearch = [enableAdvancedSearch boolValue];
    
    NSNumber* enableVersionControl = [loadedSettings objectForKey:@"enableVersionControl"];
    if (enableVersionControl) self.enableVersionControl = [enableVersionControl boolValue];
    
    NSNumber* autoSaveInterval = [loadedSettings objectForKey:@"autoSaveInterval"];
    if (autoSaveInterval) self.autoSaveInterval = [autoSaveInterval integerValue];
    
    NSString* backupDir = [loadedSettings objectForKey:@"backupDirectory"];
    if (backupDir) self.backupDirectory = backupDir;
    
    LogMessageInfo(LOG_SECTION_APP, @"📋 Settings chargés avec succès");
    LogMessageDebug(LOG_SECTION_APP, @"🔧 Vault actuel: %@", self.currentVaultPath ?: @"Aucun");
}

- (void)saveSettings {
    NSMutableDictionary* settingsToSave = [NSMutableDictionary dictionary];
    
    // Vault Settings
    if (self.currentVaultPath) [settingsToSave setObject:self.currentVaultPath forKey:@"currentVaultPath"];
    if (self.defaultVaultPath) [settingsToSave setObject:self.defaultVaultPath forKey:@"defaultVaultPath"];
    [settingsToSave setObject:self.recentVaults forKey:@"recentVaults"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.autoLoadLastVault] forKey:@"autoLoadLastVault"];
    
    // UI Settings
    [settingsToSave setObject:[NSNumber numberWithFloat:self.sidebarWidth] forKey:@"sidebarWidth"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.showHiddenFiles] forKey:@"showHiddenFiles"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.autoFocusSearch] forKey:@"autoFocusSearch"];
    [settingsToSave setObject:self.preferredTheme forKey:@"preferredTheme"];
    
    // Editor Settings
    [settingsToSave setObject:self.defaultFont forKey:@"defaultFont"];
    [settingsToSave setObject:[NSNumber numberWithFloat:self.defaultFontSize] forKey:@"defaultFontSize"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.autoSave] forKey:@"autoSave"];
    [settingsToSave setObject:[NSNumber numberWithInteger:self.undoLevels] forKey:@"undoLevels"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.showLineNumbers] forKey:@"showLineNumbers"];
    
    // Search Settings
    [settingsToSave setObject:[NSNumber numberWithBool:self.liveSearch] forKey:@"liveSearch"];
    [settingsToSave setObject:[NSNumber numberWithInteger:self.searchDelayMs] forKey:@"searchDelayMs"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.caseSensitiveSearch] forKey:@"caseSensitiveSearch"];
    [settingsToSave setObject:self.searchHistory forKey:@"searchHistory"];
    
    // Logging Settings
    [settingsToSave setObject:[NSNumber numberWithInteger:self.logLevel] forKey:@"logLevel"];
    [settingsToSave setObject:[NSNumber numberWithUnsignedInteger:self.maxLogSize] forKey:@"maxLogSize"];
    [settingsToSave setObject:[NSNumber numberWithInteger:self.maxLogFiles] forKey:@"maxLogFiles"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.enableFileLogging] forKey:@"enableFileLogging"];
    
    // Window Settings
    [settingsToSave setObject:NSStringFromRect(self.lastWindowFrame) forKey:@"lastWindowFrame"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.rememberWindowPosition] forKey:@"rememberWindowPosition"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.startMaximized] forKey:@"startMaximized"];
    
    // Advanced Settings
    [settingsToSave setObject:[NSNumber numberWithBool:self.enableAdvancedSearch] forKey:@"enableAdvancedSearch"];
    [settingsToSave setObject:[NSNumber numberWithBool:self.enableVersionControl] forKey:@"enableVersionControl"];
    [settingsToSave setObject:[NSNumber numberWithInteger:self.autoSaveInterval] forKey:@"autoSaveInterval"];
    if (self.backupDirectory) [settingsToSave setObject:self.backupDirectory forKey:@"backupDirectory"];
    
    BOOL success = [settingsToSave writeToFile:self.settingsFilePath atomically:YES];
    if (success) {
        LogMessageDebug(LOG_SECTION_APP, @"💾 Settings sauvegardés avec succès");
    } else {
        LogMessageError(LOG_SECTION_APP, @"❌ Erreur sauvegarde settings: %@", self.settingsFilePath);
    }
}

// Méthodes utilitaires
- (void)addRecentVault:(NSString*)vaultPath {
    if (!vaultPath) return;
    
    // Retirer s'il existe déjà
    [self.recentVaults removeObject:vaultPath];
    
    // Ajouter en première position
    [self.recentVaults insertObject:vaultPath atIndex:0];
    
    // Limiter à 10 vaults récents
    if ([self.recentVaults count] > 10) {
        [self.recentVaults removeObjectsInRange:NSMakeRange(10, [self.recentVaults count] - 10)];
    }
    
    LogMessageDebug(LOG_SECTION_VAULT, @"📁 Vault ajouté aux récents: %@", vaultPath);
    [self saveSettings];
}

- (void)removeRecentVault:(NSString*)vaultPath {
    [self.recentVaults removeObject:vaultPath];
    LogMessageDebug(LOG_SECTION_VAULT, @"🗑️ Vault retiré des récents: %@", vaultPath);
    [self saveSettings];
}

- (void)addSearchToHistory:(NSString*)searchTerm {
    if (!searchTerm || [searchTerm length] == 0) return;
    
    // Retirer s'il existe déjà
    [self.searchHistory removeObject:searchTerm];
    
    // Ajouter en première position
    [self.searchHistory insertObject:searchTerm atIndex:0];
    
    // Limiter à 50 recherches
    if ([self.searchHistory count] > 50) {
        [self.searchHistory removeObjectsInRange:NSMakeRange(50, [self.searchHistory count] - 50)];
    }
    
    LogMessageDebugSub(LOG_SECTION_SEARCH, @"History", @"Recherche ajoutée: %@", searchTerm);
}

- (NSArray*)getRecentSearches:(NSInteger)count {
    NSInteger actualCount = MIN(count, [self.searchHistory count]);
    return [self.searchHistory subarrayWithRange:NSMakeRange(0, actualCount)];
}

- (void)resetToDefaults {
    LogMessageWarning(LOG_SECTION_APP, @"🔄 Reset de tous les settings aux valeurs par défaut");
    [self setDefaults];
    [self saveSettings];
}

- (void)resetSection:(NSString*)section {
    LogMessageWarning(LOG_SECTION_APP, @"🔄 Reset de la section: %@", section);
    // Implémentation spécifique par section si nécessaire
    [self saveSettings];
}

- (BOOL)exportSettingsToFile:(NSString*)filePath {
    NSMutableDictionary* exportDict = [NSMutableDictionary dictionary];
    // Ajouter tous les settings actuels
    [self saveSettings]; // S'assurer que tout est à jour
    NSDictionary* currentSettings = [NSDictionary dictionaryWithContentsOfFile:self.settingsFilePath];
    [exportDict setObject:currentSettings forKey:@"settings"];
    [exportDict setObject:[NSDate date] forKey:@"exportDate"];
    [exportDict setObject:@"ElephantNotes V4" forKey:@"application"];
    
    BOOL success = [exportDict writeToFile:filePath atomically:YES];
    LogMessageInfo(LOG_SECTION_APP, @"📤 Export settings: %@ -> %@", success ? @"SUCCESS" : @"FAILED", filePath);
    return success;
}

- (BOOL)importSettingsFromFile:(NSString*)filePath {
    NSDictionary* importDict = [NSDictionary dictionaryWithContentsOfFile:filePath];
    if (!importDict) {
        LogMessageError(LOG_SECTION_APP, @"❌ Impossible d'importer depuis: %@", filePath);
        return NO;
    }
    
    NSDictionary* importedSettings = [importDict objectForKey:@"settings"];
    if (!importedSettings) {
        LogMessageError(LOG_SECTION_APP, @"❌ Format d'import invalide");
        return NO;
    }
    
    // Sauvegarder les settings actuels
    NSString* backupPath = [self.settingsFilePath stringByAppendingString:@".backup"];
    [[NSFileManager defaultManager] copyItemAtPath:self.settingsFilePath toPath:backupPath error:nil];
    
    // Écrire les nouveaux settings
    BOOL success = [importedSettings writeToFile:self.settingsFilePath atomically:YES];
    if (success) {
        [self loadSettings]; // Recharger
        LogMessageInfo(LOG_SECTION_APP, @"📥 Import settings réussi depuis: %@", filePath);
    } else {
        LogMessageError(LOG_SECTION_APP, @"❌ Échec import settings");
    }
    
    return success;
}

- (BOOL)validateSettings {
    // Valider les chemins
    if (self.currentVaultPath) {
        NSFileManager* fm = [NSFileManager defaultManager];
        if (![fm fileExistsAtPath:self.currentVaultPath]) {
            LogMessageWarning(LOG_SECTION_APP, @"⚠️ Vault path invalide: %@", self.currentVaultPath);
            return NO;
        }
    }
    
    // Valider les valeurs numériques
    if (self.sidebarWidth < 30 || self.sidebarWidth > 300) {
        LogMessageWarning(LOG_SECTION_APP, @"⚠️ Largeur sidebar invalide: %f", self.sidebarWidth);
        return NO;
    }
    
    if (self.defaultFontSize < 8 || self.defaultFontSize > 72) {
        LogMessageWarning(LOG_SECTION_APP, @"⚠️ Taille de police invalide: %f", self.defaultFontSize);
        return NO;
    }
    
    return YES;
}

- (NSArray*)getValidationErrors {
    NSMutableArray* errors = [NSMutableArray array];
    
    if (self.currentVaultPath) {
        NSFileManager* fm = [NSFileManager defaultManager];
        if (![fm fileExistsAtPath:self.currentVaultPath]) {
            [errors addObject:[NSString stringWithFormat:@"Vault path invalide: %@", self.currentVaultPath]];
        }
    }
    
    if (self.sidebarWidth < 30 || self.sidebarWidth > 300) {
        [errors addObject:[NSString stringWithFormat:@"Largeur sidebar invalide: %f", self.sidebarWidth]];
    }
    
    if (self.defaultFontSize < 8 || self.defaultFontSize > 72) {
        [errors addObject:[NSString stringWithFormat:@"Taille de police invalide: %f", self.defaultFontSize]];
    }
    
    return errors;
}

@end