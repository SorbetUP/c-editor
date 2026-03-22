// search_interface.c - Implémentation de l'interface de recherche avec arborescence

#include "search_interface.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
#include <time.h>

// Structure interne de l'interface
struct SearchInterface {
    SearchInterfaceConfig config;
    FileTree* tree;
    SearchBar* search_bar;
    void* search_engine;           // Référence vers advanced_search engine
    void* user_data;
    SearchInterfaceResult last_error;
    
    // État UI
    bool needs_refresh;
    bool search_active;
    time_t last_search_time;
};

// ========== Utilitaires de fichiers ==========

static bool is_directory(const char* path) {
    struct stat st;
    return (stat(path, &st) == 0) && S_ISDIR(st.st_mode);
}

static bool is_hidden_file(const char* name) {
    return name[0] == '.';
}

static bool matches_filter(const char* filename, const char* filter) {
    if (!filter || strlen(filter) == 0) return true;
    
    const char* ext = strrchr(filename, '.');
    if (!ext) return false;
    
    return strstr(filter, ext) != NULL;
}

// ========== Gestion des nœuds de l'arbre ==========

static FileTreeNode* file_tree_node_create(const char* name, const char* full_path, bool is_directory) {
    FileTreeNode* node = calloc(1, sizeof(FileTreeNode));
    if (!node) return NULL;
    
    node->name = strdup(name);
    node->full_path = strdup(full_path);
    node->is_directory = is_directory;
    node->is_expanded = false;
    node->is_visible = true;
    node->depth = 0;
    
    node->children = NULL;
    node->num_children = 0;
    node->children_capacity = 0;
    
    // Obtenir les métadonnées du fichier
    struct stat st;
    if (stat(full_path, &st) == 0) {
        node->last_modified = st.st_mtime;
        node->file_size = st.st_size;
    }
    
    return node;
}

static void file_tree_node_destroy(FileTreeNode* node) {
    if (!node) return;
    
    free(node->name);
    free(node->full_path);
    
    for (int i = 0; i < node->num_children; i++) {
        file_tree_node_destroy(node->children[i]);
    }
    free(node->children);
    free(node);
}

static bool file_tree_node_add_child(FileTreeNode* parent, FileTreeNode* child) {
    if (!parent || !child) return false;
    
    // Agrandir le tableau si nécessaire
    if (parent->num_children >= parent->children_capacity) {
        int new_capacity = parent->children_capacity == 0 ? 8 : parent->children_capacity * 2;
        FileTreeNode** new_children = realloc(parent->children, 
                                            new_capacity * sizeof(FileTreeNode*));
        if (!new_children) return false;
        
        parent->children = new_children;
        parent->children_capacity = new_capacity;
    }
    
    parent->children[parent->num_children] = child;
    child->parent = parent;
    child->depth = parent->depth + 1;
    parent->num_children++;
    
    return true;
}

// Fonction de comparaison pour trier les nœuds (dossiers d'abord, puis alphabétique)
static int compare_nodes(const void* a, const void* b) {
    FileTreeNode* node_a = *(FileTreeNode**)a;
    FileTreeNode* node_b = *(FileTreeNode**)b;
    
    // Dossiers d'abord
    if (node_a->is_directory && !node_b->is_directory) return -1;
    if (!node_a->is_directory && node_b->is_directory) return 1;
    
    // Puis ordre alphabétique
    return strcmp(node_a->name, node_b->name);
}

static bool file_tree_node_sort_children(FileTreeNode* node) {
    if (!node || node->num_children <= 1) return true;
    
    qsort(node->children, node->num_children, sizeof(FileTreeNode*), compare_nodes);
    return true;
}

// ========== Construction de l'arborescence ==========

