//
//  ENFilesTab.h
//  ElephantNotes V4 - Module Files pour la gestion des fichiers et dossiers
//

#import "ENTabBase.h"

@interface ENFilesTab : ENTabBase

@property (nonatomic, copy) NSString* currentDirectory;
@property (nonatomic, strong) NSArray* currentFiles;
@property (nonatomic, assign) BOOL showHiddenFiles;

// Méthodes de navigation
- (void)navigateToDirectory:(NSString*)path;
- (void)navigateUp;
- (void)refreshCurrentDirectory;

// Méthodes de gestion des fichiers
- (void)createNewFolder:(NSString*)folderName;
- (void)deleteFile:(NSString*)filePath;
- (void)renameFile:(NSString*)oldPath toName:(NSString*)newName;

// Méthodes d'interface
- (NSString*)generateFilesContent;
- (NSString*)generateDirectoryListContent;
- (NSString*)generateVaultManagementContent;

@end