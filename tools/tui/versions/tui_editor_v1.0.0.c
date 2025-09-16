#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <signal.h>
#include <ctype.h>
#include <errno.h>
#include <time.h>
#include <stdarg.h>
#include "cursor_manager.h"
#include "markdown.h"

#define MAX_LINES 1000
#define MAX_LINE_LENGTH 4096
#define RENDER_BUFFER_SIZE 65536  // Increased buffer size
#define CTRL_KEY(k) ((k) & 0x1f)
#define TAB_STOP 4

// Terminal state
static struct termios orig_termios;

// Editor state
typedef struct {
    char lines[MAX_LINES][MAX_LINE_LENGTH];
    int line_count;
    int cursor_line;
    int cursor_col;
    int screen_rows;
    int screen_cols;
    int row_offset;
    int col_offset;
    char status_msg[256];
    char filename[256];
    int dirty;
    int show_line_numbers;
    time_t status_msg_time;
} editor_state_t;

static editor_state_t E = {0};

// Forward declarations
void editor_refresh_screen();
void editor_set_status_message(const char *fmt, ...);
void disable_raw_mode();

// Signal handler for clean exit
void handle_sigint(int sig) {
    (void)sig;
    write(STDOUT_FILENO, "\x1b[2J", 4);
    write(STDOUT_FILENO, "\x1b[H", 3);
    disable_raw_mode();
    exit(0);
}

// Terminal control
void die(const char *s) {
    write(STDOUT_FILENO, "\x1b[2J", 4);
    write(STDOUT_FILENO, "\x1b[H", 3);
    perror(s);
    exit(1);
}

void disable_raw_mode() {
    if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &orig_termios) == -1)
        die("tcsetattr");
}

void enable_raw_mode() {
    if (tcgetattr(STDIN_FILENO, &orig_termios) == -1) die("tcgetattr");
    atexit(disable_raw_mode);
    
    // Set up signal handlers
    signal(SIGINT, handle_sigint);
    signal(SIGTERM, handle_sigint);
    signal(SIGWINCH, handle_sigint);  // Handle window resize
    
    struct termios raw = orig_termios;
    raw.c_iflag &= ~(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw.c_oflag &= ~(OPOST);
    raw.c_cflag |= (CS8);
    raw.c_lflag &= ~(ECHO | ICANON | IEXTEN | ISIG);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;
    
    if (tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw) == -1) die("tcsetattr");
}

int read_key() {
    int nread;
    char c;
    while ((nread = read(STDIN_FILENO, &c, 1)) != 1) {
        if (nread == -1 && errno != EAGAIN) die("read");
    }
    
    if (c == '\x1b') {
        char seq[3];
        if (read(STDIN_FILENO, &seq[0], 1) != 1) return '\x1b';
        if (read(STDIN_FILENO, &seq[1], 1) != 1) return '\x1b';
        
        if (seq[0] == '[') {
            switch (seq[1]) {
                case 'A': return 1000; // Up arrow
                case 'B': return 1001; // Down arrow
                case 'C': return 1002; // Right arrow
                case 'D': return 1003; // Left arrow
                case 'H': return 1004; // Home
                case 'F': return 1005; // End
            }
        }
    }
    return c;
}

int get_cursor_position(int *rows, int *cols) {
    char buf[32];
    unsigned int i = 0;
    
    if (write(STDOUT_FILENO, "\x1b[6n", 4) != 4) return -1;
    
    while (i < sizeof(buf) - 1) {
        if (read(STDIN_FILENO, &buf[i], 1) != 1) break;
        if (buf[i] == 'R') break;
        i++;
    }
    buf[i] = '\0';
    
    if (buf[0] != '\x1b' || buf[1] != '[') return -1;
    if (sscanf(&buf[2], "%d;%d", rows, cols) != 2) return -1;
    
    return 0;
}

int get_window_size(int *rows, int *cols) {
    struct winsize ws;
    
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0) {
        if (write(STDOUT_FILENO, "\x1b[999C\x1b[999B", 12) != 12) return -1;
        return get_cursor_position(rows, cols);
    } else {
        *cols = ws.ws_col;
        *rows = ws.ws_row;
        return 0;
    }
}

