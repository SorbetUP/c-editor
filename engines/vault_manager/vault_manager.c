// vault_manager.c - Implémentation du gestionnaire de vaults
#include "vault_manager.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>
#include <dirent.h>
#include <json-c/json.h>

// Configuration globale
static VaultManagerConfig g_config = {0};
static bool g_initialized = false;
static VaultResult g_last_error = VAULT_SUCCESS;
static char g_error_message[512] = {0};

// Constantes
#define VAULT_CONFIG_FILE ".elephantnotes_vault"
#define VAULT_REGISTRY_FILE "vault_registry.json"
#define APP_CONFIG_DIR "Library/Application Support/ElephantNotes"

// Utilitaires internes
static void set_error(VaultResult result, const char* message) {
    g_last_error = result;
    if (message) {
        strncpy(g_error_message, message, sizeof(g_error_message) - 1);
        g_error_message[sizeof(g_error_message) - 1] = '\0';
    }
}

static VaultResult create_directory_recursive(const char* path);

static char* get_app_config_dir(void) {
    const char* home = getenv("HOME");
    if (!home) return NULL;
    
    char* config_dir = malloc(strlen(home) + strlen(APP_CONFIG_DIR) + 2);
    if (!config_dir) return NULL;
    snprintf(config_dir, strlen(home) + strlen(APP_CONFIG_DIR) + 2,
             "%s/%s", home, APP_CONFIG_DIR);

    if (create_directory_recursive(config_dir) != VAULT_SUCCESS) {
        free(config_dir);
        return NULL;
    }
    
    return config_dir;
}

static char* get_vault_registry_path(void) {
    char* config_dir = get_app_config_dir();
    if (!config_dir) return NULL;
    
    char* registry_path = malloc(strlen(config_dir) + strlen(VAULT_REGISTRY_FILE) + 2);
    if (!registry_path) {
        free(config_dir);
        return NULL;
    }
    snprintf(registry_path, strlen(config_dir) + strlen(VAULT_REGISTRY_FILE) + 2,
             "%s/%s", config_dir, VAULT_REGISTRY_FILE);
    
    free(config_dir);
    return registry_path;
}

static bool file_exists(const char* path) {
    struct stat st;
    return stat(path, &st) == 0;
}

static VaultResult create_directory_recursive(const char* path) {
    char* temp_path = strdup(path);
    if (!temp_path) return VAULT_ERROR_OUT_OF_MEMORY;
    
    // Créer récursivement les répertoires
    for (char* p = strchr(temp_path + 1, '/'); p; p = strchr(p + 1, '/')) {
        *p = '\0';
        if (mkdir(temp_path, 0755) != 0 && errno != EEXIST) {
            free(temp_path);
            return VAULT_ERROR_PERMISSION;
        }
        *p = '/';
    }
    
    // Créer le répertoire final
    if (mkdir(temp_path, 0755) != 0 && errno != EEXIST) {
        free(temp_path);
        return VAULT_ERROR_PERMISSION;
    }
    
    free(temp_path);
    return VAULT_SUCCESS;
}

// ========== API Principal ==========

VaultResult vault_manager_init(void) {
    if (g_initialized) return VAULT_SUCCESS;
    
    // Configuration par défaut
    char* home = getenv("HOME");
    if (home) {
        g_config.default_vault_location = malloc(strlen(home) + 20);
        if (!g_config.default_vault_location) return VAULT_ERROR_OUT_OF_MEMORY;
        snprintf(g_config.default_vault_location, strlen(home) + 20,
                 "%s/Documents/Notes", home);
    }
    
    g_config.auto_create_folders = true;
    g_config.auto_backup = true;
    g_config.max_recent_vaults = 10;
    g_config.enable_encryption = false;  // Pour plus tard
    g_config.preferred_editor = strdup("ElephantNotes");
    if (!g_config.preferred_editor) {
        free(g_config.default_vault_location);
        g_config.default_vault_location = NULL;
        return VAULT_ERROR_OUT_OF_MEMORY;
    }
    
    g_initialized = true;
    return VAULT_SUCCESS;
}

