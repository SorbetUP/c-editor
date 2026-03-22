// file_manager.h - Cross-platform file management library
// Handles file operations for markdown documents

#ifndef FILE_MANAGER_H
#define FILE_MANAGER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

// Export macros for different platforms
#ifdef _WIN32
#ifdef BUILDING_FILE_DLL
#define FILE_API __declspec(dllexport)
#else
#define FILE_API __declspec(dllimport)
#endif
#else
#define FILE_API __attribute__((visibility("default")))
#endif

// File operations result codes
typedef enum {
    FILE_SUCCESS = 0,
    FILE_ERROR_NOT_FOUND = -1,
    FILE_ERROR_PERMISSION = -2,
    FILE_ERROR_OUT_OF_MEMORY = -3,
    FILE_ERROR_IO = -4,
    FILE_ERROR_INVALID_PATH = -5,
    FILE_ERROR_EXISTS = -6,
    FILE_ERROR_TOO_LARGE = -7,
    FILE_ERROR_UNSUPPORTED_TYPE = -8
} FileResult;

// File type classification
typedef enum {
    FILE_TYPE_UNKNOWN = 0,
    FILE_TYPE_MARKDOWN = 1,
    FILE_TYPE_TEXT = 2,
    FILE_TYPE_IMAGE = 3,
    FILE_TYPE_DOCUMENT = 4,
    FILE_TYPE_FOLDER = 5,
    FILE_TYPE_EXECUTABLE = 6,
    FILE_TYPE_ARCHIVE = 7,
    FILE_TYPE_CODE = 8,
    FILE_TYPE_AUDIO = 9,
    FILE_TYPE_VIDEO = 10
} FileType;

// File information structure
typedef struct {
    char* path;
    char* name;
    char* extension;
    size_t size;
    time_t modified;
    time_t created;
    bool is_directory;
    bool is_readable;
    bool is_writable;
    FileType file_type;
    char* mime_type;
    char* description;  // Human-readable description
} HybridFileInfo;

// Directory listing
typedef struct {
    HybridFileInfo* files;
    int count;
    char* directory_path;
} DirectoryListing;

// File content structure
typedef struct {
    char* content;
    size_t size;
    char* encoding; // UTF-8, UTF-16, etc.
} FileContent;

// Recent files management
typedef struct {
    char** paths;
    int count;
    int max_count;
} RecentFiles;

// Configuration
typedef struct {
    size_t max_file_size;      // Maximum file size to load (default: 10MB)
    int max_recent_files;      // Maximum recent files (default: 10)
    bool auto_backup;          // Auto backup before save
    bool create_missing_dirs;  // Create directories if they don't exist
    char* backup_extension;    // Backup file extension (default: .bak)
    char* default_extension;   // Default extension for new files (.md)
} FileManagerConfig;

// Core file operations
FILE_API FileResult file_read(const char* path, FileContent** content);
FILE_API FileResult file_write(const char* path, const char* content, size_t size);
FILE_API FileResult file_append(const char* path, const char* content, size_t size);
FILE_API FileResult file_delete(const char* path);
FILE_API FileResult file_copy(const char* src, const char* dest);
FILE_API FileResult file_move(const char* src, const char* dest);

// File information
FILE_API FileResult file_get_info(const char* path, HybridFileInfo** info);
FILE_API bool file_exists(const char* path);
FILE_API bool file_is_markdown(const char* path);
FILE_API FileResult file_get_size(const char* path, size_t* size);

// Directory operations
FILE_API FileResult dir_create(const char* path);
FILE_API FileResult dir_list(const char* path, DirectoryListing** listing);
FILE_API FileResult dir_list_markdown(const char* path, DirectoryListing** listing);
FILE_API FileResult dir_delete(const char* path, bool recursive);

// Path utilities
FILE_API char* path_get_directory(const char* path);
FILE_API char* path_get_filename(const char* path);
FILE_API char* path_get_extension(const char* path);
FILE_API char* path_get_basename(const char* path); // filename without extension
FILE_API char* path_join(const char* dir, const char* filename);
FILE_API char* path_normalize(const char* path);
FILE_API bool path_is_absolute(const char* path);

// Backup operations
FILE_API FileResult file_create_backup(const char* path);
FILE_API FileResult file_restore_backup(const char* path);
FILE_API FileResult file_list_backups(const char* path, DirectoryListing** backups);

// Advanced backup with comparison
FILE_API bool file_content_differs(const char* path, const char* new_content, size_t new_size);
FILE_API FileResult file_save_with_backup(const char* path, const char* content, size_t size);
FILE_API FileResult file_create_timestamped_backup(const char* path);
FILE_API char* file_get_backup_path(const char* original_path, const char* timestamp);

// Content comparison utilities
FILE_API bool file_compare_content(const char* path1, const char* path2);
FILE_API FileResult file_get_content_hash(const char* path, char** hash);
FILE_API bool file_needs_backup(const char* path, const char* new_content, size_t new_size);

// Recent files management
FILE_API RecentFiles* recent_files_create(int max_count);
FILE_API void recent_files_destroy(RecentFiles* recent);
FILE_API FileResult recent_files_add(RecentFiles* recent, const char* path);
FILE_API FileResult recent_files_remove(RecentFiles* recent, const char* path);
FILE_API FileResult recent_files_clear(RecentFiles* recent);
FILE_API FileResult recent_files_save(RecentFiles* recent, const char* config_path);
FILE_API FileResult recent_files_load(RecentFiles* recent, const char* config_path);

// Auto-save functionality
FILE_API FileResult file_auto_save(const char* path, const char* content, size_t size);
FILE_API FileResult file_recover_auto_save(const char* path, FileContent** content);
FILE_API FileResult file_clean_auto_save(const char* path);

// File watching (for auto-reload)
typedef void (*FileWatchCallback)(const char* path, void* user_data);
FILE_API FileResult file_watch_start(const char* path, FileWatchCallback callback, void* user_data);
FILE_API FileResult file_watch_stop(const char* path);

// Memory management
FILE_API void file_free_content(FileContent* content);
FILE_API void file_free_info(HybridFileInfo* info);
FILE_API void file_free_directory_listing(DirectoryListing* listing);
FILE_API void file_free_string(char* str);

// Configuration
FILE_API void file_manager_set_config(const FileManagerConfig* config);
FILE_API FileManagerConfig file_manager_get_config(void);
FILE_API FileResult file_manager_init(void);
FILE_API void file_manager_cleanup(void);

// Error handling
FILE_API const char* file_get_error_message(FileResult result);
FILE_API FileResult file_get_last_error(void);
FILE_API void file_clear_last_error(void);

// Platform-specific utilities
FILE_API char* file_get_documents_dir(void);
FILE_API char* file_get_temp_dir(void);
FILE_API char* file_get_app_data_dir(const char* app_name);
FILE_API char* file_get_desktop_dir(void);

// Dialog helpers (for integration with UI)
typedef struct {
    char** extensions;     // File extensions to filter
    int extension_count;
    char* title;          // Dialog title
    char* default_name;   // Default filename
    char* initial_dir;    // Initial directory
} FileDialogOptions;

FILE_API char* file_dialog_open(const FileDialogOptions* options);
FILE_API char* file_dialog_save(const FileDialogOptions* options);
FILE_API char* file_dialog_select_folder(const char* title, const char* initial_dir);

// Validation
FILE_API bool file_is_valid_markdown_name(const char* name);
FILE_API FileResult file_validate_content(const char* content, size_t size);
FILE_API bool file_has_unsaved_changes(const char* original, const char* current);

#ifdef __cplusplus
}
#endif

#endif // FILE_MANAGER_H