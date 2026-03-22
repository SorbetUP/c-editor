// vault_manager.h - Gestion des vaults ElephantNotes
// Système de dossiers racine pour organiser les notes

#ifndef VAULT_MANAGER_H
#define VAULT_MANAGER_H

#include <stdbool.h>
#include <stddef.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Export macros
#ifdef _WIN32
#ifdef BUILDING_VAULT_DLL
#define VAULT_API __declspec(dllexport)
#else
#define VAULT_API __declspec(dllimport)
#endif
#else
#define VAULT_API __attribute__((visibility("default")))
#endif

// Codes de résultat
typedef enum {
    VAULT_SUCCESS = 0,
    VAULT_ERROR_NOT_FOUND = -1,
    VAULT_ERROR_PERMISSION = -2,
    VAULT_ERROR_OUT_OF_MEMORY = -3,
    VAULT_ERROR_IO = -4,
    VAULT_ERROR_INVALID_PATH = -5,
    VAULT_ERROR_EXISTS = -6,
    VAULT_ERROR_CORRUPTED = -7,
    VAULT_ERROR_LOCKED = -8
} VaultResult;

// Types de vault
typedef enum {
    VAULT_TYPE_LOCAL = 0,      // Vault local simple
    VAULT_TYPE_ENCRYPTED = 1,  // Vault chiffré
    VAULT_TYPE_CLOUD = 2,      // Vault synchronisé cloud
    VAULT_TYPE_SHARED = 3      // Vault partagé (équipe)
} VaultType;

// Configuration d'un vault
typedef struct {
    char* name;               // Nom du vault
    char* path;               // Chemin vers le vault
    char* description;        // Description
    VaultType type;           // Type de vault
    bool encrypted;           // Vault chiffré ?
    bool sync_enabled;        // Synchronisation activée ?
    char* sync_url;          // URL de synchronisation
    time_t created;          // Date de création
    time_t last_accessed;    // Dernier accès
    size_t total_notes;      // Nombre total de notes
    size_t total_size;       // Taille totale en octets
    char* icon_name;         // Nom de l'icône
    char* color;             // Couleur du vault (hex)
} VaultConfig;

// Informations sur un vault
typedef struct {
    VaultConfig config;
    bool is_locked;          // Vault verrouillé ?
    bool is_healthy;         // Vault en bon état ?
    char* last_error;        // Dernière erreur
    int note_count;          // Nombre de notes
    int folder_count;        // Nombre de dossiers
    size_t size_on_disk;     // Taille sur disque
} VaultInfo;

// Liste des vaults
typedef struct {
    VaultInfo* vaults;
    int count;
    int capacity;
    char* default_vault_path;  // Vault par défaut
} VaultRegistry;

// Structure pour la création de vault
typedef struct {
    char* name;
    char* path;
    char* description;
    VaultType type;
    bool encrypt;
    char* password;          // Si chiffrement
    bool create_sample_notes;
    char* template_name;     // Template à utiliser
} VaultCreationOptions;

// ========== API Principal ==========

// Initialisation et nettoyage
VAULT_API VaultResult vault_manager_init(void);
VAULT_API void vault_manager_cleanup(void);

// Gestion du registre des vaults
VAULT_API VaultResult vault_registry_load(VaultRegistry** registry);
VAULT_API VaultResult vault_registry_save(VaultRegistry* registry);
VAULT_API VaultResult vault_registry_add(VaultRegistry* registry, const char* vault_path);
VAULT_API VaultResult vault_registry_remove(VaultRegistry* registry, const char* vault_path);
VAULT_API VaultResult vault_registry_set_default(VaultRegistry* registry, const char* vault_path);
VAULT_API void vault_registry_free(VaultRegistry* registry);

// Création et configuration de vaults
VAULT_API VaultResult vault_create(const VaultCreationOptions* options, VaultInfo** info);
VAULT_API VaultResult vault_load(const char* vault_path, VaultInfo** info);
VAULT_API VaultResult vault_save_config(VaultInfo* info);
VAULT_API VaultResult vault_validate(const char* vault_path, bool* is_valid);
VAULT_API VaultResult vault_repair(const char* vault_path);

