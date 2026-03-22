#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "cursor_manager.h"

#define MAX_LINES 100
#define MAX_LINE_LENGTH 1024

// Editor state
typedef struct {
    char lines[MAX_LINES][MAX_LINE_LENGTH];
    int line_count;
    int cursor_line;
    int cursor_col;
} scriptable_editor_t;

static scriptable_editor_t editor = {0};

// Initialize editor with test content
void init_editor() {
    editor.cursor_line = 0;
    editor.cursor_col = 0;
    editor.line_count = 6;
    
    strcpy(editor.lines[0], "# TUI Editor - Test Cursor Management");
    strcpy(editor.lines[1], "");
    strcpy(editor.lines[2], "- *Italique* test");
    strcpy(editor.lines[3], "- **Gras** test");
    strcpy(editor.lines[4], "- ==Surligné== test");
    strcpy(editor.lines[5], "- ++Souligné++ test");
}

// Display current state
void display_state() {
    printf("\n" "┌─────────────────────────────────────────────────────────┐\n");
    printf("│ ÉDITEUR TUI SCRIPTABLE - État Actuel                   │\n");
    printf("└─────────────────────────────────────────────────────────┘\n");
    
    for (int i = 0; i < editor.line_count; i++) {
        char prefix[10];
        if (i == editor.cursor_line) {
            sprintf(prefix, "→%2d", i + 1);
        } else {
            sprintf(prefix, " %2d", i + 1);
        }
        
        printf("%s│ %s", prefix, editor.lines[i]);
        
        // Show cursor position on current line
        if (i == editor.cursor_line) {
            printf("\n     ");
            for (int j = 0; j < editor.cursor_col + 2; j++) {
                printf(" ");
            }
            printf("↑ (col %d)", editor.cursor_col);
        }
        printf("\n");
    }
    
    // Show formatting context at cursor
    const char* current_line = editor.lines[editor.cursor_line];
    formatting_context_t ctx = cursor_analyze_formatting(current_line, editor.cursor_col);
    
    const char* format_type = "";
    switch(ctx.type) {
        case MARKER_NONE: format_type = "NONE"; break;
        case MARKER_BOLD: format_type = "BOLD"; break;
        case MARKER_ITALIC: format_type = "ITALIC"; break;
        case MARKER_HIGHLIGHT: format_type = "HIGHLIGHT"; break;
        case MARKER_UNDERLINE: format_type = "UNDERLINE"; break;
        case MARKER_HEADER: format_type = "HEADER"; break;
    }
    
    printf("\n📍 Curseur: Ligne %d, Col %d | Formatage: %s%s\n", 
           editor.cursor_line + 1, editor.cursor_col + 1, 
           format_type, ctx.inside_marker ? " (INSIDE)" : "");
    
    printf("═══════════════════════════════════════════════════════════\n");
}

// Movement commands
void move_cursor(int line, int col) {
    if (line >= 0 && line < editor.line_count) {
        editor.cursor_line = line;
        int line_length = strlen(editor.lines[line]);
        if (col >= 0 && col <= line_length) {
            editor.cursor_col = col;
        } else if (col < 0) {
            editor.cursor_col = 0;
        } else {
            editor.cursor_col = line_length;
        }
        printf("🚶 Curseur déplacé à: ligne %d, colonne %d\n", line + 1, col);
    }
}

void move_to_middle_of_formatting(int line) {
    if (line >= 0 && line < editor.line_count) {
        const char* text = editor.lines[line];
        int len = strlen(text);
        
        // Find formatting in the line
        for (int pos = 0; pos < len; pos++) {
            formatting_context_t ctx = cursor_analyze_formatting(text, pos);
            if (ctx.inside_marker && ctx.type != MARKER_NONE) {
                // Found formatting, go to middle
                int middle = (ctx.start_pos + ctx.end_pos) / 2;
                move_cursor(line, middle);
                printf("🎯 Curseur placé au centre du formatage %s\n", 
                       ctx.type == MARKER_BOLD ? "BOLD" :
                       ctx.type == MARKER_ITALIC ? "ITALIC" :
                       ctx.type == MARKER_HIGHLIGHT ? "HIGHLIGHT" : "UNKNOWN");
                return;
            }
        }
        printf("⚠️ Aucun formatage trouvé sur la ligne %d\n", line + 1);
    }
}