void vault_manager_cleanup(void) {
    if (!g_initialized) return;
    
    free(g_config.default_vault_location);
    free(g_config.preferred_editor);
    memset(&g_config, 0, sizeof(g_config));
    
    g_initialized = false;
}

bool vault_is_first_launch(void) {
    char* registry_path = get_vault_registry_path();
    if (!registry_path) return true;
    
    bool first_launch = !file_exists(registry_path);
    free(registry_path);
    
    return first_launch;
}

VaultResult vault_mark_first_launch_complete(void) {
    // Créer un registre vide pour marquer que ce n'est plus le premier lancement
    VaultRegistry* registry = calloc(1, sizeof(VaultRegistry));
    if (!registry) return VAULT_ERROR_OUT_OF_MEMORY;
    
    VaultResult result = vault_registry_save(registry);
    vault_registry_free(registry);
    
    return result;
}

VaultResult vault_get_default_vault_location(char** path) {
    if (!path) return VAULT_ERROR_INVALID_PATH;
    
    if (!g_initialized) vault_manager_init();
    
    *path = strdup(g_config.default_vault_location);
    if (!*path) return VAULT_ERROR_OUT_OF_MEMORY;
    return VAULT_SUCCESS;
}

VaultResult vault_suggest_vault_name(const char* base_path, char** suggested_name) {
    if (!base_path || !suggested_name) return VAULT_ERROR_INVALID_PATH;
    
    char* base_name = "Mon Vault";
    char test_path[1024];
    int counter = 1;
    
    // Essayer "Mon Vault", puis "Mon Vault 2", etc.
    while (counter < 100) {
        if (counter == 1) {
            snprintf(test_path, sizeof(test_path), "%s/%s", base_path, base_name);
        } else {
            snprintf(test_path, sizeof(test_path), "%s/%s %d", base_path, base_name, counter);
        }
        
        if (!file_exists(test_path)) {
            if (counter == 1) {
                *suggested_name = strdup(base_name);
            } else {
                *suggested_name = malloc(strlen(base_name) + 20);
                if (!*suggested_name) return VAULT_ERROR_OUT_OF_MEMORY;
                sprintf(*suggested_name, "%s %d", base_name, counter);
            }
            if (!*suggested_name) return VAULT_ERROR_OUT_OF_MEMORY;
            return VAULT_SUCCESS;
        }
        counter++;
    }
    
    *suggested_name = strdup("Nouveau Vault");
    if (!*suggested_name) return VAULT_ERROR_OUT_OF_MEMORY;
    return VAULT_SUCCESS;
}

