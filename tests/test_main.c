#include "../src/editor.h"
#include "../src/markdown.h"
#include "../src/json.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <math.h>

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

#define ASSERT_FLOAT_EQ(expected, actual, tolerance) \
    do { \
        if (fabs((expected) - (actual)) > (tolerance)) { \
            printf("FAILED\n  Expected: %.3f\n  Actual: %.3f\n", (expected), (actual)); \
            return; \
        } \
    } while(0)

#define TEST_END \
        printf("PASSED\n"); \
        passed_count++; \
    } while(0)

void test_simple_paragraph(void) {
    TEST("Simple paragraph with inline styles");
    
    const char *markdown = "Bonjour *monde* en **C** et ***Markdown***.";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_STR_EQ("Bonjour monde en C et Markdown.", text->text);
    ASSERT_EQ(0, text->level);
    ASSERT_TRUE(text->bold);
    ASSERT_TRUE(text->italic);
    
    char *exported_md;
    result = json_to_markdown(&doc, &exported_md);
    ASSERT_EQ(0, result);
    
    doc_free(&doc);
    free(exported_md);
    
    TEST_END;
}

void test_headers(void) {
    TEST("Headers with level detection and uppercase");
    
    const char *markdown = "# Titre 1\n## Titre 2\nParagraphe";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(3, doc.elements_len);
    
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    ASSERT_EQ(1, doc.elements[0].as.text.level);
    ASSERT_STR_EQ("TITRE 1", doc.elements[0].as.text.text);
    ASSERT_TRUE(doc.elements[0].as.text.bold);
    ASSERT_EQ(28, doc.elements[0].as.text.font_size);
    
    ASSERT_EQ(T_TEXT, doc.elements[1].kind);
    ASSERT_EQ(2, doc.elements[1].as.text.level);
    ASSERT_STR_EQ("TITRE 2", doc.elements[1].as.text.text);
    ASSERT_TRUE(doc.elements[1].as.text.bold);
    ASSERT_EQ(24, doc.elements[1].as.text.font_size);
    
    ASSERT_EQ(T_TEXT, doc.elements[2].kind);
    ASSERT_EQ(0, doc.elements[2].as.text.level);
    ASSERT_STR_EQ("Paragraphe", doc.elements[2].as.text.text);
    ASSERT_TRUE(!doc.elements[2].as.text.bold);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_image_with_attributes(void) {
    TEST("Image with attributes parsing");
    
    const char *markdown = "![alt](https://x/img.png){w=160 h=120 a=0.9 align=right}";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_IMAGE, doc.elements[0].kind);
    
    ElementImage *image = &doc.elements[0].as.image;
    ASSERT_STR_EQ("alt", image->alt);
    ASSERT_STR_EQ("https://x/img.png", image->src);
    ASSERT_EQ(ALIGN_RIGHT, image->align);
    ASSERT_EQ(160, image->width);
    ASSERT_EQ(120, image->height);
    ASSERT_FLOAT_EQ(0.9f, image->alpha, 0.01f);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_table_parsing(void) {
    TEST("Table parsing with multiple rows");
    
    const char *markdown = "| A | B | C |\n|---|---|---|\n| 1 | 2 |   |";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TABLE, doc.elements[0].kind);
    
    ElementTable *table = &doc.elements[0].as.table;
    ASSERT_EQ(2, table->rows);
    ASSERT_EQ(3, table->cols);
    
    ASSERT_STR_EQ("A", table->cells[0][0]->text);
    ASSERT_STR_EQ("B", table->cells[0][1]->text);
    ASSERT_STR_EQ("C", table->cells[0][2]->text);
    
    ASSERT_STR_EQ("1", table->cells[1][0]->text);
    ASSERT_STR_EQ("2", table->cells[1][1]->text);
    ASSERT_STR_EQ("", table->cells[1][2]->text);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_highlight_and_underline(void) {
    TEST("Highlight and underline parsing");
    
    const char *markdown = "==note== et ++important++";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_STR_EQ("note et important", text->text);
    ASSERT_TRUE(text->has_highlight);
    ASSERT_TRUE(text->has_underline);
    
    ASSERT_FLOAT_EQ(1.0f, text->highlight_color.r, 0.01f);
    ASSERT_FLOAT_EQ(1.0f, text->highlight_color.g, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, text->highlight_color.b, 0.01f);
    ASSERT_FLOAT_EQ(0.3f, text->highlight_color.a, 0.01f);
    
    ASSERT_FLOAT_EQ(0.0f, text->underline_color.r, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, text->underline_color.g, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, text->underline_color.b, 0.01f);
    ASSERT_FLOAT_EQ(0.4f, text->underline_color.a, 0.01f);
    ASSERT_EQ(7, text->underline_gap);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_editor_char_input(void) {
    TEST("Character-by-character editor input");
    
    Document doc;
    editor_init(&doc);
    
    const char *input = "Hello *world*";
    for (size_t i = 0; i < strlen(input); i++) {
        editor_feed_char(&doc, input[i]);
    }
    editor_commit_line(&doc);
    
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_TEXT, doc.elements[0].kind);
    ASSERT_STR_EQ("Hello world", doc.elements[0].as.text.text);
    ASSERT_TRUE(doc.elements[0].as.text.italic);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_json_round_trip(void) {
    TEST("JSON round-trip conversion");
    
    const char *original_md = "# Title\nThis is **bold** and *italic* text.\n![image](url.png)";
    
    Document doc1;
    int result = markdown_to_json(original_md, &doc1);
    ASSERT_EQ(0, result);
    
    char *json;
    result = json_stringify(&doc1, &json);
    ASSERT_EQ(0, result);
    ASSERT_TRUE(json != NULL);
    
    Document doc2;
    result = json_parse(json, &doc2);
    ASSERT_EQ(0, result);
    
    char *exported_md;
    result = json_to_markdown(&doc2, &exported_md);
    ASSERT_EQ(0, result);
    
    doc_free(&doc1);
    doc_free(&doc2);
    free(json);
    free(exported_md);
    
    TEST_END;
}

void test_edge_cases(void) {
    TEST("Edge cases and malformed input");
    
    const char *unclosed_bold = "*bonjour";
    Document doc;
    int result = markdown_to_json(unclosed_bold, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_STR_EQ("*bonjour", doc.elements[0].as.text.text);
    ASSERT_TRUE(!doc.elements[0].as.text.italic);
    doc_free(&doc);
    
    const char *empty_image = "![](http://x)";
    result = markdown_to_json(empty_image, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    ASSERT_EQ(T_IMAGE, doc.elements[0].kind);
    ASSERT_STR_EQ("", doc.elements[0].as.image.alt);
    ASSERT_STR_EQ("http://x", doc.elements[0].as.image.src);
    ASSERT_EQ(ALIGN_LEFT, doc.elements[0].as.image.align);
    ASSERT_FLOAT_EQ(1.0f, doc.elements[0].as.image.alpha, 0.01f);
    doc_free(&doc);
    
    TEST_END;
}

void test_nested_styles(void) {
    TEST("Nested and complex inline styles");
    
    const char *markdown = "***bold and italic*** with ==**bold highlight**==";
    
    Document doc;
    int result = markdown_to_json(markdown, &doc);
    ASSERT_EQ(0, result);
    ASSERT_EQ(1, doc.elements_len);
    
    ElementText *text = &doc.elements[0].as.text;
    ASSERT_STR_EQ("bold and italic with bold highlight", text->text);
    ASSERT_TRUE(text->bold);
    ASSERT_TRUE(text->italic);
    ASSERT_TRUE(text->has_highlight);
    
    doc_free(&doc);
    
    TEST_END;
}

void test_rgba_colors(void) {
    TEST("RGBA color format validation");
    
    Document doc;
    editor_init(&doc);
    
    ASSERT_FLOAT_EQ(0.0f, doc.default_text_color.r, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, doc.default_text_color.g, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, doc.default_text_color.b, 0.01f);
    ASSERT_FLOAT_EQ(1.0f, doc.default_text_color.a, 0.01f);
    
    ASSERT_FLOAT_EQ(1.0f, doc.default_highlight_color.r, 0.01f);
    ASSERT_FLOAT_EQ(1.0f, doc.default_highlight_color.g, 0.01f);
    ASSERT_FLOAT_EQ(0.0f, doc.default_highlight_color.b, 0.01f);
    ASSERT_FLOAT_EQ(0.3f, doc.default_highlight_color.a, 0.01f);
    
    char *json;
    int result = json_stringify(&doc, &json);
    ASSERT_EQ(0, result);
    
    ASSERT_TRUE(strstr(json, "[0.000,0.000,0.000,1.000]") != NULL);
    ASSERT_TRUE(strstr(json, "[1.000,1.000,0.000,0.300]") != NULL);
    
    doc_free(&doc);
    free(json);
    
    TEST_END;
}

void test_inline_loop_regression(void) {
    TEST("Inline parsing infinite loop regression test");
    
    const char *test_cases[] = {
        "Hello World",
        "Text with *unclosed italic",  
        "Text with **unclosed bold",
        "Text with ==unclosed highlight",
        "Text with ++unclosed underline"
    };
    
    for (size_t i = 0; i < sizeof(test_cases)/sizeof(test_cases[0]); i++) {
        InlineSpan spans[32];
        int span_count = parse_inline_styles(test_cases[i], spans, 32);
        ASSERT_TRUE(span_count >= 0);
    }
    
    TEST_END;
}

int main(void) {
    printf("Running C Editor Core Tests\n");
    printf("==========================\n\n");
    
    test_simple_paragraph();
    test_headers();
    test_image_with_attributes();
    test_table_parsing();
    test_highlight_and_underline();
    test_editor_char_input();
    test_json_round_trip();
    test_edge_cases();
    test_nested_styles();
    test_rgba_colors();
    test_inline_loop_regression();
    
    printf("\nTest Results: %d/%d tests passed\n", passed_count, test_count);
    
    if (passed_count == test_count) {
        printf("All tests passed! ✓\n");
        return 0;
    } else {
        printf("Some tests failed. ✗\n");
        return 1;
    }
}
