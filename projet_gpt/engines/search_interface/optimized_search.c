// optimized_search.c - Implémentation des optimisations de performance

#include "optimized_search.h"
#include <math.h>
#include <pthread.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

// ========== Configuration par défaut ==========

OptimizationConfig get_default_optimization_config(void) {
    OptimizationConfig config = {
        // Cache
        .enable_lru_cache = true,
        .cache_size = 1000,
        .enable_query_cache = true,
        .query_cache_size = 100,
        
        // Embeddings
        .enable_embedding_quantization = true,
        .quantized_bits = 8,
        .enable_dimension_reduction = true,
        .reduced_dimension = 256,
        
        // Algorithme
        .enable_early_termination = true,
        .early_stop_threshold = 0.95f,
        .enable_parallel_search = true,
        .num_search_threads = 4,
        
        // I/O
        .enable_async_indexing = false, // Pour simplicité
        .enable_mmap_files = false,     // Éviter complications
        .enable_file_buffering = true,
        .buffer_size_kb = 64,
        
        // Mémoire
        .enable_memory_pooling = true,
        .enable_zero_copy = true,
        .enable_compress_embeddings = false, // Éviter complexité
        
        // Index
        .enable_hierarchical_index = true,
        .enable_inverted_index = false,
        .enable_bloom_filter = true
    };
    return config;
}

// ========== Cache LRU ==========

static uint32_t hash_key(const char* key) {
    uint32_t hash = 5381;
    int c;
    while ((c = *key++)) {
        hash = ((hash << 5) + hash) + c;
    }
    return hash;
}

LRUCache* lru_cache_create(int capacity) {
    LRUCache* cache = malloc(sizeof(LRUCache));
    if (!cache) return NULL;
    
    cache->capacity = capacity;
    cache->size = 0;
    cache->head = cache->tail = NULL;
    
    // Table de hachage
    cache->hash_table = calloc(capacity * 2, sizeof(CacheNode*));
    if (!cache->hash_table) {
        free(cache);
        return NULL;
    }
    
    pthread_mutex_init(&cache->mutex, NULL);
    return cache;
}

void lru_cache_destroy(LRUCache* cache) {
    if (!cache) return;
    
    pthread_mutex_lock(&cache->mutex);
    
    CacheNode* current = cache->head;
    while (current) {
        CacheNode* next = current->next;
        free(current->key);
        free(current->embedding);
        free(current);
        current = next;
    }
    
    free(cache->hash_table);
    pthread_mutex_unlock(&cache->mutex);
    pthread_mutex_destroy(&cache->mutex);
    free(cache);
}

static void remove_node(LRUCache* cache, CacheNode* node) {
    if (node->prev) node->prev->next = node->next;
    else cache->head = node->next;
    
    if (node->next) node->next->prev = node->prev;
    else cache->tail = node->prev;
}

static void add_to_head(LRUCache* cache, CacheNode* node) {
    node->next = cache->head;
    node->prev = NULL;
    
    if (cache->head) cache->head->prev = node;
    cache->head = node;
    
    if (!cache->tail) cache->tail = node;
}

float* lru_cache_get(LRUCache* cache, const char* key, int* dimension) {
    if (!cache || !key) return NULL;
    
    pthread_mutex_lock(&cache->mutex);
    
    uint32_t hash = hash_key(key) % (cache->capacity * 2);
    CacheNode* node = cache->hash_table[hash];
    
    // Chercher dans la chaîne de collision
    while (node && strcmp(node->key, key) != 0) {
        node = node->next;
    }
    
    if (node) {
        // Cache hit - remonter en tête
        remove_node(cache, node);
        add_to_head(cache, node);
        node->last_access = time(NULL);
        
        *dimension = node->dimension;
        pthread_mutex_unlock(&cache->mutex);
        return node->embedding;
    }
    
    pthread_mutex_unlock(&cache->mutex);
    return NULL; // Cache miss
}

void lru_cache_put(LRUCache* cache, const char* key, float* embedding, int dimension) {
    if (!cache || !key || !embedding) return;
    
    pthread_mutex_lock(&cache->mutex);
    
    // Créer nouveau nœud
    CacheNode* node = malloc(sizeof(CacheNode));
    node->key = strdup(key);
    node->embedding = malloc(dimension * sizeof(float));
    memcpy(node->embedding, embedding, dimension * sizeof(float));
    node->dimension = dimension;
    node->last_access = time(NULL);
    
    // Ajouter en tête
    add_to_head(cache, node);
    cache->size++;
    
    // Ajouter à la table de hachage
    uint32_t hash = hash_key(key) % (cache->capacity * 2);
    node->next = cache->hash_table[hash];
    cache->hash_table[hash] = node;
    
    // Éviction si nécessaire
    if (cache->size > cache->capacity) {
        CacheNode* tail = cache->tail;
        remove_node(cache, tail);
        
        // Retirer de la table de hachage
        uint32_t tail_hash = hash_key(tail->key) % (cache->capacity * 2);
        CacheNode** slot = &cache->hash_table[tail_hash];
        while (*slot && *slot != tail) {
            slot = &(*slot)->next;
        }
        if (*slot) *slot = tail->next;
        
        free(tail->key);
        free(tail->embedding);
        free(tail);
        cache->size--;
    }
    
    pthread_mutex_unlock(&cache->mutex);
}

