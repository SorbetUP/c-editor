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

typedef struct {
  char *data;
  size_t len;
  size_t capacity;
} HtmlBuilder;

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

static bool html_builder_reserve(HtmlBuilder *builder, size_t extra) {
  size_t needed = builder->len + extra + 1;
  if (needed <= builder->capacity) {
    return true;
  }

  size_t new_capacity = builder->capacity == 0 ? 128 : builder->capacity;
  while (new_capacity < needed) {
    new_capacity *= 2;
  }

  char *new_data = g_allocator.realloc_fn(builder->data, new_capacity);
  if (!new_data) {
    return false;
  }

  builder->data = new_data;
  builder->capacity = new_capacity;
  return true;
}

static bool html_builder_append_n(HtmlBuilder *builder, const char *text,
                                  size_t len) {
  if (!text || len == 0) {
    return true;
  }

  if (!html_builder_reserve(builder, len)) {
    return false;
  }

  memcpy(builder->data + builder->len, text, len);
  builder->len += len;
  builder->data[builder->len] = '\0';
  return true;
}

static bool html_builder_append(HtmlBuilder *builder, const char *text) {
  return html_builder_append_n(builder, text, text ? strlen(text) : 0);
}

static bool html_builder_append_char(HtmlBuilder *builder, char ch) {
  return html_builder_append_n(builder, &ch, 1);
}

static bool html_builder_append_escaped(HtmlBuilder *builder, const char *text) {
  if (!text) {
    return true;
  }

  for (const unsigned char *p = (const unsigned char *)text; *p; ++p) {
    switch (*p) {
    case '&':
      if (!html_builder_append(builder, "&amp;")) {
        return false;
      }
      break;
    case '<':
      if (!html_builder_append(builder, "&lt;")) {
        return false;
      }
      break;
    case '>':
      if (!html_builder_append(builder, "&gt;")) {
        return false;
      }
      break;
    case '"':
      if (!html_builder_append(builder, "&quot;")) {
        return false;
      }
      break;
    case '\'':
      if (!html_builder_append(builder, "&#39;")) {
        return false;
      }
      break;
    default:
      if (!html_builder_append_char(builder, (char)*p)) {
        return false;
      }
      break;
    }
  }

  return true;
}

static bool html_builder_appendf(HtmlBuilder *builder, const char *format, ...) {
  va_list args;
  va_start(args, format);
  va_list copy;
  va_copy(copy, args);
  int needed = vsnprintf(NULL, 0, format, copy);
  va_end(copy);

  if (needed < 0) {
    va_end(args);
    return false;
  }

  if (!html_builder_reserve(builder, (size_t)needed)) {
    va_end(args);
    return false;
  }

  vsnprintf(builder->data + builder->len, builder->capacity - builder->len,
            format, args);
  va_end(args);
  builder->len += (size_t)needed;
  return true;
}

static bool render_text_inline_html(HtmlBuilder *builder,
                                    const ElementText *text);

static bool render_text_span_html(HtmlBuilder *builder, const TextSpan *span) {
  TextSpan temp = {0};
  if (!span) {
    span = &temp;
  }

  if (span->is_image && span->image_src) {
    if (!html_builder_append(builder, "<img src=\"") ||
        !html_builder_append_escaped(builder, span->image_src) ||
        !html_builder_append(builder, "\" alt=\"") ||
        !html_builder_append_escaped(builder,
                                     span->image_alt ? span->image_alt : "") ||
        !html_builder_append(builder, "\" />")) {
      return false;
    }
    return true;
  }

  if (span->is_link && span->link_href) {
    TextSpan inner = *span;
    inner.is_link = false;
    inner.link_href = NULL;
    inner.is_note_link = false;
    inner.is_image = false;
    inner.image_src = NULL;
    inner.image_alt = NULL;

    if (!html_builder_append(builder, "<a href=\"") ||
        !html_builder_append_escaped(builder, span->link_href) ||
        !html_builder_append(builder, "\">") ||
        !render_text_span_html(builder, &inner) ||
        !html_builder_append(builder, "</a>")) {
      return false;
    }
    return true;
  }

  if (span->strikethrough && !html_builder_append(builder, "<del>")) {
    return false;
  }
  if (span->has_highlight && !html_builder_append(builder, "<mark>")) {
    return false;
  }
  if (span->has_underline && !html_builder_append(builder, "<u>")) {
    return false;
  }
  if (span->bold && !html_builder_append(builder, "<strong>")) {
    return false;
  }
  if (span->italic && !html_builder_append(builder, "<em>")) {
    return false;
  }
  if (span->code && !html_builder_append(builder, "<code>")) {
    return false;
  }

  if (!html_builder_append_escaped(builder, span->text ? span->text : "")) {
    return false;
  }

  if (span->code && !html_builder_append(builder, "</code>")) {
    return false;
  }
  if (span->italic && !html_builder_append(builder, "</em>")) {
    return false;
  }
  if (span->bold && !html_builder_append(builder, "</strong>")) {
    return false;
  }
  if (span->has_underline && !html_builder_append(builder, "</u>")) {
    return false;
  }
  if (span->has_highlight && !html_builder_append(builder, "</mark>")) {
    return false;
  }
  if (span->strikethrough && !html_builder_append(builder, "</del>")) {
    return false;
  }

  return true;
}

