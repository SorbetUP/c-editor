// hybrid_editor_core.c - Core hybrid editor logic implementation
#include "hybrid_editor_core.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Global configuration
static HybridConfig g_config = {
    .enable_bold = true,
    .enable_italic = true,
    .enable_highlight = true,
    .enable_headers = true,
    .enable_lists = true,
    .strict_markdown = true,
    .max_line_length = 10000
};

// Helper function to count lines in text
HYBRID_API int hybrid_count_lines(const char* text) {
    if (!text) return 0;
    
    int count = 1;
    const char* p = text;
    while (*p) {
        if (*p == '\n') count++;
        p++;
    }
    return count;
}

// Parse text into lines structure
HYBRID_API TextLines* hybrid_parse_text(const char* text) {
    if (!text) return NULL;
    
    TextLines* lines = malloc(sizeof(TextLines));
    if (!lines) return NULL;
    
    int line_count = hybrid_count_lines(text);
    lines->lines = malloc(sizeof(LineInfo) * line_count);
    if (!lines->lines) {
        free(lines);
        return NULL;
    }
    
    lines->line_count = line_count;
    lines->total_length = strlen(text);
    
    // Parse each line
    const char* line_start = text;
    int char_pos = 0;
    
    for (int i = 0; i < line_count; i++) {
        lines->lines[i].line_index = i;
        lines->lines[i].char_start = char_pos;
        
        // Find end of line
        const char* line_end = strchr(line_start, '\n');
        if (line_end) {
            lines->lines[i].length = line_end - line_start;
            lines->lines[i].char_end = char_pos + lines->lines[i].length;
            char_pos = lines->lines[i].char_end + 1; // +1 for \n
            line_start = line_end + 1;
        } else {
            // Last line without \n
            lines->lines[i].length = strlen(line_start);
            lines->lines[i].char_end = char_pos + lines->lines[i].length;
        }
    }
    
    return lines;
}

// Free text lines structure
HYBRID_API void hybrid_free_text_lines(TextLines* lines) {
    if (lines) {
        free(lines->lines);
        free(lines);
    }
}

// Get line index at cursor position
HYBRID_API int hybrid_get_line_at_cursor(const TextLines* lines, int cursor_pos) {
    if (!lines || cursor_pos < 0) return 0;
    
    for (int i = 0; i < lines->line_count; i++) {
        if (cursor_pos >= lines->lines[i].char_start && cursor_pos <= lines->lines[i].char_end) {
            return i;
        }
    }
    
    return lines->line_count - 1; // Default to last line
}

// Get line info by index
HYBRID_API LineInfo* hybrid_get_line_info(const TextLines* lines, int line_index) {
    if (!lines || line_index < 0 || line_index >= lines->line_count) {
        return NULL;
    }
    return &lines->lines[line_index];
}

// Check if line should be rendered (not current line)
HYBRID_API bool hybrid_should_render_line(int line_index, int current_line) {
    return line_index != current_line;
}

// Check if line is current line
HYBRID_API bool hybrid_is_current_line(int line_index, int current_line) {
    return line_index == current_line;
}

// Detect markdown format from line content
HYBRID_API MarkdownFormat hybrid_detect_line_format(const char* line) {
    if (!line) return MD_FORMAT_NONE;
    
    MarkdownFormat format = MD_FORMAT_NONE;
    
    // Headers
    if (g_config.enable_headers) {
        if (strncmp(line, "# ", 2) == 0) {
            format |= MD_FORMAT_HEADER1;
        } else if (strncmp(line, "## ", 3) == 0) {
            format |= MD_FORMAT_HEADER2;
        } else if (strncmp(line, "### ", 4) == 0) {
            format |= MD_FORMAT_HEADER3;
        }
    }
    
    // Lists
    if (g_config.enable_lists) {
        if (strncmp(line, "- ", 2) == 0 || strncmp(line, "* ", 2) == 0 || strncmp(line, "+ ", 2) == 0) {
            format |= MD_FORMAT_LIST;
        }
    }
    
    // Inline formats
    if (g_config.enable_bold && strstr(line, "**")) {
        format |= MD_FORMAT_BOLD;
    }
    if (g_config.enable_italic && strstr(line, "*")) {
        // Check if it's italic (single *) not part of bold (**)
        const char* pos = strstr(line, "*");
        while (pos) {
            if (pos > line && *(pos-1) == '*') {
                // Part of **
                pos = strstr(pos + 1, "*");
                continue;
            }
            if (*(pos+1) == '*') {
                // Start of **
                pos = strstr(pos + 2, "*");
                continue;
            }
            // Found single *, check for closing
            const char* close = strstr(pos + 1, "*");
            if (close && *(close+1) != '*') {
                format |= MD_FORMAT_ITALIC;
                break;
            }
            pos = strstr(pos + 1, "*");
        }
    }
    if (g_config.enable_highlight && strstr(line, "==")) {
        format |= MD_FORMAT_HIGHLIGHT;
    }
    
    return format;
}