VaultResult vault_create(const VaultCreationOptions* options, VaultInfo** info) {
    if (!options || !options->name || !options->path) {
        set_error(VAULT_ERROR_INVALID_PATH, "Options invalides pour la création du vault");
        return VAULT_ERROR_INVALID_PATH;
    }
    
    // Créer le chemin complet
    char* full_path = malloc(strlen(options->path) + strlen(options->name) + 2);
    if (!full_path) return VAULT_ERROR_OUT_OF_MEMORY;
    sprintf(full_path, "%s/%s", options->path, options->name);
    
    // Vérifier que le vault n'existe pas déjà
    if (file_exists(full_path)) {
        free(full_path);
        set_error(VAULT_ERROR_EXISTS, "Un vault avec ce nom existe déjà");
        return VAULT_ERROR_EXISTS;
    }
    
    // Créer le répertoire
    VaultResult result = create_directory_recursive(full_path);
    if (result != VAULT_SUCCESS) {
        free(full_path);
        set_error(result, "Impossible de créer le répertoire du vault");
        return result;
    }
    
    // Créer la structure du vault
    char config_path[1024];
    sprintf(config_path, "%s/%s", full_path, VAULT_CONFIG_FILE);
    
    // Créer le fichier de configuration JSON
    json_object* config_json = json_object_new_object();
    json_object* name_obj = json_object_new_string(options->name);
    json_object* type_obj = json_object_new_int(options->type);
    json_object* description_obj = json_object_new_string(options->description ? options->description : "");
    json_object* created_obj = json_object_new_int64(time(NULL));
    json_object* version_obj = json_object_new_string("1.0");
    
    json_object_object_add(config_json, "name", name_obj);
    json_object_object_add(config_json, "type", type_obj);
    json_object_object_add(config_json, "description", description_obj);
    json_object_object_add(config_json, "created", created_obj);
    json_object_object_add(config_json, "version", version_obj);
    
    FILE* config_file = fopen(config_path, "w");
    if (!config_file) {
        json_object_put(config_json);
        free(full_path);
        set_error(VAULT_ERROR_IO, "Impossible de créer le fichier de configuration");
        return VAULT_ERROR_IO;
    }
    
    fprintf(config_file, "%s\n", json_object_to_json_string_ext(config_json, JSON_C_TO_STRING_PRETTY));
    fclose(config_file);
    json_object_put(config_json);
    
    // Créer la structure de dossiers de base
    char notes_path[1024];
    sprintf(notes_path, "%s/Notes", full_path);
    mkdir(notes_path, 0755);
    
    char attachments_path[1024];
    sprintf(attachments_path, "%s/Attachments", full_path);
    mkdir(attachments_path, 0755);
    
    char templates_path[1024];
    sprintf(templates_path, "%s/Templates", full_path);
    mkdir(templates_path, 0755);
    
    // Créer des notes d'exemple si demandé
    if (options->create_sample_notes) {
        vault_create_sample_structure(full_path);
    }
    
    // Charger les informations du vault créé
    result = vault_load(full_path, info);
    
    free(full_path);
    return result;
}

VaultResult vault_load(const char* vault_path, VaultInfo** info) {
    if (!vault_path || !info) return VAULT_ERROR_INVALID_PATH;
    
    // Vérifier que le vault existe
    char config_path[1024];
    sprintf(config_path, "%s/%s", vault_path, VAULT_CONFIG_FILE);
    
    if (!file_exists(config_path)) {
        set_error(VAULT_ERROR_NOT_FOUND, "Configuration du vault introuvable");
        return VAULT_ERROR_NOT_FOUND;
    }
    
    // Allouer la structure d'informations
    *info = calloc(1, sizeof(VaultInfo));
    if (!*info) return VAULT_ERROR_OUT_OF_MEMORY;
    
    // Charger la configuration depuis JSON
    FILE* config_file = fopen(config_path, "r");
    if (!config_file) {
        free(*info);
        *info = NULL;
        return VAULT_ERROR_IO;
    }
    
    fseek(config_file, 0, SEEK_END);
    long file_size = ftell(config_file);
    fseek(config_file, 0, SEEK_SET);
    
    char* json_string = malloc(file_size + 1);
    if (!json_string) {
        fclose(config_file);
        free(*info);
        *info = NULL;
        return VAULT_ERROR_OUT_OF_MEMORY;
    }
    fread(json_string, 1, file_size, config_file);
    json_string[file_size] = '\0';
    fclose(config_file);
    
    json_object* config_json = json_tokener_parse(json_string);
    free(json_string);
    
    if (!config_json) {
        free(*info);
        *info = NULL;
        set_error(VAULT_ERROR_CORRUPTED, "Configuration du vault corrompue");
        return VAULT_ERROR_CORRUPTED;
    }
    
    // Extraire les informations
    json_object* name_obj;
    if (json_object_object_get_ex(config_json, "name", &name_obj)) {
        (*info)->config.name = strdup(json_object_get_string(name_obj));
    }
    
    json_object* description_obj;
    if (json_object_object_get_ex(config_json, "description", &description_obj)) {
        (*info)->config.description = strdup(json_object_get_string(description_obj));
    }
    
    json_object* type_obj;
    if (json_object_object_get_ex(config_json, "type", &type_obj)) {
        (*info)->config.type = json_object_get_int(type_obj);
    }
    
    json_object* created_obj;
    if (json_object_object_get_ex(config_json, "created", &created_obj)) {
        (*info)->config.created = json_object_get_int64(created_obj);
    }
    
    (*info)->config.path = strdup(vault_path);
    (*info)->is_locked = false;
    (*info)->is_healthy = true;
    (*info)->last_error = NULL;
    
    // Calculer les statistiques
    vault_update_stats(vault_path);
    
    json_object_put(config_json);
    return VAULT_SUCCESS;
}

