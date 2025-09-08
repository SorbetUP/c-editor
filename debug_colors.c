#include "src/editor.h"
#include "src/json.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    Document doc;
    editor_init(&doc);
    
    // Add text with inline styles to get all colors
    const char *line = "Hello ==highlighted== and ++underlined++ world";
    for (size_t i = 0; i < strlen(line); i++) {
        editor_feed_char(&doc, line[i]);
    }
    editor_commit_line(&doc);
    
    printf("Default colors in doc:\n");
    printf("  text_color: [%.3f,%.3f,%.3f,%.3f]\n", 
           doc.default_text_color.r, doc.default_text_color.g,
           doc.default_text_color.b, doc.default_text_color.a);
    printf("  underline_color: [%.3f,%.3f,%.3f,%.3f]\n",
           doc.default_underline_color.r, doc.default_underline_color.g,
           doc.default_underline_color.b, doc.default_underline_color.a);
    printf("  highlight_color: [%.3f,%.3f,%.3f,%.3f]\n",
           doc.default_highlight_color.r, doc.default_highlight_color.g,
           doc.default_highlight_color.b, doc.default_highlight_color.a);
    
    char *json;
    int result = json_stringify(&doc, &json);
    if (result == 0 && json) {
        printf("\nFull JSON output:\n%s\n", json);
        free(json);
    } else {
        printf("Failed to generate JSON\n");
    }
    
    doc_free(&doc);
    return 0;
}