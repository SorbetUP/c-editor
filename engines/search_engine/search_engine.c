#include "search_engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <time.h>
#include <regex.h>
#include <math.h>

#define HASH_TABLE_SIZE 10007
#define MAX_WORD_LENGTH 256
#define MAX_LINE_LENGTH 4096

// Hash function for word index
static size_t hash_word(const char* word) {
    size_t hash = 5381;
    for (int i = 0; word[i]; i++) {
        hash = ((hash << 5) + hash) + (unsigned char)tolower(word[i]);
    }
    return hash % HASH_TABLE_SIZE;
}

// Create a new search engine index
search_index_t* search_engine_create(size_t max_documents) {
    search_index_t* index = calloc(1, sizeof(search_index_t));
    if (!index) return NULL;
    
    index->documents = calloc(max_documents, sizeof(search_document_t*));
    if (!index->documents) {
        free(index);
        return NULL;
    }
    
    index->max_documents = max_documents;
    index->document_count = 0;
    
    // Initialize word index
    index->word_index = calloc(1, sizeof(word_index_t));
    if (!index->word_index) {
        free(index->documents);
        free(index);
        return NULL;
    }
    
    index->word_index->entries = calloc(HASH_TABLE_SIZE, sizeof(word_index_entry_t*));
    if (!index->word_index->entries) {
        free(index->word_index);
        free(index->documents);
        free(index);
        return NULL;
    }
    
    index->word_index->table_size = HASH_TABLE_SIZE;
    index->embeddings_enabled = false;
    index->embedding_dimension = 0;
    
    return index;
}

// Destroy search engine and free memory
void search_engine_destroy(search_index_t* index) {
    if (!index) return;
    
    // Free documents
    for (int i = 0; i < index->document_count; i++) {
        if (index->documents[i]) {
            free(index->documents[i]->filepath);
            free(index->documents[i]->content);
            
            if (index->documents[i]->lines) {
                for (int j = 0; j < index->documents[i]->line_count; j++) {
                    free(index->documents[i]->lines[j]);
                }
                free(index->documents[i]->lines);
            }
            
            free(index->documents[i]);
        }
    }
    free(index->documents);
    
    // Free word index
    if (index->word_index) {
        for (size_t i = 0; i < index->word_index->table_size; i++) {
            word_index_entry_t* entry = index->word_index->entries[i];
            while (entry) {
                word_index_entry_t* next = entry->next;
                free(entry->word);
                free(entry->document_ids);
                free(entry->positions);
                free(entry);
                entry = next;
            }
        }
        free(index->word_index->entries);
        free(index->word_index);
    }
    
    // Free embeddings
    if (index->embeddings) {
        for (int i = 0; i < index->document_count; i++) {
            free(index->embeddings[i]);
        }
        free(index->embeddings);
    }
    
    free(index);
}

// Parse content into lines
static char** parse_lines(const char* content, int* line_count) {
    if (!content) return NULL;
    
    // Count lines
    int count = 1;
    for (const char* p = content; *p; p++) {
        if (*p == '\n') count++;
    }
    
    char** lines = malloc(count * sizeof(char*));
    if (!lines) return NULL;
    
    // Split content into lines
    const char* start = content;
    int line_idx = 0;
    
    for (const char* p = content; *p; p++) {
        if (*p == '\n' || *(p + 1) == '\0') {
            size_t len = p - start + (*p != '\n' ? 1 : 0);
            lines[line_idx] = malloc(len + 1);
            if (lines[line_idx]) {
                strncpy(lines[line_idx], start, len);
                lines[line_idx][len] = '\0';
            }
            line_idx++;
            start = p + 1;
        }
    }
    
    *line_count = count;
    return lines;
}

