// SearchManager.h - Interface Objective-C pour le système de recherche optimisé
// Intègre advanced_search + search_interface + optimisations

#import <Foundation/Foundation.h>

@class ENSearchResult, ENFileTreeNode;

// Configuration de recherche
@interface SearchConfiguration : NSObject

@property (assign) BOOL enableCache;
@property (assign) NSInteger cacheSize;
@property (assign) BOOL enableDimensionReduction;
@property (assign) NSInteger reducedDimension;
@property (assign) BOOL enableEarlyTermination;
@property (assign) float earlyStopThreshold;
@property (assign) float similarityThreshold;
@property (assign) NSInteger maxResults;

+ (instancetype)defaultConfiguration;

@end

// Résultat de recherche Objective-C
@interface ENSearchResult : NSObject

@property (strong) NSString *filePath;
@property (strong) NSString *fileName;
@property (strong) NSString *contentPreview;
@property (assign) float relevanceScore;
@property (assign) NSInteger fileSize;
@property (strong) NSDate *lastModified;
@property (strong) NSString *matchType;
@property (assign) NSInteger matchPosition;

@end

// Nœud de l'arborescence Objective-C
@interface ENFileTreeNode : NSObject

@property (strong) NSString *name;
@property (strong) NSString *fullPath;
@property (assign) BOOL isDirectory;
@property (assign) BOOL isExpanded;
@property (assign) BOOL isVisible;
@property (assign) NSInteger depth;
@property (nonatomic, weak) ENFileTreeNode *parent;
@property (strong) NSMutableArray<ENFileTreeNode *> *children;
@property (assign) NSInteger fileSize;
@property (strong) NSDate *lastModified;
@property (assign) BOOL isSelected;

@end

// Statistiques de performance
@interface SearchStatistics : NSObject

@property (assign) NSInteger totalFilesIndexed;
@property (assign) NSInteger totalQueries;
@property (assign) double averageQueryTime;
@property (assign) NSInteger cacheHits;
@property (assign) NSInteger cacheMisses;
@property (nonatomic, readonly) double cacheEfficiency;
@property (assign) NSInteger memoryUsage;

@end

// Protocole pour callbacks de recherche
@protocol SearchManagerDelegate <NSObject>

@optional
- (void)searchManager:(id)manager didUpdateProgress:(float)progress status:(NSString *)status;
- (void)searchManager:(id)manager didCompleteIndexing:(NSInteger)filesIndexed;
- (void)searchManager:(id)manager didFindResults:(NSArray<ENSearchResult *> *)results forQuery:(NSString *)query;
- (void)searchManager:(id)manager didExpandDirectory:(NSString *)directoryPath;

@end

// Gestionnaire principal de recherche
@interface SearchManager : NSObject

@property (nonatomic, weak) id<SearchManagerDelegate> delegate;
@property (strong, readonly) SearchConfiguration *configuration;
@property (strong, readonly) SearchStatistics *statistics;
@property (strong, readonly) NSString *currentVaultPath;
@property (assign, readonly) BOOL isIndexing;

// Initialisation
- (instancetype)initWithConfiguration:(SearchConfiguration *)config;
+ (instancetype)sharedManager;

// Gestion de l'index
- (BOOL)setVaultPath:(NSString *)vaultPath error:(NSError **)error;
- (void)indexVaultWithCompletion:(void(^)(BOOL success, NSError *error))completion;
- (void)refreshIndexWithCompletion:(void(^)(BOOL success, NSError *error))completion;

// Recherche
- (void)searchWithQuery:(NSString *)query 
             completion:(void(^)(NSArray<ENSearchResult *> *results, NSError *error))completion;

- (void)searchSemanticWithQuery:(NSString *)query 
                     completion:(void(^)(NSArray<ENSearchResult *> *results, NSError *error))completion;

- (void)searchFilenameWithQuery:(NSString *)query 
                     completion:(void(^)(NSArray<ENSearchResult *> *results, NSError *error))completion;

// Arborescence de fichiers
- (ENFileTreeNode *)getRootNode;
- (BOOL)expandNode:(ENFileTreeNode *)node error:(NSError **)error;
- (BOOL)collapseNode:(ENFileTreeNode *)node error:(NSError **)error;
- (NSArray<ENFileTreeNode *> *)getVisibleNodes;
- (ENFileTreeNode *)findNodeAtPath:(NSString *)path;

// Suggestions et historique
- (NSArray<NSString *> *)getSuggestionsForPartialQuery:(NSString *)partialQuery;
- (NSArray<NSString *> *)getRecentQueries;
- (void)addRecentQuery:(NSString *)query;

// Cache et optimisations
- (void)clearCache;
- (void)warmCache;
- (SearchStatistics *)getDetailedStatistics;

// Utilitaires
- (void)enableDebugMode:(BOOL)enabled;
- (NSString *)getDebugInformation;

@end

// Constantes d'erreur
extern NSString * const SearchManagerErrorDomain;
extern NSInteger const SearchManagerErrorInvalidPath;
extern NSInteger const SearchManagerErrorIndexingFailed;
extern NSInteger const SearchManagerErrorSearchFailed;
extern NSInteger const SearchManagerErrorInvalidConfiguration;