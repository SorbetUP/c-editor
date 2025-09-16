// professional_file_manager.h - Enterprise-grade file management system
// Advanced version control, session management, and collaboration features

#ifndef PROFESSIONAL_FILE_MANAGER_H
#define PROFESSIONAL_FILE_MANAGER_H

#include "file_manager.h"
#include <stdbool.h>
#include <stdint.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Version control system
typedef struct {
    uint32_t version_id;
    char* timestamp;
    char* author;
    char* comment;
    char* content_hash;
    size_t content_size;
    char* diff_from_previous;
} FileVersion;

typedef struct {
    FileVersion* versions;
    int count;
    int capacity;
    char* file_path;
} VersionHistory;

// Session management
typedef struct {
    char* file_path;
    uint32_t cursor_position;
    uint32_t selection_start;
    uint32_t selection_end;
    int scroll_position;
    char* last_search;
    time_t last_accessed;
    bool is_modified;
    char* auto_save_path;
} FileSession;

typedef struct {
    FileSession* sessions;
    int count;
    int capacity;
    char* workspace_path;
    time_t last_saved;
} WorkspaceSession;

// Conflict resolution
typedef enum {
    CONFLICT_NONE = 0,
    CONFLICT_EXTERNAL_CHANGE = 1,
    CONFLICT_CONCURRENT_EDIT = 2,
    CONFLICT_PERMISSION_DENIED = 3,
    CONFLICT_FILE_MOVED = 4,
    CONFLICT_FILE_DELETED = 5
} ConflictType;

typedef struct {
    ConflictType type;
    char* file_path;
    char* local_content;
    char* remote_content;
    time_t local_timestamp;
    time_t remote_timestamp;
    char* resolution_strategy;
} FileConflict;

// File monitoring and synchronization
typedef struct {
    char* file_path;
    char* content_hash;
    time_t last_modified;
    size_t file_size;
    bool is_locked;
    char* locked_by;
} FileState;

typedef struct {
    FileState* files;
    int count;
    int capacity;
} FileMonitor;

// Backup strategies
typedef enum {
    BACKUP_NONE = 0,
    BACKUP_SIMPLE = 1,          // .bak files
    BACKUP_VERSIONED = 2,       // .bak.1, .bak.2, etc.
    BACKUP_TIMESTAMPED = 3,     // .2024-01-01_12-30-45.bak
    BACKUP_GIT_LIKE = 4,        // Full version control
    BACKUP_INCREMENTAL = 5      // Only changes since last backup
} BackupStrategy;

typedef struct {
    BackupStrategy strategy;
    int max_versions;
    int cleanup_after_days;
    bool compress_backups;
    char* backup_directory;
    bool remote_backup;
    char* remote_endpoint;
} BackupConfig;

// Professional file operations
typedef struct {
    bool auto_save_enabled;
    int auto_save_interval_ms;  // Auto-save interval in milliseconds
    bool version_control_enabled;
    bool conflict_detection_enabled;
    bool file_monitoring_enabled;
    bool session_recovery_enabled;
    char* workspace_config_path;
    BackupConfig backup_config;
    int max_file_history;
    bool enable_file_locking;
    char* author_name;
    char* author_email;
} ProfessionalConfig;

// Version control operations
FILE_API FileResult version_create_history(const char* file_path, VersionHistory** history);
FILE_API FileResult version_add_version(VersionHistory* history, const char* content, 
                                       size_t size, const char* author, const char* comment);
FILE_API FileResult version_get_version(VersionHistory* history, uint32_t version_id, 
                                       FileContent** content);
FILE_API FileResult version_list_versions(VersionHistory* history, FileVersion** versions, int* count);
FILE_API FileResult version_revert_to(const char* file_path, uint32_t version_id);
FILE_API FileResult version_compare_versions(VersionHistory* history, uint32_t v1, uint32_t v2, 
                                            char** diff);
FILE_API FileResult version_merge_versions(VersionHistory* history, uint32_t base, 
                                         uint32_t v1, uint32_t v2, char** merged_content);
FILE_API void version_free_history(VersionHistory* history);

// Session management
FILE_API WorkspaceSession* session_create_workspace(const char* workspace_path);
FILE_API FileResult session_save_workspace(WorkspaceSession* workspace);
FILE_API FileResult session_load_workspace(const char* workspace_path, WorkspaceSession** workspace);
FILE_API FileResult session_add_file(WorkspaceSession* workspace, const char* file_path);
FILE_API FileResult session_remove_file(WorkspaceSession* workspace, const char* file_path);
FILE_API FileResult session_update_cursor(WorkspaceSession* workspace, const char* file_path, 
                                         uint32_t position, uint32_t sel_start, uint32_t sel_end);
FILE_API FileResult session_set_scroll_position(WorkspaceSession* workspace, const char* file_path, 
                                               int scroll_pos);
FILE_API FileResult session_get_file_state(WorkspaceSession* workspace, const char* file_path, 
                                          FileSession** session);
