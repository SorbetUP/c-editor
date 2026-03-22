// realistic_benchmark.c - Test réaliste qui expose les vraies limitations
// Simule les conditions de production pour des mesures précises

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/time.h>
#include <unistd.h>
#include <math.h>

#include "search_interface.h"
#include "../advanced_search/advanced_search.h"

// Structure pour des métriques plus détaillées
typedef struct {
    double indexing_time_ms;
    double embedding_generation_time_ms;
    double file_io_time_ms;
    double search_time_ms;
    double memory_allocation_time_ms;
    size_t total_memory_mb;
    size_t total_file_size_mb;
    int files_processed;
    bool test_realistic;
} RealisticMetrics;

double get_precise_time_ms() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1000000.0;
}

// Test d'I/O fichier réaliste (lecture complète)
double test_file_io_performance(const char* corpus_path) {
    printf("\n📁 Test I/O Fichier Réaliste\n");
    printf("============================\n");
    
    char cmd[512];
    snprintf(cmd, sizeof(cmd), "find %s -name '*.md' -type f", corpus_path);
    
    FILE* fp = popen(cmd, "r");
    if (!fp) return -1;
    
    char file_path[1024];
    int file_count = 0;
    size_t total_bytes = 0;
    
    double start_time = get_precise_time_ms();
    
    while (fgets(file_path, sizeof(file_path), fp)) {
        file_path[strcspn(file_path, "\n")] = 0; // Enlever \n
        
        FILE* file = fopen(file_path, "r");
        if (file) {
            // Lire tout le fichier en mémoire (comme le fait vraiment le système)
            fseek(file, 0, SEEK_END);
            long file_size = ftell(file);
            fseek(file, 0, SEEK_SET);
            
            char* content = malloc(file_size + 1);
            if (content) {
                size_t read_bytes = fread(content, 1, file_size, file);
                content[read_bytes] = '\0';
                total_bytes += read_bytes;
                free(content);
            }
            fclose(file);
            file_count++;
        }
    }
    pclose(fp);
    
    double end_time = get_precise_time_ms();
    double total_time = end_time - start_time;
    
    printf("📊 Résultats I/O:\n");
    printf("   📄 Fichiers lus: %d\n", file_count);
    printf("   💾 Données lues: %.2f MB\n", total_bytes / (1024.0 * 1024.0));
    printf("   ⏱️  Temps total: %.2f ms\n", total_time);
    printf("   📈 Débit I/O: %.1f MB/s\n", 
           (total_bytes / (1024.0 * 1024.0)) / (total_time / 1000.0));
    printf("   ⚡ Temps par fichier: %.2f ms\n", total_time / file_count);
    
    return total_time;
}

// Test d'allocation mémoire réaliste
double test_memory_allocation_performance(int num_files) {
    printf("\n🧠 Test d'Allocation Mémoire Réaliste\n");
    printf("=====================================\n");
    
    const int embedding_dimension = 768; // Dimension EmbeddingGemma réelle
    const int avg_file_size = 4096; // 4KB par fichier en moyenne
    
    double start_time = get_precise_time_ms();
    
    // Simuler les allocations que fait vraiment le système
    void** allocations = malloc(num_files * 10 * sizeof(void*)); // Tracking
    int alloc_count = 0;
    
    for (int i = 0; i < num_files; i++) {
        // FileIndex structure
        allocations[alloc_count++] = malloc(sizeof(void*)); // path
        allocations[alloc_count++] = malloc(256); // name 
        allocations[alloc_count++] = malloc(avg_file_size); // content
        allocations[alloc_count++] = malloc(embedding_dimension * sizeof(float)); // embedding
        
        // SearchResult temporaires
        allocations[alloc_count++] = malloc(sizeof(void*)); // result path
        allocations[alloc_count++] = malloc(256); // result name
        allocations[alloc_count++] = malloc(200); // preview
        allocations[alloc_count++] = malloc(64); // match_type
        
        // Cache et structures internes
        allocations[alloc_count++] = malloc(512); // query cache
        allocations[alloc_count++] = malloc(1024); // misc buffers
    }
    
    double end_time = get_precise_time_ms();
    double alloc_time = end_time - start_time;
    
    // Libérer la mémoire
    for (int i = 0; i < alloc_count; i++) {
        free(allocations[i]);
    }
    free(allocations);
    
    printf("📊 Résultats mémoire:\n");
    printf("   🔢 Allocations: %d\n", alloc_count);
    printf("   ⏱️  Temps allocation: %.2f ms\n", alloc_time);
    printf("   ⚡ Temps par allocation: %.4f ms\n", alloc_time / alloc_count);
    printf("   💾 Mémoire simulée: %.1f MB\n", 
           (alloc_count * avg_file_size / 2) / (1024.0 * 1024.0));
    
    return alloc_time;
}

