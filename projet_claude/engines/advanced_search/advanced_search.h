// advanced_search.h - Bibliothèque de recherche avancée avec embeddings et optimisations
// Utilise EmbeddingGemma + FAISS + techniques d'optimisation pour recherche ultra-rapide

#ifndef ADVANCED_SEARCH_H
#define ADVANCED_SEARCH_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Configuration de performance
#define EMBEDDING_DIMENSION_FULL 768        // Dimension complète EmbeddingGemma
#define EMBEDDING_DIMENSION_FAST 256        // Dimension optimisée pour vitesse
#define EMBEDDING_DIMENSION_ULTRA_FAST 128  // Dimension ultra-rapide
#define MAX_QUERY_LENGTH 1024
#define MAX_RESULTS 100
#define CACHE_SIZE 1000

// Types de recherche
typedef enum {
    SEARCH_TYPE_EXACT = 0,       // Recherche exacte (nom de fichier)
    SEARCH_TYPE_FUZZY = 1,       // Recherche floue (Levenshtein)
    SEARCH_TYPE_SEMANTIC = 2,    // Recherche sémantique (embedding)
    SEARCH_TYPE_HYBRID = 3       // Combinaison de toutes les techniques
} SearchType;

// Modes de performance
typedef enum {
    SEARCH_MODE_ACCURACY = 0,    // Précision maximale (768D)
    SEARCH_MODE_BALANCED = 1,    // Équilibré (256D)  
    SEARCH_MODE_SPEED = 2        // Vitesse maximale (128D)
} SearchMode;

// Stratégies d'indexation
typedef enum {
    INDEX_STRATEGY_FLAT = 0,     // Index plat (petits corpus)
    INDEX_STRATEGY_IVF = 1,      // Inverted File (moyens corpus)
    INDEX_STRATEGY_HNSW = 2,     // Hierarchical NSW (grands corpus)
    INDEX_STRATEGY_LSH = 3       // Locality Sensitive Hashing (très grands)
} IndexStrategy;

// Résultat de recherche
typedef struct {
    char* file_path;             // Chemin du fichier
    char* file_name;             // Nom du fichier
    char* content_preview;       // Aperçu du contenu (max 200 chars)
    float relevance_score;       // Score de pertinence (0.0 - 1.0)
    int64_t file_size;          // Taille du fichier
    time_t last_modified;       // Dernière modification
    char* match_type;           // Type de correspondance trouvée
    int match_position;         // Position de la correspondance dans le texte
} SearchResult;

// Index de fichier
typedef struct {
    char* path;                 // Chemin du fichier
    char* name;                 // Nom du fichier
    char* content;              // Contenu textuel
    float* embedding_vector;    // Vecteur d'embedding
    int embedding_dimension;    // Dimension du vecteur
    size_t content_hash;        // Hash du contenu pour détection de changement
    time_t indexed_time;        // Timestamp d'indexation
    int64_t file_size;
    time_t last_modified;
} FileIndex;

// Configuration du moteur de recherche
typedef struct {
    SearchMode mode;            // Mode de performance
    IndexStrategy strategy;     // Stratégie d'indexation
    int max_results;           // Nombre max de résultats
    float similarity_threshold; // Seuil de similarité minimum
    bool enable_caching;       // Activer le cache des requêtes
    bool enable_prefetch;      // Activer le prefetch des embeddings
    char* model_path;          // Chemin vers le modèle d'embedding
    char* index_path;          // Chemin de sauvegarde de l'index
} SearchConfig;

// Statistiques de performance
typedef struct {
    int total_files_indexed;   // Nombre total de fichiers indexés
    int total_queries;         // Nombre total de requêtes
    double avg_query_time_ms;  // Temps moyen de requête (ms)
    double avg_index_time_ms;  // Temps moyen d'indexation par fichier (ms)
    size_t memory_usage_mb;    // Utilisation mémoire (MB)
    int cache_hits;           // Hits de cache
    int cache_misses;         // Misses de cache
    time_t last_index_update; // Dernière mise à jour de l'index
} SearchStats;

// Contexte du moteur de recherche
typedef struct SearchEngine SearchEngine;

// Callbacks pour monitoring
typedef void (*SearchProgressCallback)(float progress, const char* status, void* user_data);
typedef void (*SearchStatsCallback)(const SearchStats* stats, void* user_data);

// ========== API Principal ==========

// Initialisation et configuration
SearchEngine* search_engine_create(const SearchConfig* config);
void search_engine_destroy(SearchEngine* engine);
bool search_engine_set_config(SearchEngine* engine, const SearchConfig* config);
SearchConfig search_engine_get_default_config(void);

// Gestion de l'index
bool search_engine_index_directory(SearchEngine* engine, const char* directory_path, 
                                 SearchProgressCallback progress_cb, void* user_data);
