//
//  ENSearchTab.h
//  ElephantNotes V4 - Module Search (interface markdown)
//

#import "ENTabBase.h"
#include "advanced_search.h"

@interface ENSearchTab : ENTabBase

@property (nonatomic, assign) SearchEngine* searchEngine;
@property (nonatomic, assign) bool searchIndexReady;
@property (nonatomic, copy) NSString* currentQuery;
@property (nonatomic, strong) NSArray* searchResults;
@property (nonatomic, assign) bool isSearching;

// Méthodes spécifiques à la recherche
- (NSArray*)getAllFilesInVault;
- (NSString*)generateSearchContent;
- (void)initializeSearchEngine;
- (void)performSearch:(NSString*)query;
- (void)clearSearch;
- (NSString*)formatSearchResults:(NSArray*)results forQuery:(NSString*)query;

@end