VaultResult vault_create_sample_structure(const char* vault_path) {
    if (!vault_path) return VAULT_ERROR_INVALID_PATH;
    
    // Créer quelques notes d'exemple
    char note_path[1024];
    
    // Note de bienvenue
    sprintf(note_path, "%s/Notes/Bienvenue.md", vault_path);
    FILE* welcome_file = fopen(note_path, "w");
    if (welcome_file) {
        fprintf(welcome_file, "# Bienvenue dans votre vault ElephantNotes !\n\n");
        fprintf(welcome_file, "## 🎉 Félicitations !\n\n");
        fprintf(welcome_file, "Vous avez créé votre premier **vault** ElephantNotes. Un vault est un espace de travail organisé pour vos notes.\n\n");
        fprintf(welcome_file, "## 📁 Structure de votre vault\n\n");
        fprintf(welcome_file, "- **Notes/** - Vos notes markdown\n");
        fprintf(welcome_file, "- **Attachments/** - Fichiers joints (images, PDFs, etc.)\n");
        fprintf(welcome_file, "- **Templates/** - Modèles de notes réutilisables\n\n");
        fprintf(welcome_file, "## ✨ Fonctionnalités professionnelles\n\n");
        fprintf(welcome_file, "- 🔄 **Auto-sauvegarde** toutes les 3 secondes\n");
        fprintf(welcome_file, "- 📚 **Contrôle de version** automatique\n");
        fprintf(welcome_file, "- 🔍 **Détection de conflits** en temps réel\n");
        fprintf(welcome_file, "- 💾 **Récupération de session** après crash\n");
        fprintf(welcome_file, "- ⌘+K **Instantanés manuels** avec commentaires\n\n");
        fprintf(welcome_file, "## 🚀 Commencer\n\n");
        fprintf(welcome_file, "1. Créez de nouvelles notes avec **⌘+N**\n");
        fprintf(welcome_file, "2. Organisez vos notes dans des dossiers\n");
        fprintf(welcome_file, "3. Utilisez les **raccourcis professionnels** :\n");
        fprintf(welcome_file, "   - ⌘+K : Créer un instantané\n");
        fprintf(welcome_file, "   - ⌘+I : Statistiques du vault\n");
        fprintf(welcome_file, "   - ⌘+H : Historique des versions\n\n");
        fprintf(welcome_file, "Bon travail ! 📝\n");
        fclose(welcome_file);
    }
    
    // Note Dashboard spéciale
    sprintf(note_path, "%s/Notes/dashboard.md", vault_path);
    FILE* dashboard_file = fopen(note_path, "w");
    if (dashboard_file) {
        fprintf(dashboard_file, "# 🏠 Dashboard ElephantNotes\n\n");
        fprintf(dashboard_file, "## 📊 Vue d'ensemble du vault\n\n");
        fprintf(dashboard_file, "Bienvenue dans votre espace de travail personnel !\n\n");
        fprintf(dashboard_file, "### 📝 Statistiques rapides\n");
        fprintf(dashboard_file, "- **Vault actuel** : %s\n", vault_path);
        fprintf(dashboard_file, "- **Notes créées** : En cours de calcul...\n");
        fprintf(dashboard_file, "- **Dernière modification** : Maintenant\n\n");
        fprintf(dashboard_file, "### 🚀 Actions rapides\n\n");
        fprintf(dashboard_file, "- [📄 Nouvelle note](command://new-note) - Créer une nouvelle note\n");
        fprintf(dashboard_file, "- [📂 Parcourir notes](command://browse-notes) - Explorer vos notes\n");
        fprintf(dashboard_file, "- [🔍 Rechercher](command://search) - Rechercher dans le vault\n");
        fprintf(dashboard_file, "- [⚙️ Paramètres](command://settings) - Configurer le vault\n\n");
        fprintf(dashboard_file, "### 📅 Aujourd'hui\n\n");
        fprintf(dashboard_file, "- [ ] Organiser mes notes\n");
        fprintf(dashboard_file, "- [ ] Créer un nouveau projet\n");
        fprintf(dashboard_file, "- [ ] Réviser mes objectifs\n\n");
        fprintf(dashboard_file, "### 💡 Conseils\n\n");
        fprintf(dashboard_file, "- Utilisez les **raccourcis clavier** pour une productivité maximale\n");
        fprintf(dashboard_file, "- Organisez vos notes par **projets** ou **thèmes**\n");
        fprintf(dashboard_file, "- Profitez de l'**auto-sauvegarde** et du **contrôle de version**\n\n");
        fprintf(dashboard_file, "---\n");
        fprintf(dashboard_file, "*Cette note est votre tableau de bord personnel. Modifiez-la selon vos besoins !*\n");
        fclose(dashboard_file);
    }
    
    // Note Settings spéciale  
    sprintf(note_path, "%s/Notes/settings.md", vault_path);
    FILE* settings_file = fopen(note_path, "w");
    if (settings_file) {
        fprintf(settings_file, "# ⚙️ Paramètres ElephantNotes\n\n");
        fprintf(settings_file, "## 🎛️ Configuration du vault\n\n");
        fprintf(settings_file, "### 📁 Informations générales\n");
        fprintf(settings_file, "- **Nom du vault** : Configuration dans les propriétés\n");
        fprintf(settings_file, "- **Emplacement** : %s\n", vault_path);
        fprintf(settings_file, "- **Type** : Vault local\n");
        fprintf(settings_file, "- **Chiffrement** : Désactivé\n\n");
        fprintf(settings_file, "### 💾 Fonctionnalités professionnelles\n\n");
        fprintf(settings_file, "#### Auto-sauvegarde\n");
        fprintf(settings_file, "- ✅ **Activée** - Intervalle : 3 secondes\n");
        fprintf(settings_file, "- 📁 Emplacement : `.autosave/`\n\n");
        fprintf(settings_file, "#### Contrôle de version\n");
        fprintf(settings_file, "- ✅ **Activé** - Versions automatiques à chaque sauvegarde\n");
        fprintf(settings_file, "- 📚 Instantanés manuels avec ⌘+K\n");
        fprintf(settings_file, "- 🗄️ Maximum 20 versions par fichier\n\n");
        fprintf(settings_file, "#### Détection de conflits\n");
        fprintf(settings_file, "- ✅ **Activée** - Vérification toutes les 10 secondes\n");
        fprintf(settings_file, "- 🔍 Surveillance des modifications externes\n\n");
        fprintf(settings_file, "#### Récupération de session\n");
        fprintf(settings_file, "- ✅ **Activée** - Restauration automatique après crash\n");
        fprintf(settings_file, "- 💾 Sauvegarde de l'état : position curseur, fichiers ouverts\n\n");
        fprintf(settings_file, "### 🎨 Interface utilisateur\n\n");
        fprintf(settings_file, "- **Thème** : Clair (par défaut)\n");
        fprintf(settings_file, "- **Police** : Monaco 14pt\n");
        fprintf(settings_file, "- **Barre latérale** : 60px, icônes Bootstrap\n");
        fprintf(settings_file, "- **Éditeur** : Marge gauche 70px pour la superposition\n\n");
        fprintf(settings_file, "### 🔧 Actions de maintenance\n\n");
        fprintf(settings_file, "- [🧹 Nettoyer versions anciennes](command://clean-versions)\n");
        fprintf(settings_file, "- [📊 Vérifier intégrité vault](command://verify-vault)\n");
        fprintf(settings_file, "- [📦 Exporter vault](command://export-vault)\n");
        fprintf(settings_file, "- [🔄 Synchroniser](command://sync-vault)\n\n");
        fprintf(settings_file, "### 📋 Gestion des vaults\n\n");
        fprintf(settings_file, "- [📁 Gestionnaire de vaults](command://vault-manager) - ⌘+V\n");
        fprintf(settings_file, "- [➕ Créer nouveau vault](command://new-vault) - ⌘+Shift+V\n");
        fprintf(settings_file, "- [🔄 Changer de vault actif](command://switch-vault)\n\n");
        fprintf(settings_file, "---\n");
        fprintf(settings_file, "*Modifiez cette note pour personnaliser vos paramètres et préférences.*\n");
        fclose(settings_file);
    }
    
    // Guide de démarrage rapide
    sprintf(note_path, "%s/Notes/Guide de démarrage.md", vault_path);
    FILE* guide_file = fopen(note_path, "w");
    if (guide_file) {
        fprintf(guide_file, "# Guide de démarrage rapide\n\n");
        fprintf(guide_file, "## Markdown de base\n\n");
        fprintf(guide_file, "### Formatage du texte\n");
        fprintf(guide_file, "- **Gras** avec `**texte**`\n");
        fprintf(guide_file, "- *Italique* avec `*texte*`\n");
        fprintf(guide_file, "- ==Surligné== avec `==texte==`\n\n");
        fprintf(guide_file, "### Listes\n");
        fprintf(guide_file, "- Élément 1\n");
        fprintf(guide_file, "- Élément 2\n");
        fprintf(guide_file, "  - Sous-élément\n\n");
        fprintf(guide_file, "### Liens et images\n");
        fprintf(guide_file, "- [Lien](https://example.com)\n");
        fprintf(guide_file, "- ![Image](image.png)\n\n");
        fprintf(guide_file, "## Organisation\n\n");
        fprintf(guide_file, "Organisez vos notes par :\n");
        fprintf(guide_file, "- 📂 **Projets** - Un dossier par projet\n");
        fprintf(guide_file, "- 📅 **Dates** - Notes quotidiennes/hebdomadaires\n");
        fprintf(guide_file, "- 🏷️ **Tags** - Utilisez #tag dans vos notes\n");
        fprintf(guide_file, "- 📚 **Catégories** - Personnel, Travail, Études\n\n");
        fclose(guide_file);
    }
    
    // Template de note quotidienne
    sprintf(note_path, "%s/Templates/Note quotidienne.md", vault_path);
    FILE* template_file = fopen(note_path, "w");
    if (template_file) {
        fprintf(template_file, "# Journal - {{date}}\n\n");
        fprintf(template_file, "## 🎯 Objectifs du jour\n");
        fprintf(template_file, "- [ ] \n");
        fprintf(template_file, "- [ ] \n");
        fprintf(template_file, "- [ ] \n\n");
        fprintf(template_file, "## 📝 Notes\n\n\n");
        fprintf(template_file, "## ✅ Accomplissements\n\n\n");
        fprintf(template_file, "## 🤔 Réflexions\n\n\n");
        fprintf(template_file, "## 📋 À faire demain\n");
        fprintf(template_file, "- [ ] \n");
        fprintf(template_file, "- [ ] \n");
        fclose(template_file);
    }
    
    return VAULT_SUCCESS;
}

