#include "src/editor.h"
#include "src/markdown.h"
#include "src/json.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    Document doc;
    editor_init(&doc);
    
    printf("Testing table separator detection:\n");
    
    // Test table separator detection
    const char *sep = "| --- | --- | --- |";
    printf("Is '%s' a table separator? %s\n", sep, is_table_separator_line(sep) ? "YES" : "NO");
    
    // Test pipe detection
    const char *header = "| Name | Age | City |";
    printf("Header line: '%s'\n", header);
    
    // Simulate header input
    for (size_t i = 0; i < strlen(header); i++) {
        editor_feed_char(&doc, header[i]);
    }
    editor_commit_line(&doc);
    
    printf("After header commit - elements: %zu\n", doc.elements_len);
    if (doc.elements_len > 0) {
        printf("  Element 0 type: %d\n", doc.elements[0].kind);
        if (doc.elements[0].kind == T_TEXT) {
            printf("  Text content: '%s'\n", doc.elements[0].as.text.text ? doc.elements[0].as.text.text : "NULL");
        }
    }
    
    // Simulate separator input
    for (size_t i = 0; i < strlen(sep); i++) {
        editor_feed_char(&doc, sep[i]);
    }
    printf("About to commit separator line...\n");
    editor_commit_line(&doc);
    
    printf("After separator commit - elements: %zu\n", doc.elements_len);
    if (doc.elements_len > 0) {
        printf("  Element 0 type: %d (T_TEXT=0, T_IMAGE=1, T_TABLE=2)\n", doc.elements[0].kind);
        if (doc.elements[0].kind == T_TABLE) {
            ElementTable *table = &doc.elements[0].as.table;
            printf("  Table: %zu rows, %zu cols\n", table->rows, table->cols);
        }
    }
    
    doc_free(&doc);
    return 0;
}