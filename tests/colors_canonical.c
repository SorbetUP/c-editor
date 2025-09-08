#include "../src/editor.h"
#include "../src/json.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

static int test_count = 0;
static int passed_count = 0;

#define TEST(name) \
    do { \
        test_count++; \
        printf("Running test: %s... ", name); \
        fflush(stdout);

#define ASSERT_TRUE(condition) \
    do { \
        if (!(condition)) { \
            printf("FAILED\n  Assertion failed: %s\n", #condition); \
            return; \
        } \
    } while(0)

#define ASSERT_STR_EQ(expected, actual) \
    do { \
        if (!actual || strcmp((expected), (actual)) != 0) { \
            printf("FAILED\n  Expected: '%s'\n  Actual: '%s'\n", (expected), (actual) ? (actual) : "NULL"); \
            return; \
        } \
    } while(0)

#define ASSERT_EQ(expected, actual) \
    do { \
        if ((expected) != (actual)) { \
            printf("FAILED\n  Expected: %d\n  Actual: %d\n", (int)(expected), (int)(actual)); \
            return; \
        } \
    } while(0)

#define TEST_END \
        printf("PASSED\n"); \
        passed_count++; \
    } while(0)

void test_colors_canonical_format(void) {
    TEST("Colors output in canonical [0.0-1.0] format");
    
    Document doc;
    editor_init(&doc);
    
    // Add text with inline styles to get all colors
    const char *line = "Hello ==highlighted== and ++underlined++ world";
    for (size_t i = 0; i < strlen(line); i++) {
        editor_feed_char(&doc, line[i]);
    }
    editor_commit_line(&doc);
    
    char *json;
    int result = json_stringify(&doc, &json);
    ASSERT_EQ(0, result);
    ASSERT_TRUE(json != NULL);
    
    // Verify colors are in float format [0.000,0.000,0.000,1.000]
    ASSERT_TRUE(strstr(json, "[0.000,0.000,0.000,1.000]") != NULL);
    
    // Verify no 255 values or parentheses
    ASSERT_TRUE(strstr(json, "255") == NULL);
    ASSERT_TRUE(strstr(json, "(") == NULL);
    ASSERT_TRUE(strstr(json, ")") == NULL);
    
    doc_free(&doc);
    free(json);
    
    TEST_END;
}

void test_color_normalization(void) {
    TEST("Color values normalized to [0.0-1.0] range");
    
    Document doc;
    editor_init(&doc);
    
    // Force some out-of-range values
    doc.default_text_color.r = 1.5f;  // Should be clamped to 1.0
    doc.default_text_color.g = -0.5f; // Should be clamped to 0.0
    doc.default_text_color.b = 0.5f;  // Should remain 0.5
    doc.default_text_color.a = 2.0f;  // Should be clamped to 1.0
    
    // Add text to trigger color output
    const char *line = "Test text";
    for (size_t i = 0; i < strlen(line); i++) {
        editor_feed_char(&doc, line[i]);
    }
    editor_commit_line(&doc);
    
    char *json;
    int result = json_stringify(&doc, &json);
    ASSERT_EQ(0, result);
    
    // Verify clamping worked
    ASSERT_TRUE(strstr(json, "[1.000,0.000,0.500,1.000]") != NULL);
    
    doc_free(&doc);
    free(json);
    
    TEST_END;
}

int main(void) {
    printf("Running Color Format Tests\n");
    printf("==========================\n\n");
    
    test_colors_canonical_format();
    test_color_normalization();
    
    printf("\nTest Results: %d/%d tests passed\n", passed_count, test_count);
    
    if (passed_count == test_count) {
        printf("All tests passed! ✓\n");
        return 0;
    } else {
        printf("Some tests failed. ✗\n");
        return 1;
    }
}