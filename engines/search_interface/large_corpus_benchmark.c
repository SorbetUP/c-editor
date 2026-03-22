// large_corpus_benchmark.c - Test de performance sur grand corpus
// Test sur 2000+ notes pour évaluer les performances réelles

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/time.h>
#include <unistd.h>

#include "search_interface.h"
#include "../advanced_search/advanced_search.h"

// Structure pour les métriques de performance
typedef struct {
    double indexing_time_ms;
    double tree_build_time_ms;
    double avg_search_time_ms;
    double min_search_time_ms;
    double max_search_time_ms;
    int total_files_indexed;
    int total_searches_performed;
    size_t peak_memory_mb;
    double throughput_searches_per_sec;
} PerformanceMetrics;

// Fonction pour obtenir le temps en millisecondes
double get_time_ms() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return tv.tv_sec * 1000.0 + tv.tv_usec / 1000.0;
}

// Fonction pour estimer l'utilisation mémoire (approximative)
size_t get_memory_usage_mb() {
    FILE* file = fopen("/proc/self/status", "r");
    if (!file) return 0;
    
    char line[128];
    size_t vmrss = 0;
    
    while (fgets(line, 128, file) != NULL) {
        if (strncmp(line, "VmRSS:", 6) == 0) {
            sscanf(line, "VmRSS: %zu kB", &vmrss);
            break;
        }
    }
    fclose(file);
    
    return vmrss / 1024; // Convertir en MB
}

// Callback pour afficher le progrès d'indexation
void progress_callback_large(float progress, const char* status, void* user_data) {
    static int last_percent = -1;
    int current_percent = (int)(progress * 100);
    
    if (current_percent != last_percent && current_percent % 5 == 0) {
        printf("\r🔄 Indexation: %d%% - %s", current_percent, status);
        fflush(stdout);
        last_percent = current_percent;
    }
}

// Test d'indexation massive
PerformanceMetrics benchmark_large_indexing(const char* corpus_path) {
    printf("\n🚀 Test d'Indexation Massive\n");
    printf("============================\n");
    
    PerformanceMetrics metrics = {0};
    
    // Configuration optimisée pour performance
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_BALANCED; // Équilibre performance/précision
    config.similarity_threshold = 0.2f;
    config.enable_caching = true;
    
    SearchEngine* engine = search_engine_create(&config);
    if (!engine) {
        printf("❌ Erreur: Impossible de créer le moteur de recherche\n");
        return metrics;
    }
    
    printf("📁 Corpus: %s\n", corpus_path);
    
    // Mesurer le temps d'indexation
    double start_time = get_time_ms();
    size_t start_memory = get_memory_usage_mb();
    
    bool success = search_engine_index_directory(engine, corpus_path, 
                                               progress_callback_large, NULL);
    
    double end_time = get_time_ms();
    size_t end_memory = get_memory_usage_mb();
    
    printf("\n");
    
    if (!success) {
        printf("❌ Erreur lors de l'indexation\n");
        search_engine_destroy(engine);
        return metrics;
    }
    
    metrics.indexing_time_ms = end_time - start_time;
    metrics.peak_memory_mb = end_memory > start_memory ? end_memory - start_memory : end_memory;
    
    SearchStats stats = search_engine_get_stats(engine);
    metrics.total_files_indexed = stats.total_files_indexed;
    
    printf("✅ Indexation terminée!\n");
    printf("📊 Résultats:\n");
    printf("   📄 Fichiers indexés: %d\n", metrics.total_files_indexed);
    printf("   ⏱️  Temps d'indexation: %.2f ms (%.2f secondes)\n", 
           metrics.indexing_time_ms, metrics.indexing_time_ms / 1000.0);
    printf("   📈 Vitesse d'indexation: %.1f fichiers/seconde\n", 
           metrics.total_files_indexed / (metrics.indexing_time_ms / 1000.0));
    printf("   💾 Mémoire utilisée: %zu MB\n", metrics.peak_memory_mb);
    printf("   ⚡ Temps moyen par fichier: %.3f ms\n", 
           metrics.indexing_time_ms / metrics.total_files_indexed);
    
    search_engine_destroy(engine);
    return metrics;
}