// Editor functions
void tui_editor_init() {
    E.cursor_line = 0;
    E.cursor_col = 0;
    E.row_offset = 0;
    E.col_offset = 0;
    E.line_count = 1;
    E.dirty = 0;
    strcpy(E.lines[0], "# TUI Editor - Test Cursor Management");
    strcpy(E.lines[1], "");
    strcpy(E.lines[2], "- *Italique* test");
    strcpy(E.lines[3], "- **Gras** test");  
    strcpy(E.lines[4], "- ==Surligné== test");
    strcpy(E.lines[5], "- ++Souligné++ test");
    E.line_count = 6;
    
    if (get_window_size(&E.screen_rows, &E.screen_cols) == -1) die("get_window_size");
    E.screen_rows -= 2; // Leave room for status bar
    
    strcpy(E.filename, "untitled.md");
    E.show_line_numbers = 1;
    editor_set_status_message("TUI Editor Enhanced | Press Ctrl-H for help | Ctrl-Q to quit");
}

// Status message with timestamp
void editor_set_status_message(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(E.status_msg, sizeof(E.status_msg), fmt, ap);
    va_end(ap);
    E.status_msg_time = time(NULL);
}

void editor_scroll() {
    if (E.cursor_line < E.row_offset) {
        E.row_offset = E.cursor_line;
    }
    if (E.cursor_line >= E.row_offset + E.screen_rows) {
        E.row_offset = E.cursor_line - E.screen_rows + 1;
    }
    if (E.cursor_col < E.col_offset) {
        E.col_offset = E.cursor_col;
    }
    if (E.cursor_col >= E.col_offset + E.screen_cols) {
        E.col_offset = E.cursor_col - E.screen_cols + 1;
    }
}

void editor_draw_rows(char *buf, int *len, int bufsize) {
    int y;
    for (y = 0; y < E.screen_rows; y++) {
        int filerow = y + E.row_offset;
        
        // Line number area width
        int line_num_width = E.show_line_numbers ? 5 : 0;
        
        if (filerow >= E.line_count) {
            if (E.line_count == 0 && y == E.screen_rows / 3) {
                char welcome[80];
                int welcomelen = snprintf(welcome, sizeof(welcome),
                    "TUI Editor Enhanced -- Smart Cursor Management");
                if (welcomelen > E.screen_cols - line_num_width) 
                    welcomelen = E.screen_cols - line_num_width;
                
                // Line number padding
                if (E.show_line_numbers) {
                    *len += snprintf(buf + *len, bufsize - *len, "     ");
                }
                
                int padding = (E.screen_cols - line_num_width - welcomelen) / 2;
                if (padding) {
                    *len += snprintf(buf + *len, bufsize - *len, "~");
                    padding--;
                }
                while (padding--) 
                    *len += snprintf(buf + *len, bufsize - *len, " ");
                *len += snprintf(buf + *len, bufsize - *len, "%s", welcome);
            } else {
                // Line number padding for empty lines
                if (E.show_line_numbers) {
                    *len += snprintf(buf + *len, bufsize - *len, "     ");
                }
                *len += snprintf(buf + *len, bufsize - *len, "~");
            }
        } else {
            // Line numbers
            if (E.show_line_numbers) {
                if (filerow == E.cursor_line) {
                    *len += snprintf(buf + *len, bufsize - *len, "\x1b[33m%4d \x1b[0m", filerow + 1);
                } else {
                    *len += snprintf(buf + *len, bufsize - *len, "\x1b[90m%4d \x1b[0m", filerow + 1);
                }
            }
            
            // Line content
            int len_line = strlen(E.lines[filerow]);
            int display_len = len_line;
            int col_start = E.col_offset;
            
            if (col_start > len_line) col_start = len_line;
            display_len -= col_start;
            
            int available_width = E.screen_cols - line_num_width;
            if (display_len > available_width) display_len = available_width;
            
            if (display_len > 0) {
                // Basic markdown syntax highlighting
                char *line = E.lines[filerow] + col_start;
                for (int i = 0; i < display_len && *len < bufsize - 20; i++) {
                    char c = line[i];
                    // Very basic highlighting
                    if (i == 0 && c == '#') {
                        *len += snprintf(buf + *len, bufsize - *len, "\x1b[1;34m%c", c); // Blue header
                    } else if (c == '*' && i < display_len - 1 && line[i+1] == '*') {
                        *len += snprintf(buf + *len, bufsize - *len, "\x1b[1m%c", c); // Bold start
                    } else if (c == '*' && i > 0 && line[i-1] == '*') {
                        *len += snprintf(buf + *len, bufsize - *len, "%c\x1b[0m", c); // Bold end
                    } else if (c == '=' && i < display_len - 1 && line[i+1] == '=') {
                        *len += snprintf(buf + *len, bufsize - *len, "\x1b[43m%c", c); // Highlight start
                    } else if (c == '=' && i > 0 && line[i-1] == '=') {
                        *len += snprintf(buf + *len, bufsize - *len, "%c\x1b[0m", c); // Highlight end
                    } else {
                        *len += snprintf(buf + *len, bufsize - *len, "%c", c);
                    }
                }
                *len += snprintf(buf + *len, bufsize - *len, "\x1b[0m"); // Reset colors
            }
        }
        
        *len += snprintf(buf + *len, bufsize - *len, "\x1b[K\r\n"); // Clear line and newline
    }
}

