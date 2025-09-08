#include "src/editor.h"
#include "src/markdown.h"
#include <stdio.h>

int main() {
    Document doc = {0};
    int result = markdown_to_json("==**==", &doc);
    printf("Parse result: %d\n", result);
    
    if (doc.elements_len > 0 && doc.elements[0].kind == T_TEXT) {
        ElementText *text = &doc.elements[0].as.text;
        printf("Spans: %zu\n", text->spans_count);
        
        for (size_t s = 0; s < text->spans_count; s++) {
            const char *span_text = text->spans[s].text ? text->spans[s].text : "NULL";
            printf("  [%zu] '%s' (bold=%d, highlight=%d)\n", s, span_text, 
                   text->spans[s].bold, text->spans[s].has_highlight);
        }
    }
    
    doc_free(&doc);
    return 0;
}
