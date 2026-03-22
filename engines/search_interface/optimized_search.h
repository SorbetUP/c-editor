// optimized_search.h - Version optimisée du moteur de recherche
// Implémente les optimisations identifiées par le benchmark réaliste

#ifndef OPTIMIZED_SEARCH_H
#define OPTIMIZED_SEARCH_H

#include "../advanced_search/advanced_search.h"
#include <pthread.h>

#ifdef __cplusplus
extern "C" {
#endif

// Configuration d'optimisations
typedef struct {
    // Cache intelligent
    bool enable_lru_cache;           // Cache LRU pour embeddings
    int cache_size;                  // Taille du cache (nombre d'embeddings)
    bool enable_query_cache;         // Cache des résultats de requêtes
    int query_cache_size;           // Taille du cache de requêtes
    
    // Optimisations embedding
    bool enable_embedding_quantization; // Quantification des embeddings
    int quantized_bits;             // Nombre de bits pour quantification (8/16)
    bool enable_dimension_reduction; // Réduction de dimension PCA/SVD
    int reduced_dimension;          // Dimension réduite (128/256 vs 768)
    
    // Optimisations algorithme
    bool enable_early_termination;  // Arrêt précoce de recherche
    float early_stop_threshold;     // Seuil d'arrêt précoce
    bool enable_parallel_search;    // Recherche parallèle multi-thread
    int num_search_threads;         // Nombre de threads de recherche
    
    // Optimisations I/O
    bool enable_async_indexing;     // Indexation asynchrone
    bool enable_mmap_files;         // Memory mapping des fichiers
    bool enable_file_buffering;     // Buffering des lectures fichier
    int buffer_size_kb;             // Taille buffer en KB
    
    // Optimisations mémoire
    bool enable_memory_pooling;     // Pool de mémoire pré-allouée
    bool enable_zero_copy;          // Éviter copies mémoire inutiles
    bool enable_compress_embeddings; // Compression des embeddings stockés
    
    // Optimisations index
    bool enable_hierarchical_index; // Index hiérarchique (HNSW-like)
    bool enable_inverted_index;     // Index inversé pour mots-clés
    bool enable_bloom_filter;       // Filtre de Bloom pour pré-filtrage
    
} OptimizationConfig;

// Cache LRU pour embeddings
typedef struct CacheNode {
    char* key;                      // Clé (hash du contenu)
    float* embedding;               // Embedding mis en cache
    int dimension;                  // Dimension de l'embedding
    time_t last_access;            // Dernier accès
    struct CacheNode* prev;
    struct CacheNode* next;
} CacheNode;

typedef struct {
    CacheNode* head;
    CacheNode* tail;
    CacheNode** hash_table;        // Table de hachage pour accès O(1)
    int capacity;
    int size;
    pthread_mutex_t mutex;         // Thread-safe
} LRUCache;

// Pool de mémoire optimisé
typedef struct MemoryBlock {
    void* data;
    size_t size;
    bool in_use;
    struct MemoryBlock* next;
} MemoryBlock;

typedef struct {
    MemoryBlock* blocks;
    size_t total_size;
    size_t used_size;
    pthread_mutex_t mutex;
} MemoryPool;

// Index hiérarchique optimisé
typedef struct IndexNode {
    float* centroid;               // Centroïde du cluster
    int* file_indices;             // Indices des fichiers dans ce cluster
    int num_files;
    int capacity;
    struct IndexNode** children;   // Sous-clusters
    int num_children;
    int level;                     // Niveau dans la hiérarchie
} IndexNode;

typedef struct {
    IndexNode* root;
    int max_cluster_size;          // Taille max d'un cluster
    int num_levels;                // Nombre de niveaux
    float cluster_threshold;       // Seuil de similarité pour clustering
} HierarchicalIndex;

// Moteur de recherche optimisé
typedef struct OptimizedSearchEngine {
    SearchEngine* base_engine;     // Moteur de base
    OptimizationConfig config;     // Configuration optimisations
    
    // Caches
    LRUCache* embedding_cache;
    LRUCache* query_cache;
    
    // Pools mémoire
    MemoryPool* memory_pool;
    
    // Index optimisés
    HierarchicalIndex* hierarchical_index;
    
    // Threading
    pthread_t* worker_threads;
    pthread_mutex_t index_mutex;
    pthread_cond_t work_available;
    
    // Statistiques d'optimisation
    struct {
        int cache_hits;
        int cache_misses;
        double avg_search_time_optimized;
        double memory_saved_percent;
        int parallel_searches_executed;
    } opt_stats;
    
} OptimizedSearchEngine;

// ========== API d'optimisation ==========

// Configuration
OptimizationConfig get_default_optimization_config(void);
OptimizedSearchEngine* optimized_search_create(const SearchConfig* base_config, 
                                              const OptimizationConfig* opt_config);
void optimized_search_destroy(OptimizedSearchEngine* engine);

// Cache LRU
LRUCache* lru_cache_create(int capacity);
void lru_cache_destroy(LRUCache* cache);
float* lru_cache_get(LRUCache* cache, const char* key, int* dimension);
void lru_cache_put(LRUCache* cache, const char* key, float* embedding, int dimension);
void lru_cache_clear(LRUCache* cache);

// Pool mémoire
MemoryPool* memory_pool_create(size_t initial_size);
void memory_pool_destroy(MemoryPool* pool);
void* memory_pool_alloc(MemoryPool* pool, size_t size);
void memory_pool_free(MemoryPool* pool, void* ptr);
void memory_pool_reset(MemoryPool* pool);

// Embedding optimisé
float* generate_optimized_embedding(OptimizedSearchEngine* engine, const char* text);
float* quantize_embedding(const float* embedding, int dimension, int bits);
float* reduce_embedding_dimension(const float* embedding, int original_dim, int target_dim);
void compress_embedding(const float* embedding, int dimension, unsigned char** compressed, size_t* compressed_size);

// Recherche optimisée
SearchResult* optimized_search_semantic(OptimizedSearchEngine* engine, const char* query, int* num_results);
SearchResult* optimized_search_parallel(OptimizedSearchEngine* engine, const char* query, int* num_results);
SearchResult* optimized_search_hierarchical(OptimizedSearchEngine* engine, const char* query, int* num_results);

// Index hiérarchique
HierarchicalIndex* hierarchical_index_create(int max_cluster_size, float threshold);
void hierarchical_index_destroy(HierarchicalIndex* index);
bool hierarchical_index_add_file(HierarchicalIndex* index, int file_id, const float* embedding, int dimension);
SearchResult* hierarchical_index_search(HierarchicalIndex* index, const float* query_embedding, 
                                       int dimension, int max_results, const SearchEngine* base_engine);

// Optimisations spécifiques
typedef struct {
    char* content_hash;            // Hash pour éviter re-processing
    float* embedding_fast;         // Embedding dimension réduite
    float* embedding_full;         // Embedding dimension complète (lazy)
    bool full_computed;            // Est-ce que full est calculé ?
    time_t last_access;
} LazyEmbedding;

LazyEmbedding* lazy_embedding_create(const char* content);
void lazy_embedding_destroy(LazyEmbedding* embedding);
float* lazy_embedding_get_fast(LazyEmbedding* embedding);
float* lazy_embedding_get_full(LazyEmbedding* embedding);

// Filtrage rapide
typedef struct {
    unsigned char* bits;           // Bits du filtre
    int size;                      // Taille en bits
    int num_hashes;               // Nombre de fonctions de hachage
} BloomFilter;

BloomFilter* bloom_filter_create(int expected_items, double false_positive_rate);
void bloom_filter_destroy(BloomFilter* filter);
void bloom_filter_add(BloomFilter* filter, const char* item);
bool bloom_filter_contains(BloomFilter* filter, const char* item);

// Mesures de performance optimisées
typedef struct {
    double indexing_time_optimized;
    double search_time_optimized;
    double memory_usage_optimized;
    double cache_hit_rate;
    double compression_ratio;
    int parallel_efficiency_percent;
} OptimizedMetrics;

OptimizedMetrics benchmark_optimized_engine(OptimizedSearchEngine* engine, const char* corpus_path);

// Threading et parallélisme
typedef struct {
    OptimizedSearchEngine* engine;
    const char* query;
    SearchResult** results;
    int* num_results;
    int thread_id;
    void* barrier;
} SearchTask;

void* parallel_search_worker(void* arg);
SearchResult* merge_search_results(SearchResult** result_arrays, int* result_counts, int num_arrays, int max_total);

// Optimisations avancées
bool enable_simd_operations(void);  // Détection SIMD (AVX/SSE)
float simd_cosine_similarity(const float* a, const float* b, int dimension);
void simd_normalize_vector(float* vector, int dimension);

// Debug et profiling
void print_optimization_stats(OptimizedSearchEngine* engine);
void profile_search_operation(OptimizedSearchEngine* engine, const char* query);
char* generate_optimization_report(OptimizedSearchEngine* engine);

#ifdef __cplusplus
}
#endif

#endif // OPTIMIZED_SEARCH_H