void editor_draw_status_bar(char *buf, int *len, int bufsize) {
    *len += snprintf(buf + *len, bufsize - *len, "\x1b[7m"); // Reverse video
    
    char status[256];
    char rstatus[256];
    
    // Test cursor management functions
    const char *current_line = E.cursor_line < E.line_count ? E.lines[E.cursor_line] : "";
    formatting_context_t ctx = cursor_analyze_formatting(current_line, E.cursor_col);
    const char *fmt_type = "";
    switch(ctx.type) {
        case MARKER_BOLD: fmt_type = "BOLD"; break;
        case MARKER_ITALIC: fmt_type = "ITALIC"; break;
        case MARKER_HIGHLIGHT: fmt_type = "HIGHLIGHT"; break;
        case MARKER_UNDERLINE: fmt_type = "UNDERLINE"; break;
        case MARKER_HEADER: fmt_type = "HEADER"; break;
        default: fmt_type = "NONE"; break;
    }
    
    int len_status = snprintf(status, sizeof(status), "%.20s - %d lines %s%s",
                            E.filename, E.line_count, E.dirty ? "(modified) " : "",
                            E.show_line_numbers ? "[LN] " : "");
    int len_rstatus = snprintf(rstatus, sizeof(rstatus), 
                             "L%d,C%d | %s%s", 
                             E.cursor_line + 1, E.cursor_col + 1,
                             fmt_type, ctx.inside_marker ? " (INSIDE)" : "");
    
    if (len_status > E.screen_cols) len_status = E.screen_cols;
    *len += snprintf(buf + *len, bufsize - *len, "%s", status);
    
    while (len_status < E.screen_cols) {
        if (E.screen_cols - len_status == len_rstatus) {
            *len += snprintf(buf + *len, bufsize - *len, "%s", rstatus);
            break;
        } else {
            *len += snprintf(buf + *len, bufsize - *len, " ");
            len_status++;
        }
    }
    *len += snprintf(buf + *len, bufsize - *len, "\x1b[m"); // Reset
    *len += snprintf(buf + *len, bufsize - *len, "\r\n");
}

void editor_draw_message_bar(char *buf, int *len, int bufsize) {
    *len += snprintf(buf + *len, bufsize - *len, "\x1b[K");
    
    // Only show message if it's recent (within 5 seconds)
    if (E.status_msg_time && time(NULL) - E.status_msg_time < 5) {
        int msglen = strlen(E.status_msg);
        if (msglen > E.screen_cols) msglen = E.screen_cols;
        if (msglen > 0) {
            *len += snprintf(buf + *len, bufsize - *len, "%.*s", msglen, E.status_msg);
        }
    }
}

