#include "src/markdown.h"
#include <stdio.h>

int main(void) {
    printf("Calling parse_inline_styles with 'Hello World'\n");
    
    InlineSpan spans[32];
    int span_count = parse_inline_styles("Hello World", spans, 32);
    
    printf("Result: %d spans\n", span_count);
    return 0;
}