// Test de performance de recherche sur le corpus
PerformanceMetrics benchmark_search_performance(const char* corpus_path) {
    printf("\n🔍 Test de Performance de Recherche\n");
    printf("===================================\n");
    
    PerformanceMetrics metrics = {0};
    
    // Recréer le moteur et réindexer
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_BALANCED;
    config.similarity_threshold = 0.2f;
    config.enable_caching = true;
    
    SearchEngine* engine = search_engine_create(&config);
    printf("🔄 Ré-indexation pour les tests de recherche...\n");
    search_engine_index_directory(engine, corpus_path, NULL, NULL);
    
    SearchStats stats = search_engine_get_stats(engine);
    metrics.total_files_indexed = stats.total_files_indexed;
    
    // Requêtes de test variées
    const char* test_queries[] = {
        "artificial intelligence",
        "machine learning", 
        "neural networks",
        "deep learning",
        "python",
        "javascript",
        "react",
        "tensorflow",
        "algorithm",
        "data science",
        "project management",
        "agile development",
        "cloud computing",
        "cybersecurity",
        "blockchain",
        "research methodology",
        "documentation",
        "meeting notes",
        "daily standup",
        "code review"
    };
    
    int num_queries = sizeof(test_queries) / sizeof(test_queries[0]);
    double total_search_time = 0.0;
    double min_time = 999999.0;
    double max_time = 0.0;
    
    printf("🎯 Test de %d requêtes différentes...\n", num_queries);
    
    for (int i = 0; i < num_queries; i++) {
        printf("   %2d/%-2d Recherche: '%-20s' ", i+1, num_queries, test_queries[i]);
        
        // Recherche sémantique
        double start_time = get_time_ms();
        int num_results = 0;
        SearchResult* results = search_semantic_similar(engine, test_queries[i], &num_results);
        double end_time = get_time_ms();
        
        double query_time = end_time - start_time;
        total_search_time += query_time;
        
        if (query_time < min_time) min_time = query_time;
        if (query_time > max_time) max_time = query_time;
        
        printf("→ %.3f ms (%d résultats)\n", query_time, num_results);
        
        if (results) search_results_free(results, num_results);
        
        // Petite pause pour éviter la surcharge
        usleep(1000); // 1ms
    }
    
    metrics.avg_search_time_ms = total_search_time / num_queries;
    metrics.min_search_time_ms = min_time;
    metrics.max_search_time_ms = max_time;
    metrics.total_searches_performed = num_queries;
    metrics.throughput_searches_per_sec = 1000.0 / metrics.avg_search_time_ms;
    
    printf("\n📊 Statistiques de recherche:\n");
    printf("   📋 Requêtes testées: %d\n", num_queries);
    printf("   ⏱️  Temps moyen: %.3f ms\n", metrics.avg_search_time_ms);
    printf("   🚀 Temps minimum: %.3f ms\n", metrics.min_search_time_ms);
    printf("   🐌 Temps maximum: %.3f ms\n", metrics.max_search_time_ms);
    printf("   📈 Débit: %.1f recherches/seconde\n", metrics.throughput_searches_per_sec);
    
    search_engine_destroy(engine);
    return metrics;
}