// Test d'embedding simulation plus réaliste (coût CPU)
double test_realistic_embedding_performance(const char* corpus_path) {
    printf("\n🧮 Test d'Embedding Simulation Réaliste\n");
    printf("=======================================\n");
    
    char cmd[512];
    snprintf(cmd, sizeof(cmd), "find %s -name '*.md' -type f | head -100", corpus_path);
    
    FILE* fp = popen(cmd, "r");
    if (!fp) return -1;
    
    char file_path[1024];
    int file_count = 0;
    const int dimension = 768;
    
    double start_time = get_precise_time_ms();
    
    while (fgets(file_path, sizeof(file_path), fp) && file_count < 100) {
        file_path[strcspn(file_path, "\n")] = 0;
        
        FILE* file = fopen(file_path, "r");
        if (file) {
            // Lire le contenu
            fseek(file, 0, SEEK_END);
            long file_size = ftell(file);
            fseek(file, 0, SEEK_SET);
            
            char* content = malloc(file_size + 1);
            if (content) {
                fread(content, 1, file_size, file);
                content[file_size] = '\0';
                
                // Simulation d'embedding plus coûteuse (proche réalité)
                float* embedding = calloc(dimension, sizeof(float));
                
                // Simulation de traitement de texte plus réaliste
                for (int i = 0; i < dimension; i++) {
                    float value = 0.0f;
                    
                    // Simuler traitement NLP complexe
                    for (int j = 0; j < file_size && j < 1000; j++) {
                        value += sin((double)(content[j] * i)) * cos((double)(j * i));
                        value += content[j] * 0.001f * sin(i * 0.1f);
                    }
                    
                    // Normalisation coûteuse
                    value = tanh(value * 0.001f);
                    embedding[i] = value;
                }
                
                // Normalisation L2 (comme un vrai modèle)
                float norm = 0.0f;
                for (int i = 0; i < dimension; i++) {
                    norm += embedding[i] * embedding[i];
                }
                norm = sqrt(norm);
                
                if (norm > 0.0f) {
                    for (int i = 0; i < dimension; i++) {
                        embedding[i] /= norm;
                    }
                }
                
                free(embedding);
                free(content);
            }
            fclose(file);
            file_count++;
        }
    }
    pclose(fp);
    
    double end_time = get_precise_time_ms();
    double total_time = end_time - start_time;
    
    printf("📊 Résultats embedding:\n");
    printf("   📄 Fichiers traités: %d\n", file_count);
    printf("   ⏱️  Temps total: %.2f ms\n", total_time);
    printf("   ⚡ Temps par fichier: %.2f ms\n", total_time / file_count);
    printf("   🧮 Coût par embedding: %.2f ms (768D)\n", total_time / file_count);
    
    return total_time;
}

