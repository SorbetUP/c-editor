#ifndef SEARCH_ENGINE_H
#define SEARCH_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <time.h>

// Search result structure
typedef struct {
    int document_id;
    int line_number;
    int column_start;
    int column_end;
    float relevance_score;
    char* context;
    char* matched_text;
} search_result_t;

// Search query structure
typedef struct {
    char* query;
    bool case_sensitive;
    bool regex_mode;
    bool whole_words_only;
    bool fuzzy_search;
    float fuzzy_threshold;
    int max_results;
} search_query_t;

// Document structure for indexing
typedef struct {
    int document_id;
    char* filepath;
    char* content;
    char** lines;
    int line_count;
    size_t content_length;
    time_t last_modified;
} search_document_t;

// Search index structure
typedef struct {
    search_document_t** documents;
    int document_count;
    int max_documents;
    
    // Word index for fast searching
    struct word_index* word_index;
    
    // Embedding support (for future ML integration)
    float** embeddings;
    int embedding_dimension;
    bool embeddings_enabled;
} search_index_t;

// Word index entry
typedef struct word_index_entry {
    char* word;
    int* document_ids;
    int* positions;
    int occurrence_count;
    struct word_index_entry* next;
} word_index_entry_t;

// Word index hash table
typedef struct word_index {
    word_index_entry_t** entries;
    size_t table_size;
    size_t entry_count;
} word_index_t;

// Search engine API
search_index_t* search_engine_create(size_t max_documents);
void search_engine_destroy(search_index_t* index);

// Document management
int search_engine_add_document(search_index_t* index, const char* filepath, const char* content);
int search_engine_remove_document(search_index_t* index, int document_id);
int search_engine_update_document(search_index_t* index, int document_id, const char* content);

// Search operations
search_result_t* search_engine_search(search_index_t* index, const search_query_t* query, int* result_count);
search_result_t* search_engine_search_fuzzy(search_index_t* index, const char* query, float threshold, int* result_count);
search_result_t* search_engine_search_regex(search_index_t* index, const char* pattern, int* result_count);

// Advanced search features
search_result_t* search_engine_search_similar(search_index_t* index, const char* text, int* result_count);
search_result_t* search_engine_search_semantic(search_index_t* index, const char* query, int* result_count);

// Indexing operations
void search_engine_rebuild_index(search_index_t* index);
void search_engine_optimize_index(search_index_t* index);

// Utility functions
void search_engine_free_results(search_result_t* results, int count);
char* search_engine_highlight_matches(const char* text, const char* query, const char* highlight_start, const char* highlight_end);

// Statistics and debugging
void search_engine_print_stats(const search_index_t* index);
bool search_engine_validate_index(const search_index_t* index);

// Configuration
void search_engine_set_embedding_dimension(search_index_t* index, int dimension);
void search_engine_enable_embeddings(search_index_t* index, bool enable);

// Persistence (for saving/loading index)
int search_engine_save_index(const search_index_t* index, const char* filepath);
search_index_t* search_engine_load_index(const char* filepath);

#endif // SEARCH_ENGINE_H