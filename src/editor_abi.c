// editor_abi.c - Implementation of stable ABI interface
#include "editor_abi.h"
#include "editor.h"
#include "json.h"
#include "markdown.h"
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Global state
static bool g_initialized = false;
static EditorResult g_last_error = EDITOR_SUCCESS;
static EditorAllocator g_allocator = {0};
static bool g_debug_enabled = false;
static void (*g_log_callback)(int, const char *) = NULL;
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
static void *default_malloc(size_t size) { return malloc(size); }

static void default_free(void *ptr) { free(ptr); }

static void *default_realloc(void *ptr, size_t size) {
  return realloc(ptr, size);
}

// Logging helper
static void log_debug(const char *format, ...) {
  if (!g_debug_enabled)
    return;

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
static void set_last_error(EditorResult error) { g_last_error = error; }

// Library initialization
EDITOR_API EditorResult editor_library_init(void) {
  EditorAllocator default_allocator = {.malloc_fn = default_malloc,
                                       .free_fn = default_free,
                                       .realloc_fn = default_realloc};
  return editor_library_init_with_allocator(&default_allocator);
}

EDITOR_API EditorResult
editor_library_init_with_allocator(const EditorAllocator *allocator) {
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
  if (!g_initialized)
    return;

  g_initialized = false;
  g_last_error = EDITOR_SUCCESS;
  memset(&g_allocator, 0, sizeof(g_allocator));

  log_debug("Editor library cleaned up");
}

// Version information
EDITOR_API void editor_get_version(int *major, int *minor, int *patch) {
  if (major)
    *major = EDITOR_ABI_VERSION_MAJOR;
  if (minor)
    *minor = EDITOR_ABI_VERSION_MINOR;
  if (patch)
    *patch = EDITOR_ABI_VERSION_PATCH;
}

EDITOR_API const char *editor_get_version_string(void) {
  static char version_str[32];
  snprintf(version_str, sizeof(version_str), "%d.%d.%d",
           EDITOR_ABI_VERSION_MAJOR, EDITOR_ABI_VERSION_MINOR,
           EDITOR_ABI_VERSION_PATCH);
  return version_str;
}

// Document parsing - stateless API
EDITOR_API EditorResult editor_parse_markdown(const char *markdown,
                                              char **out_json) {
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

// Simple wrapper for WASM that returns JSON directly
EDITOR_API const char *editor_parse_markdown_simple(const char *markdown) {
  log_debug("editor_parse_markdown_simple called with: %s", markdown ? markdown : "NULL");
  
  static char *last_result = NULL;
  
  // Libérer le résultat précédent
  if (last_result) {
    log_debug("Freeing previous result");
    g_allocator.free_fn(last_result);
    last_result = NULL;
  }
  
  if (!markdown) {
    printf("[EDITOR ERROR] Input markdown is NULL\n");
    return NULL;
  }
  
  char *json_result = NULL;
  EditorResult result = editor_parse_markdown(markdown, &json_result);
  
  log_debug("editor_parse_markdown returned: %d", result);
  log_debug("json_result pointer: %p", (void*)json_result);
  
  if (result == EDITOR_SUCCESS && json_result) {
    last_result = json_result;
    log_debug("Returning JSON result of length: %zu", strlen(json_result));
    return json_result;
  }
  
  printf("[EDITOR ERROR] Failed to parse markdown: result=%d, json_result=%p\n", result, (void*)json_result);
  return NULL;
}

EDITOR_API EditorResult editor_export_markdown(const char *json,
                                               char **out_markdown) {
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

EDITOR_API EditorResult editor_export_json_canonical(const char *json,
                                                     char **out_canonical) {
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

EDITOR_API EditorState *editor_state_create(void) {
  if (!g_initialized) {
    set_last_error(EDITOR_ERROR_NOT_INITIALIZED);
    return NULL;
  }

  EditorState *state = g_allocator.malloc_fn(sizeof(EditorState));
  if (!state) {
    set_last_error(EDITOR_ERROR_OUT_OF_MEMORY);
    return NULL;
  }

  editor_init(&state->document);

  state->initialized = true;
  log_debug("Created editor state");
  return state;
}

EDITOR_API void editor_state_destroy(EditorState *state) {
  if (!state)
    return;

  if (state->initialized) {
    doc_free(&state->document);
  }

  g_allocator.free_fn(state);
  log_debug("Destroyed editor state");
}

EDITOR_API EditorResult editor_state_reset(EditorState *state) {
  if (!state || !state->initialized) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  doc_free(&state->document);
  editor_init(&state->document);

  return EDITOR_SUCCESS;
}

// Character input simulation
EDITOR_API EditorResult editor_state_input_char(EditorState *state,
                                                int32_t char_code) {
  if (!state || !state->initialized) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  editor_feed_char(&state->document, (unsigned)char_code);
  return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_input_string(EditorState *state,
                                                  const char *text) {
  if (!state || !state->initialized || !text) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  for (const char *c = text; *c; c++) {
    editor_feed_char(&state->document, (unsigned)*c);
  }

  return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_backspace(EditorState *state) {
  if (!state || !state->initialized) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  // Simulate backspace (ASCII 8)
  editor_feed_char(&state->document, 8);
  return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_state_delete(EditorState *state) {
  if (!state || !state->initialized) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  // Simulate delete (ASCII 127)
  editor_feed_char(&state->document, 127);
  return EDITOR_SUCCESS;
}

// Document retrieval
EDITOR_API EditorResult editor_state_get_document(EditorState *state,
                                                  char **out_json) {
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

EDITOR_API EditorResult editor_state_get_markdown(EditorState *state,
                                                  char **out_markdown) {
  if (!state || !state->initialized || !out_markdown) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  char *json = NULL;
  EditorResult result = editor_state_get_document(state, &json);
  if (result != EDITOR_SUCCESS) {
    return result;
  }

  result = editor_export_markdown(json, out_markdown);
  editor_free_string(json);

  return result;
}

// Memory management
EDITOR_API void editor_free_string(char *str) {
  if (str) {
    g_allocator.free_fn(str);
  }
}

// Error handling
EDITOR_API const char *editor_get_error_message(EditorResult result) {
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

EDITOR_API EditorResult editor_get_last_error(void) { return g_last_error; }

EDITOR_API void editor_clear_last_error(void) { g_last_error = EDITOR_SUCCESS; }

// Debug and diagnostics
EDITOR_API void editor_enable_debug_logging(bool enabled) {
  g_debug_enabled = enabled;
  log_debug("Debug logging %s", enabled ? "enabled" : "disabled");
}

EDITOR_API void editor_set_log_callback(void (*callback)(int level,
                                                         const char *message)) {
  g_log_callback = callback;
}

// Validation
EDITOR_API bool editor_is_valid_markdown(const char *markdown) {
  if (!markdown)
    return false;

  Document doc = {0};
  int result = markdown_to_json(markdown, &doc);
  if (result == 0) {
    doc_free(&doc);
    return true;
  }
  return false;
}

EDITOR_API bool editor_is_valid_json(const char *json) {
  if (!json)
    return false;

  Document doc = {0};
  int result = json_parse(json, &doc);
  if (result == 0) {
    doc_free(&doc);
    return true;
  }
  return false;
}

// Utility functions
EDITOR_API size_t editor_estimate_json_size(const char *markdown) {
  if (!markdown)
    return 0;
  // Rough estimate: JSON is typically 2-3x markdown size
  return strlen(markdown) * 3;
}

EDITOR_API size_t editor_estimate_markdown_size(const char *json) {
  if (!json)
    return 0;
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
EDITOR_API EditorResult editor_set_config(const EditorConfig *config) {
  if (!config) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  g_config = *config;
  log_debug("Configuration updated");
  return EDITOR_SUCCESS;
}

EDITOR_API EditorResult editor_get_config(EditorConfig *config) {
  if (!config) {
    set_last_error(EDITOR_ERROR_INVALID_PARAMETER);
    return EDITOR_ERROR_INVALID_PARAMETER;
  }

  *config = g_config;
  return EDITOR_SUCCESS;
}

// Convert markdown directly to HTML - WASM optimized
EDITOR_API const char *editor_markdown_to_html(const char *markdown) {
  log_debug("editor_markdown_to_html called with: %s", markdown ? markdown : "NULL");
  
  static char *last_html_result = NULL;
  
  // Libérer le résultat précédent
  if (last_html_result) {
    log_debug("Freeing previous HTML result");
    g_allocator.free_fn(last_html_result);
    last_html_result = NULL;
  }
  
  if (!markdown || !g_initialized) {
    printf("[EDITOR ERROR] Invalid input or not initialized\n");
    return NULL;
  }
  
  // Parse to JSON first
  char *json_result = NULL;
  EditorResult result = editor_parse_markdown(markdown, &json_result);
  
  if (result != EDITOR_SUCCESS || !json_result) {
    printf("[EDITOR ERROR] Failed to parse markdown to JSON\n");
    return NULL;
  }
  
  // Allocate HTML buffer
  size_t html_len = strlen(json_result) * 3 + 1024; // Large estimate
  char *html_buffer = (char*)g_allocator.malloc_fn(html_len);
  if (!html_buffer) {
    g_allocator.free_fn(json_result);
    return NULL;
  }
  
  // Simple approach: extract text and spans from JSON
  html_buffer[0] = '\0';
  char *html_pos = html_buffer;
  size_t remaining = html_len - 1;
  
  // Find spans array in JSON
  const char *spans_start = strstr(json_result, "\"spans\":[");
  if (spans_start) {
    spans_start += 9; // Skip "spans":[
    const char *current = spans_start;
    
    while (*current && *current != ']') {
      if (*current == '{') {
        // Parse span object
        const char *span_end = strchr(current, '}');
        if (!span_end) break;
        
        // Extract text
        const char *text_start = strstr(current, "\"text\":\"");
        if (text_start && text_start < span_end) {
          text_start += 8;
          const char *text_end = strchr(text_start, '"');
          if (text_end && text_end < span_end) {
            
            // Check formatting
            bool is_bold = strstr(current, "\"bold\":true") && strstr(current, "\"bold\":true") < span_end;
            bool is_italic = strstr(current, "\"italic\":true") && strstr(current, "\"italic\":true") < span_end;
            bool has_underline = strstr(current, "\"has_underline\":true") && strstr(current, "\"has_underline\":true") < span_end;
            bool has_highlight = strstr(current, "\"has_highlight\":true") && strstr(current, "\"has_highlight\":true") < span_end;
            
            // Open tags
            if (is_bold && remaining > 8) {
              strcpy(html_pos, "<strong>");
              html_pos += 8; remaining -= 8;
            }
            if (is_italic && remaining > 4) {
              strcpy(html_pos, "<em>");
              html_pos += 4; remaining -= 4;
            }
            if (has_underline && remaining > 3) {
              strcpy(html_pos, "<u>");
              html_pos += 3; remaining -= 3;
            }
            if (has_highlight && remaining > 6) {
              strcpy(html_pos, "<mark>");
              html_pos += 6; remaining -= 6;
            }
            
            // Copy text
            size_t text_len = text_end - text_start;
            if (text_len > 0 && remaining > text_len) {
              memcpy(html_pos, text_start, text_len);
              html_pos += text_len; remaining -= text_len;
            }
            
            // Close tags (reverse order)
            if (has_highlight && remaining > 7) {
              strcpy(html_pos, "</mark>");
              html_pos += 7; remaining -= 7;
            }
            if (has_underline && remaining > 4) {
              strcpy(html_pos, "</u>");
              html_pos += 4; remaining -= 4;
            }
            if (is_italic && remaining > 5) {
              strcpy(html_pos, "</em>");
              html_pos += 5; remaining -= 5;
            }
            if (is_bold && remaining > 9) {
              strcpy(html_pos, "</strong>");
              html_pos += 9; remaining -= 9;
            }
          }
        }
        current = span_end + 1;
      } else {
        current++;
      }
    }
  } else {
    // Fallback: just extract main text
    const char *text_start = strstr(json_result, "\"text\":\"");
    if (text_start) {
      text_start += 8;
      const char *text_end = strchr(text_start, '"');
      if (text_end) {
        size_t text_len = text_end - text_start;
        if (text_len > 0 && remaining > text_len) {
          memcpy(html_pos, text_start, text_len);
          html_pos += text_len;
        }
      }
    }
  }
  
  // Set null terminator FIRST
  *html_pos = '\0';
  
  // Check for header level
  const char *level_str = strstr(json_result, "\"level\":");
  if (level_str) {
    int level = atoi(level_str + 8);
    if (level > 0 && level <= 6) {
      // Wrap in header tags
      size_t current_len = strlen(html_buffer);
      if (current_len > 0 && current_len < html_len - 20) { // Ensure space for header tags
        char temp_buffer[html_len];
        strcpy(temp_buffer, html_buffer);
        snprintf(html_buffer, html_len, "<h%d>%s</h%d>", level, temp_buffer, level);
      }
    }
  }
  
  // Clean up
  g_allocator.free_fn(json_result);
  
  // Store result
  last_html_result = html_buffer;
  
  log_debug("Generated HTML result: %s", html_buffer);
  return html_buffer;
}
