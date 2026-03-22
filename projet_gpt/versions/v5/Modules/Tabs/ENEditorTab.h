//
//  ENEditorTab.h
//  ElephantNotes V4 - Module Editor pour l'édition de fichiers
//

#import "ENTabBase.h"

@interface ENEditorTab : ENTabBase

@property (nonatomic, copy) NSString* currentFilePath;
@property (nonatomic, copy) NSString* currentFileName;
@property (nonatomic, copy) NSString* currentFileContent;
@property (nonatomic, assign) BOOL hasUnsavedChanges;
@property (nonatomic, assign) BOOL isFileLoaded;

// Méthodes de gestion des fichiers
- (void)loadFile:(NSString*)filePath;
- (void)saveCurrentFile;
- (void)saveAsNewFile:(NSString*)filePath;
- (void)createNewFile;
- (void)closeCurrentFile;
- (void)setDefaultCreationDirectory:(NSString*)directory;
- (void)prepareNewFileInDirectory:(NSString*)directory;
- (BOOL)openNoteLink:(NSString*)linkPath;

// Méthodes d'édition
- (NSString*)generateEditorContent;
- (void)updateFileContent:(NSString*)content;

// Méthodes d'interface
- (NSString*)generateFileListContent;
- (NSString*)generateWelcomeContent;

@end
