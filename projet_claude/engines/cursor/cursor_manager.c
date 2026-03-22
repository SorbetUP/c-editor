#include "cursor_manager.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#define MAX_LINE_LENGTH 4096
#ifndef DEBUG_CURSOR
#define DEBUG_CURSOR 1
#endif

// Helper macro for debug output
#if DEBUG_CURSOR
#define CURSOR_DEBUG(fmt, ...) printf("[CURSOR] " fmt "\n", ##__VA_ARGS__)
#else
#define CURSOR_DEBUG(fmt, ...)
#endif

// Helper function to safely allocate and copy string
static char* safe_strdup(const char* str) {
    if (!str) return NULL;
    size_t len = strlen(str);
    char* copy = malloc(len + 1);
    if (copy) {
        memcpy(copy, str, len + 1);
    }
    return copy;
}

// Helper function to safely copy substring
static char* safe_substr(const char* str, int start, int length) {
    if (!str || start < 0 || length < 0) return NULL;
    
    int str_len = strlen(str);
    if (start >= str_len) return safe_strdup("");
    
    int actual_length = (start + length > str_len) ? str_len - start : length;
    char* result = malloc(actual_length + 1);
    if (result) {
        memcpy(result, str + start, actual_length);
        result[actual_length] = '\0';
    }
    return result;
}

// Analyze formatting context at a given position
formatting_context_t cursor_analyze_formatting(const char* content, int position) {
    formatting_context_t context = {MARKER_NONE, -1, -1, 0, false};
    
    if (!content || position < 0) {
        return context;
    }
    
    int len = strlen(content);
    if (position >= len) {
        position = len;
    }
    
    CURSOR_DEBUG("Analyzing formatting at position %d in: \"%s\"", position, content);
    
    // Check for header first
    if (content[0] == '#') {
        int header_end = 0;
        while (header_end < len && content[header_end] == '#') header_end++;
        if (header_end < len && content[header_end] == ' ') {
            context.type = MARKER_HEADER;
            context.start_pos = 0;
            context.end_pos = header_end + 1;
            context.marker_length = header_end + 1;
            context.inside_marker = (position < header_end + 1);
            CURSOR_DEBUG("Found header marker, inside: %s", context.inside_marker ? "yes" : "no");
            return context;
        }
    }
    
    // Check for highlight markers ==
    for (int i = 0; i < len - 1; i++) {
        if (content[i] == '=' && content[i + 1] == '=') {
            // Find closing ==
            for (int j = i + 2; j < len - 1; j++) {
                if (content[j] == '=' && content[j + 1] == '=') {
                    if (position >= i && position <= j + 1) {
                        context.type = MARKER_HIGHLIGHT;
                        context.start_pos = i;
                        context.end_pos = j + 1;
                        context.marker_length = 2;
                        context.inside_marker = (position > i + 1 && position < j);
                        CURSOR_DEBUG("Found highlight marker at %d-%d, inside: %s", i, j+1, context.inside_marker ? "yes" : "no");
                        return context;
                    }
                    break;
                }
            }
        }
    }
    
    // Check for bold markers **
    for (int i = 0; i < len - 1; i++) {
        if (content[i] == '*' && content[i + 1] == '*') {
            // Find closing **
            for (int j = i + 2; j < len - 1; j++) {
                if (content[j] == '*' && content[j + 1] == '*') {
                    if (position >= i && position <= j + 1) {
                        context.type = MARKER_BOLD;
                        context.start_pos = i;
                        context.end_pos = j + 1;
                        context.marker_length = 2;
                        context.inside_marker = (position > i + 1 && position < j);
                        CURSOR_DEBUG("Found bold marker at %d-%d, inside: %s", i, j+1, context.inside_marker ? "yes" : "no");
                        return context;
                    }
                    break;
                }
            }
        }
    }
    
    // Check for italic markers * (but not part of **)
    for (int i = 0; i < len; i++) {
        if (content[i] == '*' && 
            (i == 0 || content[i-1] != '*') && 
            (i == len-1 || content[i+1] != '*')) {
            
            // Find closing *
            for (int j = i + 1; j < len; j++) {
                if (content[j] == '*' && 
                    (j == len-1 || content[j+1] != '*') &&
                    (j == 0 || content[j-1] != '*')) {
                    
                    if (position >= i && position <= j) {
                        context.type = MARKER_ITALIC;
                        context.start_pos = i;
                        context.end_pos = j;
                        context.marker_length = 1;
                        context.inside_marker = (position > i && position < j);
                        CURSOR_DEBUG("Found italic marker at %d-%d, inside: %s", i, j, context.inside_marker ? "yes" : "no");
                        return context;
                    }
                    break;
                }
            }
        }
    }
    
    // Check for underline markers ++
    for (int i = 0; i < len - 1; i++) {
        if (content[i] == '+' && content[i + 1] == '+') {
            // Find closing ++
            for (int j = i + 2; j < len - 1; j++) {
                if (content[j] == '+' && content[j + 1] == '+') {
                    if (position >= i && position <= j + 1) {
                        context.type = MARKER_UNDERLINE;
                        context.start_pos = i;
                        context.end_pos = j + 1;
                        context.marker_length = 2;
                        context.inside_marker = (position > i + 1 && position < j);
                        CURSOR_DEBUG("Found underline marker at %d-%d, inside: %s", i, j+1, context.inside_marker ? "yes" : "no");
                        return context;
                    }
                    break;
                }
            }
        }
    }
    
    CURSOR_DEBUG("No formatting markers found at position %d", position);
    return context;
}