static bool render_text_inline_html(HtmlBuilder *builder,
                                    const ElementText *text) {
  if (!text) {
    return true;
  }

  if (text->spans && text->spans_count > 0) {
    for (size_t i = 0; i < text->spans_count; i++) {
      if (!render_text_span_html(builder, &text->spans[i])) {
        return false;
      }
    }
    return true;
  }

  TextSpan span = {0};
  span.text = text->text;
  span.bold = text->bold;
  span.italic = text->italic;
  span.has_highlight = text->has_highlight;
  span.highlight_color = text->highlight_color;
  span.has_underline = text->has_underline;
  span.underline_color = text->underline_color;
  span.underline_gap = text->underline_gap;

  return render_text_span_html(builder, &span);
}

static bool render_text_element_html(HtmlBuilder *builder,
                                     const ElementText *text) {
  if (!text) {
    return true;
  }

  if (text->level > 0 && text->level <= 6) {
    if (!html_builder_appendf(builder, "<h%d>", text->level) ||
        !render_text_inline_html(builder, text) ||
        !html_builder_appendf(builder, "</h%d>", text->level)) {
      return false;
    }
    return true;
  }

  return render_text_inline_html(builder, text);
}

static bool render_list_html(HtmlBuilder *builder, const ElementList *list) {
  if (!list) {
    return true;
  }

  bool is_task_list = list->kind == LIST_KIND_TASK;
  if (!is_task_list) {
    for (size_t i = 0; i < list->item_count; i++) {
      if (list->items[i].has_checkbox || list->items[i].is_task) {
        is_task_list = true;
        break;
      }
    }
  }

  if (list->kind == LIST_KIND_DEFINITION) {
    if (!html_builder_append(builder, "<dl>")) {
      return false;
    }
    for (size_t i = 0; i < list->item_count; i++) {
      const ElementListItem *item = &list->items[i];
      if (!html_builder_append(builder, "<dt>") ||
          !render_text_inline_html(builder, &item->term) ||
          !html_builder_append(builder, "</dt><dd>") ||
          !render_text_inline_html(builder, &item->definition) ||
          !html_builder_append(builder, "</dd>")) {
        return false;
      }
    }
    return html_builder_append(builder, "</dl>");
  }

  if (list->ordered) {
    if (!html_builder_appendf(builder, "<ol start=\"%d\">", list->start_index)) {
      return false;
    }
  } else if (is_task_list) {
    if (!html_builder_append(builder, "<ul class=\"task-list\">")) {
      return false;
    }
  } else if (!html_builder_append(builder, "<ul>")) {
    return false;
  }

  for (size_t i = 0; i < list->item_count; i++) {
    const ElementListItem *item = &list->items[i];
    if (!html_builder_append(builder, "<li>")) {
      return false;
    }
    if (item->has_checkbox) {
      if (!html_builder_append(builder,
                               item->checkbox_checked
                                   ? "<input type=\"checkbox\" checked disabled /> "
                                   : "<input type=\"checkbox\" disabled /> ")) {
        return false;
      }
    }
    if (!render_text_inline_html(builder, &item->text) ||
        !html_builder_append(builder, "</li>")) {
      return false;
    }
  }

  return html_builder_append(builder, list->ordered ? "</ol>" : "</ul>");
}

static bool render_quote_html(HtmlBuilder *builder, const ElementQuote *quote) {
  if (!quote) {
    return true;
  }

  if (!html_builder_append(builder, "<blockquote>")) {
    return false;
  }

  for (size_t i = 0; i < quote->item_count; i++) {
    if (i > 0 && !html_builder_append(builder, "<br />")) {
      return false;
    }
    if (!render_text_inline_html(builder, &quote->items[i])) {
      return false;
    }
  }

  return html_builder_append(builder, "</blockquote>");
}