FILE_API void session_free_workspace(WorkspaceSession* workspace);

// Auto-save and recovery
FILE_API FileResult auto_save_start(const char* file_path, int interval_ms);
FILE_API FileResult auto_save_stop(const char* file_path);
FILE_API FileResult auto_save_save_now(const char* file_path, const char* content, size_t size);
FILE_API FileResult auto_save_recover(const char* file_path, FileContent** content);
FILE_API FileResult auto_save_cleanup(const char* file_path);
FILE_API FileResult auto_save_list_recoverable(char*** file_paths, int* count);

// Conflict detection and resolution
FILE_API FileResult conflict_check_file(const char* file_path, FileConflict** conflict);
FILE_API FileResult conflict_resolve_manual(FileConflict* conflict, const char* resolved_content);
FILE_API FileResult conflict_resolve_auto(FileConflict* conflict, const char* strategy); // "local", "remote", "merge"
FILE_API FileResult conflict_create_merge_file(FileConflict* conflict, char** merge_path);
FILE_API void conflict_free(FileConflict* conflict);

// File monitoring
FILE_API FileMonitor* monitor_create(void);
FILE_API FileResult monitor_add_file(FileMonitor* monitor, const char* file_path);
FILE_API FileResult monitor_remove_file(FileMonitor* monitor, const char* file_path);
FILE_API FileResult monitor_check_changes(FileMonitor* monitor, FileConflict*** conflicts, int* count);
FILE_API FileResult monitor_update_state(FileMonitor* monitor, const char* file_path);
FILE_API void monitor_free(FileMonitor* monitor);

// File locking (for collaboration)
FILE_API FileResult lock_acquire(const char* file_path, const char* user_id);
FILE_API FileResult lock_release(const char* file_path, const char* user_id);
FILE_API FileResult lock_check_status(const char* file_path, bool* is_locked, char** locked_by);
FILE_API FileResult lock_force_release(const char* file_path, const char* admin_user);

// Advanced backup operations
FILE_API FileResult backup_create_strategy(const char* file_path, BackupStrategy strategy);
FILE_API FileResult backup_create_incremental(const char* file_path, const char* content, size_t size);
FILE_API FileResult backup_list_all(const char* file_path, FileVersion** backups, int* count);
FILE_API FileResult backup_restore_by_timestamp(const char* file_path, time_t timestamp);
FILE_API FileResult backup_cleanup_old(const char* file_path, int keep_count, int max_age_days);
FILE_API FileResult backup_compress_archive(const char* file_path, const char* archive_path);

// Statistics and analytics
typedef struct {
    int total_files_managed;
    int total_versions_created;
    int total_auto_saves;
    int conflicts_detected;
    int conflicts_resolved;
    size_t total_storage_used;
    time_t last_activity;
    int active_sessions;
} FileManagerStats;

FILE_API FileResult stats_get_current(FileManagerStats* stats);
FILE_API FileResult stats_reset(void);
FILE_API FileResult stats_export_report(const char* output_path);

// Workspace templates
FILE_API FileResult template_create_workspace(const char* template_name, const char* workspace_path);
FILE_API FileResult template_save_workspace_as(const char* workspace_path, const char* template_name);
FILE_API FileResult template_list_available(char*** template_names, int* count);

// Professional configuration
FILE_API void professional_set_config(const ProfessionalConfig* config);
FILE_API ProfessionalConfig professional_get_config(void);
FILE_API FileResult professional_init(void);
FILE_API void professional_cleanup(void);

// Import/Export functionality
FILE_API FileResult export_workspace_archive(const char* workspace_path, const char* archive_path);
FILE_API FileResult import_workspace_archive(const char* archive_path, const char* target_path);
FILE_API FileResult export_version_history(const char* file_path, const char* export_path);
FILE_API FileResult import_version_history(const char* file_path, const char* import_path);

// Security features
FILE_API FileResult security_encrypt_file(const char* file_path, const char* password);
FILE_API FileResult security_decrypt_file(const char* file_path, const char* password);
FILE_API FileResult security_set_permissions(const char* file_path, int permissions);
FILE_API FileResult security_audit_access(const char* file_path, char** audit_log);

// Cloud integration hooks
typedef struct {
    char* provider_name;     // "dropbox", "google_drive", "onedrive", etc.
    char* access_token;
    char* refresh_token;
    char* remote_path;
    bool auto_sync;
    int sync_interval_minutes;
} CloudConfig;

FILE_API FileResult cloud_configure(const CloudConfig* config);
FILE_API FileResult cloud_sync_file(const char* local_path);
FILE_API FileResult cloud_sync_workspace(const char* workspace_path);
FILE_API FileResult cloud_check_sync_status(const char* file_path, bool* is_synced, time_t* last_sync);

#ifdef __cplusplus
}
#endif

#endif // PROFESSIONAL_FILE_MANAGER_H