// Simulate Enter key
void press_enter() {
    printf("\n🔑 Appui sur ENTRÉE...\n");
    
    const char* current_line = editor.lines[editor.cursor_line];
    cursor_operation_result_t result = cursor_handle_enter_key(editor.cursor_col, current_line, true);
    
    if (result.success) {
        // Update current line
        if (result.before_cursor) {
            strcpy(editor.lines[editor.cursor_line], result.before_cursor);
        }
        
        // Insert new line
        for (int i = editor.line_count; i > editor.cursor_line; i--) {
            strcpy(editor.lines[i], editor.lines[i - 1]);
        }
        editor.line_count++;
        editor.cursor_line++;
        
        if (result.after_cursor) {
            strcpy(editor.lines[editor.cursor_line], result.after_cursor);
        } else {
            strcpy(editor.lines[editor.cursor_line], "");
        }
        
        editor.cursor_col = result.new_position.position;
        
        printf("✅ Division réussie:\n");
        printf("   Ligne précédente: \"%s\"\n", result.before_cursor ? result.before_cursor : "");
        printf("   Nouvelle ligne: \"%s\"\n", result.after_cursor ? result.after_cursor : "");
        printf("   Curseur à: col %d\n", editor.cursor_col);
    } else {
        printf("❌ Échec de la division: %s\n", result.error_message ? result.error_message : "Erreur inconnue");
    }
    
    cursor_free_result(&result);
}

// Simulate Backspace
void press_backspace() {
    printf("\n🔑 Appui sur BACKSPACE...\n");
    
    if (editor.cursor_col > 0) {
        // Delete character
        char* line = editor.lines[editor.cursor_line];
        int len = strlen(line);
        memmove(&line[editor.cursor_col - 1], &line[editor.cursor_col], len - editor.cursor_col + 1);
        editor.cursor_col--;
        printf("🔤 Caractère supprimé\n");
    } else if (editor.cursor_line > 0) {
        // Merge lines
        char* prev_line = editor.lines[editor.cursor_line - 1];
        char* curr_line = editor.lines[editor.cursor_line];
        
        cursor_operation_result_t result = cursor_merge_lines(prev_line, curr_line, true);
        
        if (result.success && result.before_cursor) {
            strcpy(editor.lines[editor.cursor_line - 1], result.before_cursor);
            editor.cursor_col = result.new_position.position;
            
            // Remove current line
            for (int i = editor.cursor_line; i < editor.line_count - 1; i++) {
                strcpy(editor.lines[i], editor.lines[i + 1]);
            }
            editor.line_count--;
            editor.cursor_line--;
            
            printf("✅ Fusion réussie: \"%s\" (curseur à col %d)\n", 
                   result.before_cursor, editor.cursor_col);
        } else {
            printf("❌ Échec de la fusion\n");
        }
        
        cursor_free_result(&result);
    }
}

// Type text
void type_text(const char* text) {
    printf("\n⌨️  Frappe: \"%s\"\n", text);
    
    char* line = editor.lines[editor.cursor_line];
    int len = strlen(line);
    int text_len = strlen(text);
    
    // Make room for new text
    memmove(&line[editor.cursor_col + text_len], &line[editor.cursor_col], len - editor.cursor_col + 1);
    
    // Insert text
    memcpy(&line[editor.cursor_col], text, text_len);
    editor.cursor_col += text_len;
}

// Execute a script of commands
void execute_script() {
    printf("🎬 DÉMARRAGE DU SCRIPT D'INTERACTION\n");
    
    display_state();
    
    printf("\n📝 Test 1: Aller au centre de '**Gras**' et appuyer sur Entrée\n");
    move_to_middle_of_formatting(3); // Line with "- **Gras** test"
    display_state();
    
    press_enter();
    display_state();
    
    printf("\n📝 Test 2: Supprimer pour fusionner les lignes\n");
    press_backspace();
    display_state();
    
    printf("\n📝 Test 3: Aller au centre de '*Italique*' et diviser\n");
    move_to_middle_of_formatting(2); // Line with "- *Italique* test"
    display_state();
    
    press_enter();
    display_state();
    
    printf("\n📝 Test 4: Taper du texte\n");
    type_text("NOUVEAU");
    display_state();
    
    printf("\n📝 Test 5: Fusionner à nouveau\n");
    move_cursor(editor.cursor_line + 1, 0); // Aller au début de la ligne suivante
    press_backspace();
    display_state();
    
    printf("\n🎊 SCRIPT TERMINÉ!\n");
}

int main(int argc, char* argv[]) {
    init_editor();
    
    if (argc > 1 && strcmp(argv[1], "interactive") == 0) {
        printf("Mode interactif non supporté dans cet environnement.\n");
        printf("Utilisation: %s [script]\n", argv[0]);
        return 1;
    }
    
    execute_script();
    return 0;
}