VaultResult vault_registry_load(VaultRegistry** registry) {
    if (!registry) return VAULT_ERROR_INVALID_PATH;
    
    *registry = calloc(1, sizeof(VaultRegistry));
    if (!*registry) return VAULT_ERROR_OUT_OF_MEMORY;
    
    char* registry_path = get_vault_registry_path();
    if (!registry_path) {
        free(*registry);
        *registry = NULL;
        return VAULT_ERROR_OUT_OF_MEMORY;
    }
    
    if (!file_exists(registry_path)) {
        free(registry_path);
        return VAULT_SUCCESS; // Registre vide
    }
    
    FILE* file = fopen(registry_path, "r");
    free(registry_path);
    
    if (!file) return VAULT_ERROR_IO;
    
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    char* json_string = malloc(file_size + 1);
    if (!json_string) {
        fclose(file);
        vault_registry_free(*registry);
        *registry = NULL;
        return VAULT_ERROR_OUT_OF_MEMORY;
    }
    fread(json_string, 1, file_size, file);
    json_string[file_size] = '\0';
    fclose(file);
    
    json_object* root = json_tokener_parse(json_string);
    free(json_string);
    
    if (!root) return VAULT_ERROR_CORRUPTED;
    
    // Charger le vault par défaut
    json_object* default_obj;
    if (json_object_object_get_ex(root, "default_vault", &default_obj)) {
        (*registry)->default_vault_path = strdup(json_object_get_string(default_obj));
    }
    
    // Charger la liste des vaults
    json_object* vaults_array;
    if (json_object_object_get_ex(root, "vaults", &vaults_array)) {
        int array_len = json_object_array_length(vaults_array);
        if (array_len > 0) {
            (*registry)->vaults = malloc(sizeof(VaultInfo) * array_len);
            if (!(*registry)->vaults) {
                json_object_put(root);
                vault_registry_free(*registry);
                *registry = NULL;
                return VAULT_ERROR_OUT_OF_MEMORY;
            }
            (*registry)->capacity = array_len;
            
            for (int i = 0; i < array_len; i++) {
                json_object* vault_obj = json_object_array_get_idx(vaults_array, i);
                json_object* path_obj;
                
                if (json_object_object_get_ex(vault_obj, "path", &path_obj)) {
                    const char* vault_path = json_object_get_string(path_obj);
                    VaultInfo* info = NULL;
                    
                    if (vault_load(vault_path, &info) == VAULT_SUCCESS && info) {
                        (*registry)->vaults[(*registry)->count] = *info;
                        (*registry)->count++;
                        free(info); // On a copié la structure
                    }
                }
            }
        }
    }
    
    json_object_put(root);
    return VAULT_SUCCESS;
}