// ========== Pool mémoire ==========

MemoryPool* memory_pool_create(size_t initial_size) {
    MemoryPool* pool = malloc(sizeof(MemoryPool));
    if (!pool) return NULL;
    
    pool->blocks = malloc(sizeof(MemoryBlock));
    pool->blocks->data = malloc(initial_size);
    pool->blocks->size = initial_size;
    pool->blocks->in_use = false;
    pool->blocks->next = NULL;
    
    pool->total_size = initial_size;
    pool->used_size = 0;
    
    pthread_mutex_init(&pool->mutex, NULL);
    return pool;
}

void* memory_pool_alloc(MemoryPool* pool, size_t size) {
    if (!pool) return malloc(size); // Fallback
    
    pthread_mutex_lock(&pool->mutex);
    
    // Chercher un bloc libre suffisant
    MemoryBlock* block = pool->blocks;
    while (block) {
        if (!block->in_use && block->size >= size) {
            block->in_use = true;
            pool->used_size += size;
            pthread_mutex_unlock(&pool->mutex);
            return block->data;
        }
        block = block->next;
    }
    
    pthread_mutex_unlock(&pool->mutex);
    return malloc(size); // Fallback si pool plein
}

void memory_pool_free(MemoryPool* pool, void* ptr) {
    if (!pool) {
        free(ptr);
        return;
    }
    
    pthread_mutex_lock(&pool->mutex);
    
    MemoryBlock* block = pool->blocks;
    while (block) {
        if (block->data == ptr) {
            block->in_use = false;
            pool->used_size -= block->size;
            break;
        }
        block = block->next;
    }
    
    pthread_mutex_unlock(&pool->mutex);
}

// ========== Embedding optimisé ==========

static float* generate_fast_embedding(const char* text, int target_dimension) {
    // Version rapide avec dimension réduite
    float* embedding = calloc(target_dimension, sizeof(float));
    if (!embedding) return NULL;
    
    int text_len = strlen(text);
    uint32_t hash = hash_key(text);
    
    // Algorithme simplifié pour vitesse
    srand(hash);
    
    for (int i = 0; i < target_dimension; i++) {
        float value = 0.0f;
        
        // Moins de calculs complexes
        if (i < text_len) {
            value += (float)text[i] / 255.0f;
        }
        
        // Random déterministe plus simple
        value += ((float)rand() / RAND_MAX - 0.5f) * 0.1f;
        
        embedding[i] = value;
    }
    
    // Normalisation simplifiée
    float norm = 0.0f;
    for (int i = 0; i < target_dimension; i++) {
        norm += embedding[i] * embedding[i];
    }
    norm = sqrtf(norm);
    
    if (norm > 0.0f) {
        for (int i = 0; i < target_dimension; i++) {
            embedding[i] /= norm;
        }
    }
    
    return embedding;
}

float* generate_optimized_embedding(OptimizedSearchEngine* engine, const char* text) {
    if (!engine || !text) return NULL;
    
    // Hash pour clé de cache
    char cache_key[256];
    snprintf(cache_key, sizeof(cache_key), "text_%u", hash_key(text));
    
    // Vérifier cache d'abord
    if (engine->config.enable_lru_cache) {
        int cached_dim;
        float* cached = lru_cache_get(engine->embedding_cache, cache_key, &cached_dim);
        if (cached) {
            engine->opt_stats.cache_hits++;
            
            // Copier pour éviter corruption
            float* result = malloc(cached_dim * sizeof(float));
            memcpy(result, cached, cached_dim * sizeof(float));
            return result;
        }
        engine->opt_stats.cache_misses++;
    }
    
    // Générer embedding
    int dimension = engine->config.enable_dimension_reduction ? 
                   engine->config.reduced_dimension : 768;
    
    float* embedding = generate_fast_embedding(text, dimension);
    
    // Mettre en cache
    if (embedding && engine->config.enable_lru_cache) {
        lru_cache_put(engine->embedding_cache, cache_key, embedding, dimension);
    }
    
    return embedding;
}

