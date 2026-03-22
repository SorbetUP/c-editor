// advanced_search.c - Implémentation de la recherche avancée avec embeddings
#include "advanced_search.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <fcntl.h>

// Structure interne du moteur de recherche
struct SearchEngine {
    SearchConfig config;
    FileIndex* file_indices;
    int num_files;
    int capacity;
    SearchStats stats;
    SearchResult_t last_error;
    
    // Cache des requêtes récentes
    char** recent_queries;
    int num_recent_queries;
    
    // Index vectoriel (simulation FAISS)
    float** embedding_matrix;   // Matrice des embeddings
    int* file_mapping;          // Mapping index -> file_id
    
    // Cache des embeddings
    struct {
        char* query;
        float* embedding;
        time_t timestamp;
    } embedding_cache[CACHE_SIZE];
    int cache_size;
    
    bool debug_enabled;
};

// ========== Utilitaires internes ==========

static uint32_t hash_string(const char* str) {
    uint32_t hash = 5381;
    int c;
    while ((c = *str++)) {
        hash = ((hash << 5) + hash) + c;
    }
    return hash;
}

static float levenshtein_distance_normalized(const char* s1, const char* s2) {
    int len1 = strlen(s1);
    int len2 = strlen(s2);
    int max_len = (len1 > len2) ? len1 : len2;
    
    if (max_len == 0) return 1.0f;
    
    // Matrice de programmation dynamique
    int** dp = malloc((len1 + 1) * sizeof(int*));
    for (int i = 0; i <= len1; i++) {
        dp[i] = malloc((len2 + 1) * sizeof(int));
    }
    
    // Initialisation
    for (int i = 0; i <= len1; i++) dp[i][0] = i;
    for (int j = 0; j <= len2; j++) dp[0][j] = j;
    
    // Calcul de la distance
    for (int i = 1; i <= len1; i++) {
        for (int j = 1; j <= len2; j++) {
            int cost = (s1[i-1] == s2[j-1]) ? 0 : 1;
            
            int min_val = dp[i-1][j] + 1;      // Suppression
            int insert = dp[i][j-1] + 1;      // Insertion
            int subst = dp[i-1][j-1] + cost;  // Substitution
            
            if (insert < min_val) min_val = insert;
            if (subst < min_val) min_val = subst;
            
            dp[i][j] = min_val;
        }
    }
    
    int distance = dp[len1][len2];
    
    // Libération mémoire
    for (int i = 0; i <= len1; i++) {
        free(dp[i]);
    }
    free(dp);
    
    return 1.0f - ((float)distance / max_len);
}

// Simulation d'embedding avec EmbeddingGemma (algorithme simplifié)
static float* generate_embedding_simulation(const char* text, int dimension) {
    float* embedding = calloc(dimension, sizeof(float));
    if (!embedding) return NULL;
    
    // Simulation basée sur les caractéristiques du texte
    int text_len = strlen(text);
    uint32_t hash = hash_string(text);
    
    // Génération de vecteur pseudo-aléatoire mais déterministe
    srand(hash);
    
    for (int i = 0; i < dimension; i++) {
        // Combinaison de différentes caractéristiques
        float component = 0.0f;
        
        // Composante basée sur la position dans le texte
        if (i < text_len) {
            component += (float)text[i] / 255.0f;
        }
        
        // Composante aléatoire déterministe  
        component += ((float)rand() / RAND_MAX - 0.5f) * 0.1f;
        
        // Composante basée sur la fréquence des caractères
        int char_count = 0;
        for (int j = 0; j < text_len; j++) {
            if (text[j] == ('a' + (i % 26))) char_count++;
        }
        component += (float)char_count / text_len * 0.2f;
        
        embedding[i] = component;
    }
    
    // Normalisation du vecteur
    float norm = 0.0f;
    for (int i = 0; i < dimension; i++) {
        norm += embedding[i] * embedding[i];
    }
    norm = sqrtf(norm);
    
    if (norm > 0.0f) {
        for (int i = 0; i < dimension; i++) {
            embedding[i] /= norm;
        }
    }
    
    return embedding;
}

