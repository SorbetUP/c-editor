#include "editor.h"
#include "json.h"
#include "markdown.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static void free_element_text(ElementText *text);

static void ensure_elements_capacity(Document *doc, size_t needed) {
  if (doc->elements_capacity < needed) {
    size_t new_capacity =
        doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
    while (new_capacity < needed)
      new_capacity *= 2;

    doc->elements = realloc(doc->elements, new_capacity * sizeof(Element));
    doc->elements_capacity = new_capacity;
  }
}

static void ensure_line_capacity(Document *doc, size_t needed) {
  if (doc->current_line_capacity < needed) {
    size_t new_capacity =
        doc->current_line_capacity == 0 ? 64 : doc->current_line_capacity * 2;
    while (new_capacity < needed)
      new_capacity *= 2;

    doc->current_line = realloc(doc->current_line, new_capacity);
    doc->current_line_capacity = new_capacity;
  }
}

static char *strdup_safe(const char *s) {
  if (!s)
    return NULL;
  size_t len = strlen(s);
  char *copy = malloc(len + 1);
  memcpy(copy, s, len + 1);
  return copy;
}

static void init_default_rgba(RGBA *rgba, float r, float g, float b, float a) {
  rgba->r = r;
  rgba->g = g;
  rgba->b = b;
  rgba->a = a;
}

void editor_init(Document *doc) {
  memset(doc, 0, sizeof(Document));

  doc->name = strdup_safe("new note");
  doc->default_font = strdup_safe("Helvetica");
  doc->default_fontsize = 11;

  init_default_rgba(&doc->default_text_color, 0.0f, 0.0f, 0.0f, 1.0f);
  init_default_rgba(&doc->default_highlight_color, 1.0f, 1.0f, 0.0f, 0.3f);
  init_default_rgba(&doc->default_underline_color, 0.0f, 0.0f, 0.0f, 0.4f);
  doc->default_underline_gap = 7;

  doc->created = time(NULL);
  doc->updated = doc->created;

  ensure_line_capacity(doc, 64);
  doc->current_line[0] = '\0';
}

static int count_header_level(const char *line) {
  int count = 0;
  while (line[count] == '#' && count < 6) {
    count++;
  }
  if (count > 0 && (line[count] == ' ' || line[count] == '\t')) {
    return count;
  }
  return 0;
}

static void to_uppercase(char *str) {
  while (*str) {
    *str = toupper(*str);
    str++;
  }
}

static ElementText create_text_element(Document *doc, const char *text,
                                       int level) {
  ElementText elem;
  memset(&elem, 0, sizeof(elem));

  elem.text = strdup_safe(text);
  elem.font = NULL;
  elem.align = ALIGN_LEFT;
  elem.font_size = level > 0 ? (28 - (level - 1) * 4) : doc->default_fontsize;
  elem.color = doc->default_text_color;
  elem.level = level;

  if (level > 0) {
    elem.bold = true;
    to_uppercase(elem.text);
  }

  return elem;
}

void editor_feed_char(Document *doc, unsigned codepoint) {
  if (codepoint == '\n') {
    editor_commit_line(doc);
    return;
  }

  if (codepoint > 127) {
    char utf8[5];
    int len = 0;
    if (codepoint <= 0x7F) {
      utf8[0] = codepoint;
      len = 1;
    } else if (codepoint <= 0x7FF) {
      utf8[0] = 0xC0 | (codepoint >> 6);
      utf8[1] = 0x80 | (codepoint & 0x3F);
      len = 2;
    } else if (codepoint <= 0xFFFF) {
      utf8[0] = 0xE0 | (codepoint >> 12);
      utf8[1] = 0x80 | ((codepoint >> 6) & 0x3F);
      utf8[2] = 0x80 | (codepoint & 0x3F);
      len = 3;
    } else {
      utf8[0] = 0xF0 | (codepoint >> 18);
      utf8[1] = 0x80 | ((codepoint >> 12) & 0x3F);
      utf8[2] = 0x80 | ((codepoint >> 6) & 0x3F);
      utf8[3] = 0x80 | (codepoint & 0x3F);
      len = 4;
    }
    utf8[len] = '\0';

    ensure_line_capacity(doc, doc->current_line_len + len + 1);
    strcat(doc->current_line + doc->current_line_len, utf8);
    doc->current_line_len += len;
  } else {
    ensure_line_capacity(doc, doc->current_line_len + 2);
    doc->current_line[doc->current_line_len] = (char)codepoint;
    doc->current_line[doc->current_line_len + 1] = '\0';
    doc->current_line_len++;
  }
}

