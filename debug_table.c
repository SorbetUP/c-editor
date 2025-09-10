#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "src/editor.h"
#include "src/markdown.h"
#include "src/json.h"

int main() {
    const char* test_md = 
"| C | UTF-8 |\n"
"|---|---|\n"
"| *Bonjour | ==image |\n"
"| **UTF-8 | *titre |\n"
"|  | monde |\n";

    printf("=== ORIGINAL TABLE ===\n%s\n", test_md);

    Document d = {0};
    if (markdown_to_json(test_md, &d) != 0) {
        printf("Failed to parse markdown\n");
        return 1;
    }

    // Debug table structure
    for (size_t i = 0; i < d.elements_len; i++) {
        if (d.elements[i].kind == T_TABLE) {
            ElementTable *table = &d.elements[i].as.table;
            printf("Table: %zu rows, %zu cols\n", table->rows, table->cols);
            
            for (size_t r = 0; r < table->rows; r++) {
                printf("Row %zu: ", r);
                for (size_t c = 0; c < table->cols; c++) {
                    if (table->cells[r] && table->cells[r][c] && table->cells[r][c]->text) {
                        printf("'%s' ", table->cells[r][c]->text);
                    } else {
                        printf("(null) ");
                    }
                }
                printf("\n");
            }
        }
    }

    char *md_out = NULL;
    if (json_to_markdown(&d, &md_out) != 0) {
        printf("Failed to convert back\n");
        return 1;
    }

    printf("=== RECONSTRUCTED TABLE ===\n%s\n", md_out);

    doc_free(&d);
    free(md_out);
    return 0;
}
