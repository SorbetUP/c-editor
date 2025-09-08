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

void test_inline_markers_removed(void) {
    TEST("Inline markers removed from text spans");
    
    const char *markdown = "Bonjour *monde* en **C** et ***Markdown***.";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    
    ElementText *text = &doc.elements[0].as.text;
    
    // Debug: print spans
    for (size_t i = 0; i < text->spans_count; i++) {
        printf("span[%zu] = '%s' (bold=%d, italic=%d)\n", i, text->spans[i].text, text->spans[i].bold, text->spans[i].italic);
    }
    
    ASSERT_EQ(7, text->spans_count);
    
    // Verify no markers in any span text
    for (size_t i = 0; i < text->spans_count; i++) {
        char *span_text = text->spans[i].text;
        ASSERT_TRUE(strstr(span_text, "*") == NULL);
        ASSERT_TRUE(strstr(span_text, "**") == NULL);
        ASSERT_TRUE(strstr(span_text, "***") == NULL);
    }
    
    // Verify content and styling
    ASSERT_STR_EQ("Bonjour ", text->spans[0].text);
    ASSERT_TRUE(!text->spans[0].bold && !text->spans[0].italic);
    
    ASSERT_STR_EQ("monde", text->spans[1].text);
    ASSERT_TRUE(!text->spans[1].bold && text->spans[1].italic);
    
    ASSERT_STR_EQ(" en ", text->spans[2].text);
    ASSERT_TRUE(!text->spans[2].bold && !text->spans[2].italic);
    
    ASSERT_STR_EQ("C", text->spans[3].text);
    ASSERT_TRUE(text->spans[3].bold && !text->spans[3].italic);
    
    ASSERT_STR_EQ(" et ", text->spans[4].text);
    ASSERT_TRUE(!text->spans[4].bold && !text->spans[4].italic);
    
    ASSERT_STR_EQ("Markdown", text->spans[5].text);
    ASSERT_TRUE(text->spans[5].bold && text->spans[5].italic);
    
    ASSERT_STR_EQ(".", text->spans[6].text);
    ASSERT_TRUE(!text->spans[6].bold && !text->spans[6].italic);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_highlight_underline_markers_removed(void) {
    TEST("Highlight and underline markers removed");
    
    const char *markdown = "==note== et ++important++";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_EQ(3, text->spans_count);
    
    // Verify no markers in any span text
    for (size_t i = 0; i < text->spans_count; i++) {
        char *span_text = text->spans[i].text;
        ASSERT_TRUE(strstr(span_text, "==") == NULL);
        ASSERT_TRUE(strstr(span_text, "++") == NULL);
    }
    
    ASSERT_STR_EQ("note", text->spans[0].text);
    ASSERT_TRUE(text->spans[0].has_highlight);
    
    ASSERT_STR_EQ(" et ", text->spans[1].text);
    ASSERT_TRUE(!text->spans[1].has_highlight && !text->spans[1].has_underline);
    
    ASSERT_STR_EQ("important", text->spans[2].text);
    ASSERT_TRUE(text->spans[2].has_underline);
    
    doc_free(&doc);
    
    TEST_END;
}

int main(void) {
    printf("Running Inline Marker Removal Tests\n");
    printf("===================================\n\n");
    
    test_inline_markers_removed();
    test_highlight_underline_markers_removed();
    
    printf("\nTest Results: %d/%d tests passed\n", passed_count, test_count);
    
    if (passed_count == test_count) {
        printf("All tests passed! ✓\n");
        return 0;
    } else {
        printf("Some tests failed. ✗\n");
        return 1;
    }
}