static bool build_tree_recursive(FileTreeNode* parent, const char* directory_path, 
                                const SearchInterfaceConfig* config) {
    DIR* dir = opendir(directory_path);
    if (!dir) return false;
    
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        // Ignorer . et ..
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        
        // Ignorer les fichiers cachés si non configuré
        if (!config->show_hidden_files && is_hidden_file(entry->d_name)) {
            continue;
        }
        
        // Construire le chemin complet
        char full_path[1024];
        snprintf(full_path, sizeof(full_path), "%s/%s", directory_path, entry->d_name);
        
        bool is_dir = is_directory(full_path);
        
        // Appliquer le filtre de fichiers
        if (!is_dir && config->file_filter && 
            !matches_filter(entry->d_name, config->file_filter)) {
            continue;
        }
        
        // Créer le nœud
        FileTreeNode* child = file_tree_node_create(entry->d_name, full_path, is_dir);
        if (!child) {
            closedir(dir);
            return false;
        }
        
        if (!file_tree_node_add_child(parent, child)) {
            file_tree_node_destroy(child);
            closedir(dir);
            return false;
        }
    }
    
    closedir(dir);
    
    // Trier les enfants
    file_tree_node_sort_children(parent);
    
    return true;
}

// ========== Calcul des nœuds visibles ==========

static void collect_visible_nodes_recursive(FileTreeNode* node, FileTreeNode*** visible_nodes,
                                           int* count, int* capacity) {
    if (!node || !node->is_visible) return;
    
    // Agrandir le tableau si nécessaire
    if (*count >= *capacity) {
        int new_capacity = *capacity == 0 ? 32 : *capacity * 2;
        FileTreeNode** new_array = realloc(*visible_nodes, 
                                          new_capacity * sizeof(FileTreeNode*));
        if (!new_array) return;
        
        *visible_nodes = new_array;
        *capacity = new_capacity;
    }
    
    (*visible_nodes)[*count] = node;
    (*count)++;
    
    // Si le nœud est déplié, inclure ses enfants
    if (node->is_expanded && node->is_directory) {
        for (int i = 0; i < node->num_children; i++) {
            collect_visible_nodes_recursive(node->children[i], visible_nodes, count, capacity);
        }
    }
}

static void update_visible_nodes(FileTree* tree) {
    if (!tree) return;
    
    // Réinitialiser le cache
    free(tree->visible_nodes);
    tree->visible_nodes = NULL;
    tree->num_visible_nodes = 0;
    tree->visible_capacity = 0;
    
    // Collecter les nœuds visibles
    if (tree->root) {
        collect_visible_nodes_recursive(tree->root, &tree->visible_nodes,
                                      &tree->num_visible_nodes, &tree->visible_capacity);
    }
}

// ========== API Principal ==========

SearchInterfaceConfig search_interface_get_default_config(void) {
    SearchInterfaceConfig config = {0};
    
    config.width = 300;
    config.height = 600;
    config.search_bar_height = 40;
    
    config.auto_focus_search = true;
    config.live_search = true;
    config.search_delay_ms = 300;
    
    config.show_search_suggestions = true;
    config.max_suggestions = 5;
    config.highlight_matches = true;
    
    config.show_hidden_files = false;
    config.file_filter = strdup(".md,.txt,.json");
    
    config.on_file_selected = NULL;
    config.on_search_changed = NULL;
    config.on_tree_expanded = NULL;
    
    return config;
}

SearchInterface* search_interface_create(const SearchInterfaceConfig* config) {
    SearchInterface* interface = calloc(1, sizeof(SearchInterface));
    if (!interface) return NULL;
    
    // Configuration
    if (config) {
        interface->config = *config;
    } else {
        interface->config = search_interface_get_default_config();
    }
    
    // Créer l'arborescence
    interface->tree = calloc(1, sizeof(FileTree));
    if (!interface->tree) {
        free(interface);
        return NULL;
    }
    
    interface->tree->show_hidden_files = false;
    interface->tree->show_file_sizes = true;
    interface->tree->file_filter = strdup(".md,.txt,.json");
    
    // Créer la barre de recherche
    interface->search_bar = calloc(1, sizeof(SearchBar));
    if (!interface->search_bar) {
        free(interface->tree);
        free(interface);
        return NULL;
    }
    
    interface->search_bar->query = strdup("");
    interface->search_bar->placeholder = strdup("Rechercher dans les notes...");
    interface->search_bar->has_focus = interface->config.auto_focus_search;
    
    interface->last_error = SEARCH_INTERFACE_SUCCESS;
    interface->needs_refresh = true;
    
    return interface;
}