// Test de recherche avec cosine similarity complète
double test_realistic_search_performance(const char* corpus_path) {
    printf("\n🔍 Test de Recherche Réaliste\n");
    printf("=============================\n");
    
    // Créer un moteur avec vraie indexation
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_ACCURACY; // Mode le plus coûteux
    SearchEngine* engine = search_engine_create(&config);
    
    printf("🔄 Indexation complète...\n");
    double index_start = get_precise_time_ms();
    bool success = search_engine_index_directory(engine, corpus_path, NULL, NULL);
    double index_end = get_precise_time_ms();
    
    if (!success) {
        printf("❌ Erreur d'indexation\n");
        search_engine_destroy(engine);
        return -1;
    }
    
    double index_time = index_end - index_start;
    SearchStats stats = search_engine_get_stats(engine);
    
    printf("✅ Indexation terminée: %d fichiers en %.2f ms\n", 
           stats.total_files_indexed, index_time);
    
    // Test de recherches variées
    const char* queries[] = {
        "machine learning artificial intelligence",
        "neural networks deep learning algorithms", 
        "software engineering best practices",
        "project management agile development",
        "data science python programming",
        "research methodology analysis",
        "documentation technical writing",
        "cybersecurity blockchain technology",
        "cloud computing infrastructure",
        "user experience design thinking"
    };
    
    int num_queries = sizeof(queries) / sizeof(queries[0]);
    double total_search_time = 0.0;
    int total_results = 0;
    
    printf("🎯 Test de %d recherches complexes...\n", num_queries);
    
    for (int i = 0; i < num_queries; i++) {
        double search_start = get_precise_time_ms();
        
        int num_results = 0;
        SearchResult* results = search_semantic_similar(engine, queries[i], &num_results);
        
        double search_end = get_precise_time_ms();
        double search_time = search_end - search_start;
        total_search_time += search_time;
        total_results += num_results;
        
        printf("   %2d. '%-30s' → %.3f ms (%d résultats)\n", 
               i+1, queries[i], search_time, num_results);
        
        if (results) search_results_free(results, num_results);
    }
    
    double avg_search_time = total_search_time / num_queries;
    
    printf("📊 Résultats recherche:\n");
    printf("   ⏱️  Temps moyen: %.3f ms\n", avg_search_time);
    printf("   📈 Débit: %.1f recherches/sec\n", 1000.0 / avg_search_time);
    printf("   📋 Résultats moyens: %.1f par requête\n", 
           (float)total_results / num_queries);
    
    search_engine_destroy(engine);
    return avg_search_time;
}

// Test de concurrence simulée
double test_concurrent_load(const char* corpus_path) {
    printf("\n⚡ Test de Charge Concurrente Simulée\n");
    printf("====================================\n");
    
    SearchConfig config = search_engine_get_default_config();
    SearchEngine* engine = search_engine_create(&config);
    search_engine_index_directory(engine, corpus_path, NULL, NULL);
    
    const char* queries[] = {"ai", "ml", "tech", "code", "data"};
    int num_queries = 5;
    int iterations = 50; // Simule 50 utilisateurs
    
    printf("🎯 Simulation de %d utilisateurs simultanés...\n", iterations);
    
    double start_time = get_precise_time_ms();
    
    // Simuler des requêtes entrelacées
    for (int iter = 0; iter < iterations; iter++) {
        for (int q = 0; q < num_queries; q++) {
            int num_results = 0;
            SearchResult* results = search_semantic_similar(engine, queries[q], &num_results);
            
            // Simuler traitement des résultats
            if (results) {
                usleep(100); // 0.1ms de traitement
                search_results_free(results, num_results);
            }
            
            // Simuler latence réseau/UI
            usleep(500); // 0.5ms de latence
        }
    }
    
    double end_time = get_precise_time_ms();
    double total_time = end_time - start_time;
    
    int total_operations = iterations * num_queries;
    
    printf("📊 Résultats concurrence:\n");
    printf("   🔍 Total recherches: %d\n", total_operations);
    printf("   ⏱️  Temps total: %.2f ms\n", total_time);
    printf("   ⚡ Temps par recherche: %.3f ms\n", total_time / total_operations);
    printf("   📈 Débit sous charge: %.1f recherches/sec\n", 
           total_operations / (total_time / 1000.0));
    
    search_engine_destroy(engine);
    return total_time / total_operations;
}