// ========== API Principal ==========

SearchConfig search_engine_get_default_config(void) {
    SearchConfig config = {
        .mode = SEARCH_MODE_BALANCED,
        .strategy = INDEX_STRATEGY_FLAT,
        .max_results = 50,
        .similarity_threshold = 0.5f,
        .enable_caching = true,
        .enable_prefetch = false,
        .model_path = NULL,
        .index_path = NULL
    };
    return config;
}

SearchEngine* search_engine_create(const SearchConfig* config) {
    SearchEngine* engine = calloc(1, sizeof(SearchEngine));
    if (!engine) return NULL;
    
    if (config) {
        engine->config = *config;
    } else {
        engine->config = search_engine_get_default_config();
    }
    
    engine->capacity = 1000; // Capacité initiale
    engine->file_indices = malloc(engine->capacity * sizeof(FileIndex));
    engine->recent_queries = malloc(100 * sizeof(char*));
    
    if (!engine->file_indices || !engine->recent_queries) {
        free(engine->file_indices);
        free(engine->recent_queries);
        free(engine);
        return NULL;
    }
    
    // Initialisation des statistiques
    memset(&engine->stats, 0, sizeof(SearchStats));
    engine->stats.last_index_update = time(NULL);
    
    engine->debug_enabled = false;
    engine->last_error = SEARCH_SUCCESS;
    
    return engine;
}

void search_engine_destroy(SearchEngine* engine) {
    if (!engine) return;
    
    // Libération des indices de fichiers
    for (int i = 0; i < engine->num_files; i++) {
        free(engine->file_indices[i].path);
        free(engine->file_indices[i].name);
        free(engine->file_indices[i].content);
        free(engine->file_indices[i].embedding_vector);
    }
    free(engine->file_indices);
    
    // Libération des requêtes récentes
    for (int i = 0; i < engine->num_recent_queries; i++) {
        free(engine->recent_queries[i]);
    }
    free(engine->recent_queries);
    
    // Libération de la matrice d'embeddings
    if (engine->embedding_matrix) {
        for (int i = 0; i < engine->num_files; i++) {
            free(engine->embedding_matrix[i]);
        }
        free(engine->embedding_matrix);
    }
    free(engine->file_mapping);
    
    // Libération du cache
    for (int i = 0; i < engine->cache_size; i++) {
        free(engine->embedding_cache[i].query);
        free(engine->embedding_cache[i].embedding);
    }
    
    free(engine);
}