// ========== Recherche avec arrêt précoce ==========

static int compare_search_results(const void* a, const void* b) {
    const SearchResult* result_a = (const SearchResult*)a;
    const SearchResult* result_b = (const SearchResult*)b;
    
    if (result_a->relevance_score > result_b->relevance_score) return -1;
    if (result_a->relevance_score < result_b->relevance_score) return 1;
    return 0;
}

SearchResult* optimized_search_semantic(OptimizedSearchEngine* engine, const char* query, int* num_results) {
    if (!engine || !query || !num_results) return NULL;
    
    *num_results = 0;
    
    // Vérifier cache de requêtes
    char query_key[256];
    snprintf(query_key, sizeof(query_key), "query_%u", hash_key(query));
    
    if (engine->config.enable_query_cache) {
        int cached_dim;
        SearchResult* cached = (SearchResult*)lru_cache_get(engine->query_cache, query_key, &cached_dim);
        if (cached) {
            *num_results = cached_dim;
            return cached;
        }
    }
    
    // Générer embedding de requête optimisé
    float* query_embedding = generate_optimized_embedding(engine, query);
    if (!query_embedding) return NULL;
    
    int dimension = engine->config.enable_dimension_reduction ? 
                   engine->config.reduced_dimension : 768;
    
    SearchEngine* base = engine->base_engine;
    int max_results = base->config.max_results;
    
    SearchResult* results = malloc(max_results * sizeof(SearchResult));
    if (!results) {
        free(query_embedding);
        return NULL;
    }
    
    int result_count = 0;
    float best_score = 0.0f;
    
    // Recherche avec arrêt précoce
    for (int i = 0; i < base->num_files && result_count < max_results; i++) {
        // Calculer similarité
        float similarity = calculate_cosine_similarity(query_embedding, 
            base->file_indices[i].embedding_vector, dimension);
        
        // Arrêt précoce si score parfait atteint
        if (engine->config.enable_early_termination && 
            similarity >= engine->config.early_stop_threshold) {
            
            // Ajouter ce résultat parfait
            if (similarity >= base->config.similarity_threshold) {
                SearchResult* result = &results[result_count];
                result->file_path = strdup(base->file_indices[i].path);
                result->file_name = strdup(base->file_indices[i].name);
                result->relevance_score = similarity;
                result->content_preview = strndup(base->file_indices[i].content, 200);
                result->match_type = strdup("semantic_early");
                result->file_size = base->file_indices[i].file_size;
                result->last_modified = base->file_indices[i].last_modified;
                result->match_position = 0;
                
                result_count++;
                best_score = similarity;
            }
            break; // Arrêt précoce
        }
        
        // Ajouter résultat normal
        if (similarity >= base->config.similarity_threshold) {
            SearchResult* result = &results[result_count];
            result->file_path = strdup(base->file_indices[i].path);
            result->file_name = strdup(base->file_indices[i].name);
            result->relevance_score = similarity;
            result->content_preview = strndup(base->file_indices[i].content, 200);
            result->match_type = strdup("semantic_optimized");
            result->file_size = base->file_indices[i].file_size;
            result->last_modified = base->file_indices[i].last_modified;
            result->match_position = 0;
            
            result_count++;
            if (similarity > best_score) best_score = similarity;
        }
    }
    
    // Trier les résultats
    if (result_count > 1) {
        qsort(results, result_count, sizeof(SearchResult), compare_search_results);
    }
    
    *num_results = result_count;
    
    // Mettre en cache le résultat
    if (engine->config.enable_query_cache && result_count > 0) {
        lru_cache_put(engine->query_cache, query_key, (float*)results, result_count);
    }
    
    free(query_embedding);
    return results;
}

// ========== Moteur optimisé ==========

OptimizedSearchEngine* optimized_search_create(const SearchConfig* base_config, 
                                              const OptimizationConfig* opt_config) {
    OptimizedSearchEngine* engine = malloc(sizeof(OptimizedSearchEngine));
    if (!engine) return NULL;
    
    // Moteur de base
    engine->base_engine = search_engine_create(base_config);
    if (!engine->base_engine) {
        free(engine);
        return NULL;
    }
    
    // Configuration
    if (opt_config) {
        engine->config = *opt_config;
    } else {
        engine->config = get_default_optimization_config();
    }
    
    // Caches
    if (engine->config.enable_lru_cache) {
        engine->embedding_cache = lru_cache_create(engine->config.cache_size);
    } else {
        engine->embedding_cache = NULL;
    }
    
    if (engine->config.enable_query_cache) {
        engine->query_cache = lru_cache_create(engine->config.query_cache_size);
    } else {
        engine->query_cache = NULL;
    }
    
    // Pool mémoire
    if (engine->config.enable_memory_pooling) {
        engine->memory_pool = memory_pool_create(1024 * 1024); // 1MB initial
    } else {
        engine->memory_pool = NULL;
    }
    
    // Initialiser statistiques
    memset(&engine->opt_stats, 0, sizeof(engine->opt_stats));
    
    // Mutexes
    pthread_mutex_init(&engine->index_mutex, NULL);
    
    return engine;
}

