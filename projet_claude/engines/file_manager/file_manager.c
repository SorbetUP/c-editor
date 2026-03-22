// file_manager.c - Cross-platform file management implementation
#include "file_manager.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#include <time.h>
#include <errno.h>

#ifdef __APPLE__
#include <pwd.h>
#include <libgen.h>
#endif

// Global configuration
static FileManagerConfig g_config = {
    .max_file_size = 10 * 1024 * 1024, // 10MB
    .max_recent_files = 10,
    .auto_backup = true,
    .create_missing_dirs = true,
    .backup_extension = ".bak",
    .default_extension = ".md"
};

static FileResult g_last_error = FILE_SUCCESS;

// Helper functions
static void set_last_error(FileResult error) {
    g_last_error = error;
}

static bool is_markdown_extension(const char* ext) {
    if (!ext) return false;
    return (strcmp(ext, ".md") == 0 || 
            strcmp(ext, ".markdown") == 0 || 
            strcmp(ext, ".mdown") == 0 ||
            strcmp(ext, ".mkd") == 0);
}

// File type detection based on extension
static FileType detect_file_type_by_extension(const char* ext) {
    if (!ext) return FILE_TYPE_UNKNOWN;
    
    // Markdown files
    if (is_markdown_extension(ext)) {
        return FILE_TYPE_MARKDOWN;
    }
    
    // Text files
    if (strcmp(ext, ".txt") == 0 || strcmp(ext, ".text") == 0) {
        return FILE_TYPE_TEXT;
    }
    
    // Image files
    if (strcmp(ext, ".jpg") == 0 || strcmp(ext, ".jpeg") == 0 || 
        strcmp(ext, ".png") == 0 || strcmp(ext, ".gif") == 0 ||
        strcmp(ext, ".bmp") == 0 || strcmp(ext, ".tiff") == 0 ||
        strcmp(ext, ".webp") == 0 || strcmp(ext, ".svg") == 0) {
        return FILE_TYPE_IMAGE;
    }
    
    // Document files
    if (strcmp(ext, ".pdf") == 0 || strcmp(ext, ".doc") == 0 || 
        strcmp(ext, ".docx") == 0 || strcmp(ext, ".rtf") == 0 ||
        strcmp(ext, ".odt") == 0 || strcmp(ext, ".pages") == 0) {
        return FILE_TYPE_DOCUMENT;
    }
    
    // Executable files
    if (strcmp(ext, ".exe") == 0 || strcmp(ext, ".app") == 0 || 
        strcmp(ext, ".dmg") == 0 || strcmp(ext, ".pkg") == 0) {
        return FILE_TYPE_EXECUTABLE;
    }
    
    // Archive files
    if (strcmp(ext, ".zip") == 0 || strcmp(ext, ".tar") == 0 || 
        strcmp(ext, ".gz") == 0 || strcmp(ext, ".rar") == 0 ||
        strcmp(ext, ".7z") == 0 || strcmp(ext, ".bz2") == 0) {
        return FILE_TYPE_ARCHIVE;
    }
    
    // Code files
    if (strcmp(ext, ".c") == 0 || strcmp(ext, ".h") == 0 ||
        strcmp(ext, ".cpp") == 0 || strcmp(ext, ".hpp") == 0 ||
        strcmp(ext, ".m") == 0 || strcmp(ext, ".mm") == 0 ||
        strcmp(ext, ".js") == 0 || strcmp(ext, ".ts") == 0 ||
        strcmp(ext, ".py") == 0 || strcmp(ext, ".java") == 0 ||
        strcmp(ext, ".swift") == 0 || strcmp(ext, ".go") == 0 ||
        strcmp(ext, ".rs") == 0 || strcmp(ext, ".php") == 0 ||
        strcmp(ext, ".rb") == 0 || strcmp(ext, ".css") == 0 ||
        strcmp(ext, ".html") == 0 || strcmp(ext, ".xml") == 0 ||
        strcmp(ext, ".json") == 0 || strcmp(ext, ".yaml") == 0 ||
        strcmp(ext, ".yml") == 0) {
        return FILE_TYPE_CODE;
    }
    
    // Audio files
    if (strcmp(ext, ".mp3") == 0 || strcmp(ext, ".wav") == 0 ||
        strcmp(ext, ".flac") == 0 || strcmp(ext, ".aac") == 0 ||
        strcmp(ext, ".ogg") == 0 || strcmp(ext, ".m4a") == 0) {
        return FILE_TYPE_AUDIO;
    }
    
    // Video files
    if (strcmp(ext, ".mp4") == 0 || strcmp(ext, ".avi") == 0 ||
        strcmp(ext, ".mov") == 0 || strcmp(ext, ".mkv") == 0 ||
        strcmp(ext, ".wmv") == 0 || strcmp(ext, ".flv") == 0 ||
        strcmp(ext, ".webm") == 0) {
        return FILE_TYPE_VIDEO;
    }
    
    return FILE_TYPE_UNKNOWN;
}

