#include <stdio.h>
#include <string.h>
#include "cursor_manager.h"

void print_separator() {
    printf("\n" "═══════════════════════════════════════════════════════\n");
}

void test_formatting_detection() {
    printf("🔍 TEST: Détection de formatage\n");
    
    const char* test_strings[] = {
        "**Gras**",
        "*Italique*", 
        "==Surligné==",
        "++Souligné++",
        "# Titre",
        "Normal text"
    };
    
    for (int i = 0; i < 6; i++) {
        const char* text = test_strings[i];
        int len = strlen(text);
        
        printf("\nTexte: \"%s\"\n", text);
        
        for (int pos = 0; pos <= len; pos++) {
            formatting_context_t ctx = cursor_analyze_formatting(text, pos);
            const char* type_name = "";
            
            switch(ctx.type) {
                case MARKER_NONE: type_name = "NONE"; break;
                case MARKER_BOLD: type_name = "BOLD"; break;
                case MARKER_ITALIC: type_name = "ITALIC"; break;
                case MARKER_HIGHLIGHT: type_name = "HIGHLIGHT"; break;
                case MARKER_UNDERLINE: type_name = "UNDERLINE"; break;
                case MARKER_HEADER: type_name = "HEADER"; break;
            }
            
            printf("  Pos %2d: %s%s\n", pos, type_name, 
                   ctx.inside_marker ? " (INSIDE)" : "");
        }
    }
}

void test_enter_key_handling() {
    printf("\n🎯 TEST: Gestion de la touche Entrée\n");
    
    struct {
        const char* text;
        int position;
        const char* expected_before;
        const char* expected_after;
    } tests[] = {
        {"**Gras**", 4, "**Gr", "as**"},
        {"*Italique*", 5, "*Ital", "ique*"},
        {"==Surligné==", 6, "==Surli", "gné=="},
        {"- *Item*", 4, "- *It", "em*"},
        {"Normal text", 6, "Normal", " text"}
    };
    
    for (int i = 0; i < 5; i++) {
        printf("\nTest %d: \"%s\" à la position %d\n", 
               i+1, tests[i].text, tests[i].position);
        
        cursor_operation_result_t result = cursor_handle_enter_key(
            tests[i].position, tests[i].text, true);
        
        if (result.success) {
            printf("  ✅ Succès:\n");
            printf("    Avant: \"%s\"\n", result.before_cursor ? result.before_cursor : "");
            printf("    Après: \"%s\"\n", result.after_cursor ? result.after_cursor : "");
            printf("    Position: %d\n", result.new_position.position);
            
            // Vérification
            bool before_ok = (tests[i].expected_before && result.before_cursor && 
                             strcmp(tests[i].expected_before, result.before_cursor) == 0);
            bool after_ok = (tests[i].expected_after && result.after_cursor &&
                            strcmp(tests[i].expected_after, result.after_cursor) == 0);
            
            if (before_ok && after_ok) {
                printf("    🎉 Résultat conforme aux attentes!\n");
            } else {
                printf("    ⚠️  Différence détectée:\n");
                printf("       Attendu avant: \"%s\"\n", tests[i].expected_before);
                printf("       Attendu après: \"%s\"\n", tests[i].expected_after);
            }
        } else {
            printf("  ❌ Échec: %s\n", result.error_message ? result.error_message : "Erreur inconnue");
        }
        
        cursor_free_result(&result);
    }
}

void test_line_merging() {
    printf("\n🔗 TEST: Fusion de lignes\n");
    
    struct {
        const char* line1;
        const char* line2;
        const char* expected_result;
        int expected_cursor;
    } tests[] = {
        {"**Gr", "as**", "**Gras**", 4},
        {"*It", "aly*", "*Italy*", 3},
        {"==Surli", "gné==", "==Surligné==", 7},
        {"Hello", " World", "Hello World", 5},
        {"", "Text", "Text", 0}
    };
    
    for (int i = 0; i < 5; i++) {
        printf("\nTest %d: \"%s\" + \"%s\"\n", 
               i+1, tests[i].line1, tests[i].line2);
        
        cursor_operation_result_t result = cursor_merge_lines(
            tests[i].line1, tests[i].line2, true);
        
        if (result.success) {
            printf("  ✅ Succès:\n");
            printf("    Résultat: \"%s\"\n", result.before_cursor ? result.before_cursor : "");
            printf("    Position curseur: %d\n", result.new_position.position);
            
            // Vérification
            bool result_ok = (result.before_cursor && 
                             strcmp(tests[i].expected_result, result.before_cursor) == 0);
            bool cursor_ok = (result.new_position.position == tests[i].expected_cursor);
            
            if (result_ok && cursor_ok) {
                printf("    🎉 Résultat conforme aux attentes!\n");
            } else {
                printf("    ⚠️  Différence détectée:\n");
                printf("       Attendu: \"%s\" (curseur à %d)\n", 
                       tests[i].expected_result, tests[i].expected_cursor);
            }
        } else {
            printf("  ❌ Échec: %s\n", result.error_message ? result.error_message : "Erreur inconnue");
        }
        
        cursor_free_result(&result);
    }
}

void test_position_adjustment() {
    printf("\n🔧 TEST: Ajustement de position\n");
    
    struct {
        const char* text;
        int input_pos;
        int expected_pos;
    } tests[] = {
        {"**Gras**", 3, 2},    // Au milieu des **, ajusté au début
        {"*Italique*", 6, 6},  // Au milieu du texte, pas d'ajustement
        {"==Text==", 1, 0},    // Au milieu de ==, ajusté au début
        {"Normal", 3, 3}       // Texte normal, pas d'ajustement
    };
    
    for (int i = 0; i < 4; i++) {
        printf("\nTest %d: \"%s\" position %d\n", 
               i+1, tests[i].text, tests[i].input_pos);
        
        cursor_position_t result = cursor_adjust_for_formatting(
            tests[i].input_pos, tests[i].text, true);
        
        if (result.is_valid) {
            printf("  Position ajustée: %d -> %d\n", 
                   tests[i].input_pos, result.position);
            
            if (result.position == tests[i].expected_pos) {
                printf("  ✅ Ajustement correct!\n");
            } else {
                printf("  ⚠️  Attendu: %d\n", tests[i].expected_pos);
            }
        } else {
            printf("  ❌ Ajustement invalide\n");
        }
    }
}

int main() {
    printf("🚀 DÉMONSTRATION - Bibliothèque C de Gestion de Curseur\n");
    printf("═══════════════════════════════════════════════════════\n");
    
    test_formatting_detection();
    print_separator();
    
    test_enter_key_handling();
    print_separator();
    
    test_line_merging();
    print_separator();
    
    test_position_adjustment();
    print_separator();
    
    printf("\n🎊 Tests terminés!\n");
    printf("Pour utiliser l'éditeur TUI interactif:\n");
    printf("  make -f Makefile.tui run\n");
    printf("  (Nécessite un terminal interactif)\n\n");
    
    return 0;
}