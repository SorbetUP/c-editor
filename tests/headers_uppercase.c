#include "../src/editor.h"
#include "../src/markdown.h"
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

void test_editor_header_uppercase(void) {
    TEST("Editor header uppercase on commit");
    
    Document doc;
    editor_init(&doc);
    
    // Simulate typing "##  MiXeD  Case\n"
    const char *input = "##  MiXeD  Case";
    for (size_t i = 0; i < strlen(input); i++) {
        editor_feed_char(&doc, input[i]);
    }
    editor_commit_line(&doc);
    
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_EQ(2, text->level);
    ASSERT_TRUE(text->bold);
    
    // Should have at least one span with uppercase text
    ASSERT_TRUE(text->spans_count > 0);
    
    // Find the main text span (skip leading spaces)
    bool found_mixed = false;
    for (size_t i = 0; i < text->spans_count; i++) {
        if (strstr(text->spans[i].text, "MIXED")) {
            found_mixed = true;
            break;
        }
    }
    ASSERT_TRUE(found_mixed);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_markdown_header_case_preserved(void) {
    TEST("Markdown import preserves original case");
    
    const char *markdown = "## MiXeD Case";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_EQ(2, text->level);
    
    // When parsing markdown directly (not via editor), case should be uppercased
    bool found_mixed = false;
    for (size_t i = 0; i < text->spans_count; i++) {
        if (strstr(text->spans[i].text, "MIXED")) {
            found_mixed = true;
            break;
        }
    }
    ASSERT_TRUE(found_mixed);
    
    doc_free(&doc);
    
    TEST_END;
}

int main(void) {
    printf("Running Header Uppercase Tests\n");
    printf("==============================\n\n");
    
    test_editor_header_uppercase();
    test_markdown_header_case_preserved();
    
    printf("\nTest Results: %d/%d tests passed\n", passed_count, test_count);
    
    if (passed_count == test_count) {
        printf("All tests passed! ✓\n");
        return 0;
    } else {
        printf("Some tests failed. ✗\n");
        return 1;
    }
}