// Get MIME type based on file type
static const char* get_mime_type_for_file_type(FileType type, const char* ext) {
    switch (type) {
        case FILE_TYPE_MARKDOWN:
            return "text/markdown";
        case FILE_TYPE_TEXT:
            return "text/plain";
        case FILE_TYPE_IMAGE:
            if (ext) {
                if (strcmp(ext, ".jpg") == 0 || strcmp(ext, ".jpeg") == 0) return "image/jpeg";
                if (strcmp(ext, ".png") == 0) return "image/png";
                if (strcmp(ext, ".gif") == 0) return "image/gif";
                if (strcmp(ext, ".bmp") == 0) return "image/bmp";
                if (strcmp(ext, ".svg") == 0) return "image/svg+xml";
                if (strcmp(ext, ".webp") == 0) return "image/webp";
            }
            return "image/*";
        case FILE_TYPE_DOCUMENT:
            if (ext) {
                if (strcmp(ext, ".pdf") == 0) return "application/pdf";
                if (strcmp(ext, ".doc") == 0) return "application/msword";
                if (strcmp(ext, ".docx") == 0) return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
                if (strcmp(ext, ".rtf") == 0) return "application/rtf";
            }
            return "application/octet-stream";
        case FILE_TYPE_AUDIO:
            if (ext) {
                if (strcmp(ext, ".mp3") == 0) return "audio/mpeg";
                if (strcmp(ext, ".wav") == 0) return "audio/wav";
                if (strcmp(ext, ".flac") == 0) return "audio/flac";
                if (strcmp(ext, ".aac") == 0) return "audio/aac";
                if (strcmp(ext, ".ogg") == 0) return "audio/ogg";
            }
            return "audio/*";
        case FILE_TYPE_VIDEO:
            if (ext) {
                if (strcmp(ext, ".mp4") == 0) return "video/mp4";
                if (strcmp(ext, ".avi") == 0) return "video/x-msvideo";
                if (strcmp(ext, ".mov") == 0) return "video/quicktime";
                if (strcmp(ext, ".mkv") == 0) return "video/x-matroska";
                if (strcmp(ext, ".webm") == 0) return "video/webm";
            }
            return "video/*";
        case FILE_TYPE_CODE:
            return "text/plain";
        case FILE_TYPE_ARCHIVE:
            if (ext) {
                if (strcmp(ext, ".zip") == 0) return "application/zip";
                if (strcmp(ext, ".tar") == 0) return "application/x-tar";
                if (strcmp(ext, ".gz") == 0) return "application/gzip";
                if (strcmp(ext, ".rar") == 0) return "application/vnd.rar";
            }
            return "application/octet-stream";
        case FILE_TYPE_FOLDER:
            return "inode/directory";
        default:
            return "application/octet-stream";
    }
}

