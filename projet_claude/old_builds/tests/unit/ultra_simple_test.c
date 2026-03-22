#include <stdio.h>
#include <string.h>
#include <stdbool.h>

int simple_parse(const char *text) {
    if (!text) return 0;
    
    size_t len = strlen(text);
    if (len == 0) return 0;
    
    size_t i = 0;
    int iterations = 0;
    
    printf("Parsing text: '%s' (len=%zu)\n", text, len);
    
    while (i < len && iterations < 20) {  // safety limit
        printf("i=%zu, char='%c' (0x%02x)\n", i, text[i], (unsigned char)text[i]);
        
        bool matched = false;
        
        if (i + 2 < len && text[i] == '*' && text[i+1] == '*' && text[i+2] == '*') {
            printf("  Found *** at %zu\n", i);
            matched = true;
            i += 3;  // skip for now
        } else if (i + 1 < len && text[i] == '*' && text[i+1] == '*') {
            printf("  Found ** at %zu\n", i);
            matched = true;
            i += 2;  // skip for now
        } else if (text[i] == '*') {
            printf("  Found * at %zu\n", i);
            matched = true;
            i++;  // skip for now
        } else if (i + 1 < len && text[i] == '=' && text[i+1] == '=') {
            printf("  Found == at %zu\n", i);
            matched = true;
            i += 2;  // skip for now
        } else if (i + 1 < len && text[i] == '+' && text[i+1] == '+') {
            printf("  Found ++ at %zu\n", i);
            matched = true;
            i += 2;  // skip for now
        }
        
        if (!matched) {
            printf("  No match, advancing\n");
            i++;
        }
        
        iterations++;
    }
    
    if (iterations >= 20) {
        printf("LOOP DETECTED!\n");
        return -1;
    }
    
    printf("Parse completed successfully\n");
    return 0;
}

int main(void) {
    return simple_parse("Hello World");
}