// Add word to index
static void add_word_to_index(search_index_t* index, const char* word, int document_id, int position) {
    if (!word || strlen(word) == 0) return;
    
    // Convert to lowercase for case-insensitive indexing
    char lower_word[MAX_WORD_LENGTH];
    strncpy(lower_word, word, MAX_WORD_LENGTH - 1);
    lower_word[MAX_WORD_LENGTH - 1] = '\0';
    
    for (int i = 0; lower_word[i]; i++) {
        lower_word[i] = tolower(lower_word[i]);
    }
    
    size_t hash = hash_word(lower_word);
    word_index_entry_t* entry = index->word_index->entries[hash];
    
    // Find existing entry
    while (entry) {
        if (strcmp(entry->word, lower_word) == 0) {
            // Add to existing entry
            entry->occurrence_count++;
            entry->document_ids = realloc(entry->document_ids, entry->occurrence_count * sizeof(int));
            entry->positions = realloc(entry->positions, entry->occurrence_count * sizeof(int));
            
            if (entry->document_ids && entry->positions) {
                entry->document_ids[entry->occurrence_count - 1] = document_id;
                entry->positions[entry->occurrence_count - 1] = position;
            }
            return;
        }
        entry = entry->next;
    }
    
    // Create new entry
    entry = calloc(1, sizeof(word_index_entry_t));
    if (!entry) return;
    
    entry->word = strdup(lower_word);
    entry->document_ids = malloc(sizeof(int));
    entry->positions = malloc(sizeof(int));
    
    if (entry->word && entry->document_ids && entry->positions) {
        entry->document_ids[0] = document_id;
        entry->positions[0] = position;
        entry->occurrence_count = 1;
        
        // Insert at head of chain
        entry->next = index->word_index->entries[hash];
        index->word_index->entries[hash] = entry;
        index->word_index->entry_count++;
    } else {
        free(entry->word);
        free(entry->document_ids);
        free(entry->positions);
        free(entry);
    }
}

// Index a document's words
static void index_document_words(search_index_t* index, search_document_t* doc) {
    const char* content = doc->content;
    int position = 0;
    char word[MAX_WORD_LENGTH];
    int word_pos = 0;
    
    for (int i = 0; content[i]; i++) {
        char c = content[i];
        
        if (isalnum(c) || c == '_') {
            if (word_pos < MAX_WORD_LENGTH - 1) {
                word[word_pos++] = c;
            }
        } else {
            if (word_pos > 0) {
                word[word_pos] = '\0';
                if (strlen(word) > 1) { // Skip single character words
                    add_word_to_index(index, word, doc->document_id, position);
                }
                word_pos = 0;
                position++;
            }
        }
    }
    
    // Handle last word
    if (word_pos > 0) {
        word[word_pos] = '\0';
        if (strlen(word) > 1) {
            add_word_to_index(index, word, doc->document_id, position);
        }
    }
}

// Add document to search index
int search_engine_add_document(search_index_t* index, const char* filepath, const char* content) {
    if (!index || !filepath || !content) return -1;
    if (index->document_count >= index->max_documents) return -2;
    
    search_document_t* doc = calloc(1, sizeof(search_document_t));
    if (!doc) return -3;
    
    doc->document_id = index->document_count;
    doc->filepath = strdup(filepath);
    doc->content = strdup(content);
    doc->content_length = strlen(content);
    doc->last_modified = time(NULL);
    
    if (!doc->filepath || !doc->content) {
        free(doc->filepath);
        free(doc->content);
        free(doc);
        return -4;
    }
    
    // Parse content into lines
    doc->lines = parse_lines(content, &doc->line_count);
    
    // Add to document array
    index->documents[index->document_count] = doc;
    
    // Index the document's words
    index_document_words(index, doc);
    
    index->document_count++;
    
    return doc->document_id;
}

// Calculate fuzzy match score using Levenshtein distance
static float calculate_fuzzy_score(const char* str1, const char* str2) {
    if (!str1 || !str2) return 0.0f;
    
    int len1 = strlen(str1);
    int len2 = strlen(str2);
    
    if (len1 == 0) return (len2 == 0) ? 1.0f : 0.0f;
    if (len2 == 0) return 0.0f;
    
    // Dynamic programming matrix for Levenshtein distance
    int matrix[len1 + 1][len2 + 1];
    
    // Initialize first row and column
    for (int i = 0; i <= len1; i++) matrix[i][0] = i;
    for (int j = 0; j <= len2; j++) matrix[0][j] = j;
    
    // Fill the matrix
    for (int i = 1; i <= len1; i++) {
        for (int j = 1; j <= len2; j++) {
            int cost = (tolower(str1[i-1]) == tolower(str2[j-1])) ? 0 : 1;
            
            int substitute = matrix[i-1][j-1] + cost;
            int delete = matrix[i-1][j] + 1;
            int insert = matrix[i][j-1] + 1;
            
            matrix[i][j] = (substitute < delete) ? 
                          (substitute < insert ? substitute : insert) :
                          (delete < insert ? delete : insert);
        }
    }
    
    int max_len = (len1 > len2) ? len1 : len2;
    return 1.0f - (float)matrix[len1][len2] / max_len;
}

