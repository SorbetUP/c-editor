// editor_abi.c - Implementation of stable ABI interface
#include "editor_abi.h"
#include "editor.h"
#include "markdown.h"
#include "json.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdarg.h>

// Global state
static bool g_initialized = false;
static EditorResult g_last_error = EDITOR_SUCCESS;
static EditorAllocator g_allocator = {0};
static bool g_debug_enabled = false;
static void (*g_log_callback)(int, const char*) = NULL;
static EditorConfig g_config = {
    .enable_tables = true,
    .enable_images = true,
    .enable_inline_styles = true,
    .enable_headers = true,
    .strict_parsing = false,
    .max_document_size = 100 * 1024 * 1024, // 100MB
    .max_nesting_depth = 64,
};

// Default allocator using standard library
static void* default_malloc(size_t size) {
    return malloc(size);
}

static void default_free(void* ptr) {
    free(ptr);
}

static void* default_realloc(void* ptr, size_t size) {
    return realloc(ptr, size);
}

// Logging helper
static void log_debug(const char* format, ...) {
    if (!g_debug_enabled) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    if (g_log_callback) {
        g_log_callback(0, buffer);
    } else {
        printf("[EDITOR] %s\n", buffer);
    }
}

// Error handling
static void set_last_error(EditorResult error) {
    g_last_error = error;
}

// Library initialization
EDITOR_API EditorResult editor_library_init(void) {
    EditorAllocator default_allocator = {
        .malloc_fn = default_malloc,
        .free_fn = default_free,
        .realloc_fn = default_realloc
    };
    return editor_library_init_with_allocator(&default_allocator);
}

EDITOR_API EditorResult editor_library_init_with_allocator(const EditorAllocator* allocator) {
    if (g_initialized) {
        return EDITOR_SUCCESS; // Already initialized
    }
    
    if (!allocator || !allocator->malloc_fn || !allocator->free_fn) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    g_allocator = *allocator;
    g_initialized = true;
    g_last_error = EDITOR_SUCCESS;
    
    log_debug("Editor library initialized");
    return EDITOR_SUCCESS;
}

EDITOR_API void editor_library_cleanup(void) {
    if (!g_initialized) return;
    
    g_initialized = false;
    g_last_error = EDITOR_SUCCESS;
    memset(&g_allocator, 0, sizeof(g_allocator));
    
    log_debug("Editor library cleaned up");
}

// Version information
EDITOR_API void editor_get_version(int* major, int* minor, int* patch) {
    if (major) *major = EDITOR_ABI_VERSION_MAJOR;
    if (minor) *minor = EDITOR_ABI_VERSION_MINOR;
    if (patch) *patch = EDITOR_ABI_VERSION_PATCH;
}

EDITOR_API const char* editor_get_version_string(void) {
    static char version_str[32];
    snprintf(version_str, sizeof(version_str), "%d.%d.%d",
             EDITOR_ABI_VERSION_MAJOR, EDITOR_ABI_VERSION_MINOR, EDITOR_ABI_VERSION_PATCH);
    return version_str;
}

// Document parsing - stateless API
EDITOR_API EditorResult editor_parse_markdown(const char* markdown, char** out_json) {
    if (!g_initialized) {
        set_last_error(EDITOR_ERROR_NOT_INITIALIZED);
        return EDITOR_ERROR_NOT_INITIALIZED;
    }
    
    if (!markdown || !out_json) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    Document doc = {0};
    int result = markdown_to_json(markdown, &doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_PARSE_FAILED);
        return EDITOR_ERROR_PARSE_FAILED;
    }
    
    result = json_stringify(&doc, out_json);
    doc_free(&doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_EXPORT_FAILED);
        return EDITOR_ERROR_EXPORT_FAILED;
    }
    
    log_debug("Parsed markdown to JSON (%zu chars)", strlen(*out_json));
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_export_markdown(const char* json, char** out_markdown) {
    if (!g_initialized) {
        set_last_error(EDITOR_ERROR_NOT_INITIALIZED);
        return EDITOR_ERROR_NOT_INITIALIZED;
    }
    
    if (!json || !out_markdown) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    Document doc = {0};
    int result = json_parse(json, &doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_PARSE_FAILED);
        return EDITOR_ERROR_PARSE_FAILED;
    }
    
    result = json_to_markdown(&doc, out_markdown);
    doc_free(&doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_EXPORT_FAILED);
        return EDITOR_ERROR_EXPORT_FAILED;
    }
    
    log_debug("Exported JSON to markdown (%zu chars)", strlen(*out_markdown));
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_export_json_canonical(const char* json, char** out_canonical) {
    if (!g_initialized) {
        set_last_error(EDITOR_ERROR_NOT_INITIALIZED);
        return EDITOR_ERROR_NOT_INITIALIZED;
    }
    
    if (!json || !out_canonical) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    Document doc = {0};
    int result = json_parse(json, &doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_PARSE_FAILED);
        return EDITOR_ERROR_PARSE_FAILED;
    }
    
    result = json_stringify(&doc, out_canonical);
    doc_free(&doc);
    
    if (result != 0) {
        set_last_error(EDITOR_ERROR_EXPORT_FAILED);
        return EDITOR_ERROR_EXPORT_FAILED;
    }
    
    return EDITOR_SUCCESS;
}

// Editor state management
struct EditorState {
    Document document;
    bool initialized;
};

EDITOR_API EditorState* editor_state_create(void) {
    if (!g_initialized) {
        set_last_error(EDITOR_ERROR_NOT_INITIALIZED);
        return NULL;
    }
    
    EditorState* state = g_allocator.malloc_fn(sizeof(EditorState));
    if (!state) {
        set_last_error(EDITOR_ERROR_OUT_OF_MEMORY);
        return NULL;
    }
    
    editor_init(&state->document);
    
    state->initialized = true;
    log_debug("Created editor state");
    return state;
}