// Adjust cursor position to avoid problematic splits in formatting
cursor_position_t cursor_adjust_for_formatting(int position, const char* content, bool is_markdown_mode) {
    cursor_position_t result = {0, position, is_markdown_mode, true};
    
    if (!content) {
        result.is_valid = false;
        return result;
    }
    
    CURSOR_DEBUG("Adjusting position %d for formatting in: \"%s\"", position, content);
    
    formatting_context_t context = cursor_analyze_formatting(content, position);
    
    if (context.inside_marker) {
        CURSOR_DEBUG("Position %d is inside %d marker, adjusting to %d", 
                    position, context.type, context.start_pos);
        result.position = context.start_pos;
    } else {
        CURSOR_DEBUG("Position %d is safe, no adjustment needed", position);
    }
    
    return result;
}

// Convert HTML position to markdown position
cursor_position_t cursor_html_to_markdown(int html_position, const char* markdown_text) {
    cursor_position_t result = {0, 0, false, false};
    
    if (!markdown_text) {
        return result;
    }
    
    int markdown_len = strlen(markdown_text);
    int html_pos = 0;
    int markdown_pos = 0;
    
    CURSOR_DEBUG("Converting HTML position %d to markdown in: \"%s\"", html_position, markdown_text);
    
    // Handle header prefix
    if (markdown_text[0] == '#') {
        int header_len = 0;
        while (header_len < markdown_len && markdown_text[header_len] == '#') header_len++;
        if (header_len < markdown_len && markdown_text[header_len] == ' ') {
            if (html_position == 0) {
                result.position = header_len + 1;
                result.is_valid = true;
                CURSOR_DEBUG("HTML position 0 maps to markdown %d (after header)", result.position);
                return result;
            }
            markdown_pos = header_len + 1;
        }
    }
    
    // Process character by character
    while (markdown_pos < markdown_len && html_pos < html_position) {
        char current = markdown_text[markdown_pos];
        
        // Check for formatting markers
        if (current == '*' && markdown_pos + 1 < markdown_len) {
            if (markdown_text[markdown_pos + 1] == '*') {
                // Bold **text**
                int end_pos = -1;
                for (int i = markdown_pos + 2; i < markdown_len - 1; i++) {
                    if (markdown_text[i] == '*' && markdown_text[i + 1] == '*') {
                        end_pos = i;
                        break;
                    }
                }
                
                if (end_pos != -1) {
                    int inner_len = end_pos - (markdown_pos + 2);
                    if (html_position - html_pos <= inner_len) {
                        result.position = markdown_pos + 2 + (html_position - html_pos);
                        result.is_valid = true;
                        CURSOR_DEBUG("HTML %d maps to markdown %d (inside bold)", html_position, result.position);
                        return result;
                    }
                    html_pos += inner_len;
                    markdown_pos = end_pos + 2;
                    continue;
                }
            } else {
                // Italic *text*
                int end_pos = -1;
                for (int i = markdown_pos + 1; i < markdown_len; i++) {
                    if (markdown_text[i] == '*' && 
                        (i == markdown_len - 1 || markdown_text[i + 1] != '*')) {
                        end_pos = i;
                        break;
                    }
                }
                
                if (end_pos != -1) {
                    int inner_len = end_pos - (markdown_pos + 1);
                    if (html_position - html_pos <= inner_len) {
                        result.position = markdown_pos + 1 + (html_position - html_pos);
                        result.is_valid = true;
                        CURSOR_DEBUG("HTML %d maps to markdown %d (inside italic)", html_position, result.position);
                        return result;
                    }
                    html_pos += inner_len;
                    markdown_pos = end_pos + 1;
                    continue;
                }
            }
        } else if (current == '=' && markdown_pos + 1 < markdown_len && markdown_text[markdown_pos + 1] == '=') {
            // Highlight ==text==
            int end_pos = -1;
            for (int i = markdown_pos + 2; i < markdown_len - 1; i++) {
                if (markdown_text[i] == '=' && markdown_text[i + 1] == '=') {
                    end_pos = i;
                    break;
                }
            }
            
            if (end_pos != -1) {
                int inner_len = end_pos - (markdown_pos + 2);
                if (html_position - html_pos <= inner_len) {
                    result.position = markdown_pos + 2 + (html_position - html_pos);
                    result.is_valid = true;
                    CURSOR_DEBUG("HTML %d maps to markdown %d (inside highlight)", html_position, result.position);
                    return result;
                }
                html_pos += inner_len;
                markdown_pos = end_pos + 2;
                continue;
            }
        }
        
        // Regular character
        html_pos++;
        markdown_pos++;
    }
    
    result.position = markdown_pos;
    result.is_valid = true;
    CURSOR_DEBUG("HTML %d maps to markdown %d (final)", html_position, result.position);
    return result;
}

