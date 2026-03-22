// SearchManager.m - Implémentation du gestionnaire de recherche optimisé

#import "SearchManager.h"
#include "SearchBridge.h"

// Constantes d'erreur
NSString * const SearchManagerErrorDomain = @"SearchManagerErrorDomain";
NSInteger const SearchManagerErrorInvalidPath = 1001;
NSInteger const SearchManagerErrorIndexingFailed = 1002;
NSInteger const SearchManagerErrorSearchFailed = 1003;
NSInteger const SearchManagerErrorInvalidConfiguration = 1004;

// Callback C vers Objective-C
static void progress_callback_objc(float progress, const char* status, void* user_data) {
    SearchManager *manager = (__bridge SearchManager *)user_data;
    NSString *statusString = [NSString stringWithUTF8String:status];
    
    dispatch_async(dispatch_get_main_queue(), ^{
        if ([manager.delegate respondsToSelector:@selector(searchManager:didUpdateProgress:status:)]) {
            [manager.delegate searchManager:manager didUpdateProgress:progress status:statusString];
        }
    });
}

// Implémentation SearchConfiguration
@implementation SearchConfiguration

+ (instancetype)defaultConfiguration {
    SearchConfiguration *config = [[SearchConfiguration alloc] init];
    config.enableCache = YES;
    config.cacheSize = 500;
    config.enableDimensionReduction = YES;
    config.reducedDimension = 256;
    config.enableEarlyTermination = YES;
    config.earlyStopThreshold = 0.90f;
    config.similarityThreshold = 0.2f;
    config.maxResults = 50;
    return config;
}

@end

// Implémentation ENSearchResult
@implementation ENSearchResult

- (NSString *)description {
    return [NSString stringWithFormat:@"SearchResult: %@ (score: %.3f)", self.fileName, self.relevanceScore];
}

@end

// Implémentation ENFileTreeNode
@implementation ENFileTreeNode

- (instancetype)init {
    self = [super init];
    if (self) {
        _children = [[NSMutableArray alloc] init];
        _isExpanded = NO;
        _isVisible = YES;
        _isSelected = NO;
    }
    return self;
}

- (NSString *)description {
    return [NSString stringWithFormat:@"FileTreeNode: %@ (%@)", self.name, self.isDirectory ? @"dir" : @"file"];
}

@end

// Implémentation SearchStatistics
@implementation SearchStatistics

- (double)cacheEfficiency {
    if (_cacheHits + _cacheMisses == 0) return 0.0;
    return (double)_cacheHits / (_cacheHits + _cacheMisses) * 100.0;
}

@end

// Implémentation SearchManager
@implementation SearchManager {
    SearchEngine *_searchEngine;
    SearchInterface *_searchInterface;
    SearchConfiguration *_configuration;
    SearchStatistics *_statistics;
    NSString *_currentVaultPath;
    BOOL _isIndexing;
    
    // Cache simple
    NSMutableDictionary *_embeddingCache;
    NSMutableDictionary *_queryCache;
    NSMutableArray *_recentQueries;
    
    dispatch_queue_t _searchQueue;
    dispatch_queue_t _indexQueue;
}

static SearchManager *sharedInstance = nil;

+ (instancetype)sharedManager {
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        SearchConfiguration *defaultConfig = [SearchConfiguration defaultConfiguration];
        sharedInstance = [[SearchManager alloc] initWithConfiguration:defaultConfig];
    });
    return sharedInstance;
}

- (instancetype)initWithConfiguration:(SearchConfiguration *)config {
    self = [super init];
    if (self) {
        _configuration = config ?: [SearchConfiguration defaultConfiguration];
        _statistics = [[SearchStatistics alloc] init];
        _embeddingCache = [[NSMutableDictionary alloc] init];
        _queryCache = [[NSMutableDictionary alloc] init];
        _recentQueries = [[NSMutableArray alloc] init];
        _isIndexing = NO;
        
        // Queues pour threading
        _searchQueue = dispatch_queue_create("com.elephantnotes.search", DISPATCH_QUEUE_CONCURRENT);
        _indexQueue = dispatch_queue_create("com.elephantnotes.index", DISPATCH_QUEUE_SERIAL);
        
        [self setupSearchEngine];
        [self setupSearchInterface];
    }
    return self;
}

- (void)dealloc {
    if (_searchEngine) {
        search_engine_destroy(_searchEngine);
    }
    if (_searchInterface) {
        search_interface_destroy(_searchInterface);
    }
    // Pas besoin de [super dealloc] avec ARC
}