// Test de performance de l'interface sur le corpus
PerformanceMetrics benchmark_interface_performance(const char* corpus_path) {
    printf("\n🌳 Test de Performance de l'Interface\n");
    printf("=====================================\n");
    
    PerformanceMetrics metrics = {0};
    
    SearchInterfaceConfig config = search_interface_get_default_config();
    config.show_hidden_files = false;
    config.file_filter = strdup(".md");
    
    SearchInterface* interface = search_interface_create(&config);
    if (!interface) {
        printf("❌ Erreur: Impossible de créer l'interface\n");
        return metrics;
    }
    
    printf("🏗️  Construction de l'arborescence...\n");
    
    double start_time = get_time_ms();
    bool success = search_interface_set_root_directory(interface, corpus_path);
    double end_time = get_time_ms();
    
    if (!success) {
        printf("❌ Erreur lors de la construction de l'arborescence\n");
        search_interface_destroy(interface);
        return metrics;
    }
    
    metrics.tree_build_time_ms = end_time - start_time;
    
    // Obtenir le nombre de nœuds visibles
    int visible_count = 0;
    FileTreeNode** visible_nodes = search_interface_get_visible_nodes(interface, &visible_count);
    
    printf("✅ Arborescence construite!\n");
    printf("📊 Résultats:\n");
    printf("   🌲 Nœuds visibles: %d\n", visible_count);
    printf("   ⏱️  Temps de construction: %.2f ms\n", metrics.tree_build_time_ms);
    printf("   ⚡ Vitesse: %.1f nœuds/ms\n", visible_count / metrics.tree_build_time_ms);
    
    // Test d'expansion de nœuds
    printf("\n🔄 Test d'expansion de nœuds...\n");
    
    const char* test_paths[] = {
        "large_test_vault/Notes",
        "large_test_vault/Notes/Projects", 
        "large_test_vault/Notes/Research",
        "large_test_vault/Documentation"
    };
    
    double total_expand_time = 0.0;
    int successful_expansions = 0;
    
    for (int i = 0; i < 4; i++) {
        start_time = get_time_ms();
        bool expanded = search_interface_expand_node(interface, test_paths[i]);
        end_time = get_time_ms();
        
        if (expanded) {
            successful_expansions++;
            double expand_time = end_time - start_time;
            total_expand_time += expand_time;
            printf("   ✅ Expansion '%s': %.3f ms\n", 
                   strrchr(test_paths[i], '/') + 1, expand_time);
        }
    }
    
    if (successful_expansions > 0) {
        printf("   📊 Temps moyen d'expansion: %.3f ms\n", 
               total_expand_time / successful_expansions);
    }
    
    search_interface_destroy(interface);
    free(config.file_filter);
    return metrics;
}

// Test de stress - recherches multiples simultanées
void stress_test(const char* corpus_path) {
    printf("\n💪 Test de Stress - Recherches Intensives\n");
    printf("=========================================\n");
    
    SearchConfig config = search_engine_get_default_config();
    config.mode = SEARCH_MODE_SPEED; // Mode vitesse pour stress test
    config.enable_caching = true;
    
    SearchEngine* engine = search_engine_create(&config);
    printf("🔄 Indexation pour test de stress...\n");
    search_engine_index_directory(engine, corpus_path, NULL, NULL);
    
    const char* stress_queries[] = {
        "ai", "ml", "dev", "code", "tech", "data", "web", "app", "api", "db"
    };
    
    int num_stress_queries = sizeof(stress_queries) / sizeof(stress_queries[0]);
    int iterations = 100; // 100 itérations de chaque requête
    
    printf("🎯 Exécution de %d itérations x %d requêtes = %d recherches totales\n",
           iterations, num_stress_queries, iterations * num_stress_queries);
    
    double start_time = get_time_ms();
    int total_results = 0;
    
    for (int iter = 0; iter < iterations; iter++) {
        if (iter % 20 == 0) {
            printf("   🔄 Itération %d/%d\n", iter + 1, iterations);
        }
        
        for (int q = 0; q < num_stress_queries; q++) {
            int num_results = 0;
            SearchResult* results = search_semantic_similar(engine, stress_queries[q], &num_results);
            total_results += num_results;
            if (results) search_results_free(results, num_results);
        }
    }
    
    double end_time = get_time_ms();
    double total_time = end_time - start_time;
    
    printf("✅ Test de stress terminé!\n");
    printf("📊 Résultats:\n");
    printf("   🔍 Total recherches: %d\n", iterations * num_stress_queries);
    printf("   📋 Total résultats: %d\n", total_results);
    printf("   ⏱️  Temps total: %.2f ms (%.2f secondes)\n", total_time, total_time / 1000.0);
    printf("   ⚡ Débit moyen: %.1f recherches/seconde\n", 
           (iterations * num_stress_queries) / (total_time / 1000.0));
    printf("   📈 Résultats/seconde: %.1f\n", total_results / (total_time / 1000.0));
    
    search_engine_destroy(engine);
}