bool search_engine_index_file(SearchEngine* engine, const char* file_path) {
    if (!engine || !file_path) {
        if (engine) engine->last_error = SEARCH_ERROR_INVALID_PARAM;
        return false;
    }
    
    struct stat file_stat;
    if (stat(file_path, &file_stat) != 0) {
        engine->last_error = SEARCH_ERROR_IO_ERROR;
        return false;
    }
    
    // Vérifier si le fichier est déjà indexé
    for (int i = 0; i < engine->num_files; i++) {
        if (strcmp(engine->file_indices[i].path, file_path) == 0) {
            // Vérifier si le fichier a été modifié
            if (engine->file_indices[i].last_modified >= file_stat.st_mtime) {
                return true; // Déjà à jour
            }
            // Mettre à jour l'index existant
            // TODO: Implémentation de la mise à jour
        }
    }
    
    // Agrandir le tableau si nécessaire
    if (engine->num_files >= engine->capacity) {
        engine->capacity *= 2;
        FileIndex* new_indices = realloc(engine->file_indices, 
                                        engine->capacity * sizeof(FileIndex));
        if (!new_indices) {
            engine->last_error = SEARCH_ERROR_OUT_OF_MEMORY;
            return false;
        }
        engine->file_indices = new_indices;
    }
    
    // Lire le contenu du fichier
    FILE* file = fopen(file_path, "r");
    if (!file) {
        engine->last_error = SEARCH_ERROR_IO_ERROR;
        return false;
    }
    
    // Déterminer la taille du fichier
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    char* content = malloc(file_size + 1);
    if (!content) {
        fclose(file);
        engine->last_error = SEARCH_ERROR_OUT_OF_MEMORY;
        return false;
    }
    
    fread(content, 1, file_size, file);
    content[file_size] = '\0';
    fclose(file);
    
    // Créer l'index
    FileIndex* index = &engine->file_indices[engine->num_files];
    index->path = strdup(file_path);
    index->name = strdup(strrchr(file_path, '/') ? strrchr(file_path, '/') + 1 : file_path);
    index->content = content;
    index->file_size = file_size;
    index->last_modified = file_stat.st_mtime;
    index->indexed_time = time(NULL);
    index->content_hash = hash_string(content);
    
    // Générer l'embedding
    int dimension;
    switch (engine->config.mode) {
        case SEARCH_MODE_ACCURACY: dimension = EMBEDDING_DIMENSION_FULL; break;
        case SEARCH_MODE_BALANCED: dimension = EMBEDDING_DIMENSION_FAST; break;
        case SEARCH_MODE_SPEED: dimension = EMBEDDING_DIMENSION_ULTRA_FAST; break;
        default: dimension = EMBEDDING_DIMENSION_FAST; break;
    }
    
    index->embedding_dimension = dimension;
    index->embedding_vector = generate_embedding_simulation(content, dimension);
    
    if (!index->embedding_vector) {
        free(index->path);
        free(index->name);
        free(content);
        engine->last_error = SEARCH_ERROR_EMBEDDING_FAILED;
        return false;
    }
    
    engine->num_files++;
    engine->stats.total_files_indexed++;
    
    if (engine->debug_enabled) {
        printf("Debug: Indexed file %s (%ld bytes, %d dimensions)\n", 
               file_path, file_size, dimension);
    }
    
    return true;
}

bool search_engine_index_directory(SearchEngine* engine, const char* directory_path,
                                  SearchProgressCallback progress_cb, void* user_data) {
    if (!engine || !directory_path) {
        if (engine) engine->last_error = SEARCH_ERROR_INVALID_PARAM;
        return false;
    }
    
    DIR* dir = opendir(directory_path);
    if (!dir) {
        engine->last_error = SEARCH_ERROR_IO_ERROR;
        return false;
    }
    
    // Compter les fichiers d'abord (pour le callback de progrès)
    int total_files = 0;
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG && 
            (strstr(entry->d_name, ".md") || strstr(entry->d_name, ".txt"))) {
            total_files++;
        }
    }
    rewinddir(dir);
    
    int processed_files = 0;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG) {
            // Vérifier l'extension du fichier
            char* ext = strrchr(entry->d_name, '.');
            if (ext && (strcmp(ext, ".md") == 0 || strcmp(ext, ".txt") == 0)) {
                char full_path[1024];
                snprintf(full_path, sizeof(full_path), "%s/%s", directory_path, entry->d_name);
                
                if (search_engine_index_file(engine, full_path)) {
                    processed_files++;
                    
                    if (progress_cb) {
                        float progress = (float)processed_files / total_files;
                        char status[256];
                        snprintf(status, sizeof(status), "Indexing %s", entry->d_name);
                        progress_cb(progress, status, user_data);
                    }
                }
            }
        } else if (entry->d_type == DT_DIR && strcmp(entry->d_name, ".") != 0 && strcmp(entry->d_name, "..") != 0) {
            // Indexation récursive des sous-dossiers
            char subdir_path[1024];
            snprintf(subdir_path, sizeof(subdir_path), "%s/%s", directory_path, entry->d_name);
            search_engine_index_directory(engine, subdir_path, progress_cb, user_data);
        }
    }
    
    closedir(dir);
    engine->stats.last_index_update = time(NULL);
    
    return true;
}

