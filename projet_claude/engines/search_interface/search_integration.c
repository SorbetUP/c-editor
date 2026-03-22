// search_integration.c - Démonstration d'intégration complète des deux bibliothèques
// Interface de recherche + Moteur de recherche avancée avec embedding

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>

#include "search_interface.h"
#include "../advanced_search/advanced_search.h"

// Structure pour lier les deux systèmes
typedef struct {
    SearchInterface* interface;
    SearchEngine* search_engine;
    bool live_search_enabled;
    time_t last_search_time;
    char* last_query;
} IntegratedSearchSystem;

// Callbacks pour l'intégration
void on_file_selected_integrated(const char* file_path, void* user_data) {
    printf("📂 Fichier sélectionné: %s\n", file_path);
    
    // Ici on pourrait ouvrir le fichier dans l'éditeur
    printf("   Ouverture dans l'éditeur...\n");
}

void on_search_changed_integrated(const char* query, void* user_data) {
    IntegratedSearchSystem* system = (IntegratedSearchSystem*)user_data;
    
    printf("🔍 Recherche: '%s'\n", query);
    
    if (strlen(query) == 0) {
        printf("   Recherche vidée - affichage de l'arborescence complète\n");
        return;
    }
    
    if (strlen(query) < 2) {
        printf("   Requête trop courte - attente de plus de caractères\n");
        return;
    }
    
    // Éviter les recherches trop fréquentes
    time_t now = time(NULL);
    if (now - system->last_search_time < 1 && 
        system->last_query && strcmp(system->last_query, query) == 0) {
        return;
    }
    
    system->last_search_time = now;
    free(system->last_query);
    system->last_query = strdup(query);
    
    printf("   🧠 Lancement de la recherche sémantique...\n");
    
    // Recherche sémantique
    int num_semantic = 0;
    SearchResult* semantic_results = search_semantic_similar(system->search_engine, query, &num_semantic);
    
    if (semantic_results && num_semantic > 0) {
        printf("   📋 Résultats sémantiques (%d trouvés):\n", num_semantic);
        for (int i = 0; i < num_semantic && i < 3; i++) {
            printf("      %d. %s (score: %.3f)\n", 
                   i + 1, semantic_results[i].file_name, semantic_results[i].relevance_score);
            printf("         Preview: \"%.40s...\"\n", semantic_results[i].content_preview);
        }
        search_results_free(semantic_results, num_semantic);
    }
    
    // Recherche floue dans les noms de fichiers
    int num_fuzzy = 0;
    SearchResult* fuzzy_results = search_filename_fuzzy(system->search_engine, query, &num_fuzzy);
    
    if (fuzzy_results && num_fuzzy > 0) {
        printf("   📁 Correspondances dans les noms de fichiers (%d trouvés):\n", num_fuzzy);
        for (int i = 0; i < num_fuzzy && i < 2; i++) {
            printf("      %d. %s (score: %.3f)\n", 
                   i + 1, fuzzy_results[i].file_name, fuzzy_results[i].relevance_score);
        }
        search_results_free(fuzzy_results, num_fuzzy);
    }
    
    printf("\n");
}

void on_tree_expanded_integrated(const char* directory_path, void* user_data) {
    printf("📂 Dossier déplié: %s\n", directory_path);
}