// Get human-readable description for file type
static const char* get_description_for_file_type(FileType type, const char* ext) {
    switch (type) {
        case FILE_TYPE_MARKDOWN:
            return "Markdown Document";
        case FILE_TYPE_TEXT:
            return "Text Document";
        case FILE_TYPE_IMAGE:
            if (ext) {
                if (strcmp(ext, ".jpg") == 0 || strcmp(ext, ".jpeg") == 0) return "JPEG Image";
                if (strcmp(ext, ".png") == 0) return "PNG Image";
                if (strcmp(ext, ".gif") == 0) return "GIF Image";
                if (strcmp(ext, ".bmp") == 0) return "Bitmap Image";
                if (strcmp(ext, ".svg") == 0) return "SVG Vector Image";
                if (strcmp(ext, ".webp") == 0) return "WebP Image";
            }
            return "Image File";
        case FILE_TYPE_DOCUMENT:
            if (ext) {
                if (strcmp(ext, ".pdf") == 0) return "PDF Document";
                if (strcmp(ext, ".doc") == 0) return "Word Document";
                if (strcmp(ext, ".docx") == 0) return "Word Document";
                if (strcmp(ext, ".rtf") == 0) return "Rich Text Document";
                if (strcmp(ext, ".pages") == 0) return "Pages Document";
            }
            return "Document";
        case FILE_TYPE_AUDIO:
            if (ext) {
                if (strcmp(ext, ".mp3") == 0) return "MP3 Audio";
                if (strcmp(ext, ".wav") == 0) return "WAV Audio";
                if (strcmp(ext, ".flac") == 0) return "FLAC Audio";
                if (strcmp(ext, ".aac") == 0) return "AAC Audio";
                if (strcmp(ext, ".ogg") == 0) return "OGG Audio";
            }
            return "Audio File";
        case FILE_TYPE_VIDEO:
            if (ext) {
                if (strcmp(ext, ".mp4") == 0) return "MP4 Video";
                if (strcmp(ext, ".avi") == 0) return "AVI Video";
                if (strcmp(ext, ".mov") == 0) return "QuickTime Video";
                if (strcmp(ext, ".mkv") == 0) return "Matroska Video";
                if (strcmp(ext, ".webm") == 0) return "WebM Video";
            }
            return "Video File";
        case FILE_TYPE_CODE:
            if (ext) {
                if (strcmp(ext, ".c") == 0 || strcmp(ext, ".h") == 0) return "C Source Code";
                if (strcmp(ext, ".cpp") == 0 || strcmp(ext, ".hpp") == 0) return "C++ Source Code";
                if (strcmp(ext, ".m") == 0 || strcmp(ext, ".mm") == 0) return "Objective-C Source Code";
                if (strcmp(ext, ".js") == 0) return "JavaScript Code";
                if (strcmp(ext, ".ts") == 0) return "TypeScript Code";
                if (strcmp(ext, ".py") == 0) return "Python Script";
                if (strcmp(ext, ".java") == 0) return "Java Source Code";
                if (strcmp(ext, ".swift") == 0) return "Swift Source Code";
                if (strcmp(ext, ".html") == 0) return "HTML Document";
                if (strcmp(ext, ".css") == 0) return "CSS Stylesheet";
                if (strcmp(ext, ".json") == 0) return "JSON Data";
                if (strcmp(ext, ".xml") == 0) return "XML Document";
            }
            return "Source Code";
        case FILE_TYPE_ARCHIVE:
            if (ext) {
                if (strcmp(ext, ".zip") == 0) return "ZIP Archive";
                if (strcmp(ext, ".tar") == 0) return "TAR Archive";
                if (strcmp(ext, ".gz") == 0) return "GZIP Archive";
                if (strcmp(ext, ".rar") == 0) return "RAR Archive";
                if (strcmp(ext, ".7z") == 0) return "7-Zip Archive";
            }
            return "Archive File";
        case FILE_TYPE_EXECUTABLE:
            return "Executable File";
        case FILE_TYPE_FOLDER:
            return "Folder";
        default:
            return "Unknown File Type";
    }
}

