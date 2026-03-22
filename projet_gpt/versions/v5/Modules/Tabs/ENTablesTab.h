//
//  ENTablesTab.h
//  ElephantNotes V5 - Module Tables pour la création et gestion des tableaux
//

#import "ENTabBase.h"

@interface ENTablesTab : ENTabBase

@property (nonatomic, copy) NSString* currentTableMarkdown;
@property (nonatomic, assign) NSInteger tableRows;
@property (nonatomic, assign) NSInteger tableCols;
@property (nonatomic, assign) BOOL isEditingTable;

// Méthodes de gestion des tableaux
- (void)createNewTable:(NSInteger)rows columns:(NSInteger)cols;
- (void)addTableRow;
- (void)addTableColumn;
- (void)removeTableRow:(NSInteger)row;
- (void)removeTableColumn:(NSInteger)col;

// Méthodes d'interface
- (NSString*)generateTablesContent;
- (NSString*)generateTableExamples;
- (NSString*)generateTableHelp;

// Utilitaires
- (NSString*)parseTableFromMarkdown:(NSString*)markdown;
- (NSString*)formatTableMarkdown:(NSArray*)rows;

@end