void editor_refresh_screen() {
    editor_scroll();
    
    static char buf[RENDER_BUFFER_SIZE] = {0};
    int len = 0;
    
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[?25l"); // Hide cursor
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[H"); // Move cursor to top
    
    editor_draw_rows(buf, &len, sizeof(buf));
    editor_draw_status_bar(buf, &len, sizeof(buf));  
    editor_draw_message_bar(buf, &len, sizeof(buf));
    
    // Position cursor (account for line numbers)
    int line_num_width = E.show_line_numbers ? 5 : 0;
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[%d;%dH", 
                   (E.cursor_line - E.row_offset) + 1,
                   (E.cursor_col - E.col_offset) + line_num_width + 1);
    
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[?25h"); // Show cursor
    
    write(STDOUT_FILENO, buf, len);
}

void editor_insert_char(int c) {
    // Bounds checking
    if (E.cursor_line >= MAX_LINES) {
        editor_set_status_message("Error: Maximum lines reached (%d)", MAX_LINES);
        return;
    }
    
    if (E.cursor_line == E.line_count) {
        // Add new line if at end
        if (E.line_count >= MAX_LINES - 1) {
            editor_set_status_message("Error: Maximum lines reached");
            return;
        }
        strcpy(E.lines[E.line_count], "");
        E.line_count++;
    }
    
    char *line = E.lines[E.cursor_line];
    int len = strlen(line);
    
    // Check line length limit
    if (len >= MAX_LINE_LENGTH - 1) {
        editor_set_status_message("Error: Line too long (max %d chars)", MAX_LINE_LENGTH - 1);
        return;
    }
    
    if (E.cursor_col < 0) E.cursor_col = 0;
    if (E.cursor_col > len) E.cursor_col = len;
    
    // Insert character
    memmove(&line[E.cursor_col + 1], &line[E.cursor_col], len - E.cursor_col + 1);
    line[E.cursor_col] = c;
    E.cursor_col++;
    E.dirty = 1;
}

void editor_delete_char() {
    if (E.cursor_line >= E.line_count) return;
    if (E.cursor_col == 0 && E.cursor_line == 0) return;
    
    char *line = E.lines[E.cursor_line];
    if (E.cursor_col > 0) {
        // Delete character before cursor
        int len = strlen(line);
        memmove(&line[E.cursor_col - 1], &line[E.cursor_col], len - E.cursor_col + 1);
        E.cursor_col--;
        E.dirty = 1;
    } else {
        // Merge with previous line - use our cursor management!
        if (E.cursor_line > 0) {
            char *prev_line = E.lines[E.cursor_line - 1];
            char *curr_line = E.lines[E.cursor_line];
            
            // Use C cursor management for smart merge
            cursor_operation_result_t result = cursor_merge_lines(prev_line, curr_line, true);
            
            if (result.success && result.before_cursor) {
                // Update previous line with merged content
                strcpy(E.lines[E.cursor_line - 1], result.before_cursor);
                E.cursor_col = result.new_position.position;
                
                // Remove current line
                for (int i = E.cursor_line; i < E.line_count - 1; i++) {
                    strcpy(E.lines[i], E.lines[i + 1]);
                }
                E.line_count--;
                E.cursor_line--;
                
                editor_set_status_message("Smart merge: cursor at position %d", E.cursor_col);
            } else {
                // Fallback to simple merge
                int prev_len = strlen(prev_line);
                strcat(prev_line, curr_line);
                E.cursor_col = prev_len;
                
                // Remove current line
                for (int i = E.cursor_line; i < E.line_count - 1; i++) {
                    strcpy(E.lines[i], E.lines[i + 1]);
                }
                E.line_count--;
                E.cursor_line--;
                
                editor_set_status_message("Simple merge");
            }
            
            cursor_free_result(&result);
            E.dirty = 1;
        }
    }
}

void editor_insert_newline() {
    if (E.cursor_line >= E.line_count) {
        strcpy(E.lines[E.line_count], "");
        E.line_count++;
        return;
    }
    
    char *line = E.lines[E.cursor_line];
    
    // Use C cursor management for smart Enter handling
    cursor_operation_result_t result = cursor_handle_enter_key(E.cursor_col, line, true);
    
    if (result.success) {
        // Update current line with before_cursor content
        if (result.before_cursor) {
            strcpy(E.lines[E.cursor_line], result.before_cursor);
        }
        
        // Insert new line with after_cursor content
        for (int i = E.line_count; i > E.cursor_line; i--) {
            strcpy(E.lines[i], E.lines[i - 1]);
        }
        E.line_count++;
        E.cursor_line++;
        
        if (result.after_cursor) {
            strcpy(E.lines[E.cursor_line], result.after_cursor);
        } else {
            strcpy(E.lines[E.cursor_line], "");
        }
        
        E.cursor_col = result.new_position.position;
        
        editor_set_status_message("Smart split: \"%s\" | \"%s\"", 
            result.before_cursor ? result.before_cursor : "",
            result.after_cursor ? result.after_cursor : "");
    } else {
        // Fallback to simple split
        
        // Make room for new line
        for (int i = E.line_count; i > E.cursor_line; i--) {
            strcpy(E.lines[i], E.lines[i - 1]);
        }
        E.line_count++;
        
        // Split line
        strcpy(E.lines[E.cursor_line + 1], &line[E.cursor_col]);
        line[E.cursor_col] = '\0';
        
        E.cursor_line++;
        E.cursor_col = 0;
        
        editor_set_status_message("Simple split");
    }
    
    cursor_free_result(&result);
    E.dirty = 1;
}

void editor_move_cursor(int key) {
    char *line = (E.cursor_line >= E.line_count) ? NULL : E.lines[E.cursor_line];
    
    switch (key) {
        case 1003: // Left arrow
            if (E.cursor_col != 0) {
                E.cursor_col--;
            } else if (E.cursor_line > 0) {
                E.cursor_line--;
                E.cursor_col = strlen(E.lines[E.cursor_line]);
            }
            break;
        case 1002: // Right arrow
            if (line && E.cursor_col < (int)strlen(line)) {
                E.cursor_col++;
            } else if (E.cursor_line < E.line_count - 1) {
                E.cursor_line++;
                E.cursor_col = 0;
            }
            break;
        case 1000: // Up arrow
            if (E.cursor_line > 0) {
                E.cursor_line--;
                int len = strlen(E.lines[E.cursor_line]);
                if (E.cursor_col > len) E.cursor_col = len;
            }
            break;
        case 1001: // Down arrow
            if (E.cursor_line < E.line_count - 1) {
                E.cursor_line++;
                int len = strlen(E.lines[E.cursor_line]);
                if (E.cursor_col > len) E.cursor_col = len;
            }
            break;
        case 1004: // Home
            E.cursor_col = 0;
            break;
        case 1005: // End
            if (line) E.cursor_col = strlen(line);
            break;
    }
}

// File operations
void editor_save_file() {
    if (strlen(E.filename) == 0) {
        strcpy(E.filename, "untitled.md");
    }
    
    FILE *fp = fopen(E.filename, "w");
    if (!fp) {
        editor_set_status_message("Error: Cannot save file '%s': %s", E.filename, strerror(errno));
        return;
    }
    
    int bytes_written = 0;
    for (int i = 0; i < E.line_count; i++) {
        int len = strlen(E.lines[i]);
        if (fwrite(E.lines[i], 1, len, fp) != (size_t)len) {
            fclose(fp);
            editor_set_status_message("Error writing file");
            return;
        }
        bytes_written += len;
        if (i < E.line_count - 1) {
            if (fwrite("\n", 1, 1, fp) != 1) {
                fclose(fp);
                editor_set_status_message("Error writing newline");
                return;
            }
            bytes_written++;
        }
    }
    
    fclose(fp);
    E.dirty = 0;
    editor_set_status_message("Saved %d bytes to '%s'", bytes_written, E.filename);
}

// Help display
void editor_show_help() {
    // Clear screen and show help
    write(STDOUT_FILENO, "\x1b[2J", 4);  // Clear screen
    write(STDOUT_FILENO, "\x1b[H", 3);   // Move cursor to top
    
    char help[] = 
        "\x1b[1mTUI Editor Enhanced - Help\x1b[0m\r\n\r\n"
        "\x1b[1mNavigation:\x1b[0m\r\n"
        "  Arrow Keys       - Move cursor\r\n"
        "  Home/End         - Beginning/End of line\r\n"
        "  Page Up/Down     - Scroll by page\r\n\r\n"
        "\x1b[1mEditing:\x1b[0m\r\n"
        "  Enter            - Smart line split (preserves formatting)\r\n"
        "  Backspace        - Smart delete/merge\r\n"
        "  Tab              - Insert tab/spaces\r\n\r\n"
        "\x1b[1mFile Operations:\x1b[0m\r\n"
        "  Ctrl+S           - Save file\r\n"
        "  Ctrl+O           - Open file\r\n\r\n"
        "\x1b[1mView Options:\x1b[0m\r\n"
        "  Ctrl+L           - Toggle line numbers\r\n\r\n"
        "\x1b[1mSystem:\x1b[0m\r\n"
        "  Ctrl+H           - Show this help\r\n"
        "  Ctrl+Q           - Quit\r\n"
        "  Ctrl+C           - Force quit\r\n\r\n"
        "\x1b[1mSmart Cursor Features:\x1b[0m\r\n"
        "  - Intelligent markdown formatting detection\r\n"
        "  - Smart Enter key preserves **bold**, *italic*, ==highlight==\r\n"
        "  - Smart Backspace reconnects split formatting\r\n"
        "  - Real-time cursor context in status bar\r\n\r\n"
        "\x1b[3mPress any key to return to editor...\x1b[0m";
    
    write(STDOUT_FILENO, help, strlen(help));
    
    // Wait for keypress
    read_key();
}

// Toggle line numbers
void editor_toggle_line_numbers() {
    E.show_line_numbers = !E.show_line_numbers;
    editor_set_status_message("Line numbers %s", E.show_line_numbers ? "ON" : "OFF");
}

// Handle tab insertion  
void editor_insert_tab() {
    for (int i = 0; i < TAB_STOP; i++) {
        editor_insert_char(' ');
    }
}

void editor_process_keypress() {
    int c = read_key();
    
    switch (c) {
        case CTRL_KEY('q'):
            if (E.dirty) {
                editor_set_status_message("File has unsaved changes. Save first or press Ctrl+Q again to quit");
                // Simple quit confirmation - press Ctrl+Q twice to quit without saving
                int c2 = read_key();
                if (c2 == CTRL_KEY('q')) {
                    write(STDOUT_FILENO, "\x1b[2J", 4);
                    write(STDOUT_FILENO, "\x1b[H", 3);
                    exit(0);
                }
                return;
            }
            write(STDOUT_FILENO, "\x1b[2J", 4);
            write(STDOUT_FILENO, "\x1b[H", 3);
            exit(0);
            break;
            
        case CTRL_KEY('s'):
            editor_save_file();
            break;
            
        case CTRL_KEY('h'):
            editor_show_help();
            break;
            
        case CTRL_KEY('l'):
            editor_toggle_line_numbers();
            break;
            
        case '\r':
            editor_insert_newline();
            break;
            
        case '\t':  // Tab key
            editor_insert_tab();
            break;
            
        case 127: // Backspace
        case CTRL_KEY('?'):  // Alternative backspace
            editor_delete_char();
            break;
            
        case 1000: // Up
        case 1001: // Down  
        case 1002: // Right
        case 1003: // Left
        case 1004: // Home
        case 1005: // End
            editor_move_cursor(c);
            break;
            
        case '\x1b':
            // Escape key - do nothing for now
            break;
            
        default:
            if (c >= 32 && c < 127) {
                editor_insert_char(c);
            }
            break;
    }
}

int main() {
    enable_raw_mode();
    tui_editor_init();
    
    while (1) {
        editor_refresh_screen();
        editor_process_keypress();
    }
    
    return 0;
}