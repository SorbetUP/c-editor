#ifndef CURSOR_MANAGER_H
#define CURSOR_MANAGER_H

#include <stddef.h>
#include <stdbool.h>

// Cursor position structure
typedef struct {
    int line_index;
    int position;
    bool is_markdown_mode;
    bool is_valid;
} cursor_position_t;

// Cursor operation result
typedef struct {
    bool success;
    cursor_position_t new_position;
    char* before_cursor;
    char* after_cursor;
    char* error_message;
} cursor_operation_result_t;

// Formatting marker types
typedef enum {
    MARKER_NONE = 0,
    MARKER_BOLD,        // **text**
    MARKER_ITALIC,      // *text*
    MARKER_HIGHLIGHT,   // ==text==
    MARKER_UNDERLINE,   // ++text++
    MARKER_HEADER       // # text
} formatting_marker_t;

// Formatting context
typedef struct {
    formatting_marker_t type;
    int start_pos;
    int end_pos;
    int marker_length;
    bool inside_marker;
} formatting_context_t;

// Core cursor management functions
cursor_position_t cursor_html_to_markdown(int html_position, const char* markdown_text);
cursor_position_t cursor_markdown_to_html(int markdown_position, const char* markdown_text);

// Position adjustment for formatting
cursor_position_t cursor_adjust_for_formatting(int position, const char* content, bool is_markdown_mode);
formatting_context_t cursor_analyze_formatting(const char* content, int position);

// Enter key handling
cursor_operation_result_t cursor_handle_enter_key(int position, const char* content, bool is_markdown_mode);
cursor_operation_result_t cursor_split_line(int position, const char* content);

// Line merging
cursor_operation_result_t cursor_merge_lines(const char* line1, const char* line2, bool add_space);

// Utility functions
bool cursor_is_inside_formatting(const char* content, int position, formatting_marker_t marker_type);
int cursor_find_safe_split_position(const char* content, int position);
char* cursor_extract_before_position(const char* content, int position);
char* cursor_extract_after_position(const char* content, int position);

// Memory management
void cursor_free_result(cursor_operation_result_t* result);

// Word navigation functions
cursor_position_t cursor_move_word_left(const char* content, int position);
cursor_position_t cursor_move_word_right(const char* content, int position);
cursor_position_t cursor_move_to_line_start(const char* content, int position);
cursor_position_t cursor_move_to_line_end(const char* content, int position);

// Advanced formatting detection
bool cursor_is_at_word_boundary(const char* content, int position);
bool cursor_is_whitespace_char(char c);
bool cursor_is_word_char(char c);

// Smart indentation
cursor_operation_result_t cursor_smart_indent(const char* content, int position);
int cursor_get_line_indentation(const char* line);
char* cursor_create_indented_line(const char* content, int indent_level);

// Bracket and quote matching
cursor_position_t cursor_find_matching_bracket(const char* content, int position);
cursor_position_t cursor_find_matching_quote(const char* content, int position);

// Advanced line operations
cursor_operation_result_t cursor_duplicate_line(const char* content, int position);
cursor_operation_result_t cursor_delete_line(const char* content, int position);
cursor_operation_result_t cursor_move_line_up(const char* content, int position);
cursor_operation_result_t cursor_move_line_down(const char* content, int position);

// Debug and validation
bool cursor_validate_position(const char* content, int position);
void cursor_print_debug(const char* content, int position);

#endif // CURSOR_MANAGER_H