VaultResult vault_registry_save(VaultRegistry* registry) {
    if (!registry) return VAULT_ERROR_INVALID_PATH;

    char* registry_path = get_vault_registry_path();
    if (!registry_path) return VAULT_ERROR_OUT_OF_MEMORY;
    
    json_object* root = json_object_new_object();
    
    // Sauvegarder le vault par défaut
    if (registry->default_vault_path) {
        json_object* default_obj = json_object_new_string(registry->default_vault_path);
        json_object_object_add(root, "default_vault", default_obj);
    }
    
    // Sauvegarder la liste des vaults
    json_object* vaults_array = json_object_new_array();
    for (int i = 0; i < registry->count; i++) {
        json_object* vault_obj = json_object_new_object();
        json_object* path_obj = json_object_new_string(registry->vaults[i].config.path);
        json_object_object_add(vault_obj, "path", path_obj);
        json_object_array_add(vaults_array, vault_obj);
    }
    json_object_object_add(root, "vaults", vaults_array);
    
    FILE* file = fopen(registry_path, "w");
    free(registry_path);
    
    if (!file) {
        json_object_put(root);
        return VAULT_ERROR_IO;
    }
    
    const char* json_string = json_object_to_json_string_ext(root, JSON_C_TO_STRING_PRETTY);
    fprintf(file, "%s\n", json_string);
    fclose(file);
    json_object_put(root);
    return VAULT_SUCCESS;
}

