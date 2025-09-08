#include "src/editor.h"
#include "src/markdown.h"
#include "src/json.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    // Test simple pour voir les marqueurs
    Document doc = {0};
    int result = markdown_to_json("**bold** text", &doc);
    printf("Import result: %d\n", result);
    
    if (doc.elements_len > 0 && doc.elements[0].kind == T_TEXT) {
        ElementText *text = &doc.elements[0].as.text;
        printf("Spans count: %zu\n", text->spans_count);
        
        for (size_t i = 0; i < text->spans_count; i++) {
            printf("Span[%zu]: '%s' (bold=%d, italic=%d)\n", 
                   i, text->spans[i].text ? text->spans[i].text : "NULL",
                   text->spans[i].bold, text->spans[i].italic);
        }
    }
    
    doc_free(&doc);
    return 0;
}