// Démonstration de navigation interactive
void interactive_demo(IntegratedSearchSystem* system) {
    printf("\n🎮 Mode Interactif - Démonstration de Navigation\n");
    printf("=============================================\n");
    printf("Commandes disponibles:\n");
    printf("  s <query>   - Rechercher\n");
    printf("  e <path>    - Déplier un dossier\n");
    printf("  c <path>    - Replier un dossier\n");
    printf("  t           - Afficher l'arborescence\n");
    printf("  stats       - Afficher les statistiques\n");
    printf("  q           - Quitter\n");
    printf("\n");
    
    char input[1024];
    char command[32];
    char argument[992];
    
    while (true) {
        printf("search> ");
        fflush(stdout);
        
        if (!fgets(input, sizeof(input), stdin)) break;
        
        // Enlever le retour à la ligne
        input[strcspn(input, "\n")] = 0;
        
        if (strlen(input) == 0) continue;
        
        // Parser la commande
        if (sscanf(input, "%31s %991[^\n]", command, argument) < 1) continue;
        
        if (strcmp(command, "q") == 0 || strcmp(command, "quit") == 0) {
            break;
        } else if (strcmp(command, "s") == 0 || strcmp(command, "search") == 0) {
            if (strlen(argument) > 0) {
                search_interface_set_search_query(system->interface, argument);
            } else {
                printf("Usage: s <query>\n");
            }
        } else if (strcmp(command, "e") == 0 || strcmp(command, "expand") == 0) {
            if (strlen(argument) > 0) {
                if (search_interface_expand_node(system->interface, argument)) {
                    printf("✅ Dossier déplié: %s\n", argument);
                } else {
                    printf("❌ Impossible de déplier: %s\n", argument);
                }
            } else {
                printf("Usage: e <path>\n");
            }
        } else if (strcmp(command, "c") == 0 || strcmp(command, "collapse") == 0) {
            if (strlen(argument) > 0) {
                if (search_interface_collapse_node(system->interface, argument)) {
                    printf("✅ Dossier replié: %s\n", argument);
                } else {
                    printf("❌ Impossible de replier: %s\n", argument);
                }
            } else {
                printf("Usage: c <path>\n");
            }
        } else if (strcmp(command, "t") == 0 || strcmp(command, "tree") == 0) {
            search_interface_print_tree(system->interface);
        } else if (strcmp(command, "stats") == 0) {
            SearchStats stats = search_engine_get_stats(system->search_engine);
            printf("📊 Statistiques du moteur de recherche:\n");
            printf("   Fichiers indexés: %d\n", stats.total_files_indexed);
            printf("   Requêtes effectuées: %d\n", stats.total_queries);
            printf("   Temps moyen de requête: %.2f ms\n", stats.avg_query_time_ms);
            printf("   Utilisation mémoire: %zu MB\n", stats.memory_usage_mb);
        } else {
            printf("Commande inconnue: %s\n", command);
        }
    }
}

// Tests de performance combinés
void performance_benchmark(IntegratedSearchSystem* system) {
    printf("\n🚀 Benchmark de Performance Intégrée\n");
    printf("====================================\n");
    
    // Test d'indexation
    clock_t start = clock();
    search_interface_refresh_tree(system->interface);
    clock_t end = clock();
    double tree_time = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;
    printf("⏱️  Construction arborescence: %.2f ms\n", tree_time);
    
    // Test de recherches multiples
    const char* test_queries[] = {
        "project", "alpha", "template", "meeting", "document", "research"
    };
    int num_queries = sizeof(test_queries) / sizeof(test_queries[0]);
    
    double total_search_time = 0.0;
    
    for (int i = 0; i < num_queries; i++) {
        start = clock();
        search_interface_set_search_query(system->interface, test_queries[i]);
        end = clock();
        
        double query_time = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;
        total_search_time += query_time;
        printf("⏱️  Recherche '%s': %.2f ms\n", test_queries[i], query_time);
    }
    
    printf("📊 Temps moyen par recherche: %.2f ms\n", total_search_time / num_queries);
    
    SearchStats stats = search_engine_get_stats(system->search_engine);
    printf("📊 Performance globale:\n");
    printf("   Total fichiers: %d\n", stats.total_files_indexed);
    printf("   Total requêtes: %d\n", stats.total_queries);
    printf("   Mémoire utilisée: %zu MB\n", stats.memory_usage_mb);
}