VaultResult vault_registry_add(VaultRegistry* registry, const char* vault_path) {
    if (!registry || !vault_path) return VAULT_ERROR_INVALID_PATH;
    
    // Vérifier que le vault n'est pas déjà dans le registre
    for (int i = 0; i < registry->count; i++) {
        if (strcmp(registry->vaults[i].config.path, vault_path) == 0) {
            return VAULT_SUCCESS; // Déjà présent
        }
    }
    
    // Charger les informations du vault
    VaultInfo* info = NULL;
    VaultResult result = vault_load(vault_path, &info);
    if (result != VAULT_SUCCESS) return result;
    
    // Agrandir le tableau si nécessaire
    if (registry->count >= registry->capacity) {
        int new_capacity = registry->capacity ? registry->capacity * 2 : 10;
        VaultInfo* new_vaults =
            realloc(registry->vaults, sizeof(VaultInfo) * new_capacity);
        if (!new_vaults) {
            vault_free_info(info);
            free(info);
            return VAULT_ERROR_OUT_OF_MEMORY;
        }
        registry->vaults = new_vaults;
        registry->capacity = new_capacity;
    }
    
    // Ajouter le vault
    registry->vaults[registry->count] = *info;
    registry->count++;
    
    // Définir comme vault par défaut si c'est le premier
    if (registry->count == 1 && !registry->default_vault_path) {
        registry->default_vault_path = strdup(vault_path);
        if (!registry->default_vault_path) {
            registry->count--;
            return VAULT_ERROR_OUT_OF_MEMORY;
        }
    }
    
    free(info);
    return VAULT_SUCCESS;
}