void optimized_search_destroy(OptimizedSearchEngine* engine) {
    if (!engine) return;
    
    search_engine_destroy(engine->base_engine);
    
    if (engine->embedding_cache) lru_cache_destroy(engine->embedding_cache);
    if (engine->query_cache) lru_cache_destroy(engine->query_cache);
    if (engine->memory_pool) memory_pool_destroy(engine->memory_pool);
    
    pthread_mutex_destroy(&engine->index_mutex);
    free(engine);
}

// ========== Métriques optimisées ==========

OptimizedMetrics benchmark_optimized_engine(OptimizedSearchEngine* engine, const char* corpus_path) {
    OptimizedMetrics metrics = {0};
    
    if (!engine || !corpus_path) return metrics;
    
    printf("\n🚀 Benchmark Moteur Optimisé\n");
    printf("============================\n");
    
    // Test d'indexation optimisée
    double start_time = get_time_ms();
    bool success = search_engine_index_directory(engine->base_engine, corpus_path, NULL, NULL);
    double end_time = get_time_ms();
    
    if (success) {
        metrics.indexing_time_optimized = end_time - start_time;
        SearchStats stats = search_engine_get_stats(engine->base_engine);
        printf("✅ Indexation optimisée: %d fichiers en %.2f ms\n", 
               stats.total_files_indexed, metrics.indexing_time_optimized);
    }
    
    // Test de recherche optimisée
    const char* test_queries[] = {
        "artificial intelligence", "machine learning", "programming", "research", "documentation"
    };
    int num_queries = sizeof(test_queries) / sizeof(test_queries[0]);
    
    double total_search_time = 0.0;
    int total_results = 0;
    
    for (int i = 0; i < num_queries; i++) {
        start_time = get_time_ms();
        int num_results = 0;
        SearchResult* results = optimized_search_semantic(engine, test_queries[i], &num_results);
        end_time = get_time_ms();
        
        double search_time = end_time - start_time;
        total_search_time += search_time;
        total_results += num_results;
        
        printf("   🔍 '%s': %.3f ms (%d résultats)\n", test_queries[i], search_time, num_results);
        
        if (results) search_results_free(results, num_results);
    }
    
    metrics.search_time_optimized = total_search_time / num_queries;
    metrics.cache_hit_rate = (float)engine->opt_stats.cache_hits / 
                           (engine->opt_stats.cache_hits + engine->opt_stats.cache_misses);
    
    printf("📊 Résultats optimisés:\n");
    printf("   ⏱️  Recherche moyenne: %.3f ms\n", metrics.search_time_optimized);
    printf("   💾 Taux de cache: %.1f%%\n", metrics.cache_hit_rate * 100);
    printf("   📈 Cache hits: %d, misses: %d\n", 
           engine->opt_stats.cache_hits, engine->opt_stats.cache_misses);
    
    return metrics;
}

void print_optimization_stats(OptimizedSearchEngine* engine) {
    if (!engine) return;
    
    printf("\n📊 STATISTIQUES D'OPTIMISATION\n");
    printf("==============================\n");
    printf("💾 Cache embeddings: %s\n", engine->config.enable_lru_cache ? "✅" : "❌");
    printf("🔍 Cache requêtes: %s\n", engine->config.enable_query_cache ? "✅" : "❌");
    printf("⚡ Arrêt précoce: %s\n", engine->config.enable_early_termination ? "✅" : "❌");
    printf("🧮 Dimension réduite: %s (%d → %d)\n", 
           engine->config.enable_dimension_reduction ? "✅" : "❌",
           768, engine->config.reduced_dimension);
    printf("🏊 Pool mémoire: %s\n", engine->config.enable_memory_pooling ? "✅" : "❌");
    
    printf("\n📈 Performance:\n");
    printf("   Cache hits: %d\n", engine->opt_stats.cache_hits);
    printf("   Cache misses: %d\n", engine->opt_stats.cache_misses);
    printf("   Taux de succès: %.1f%%\n", 
           (float)engine->opt_stats.cache_hits / 
           (engine->opt_stats.cache_hits + engine->opt_stats.cache_misses) * 100);
}