EDITOR_API void editor_state_destroy(EditorState* state) {
    if (!state) return;
    
    if (state->initialized) {
        doc_free(&state->document);
    }
    
    g_allocator.free_fn(state);
    log_debug("Destroyed editor state");
}

EDITOR_API EditorResult editor_state_reset(EditorState* state) {
    if (!state || !state->initialized) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    doc_free(&state->document);
    editor_init(&state->document);
    
    return EDITOR_SUCCESS;
}

// Character input simulation
EDITOR_API EditorResult editor_state_input_char(EditorState* state, int32_t char_code) {
    if (!state || !state->initialized) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    editor_feed_char(&state->document, (unsigned)char_code);
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_input_string(EditorState* state, const char* text) {
    if (!state || !state->initialized || !text) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    for (const char* c = text; *c; c++) {
        editor_feed_char(&state->document, (unsigned)*c);
    }
    
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_backspace(EditorState* state) {
    if (!state || !state->initialized) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    // Simulate backspace (ASCII 8)
    editor_feed_char(&state->document, 8);
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_delete(EditorState* state) {
    if (!state || !state->initialized) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    // Simulate delete (ASCII 127)
    editor_feed_char(&state->document, 127);
    return EDITOR_SUCCESS;
}

// Document retrieval
EDITOR_API EditorResult editor_state_get_document(EditorState* state, char** out_json) {
    if (!state || !state->initialized || !out_json) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    int result = json_stringify(&state->document, out_json);
    if (result != 0) {
        set_last_error(EDITOR_ERROR_EXPORT_FAILED);
        return EDITOR_ERROR_EXPORT_FAILED;
    }
    
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_get_markdown(EditorState* state, char** out_markdown) {
    if (!state || !state->initialized || !out_markdown) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    char* json = NULL;
    EditorResult result = editor_state_get_document(state, &json);
    if (result != EDITOR_SUCCESS) {
        return result;
    }
    
    result = editor_export_markdown(json, out_markdown);
    editor_free_string(json);
    
    return result;
}

// Memory management
EDITOR_API void editor_free_string(char* str) {
    if (str) {
        g_allocator.free_fn(str);
    }
}

// Error handling
EDITOR_API const char* editor_get_error_message(EditorResult result) {
    switch (result) {
        case EDITOR_SUCCESS:
            return "Success";
        case EDITOR_ERROR_INVALID_PARAMETER:
            return "Invalid parameter";
        case EDITOR_ERROR_OUT_OF_MEMORY:
            return "Out of memory";
        case EDITOR_ERROR_PARSE_FAILED:
            return "Parse failed";
        case EDITOR_ERROR_EXPORT_FAILED:
            return "Export failed";
        case EDITOR_ERROR_NOT_INITIALIZED:
            return "Library not initialized";
        default:
            return "Unknown error";
    }
}

EDITOR_API EditorResult editor_get_last_error(void) {
    return g_last_error;
}

EDITOR_API void editor_clear_last_error(void) {
    g_last_error = EDITOR_SUCCESS;
}

// Debug and diagnostics
EDITOR_API void editor_enable_debug_logging(bool enabled) {
    g_debug_enabled = enabled;
    log_debug("Debug logging %s", enabled ? "enabled" : "disabled");
}

EDITOR_API void editor_set_log_callback(void (*callback)(int level, const char* message)) {
    g_log_callback = callback;
}

// Validation
EDITOR_API bool editor_is_valid_markdown(const char* markdown) {
    if (!markdown) return false;
    
    Document doc = {0};
    int result = markdown_to_json(markdown, &doc);
    if (result == 0) {
        doc_free(&doc);
        return true;
    }
    return false;
}

EDITOR_API bool editor_is_valid_json(const char* json) {
    if (!json) return false;
    
    Document doc = {0};
    int result = json_parse(json, &doc);
    if (result == 0) {
        doc_free(&doc);
        return true;
    }
    return false;
}

// Utility functions
EDITOR_API size_t editor_estimate_json_size(const char* markdown) {
    if (!markdown) return 0;
    // Rough estimate: JSON is typically 2-3x markdown size
    return strlen(markdown) * 3;
}

EDITOR_API size_t editor_estimate_markdown_size(const char* json) {
    if (!json) return 0;
    // Rough estimate: Markdown is typically 1/2 to 2/3 of JSON size
    return strlen(json) * 2 / 3;
}

// Feature detection
EDITOR_API uint32_t editor_get_features(void) {
    uint32_t features = 0;
    
    features |= EDITOR_FEATURE_TABLES;
    features |= EDITOR_FEATURE_IMAGES;
    features |= EDITOR_FEATURE_INLINE_STYLES;
    features |= EDITOR_FEATURE_HEADERS;
    features |= EDITOR_FEATURE_COLORS;
    
#ifdef EDITOR_THREAD_SAFE
    features |= EDITOR_FEATURE_THREAD_SAFE;
#endif
    
    return features;
}

EDITOR_API bool editor_has_feature(EditorFeatures feature) {
    return (editor_get_features() & feature) != 0;
}

// Configuration
EDITOR_API EditorResult editor_set_config(const EditorConfig* config) {
    if (!config) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    g_config = *config;
    log_debug("Configuration updated");
    return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_get_config(EditorConfig* config) {
    if (!config) {
        set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
        return EDITOR_ERROR_INVALID_PARAMETER;
    }
    
    *config = g_config;
    return EDITOR_SUCCESS;
}