// Handle Enter key with smart cursor positioning
cursor_operation_result_t cursor_handle_enter_key(int position, const char* content, bool is_markdown_mode) {
    cursor_operation_result_t result = {false, {0, 0, false, false}, NULL, NULL, NULL};
    
    if (!content) {
        result.error_message = safe_strdup("Invalid content");
        return result;
    }
    
    CURSOR_DEBUG("Handling Enter key at position %d in: \"%s\"", position, content);
    
    // First, adjust position for formatting if needed
    cursor_position_t adjusted = cursor_adjust_for_formatting(position, content, is_markdown_mode);
    if (!adjusted.is_valid) {
        result.error_message = safe_strdup("Failed to adjust position");
        return result;
    }
    
    // If position was adjusted, use the new position
    int final_position = adjusted.position;
    
    CURSOR_DEBUG("Using final position %d for line split", final_position);
    
    // Perform the line split
    cursor_operation_result_t split_result = cursor_split_line(final_position, content);
    if (!split_result.success) {
        return split_result; // Return the error
    }
    
    // Set the new cursor position (start of new line)
    result.success = true;
    result.new_position.line_index = 1; // Next line
    result.new_position.position = 0;   // Start of line
    result.new_position.is_markdown_mode = true;
    result.new_position.is_valid = true;
    result.before_cursor = split_result.before_cursor;
    result.after_cursor = split_result.after_cursor;
    
    // Don't double-free, transfer ownership
    split_result.before_cursor = NULL;
    split_result.after_cursor = NULL;
    cursor_free_result(&split_result);
    
    CURSOR_DEBUG("Enter key handled successfully, cursor at start of new line");
    return result;
}

// Split line at given position
cursor_operation_result_t cursor_split_line(int position, const char* content) {
    cursor_operation_result_t result = {false, {0, 0, false, false}, NULL, NULL, NULL};
    
    if (!content) {
        result.error_message = safe_strdup("Invalid content");
        return result;
    }
    
    int len = strlen(content);
    if (position < 0) position = 0;
    if (position > len) position = len;
    
    CURSOR_DEBUG("Splitting line at position %d", position);
    
    // Extract before and after cursor
    result.before_cursor = safe_substr(content, 0, position);
    result.after_cursor = safe_substr(content, position, len - position);
    
    if (!result.before_cursor || !result.after_cursor) {
        result.error_message = safe_strdup("Failed to allocate memory for split");
        cursor_free_result(&result);
        return result;
    }
    
    result.success = true;
    CURSOR_DEBUG("Line split: \"%s\" | \"%s\"", result.before_cursor, result.after_cursor);
    
    return result;
}