// Opérations sur les vaults
VAULT_API VaultResult vault_open(const char* vault_path, const char* password);
VAULT_API VaultResult vault_close(const char* vault_path);
VAULT_API VaultResult vault_lock(const char* vault_path);
VAULT_API VaultResult vault_unlock(const char* vault_path, const char* password);
VAULT_API VaultResult vault_change_password(const char* vault_path, const char* old_pass, const char* new_pass);

// Gestion des fichiers dans le vault
VAULT_API VaultResult vault_create_note(const char* vault_path, const char* note_name, char** note_path);
VAULT_API VaultResult vault_create_folder(const char* vault_path, const char* folder_path);
VAULT_API VaultResult vault_list_notes(const char* vault_path, char*** note_paths, int* count);
VAULT_API VaultResult vault_list_folders(const char* vault_path, char*** folder_paths, int* count);
VAULT_API VaultResult vault_move_note(const char* vault_path, const char* from, const char* to);
VAULT_API VaultResult vault_delete_note(const char* vault_path, const char* note_path);

// Recherche dans le vault
VAULT_API VaultResult vault_search_content(const char* vault_path, const char* query, 
                                          char*** results, int* count);
VAULT_API VaultResult vault_search_titles(const char* vault_path, const char* query, 
                                         char*** results, int* count);
VAULT_API VaultResult vault_get_recent_notes(const char* vault_path, char*** results, int* count);

// Statistiques et informations
VAULT_API VaultResult vault_get_stats(const char* vault_path, VaultInfo* info);
VAULT_API VaultResult vault_update_stats(const char* vault_path);
VAULT_API VaultResult vault_get_note_count(const char* vault_path, int* count);
VAULT_API VaultResult vault_get_total_size(const char* vault_path, size_t* size);

// Utilitaires de chemin
VAULT_API bool vault_is_valid_path(const char* path);
VAULT_API bool vault_is_note_path(const char* vault_path, const char* file_path);
VAULT_API char* vault_get_relative_path(const char* vault_path, const char* file_path);
VAULT_API char* vault_get_absolute_path(const char* vault_path, const char* relative_path);
VAULT_API char* vault_normalize_name(const char* name);

// Templates et échantillons
VAULT_API VaultResult vault_create_sample_structure(const char* vault_path);
VAULT_API VaultResult vault_apply_template(const char* vault_path, const char* template_name);
VAULT_API VaultResult vault_list_templates(char*** template_names, int* count);

// Synchronisation (si supportée)
VAULT_API VaultResult vault_sync_start(const char* vault_path);
VAULT_API VaultResult vault_sync_stop(const char* vault_path);
VAULT_API VaultResult vault_sync_status(const char* vault_path, bool* is_syncing, char** status);

// Import/Export
VAULT_API VaultResult vault_export_archive(const char* vault_path, const char* archive_path);
VAULT_API VaultResult vault_import_archive(const char* archive_path, const char* target_vault_path);
VAULT_API VaultResult vault_export_notes(const char* vault_path, const char* format, const char* output_path);

// Détection du premier lancement
VAULT_API bool vault_is_first_launch(void);
VAULT_API VaultResult vault_mark_first_launch_complete(void);
VAULT_API VaultResult vault_get_default_vault_location(char** path);
VAULT_API VaultResult vault_suggest_vault_name(const char* base_path, char** suggested_name);

// Gestion des erreurs
VAULT_API const char* vault_get_error_message(VaultResult result);
VAULT_API VaultResult vault_get_last_error(void);
VAULT_API void vault_clear_last_error(void);

// Nettoyage mémoire
VAULT_API void vault_free_info(VaultInfo* info);
VAULT_API void vault_free_string(char* str);
VAULT_API void vault_free_string_array(char** array, int count);

// Configuration globale
typedef struct {
    char* default_vault_location;   // Emplacement par défaut pour nouveaux vaults
    bool auto_create_folders;       // Créer automatiquement les dossiers
    bool auto_backup;               // Sauvegarde automatique
    int max_recent_vaults;          // Nombre max de vaults récents
    bool enable_encryption;         // Chiffrement disponible
    char* preferred_editor;         // Éditeur préféré
} VaultManagerConfig;

VAULT_API void vault_manager_set_config(const VaultManagerConfig* config);
VAULT_API VaultManagerConfig vault_manager_get_config(void);

#ifdef __cplusplus
}
#endif

#endif // VAULT_MANAGER_H