// Recherche exacte dans les noms de fichiers
SearchResult* search_filename_fuzzy(SearchEngine* engine, const char* query, int* num_results) {
    if (!engine || !query || !num_results) {
        if (engine) engine->last_error = SEARCH_ERROR_INVALID_PARAM;
        if (num_results) *num_results = 0;
        return NULL;
    }
    
    clock_t start = clock();
    
    SearchResult* results = malloc(engine->config.max_results * sizeof(SearchResult));
    if (!results) {
        engine->last_error = SEARCH_ERROR_OUT_OF_MEMORY;
        *num_results = 0;
        return NULL;
    }
    
    int result_count = 0;
    
    for (int i = 0; i < engine->num_files && result_count < engine->config.max_results; i++) {
        float similarity = levenshtein_distance_normalized(query, engine->file_indices[i].name);
        
        if (similarity >= engine->config.similarity_threshold) {
            SearchResult* result = &results[result_count];
            result->file_path = strdup(engine->file_indices[i].path);
            result->file_name = strdup(engine->file_indices[i].name);
            result->relevance_score = similarity;
            result->file_size = engine->file_indices[i].file_size;
            result->last_modified = engine->file_indices[i].last_modified;
            result->match_type = strdup("filename_fuzzy");
            result->match_position = 0;
            
            // Aperçu du contenu
            int preview_len = strlen(engine->file_indices[i].content);
            if (preview_len > 200) preview_len = 200;
            result->content_preview = malloc(preview_len + 1);
            strncpy(result->content_preview, engine->file_indices[i].content, preview_len);
            result->content_preview[preview_len] = '\0';
            
            result_count++;
        }
    }
    
    clock_t end = clock();
    double query_time = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;
    
    // Mise à jour des statistiques
    engine->stats.total_queries++;
    engine->stats.avg_query_time_ms = (engine->stats.avg_query_time_ms * 
        (engine->stats.total_queries - 1) + query_time) / engine->stats.total_queries;
    
    *num_results = result_count;
    return results;
}

// Recherche sémantique avec embeddings
SearchResult* search_semantic_similar(SearchEngine* engine, const char* query, int* num_results) {
    if (!engine || !query || !num_results) {
        if (engine) engine->last_error = SEARCH_ERROR_INVALID_PARAM;
        if (num_results) *num_results = 0;
        return NULL;
    }
    
    clock_t start = clock();
    
    // Générer l'embedding de la requête
    int dimension = (engine->num_files > 0) ? engine->file_indices[0].embedding_dimension : 
                    EMBEDDING_DIMENSION_FAST;
    float* query_embedding = generate_embedding_simulation(query, dimension);
    
    if (!query_embedding) {
        engine->last_error = SEARCH_ERROR_EMBEDDING_FAILED;
        *num_results = 0;
        return NULL;
    }
    
    SearchResult* results = malloc(engine->config.max_results * sizeof(SearchResult));
    if (!results) {
        free(query_embedding);
        engine->last_error = SEARCH_ERROR_OUT_OF_MEMORY;
        *num_results = 0;
        return NULL;
    }
    
    int result_count = 0;
    
    for (int i = 0; i < engine->num_files && result_count < engine->config.max_results; i++) {
        float similarity = calculate_cosine_similarity(query_embedding, 
            engine->file_indices[i].embedding_vector, dimension);
        
        if (similarity >= engine->config.similarity_threshold) {
            SearchResult* result = &results[result_count];
            result->file_path = strdup(engine->file_indices[i].path);
            result->file_name = strdup(engine->file_indices[i].name);
            result->relevance_score = similarity;
            result->file_size = engine->file_indices[i].file_size;
            result->last_modified = engine->file_indices[i].last_modified;
            result->match_type = strdup("semantic_similar");
            result->match_position = -1;
            
            // Aperçu du contenu
            int preview_len = strlen(engine->file_indices[i].content);
            if (preview_len > 200) preview_len = 200;
            result->content_preview = malloc(preview_len + 1);
            strncpy(result->content_preview, engine->file_indices[i].content, preview_len);
            result->content_preview[preview_len] = '\0';
            
            result_count++;
        }
    }
    
    free(query_embedding);
    
    clock_t end = clock();
    double query_time = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;
    
    // Mise à jour des statistiques
    engine->stats.total_queries++;
    engine->stats.avg_query_time_ms = (engine->stats.avg_query_time_ms * 
        (engine->stats.total_queries - 1) + query_time) / engine->stats.total_queries;
    
    *num_results = result_count;
    return results;
}

