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

void test_table_creation(void) {
    TEST("Table creation from header and separator");
    
    Document doc;
    editor_init(&doc);
    
    // Input table header
    const char *header = "| Name | Age | City |";
    for (size_t i = 0; i < strlen(header); i++) {
        editor_feed_char(&doc, header[i]);
    }
    editor_commit_line(&doc);
    
    // Should have one text element
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    
    // Input table separator
    const char *separator = "| --- | --- | --- |";
    for (size_t i = 0; i < strlen(separator); i++) {
        editor_feed_char(&doc, separator[i]);
    }
    editor_commit_line(&doc);
    
    // Should now have one table element
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TABLE, doc.elements[0].kind);
    
    ElementTable *table = &doc.elements[0].as.table;
    ASSERT_EQ(3, table->cols);
    ASSERT_EQ(1, table->rows);
    
    // Check header cell contents
    ASSERT_STR_EQ("Name", table->cells[0][0]->text);
    ASSERT_STR_EQ("Age", table->cells[0][1]->text);
    ASSERT_STR_EQ("City", table->cells[0][2]->text);
    
    // Header should be bold
    ASSERT_TRUE(table->cells[0][0]->bold);
    ASSERT_TRUE(table->cells[0][1]->bold);
    ASSERT_TRUE(table->cells[0][2]->bold);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_table_json_output(void) {
    TEST("Table JSON serialization");
    
    Document doc;
    editor_init(&doc);
    
    // Create a table
    const char *header = "| Product | Price |";
    for (size_t i = 0; i < strlen(header); i++) {
        editor_feed_char(&doc, header[i]);
    }
    editor_commit_line(&doc);
    
    const char *separator = "| --- | --- |";
    for (size_t i = 0; i < strlen(separator); i++) {
        editor_feed_char(&doc, separator[i]);
    }
    editor_commit_line(&doc);
    
    // Generate JSON
    char *json;
    int result = json_stringify(&doc, &json);
    ASSERT_EQ(0, result);
    ASSERT_TRUE(json != NULL);
    
    // Verify table type in JSON
    ASSERT_TRUE(strstr(json, "\"type\":\"table\"") != NULL);
    
    // Verify table structure
    ASSERT_TRUE(strstr(json, "Product") != NULL);
    ASSERT_TRUE(strstr(json, "Price") != NULL);
    
    printf("JSON output: %s\n", json);
    
    doc_free(&doc);
    free(json);
    
    TEST_END;
}

void test_non_table_line_ignored(void) {
    TEST("Non-table separator lines ignored");
    
    Document doc;
    editor_init(&doc);
    
    // Input regular text
    const char *text = "This is just text";
    for (size_t i = 0; i < strlen(text); i++) {
        editor_feed_char(&doc, text[i]);
    }
    editor_commit_line(&doc);
    
    // Input separator (but no pipe in previous line)
    const char *separator = "| --- | --- |";
    for (size_t i = 0; i < strlen(separator); i++) {
        editor_feed_char(&doc, separator[i]);
    }
    editor_commit_line(&doc);
    
    // Should have two text elements, no table
    ASSERT_EQ(2, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    ASSERT_EQ(T_TEXT, doc.elements[1].kind);
    
    doc_free(&doc);
    
    TEST_END;
}

int main(void) {
    printf("Running Table Parsing Tests\n");
    printf("===========================\n\n");
    
    test_table_creation();
    test_table_json_output();
    test_non_table_line_ignored();
    
    printf("\nTest Results: %d/%d tests passed\n", passed_count, test_count);
    
    if (passed_count == test_count) {
        printf("All tests passed! ✓\n");
        return 0;
    } else {
        printf("Some tests failed. ✗\n");
        return 1;
    }
}
