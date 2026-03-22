// search_interface.h - Interface de recherche avec barre de recherche et arborescence
// Compatible avec advanced_search.h pour recherche sémantique ultra-rapide

#ifndef SEARCH_INTERFACE_H
#define SEARCH_INTERFACE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Structure pour un nœud de l'arborescence
typedef struct FileTreeNode {
    char* name;                     // Nom du fichier/dossier
    char* full_path;               // Chemin complet
    bool is_directory;             // Est-ce un dossier ?
    bool is_expanded;              // Est-ce que le dossier est déplié ?
    bool is_visible;               // Est-ce visible dans le filtre ?
    int depth;                     // Profondeur dans l'arbre
    
    // Navigation
    struct FileTreeNode* parent;   // Parent dans l'arbre
    struct FileTreeNode** children; // Enfants
    int num_children;              // Nombre d'enfants
    int children_capacity;         // Capacité du tableau enfants
    
    // Métadonnées
    time_t last_modified;
    int64_t file_size;
    
    // UI state
    bool is_selected;              // Est-ce sélectionné ?
    bool is_highlighted;           // Est-ce mis en surbrillance ?
} FileTreeNode;

// Structure pour l'arborescence complète
typedef struct {
    FileTreeNode* root;            // Nœud racine
    FileTreeNode** visible_nodes;  // Nœuds visibles (cache pour UI)
    int num_visible_nodes;         // Nombre de nœuds visibles
    int visible_capacity;          // Capacité du cache
    
    // État de l'interface
    int scroll_position;           // Position de défilement
    int selected_index;            // Index du nœud sélectionné
    
    // Configuration
    bool show_hidden_files;        // Afficher les fichiers cachés
    bool show_file_sizes;          // Afficher les tailles de fichiers
    char* file_filter;             // Filtre par extension (ex: ".md,.txt")
} FileTree;

// Configuration de l'interface de recherche
typedef struct {
    // Dimensions
    int width;                     // Largeur de l'interface
    int height;                    // Hauteur de l'interface
    int search_bar_height;         // Hauteur de la barre de recherche
    
    // Comportement
    bool auto_focus_search;        // Focus automatique sur la barre
    bool live_search;              // Recherche en temps réel
    int search_delay_ms;           // Délai avant recherche (debounce)
    
    // Style
    bool show_search_suggestions;  // Afficher les suggestions
    int max_suggestions;           // Nombre max de suggestions
    bool highlight_matches;        // Surligner les correspondances
    
    // Filtres
    bool show_hidden_files;        // Afficher les fichiers cachés
    char* file_filter;             // Filtre par extension (ex: ".md,.txt")
    
    // Callbacks
    void (*on_file_selected)(const char* file_path, void* user_data);
    void (*on_search_changed)(const char* query, void* user_data);
    void (*on_tree_expanded)(const char* directory_path, void* user_data);
} SearchInterfaceConfig;

// État de la barre de recherche
typedef struct {
    char* query;                   // Requête actuelle
    char* placeholder;             // Texte placeholder
    bool has_focus;                // La barre a-t-elle le focus ?
    int cursor_position;           // Position du curseur
    int selection_start;           // Début de la sélection
    int selection_end;             // Fin de la sélection
    
    // Suggestions
    char** suggestions;            // Liste des suggestions
    int num_suggestions;           // Nombre de suggestions
    int selected_suggestion;       // Suggestion sélectionnée
    bool show_suggestions;         // Afficher les suggestions ?
    
    // Historique
    char** history;                // Historique des recherches
    int num_history;               // Nombre d'entrées dans l'historique
    int history_position;          // Position dans l'historique
} SearchBar;

// Contexte principal de l'interface
typedef struct SearchInterface SearchInterface;

// ========== API Principal ==========

// Initialisation
SearchInterface* search_interface_create(const SearchInterfaceConfig* config);
void search_interface_destroy(SearchInterface* interface);
bool search_interface_set_config(SearchInterface* interface, const SearchInterfaceConfig* config);
SearchInterfaceConfig search_interface_get_default_config(void);

