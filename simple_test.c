#include "src/editor.h"
#include <stdio.h>

int main(void) {
    Document doc;
    editor_init(&doc);
    
    printf("Document initialized successfully\n");
    
    const char *test = "# Hello World";
    int result = json_import_markdown(test, &doc);
    
    printf("Import result: %d\n", result);
    printf("Elements: %zu\n", doc.elements_len);
    
    if (doc.elements_len > 0) {
        printf("First element type: %d\n", doc.elements[0].kind);
        if (doc.elements[0].kind == T_TEXT) {
            printf("Text: %s\n", doc.elements[0].as.text.text);
            printf("Level: %d\n", doc.elements[0].as.text.level);
        }
    }
    
    doc_free(&doc);
    return 0;
}