int main(int argc, char* argv[]) {
    printf("🚀 Système de Recherche Intégré - Test Complet\n");
    printf("==============================================\n");
    
    // Créer les données de test
    system("cd ../search_interface && make testdata 2>/dev/null");
    system("cd ../advanced_search && make testdata 2>/dev/null");
    
    printf("📝 Données de test créées\n");
    
    // Initialiser le système intégré
    IntegratedSearchSystem integrated_system = {0};
    
    // Créer l'interface de recherche
    SearchInterfaceConfig ui_config = search_interface_get_default_config();
    ui_config.auto_focus_search = true;
    ui_config.live_search = true;
    ui_config.show_hidden_files = false;
    ui_config.on_file_selected = on_file_selected_integrated;
    ui_config.on_search_changed = on_search_changed_integrated;
    ui_config.on_tree_expanded = on_tree_expanded_integrated;
    
    integrated_system.interface = search_interface_create(&ui_config);
    if (!integrated_system.interface) {
        printf("❌ Erreur: Impossible de créer l'interface de recherche\n");
        return 1;
    }
    
    // Créer le moteur de recherche avancée
    SearchConfig search_config = search_engine_get_default_config();
    search_config.mode = SEARCH_MODE_BALANCED;
    search_config.similarity_threshold = 0.3f;
    search_config.enable_caching = true;
    
    integrated_system.search_engine = search_engine_create(&search_config);
    if (!integrated_system.search_engine) {
        printf("❌ Erreur: Impossible de créer le moteur de recherche\n");
        search_interface_destroy(integrated_system.interface);
        return 1;
    }
    
    printf("✅ Moteur de recherche créé\n");
    
    // Configurer l'interface avec user_data
    search_interface_set_user_data(integrated_system.interface, &integrated_system);
    
    // Définir le répertoire racine pour l'interface
    if (!search_interface_set_root_directory(integrated_system.interface, "../search_interface/test_vault")) {
        printf("❌ Erreur: Impossible de définir le répertoire racine\n");
        search_engine_destroy(integrated_system.search_engine);
        search_interface_destroy(integrated_system.interface);
        return 1;
    }
    
    printf("✅ Interface configurée avec test_vault\n");
    
    // Indexer le répertoire avec le moteur de recherche
    printf("🔄 Indexation du répertoire...\n");
    if (!search_engine_index_directory(integrated_system.search_engine, 
                                     "../search_interface/test_vault", NULL, NULL)) {
        printf("❌ Erreur: Impossible d'indexer le répertoire\n");
    } else {
        SearchStats stats = search_engine_get_stats(integrated_system.search_engine);
        printf("✅ Indexation terminée: %d fichiers indexés\n", stats.total_files_indexed);
    }
    
    // Afficher l'arborescence initiale
    printf("\n🌳 Arborescence initiale:\n");
    search_interface_print_tree(integrated_system.interface);
    
    // Tests de démonstration automatiques
    printf("\n🧪 Tests de Démonstration Automatiques\n");
    printf("=====================================\n");
    
    // Test 1: Recherche sémantique
    printf("\n1️⃣ Test de recherche sémantique...\n");
    search_interface_set_search_query(integrated_system.interface, "artificial intelligence");
    
    sleep(1);
    
    // Test 2: Recherche dans les noms de fichiers
    printf("\n2️⃣ Test de recherche dans les noms...\n");
    search_interface_set_search_query(integrated_system.interface, "alpha");
    
    sleep(1);
    
    // Test 3: Expansion d'arborescence
    printf("\n3️⃣ Test d'expansion d'arborescence...\n");
    search_interface_expand_node(integrated_system.interface, "../search_interface/test_vault/Notes");
    search_interface_print_tree(integrated_system.interface);
    
    // Test de performance si demandé
    if (argc > 1 && strcmp(argv[1], "--benchmark") == 0) {
        performance_benchmark(&integrated_system);
    }
    
    // Mode interactif si demandé
    if (argc > 1 && strcmp(argv[1], "--interactive") == 0) {
        interactive_demo(&integrated_system);
    }
    
    printf("\n🎉 Tests terminés avec succès!\n");
    printf("💡 Utilisez --benchmark pour les tests de performance\n");
    printf("💡 Utilisez --interactive pour le mode interactif\n");
    
    // Nettoyage
    free(integrated_system.last_query);
    search_engine_destroy(integrated_system.search_engine);
    search_interface_destroy(integrated_system.interface);
    
    return 0;
}