// Recherche hybride combinant toutes les techniques
SearchResult* search_hybrid_advanced(SearchEngine* engine, const char* query, int* num_results) {
    if (!engine || !query || !num_results) {
        if (engine) engine->last_error = SEARCH_ERROR_INVALID_PARAM;
        if (num_results) *num_results = 0;
        return NULL;
    }
    
    // Combiner résultats de différents types de recherche
    int fuzzy_count = 0, semantic_count = 0;
    SearchResult* fuzzy_results = search_filename_fuzzy(engine, query, &fuzzy_count);
    SearchResult* semantic_results = search_semantic_similar(engine, query, &semantic_count);
    
    // Fusionner et déduplicater les résultats
    // TODO: Implémentation complète de la fusion
    
    // Pour l'instant, retourner les résultats sémantiques
    search_results_free(fuzzy_results, fuzzy_count);
    *num_results = semantic_count;
    return semantic_results;
}

float calculate_cosine_similarity(const float* vec1, const float* vec2, int dimension) {
    float dot_product = 0.0f;
    float norm1 = 0.0f;
    float norm2 = 0.0f;
    
    for (int i = 0; i < dimension; i++) {
        dot_product += vec1[i] * vec2[i];
        norm1 += vec1[i] * vec1[i];
        norm2 += vec2[i] * vec2[i];
    }
    
    if (norm1 == 0.0f || norm2 == 0.0f) return 0.0f;
    
    return dot_product / (sqrtf(norm1) * sqrtf(norm2));
}

void search_results_free(SearchResult* results, int num_results) {
    if (!results) return;
    
    for (int i = 0; i < num_results; i++) {
        free(results[i].file_path);
        free(results[i].file_name);
        free(results[i].content_preview);
        free(results[i].match_type);
    }
    free(results);
}

SearchStats search_engine_get_stats(SearchEngine* engine) {
    if (!engine) {
        SearchStats empty_stats = {0};
        return empty_stats;
    }
    return engine->stats;
}

const char* search_engine_get_error_message(SearchResult_t error) {
    switch (error) {
        case SEARCH_SUCCESS: return "Success";
        case SEARCH_ERROR_INVALID_PARAM: return "Invalid parameter";
        case SEARCH_ERROR_OUT_OF_MEMORY: return "Out of memory";
        case SEARCH_ERROR_IO_ERROR: return "I/O error";
        case SEARCH_ERROR_MODEL_NOT_FOUND: return "Embedding model not found";
        case SEARCH_ERROR_INDEX_CORRUPTED: return "Search index corrupted";
        case SEARCH_ERROR_EMBEDDING_FAILED: return "Embedding generation failed";
        default: return "Unknown error";
    }
}

SearchResult_t search_engine_get_last_error(SearchEngine* engine) {
    if (!engine) return SEARCH_ERROR_INVALID_PARAM;
    return engine->last_error;
}

void search_engine_clear_error(SearchEngine* engine) {
    if (engine) {
        engine->last_error = SEARCH_SUCCESS;
    }
}