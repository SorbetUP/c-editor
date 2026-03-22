// hybrid_editor_core.h - Core hybrid editor logic library
// Platform-agnostic hybrid editor functionality

#ifndef HYBRID_EDITOR_CORE_H
#define HYBRID_EDITOR_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Export macros for different platforms
#ifdef _WIN32
#ifdef BUILDING_HYBRID_DLL
#define HYBRID_API __declspec(dllexport)
#else
#define HYBRID_API __declspec(dllimport)
#endif
#else
#define HYBRID_API __attribute__((visibility("default")))
#endif

// Text position and range structures
typedef struct {
    int start;
    int end;
} HybridTextRange;

typedef struct {
    int line_index;
    int char_start;
    int char_end;
    int length;
} LineInfo;

typedef struct {
    LineInfo* lines;
    int line_count;
    int total_length;
} TextLines;

// Markdown format detection
typedef enum {
    MD_FORMAT_NONE = 0,
    MD_FORMAT_BOLD = 1,
    MD_FORMAT_ITALIC = 2, 
    MD_FORMAT_HIGHLIGHT = 4,
    MD_FORMAT_HEADER1 = 8,
    MD_FORMAT_HEADER2 = 16,
    MD_FORMAT_HEADER3 = 32,
    MD_FORMAT_LIST = 64
} MarkdownFormat;

typedef struct {
    MarkdownFormat format;
    HybridTextRange range;
    HybridTextRange content_range; // Range of actual content (without markup)
} FormatInfo;

typedef struct {
    FormatInfo* formats;
    int format_count;
} LineFormats;

// Core hybrid editor functions
HYBRID_API TextLines* hybrid_parse_text(const char* text);
HYBRID_API void hybrid_free_text_lines(TextLines* lines);

HYBRID_API int hybrid_get_line_at_cursor(const TextLines* lines, int cursor_pos);
HYBRID_API LineInfo* hybrid_get_line_info(const TextLines* lines, int line_index);

HYBRID_API bool hybrid_should_render_line(int line_index, int current_line);
HYBRID_API bool hybrid_is_current_line(int line_index, int current_line);

// Markdown analysis functions
HYBRID_API LineFormats* hybrid_analyze_markdown_line(const char* line);
HYBRID_API void hybrid_free_line_formats(LineFormats* formats);

HYBRID_API MarkdownFormat hybrid_detect_line_format(const char* line);
HYBRID_API bool hybrid_line_contains_format(const char* line, MarkdownFormat format);

// Text manipulation for hiding markup
HYBRID_API char* hybrid_strip_markdown_markup(const char* line);
HYBRID_API HybridTextRange* hybrid_find_markup_ranges(const char* line, int* range_count);

// HTML integration (uses existing editor_markdown_to_html)
HYBRID_API MarkdownFormat hybrid_detect_format_from_html(const char* html);
HYBRID_API bool hybrid_html_contains_tag(const char* html, const char* tag);

// Utility functions
HYBRID_API char* hybrid_get_line_content(const char* text, int line_index);
HYBRID_API int hybrid_count_lines(const char* text);
HYBRID_API HybridTextRange hybrid_get_word_at_position(const char* text, int position);

// Configuration
typedef struct {
    bool enable_bold;
    bool enable_italic;
    bool enable_highlight;
    bool enable_headers;
    bool enable_lists;
    bool strict_markdown;
    int max_line_length;
} HybridConfig;

HYBRID_API void hybrid_set_config(const HybridConfig* config);
HYBRID_API HybridConfig hybrid_get_config(void);

// Error handling
typedef enum {
    HYBRID_SUCCESS = 0,
    HYBRID_ERROR_NULL_POINTER = -1,
    HYBRID_ERROR_INVALID_LINE = -2,
    HYBRID_ERROR_OUT_OF_MEMORY = -3,
    HYBRID_ERROR_INVALID_FORMAT = -4
} HybridResult;

HYBRID_API const char* hybrid_get_error_message(HybridResult result);

#ifdef __cplusplus
}
#endif

#endif // HYBRID_EDITOR_CORE_H