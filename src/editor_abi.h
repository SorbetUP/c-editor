// editor_abi.h - Stable ABI interface for Flutter FFI
// This header provides a C-compatible interface that remains stable across
// versions

#ifndef EDITOR_ABI_H
#define EDITOR_ABI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Version information
#define EDITOR_ABI_VERSION_MAJOR 1
#define EDITOR_ABI_VERSION_MINOR 0
#define EDITOR_ABI_VERSION_PATCH 0

// Export macros for different platforms
#ifdef _WIN32
#ifdef BUILDING_EDITOR_DLL
#define EDITOR_API __declspec(dllexport)
#else
#define EDITOR_API __declspec(dllimport)
#endif
#else
#define EDITOR_API __attribute__((visibility("default")))
#endif

// Opaque handle for editor state
typedef struct EditorState EditorState;

// Result codes
typedef enum {
  EDITOR_SUCCESS = 0,
  EDITOR_ERROR_INVALID_PARAMETER = -1,
  EDITOR_ERROR_OUT_OF_MEMORY = -2,
  EDITOR_ERROR_PARSE_FAILED = -3,
  EDITOR_ERROR_EXPORT_FAILED = -4,
  EDITOR_ERROR_NOT_INITIALIZED = -5,
} EditorResult;

// Memory management function types
typedef void *(*EditorMallocFunc)(size_t size);
typedef void (*EditorFreeFunc)(void *ptr);
typedef void *(*EditorReallocFunc)(void *ptr, size_t size);

// Memory allocator configuration
typedef struct {
  EditorMallocFunc malloc_fn;
  EditorFreeFunc free_fn;
  EditorReallocFunc realloc_fn;
} EditorAllocator;

// Library initialization and cleanup
EDITOR_API EditorResult editor_library_init(void);
EDITOR_API EditorResult
editor_library_init_with_allocator(const EditorAllocator *allocator);
EDITOR_API void editor_library_cleanup(void);

// Version information
EDITOR_API void editor_get_version(int *major, int *minor, int *patch);
EDITOR_API const char *editor_get_version_string(void);

// Document parsing - stateless API
EDITOR_API EditorResult editor_parse_markdown(const char *markdown,
                                              char **out_json);
EDITOR_API EditorResult editor_export_markdown(const char *json,
                                               char **out_markdown);
EDITOR_API EditorResult editor_export_json_canonical(const char *json,
                                                     char **out_canonical);

// Editor state management - stateful API for character-by-character input
EDITOR_API EditorState *editor_state_create(void);
EDITOR_API void editor_state_destroy(EditorState *state);
EDITOR_API EditorResult editor_state_reset(EditorState *state);

// Character input simulation
EDITOR_API EditorResult editor_state_input_char(EditorState *state,
                                                int32_t char_code);
EDITOR_API EditorResult editor_state_input_string(EditorState *state,
                                                  const char *text);
EDITOR_API EditorResult editor_state_backspace(EditorState *state);
EDITOR_API EditorResult editor_state_delete(EditorState *state);

// Document retrieval from editor state
EDITOR_API EditorResult editor_state_get_document(EditorState *state,
                                                  char **out_json);
EDITOR_API EditorResult editor_state_get_markdown(EditorState *state,
                                                  char **out_markdown);

// Memory management for returned strings
EDITOR_API void editor_free_string(char *str);

// Error handling
EDITOR_API const char *editor_get_error_message(EditorResult result);
EDITOR_API EditorResult editor_get_last_error(void);
EDITOR_API void editor_clear_last_error(void);

// Debug and diagnostics
EDITOR_API void editor_enable_debug_logging(bool enabled);
EDITOR_API void editor_set_log_callback(void (*callback)(int level,
                                                         const char *message));

// Validation functions
EDITOR_API bool editor_is_valid_markdown(const char *markdown);
EDITOR_API bool editor_is_valid_json(const char *json);

// Utility functions
EDITOR_API size_t editor_estimate_json_size(const char *markdown);
EDITOR_API size_t editor_estimate_markdown_size(const char *json);

// Thread safety (if compiled with thread support)
#ifdef EDITOR_THREAD_SAFE
EDITOR_API EditorResult editor_lock(void);
EDITOR_API EditorResult editor_unlock(void);
EDITOR_API bool editor_is_thread_safe(void);
#endif

// Feature detection
typedef enum {
  EDITOR_FEATURE_TABLES = 1,
  EDITOR_FEATURE_IMAGES = 2,
  EDITOR_FEATURE_INLINE_STYLES = 4,
  EDITOR_FEATURE_HEADERS = 8,
  EDITOR_FEATURE_COLORS = 16,
  EDITOR_FEATURE_THREAD_SAFE = 32,
} EditorFeatures;

EDITOR_API uint32_t editor_get_features(void);
EDITOR_API bool editor_has_feature(EditorFeatures feature);

// Configuration
typedef struct {
  bool enable_tables;
  bool enable_images;
  bool enable_inline_styles;
  bool enable_headers;
  bool strict_parsing;
  size_t max_document_size;
  size_t max_nesting_depth;
} EditorConfig;

EDITOR_API EditorResult editor_set_config(const EditorConfig *config);
EDITOR_API EditorResult editor_get_config(EditorConfig *config);

#ifdef __cplusplus
}
#endif

#endif // EDITOR_ABI_H