VaultResult vault_registry_remove(VaultRegistry* registry, const char* vault_path) {
    if (!registry || !vault_path) return VAULT_ERROR_INVALID_PATH;
    
    // Trouver le vault à supprimer
    int index = -1;
    for (int i = 0; i < registry->count; i++) {
        if (strcmp(registry->vaults[i].config.path, vault_path) == 0) {
            index = i;
            break;
        }
    }
    
    if (index == -1) return VAULT_ERROR_NOT_FOUND;
    
    // Libérer la mémoire du vault
    vault_free_info(&registry->vaults[index]);
    
    // Décaler les éléments suivants
    for (int i = index; i < registry->count - 1; i++) {
        registry->vaults[i] = registry->vaults[i + 1];
    }
    
    registry->count--;
    
    // Si c'était le vault par défaut, le réinitialiser
    if (registry->default_vault_path && strcmp(registry->default_vault_path, vault_path) == 0) {
        free(registry->default_vault_path);
        registry->default_vault_path = NULL;
    }
    
    return VAULT_SUCCESS;
}

VaultResult vault_registry_set_default(VaultRegistry* registry, const char* vault_path) {
    if (!registry || !vault_path) return VAULT_ERROR_INVALID_PATH;
    
    free(registry->default_vault_path);
    registry->default_vault_path = strdup(vault_path);
    
    return VAULT_SUCCESS;
}

void vault_registry_free(VaultRegistry* registry) {
    if (!registry) return;
    
    for (int i = 0; i < registry->count; i++) {
        vault_free_info(&registry->vaults[i]);
    }
    
    free(registry->vaults);
    free(registry->default_vault_path);
    free(registry);
}

void vault_free_info(VaultInfo* info) {
    if (!info) return;
    
    free(info->config.name);
    free(info->config.path);
    free(info->config.description);
    free(info->config.sync_url);
    free(info->config.icon_name);
    free(info->config.color);
    free(info->last_error);
}

void vault_free_string(char* str) {
    free(str);
}

const char* vault_get_error_message(VaultResult result) {
    switch (result) {
        case VAULT_SUCCESS: return "Succès";
        case VAULT_ERROR_NOT_FOUND: return "Vault non trouvé";
        case VAULT_ERROR_PERMISSION: return "Permissions insuffisantes";
        case VAULT_ERROR_OUT_OF_MEMORY: return "Mémoire insuffisante";
        case VAULT_ERROR_IO: return "Erreur d'entrée/sortie";
        case VAULT_ERROR_INVALID_PATH: return "Chemin invalide";
        case VAULT_ERROR_EXISTS: return "Le vault existe déjà";
        case VAULT_ERROR_CORRUPTED: return "Vault corrompu";
        case VAULT_ERROR_LOCKED: return "Vault verrouillé";
        default: return "Erreur inconnue";
    }
}

VaultResult vault_get_last_error(void) {
    return g_last_error;
}

void vault_clear_last_error(void) {
    g_last_error = VAULT_SUCCESS;
    g_error_message[0] = '\0';
}

// Placeholder pour les fonctions non encore implémentées
VaultResult vault_update_stats(const char* vault_path) {
    // TODO: Implémenter le calcul des statistiques
    (void)vault_path;
    return VAULT_SUCCESS;
}