- (void)setupSearchEngine {
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_BALANCED;
    config.similarity_threshold = _configuration.similarityThreshold;
    config.max_results = (int)_configuration.maxResults;
    config.enable_caching = _configuration.enableCache;
    
    _searchEngine = search_engine_create(&config);
    
    if (!_searchEngine) {
        NSLog(@"❌ Erreur: Impossible de créer le moteur de recherche");
    }
}

- (void)setupSearchInterface {
    SearchInterfaceConfig config = search_interface_get_default_config();
    config.auto_focus_search = YES;
    config.live_search = YES;
    config.show_hidden_files = NO;
    
    _searchInterface = search_interface_create(&config);
    
    if (!_searchInterface) {
        NSLog(@"❌ Erreur: Impossible de créer l'interface de recherche");
    }
}

// MARK: - Gestion de l'index

- (BOOL)setVaultPath:(NSString *)vaultPath error:(NSError **)error {
    if (!vaultPath || vaultPath.length == 0) {
        if (error) {
            *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                         code:SearchManagerErrorInvalidPath 
                                     userInfo:@{NSLocalizedDescriptionKey: @"Chemin de vault invalide"}];
        }
        return NO;
    }
    
    // Vérifier que le chemin existe
    BOOL isDirectory;
    if (![[NSFileManager defaultManager] fileExistsAtPath:vaultPath isDirectory:&isDirectory] || !isDirectory) {
        if (error) {
            *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                         code:SearchManagerErrorInvalidPath 
                                     userInfo:@{NSLocalizedDescriptionKey: @"Le dossier vault n'existe pas"}];
        }
        return NO;
    }
    
    _currentVaultPath = [vaultPath copy];
    
    // Configurer l'interface de recherche
    const char *vaultPathC = [vaultPath UTF8String];
    bool success = search_interface_set_root_directory(_searchInterface, vaultPathC);
    
    if (!success) {
        if (error) {
            *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                         code:SearchManagerErrorInvalidPath 
                                     userInfo:@{NSLocalizedDescriptionKey: @"Impossible de configurer l'arborescence"}];
        }
        return NO;
    }
    
    return YES;
}

- (void)indexVaultWithCompletion:(void(^)(BOOL success, NSError *error))completion {
    if (!_currentVaultPath) {
        if (completion) {
            NSError *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                                 code:SearchManagerErrorInvalidPath 
                                             userInfo:@{NSLocalizedDescriptionKey: @"Aucun vault configuré"}];
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(NO, error);
            });
        }
        return;
    }
    
    _isIndexing = YES;
    
    dispatch_async(_indexQueue, ^{
        const char *vaultPathC = [self->_currentVaultPath UTF8String];
        
        bool success = search_engine_index_directory(self->_searchEngine, vaultPathC, 
                                                    progress_callback_objc, (__bridge void *)self);
        
        dispatch_async(dispatch_get_main_queue(), ^{
            self->_isIndexing = NO;
            
            if (success) {
                SearchStats stats = search_engine_get_stats(self->_searchEngine);
                self->_statistics.totalFilesIndexed = stats.total_files_indexed;
                self->_statistics.averageQueryTime = stats.avg_query_time_ms;
                self->_statistics.memoryUsage = (NSInteger)stats.memory_usage_mb;
                
                if ([self.delegate respondsToSelector:@selector(searchManager:didCompleteIndexing:)]) {
                    [self.delegate searchManager:self didCompleteIndexing:stats.total_files_indexed];
                }
                
                if (completion) completion(YES, nil);
            } else {
                NSError *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                                     code:SearchManagerErrorIndexingFailed 
                                                 userInfo:@{NSLocalizedDescriptionKey: @"Échec de l'indexation"}];
                if (completion) completion(NO, error);
            }
        });
    });
}

- (void)refreshIndexWithCompletion:(void(^)(BOOL success, NSError *error))completion {
    [self indexVaultWithCompletion:completion];
}

// MARK: - Recherche

- (void)searchWithQuery:(NSString *)query 
             completion:(void(^)(NSArray<SearchResult *> *results, NSError *error))completion {
    [self searchSemanticWithQuery:query completion:completion];
}

