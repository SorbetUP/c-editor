#include "editor.h"
#include "editor_abi.h"
#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

static void feed_ascii(Document *doc, const char *text) {
  for (const unsigned char *p = (const unsigned char *)text; *p; ++p) {
    editor_feed_char(doc, *p);
  }
}

static bool has_italic_span(const ElementText *text) {
  for (size_t i = 0; i < text->spans_count; i++) {
    if (text->spans[i].italic) {
      return true;
    }
  }
  return false;
}

static bool has_code_span(const ElementText *text) {
  for (size_t i = 0; i < text->spans_count; i++) {
    if (text->spans[i].code) {
      return true;
    }
  }
  return false;
}

static void assert_contains(const char *haystack, const char *needle) {
  assert(haystack != NULL);
  assert(needle != NULL);
  assert(strstr(haystack, needle) != NULL);
}

static void assert_exact(const char *actual, const char *expected) {
  assert(actual != NULL);
  assert(expected != NULL);
  assert(strcmp(actual, expected) == 0);
}

static void test_header_case_is_preserved(void) {
  Document doc;
  editor_init(&doc);

  feed_ascii(&doc, "# Mixed Case Header\n");

  assert(doc.elements_len == 1);
  assert(doc.elements[0].kind == T_TEXT);
  assert(doc.elements[0].as.text.level == 1);
  assert(strcmp(doc.elements[0].as.text.text, "Mixed Case Header") == 0);

  doc_free(&doc);
}

static void test_table_rows_keep_inline_styles(void) {
  Document doc;
  editor_init(&doc);

  feed_ascii(&doc, "| Name | Score |\n");
  feed_ascii(&doc, "| --- | --- |\n");
  feed_ascii(&doc, "| *Ada* | `42` |\n");

  assert(doc.elements_len == 1);
  assert(doc.elements[0].kind == T_TABLE);

  ElementTable *table = &doc.elements[0].as.table;
  assert(table->rows == 2);
  assert(table->cols == 2);
  assert(table->cells[1][0] != NULL);
  assert(table->cells[1][1] != NULL);
  assert(has_italic_span(table->cells[1][0]));
  assert(has_code_span(table->cells[1][1]));

  doc_free(&doc);
}

static void test_table_headers_keep_inline_styles(void) {
  Document doc;
  editor_init(&doc);

  feed_ascii(&doc, "| *Name* | Score |\n");
  feed_ascii(&doc, "| --- | --- |\n");

  assert(doc.elements_len == 1);
  assert(doc.elements[0].kind == T_TABLE);

  ElementTable *table = &doc.elements[0].as.table;
  assert(table->rows == 1);
  assert(table->cols == 2);
  assert(table->cells[0][0] != NULL);
  assert(has_italic_span(table->cells[0][0]));

  doc_free(&doc);
}

static void test_html_inline_rendering(void) {
  const char *html = editor_markdown_to_html("**gras** *italique* `code` "
                                             "~~barre~~ ==surligne== "
                                             "++souligne++ "
                                             "[texte](https://example.com)");
  assert_contains(html, "<strong>gras</strong>");
  assert_contains(html, "<em>italique</em>");
  assert_contains(html, "<code>code</code>");
  assert_contains(html, "<del>barre</del>");
  assert_contains(html, "<mark>surligne</mark>");
  assert_contains(html, "<u>souligne</u>");
  assert_contains(html, "<a href=\"https://example.com\">texte</a>");
}

static void test_exact_html_contracts(void) {
  assert_exact(editor_markdown_to_html("# Titre"), "<h1>Titre</h1>");
  assert_exact(editor_markdown_to_html("Texte **gras** et *italique* et `code`"),
               "Texte <strong>gras</strong> et <em>italique</em> et "
               "<code>code</code>");
  assert_exact(editor_markdown_to_html("<script> & test"),
               "&lt;script&gt; &amp; test");
  assert_exact(editor_markdown_to_html("- [x] done"),
               "<ul class=\"task-list\"><li><input type=\"checkbox\" checked "
               "disabled /> done</li></ul>");
  assert_exact(editor_markdown_to_html("| A | B |\n| --- | --- |\n| 1 | 2 |"),
               "<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody>"
               "<tr><td>1</td><td>2</td></tr></tbody></table>");
  assert_exact(editor_markdown_to_html("```c\nint x = 1;\n```"),
               "<pre><code class=\"language-c\">int x = 1;</code></pre>");
}

static void test_html_block_rendering(void) {
  const char *list_html = editor_markdown_to_html("- [x] done");
  assert_exact(list_html,
               "<ul class=\"task-list\"><li><input type=\"checkbox\" checked "
               "disabled /> done</li></ul>");

  const char *quote_html = editor_markdown_to_html("> citation");
  assert_exact(quote_html, "<blockquote>citation</blockquote>");

  const char *image_html =
      editor_markdown_to_html("![alt](https://example.com/a.png)");
  assert_exact(image_html,
               "<img src=\"https://example.com/a.png\" alt=\"alt\" />");

  const char *table_html = editor_markdown_to_html(
      "| A | B |\n| --- | --- |\n| 1 | 2 |");
  assert_exact(table_html,
               "<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody>"
               "<tr><td>1</td><td>2</td></tr></tbody></table>");

  const char *code_html =
      editor_markdown_to_html("```c\nint x = 1;\n```");
  assert_exact(code_html,
               "<pre><code class=\"language-c\">int x = 1;</code></pre>");
}

int main(void) {
  assert(editor_library_init() == EDITOR_SUCCESS);
  test_header_case_is_preserved();
  test_table_headers_keep_inline_styles();
  test_table_rows_keep_inline_styles();
  test_html_inline_rendering();
  test_exact_html_contracts();
  test_html_block_rendering();
  editor_library_cleanup();
  printf("editor tests passed\n");
  return 0;
}