// Check if line contains specific format
HYBRID_API bool hybrid_line_contains_format(const char* line, MarkdownFormat format) {
    MarkdownFormat detected = hybrid_detect_line_format(line);
    return (detected & format) != 0;
}

// Analyze line and return detailed format info
HYBRID_API LineFormats* hybrid_analyze_markdown_line(const char* line) {
    if (!line) return NULL;
    
    LineFormats* formats = malloc(sizeof(LineFormats));
    if (!formats) return NULL;
    
    // Start with reasonable capacity
    int capacity = 10;
    formats->formats = malloc(sizeof(FormatInfo) * capacity);
    formats->format_count = 0;
    
    if (!formats->formats) {
        free(formats);
        return NULL;
    }
    
    // int line_len = strlen(line); // Unused for now
    
    // Find bold **text**
    if (g_config.enable_bold) {
        const char* pos = line;
        while ((pos = strstr(pos, "**")) != NULL) {
            const char* end = strstr(pos + 2, "**");
            if (end) {
                if (formats->format_count >= capacity) {
                    capacity *= 2;
                    formats->formats = realloc(formats->formats, sizeof(FormatInfo) * capacity);
                }
                
                FormatInfo* info = &formats->formats[formats->format_count++];
                info->format = MD_FORMAT_BOLD;
                info->range.start = pos - line;
                info->range.end = (end + 2) - line;
                info->content_range.start = (pos + 2) - line;
                info->content_range.end = end - line;
                
                pos = end + 2;
            } else {
                break;
            }
        }
    }
    
    // Find italic *text* (avoiding bold)
    if (g_config.enable_italic) {
        const char* pos = line;
        while ((pos = strchr(pos, '*')) != NULL) {
            // Skip if part of bold
            if (pos > line && *(pos-1) == '*') {
                pos++;
                continue;
            }
            if (*(pos+1) == '*') {
                pos += 2;
                continue;
            }
            
            const char* end = strchr(pos + 1, '*');
            if (end && *(end+1) != '*') {
                if (formats->format_count >= capacity) {
                    capacity *= 2;
                    formats->formats = realloc(formats->formats, sizeof(FormatInfo) * capacity);
                }
                
                FormatInfo* info = &formats->formats[formats->format_count++];
                info->format = MD_FORMAT_ITALIC;
                info->range.start = pos - line;
                info->range.end = (end + 1) - line;
                info->content_range.start = (pos + 1) - line;
                info->content_range.end = end - line;
                
                pos = end + 1;
            } else {
                pos++;
            }
        }
    }
    
    // Find highlight ==text==
    if (g_config.enable_highlight) {
        const char* pos = line;
        while ((pos = strstr(pos, "==")) != NULL) {
            const char* end = strstr(pos + 2, "==");
            if (end) {
                if (formats->format_count >= capacity) {
                    capacity *= 2;
                    formats->formats = realloc(formats->formats, sizeof(FormatInfo) * capacity);
                }
                
                FormatInfo* info = &formats->formats[formats->format_count++];
                info->format = MD_FORMAT_HIGHLIGHT;
                info->range.start = pos - line;
                info->range.end = (end + 2) - line;
                info->content_range.start = (pos + 2) - line;
                info->content_range.end = end - line;
                
                pos = end + 2;
            } else {
                break;
            }
        }
    }
    
    return formats;
}

// Free line formats
HYBRID_API void hybrid_free_line_formats(LineFormats* formats) {
    if (formats) {
        free(formats->formats);
        free(formats);
    }
}

// Strip markdown markup from line
HYBRID_API char* hybrid_strip_markdown_markup(const char* line) {
    if (!line) return NULL;
    
    int len = strlen(line);
    char* result = malloc(len + 1);
    if (!result) return NULL;
    
    char* dest = result;
    const char* src = line;
    
    // Simple stripping - remove common markdown
    while (*src) {
        // Skip header markers
        if (src == line && *src == '#') {
            while (*src == '#' || *src == ' ') src++;
            continue;
        }
        
        // Skip bold markers
        if (*src == '*' && *(src+1) == '*') {
            src += 2;
            continue;
        }
        
        // Skip italic markers (single *)
        if (*src == '*' && *(src+1) != '*' && (src == line || *(src-1) != '*')) {
            src++;
            continue;
        }
        
        // Skip highlight markers
        if (*src == '=' && *(src+1) == '=') {
            src += 2;
            continue;
        }
        
        // Copy regular character
        *dest++ = *src++;
    }
    
    *dest = '\0';
    return result;
}