void search_interface_destroy(SearchInterface* interface) {
    if (!interface) return;
    
    // Détruire l'arborescence
    if (interface->tree) {
        if (interface->tree->root) {
            file_tree_node_destroy(interface->tree->root);
        }
        free(interface->tree->visible_nodes);
        free(interface->tree->file_filter);
        free(interface->tree);
    }
    
    // Détruire la barre de recherche
    if (interface->search_bar) {
        free(interface->search_bar->query);
        free(interface->search_bar->placeholder);
        
        for (int i = 0; i < interface->search_bar->num_suggestions; i++) {
            free(interface->search_bar->suggestions[i]);
        }
        free(interface->search_bar->suggestions);
        
        for (int i = 0; i < interface->search_bar->num_history; i++) {
            free(interface->search_bar->history[i]);
        }
        free(interface->search_bar->history);
        
        free(interface->search_bar);
    }
    
    free(interface);
}

bool search_interface_set_root_directory(SearchInterface* interface, const char* directory_path) {
    if (!interface || !directory_path) {
        if (interface) interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PARAM;
        return false;
    }
    
    if (!is_directory(directory_path)) {
        interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PATH;
        return false;
    }
    
    // Détruire l'ancienne arborescence
    if (interface->tree->root) {
        file_tree_node_destroy(interface->tree->root);
        interface->tree->root = NULL;
    }
    
    // Créer le nœud racine
    const char* dir_name = strrchr(directory_path, '/');
    dir_name = dir_name ? dir_name + 1 : directory_path;
    
    interface->tree->root = file_tree_node_create(dir_name, directory_path, true);
    if (!interface->tree->root) {
        interface->last_error = SEARCH_INTERFACE_ERROR_OUT_OF_MEMORY;
        return false;
    }
    
    // Construire l'arborescence
    if (!build_tree_recursive(interface->tree->root, directory_path, &interface->config)) {
        file_tree_node_destroy(interface->tree->root);
        interface->tree->root = NULL;
        interface->last_error = SEARCH_INTERFACE_ERROR_IO_ERROR;
        return false;
    }
    
    // Déplier le nœud racine par défaut
    interface->tree->root->is_expanded = true;
    
    // Mettre à jour les nœuds visibles
    update_visible_nodes(interface->tree);
    
    interface->needs_refresh = true;
    interface->last_error = SEARCH_INTERFACE_SUCCESS;
    return true;
}

bool search_interface_refresh_tree(SearchInterface* interface) {
    if (!interface || !interface->tree->root) {
        if (interface) interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PARAM;
        return false;
    }
    
    char* root_path = strdup(interface->tree->root->full_path);
    bool success = search_interface_set_root_directory(interface, root_path);
    free(root_path);
    
    return success;
}

bool search_interface_expand_node(SearchInterface* interface, const char* path) {
    if (!interface || !path) {
        if (interface) interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PARAM;
        return false;
    }
    
    // Trouver le nœud par son chemin
    // (Implémentation simplifiée - dans une vraie version, on parcourrait l'arbre)
    FileTreeNode** visible_nodes = interface->tree->visible_nodes;
    int count = interface->tree->num_visible_nodes;
    
    for (int i = 0; i < count; i++) {
        if (strcmp(visible_nodes[i]->full_path, path) == 0) {
            if (visible_nodes[i]->is_directory) {
                visible_nodes[i]->is_expanded = true;
                
                // Construire les enfants si nécessaire
                if (visible_nodes[i]->num_children == 0) {
                    build_tree_recursive(visible_nodes[i], path, &interface->config);
                }
                
                update_visible_nodes(interface->tree);
                interface->needs_refresh = true;
                
                // Callback
                if (interface->config.on_tree_expanded) {
                    interface->config.on_tree_expanded(path, interface->user_data);
                }
                
                return true;
            }
        }
    }
    
    interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PATH;
    return false;
}