static bool is_pipe_line(const char *line) {
  const char *p = line;
  while (*p && isspace(*p))
    p++;
  return *p == '|' || strchr(p, '|') != NULL;
}

void editor_commit_line(Document *doc) {
  if (!doc->current_line || doc->current_line_len == 0) {
    ensure_line_capacity(doc, 64);
    doc->current_line[0] = '\0';
    doc->current_line_len = 0;
    return;
  }

  // Check if this is a table separator line
  if (is_table_separator_line(doc->current_line)) {
    // Try to convert the previous text element to a table
    bool table_created = false;
    if (doc->elements_len > 0 &&
        doc->elements[doc->elements_len - 1].kind == T_TEXT) {
      ElementText *prev_text = &doc->elements[doc->elements_len - 1].as.text;

      // Check if previous line could be a table header (has spans indicating
      // pipe content) We need to reconstruct the original text from spans to
      // check for pipes
      bool has_pipes = false;
      if (prev_text->spans_count > 0) {
        // Reconstruct text from spans to check for pipes
        size_t total_len = 0;
        for (size_t s = 0; s < prev_text->spans_count; s++) {
          if (prev_text->spans[s].text) {
            total_len += strlen(prev_text->spans[s].text);
          }
        }

        char *reconstructed = malloc(total_len + 1);
        reconstructed[0] = '\0';
        for (size_t s = 0; s < prev_text->spans_count; s++) {
          if (prev_text->spans[s].text) {
            strcat(reconstructed, prev_text->spans[s].text);
          }
        }

        has_pipes = is_pipe_line(reconstructed);
        free(reconstructed);
      }

      if (has_pipes) {
        // Convert to table
        ElementTable table;
        memset(&table, 0, sizeof(table));

        table.grid_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
        table.background_color = (RGBA){1.0f, 1.0f, 1.0f, 1.0f};
        table.grid_size = 1;

        // Reconstruct text again for parsing (we need it)
        size_t total_len = 0;
        for (size_t s = 0; s < prev_text->spans_count; s++) {
          if (prev_text->spans[s].text) {
            total_len += strlen(prev_text->spans[s].text);
          }
        }

        char *reconstructed = malloc(total_len + 1);
        reconstructed[0] = '\0';
        for (size_t s = 0; s < prev_text->spans_count; s++) {
          if (prev_text->spans[s].text) {
            strcat(reconstructed, prev_text->spans[s].text);
          }
        }

        // Parse header row
        int col_count;
        char **header_cols = split_table_row(reconstructed, &col_count);
        free(reconstructed);

        table.cols = col_count;
        table.rows = 1;
        table.cells = malloc(sizeof(ElementText **));
        table.cells[0] = malloc(col_count * sizeof(ElementText *));

        for (int c = 0; c < col_count; c++) {
          table.cells[0][c] = malloc(sizeof(ElementText));
          memset(table.cells[0][c], 0, sizeof(ElementText));
          table.cells[0][c]->text =
              strdup_safe(header_cols[c] ? header_cols[c] : "");
          table.cells[0][c]->level = 0;
          table.cells[0][c]->align = ALIGN_LEFT;
          table.cells[0][c]->color = doc->default_text_color;
          table.cells[0][c]->bold = true; // Header row is bold
        }

        // Clean up header cols
        for (int c = 0; c < col_count; c++) {
          free(header_cols[c]);
        }
        free(header_cols);

        // Replace the previous text element with table
        free_element_text(prev_text);
        doc->elements[doc->elements_len - 1].kind = T_TABLE;
        doc->elements[doc->elements_len - 1].as.table = table;
        table_created = true;
      }
    }

    // If no table was created, process separator line as regular text
    if (!table_created) {
      // Fall through to normal text processing
    } else {
      doc->current_line[0] = '\0';
      doc->current_line_len = 0;
      return;
    }
  }

  ElementImage image;
  if (parse_image_line(doc->current_line, &image) == 0) {
    ensure_elements_capacity(doc, doc->elements_len + 1);
    doc->elements[doc->elements_len].kind = T_IMAGE;
    doc->elements[doc->elements_len].as.image = image;
    doc->elements_len++;
  } else {
    int level = count_header_level(doc->current_line);
    const char *text_start = doc->current_line;
    if (level > 0) {
      text_start += level;
      while (*text_start && (*text_start == ' ' || *text_start == '\t')) {
        text_start++;
      }
    }

    ElementText text_elem = create_text_element(doc, text_start, level);

    InlineSpan spans[32];
    int span_count = parse_inline_styles(text_elem.text, spans, 32);
    text_elem.spans = convert_spans_to_text_spans(
        text_elem.text, span_count > 0 ? spans : NULL,
        span_count > 0 ? span_count : 0, &text_elem.spans_count);
    free(text_elem.text);
    text_elem.text = NULL;

    // Reconstruct clean text from spans
    size_t total_len = 0;
    for (size_t i = 0; i < text_elem.spans_count; i++) {
      if (text_elem.spans[i].text) {
        total_len += strlen(text_elem.spans[i].text);
      }
    }

    text_elem.text = malloc(total_len + 1);
    text_elem.text[0] = '\0';
    for (size_t i = 0; i < text_elem.spans_count; i++) {
      if (text_elem.spans[i].text) {
        strcat(text_elem.text, text_elem.spans[i].text);
      }
    }

    // Set global style flags based on spans (preserve existing flags for
    // headers)
    bool preserve_bold = text_elem.bold; // Headers already have bold=true
    text_elem.bold = preserve_bold;
    text_elem.italic = false;
    text_elem.has_highlight = false;
    text_elem.has_underline = false;
    for (size_t i = 0; i < text_elem.spans_count; i++) {
      if (text_elem.spans[i].bold)
        text_elem.bold = true;
      if (text_elem.spans[i].italic)
        text_elem.italic = true;
      if (text_elem.spans[i].has_highlight) {
        text_elem.has_highlight = true;
        text_elem.highlight_color = text_elem.spans[i].highlight_color;
      }
      if (text_elem.spans[i].has_underline) {
        text_elem.has_underline = true;
        text_elem.underline_color = text_elem.spans[i].underline_color;
        text_elem.underline_gap = text_elem.spans[i].underline_gap;
      }
    }

    ensure_elements_capacity(doc, doc->elements_len + 1);
    doc->elements[doc->elements_len].kind = T_TEXT;
    doc->elements[doc->elements_len].as.text = text_elem;
    doc->elements_len++;
  }

  doc->current_line[0] = '\0';
  doc->current_line_len = 0;
  doc->updated = time(NULL);
}

