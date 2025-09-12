#include "cursor_manager.h"
#include <emscripten/emscripten.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// WebAssembly exports for cursor management

// Convert HTML position to Markdown position
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_html_to_markdown(int html_position, const char* markdown_text) {
    if (!markdown_text) return -1;
    
    cursor_position_t result = cursor_html_to_markdown(html_position, markdown_text);
    return result.is_valid ? result.position : -1;
}

// Adjust position for formatting (returns adjusted position)
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_adjust_for_formatting(int position, const char* content) {
    if (!content) return position;
    
    cursor_position_t result = cursor_adjust_for_formatting(position, content, true);
    return result.is_valid ? result.position : position;
}

// Check if position is inside formatting markers
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_is_inside_formatting(const char* content, int position) {
    if (!content) return 0;
    
    formatting_context_t context = cursor_analyze_formatting(content, position);
    return context.inside_marker ? 1 : 0;
}

// Get formatting type at position (0=none, 1=bold, 2=italic, 3=highlight, 4=underline, 5=header)
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_get_formatting_type(const char* content, int position) {
    if (!content) return 0;
    
    formatting_context_t context = cursor_analyze_formatting(content, position);
    return (int)context.type;
}

// Handle Enter key - returns JSON string with result
EMSCRIPTEN_KEEPALIVE
char* cursor_wasm_handle_enter_key(int position, const char* content) {
    if (!content) return NULL;
    
    cursor_operation_result_t result = cursor_handle_enter_key(position, content, true);
    
    // Create JSON response
    char* json = malloc(2048);
    if (!json) {
        cursor_free_result(&result);
        return NULL;
    }
    
    if (result.success) {
        snprintf(json, 2048,
            "{"
            "\"success\": true, "
            "\"beforeCursor\": \"%s\", "
            "\"afterCursor\": \"%s\", "
            "\"newPosition\": %d"
            "}",
            result.before_cursor ? result.before_cursor : "",
            result.after_cursor ? result.after_cursor : "",
            result.new_position.position
        );
    } else {
        snprintf(json, 2048,
            "{"
            "\"success\": false, "
            "\"error\": \"%s\""
            "}",
            result.error_message ? result.error_message : "Unknown error"
        );
    }
    
    cursor_free_result(&result);
    return json;
}

// Split line at position - returns JSON string with result
EMSCRIPTEN_KEEPALIVE
char* cursor_wasm_split_line(int position, const char* content) {
    if (!content) return NULL;
    
    cursor_operation_result_t result = cursor_split_line(position, content);
    
    // Create JSON response
    char* json = malloc(2048);
    if (!json) {
        cursor_free_result(&result);
        return NULL;
    }
    
    if (result.success) {
        snprintf(json, 2048,
            "{"
            "\"success\": true, "
            "\"beforeCursor\": \"%s\", "
            "\"afterCursor\": \"%s\""
            "}",
            result.before_cursor ? result.before_cursor : "",
            result.after_cursor ? result.after_cursor : ""
        );
    } else {
        snprintf(json, 2048,
            "{"
            "\"success\": false, "
            "\"error\": \"%s\""
            "}",
            result.error_message ? result.error_message : "Unknown error"
        );
    }
    
    cursor_free_result(&result);
    return json;
}

// Merge two lines - returns JSON string with result
EMSCRIPTEN_KEEPALIVE
char* cursor_wasm_merge_lines(const char* line1, const char* line2, int add_space) {
    if (!line1 || !line2) return NULL;
    
    cursor_operation_result_t result = cursor_merge_lines(line1, line2, add_space != 0);
    
    // Create JSON response
    char* json = malloc(2048);
    if (!json) {
        cursor_free_result(&result);
        return NULL;
    }
    
    if (result.success) {
        snprintf(json, 2048,
            "{"
            "\"success\": true, "
            "\"mergedContent\": \"%s\", "
            "\"cursorPosition\": %d"
            "}",
            result.before_cursor ? result.before_cursor : "",
            result.new_position.position
        );
    } else {
        snprintf(json, 2048,
            "{"
            "\"success\": false, "
            "\"error\": \"%s\""
            "}",
            result.error_message ? result.error_message : "Unknown error"
        );
    }
    
    cursor_free_result(&result);
    return json;
}

// Validate position in content
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_validate_position(const char* content, int position) {
    return cursor_validate_position(content, position) ? 1 : 0;
}

// Find safe split position (avoiding formatting markers)
EMSCRIPTEN_KEEPALIVE
int cursor_wasm_find_safe_position(const char* content, int position) {
    if (!content) return position;
    return cursor_find_safe_split_position(content, position);
}

// Free memory allocated by WASM functions
EMSCRIPTEN_KEEPALIVE
void cursor_wasm_free(void* ptr) {
    if (ptr) {
        free(ptr);
    }
}

// Debug function for testing
EMSCRIPTEN_KEEPALIVE
void cursor_wasm_debug(const char* content, int position) {
    cursor_print_debug(content, position);
}