- (void)searchSemanticWithQuery:(NSString *)query 
                     completion:(void(^)(NSArray<SearchResult *> *results, NSError *error))completion {
    if (!query || query.length == 0) {
        if (completion) {
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(@[], nil);
            });
        }
        return;
    }
    
    // Vérifier cache
    NSString *cacheKey = [NSString stringWithFormat:@"semantic_%@", query];
    NSArray *cachedResults = _queryCache[cacheKey];
    if (cachedResults) {
        _statistics.cacheHits++;
        if (completion) {
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(cachedResults, nil);
            });
        }
        return;
    }
    
    _statistics.cacheMisses++;
    
    dispatch_async(_searchQueue, ^{
        const char *queryC = [query UTF8String];
        int numResults = 0;
        
        NSDate *startTime = [NSDate date];
        SearchResult_C *results = search_semantic_similar(self->_searchEngine, queryC, &numResults);
        NSTimeInterval searchTime = -[startTime timeIntervalSinceNow] * 1000.0; // en ms
        
        NSMutableArray<ENSearchResult *> *objcResults = [[NSMutableArray alloc] init];
        
        if (results && numResults > 0) {
            for (int i = 0; i < numResults; i++) {
                ENSearchResult *result = [[ENSearchResult alloc] init];
                result.filePath = [NSString stringWithUTF8String:results[i].file_path];
                result.fileName = [NSString stringWithUTF8String:results[i].file_name];
                result.contentPreview = [NSString stringWithUTF8String:results[i].content_preview];
                result.relevanceScore = results[i].relevance_score;
                result.fileSize = results[i].file_size;
                result.lastModified = [NSDate dateWithTimeIntervalSince1970:results[i].last_modified];
                result.matchType = [NSString stringWithUTF8String:results[i].match_type];
                result.matchPosition = results[i].match_position;
                
                [objcResults addObject:result];
            }
            
            search_results_free(results, numResults);
        }
        
        // Mettre en cache
        if (objcResults.count > 0) {
            self->_queryCache[cacheKey] = [objcResults copy];
        }
        
        // Mettre à jour statistiques
        self->_statistics.totalQueries++;
        self->_statistics.averageQueryTime = 
            (self->_statistics.averageQueryTime * (self->_statistics.totalQueries - 1) + searchTime) / 
            self->_statistics.totalQueries;
        
        dispatch_async(dispatch_get_main_queue(), ^{
            [self addRecentQuery:query];
            
            if ([self.delegate respondsToSelector:@selector(searchManager:didFindResults:forQuery:)]) {
                [self.delegate searchManager:self didFindResults:objcResults forQuery:query];
            }
            
            if (completion) completion(objcResults, nil);
        });
    });
}

- (void)searchFilenameWithQuery:(NSString *)query 
                     completion:(void(^)(NSArray<SearchResult *> *results, NSError *error))completion {
    if (!query || query.length == 0) {
        if (completion) {
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(@[], nil);
            });
        }
        return;
    }
    
    dispatch_async(_searchQueue, ^{
        const char *queryC = [query UTF8String];
        int numResults = 0;
        
        SearchResult_C *results = search_filename_fuzzy(self->_searchEngine, queryC, &numResults);
        
        NSMutableArray<ENSearchResult *> *objcResults = [[NSMutableArray alloc] init];
        
        if (results && numResults > 0) {
            for (int i = 0; i < numResults; i++) {
                ENSearchResult *result = [[ENSearchResult alloc] init];
                result.filePath = [NSString stringWithUTF8String:results[i].file_path];
                result.fileName = [NSString stringWithUTF8String:results[i].file_name];
                result.contentPreview = [NSString stringWithUTF8String:results[i].content_preview];
                result.relevanceScore = results[i].relevance_score;
                result.fileSize = results[i].file_size;
                result.lastModified = [NSDate dateWithTimeIntervalSince1970:results[i].last_modified];
                result.matchType = [NSString stringWithUTF8String:results[i].match_type];
                result.matchPosition = results[i].match_position;
                
                [objcResults addObject:result];
            }
            
            search_results_free(results, numResults);
        }
        
        dispatch_async(dispatch_get_main_queue(), ^{
            if (completion) completion(objcResults, nil);
        });
    });
}

// MARK: - Arborescence

- (ENFileTreeNode *)getRootNode {
    if (!_searchInterface) return nil;
    
    int count = 0;
    FileTreeNode_C **visible_nodes = search_interface_get_visible_nodes(_searchInterface, &count);
    
    if (!visible_nodes || count == 0) return nil;
    
    // Convertir le premier nœud (racine)
    ENFileTreeNode *root = [[FileTreeNode alloc] init];
    FileTreeNode_C *rootC = visible_nodes[0];
    
    root.name = [NSString stringWithUTF8String:rootC->name];
    root.fullPath = [NSString stringWithUTF8String:rootC->full_path];
    root.isDirectory = rootC->is_directory;
    root.isExpanded = rootC->is_expanded;
    root.isVisible = rootC->is_visible;
    root.depth = rootC->depth;
    root.fileSize = rootC->file_size;
    root.lastModified = [NSDate dateWithTimeIntervalSince1970:rootC->last_modified];
    
    return root;
}