int main(int argc, char* argv[]) {
    printf("🚀 Benchmark de Performance - Grand Corpus\n");
    printf("==========================================\n");
    
    const char* corpus_path = "large_test_vault";
    
    // Vérifier que le corpus existe
    if (access(corpus_path, F_OK) != 0) {
        printf("❌ Corpus de test non trouvé: %s\n", corpus_path);
        printf("💡 Exécutez d'abord: ./generate_large_corpus.sh 2000\n");
        return 1;
    }
    
    printf("📁 Corpus de test: %s\n", corpus_path);
    
    // Compter les fichiers dans le corpus
    char cmd[256];
    snprintf(cmd, sizeof(cmd), "find %s -name '*.md' | wc -l", corpus_path);
    FILE* fp = popen(cmd, "r");
    int file_count = 0;
    if (fp) {
        fscanf(fp, "%d", &file_count);
        pclose(fp);
    }
    printf("📊 Fichiers détectés: %d\n", file_count);
    
    // Exécuter les benchmarks
    PerformanceMetrics indexing_metrics = benchmark_large_indexing(corpus_path);
    PerformanceMetrics search_metrics = benchmark_search_performance(corpus_path);
    PerformanceMetrics interface_metrics = benchmark_interface_performance(corpus_path);
    
    // Test de stress optionnel
    if (argc > 1 && strcmp(argv[1], "--stress") == 0) {
        stress_test(corpus_path);
    }
    
    // Résumé global
    printf("\n📈 RÉSUMÉ GLOBAL DES PERFORMANCES\n");
    printf("================================\n");
    printf("📁 Corpus testé: %d fichiers\n", file_count);
    printf("\n🏗️  INDEXATION:\n");
    printf("   ⏱️  Temps: %.2f ms (%.2f sec)\n", 
           indexing_metrics.indexing_time_ms, indexing_metrics.indexing_time_ms / 1000.0);
    printf("   📈 Vitesse: %.1f fichiers/sec\n", 
           indexing_metrics.total_files_indexed / (indexing_metrics.indexing_time_ms / 1000.0));
    printf("   💾 Mémoire: %zu MB\n", indexing_metrics.peak_memory_mb);
    
    printf("\n🔍 RECHERCHE:\n");
    printf("   ⏱️  Temps moyen: %.3f ms\n", search_metrics.avg_search_time_ms);
    printf("   📈 Débit: %.1f recherches/sec\n", search_metrics.throughput_searches_per_sec);
    printf("   🎯 Plage: %.3f - %.3f ms\n", 
           search_metrics.min_search_time_ms, search_metrics.max_search_time_ms);
    
    printf("\n🌳 INTERFACE:\n");
    printf("   ⏱️  Construction arbre: %.2f ms\n", interface_metrics.tree_build_time_ms);
    
    printf("\n💡 RECOMMANDATIONS:\n");
    if (search_metrics.avg_search_time_ms < 5.0) {
        printf("   ✅ Performance de recherche excellente (< 5ms)\n");
    } else if (search_metrics.avg_search_time_ms < 20.0) {
        printf("   ⚠️  Performance de recherche acceptable (< 20ms)\n");
    } else {
        printf("   ❌ Performance de recherche à optimiser (> 20ms)\n");
    }
    
    if (indexing_metrics.indexing_time_ms / indexing_metrics.total_files_indexed < 1.0) {
        printf("   ✅ Vitesse d'indexation excellente (< 1ms/fichier)\n");
    } else {
        printf("   ⚠️  Vitesse d'indexation à surveiller (> 1ms/fichier)\n");
    }
    
    printf("\n🎉 Benchmark terminé!\n");
    printf("💡 Utilisez --stress pour le test de stress intensif\n");
    
    return 0;
}