// Core file operations
FILE_API FileResult file_read(const char* path, FileContent** content) {
    if (!path || !content) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    FILE* file = fopen(path, "rb");
    if (!file) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    // Get file size
    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    if (size > (long)g_config.max_file_size) {
        fclose(file);
        set_last_error(FILE_ERROR_TOO_LARGE);
        return FILE_ERROR_TOO_LARGE;
    }
    
    // Allocate content structure
    FileContent* file_content = malloc(sizeof(FileContent));
    if (!file_content) {
        fclose(file);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    // Allocate content buffer
    file_content->content = malloc(size + 1);
    if (!file_content->content) {
        free(file_content);
        fclose(file);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    // Read content
    size_t read_size = fread(file_content->content, 1, size, file);
    file_content->content[read_size] = '\0';
    file_content->size = read_size;
    file_content->encoding = strdup("UTF-8"); // Assume UTF-8
    
    fclose(file);
    *content = file_content;
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API FileResult file_write(const char* path, const char* content, size_t size) {
    if (!path || !content) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    // Create backup if enabled
    if (g_config.auto_backup && file_exists(path)) {
        file_create_backup(path);
    }
    
    // Create directory if needed
    if (g_config.create_missing_dirs) {
        char* dir = path_get_directory(path);
        if (dir) {
            dir_create(dir);
            free(dir);
        }
    }
    
    FILE* file = fopen(path, "wb");
    if (!file) {
        set_last_error(FILE_ERROR_PERMISSION);
        return FILE_ERROR_PERMISSION;
    }
    
    size_t written = fwrite(content, 1, size, file);
    fclose(file);
    
    if (written != size) {
        set_last_error(FILE_ERROR_IO);
        return FILE_ERROR_IO;
    }
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API FileResult file_get_info(const char* path, HybridFileInfo** info) {
    if (!path || !info) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    struct stat st;
    if (stat(path, &st) != 0) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    HybridFileInfo* file_info = malloc(sizeof(HybridFileInfo));
    if (!file_info) {
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    file_info->path = strdup(path);
    file_info->name = path_get_filename(path);
    file_info->extension = path_get_extension(path);
    file_info->size = st.st_size;
    file_info->modified = st.st_mtime;
    file_info->created = st.st_ctime;
    file_info->is_directory = S_ISDIR(st.st_mode);
    file_info->is_readable = (access(path, R_OK) == 0);
    file_info->is_writable = (access(path, W_OK) == 0);
    
    // Determine file type
    if (file_info->is_directory) {
        file_info->file_type = FILE_TYPE_FOLDER;
    } else {
        file_info->file_type = detect_file_type_by_extension(file_info->extension);
    }
    
    // Set MIME type and description
    const char* mime = get_mime_type_for_file_type(file_info->file_type, file_info->extension);
    file_info->mime_type = strdup(mime);
    
    const char* desc = get_description_for_file_type(file_info->file_type, file_info->extension);
    file_info->description = strdup(desc);
    
    *info = file_info;
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API bool file_exists(const char* path) {
    if (!path) return false;
    return access(path, F_OK) == 0;
}

FILE_API bool file_is_markdown(const char* path) {
    if (!path) return false;
    char* ext = path_get_extension(path);
    bool result = is_markdown_extension(ext);
    free(ext);
    return result;
}

// Directory operations
FILE_API FileResult dir_create(const char* path) {
    if (!path) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    if (mkdir(path, 0755) != 0 && errno != EEXIST) {
        set_last_error(FILE_ERROR_PERMISSION);
        return FILE_ERROR_PERMISSION;
    }
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

// Directory operations
FILE_API FileResult dir_list(const char* path, DirectoryListing** listing) {
    if (!path || !listing) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    DIR* dir = opendir(path);
    if (!dir) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    // Count entries first (excluding hidden files)
    int count = 0;
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_name[0] == '.') continue; // Skip hidden files
        count++;
    }
    rewinddir(dir);
    
    // Allocate listing
    DirectoryListing* dir_listing = malloc(sizeof(DirectoryListing));
    if (!dir_listing) {
        closedir(dir);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    dir_listing->files = malloc(sizeof(HybridFileInfo) * count);
    if (!dir_listing->files && count > 0) {
        free(dir_listing);
        closedir(dir);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    dir_listing->count = 0;
    dir_listing->directory_path = strdup(path);
    
    // Populate with all files and directories
    while ((entry = readdir(dir)) != NULL && dir_listing->count < count) {
        if (entry->d_name[0] == '.') continue;
        
        char* full_path = path_join(path, entry->d_name);
        HybridFileInfo* info;
        if (file_get_info(full_path, &info) == FILE_SUCCESS) {
            dir_listing->files[dir_listing->count] = *info;
            free(info); // Just the struct, not the strings
            dir_listing->count++;
        }
        free(full_path);
    }
    
    closedir(dir);
    *listing = dir_listing;
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API FileResult dir_list_markdown(const char* path, DirectoryListing** listing) {
    if (!path || !listing) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    DIR* dir = opendir(path);
    if (!dir) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    // Count markdown files first
    int count = 0;
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_name[0] == '.') continue; // Skip hidden files
        
        char* full_path = path_join(path, entry->d_name);
        if (file_is_markdown(full_path)) {
            count++;
        }
        free(full_path);
    }
    rewinddir(dir);
    
    // Allocate listing
    DirectoryListing* dir_listing = malloc(sizeof(DirectoryListing));
    if (!dir_listing) {
        closedir(dir);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    dir_listing->files = malloc(sizeof(HybridFileInfo) * count);
    if (!dir_listing->files && count > 0) {
        free(dir_listing);
        closedir(dir);
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    dir_listing->count = 0;
    dir_listing->directory_path = strdup(path);
    
    // Populate with markdown files
    while ((entry = readdir(dir)) != NULL && dir_listing->count < count) {
        if (entry->d_name[0] == '.') continue;
        
        char* full_path = path_join(path, entry->d_name);
        if (file_is_markdown(full_path)) {
            HybridFileInfo* info;
            if (file_get_info(full_path, &info) == FILE_SUCCESS) {
                dir_listing->files[dir_listing->count] = *info;
                free(info); // Just the struct, not the strings
                dir_listing->count++;
            }
        }
        free(full_path);
    }
    
    closedir(dir);
    *listing = dir_listing;
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

// Path utilities
FILE_API char* path_get_directory(const char* path) {
    if (!path) return NULL;
    
    char* path_copy = strdup(path);
    char* dir = dirname(path_copy);
    char* result = strdup(dir);
    free(path_copy);
    return result;
}

FILE_API char* path_get_filename(const char* path) {
    if (!path) return NULL;
    
    const char* last_slash = strrchr(path, '/');
    if (last_slash) {
        return strdup(last_slash + 1);
    }
    return strdup(path);
}

FILE_API char* path_get_extension(const char* path) {
    if (!path) return NULL;
    
    const char* last_dot = strrchr(path, '.');
    const char* last_slash = strrchr(path, '/');
    
    // Make sure the dot is after the last slash (not in directory name)
    if (last_dot && (!last_slash || last_dot > last_slash)) {
        return strdup(last_dot);
    }
    return strdup("");
}

FILE_API char* path_get_basename(const char* path) {
    if (!path) return NULL;
    
    char* filename = path_get_filename(path);
    char* last_dot = strrchr(filename, '.');
    if (last_dot) {
        *last_dot = '\0';
    }
    return filename;
}

FILE_API char* path_join(const char* dir, const char* filename) {
    if (!dir || !filename) return NULL;
    
    size_t dir_len = strlen(dir);
    size_t filename_len = strlen(filename);
    size_t total_len = dir_len + filename_len + 2; // +1 for '/', +1 for '\0'
    
    char* result = malloc(total_len);
    if (!result) return NULL;
    
    strcpy(result, dir);
    if (dir_len > 0 && dir[dir_len - 1] != '/') {
        strcat(result, "/");
    }
    strcat(result, filename);
    
    return result;
}

// Backup operations
FILE_API FileResult file_create_backup(const char* path) {
    if (!path) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    if (!file_exists(path)) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    char* backup_path = malloc(strlen(path) + strlen(g_config.backup_extension) + 1);
    sprintf(backup_path, "%s%s", path, g_config.backup_extension);
    
    FileResult result = file_copy(path, backup_path);
    free(backup_path);
    
    return result;
}

FILE_API FileResult file_copy(const char* src, const char* dest) {
    if (!src || !dest) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    FileContent* content;
    FileResult result = file_read(src, &content);
    if (result != FILE_SUCCESS) {
        return result;
    }
    
    result = file_write(dest, content->content, content->size);
    file_free_content(content);
    
    return result;
}

// Recent files management
FILE_API RecentFiles* recent_files_create(int max_count) {
    RecentFiles* recent = malloc(sizeof(RecentFiles));
    if (!recent) return NULL;
    
    recent->paths = malloc(sizeof(char*) * max_count);
    if (!recent->paths) {
        free(recent);
        return NULL;
    }
    
    recent->count = 0;
    recent->max_count = max_count;
    
    return recent;
}

FILE_API void recent_files_destroy(RecentFiles* recent) {
    if (!recent) return;
    
    for (int i = 0; i < recent->count; i++) {
        free(recent->paths[i]);
    }
    free(recent->paths);
    free(recent);
}

FILE_API FileResult recent_files_add(RecentFiles* recent, const char* path) {
    if (!recent || !path) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    // Check if already exists and remove it
    for (int i = 0; i < recent->count; i++) {
        if (strcmp(recent->paths[i], path) == 0) {
            free(recent->paths[i]);
            // Shift everything down
            for (int j = i; j < recent->count - 1; j++) {
                recent->paths[j] = recent->paths[j + 1];
            }
            recent->count--;
            break;
        }
    }
    
    // Add to front
    if (recent->count >= recent->max_count) {
        // Remove oldest
        free(recent->paths[recent->count - 1]);
        recent->count--;
    }
    
    // Shift everything up
    for (int i = recent->count; i > 0; i--) {
        recent->paths[i] = recent->paths[i - 1];
    }
    
    recent->paths[0] = strdup(path);
    recent->count++;
    
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

// Memory management
FILE_API void file_free_content(FileContent* content) {
    if (content) {
        free(content->content);
        free(content->encoding);
        free(content);
    }
}

FILE_API void file_free_info(HybridFileInfo* info) {
    if (info) {
        free(info->path);
        free(info->name);
        free(info->extension);
        free(info->mime_type);
        free(info->description);
        free(info);
    }
}

FILE_API void file_free_directory_listing(DirectoryListing* listing) {
    if (listing) {
        for (int i = 0; i < listing->count; i++) {
            free(listing->files[i].path);
            free(listing->files[i].name);
            free(listing->files[i].extension);
            free(listing->files[i].mime_type);
            free(listing->files[i].description);
        }
        free(listing->files);
        free(listing->directory_path);
        free(listing);
    }
}

FILE_API void file_free_string(char* str) {
    free(str);
}

// Configuration
FILE_API void file_manager_set_config(const FileManagerConfig* config) {
    if (config) {
        g_config = *config;
    }
}

FILE_API FileManagerConfig file_manager_get_config(void) {
    return g_config;
}

FILE_API FileResult file_manager_init(void) {
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API void file_manager_cleanup(void) {
    // Cleanup any global resources
}

// Error handling
FILE_API const char* file_get_error_message(FileResult result) {
    switch (result) {
        case FILE_SUCCESS: return "Success";
        case FILE_ERROR_NOT_FOUND: return "File not found";
        case FILE_ERROR_PERMISSION: return "Permission denied";
        case FILE_ERROR_OUT_OF_MEMORY: return "Out of memory";
        case FILE_ERROR_IO: return "I/O error";
        case FILE_ERROR_INVALID_PATH: return "Invalid path";
        case FILE_ERROR_EXISTS: return "File already exists";
        case FILE_ERROR_TOO_LARGE: return "File too large";
        default: return "Unknown error";
    }
}

FILE_API FileResult file_get_last_error(void) {
    return g_last_error;
}

FILE_API void file_clear_last_error(void) {
    g_last_error = FILE_SUCCESS;
}

// Platform-specific utilities
FILE_API char* file_get_documents_dir(void) {
#ifdef __APPLE__
    struct passwd* pw = getpwuid(getuid());
    if (pw) {
        char* docs_path = malloc(strlen(pw->pw_dir) + 20);
        sprintf(docs_path, "%s/Documents", pw->pw_dir);
        return docs_path;
    }
#endif
    return strdup("./Documents");
}

FILE_API char* file_get_desktop_dir(void) {
#ifdef __APPLE__
    struct passwd* pw = getpwuid(getuid());
    if (pw) {
        char* desktop_path = malloc(strlen(pw->pw_dir) + 20);
        sprintf(desktop_path, "%s/Desktop", pw->pw_dir);
        return desktop_path;
    }
#endif
    return strdup("./Desktop");
}

// Advanced backup operations
FILE_API bool file_content_differs(const char* path, const char* new_content, size_t new_size) {
    if (!path || !new_content) return true;
    
    FileContent* existing;
    FileResult result = file_read(path, &existing);
    if (result != FILE_SUCCESS) {
        return true; // File doesn't exist or can't be read, so content differs
    }
    
    bool differs = (existing->size != new_size) || 
                   (memcmp(existing->content, new_content, new_size) != 0);
    
    file_free_content(existing);
    return differs;
}

FILE_API FileResult file_save_with_backup(const char* path, const char* content, size_t size) {
    if (!path || !content) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    // Check if content actually differs
    if (!file_content_differs(path, content, size)) {
        set_last_error(FILE_SUCCESS);
        return FILE_SUCCESS; // No changes, no need to save
    }
    
    // Create timestamped backup if file exists
    if (file_exists(path)) {
        FileResult backup_result = file_create_timestamped_backup(path);
        if (backup_result != FILE_SUCCESS) {
            // Log warning but continue with save
            printf("Warning: Could not create backup: %s\n", file_get_error_message(backup_result));
        }
    }
    
    // Perform the save
    return file_write(path, content, size);
}

FILE_API FileResult file_create_timestamped_backup(const char* path) {
    if (!path || !file_exists(path)) {
        set_last_error(FILE_ERROR_NOT_FOUND);
        return FILE_ERROR_NOT_FOUND;
    }
    
    // Create timestamp
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char timestamp[32];
    strftime(timestamp, sizeof(timestamp), "%Y%m%d_%H%M%S", tm_info);
    
    // Create backup path
    char* backup_path = file_get_backup_path(path, timestamp);
    if (!backup_path) {
        set_last_error(FILE_ERROR_OUT_OF_MEMORY);
        return FILE_ERROR_OUT_OF_MEMORY;
    }
    
    FileResult result = file_copy(path, backup_path);
    free(backup_path);
    
    return result;
}

FILE_API char* file_get_backup_path(const char* original_path, const char* timestamp) {
    if (!original_path) return NULL;
    
    char* dir = path_get_directory(original_path);
    char* basename = path_get_basename(original_path);
    char* ext = path_get_extension(original_path);
    
    if (!dir || !basename) {
        free(dir);
        free(basename);
        free(ext);
        return NULL;
    }
    
    // Create backup filename: basename_timestamp.bak.ext
    size_t path_len = strlen(dir) + strlen(basename) + strlen(timestamp) + strlen(ext) + 20;
    char* backup_path = malloc(path_len);
    
    if (timestamp) {
        snprintf(backup_path, path_len, "%s/%s_%s.bak%s", dir, basename, timestamp, ext);
    } else {
        snprintf(backup_path, path_len, "%s/%s.bak%s", dir, basename, ext);
    }
    
    free(dir);
    free(basename);
    free(ext);
    
    return backup_path;
}

FILE_API bool file_compare_content(const char* path1, const char* path2) {
    if (!path1 || !path2) return false;
    
    FileContent* content1 = NULL;
    FileContent* content2 = NULL;
    
    FileResult result1 = file_read(path1, &content1);
    FileResult result2 = file_read(path2, &content2);
    
    if (result1 != FILE_SUCCESS || result2 != FILE_SUCCESS) {
        file_free_content(content1);
        file_free_content(content2);
        return false;
    }
    
    bool same = (content1->size == content2->size) &&
                (memcmp(content1->content, content2->content, content1->size) == 0);
    
    file_free_content(content1);
    file_free_content(content2);
    
    return same;
}

FILE_API FileResult file_get_content_hash(const char* path, char** hash) {
    if (!path || !hash) {
        set_last_error(FILE_ERROR_INVALID_PATH);
        return FILE_ERROR_INVALID_PATH;
    }
    
    FileContent* content;
    FileResult result = file_read(path, &content);
    if (result != FILE_SUCCESS) {
        return result;
    }
    
    // Simple hash based on content size and first/last bytes
    unsigned long simple_hash = content->size;
    if (content->size > 0) {
        simple_hash += (unsigned char)content->content[0] * 31;
        if (content->size > 1) {
            simple_hash += (unsigned char)content->content[content->size - 1] * 17;
        }
    }
    
    *hash = malloc(32);
    if (*hash) {
        snprintf(*hash, 32, "%08lx", simple_hash);
    }
    
    file_free_content(content);
    set_last_error(FILE_SUCCESS);
    return FILE_SUCCESS;
}

FILE_API bool file_needs_backup(const char* path, const char* new_content, size_t new_size) {
    if (!g_config.auto_backup) return false;
    if (!file_exists(path)) return false;
    
    return file_content_differs(path, new_content, new_size);
}

// Validation
FILE_API bool file_is_valid_markdown_name(const char* name) {
    if (!name || strlen(name) == 0) return false;
    
    // Check for invalid characters
    const char* invalid_chars = "<>:\"|?*";
    for (const char* c = invalid_chars; *c; c++) {
        if (strchr(name, *c)) return false;
    }
    
    return true;
}

FILE_API bool file_has_unsaved_changes(const char* original, const char* current) {
    if (!original && !current) return false;
    if (!original || !current) return true;
    return strcmp(original, current) != 0;
}