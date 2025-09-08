#include "src/editor.h"
#include "src/markdown.h"
#include <stdio.h>

int main(void) {
    printf("Starting test...\n");
    fflush(stdout);
    
    Document doc;
    printf("About to init document...\n");
    fflush(stdout);
    
    editor_init(&doc);
    printf("Document initialized\n");
    fflush(stdout);
    
    printf("Testing simple text parsing...\n");
    const char *simple = "Hello World";
    
    printf("About to call markdown_to_json...\n");
    fflush(stdout);
    
    // Let's test inline parsing first
    InlineSpan spans[32];
    printf("Testing inline parsing...\n");
    fflush(stdout);
    
    int span_count = parse_inline_styles(simple, spans, 32);
    printf("Inline parsing done, spans: %d\n", span_count);
    fflush(stdout);
    
    int result = json_import_markdown(simple, &doc);
    
    printf("Result: %d\n", result);
    doc_free(&doc);
    return 0;
}