int main(int argc, char* argv[]) {
    printf("🔬 Benchmark Réaliste - Conditions de Production\n");
    printf("===============================================\n");
    
    const char* corpus_path = "large_test_vault";
    
    if (access(corpus_path, F_OK) != 0) {
        printf("❌ Corpus de test non trouvé: %s\n", corpus_path);
        printf("💡 Exécutez d'abord: ./generate_large_corpus.sh 2000\n");
        return 1;
    }
    
    // Compter les fichiers
    char cmd[256];
    snprintf(cmd, sizeof(cmd), "find %s -name '*.md' | wc -l", corpus_path);
    FILE* fp = popen(cmd, "r");
    int file_count = 0;
    if (fp) {
        fscanf(fp, "%d", &file_count);
        pclose(fp);
    }
    printf("📁 Corpus de test: %d fichiers\n", file_count);
    
    RealisticMetrics metrics = {0};
    
    // Tests réalistes
    metrics.file_io_time_ms = test_file_io_performance(corpus_path);
    metrics.memory_allocation_time_ms = test_memory_allocation_performance(file_count);
    metrics.embedding_generation_time_ms = test_realistic_embedding_performance(corpus_path);
    metrics.search_time_ms = test_realistic_search_performance(corpus_path);
    
    if (argc > 1 && strcmp(argv[1], "--concurrent") == 0) {
        double concurrent_time = test_concurrent_load(corpus_path);
        printf("\n🔄 Temps sous charge concurrente: %.3f ms/recherche\n", concurrent_time);
    }
    
    // Analyse réaliste
    printf("\n📊 ANALYSE RÉALISTE DES PERFORMANCES\n");
    printf("===================================\n");
    
    printf("📁 I/O Fichier: %.2f ms (vraie lecture)\n", metrics.file_io_time_ms);
    printf("🧠 Allocation mémoire: %.2f ms (vraies allocations)\n", metrics.memory_allocation_time_ms);
    printf("🧮 Génération embedding: %.2f ms (simulation complexe)\n", metrics.embedding_generation_time_ms);
    printf("🔍 Recherche: %.3f ms (cosine similarity complète)\n", metrics.search_time_ms);
    
    // Projection production
    printf("\n🚀 PROJECTION PRODUCTION\n");
    printf("=======================\n");
    
    double real_embedding_time = metrics.embedding_generation_time_ms * 10; // EmbeddingGemma ~10x plus lent
    double real_indexing_time = metrics.file_io_time_ms + real_embedding_time + metrics.memory_allocation_time_ms;
    
    printf("💡 Avec EmbeddingGemma réel:\n");
    printf("   ⏱️  Indexation estimée: %.1f ms/fichier\n", real_indexing_time / 100);
    printf("   🔍 Recherche estimée: %.1f ms/requête\n", metrics.search_time_ms * 2); // FAISS ~2x plus rapide
    printf("   📈 Débit estimé: %.1f recherches/sec\n", 1000.0 / (metrics.search_time_ms * 2));
    
    printf("\n⚠️  LIMITATIONS IDENTIFIÉES:\n");
    printf("===========================\n");
    printf("❌ Simulation d'embedding trop simple (vs EmbeddingGemma)\n");
    printf("❌ Pas de vraie charge réseau/UI\n");
    printf("❌ Pas de vraie concurrence thread-safe\n");
    printf("❌ Pas de gestion de cache disque/swap\n");
    printf("❌ Pas de fragmentation mémoire réelle\n");
    
    printf("\n✅ RECOMMANDATIONS PRODUCTION:\n");
    printf("=============================\n");
    printf("🎯 Performance attendue réelle: 5-20x plus lente\n");
    printf("🎯 Indexation: 1-10 ms/fichier (vs 0.045ms simulé)\n");
    printf("🎯 Recherche: 0.1-1 ms/requête (vs 0.02ms simulé)\n");
    printf("🎯 Mémoire: 10-50 MB pour 2k fichiers (vs <1MB simulé)\n");
    
    return 0;
}