// Merge two lines with optional spacing
cursor_operation_result_t cursor_merge_lines(const char* line1, const char* line2, bool add_space) {
    cursor_operation_result_t result = {false, {0, 0, false, false}, NULL, NULL, NULL};
    
    if (!line1 || !line2) {
        result.error_message = safe_strdup("Invalid line content");
        return result;
    }
    
    int len1 = strlen(line1);
    int len2 = strlen(line2);
    
    // Determine if we need to add space
    bool needs_space = add_space && len1 > 0 && len2 > 0 && 
                       line1[len1-1] != ' ' && line2[0] != ' ';
    
    int total_len = len1 + len2 + (needs_space ? 1 : 0);
    
    char* merged = malloc(total_len + 1);
    if (!merged) {
        result.error_message = safe_strdup("Failed to allocate memory for merge");
        return result;
    }
    
    // Copy first line
    memcpy(merged, line1, len1);
    
    // Add space if needed
    if (needs_space) {
        merged[len1] = ' ';
        len1++;
    }
    
    // Copy second line
    memcpy(merged + len1, line2, len2);
    merged[total_len] = '\0';
    
    result.success = true;
    result.before_cursor = merged;
    result.new_position.position = len1; // Cursor after first line + space
    result.new_position.is_valid = true;
    
    CURSOR_DEBUG("Lines merged: \"%s\" + \"%s\" = \"%s\" (cursor at %d)", 
                line1, line2, merged, result.new_position.position);
    
    return result;
}

// Utility functions
bool cursor_validate_position(const char* content, int position) {
    if (!content || position < 0) return false;
    return position <= (int)strlen(content);
}

int cursor_find_safe_split_position(const char* content, int position) {
    if (!content) return 0;
    
    cursor_position_t adjusted = cursor_adjust_for_formatting(position, content, true);
    return adjusted.is_valid ? adjusted.position : position;
}

char* cursor_extract_before_position(const char* content, int position) {
    if (!content || position < 0) return NULL;
    return safe_substr(content, 0, position);
}

char* cursor_extract_after_position(const char* content, int position) {
    if (!content || position < 0) return NULL;
    int len = strlen(content);
    return safe_substr(content, position, len - position);
}

// Memory management
void cursor_free_result(cursor_operation_result_t* result) {
    if (!result) return;
    
    if (result->before_cursor) {
        free(result->before_cursor);
        result->before_cursor = NULL;
    }
    if (result->after_cursor) {
        free(result->after_cursor);
        result->after_cursor = NULL;
    }
    if (result->error_message) {
        free(result->error_message);
        result->error_message = NULL;
    }
}

// Debug function
void cursor_print_debug(const char* content, int position) {
    if (!content) {
        CURSOR_DEBUG("Debug: Invalid content");
        return;
    }
    
    int len = strlen(content);
    CURSOR_DEBUG("Debug: Content \"%s\" (length %d), position %d", content, len, position);
    
    if (position >= 0 && position < len) {
        CURSOR_DEBUG("Character at position: '%c'", content[position]);
    } else if (position == len) {
        CURSOR_DEBUG("Position at end of content");
    } else {
        CURSOR_DEBUG("Position out of bounds");
    }
    
    formatting_context_t context = cursor_analyze_formatting(content, position);
    if (context.type != MARKER_NONE) {
        CURSOR_DEBUG("Formatting context: type %d, range %d-%d, inside: %s", 
                    context.type, context.start_pos, context.end_pos, 
                    context.inside_marker ? "yes" : "no");
    }
}

// ============= NEW ADVANCED CURSOR FUNCTIONS =============

