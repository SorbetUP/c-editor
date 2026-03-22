#include "markdown.h"
#include "editor.h"
#include "json.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void test_list_markers(void) {
  const char *md =
      "- classique\n"
      "[ ] tache isolée\n"
      "[x] tache terminée\n"
      "1] variante numérotée\n"
      "(1. parenthèse numérotée\n";

  Document doc;
  assert(markdown_to_json(md, &doc) == 0);
  assert(doc.elements_len == 2);

  ElementList *tasks = &doc.elements[0].as.list;
  assert(tasks->kind == LIST_KIND_TASK);
  assert(tasks->item_count == 3);
  assert(strcmp(tasks->items[0].text.text, "classique") == 0);
  assert(!tasks->items[0].has_checkbox);
  assert(tasks->items[1].has_checkbox && !tasks->items[1].checkbox_checked);
  assert(tasks->items[2].has_checkbox && tasks->items[2].checkbox_checked);

  ElementList *ordered = &doc.elements[1].as.list;
  assert(ordered->kind == LIST_KIND_ORDERED);
  assert(ordered->item_count == 2);
  assert(ordered->items[0].number == 1);
  assert(ordered->items[1].number == 1);

  char *json = NULL;
  assert(json_stringify(&doc, &json) == 0);
  assert(json != NULL);

  char *md_back = NULL;
  assert(json_to_markdown(&doc, &md_back) == 0);
  assert(md_back != NULL);
  // Ensure the regenerated markdown still contains our markers
  assert(strstr(md_back, "[ ] tache isolée") != NULL);
  assert(strstr(md_back, "[x] tache terminée") != NULL);
  assert(strstr(md_back, "1] variante numérotée") != NULL ||
         strstr(md_back, "1) variante numérotée") != NULL ||
         strstr(md_back, "1. variante numérotée") != NULL);
  assert(strstr(md_back, "(1. parenthèse numérotée") != NULL ||
         strstr(md_back, "1. parenthèse numérotée") != NULL);

  free(json);
  free(md_back);
  doc_free(&doc);
}

static void test_markdown_table(void) {
  const char *md =
      "| Name | Score | Note |\n"
      "| :--- | ---: | --- |\n"
      "| *Ada* | `42` | ok |\n"
      "| Alan | `7` |  |\n";

  Document doc;
  assert(markdown_to_json(md, &doc) == 0);
  assert(doc.elements_len == 1);
  assert(doc.elements[0].kind == T_TABLE);

  ElementTable *table = &doc.elements[0].as.table;
  assert(table->rows == 3);
  assert(table->cols == 3);

  assert(table->column_align_defined != NULL);
  assert(table->column_align != NULL);
  assert(table->column_align_defined[0]);
  assert(table->column_align[0] == ALIGN_LEFT);
  assert(table->column_align_defined[1]);
  assert(table->column_align[1] == ALIGN_RIGHT);
  assert(!table->column_align_defined[2]);

  ElementText *header_name = table->cells[0][0];
  assert(header_name && header_name->text);
  assert(strcmp(header_name->text, "Name") == 0);

  ElementText *ada_cell = table->cells[1][0];
  assert(ada_cell && ada_cell->spans_count >= 1);
  bool found_italic = false;
  for (size_t i = 0; i < ada_cell->spans_count; i++) {
    if (ada_cell->spans[i].italic)
      found_italic = true;
  }
  assert(found_italic);

  ElementText *code_cell = table->cells[1][1];
  assert(code_cell && code_cell->spans_count >= 1);
  bool found_code = false;
  for (size_t i = 0; i < code_cell->spans_count; i++) {
    if (code_cell->spans[i].code)
      found_code = true;
  }
  assert(found_code);

  char *json = NULL;
  assert(json_stringify(&doc, &json) == 0);
  assert(json != NULL);
  assert(strstr(json, "\"type\":\"table\"") != NULL);
  assert(strstr(json, "\"align\":[\"left\",\"right\",null]") != NULL);
  assert(strstr(json, "\"italic\":true") != NULL);
  assert(strstr(json, "\"code\":true") != NULL);

  char *md_back = NULL;
  assert(json_to_markdown(&doc, &md_back) == 0);
  assert(md_back != NULL);
  assert(strstr(md_back, "| Name | Score | Note |") != NULL);
  assert(strstr(md_back, "*Ada*") != NULL);
  assert(strstr(md_back, "`42`") != NULL);

  free(json);
  free(md_back);
  doc_free(&doc);
}

int main(void) {
  test_list_markers();
  test_markdown_table();
  printf("✅ list marker tests passed\n");
  printf("✅ markdown table tests passed\n");
  return 0;
}