// Find markup ranges in line
HYBRID_API HybridTextRange* hybrid_find_markup_ranges(const char* line, int* range_count) {
    if (!line || !range_count) return NULL;
    
    LineFormats* formats = hybrid_analyze_markdown_line(line);
    if (!formats) {
        *range_count = 0;
        return NULL;
    }
    
    // Convert formats to markup ranges
    HybridTextRange* ranges = malloc(sizeof(HybridTextRange) * formats->format_count * 2); // Each format has 2 markup ranges
    int count = 0;
    
    for (int i = 0; i < formats->format_count; i++) {
        FormatInfo* info = &formats->formats[i];
        
        // Opening markup
        ranges[count].start = info->range.start;
        ranges[count].end = info->content_range.start;
        count++;
        
        // Closing markup
        ranges[count].start = info->content_range.end;
        ranges[count].end = info->range.end;
        count++;
    }
    
    *range_count = count;
    hybrid_free_line_formats(formats);
    return ranges;
}

// Get line content by index
HYBRID_API char* hybrid_get_line_content(const char* text, int line_index) {
    if (!text || line_index < 0) return NULL;
    
    TextLines* lines = hybrid_parse_text(text);
    if (!lines || line_index >= lines->line_count) {
        hybrid_free_text_lines(lines);
        return NULL;
    }
    
    LineInfo* info = &lines->lines[line_index];
    char* content = malloc(info->length + 1);
    if (content) {
        strncpy(content, text + info->char_start, info->length);
        content[info->length] = '\0';
    }
    
    hybrid_free_text_lines(lines);
    return content;
}

// HTML integration functions
HYBRID_API MarkdownFormat hybrid_detect_format_from_html(const char* html) {
    if (!html) return MD_FORMAT_NONE;
    
    MarkdownFormat format = MD_FORMAT_NONE;
    
    if (strstr(html, "<h1>")) format |= MD_FORMAT_HEADER1;
    if (strstr(html, "<h2>")) format |= MD_FORMAT_HEADER2;
    if (strstr(html, "<h3>")) format |= MD_FORMAT_HEADER3;
    if (strstr(html, "<strong>")) format |= MD_FORMAT_BOLD;
    if (strstr(html, "<em>")) format |= MD_FORMAT_ITALIC;
    if (strstr(html, "<mark>")) format |= MD_FORMAT_HIGHLIGHT;
    if (strstr(html, "<li>")) format |= MD_FORMAT_LIST;
    
    return format;
}

HYBRID_API bool hybrid_html_contains_tag(const char* html, const char* tag) {
    if (!html || !tag) return false;
    return strstr(html, tag) != NULL;
}

// Configuration functions
HYBRID_API void hybrid_set_config(const HybridConfig* config) {
    if (config) {
        g_config = *config;
    }
}

HYBRID_API HybridConfig hybrid_get_config(void) {
    return g_config;
}

// Utility functions
HYBRID_API HybridTextRange hybrid_get_word_at_position(const char* text, int position) {
    HybridTextRange range = {position, position};
    if (!text || position < 0) return range;
    
    int len = strlen(text);
    if (position >= len) return range;
    
    // Find word boundaries
    int start = position;
    while (start > 0 && text[start-1] != ' ' && text[start-1] != '\n' && text[start-1] != '\t') {
        start--;
    }
    
    int end = position;
    while (end < len && text[end] != ' ' && text[end] != '\n' && text[end] != '\t') {
        end++;
    }
    
    range.start = start;
    range.end = end;
    return range;
}

// Error handling
HYBRID_API const char* hybrid_get_error_message(HybridResult result) {
    switch (result) {
        case HYBRID_SUCCESS: return "Success";
        case HYBRID_ERROR_NULL_POINTER: return "Null pointer error";
        case HYBRID_ERROR_INVALID_LINE: return "Invalid line index";
        case HYBRID_ERROR_OUT_OF_MEMORY: return "Out of memory";
        case HYBRID_ERROR_INVALID_FORMAT: return "Invalid format";
        default: return "Unknown error";
    }
}