// Create search result
static search_result_t* create_search_result(int document_id, int line_number, int col_start, int col_end, 
                                           float score, const char* context, const char* matched_text) {
    search_result_t* result = calloc(1, sizeof(search_result_t));
    if (!result) return NULL;
    
    result->document_id = document_id;
    result->line_number = line_number;
    result->column_start = col_start;
    result->column_end = col_end;
    result->relevance_score = score;
    result->context = context ? strdup(context) : NULL;
    result->matched_text = matched_text ? strdup(matched_text) : NULL;
    
    return result;
}

// Basic search implementation
search_result_t* search_engine_search(search_index_t* index, const search_query_t* query, int* result_count) {
    if (!index || !query || !query->query || !result_count) {
        *result_count = 0;
        return NULL;
    }
    
    // For now, implement simple text search
    // TODO: Use word index for better performance
    
    search_result_t* results = NULL;
    int count = 0;
    int capacity = 0;
    
    for (int doc_idx = 0; doc_idx < index->document_count; doc_idx++) {
        search_document_t* doc = index->documents[doc_idx];
        
        for (int line_idx = 0; line_idx < doc->line_count; line_idx++) {
            const char* line = doc->lines[line_idx];
            const char* match_pos = line;
            
            while ((match_pos = strstr(match_pos, query->query)) != NULL) {
                // Expand capacity if needed
                if (count >= capacity) {
                    capacity = (capacity == 0) ? 10 : capacity * 2;
                    results = realloc(results, capacity * sizeof(search_result_t));
                    if (!results) {
                        *result_count = 0;
                        return NULL;
                    }
                }
                
                int col_start = match_pos - line;
                int col_end = col_start + strlen(query->query);
                
                search_result_t* result = create_search_result(
                    doc->document_id, line_idx, col_start, col_end,
                    1.0f, line, query->query
                );
                
                if (result) {
                    results[count++] = *result;
                    free(result);
                }
                
                match_pos++; // Continue searching in same line
                
                if (count >= query->max_results) break;
            }
            
            if (count >= query->max_results) break;
        }
        
        if (count >= query->max_results) break;
    }
    
    *result_count = count;
    return results;
}

// Fuzzy search implementation
search_result_t* search_engine_search_fuzzy(search_index_t* index, const char* query, float threshold, int* result_count) {
    if (!index || !query || !result_count) {
        *result_count = 0;
        return NULL;
    }
    
    search_result_t* results = NULL;
    int count = 0;
    int capacity = 0;
    
    // Search through word index for fuzzy matches
    for (size_t i = 0; i < index->word_index->table_size; i++) {
        word_index_entry_t* entry = index->word_index->entries[i];
        
        while (entry) {
            float score = calculate_fuzzy_score(query, entry->word);
            
            if (score >= threshold) {
                // Add all occurrences of this word
                for (int j = 0; j < entry->occurrence_count; j++) {
                    if (count >= capacity) {
                        capacity = (capacity == 0) ? 10 : capacity * 2;
                        results = realloc(results, capacity * sizeof(search_result_t));
                        if (!results) {
                            *result_count = 0;
                            return NULL;
                        }
                    }
                    
                    search_result_t* result = create_search_result(
                        entry->document_ids[j], 0, 0, 0, score, entry->word, entry->word
                    );
                    
                    if (result) {
                        results[count++] = *result;
                        free(result);
                    }
                }
            }
            
            entry = entry->next;
        }
    }
    
    *result_count = count;
    return results;
}