bool search_interface_collapse_node(SearchInterface* interface, const char* path) {
    if (!interface || !path) {
        if (interface) interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PARAM;
        return false;
    }
    
    FileTreeNode** visible_nodes = interface->tree->visible_nodes;
    int count = interface->tree->num_visible_nodes;
    
    for (int i = 0; i < count; i++) {
        if (strcmp(visible_nodes[i]->full_path, path) == 0) {
            if (visible_nodes[i]->is_directory) {
                visible_nodes[i]->is_expanded = false;
                update_visible_nodes(interface->tree);
                interface->needs_refresh = true;
                return true;
            }
        }
    }
    
    interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PATH;
    return false;
}

bool search_interface_toggle_node(SearchInterface* interface, const char* path) {
    if (!interface || !path) return false;
    
    FileTreeNode** visible_nodes = interface->tree->visible_nodes;
    int count = interface->tree->num_visible_nodes;
    
    for (int i = 0; i < count; i++) {
        if (strcmp(visible_nodes[i]->full_path, path) == 0) {
            if (visible_nodes[i]->is_directory) {
                if (visible_nodes[i]->is_expanded) {
                    return search_interface_collapse_node(interface, path);
                } else {
                    return search_interface_expand_node(interface, path);
                }
            }
        }
    }
    
    return false;
}

bool search_interface_set_search_query(SearchInterface* interface, const char* query) {
    if (!interface || !query) {
        if (interface) interface->last_error = SEARCH_INTERFACE_ERROR_INVALID_PARAM;
        return false;
    }
    
    free(interface->search_bar->query);
    interface->search_bar->query = strdup(query);
    interface->search_active = strlen(query) > 0;
    interface->last_search_time = time(NULL);
    
    // Callback
    if (interface->config.on_search_changed) {
        interface->config.on_search_changed(query, interface->user_data);
    }
    
    return true;
}

const char* search_interface_get_search_query(SearchInterface* interface) {
    if (!interface || !interface->search_bar) return NULL;
    return interface->search_bar->query;
}

bool search_interface_focus_search(SearchInterface* interface) {
    if (!interface) return false;
    
    interface->search_bar->has_focus = true;
    return true;
}

FileTreeNode** search_interface_get_visible_nodes(SearchInterface* interface, int* count) {
    if (!interface || !count) return NULL;
    
    *count = interface->tree->num_visible_nodes;
    return interface->tree->visible_nodes;
}

void search_interface_set_user_data(SearchInterface* interface, void* user_data) {
    if (interface) {
        interface->user_data = user_data;
    }
}

void* search_interface_get_user_data(SearchInterface* interface) {
    return interface ? interface->user_data : NULL;
}

// Gestion des erreurs
SearchInterfaceResult search_interface_get_last_error(SearchInterface* interface) {
    if (!interface) return SEARCH_INTERFACE_ERROR_INVALID_PARAM;
    return interface->last_error;
}

const char* search_interface_get_error_message(SearchInterfaceResult error) {
    switch (error) {
        case SEARCH_INTERFACE_SUCCESS: return "Success";
        case SEARCH_INTERFACE_ERROR_INVALID_PARAM: return "Invalid parameter";
        case SEARCH_INTERFACE_ERROR_OUT_OF_MEMORY: return "Out of memory";
        case SEARCH_INTERFACE_ERROR_IO_ERROR: return "I/O error";
        case SEARCH_INTERFACE_ERROR_INVALID_PATH: return "Invalid path";
        default: return "Unknown error";
    }
}

// Debug
void search_interface_print_tree(SearchInterface* interface) {
    if (!interface || !interface->tree->root) return;
    
    printf("🌳 File Tree Structure:\n");
    printf("Root: %s (%s)\n", interface->tree->root->name, interface->tree->root->full_path);
    printf("Visible nodes: %d\n", interface->tree->num_visible_nodes);
    
    for (int i = 0; i < interface->tree->num_visible_nodes; i++) {
        FileTreeNode* node = interface->tree->visible_nodes[i];
        for (int d = 0; d < node->depth; d++) printf("  ");
        printf("%s %s (%s)\n", 
               node->is_directory ? "📁" : "📄",
               node->name,
               node->is_expanded ? "expanded" : "collapsed");
    }
}