// Helper functions for word navigation
bool cursor_is_whitespace_char(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

bool cursor_is_word_char(char c) {
    return isalnum(c) || c == '_';
}

bool cursor_is_at_word_boundary(const char* content, int position) {
    if (!content) return false;
    int len = strlen(content);
    
    if (position <= 0 || position >= len) return true;
    
    char prev_char = content[position - 1];
    char curr_char = content[position];
    
    // Boundary between word and non-word characters
    return (cursor_is_word_char(prev_char) != cursor_is_word_char(curr_char));
}

// Word navigation functions
cursor_position_t cursor_move_word_left(const char* content, int position) {
    cursor_position_t result = {0, position, true, false};
    
    if (!content || position <= 0) {
        result.position = 0;
        result.is_valid = true;
        return result;
    }
    
    int len = strlen(content);
    if (position > len) position = len;
    
    int pos = position - 1;
    
    // Skip whitespace to the left
    while (pos >= 0 && cursor_is_whitespace_char(content[pos])) {
        pos--;
    }
    
    // Move to start of current word
    while (pos >= 0 && cursor_is_word_char(content[pos])) {
        pos--;
    }
    
    // Skip non-word, non-whitespace characters
    while (pos >= 0 && !cursor_is_word_char(content[pos]) && !cursor_is_whitespace_char(content[pos])) {
        pos--;
    }
    
    result.position = pos + 1;
    result.is_valid = true;
    
    CURSOR_DEBUG("Move word left from %d to %d", position, result.position);
    return result;
}

cursor_position_t cursor_move_word_right(const char* content, int position) {
    cursor_position_t result = {0, position, true, false};
    
    if (!content) {
        result.is_valid = false;
        return result;
    }
    
    int len = strlen(content);
    if (position >= len) {
        result.position = len;
        result.is_valid = true;
        return result;
    }
    
    int pos = position;
    
    // Skip current word
    while (pos < len && cursor_is_word_char(content[pos])) {
        pos++;
    }
    
    // Skip non-word, non-whitespace characters  
    while (pos < len && !cursor_is_word_char(content[pos]) && !cursor_is_whitespace_char(content[pos])) {
        pos++;
    }
    
    // Skip whitespace
    while (pos < len && cursor_is_whitespace_char(content[pos])) {
        pos++;
    }
    
    result.position = pos;
    result.is_valid = true;
    
    CURSOR_DEBUG("Move word right from %d to %d", position, result.position);
    return result;
}

cursor_position_t cursor_move_to_line_start(const char* content, int position) {
    cursor_position_t result = {0, position, true, false};
    
    if (!content) {
        result.is_valid = false;
        return result;
    }
    
    int pos = position;
    
    // Move backwards to find start of line (or beginning of content)
    while (pos > 0 && content[pos - 1] != '\n') {
        pos--;
    }
    
    result.position = pos;
    result.is_valid = true;
    
    CURSOR_DEBUG("Move to line start from %d to %d", position, result.position);
    return result;
}

cursor_position_t cursor_move_to_line_end(const char* content, int position) {
    cursor_position_t result = {0, position, true, false};
    
    if (!content) {
        result.is_valid = false;
        return result;
    }
    
    int len = strlen(content);
    int pos = position;
    
    // Move forward to find end of line (or end of content)
    while (pos < len && content[pos] != '\n') {
        pos++;
    }
    
    result.position = pos;
    result.is_valid = true;
    
    CURSOR_DEBUG("Move to line end from %d to %d", position, result.position);
    return result;
}

// Smart indentation functions
int cursor_get_line_indentation(const char* line) {
    if (!line) return 0;
    
    int indent = 0;
    for (int i = 0; line[i] != '\0' && line[i] != '\n'; i++) {
        if (line[i] == ' ') {
            indent++;
        } else if (line[i] == '\t') {
            indent += 4; // Tab equals 4 spaces
        } else {
            break;
        }
    }
    
    return indent;
}

char* cursor_create_indented_line(const char* content, int indent_level) {
    if (!content) return NULL;
    
    size_t content_len = strlen(content);
    char* result = malloc(content_len + indent_level + 1);
    if (!result) return NULL;
    
    // Add indentation
    for (int i = 0; i < indent_level; i++) {
        result[i] = ' ';
    }
    
    // Copy content
    strcpy(result + indent_level, content);
    
    return result;
}

cursor_operation_result_t cursor_smart_indent(const char* content, int position) {
    cursor_operation_result_t result = {false, {0, 0, true, false}, NULL, NULL, NULL};
    
    if (!content) {
        result.error_message = safe_strdup("Invalid content");
        return result;
    }
    
    // Find the start of the current line
    cursor_position_t line_start = cursor_move_to_line_start(content, position);
    cursor_position_t line_end = cursor_move_to_line_end(content, position);
    
    // Extract current line
    char* current_line = safe_substr(content, line_start.position, 
                                   line_end.position - line_start.position);
    if (!current_line) {
        result.error_message = safe_strdup("Memory allocation failed");
        return result;
    }
    
    // Get indentation of previous line (if exists)
    int prev_indent = 0;
    if (line_start.position > 0) {
        // Find previous line
        int prev_line_end = line_start.position - 2; // Skip the \n
        while (prev_line_end >= 0 && content[prev_line_end] != '\n') {
            prev_line_end--;
        }
        int prev_line_start = prev_line_end + 1;
        
        char* prev_line = safe_substr(content, prev_line_start, 
                                    line_start.position - 1 - prev_line_start);
        if (prev_line) {
            prev_indent = cursor_get_line_indentation(prev_line);
            free(prev_line);
        }
    }
    
    // Create properly indented line
    char* indented_line = cursor_create_indented_line(current_line, prev_indent);
    free(current_line);
    
    if (!indented_line) {
        result.error_message = safe_strdup("Failed to create indented line");
        return result;
    }
    
    // Create result
    result.before_cursor = safe_substr(content, 0, line_start.position);
    result.after_cursor = safe_substr(content, line_end.position, 
                                    strlen(content) - line_end.position);
    
    // Concatenate with indented line
    size_t before_len = result.before_cursor ? strlen(result.before_cursor) : 0;
    size_t indented_len = strlen(indented_line);
    
    char* new_before = malloc(before_len + indented_len + 1);
    if (new_before) {
        strcpy(new_before, result.before_cursor ? result.before_cursor : "");
        strcat(new_before, indented_line);
        free(result.before_cursor);
        result.before_cursor = new_before;
    }
    
    result.new_position.position = line_start.position + indented_len;
    result.new_position.is_valid = true;
    result.success = true;
    
    free(indented_line);
    return result;
}

// Bracket matching functions
cursor_position_t cursor_find_matching_bracket(const char* content, int position) {
    cursor_position_t result = {0, -1, true, false};
    
    if (!content || position < 0 || position >= (int)strlen(content)) {
        return result;
    }
    
    char current = content[position];
    char target;
    int direction;
    
    // Determine bracket type and search direction
    switch (current) {
        case '(':
            target = ')';
            direction = 1;
            break;
        case '[':
            target = ']';
            direction = 1;
            break;
        case '{':
            target = '}';
            direction = 1;
            break;
        case ')':
            target = '(';
            direction = -1;
            break;
        case ']':
            target = '[';
            direction = -1;
            break;
        case '}':
            target = '{';
            direction = -1;
            break;
        default:
            return result; // Not a bracket
    }
    
    int len = strlen(content);
    int pos = position + direction;
    int depth = 1;
    
    while (pos >= 0 && pos < len && depth > 0) {
        if (content[pos] == current) {
            depth++;
        } else if (content[pos] == target) {
            depth--;
        }
        
        if (depth == 0) {
            result.position = pos;
            result.is_valid = true;
            break;
        }
        
        pos += direction;
    }
    
    CURSOR_DEBUG("Find matching bracket for '%c' at %d: found at %d", 
                current, position, result.position);
    return result;
}

// Line manipulation functions
cursor_operation_result_t cursor_duplicate_line(const char* content, int position) {
    cursor_operation_result_t result = {false, {0, 0, true, false}, NULL, NULL, NULL};
    
    if (!content) {
        result.error_message = safe_strdup("Invalid content");
        return result;
    }
    
    // Find line boundaries
    cursor_position_t line_start = cursor_move_to_line_start(content, position);
    cursor_position_t line_end = cursor_move_to_line_end(content, position);
    
    // Extract current line
    char* current_line = safe_substr(content, line_start.position, 
                                   line_end.position - line_start.position);
    if (!current_line) {
        result.error_message = safe_strdup("Memory allocation failed");
        return result;
    }
    
    // Create duplicated content
    size_t line_len = strlen(current_line);
    char* duplicated = malloc(line_len * 2 + 2); // +2 for \n
    if (!duplicated) {
        free(current_line);
        result.error_message = safe_strdup("Memory allocation failed");
        return result;
    }
    
    sprintf(duplicated, "%s\n%s", current_line, current_line);
    
    // Build result
    result.before_cursor = safe_substr(content, 0, line_start.position);
    result.after_cursor = safe_substr(content, line_end.position, 
                                    strlen(content) - line_end.position);
    
    // Update before_cursor to include duplicated line
    if (result.before_cursor) {
        size_t before_len = strlen(result.before_cursor);
        size_t dup_len = strlen(duplicated);
        char* new_before = malloc(before_len + dup_len + 1);
        if (new_before) {
            strcpy(new_before, result.before_cursor);
            strcat(new_before, duplicated);
            free(result.before_cursor);
            result.before_cursor = new_before;
        }
    } else {
        result.before_cursor = safe_strdup(duplicated);
    }
    
    result.new_position.position = line_start.position + line_len + 1; // Position on duplicated line
    result.new_position.is_valid = true;
    result.success = true;
    
    free(current_line);
    free(duplicated);
    return result;
}