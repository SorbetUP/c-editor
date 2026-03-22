#include "editor.h"
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

int main(void) {
  test_header_case_is_preserved();
  test_table_headers_keep_inline_styles();
  test_table_rows_keep_inline_styles();
  printf("editor tests passed\n");
  return 0;
}