// Gestion de l'arborescence
bool search_interface_set_root_directory(SearchInterface* interface, const char* directory_path);
bool search_interface_refresh_tree(SearchInterface* interface);
bool search_interface_expand_node(SearchInterface* interface, const char* path);
bool search_interface_collapse_node(SearchInterface* interface, const char* path);
bool search_interface_toggle_node(SearchInterface* interface, const char* path);

// Navigation dans l'arborescence
bool search_interface_select_node(SearchInterface* interface, int index);
bool search_interface_select_node_by_path(SearchInterface* interface, const char* path);
FileTreeNode* search_interface_get_selected_node(SearchInterface* interface);
FileTreeNode* search_interface_get_node_at_index(SearchInterface* interface, int index);

// Barre de recherche
bool search_interface_set_search_query(SearchInterface* interface, const char* query);
const char* search_interface_get_search_query(SearchInterface* interface);
bool search_interface_clear_search(SearchInterface* interface);
bool search_interface_focus_search(SearchInterface* interface);

// Gestion des suggestions
char** search_interface_get_suggestions(SearchInterface* interface, int* num_suggestions);
bool search_interface_select_suggestion(SearchInterface* interface, int index);
bool search_interface_apply_suggestion(SearchInterface* interface);

// Filtrage et recherche
bool search_interface_apply_filter(SearchInterface* interface, const char* filter);
bool search_interface_search_in_tree(SearchInterface* interface, const char* query);
bool search_interface_highlight_matches(SearchInterface* interface, const char* query);
void search_interface_clear_highlights(SearchInterface* interface);

// Événements de navigation
bool search_interface_handle_key_down(SearchInterface* interface, int key_code);
bool search_interface_handle_key_up(SearchInterface* interface, int key_code);
bool search_interface_handle_mouse_click(SearchInterface* interface, int x, int y);
bool search_interface_handle_scroll(SearchInterface* interface, int delta);

// Interface avec advanced_search
bool search_interface_connect_search_engine(SearchInterface* interface, void* search_engine);
bool search_interface_perform_semantic_search(SearchInterface* interface, const char* query);
bool search_interface_perform_fuzzy_search(SearchInterface* interface, const char* query);

// Rendering (pour intégration UI)
typedef struct {
    int x, y, width, height;       // Rectangle de rendu
    bool is_visible;               // Visible ?
    bool needs_redraw;             // Besoin de redessiner ?
    
    // Style
    uint32_t background_color;
    uint32_t text_color;
    uint32_t highlight_color;
    uint32_t border_color;
    
    // Font
    char* font_name;
    int font_size;
} RenderInfo;

RenderInfo search_interface_get_search_bar_render_info(SearchInterface* interface);
RenderInfo search_interface_get_tree_render_info(SearchInterface* interface);
FileTreeNode** search_interface_get_visible_nodes(SearchInterface* interface, int* count);

// Utilitaires
void search_interface_set_user_data(SearchInterface* interface, void* user_data);
void* search_interface_get_user_data(SearchInterface* interface);
bool search_interface_save_state(SearchInterface* interface, const char* state_file);
bool search_interface_load_state(SearchInterface* interface, const char* state_file);

// Debug et développement
void search_interface_print_tree(SearchInterface* interface);
void search_interface_validate_tree(SearchInterface* interface);
char* search_interface_get_debug_info(SearchInterface* interface);

// Gestion des erreurs
typedef enum {
    SEARCH_INTERFACE_SUCCESS = 0,
    SEARCH_INTERFACE_ERROR_INVALID_PARAM = -1,
    SEARCH_INTERFACE_ERROR_OUT_OF_MEMORY = -2,
    SEARCH_INTERFACE_ERROR_IO_ERROR = -3,
    SEARCH_INTERFACE_ERROR_INVALID_PATH = -4
} SearchInterfaceResult;

SearchInterfaceResult search_interface_get_last_error(SearchInterface* interface);
const char* search_interface_get_error_message(SearchInterfaceResult error);

#ifdef __cplusplus
}
#endif

#endif // SEARCH_INTERFACE_H