- (NSArray<ENFileTreeNode *> *)getVisibleNodes {
    if (!_searchInterface) return @[];
    
    int count = 0;
    FileTreeNode_C **visible_nodes = search_interface_get_visible_nodes(_searchInterface, &count);
    
    NSMutableArray<ENFileTreeNode *> *nodes = [[NSMutableArray alloc] init];
    
    for (int i = 0; i < count; i++) {
        ENFileTreeNode *node = [[FileTreeNode alloc] init];
        FileTreeNode_C *nodeC = visible_nodes[i];
        
        node.name = [NSString stringWithUTF8String:nodeC->name];
        node.fullPath = [NSString stringWithUTF8String:nodeC->full_path];
        node.isDirectory = nodeC->is_directory;
        node.isExpanded = nodeC->is_expanded;
        node.isVisible = nodeC->is_visible;
        node.depth = nodeC->depth;
        node.fileSize = nodeC->file_size;
        node.lastModified = [NSDate dateWithTimeIntervalSince1970:nodeC->last_modified];
        
        [nodes addObject:node];
    }
    
    return [nodes copy];
}

- (BOOL)expandNode:(ENFileTreeNode *)node error:(NSError **)error {
    if (!node || !_searchInterface) return NO;
    
    const char *pathC = [node.fullPath UTF8String];
    bool success = search_interface_expand_node(_searchInterface, pathC);
    
    if (success) {
        node.isExpanded = YES;
        
        if ([self.delegate respondsToSelector:@selector(searchManager:didExpandDirectory:)]) {
            [self.delegate searchManager:self didExpandDirectory:node.fullPath];
        }
    } else if (error) {
        *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                     code:SearchManagerErrorInvalidPath 
                                 userInfo:@{NSLocalizedDescriptionKey: @"Impossible d'étendre le nœud"}];
    }
    
    return success;
}

- (BOOL)collapseNode:(ENFileTreeNode *)node error:(NSError **)error {
    if (!node || !_searchInterface) return NO;
    
    const char *pathC = [node.fullPath UTF8String];
    bool success = search_interface_collapse_node(_searchInterface, pathC);
    
    if (success) {
        node.isExpanded = NO;
    } else if (error) {
        *error = [NSError errorWithDomain:SearchManagerErrorDomain 
                                     code:SearchManagerErrorInvalidPath 
                                 userInfo:@{NSLocalizedDescriptionKey: @"Impossible de réduire le nœud"}];
    }
    
    return success;
}

// MARK: - Suggestions et historique

- (NSArray<NSString *> *)getSuggestionsForPartialQuery:(NSString *)partialQuery {
    // Implémentation simple basée sur l'historique
    NSMutableArray *suggestions = [[NSMutableArray alloc] init];
    
    for (NSString *recentQuery in _recentQueries) {
        if ([recentQuery.lowercaseString containsString:partialQuery.lowercaseString]) {
            [suggestions addObject:recentQuery];
            if (suggestions.count >= 5) break;
        }
    }
    
    return [suggestions copy];
}

- (NSArray<NSString *> *)getRecentQueries {
    return [_recentQueries copy];
}

- (void)addRecentQuery:(NSString *)query {
    if (!query || query.length == 0) return;
    
    // Supprimer si déjà présent
    [_recentQueries removeObject:query];
    
    // Ajouter en début
    [_recentQueries insertObject:query atIndex:0];
    
    // Limiter à 20 entrées
    if (_recentQueries.count > 20) {
        [_recentQueries removeObjectsInRange:NSMakeRange(20, _recentQueries.count - 20)];
    }
}

// MARK: - Cache et optimisations

- (void)clearCache {
    [_embeddingCache removeAllObjects];
    [_queryCache removeAllObjects];
    _statistics.cacheHits = 0;
    _statistics.cacheMisses = 0;
}

- (void)warmCache {
    // Pré-charger avec des requêtes communes
    NSArray *commonQueries = @[@"note", @"document", @"project", @"meeting", @"idea"];
    
    for (NSString *query in commonQueries) {
        [self searchSemanticWithQuery:query completion:nil];
    }
}

- (SearchStatistics *)getDetailedStatistics {
    return _statistics;
}

// MARK: - Utilitaires

- (void)enableDebugMode:(BOOL)enabled {
    if (_searchEngine) {
        search_engine_enable_debug(_searchEngine, enabled);
    }
}

- (NSString *)getDebugInformation {
    if (!_searchEngine) return @"Moteur de recherche non initialisé";
    
    char *debugInfo = search_engine_get_debug_info(_searchEngine);
    NSString *info = [NSString stringWithUTF8String:debugInfo];
    free(debugInfo);
    
    return info;
}

@end