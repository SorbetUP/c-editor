#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <signal.h>
#include <ctype.h>
#include <errno.h>
#include "cursor_manager.h"
#include "markdown.h"

#define MAX_LINES 1000
#define MAX_LINE_LENGTH 4096
#define CTRL_KEY(k) ((k) & 0x1f)

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
    int dirty;
} editor_state_t;

static editor_state_t E = {0};

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
    
    snprintf(E.status_msg, sizeof(E.status_msg), "TUI Editor | C Cursor Management Test | Ctrl-Q to quit");
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

void editor_draw_rows(char *buf, int *len) {
    int y;
    for (y = 0; y < E.screen_rows; y++) {
        int filerow = y + E.row_offset;
        if (filerow >= E.line_count) {
            if (E.line_count == 0 && y == E.screen_rows / 3) {
                char welcome[80];
                int welcomelen = snprintf(welcome, sizeof(welcome),
                    "TUI Editor -- C Cursor Management Test");
                if (welcomelen > E.screen_cols) welcomelen = E.screen_cols;
                int padding = (E.screen_cols - welcomelen) / 2;
                if (padding) {
                    *len += snprintf(buf + *len, 4096 - *len, "~");
                    padding--;
                }
                while (padding--) *len += snprintf(buf + *len, 4096 - *len, " ");
                *len += snprintf(buf + *len, 4096 - *len, "%s", welcome);
            } else {
                *len += snprintf(buf + *len, 4096 - *len, "~");
            }
        } else {
            int len_line = strlen(E.lines[filerow]);
            if (len_line < E.col_offset) len_line = 0;
            else len_line -= E.col_offset;
            if (len_line > E.screen_cols) len_line = E.screen_cols;
            
            // Highlight current line
            if (filerow == E.cursor_line) {
                *len += snprintf(buf + *len, 4096 - *len, "\x1b[7m"); // Reverse video
            }
            
            *len += snprintf(buf + *len, 4096 - *len, "%.*s", len_line, 
                           E.lines[filerow] + E.col_offset);
            
            if (filerow == E.cursor_line) {
                *len += snprintf(buf + *len, 4096 - *len, "\x1b[m"); // Reset
            }
        }
        
        *len += snprintf(buf + *len, 4096 - *len, "\x1b[K"); // Clear line
        *len += snprintf(buf + *len, 4096 - *len, "\r\n");
    }
}

void editor_draw_status_bar(char *buf, int *len) {
    *len += snprintf(buf + *len, 4096 - *len, "\x1b[7m"); // Reverse video
    
    char status[256];
    char rstatus[256];
    
    // Test cursor management functions
    const char *current_line = E.lines[E.cursor_line];
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
    
    int len_status = snprintf(status, sizeof(status), "%.20s - %d lines %s",
                            "tui_test.md", E.line_count, E.dirty ? "(modified)" : "");
    int len_rstatus = snprintf(rstatus, sizeof(rstatus), 
                             "L%d,C%d | %s%s", 
                             E.cursor_line + 1, E.cursor_col + 1,
                             fmt_type, ctx.inside_marker ? " (INSIDE)" : "");
    
    if (len_status > E.screen_cols) len_status = E.screen_cols;
    *len += snprintf(buf + *len, 4096 - *len, "%s", status);
    
    while (len_status < E.screen_cols) {
        if (E.screen_cols - len_status == len_rstatus) {
            *len += snprintf(buf + *len, 4096 - *len, "%s", rstatus);
            break;
        } else {
            *len += snprintf(buf + *len, 4096 - *len, " ");
            len_status++;
        }
    }
    *len += snprintf(buf + *len, 4096 - *len, "\x1b[m"); // Reset
    *len += snprintf(buf + *len, 4096 - *len, "\r\n");
}

void editor_draw_message_bar(char *buf, int *len) {
    *len += snprintf(buf + *len, 4096 - *len, "\x1b[K");
    int msglen = strlen(E.status_msg);
    if (msglen > E.screen_cols) msglen = E.screen_cols;
    *len += snprintf(buf + *len, 4096 - *len, "%.*s", msglen, E.status_msg);
}

void editor_refresh_screen() {
    editor_scroll();
    
    char buf[4096] = {0};
    int len = 0;
    
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[?25l"); // Hide cursor
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[H"); // Move cursor to top
    
    editor_draw_rows(buf, &len);
    editor_draw_status_bar(buf, &len);  
    editor_draw_message_bar(buf, &len);
    
    // Position cursor
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[%d;%dH", 
                   (E.cursor_line - E.row_offset) + 1,
                   (E.cursor_col - E.col_offset) + 1);
    
    len += snprintf(buf + len, sizeof(buf) - len, "\x1b[?25h"); // Show cursor
    
    write(STDOUT_FILENO, buf, len);
}

void editor_insert_char(int c) {
    if (E.cursor_line == E.line_count) {
        // Add new line if at end
        strcpy(E.lines[E.line_count], "");
        E.line_count++;
    }
    
    char *line = E.lines[E.cursor_line];
    int len = strlen(line);
    
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
                
                snprintf(E.status_msg, sizeof(E.status_msg), 
                    "Smart merge: cursor at position %d", E.cursor_col);
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
                
                snprintf(E.status_msg, sizeof(E.status_msg), "Simple merge");
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
        
        snprintf(E.status_msg, sizeof(E.status_msg), 
            "Smart split: \"%s\" | \"%s\"", 
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
        
        snprintf(E.status_msg, sizeof(E.status_msg), "Simple split");
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

void editor_process_keypress() {
    int c = read_key();
    
    switch (c) {
        case CTRL_KEY('q'):
            write(STDOUT_FILENO, "\x1b[2J", 4);
            write(STDOUT_FILENO, "\x1b[H", 3);
            exit(0);
            break;
            
        case '\r':
            editor_insert_newline();
            break;
            
        case 127: // Backspace
        case CTRL_KEY('h'):
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