// Free search results
void search_engine_free_results(search_result_t* results, int count) {
    if (!results) return;
    
    for (int i = 0; i < count; i++) {
        free(results[i].context);
        free(results[i].matched_text);
    }
    
    free(results);
}

// Print statistics
void search_engine_print_stats(const search_index_t* index) {
    if (!index) return;
    
    printf("Search Engine Statistics:\n");
    printf("  Documents indexed: %d/%d\n", index->document_count, index->max_documents);
    printf("  Word index entries: %zu\n", index->word_index->entry_count);
    printf("  Hash table size: %zu\n", index->word_index->table_size);
    printf("  Embeddings enabled: %s\n", index->embeddings_enabled ? "Yes" : "No");
    
    if (index->embeddings_enabled) {
        printf("  Embedding dimension: %d\n", index->embedding_dimension);
    }
}

// Highlight matches in text
char* search_engine_highlight_matches(const char* text, const char* query, 
                                     const char* highlight_start, const char* highlight_end) {
    if (!text || !query || !highlight_start || !highlight_end) return NULL;
    
    size_t text_len = strlen(text);
    size_t query_len = strlen(query);
    size_t start_len = strlen(highlight_start);
    size_t end_len = strlen(highlight_end);
    (void)start_len; (void)end_len; // May be used for buffer sizing in full implementation
    
    // Estimate result size (pessimistic)
    size_t result_size = text_len * 2 + 1000;
    char* result = malloc(result_size);
    if (!result) return NULL;
    
    result[0] = '\0';
    const char* src = text;
    const char* match_pos;
    
    while ((match_pos = strstr(src, query)) != NULL) {
        // Copy text before match
        strncat(result, src, match_pos - src);
        
        // Add highlight start
        strcat(result, highlight_start);
        
        // Add matched text
        strncat(result, match_pos, query_len);
        
        // Add highlight end
        strcat(result, highlight_end);
        
        src = match_pos + query_len;
    }
    
    // Add remaining text
    strcat(result, src);
    
    return result;
}

// Placeholder implementations for advanced features
search_result_t* search_engine_search_regex(search_index_t* index, const char* pattern, int* result_count) {
    // TODO: Implement regex search
    (void)index; (void)pattern; // Suppress unused warnings
    *result_count = 0;
    return NULL;
}

search_result_t* search_engine_search_similar(search_index_t* index, const char* text, int* result_count) {
    // TODO: Implement similarity search using embeddings
    (void)index; (void)text; // Suppress unused warnings
    *result_count = 0;
    return NULL;
}

search_result_t* search_engine_search_semantic(search_index_t* index, const char* query, int* result_count) {
    // TODO: Implement semantic search using embeddings
    (void)index; (void)query; // Suppress unused warnings
    *result_count = 0;
    return NULL;
}

void search_engine_rebuild_index(search_index_t* index) {
    // TODO: Implement index rebuilding
    (void)index; // Suppress unused warning
}

void search_engine_optimize_index(search_index_t* index) {
    // TODO: Implement index optimization
    (void)index; // Suppress unused warning
}

bool search_engine_validate_index(const search_index_t* index) {
    // TODO: Implement index validation
    return index != NULL;
}

void search_engine_set_embedding_dimension(search_index_t* index, int dimension) {
    if (index) {
        index->embedding_dimension = dimension;
    }
}

void search_engine_enable_embeddings(search_index_t* index, bool enable) {
    if (index) {
        index->embeddings_enabled = enable;
    }
}

int search_engine_save_index(const search_index_t* index, const char* filepath) {
    // TODO: Implement index persistence
    (void)index; (void)filepath; // Suppress unused warnings
    return -1;
}

search_index_t* search_engine_load_index(const char* filepath) {
    // TODO: Implement index loading
    (void)filepath; // Suppress unused warning
    return NULL;
}

int search_engine_remove_document(search_index_t* index, int document_id) {
    // TODO: Implement document removal
    (void)index; (void)document_id; // Suppress unused warnings
    return -1;
}

int search_engine_update_document(search_index_t* index, int document_id, const char* content) {
    // TODO: Implement document updating
    (void)index; (void)document_id; (void)content; // Suppress unused warnings
    return -1;
}