#ifndef BACKUP_ENGINE_H
#define BACKUP_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <time.h>

// Backup strategies
typedef enum {
    BACKUP_STRATEGY_IMMEDIATE,  // Save immediately on change
    BACKUP_STRATEGY_TIMED,      // Save every N seconds
    BACKUP_STRATEGY_ON_IDLE,    // Save when user is idle
    BACKUP_STRATEGY_MANUAL,     // Save only when requested
    BACKUP_STRATEGY_SMART       // Adaptive based on content changes
} backup_strategy_t;

// Backup formats
typedef enum {
    BACKUP_FORMAT_PLAIN,        // Plain text backup
    BACKUP_FORMAT_COMPRESSED,   // Gzip compressed
    BACKUP_FORMAT_ENCRYPTED,    // Encrypted backup
    BACKUP_FORMAT_VERSIONED,    // Git-like versioning
    BACKUP_FORMAT_INCREMENTAL   // Only changes since last backup
} backup_format_t;

// Backup destination types
typedef enum {
    BACKUP_DEST_LOCAL,          // Local filesystem
    BACKUP_DEST_NETWORK,        // Network share
    BACKUP_DEST_CLOUD,          // Cloud storage
    BACKUP_DEST_MULTIPLE        // Multiple destinations
} backup_destination_type_t;

// File change tracking
typedef struct {
    char* filepath;
    size_t content_hash;
    time_t last_modified;
    time_t last_backup;
    bool is_dirty;
    int change_count;
} backup_file_tracker_t;

// Backup entry metadata
typedef struct {
    char* original_path;
    char* backup_path;
    time_t timestamp;
    size_t original_size;
    size_t backup_size;
    backup_format_t format;
    char* checksum;
    int version_number;
    char* description;
} backup_entry_t;

// Backup destination configuration
typedef struct {
    backup_destination_type_t type;
    char* path;
    char* credentials; // For network/cloud access
    int max_versions;
    size_t max_total_size;
    bool auto_cleanup;
} backup_destination_t;

// Backup engine configuration
typedef struct {
    backup_strategy_t strategy;
    backup_format_t default_format;
    
    // Timing settings
    int auto_save_interval;  // seconds
    int idle_threshold;      // seconds of inactivity
    int max_backup_age;      // days to keep backups
    
    // Size limits
    size_t max_file_size;    // Don't backup files larger than this
    size_t max_total_backup_size;
    
    // Destinations
    backup_destination_t** destinations;
    int destination_count;
    
    // File patterns
    char** include_patterns;
    char** exclude_patterns;
    int include_pattern_count;
    int exclude_pattern_count;
    
    // Advanced settings
    bool compress_backups;
    bool encrypt_backups;
    bool verify_integrity;
    bool keep_deleted_files;
    
    // Callback functions
    void (*on_backup_start)(const char* filepath);
    void (*on_backup_complete)(const char* filepath, bool success);
    void (*on_backup_error)(const char* filepath, const char* error);
} backup_config_t;

// Backup engine state
typedef struct {
    backup_config_t* config;
    
    // File tracking
    backup_file_tracker_t** tracked_files;
    int tracked_file_count;
    int max_tracked_files;
    
    // Backup history
    backup_entry_t** backup_history;
    int history_count;
    int max_history;
    
    // State
    bool is_running;
    bool is_backing_up;
    time_t last_backup_time;
    time_t last_activity_time;
    
    // Statistics
    int total_backups_created;
    int total_files_backed_up;
    size_t total_bytes_backed_up;
    int failed_backup_count;
    
    // Threading (for async operations)
    void* backup_thread;
    void* mutex;
    
    // Working directory for temp files
    char* temp_dir;
} backup_engine_t;

// Restoration structures
typedef struct {
    backup_entry_t* backup;
    char* restore_path;
    bool verify_checksum;
    bool overwrite_existing;
} restore_request_t;

typedef struct {
    bool success;
    char* restored_path;
    char* error_message;
    time_t restore_time;
} restore_result_t;

// API Functions

// Engine lifecycle
backup_engine_t* backup_engine_create(backup_config_t* config);
void backup_engine_destroy(backup_engine_t* engine);
int backup_engine_start(backup_engine_t* engine);
int backup_engine_stop(backup_engine_t* engine);

