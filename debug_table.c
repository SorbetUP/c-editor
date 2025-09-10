#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "src/editor.h"
#include "src/markdown.h"

int main(void) {
    const char *markdown = "| A | B | C |\n|---|---|---|\n| 1 | 2 |   |";
    
    printf("Input markdown: '%s'\n", markdown);
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    
    printf("Parse result: %d\n", result);
    printf("Elements count: %zu (expected: 1)\n", doc.elements_len);
    
    for (size_t i = 0; i < doc.elements_len; i++) {
        printf("Element %zu: kind=%d\n", i, doc.elements[i].kind);
        if (doc.elements[i].kind == T_TABLE) {
            ElementTable *table = &doc.elements[i].as.table;
            printf("  rows: %d\n", table->rows);
            printf("  cols: %d\n", table->cols);
        }
    }
    
    doc_free(&doc);
    return 0;
}