bool search_engine_index_file(SearchEngine* engine, const char* file_path);
bool search_engine_remove_file(SearchEngine* engine, const char* file_path);
bool search_engine_rebuild_index(SearchEngine* engine, SearchProgressCallback progress_cb, void* user_data);
bool search_engine_save_index(SearchEngine* engine, const char* index_path);
bool search_engine_load_index(SearchEngine* engine, const char* index_path);

// Recherche principale
SearchResult* search_engine_query(SearchEngine* engine, const char* query, 
                                SearchType search_type, int* num_results);
SearchResult* search_engine_query_with_filters(SearchEngine* engine, const char* query,
                                              SearchType search_type, const char* file_extension,
                                              time_t modified_after, int64_t min_size,
                                              int* num_results);

// Recherche spécialisée
SearchResult* search_filename_fuzzy(SearchEngine* engine, const char* query, int* num_results);
SearchResult* search_content_exact(SearchEngine* engine, const char* query, int* num_results);
SearchResult* search_semantic_similar(SearchEngine* engine, const char* query, int* num_results);
SearchResult* search_hybrid_advanced(SearchEngine* engine, const char* query, int* num_results);

// Suggestions et autocomplétion
char** search_engine_get_suggestions(SearchEngine* engine, const char* partial_query, int* num_suggestions);
char** search_engine_get_recent_queries(SearchEngine* engine, int* num_queries);
void search_engine_add_recent_query(SearchEngine* engine, const char* query);

// Optimisations de performance
bool search_engine_warm_cache(SearchEngine* engine);
bool search_engine_optimize_index(SearchEngine* engine);
bool search_engine_compact_index(SearchEngine* engine);
void search_engine_clear_cache(SearchEngine* engine);

// Statistiques et monitoring
SearchStats search_engine_get_stats(SearchEngine* engine);
void search_engine_reset_stats(SearchEngine* engine);
void search_engine_set_stats_callback(SearchEngine* engine, SearchStatsCallback callback, void* user_data);

// Utilitaires pour résultats
void search_results_free(SearchResult* results, int num_results);
SearchResult* search_results_deduplicate(SearchResult* results, int num_results, int* new_count);
SearchResult* search_results_sort_by_relevance(SearchResult* results, int num_results);
SearchResult* search_results_sort_by_date(SearchResult* results, int num_results);

// Utilitaires pour embeddings
float* generate_text_embedding(const char* text, int dimension);
float calculate_cosine_similarity(const float* vec1, const float* vec2, int dimension);
float calculate_euclidean_distance(const float* vec1, const float* vec2, int dimension);
void embedding_vector_free(float* vector);

// Configuration avancée des algorithmes
typedef struct {
    // FAISS IVF parameters
    int ivf_nlist;             // Nombre de clusters (défaut: sqrt(n))
    int ivf_nprobe;            // Nombre de clusters à explorer (défaut: 10)
    
    // HNSW parameters  
    int hnsw_m;                // Nombre de connexions (défaut: 16)
    int hnsw_ef_construction;  // Taille de la liste dynamique (défaut: 200)
    int hnsw_ef_search;        // Taille de la liste de recherche (défaut: 50)
    
    // LSH parameters
    int lsh_num_tables;        // Nombre de tables de hachage (défaut: 10)
    int lsh_num_bits;          // Nombre de bits par hash (défaut: 10)
    
    // Quantization
    bool enable_pq;            // Product Quantization
    int pq_m;                  // Nombre de sous-quantizers (défaut: 8)
    int pq_nbits;             // Bits par sous-quantizer (défaut: 8)
} AdvancedIndexConfig;

bool search_engine_set_advanced_config(SearchEngine* engine, const AdvancedIndexConfig* config);

// API de développement et debug
void search_engine_enable_debug(SearchEngine* engine, bool enable);
char* search_engine_get_debug_info(SearchEngine* engine);
bool search_engine_validate_index(SearchEngine* engine);
void search_engine_print_stats(SearchEngine* engine);

// Gestion des erreurs
typedef enum {
    SEARCH_SUCCESS = 0,
    SEARCH_ERROR_INVALID_PARAM = -1,
    SEARCH_ERROR_OUT_OF_MEMORY = -2,
    SEARCH_ERROR_IO_ERROR = -3,
    SEARCH_ERROR_MODEL_NOT_FOUND = -4,
    SEARCH_ERROR_INDEX_CORRUPTED = -5,
    SEARCH_ERROR_EMBEDDING_FAILED = -6
} SearchResult_t;

SearchResult_t search_engine_get_last_error(SearchEngine* engine);
const char* search_engine_get_error_message(SearchResult_t error);
void search_engine_clear_error(SearchEngine* engine);

#ifdef __cplusplus
}
#endif

#endif // ADVANCED_SEARCH_H