static void free_element_text(ElementText *text) {
  if (text->spans) {
    for (size_t i = 0; i < text->spans_count; i++) {
      free(text->spans[i].text);
    }
    free(text->spans);
  }
  free(text->text);
  free(text->font);
}

static void free_element_image(ElementImage *image) {
  free(image->src);
  free(image->alt);
}

static void free_element_table(ElementTable *table) {
  for (size_t r = 0; r < table->rows; r++) {
    for (size_t c = 0; c < table->cols; c++) {
      if (table->cells[r][c]) {
        free_element_text(table->cells[r][c]);
        free(table->cells[r][c]);
      }
    }
    free(table->cells[r]);
  }
  free(table->cells);
}

void doc_free(Document *doc) {
  free(doc->name);
  free(doc->default_font);
  free(doc->current_line);

  for (size_t i = 0; i < doc->elements_len; i++) {
    switch (doc->elements[i].kind) {
    case T_TEXT:
      free_element_text(&doc->elements[i].as.text);
      break;
    case T_IMAGE:
      free_element_image(&doc->elements[i].as.image);
      break;
    case T_TABLE:
      free_element_table(&doc->elements[i].as.table);
      break;
    }
  }
  free(doc->elements);
  memset(doc, 0, sizeof(Document));
}

int json_export_markdown(const Document *doc, char **out_md) {
  return json_to_markdown(doc, out_md);
}

int json_import_markdown(const char *md, Document *out_doc) {
  return markdown_to_json(md, out_doc);
}