// Configuration
backup_config_t* backup_engine_create_config(backup_strategy_t strategy);
void backup_engine_destroy_config(backup_config_t* config);
int backup_engine_add_destination(backup_config_t* config, backup_destination_t* dest);
int backup_engine_set_patterns(backup_config_t* config, const char** include, const char** exclude);

// File tracking
int backup_engine_track_file(backup_engine_t* engine, const char* filepath);
int backup_engine_untrack_file(backup_engine_t* engine, const char* filepath);
int backup_engine_track_directory(backup_engine_t* engine, const char* dirpath, bool recursive);
void backup_engine_file_changed(backup_engine_t* engine, const char* filepath);

// Backup operations
int backup_engine_backup_file(backup_engine_t* engine, const char* filepath, backup_format_t format);
int backup_engine_backup_all(backup_engine_t* engine);
int backup_engine_backup_changed_files(backup_engine_t* engine);
int backup_engine_force_backup(backup_engine_t* engine, const char* filepath);

// Restoration
backup_entry_t** backup_engine_list_backups(backup_engine_t* engine, const char* filepath, int* count);
restore_result_t* backup_engine_restore_file(backup_engine_t* engine, restore_request_t* request);
restore_result_t* backup_engine_restore_version(backup_engine_t* engine, const char* filepath, int version);
restore_result_t* backup_engine_restore_timestamp(backup_engine_t* engine, const char* filepath, time_t timestamp);

// History and metadata
backup_entry_t** backup_engine_get_history(backup_engine_t* engine, int* count);
backup_entry_t* backup_engine_get_latest_backup(backup_engine_t* engine, const char* filepath);
int backup_engine_cleanup_old_backups(backup_engine_t* engine);
int backup_engine_verify_backups(backup_engine_t* engine);

// Compression and encryption support
int backup_engine_compress_file(const char* source, const char* dest);
int backup_engine_decompress_file(const char* source, const char* dest);
int backup_engine_encrypt_file(const char* source, const char* dest, const char* key);
int backup_engine_decrypt_file(const char* source, const char* dest, const char* key);

// Utilities
char* backup_engine_calculate_checksum(const char* filepath);
bool backup_engine_verify_checksum(const char* filepath, const char* expected_checksum);
bool backup_engine_should_backup_file(backup_engine_t* engine, const char* filepath);
char* backup_engine_generate_backup_path(backup_engine_t* engine, const char* filepath, backup_format_t format);

// Statistics and monitoring
void backup_engine_get_stats(backup_engine_t* engine, int* files_tracked, int* backups_created, 
                             size_t* total_size, time_t* last_backup);
void backup_engine_print_stats(backup_engine_t* engine);
backup_file_tracker_t* backup_engine_get_file_status(backup_engine_t* engine, const char* filepath);

// Background operations (async)
int backup_engine_start_background_worker(backup_engine_t* engine);
int backup_engine_stop_background_worker(backup_engine_t* engine);
void backup_engine_schedule_backup(backup_engine_t* engine, const char* filepath, int delay_seconds);

// Cloud integration hooks (extensible)
typedef struct {
    int (*upload)(const char* local_path, const char* remote_path, const char* credentials);
    int (*download)(const char* remote_path, const char* local_path, const char* credentials);
    int (*delete_remote)(const char* remote_path, const char* credentials);
    bool (*exists_remote)(const char* remote_path, const char* credentials);
} cloud_operations_t;

int backup_engine_register_cloud_operations(backup_engine_t* engine, cloud_operations_t* ops);

// Error handling
typedef enum {
    BACKUP_ERROR_NONE = 0,
    BACKUP_ERROR_FILE_NOT_FOUND,
    BACKUP_ERROR_PERMISSION_DENIED,
    BACKUP_ERROR_DISK_FULL,
    BACKUP_ERROR_NETWORK_ERROR,
    BACKUP_ERROR_ENCRYPTION_FAILED,
    BACKUP_ERROR_COMPRESSION_FAILED,
    BACKUP_ERROR_CHECKSUM_MISMATCH,
    BACKUP_ERROR_CONFIG_INVALID,
    BACKUP_ERROR_MEMORY_ERROR
} backup_error_t;

const char* backup_engine_error_string(backup_error_t error);
backup_error_t backup_engine_get_last_error(backup_engine_t* engine);

// Persistence (save/load engine state)
int backup_engine_save_state(backup_engine_t* engine, const char* state_file);
backup_engine_t* backup_engine_load_state(const char* state_file);

#endif // BACKUP_ENGINE_H