static bool render_table_html(HtmlBuilder *builder, const ElementTable *table) {
  if (!table || table->rows == 0 || table->cols == 0) {
    return true;
  }

  size_t header_rows = 0;
  if (table->header_rows > 0 && table->header_rows <= table->rows) {
    header_rows = table->header_rows;
  } else {
    header_rows = 1;
  }

  if (!html_builder_append(builder, "<table>")) {
    return false;
  }

  if (header_rows > 0) {
    if (!html_builder_append(builder, "<thead>")) {
      return false;
    }
    for (size_t r = 0; r < header_rows; r++) {
      if (!html_builder_append(builder, "<tr>")) {
        return false;
      }
      for (size_t c = 0; c < table->cols; c++) {
        if (!html_builder_append(builder, "<th>")) {
          return false;
        }
        if (table->cells[r] && table->cells[r][c] &&
            !render_text_inline_html(builder, table->cells[r][c])) {
          return false;
        }
        if (!html_builder_append(builder, "</th>")) {
          return false;
        }
      }
      if (!html_builder_append(builder, "</tr>")) {
        return false;
      }
    }
    if (!html_builder_append(builder, "</thead>")) {
      return false;
    }
  }

  if (!html_builder_append(builder, "<tbody>")) {
    return false;
  }
  for (size_t r = header_rows; r < table->rows; r++) {
    if (!html_builder_append(builder, "<tr>")) {
      return false;
    }
    for (size_t c = 0; c < table->cols; c++) {
      if (!html_builder_append(builder, "<td>")) {
        return false;
      }
      if (table->cells[r] && table->cells[r][c] &&
          !render_text_inline_html(builder, table->cells[r][c])) {
        return false;
      }
      if (!html_builder_append(builder, "</td>")) {
        return false;
      }
    }
    if (!html_builder_append(builder, "</tr>")) {
      return false;
    }
  }
  if (!html_builder_append(builder, "</tbody></table>")) {
    return false;
  }

  return true;
}

static bool render_element_html(HtmlBuilder *builder, const Element *element) {
  if (!element) {
    return true;
  }

  switch (element->kind) {
  case T_TEXT:
    return render_text_element_html(builder, &element->as.text);
  case T_IMAGE:
    return html_builder_append(builder, "<img src=\"") &&
           html_builder_append_escaped(builder,
                                       element->as.image.src
                                           ? element->as.image.src
                                           : "") &&
           html_builder_append(builder, "\" alt=\"") &&
           html_builder_append_escaped(builder,
                                       element->as.image.alt
                                           ? element->as.image.alt
                                           : "") &&
           html_builder_append(builder, "\" />");
  case T_TABLE:
    return render_table_html(builder, &element->as.table);
  case T_CODE:
    if (!html_builder_append(builder, "<pre><code")) {
      return false;
    }
    if (element->as.code.language && element->as.code.language[0] != '\0') {
      if (!html_builder_append(builder, " class=\"language-") ||
          !html_builder_append_escaped(builder, element->as.code.language) ||
          !html_builder_append(builder, "\"")) {
        return false;
      }
    }
    return html_builder_append(builder, ">") &&
           html_builder_append_escaped(
               builder,
               element->as.code.content ? element->as.code.content : "") &&
           html_builder_append(builder, "</code></pre>");
  case T_LIST:
    return render_list_html(builder, &element->as.list);
  case T_QUOTE:
    return render_quote_html(builder, &element->as.quote);
  case T_DIVIDER:
    return html_builder_append(builder, "<hr />");
  case T_SETTINGS:
    return html_builder_append(builder, "<div class=\"settings\">") &&
           html_builder_append_escaped(
               builder,
               element->as.settings.name ? element->as.settings.name : "") &&
           html_builder_append(builder, "=") &&
           html_builder_append_escaped(
               builder,
               element->as.settings.value ? element->as.settings.value : "") &&
           html_builder_append(builder, "</div>");
  default:
    return false;
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

  Document doc = {0};
  if (markdown_to_json(markdown, &doc) != 0) {
    printf("[EDITOR ERROR] Failed to parse markdown\n");
    return NULL;
  }

  HtmlBuilder builder = {0};
  if (!html_builder_reserve(&builder, 128)) {
    doc_free(&doc);
    return NULL;
  }
  builder.data[0] = '\0';

  for (size_t i = 0; i < doc.elements_len; i++) {
    if (i > 0 && !html_builder_append(&builder, "\n")) {
      g_allocator.free_fn(builder.data);
      doc_free(&doc);
      return NULL;
    }
    if (!render_element_html(&builder, &doc.elements[i])) {
      g_allocator.free_fn(builder.data);
      doc_free(&doc);
      return NULL;
    }
  }

  doc_free(&doc);
  last_html_result = builder.data;
  log_debug("Generated HTML result: %s",
            last_html_result ? last_html_result : "(null)");
  return last_html_result;
}
