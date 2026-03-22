#include "editor.h"
#include "json.h"
#include "markdown.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static void free_element_text(ElementText *text);
static void free_element_image(ElementImage *image);
static void free_element_list(ElementList *list);
static void free_element_quote(ElementQuote *quote);
static void free_element_divider(ElementDivider *divider);

typedef struct {
  char *data;
  size_t len;
  size_t capacity;
} StringBuilder;

static bool ensure_elements_capacity(Document *doc, size_t needed) {
  if (doc->elements_capacity < needed) {
    size_t new_capacity =
        doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
    while (new_capacity < needed)
      new_capacity *= 2;

    Element *new_elements =
        realloc(doc->elements, new_capacity * sizeof(Element));
    if (!new_elements) {
      return false;
    }
    doc->elements = new_elements;
    doc->elements_capacity = new_capacity;
  }
  return true;
}

static bool ensure_line_capacity(Document *doc, size_t needed) {
  if (doc->current_line_capacity < needed) {
    size_t new_capacity =
        doc->current_line_capacity == 0 ? 64 : doc->current_line_capacity * 2;
    while (new_capacity < needed)
      new_capacity *= 2;

    char *new_line = realloc(doc->current_line, new_capacity);
    if (!new_line) {
      return false;
    }
    doc->current_line = new_line;
    doc->current_line_capacity = new_capacity;
  }
  return true;
}

static char *strdup_safe(const char *s) {
  if (!s)
    return NULL;
  size_t len = strlen(s);
  char *copy = malloc(len + 1);
  memcpy(copy, s, len + 1);
  return copy;
}

static bool sb_reserve(StringBuilder *builder, size_t extra) {
  size_t needed = builder->len + extra + 1;
  if (needed <= builder->capacity) {
    return true;
  }

  size_t new_capacity = builder->capacity == 0 ? 64 : builder->capacity;
  while (new_capacity < needed) {
    new_capacity *= 2;
  }

  char *new_data = realloc(builder->data, new_capacity);
  if (!new_data) {
    return false;
  }

  builder->data = new_data;
  builder->capacity = new_capacity;
  return true;
}

static bool sb_append_n(StringBuilder *builder, const char *text, size_t len) {
  if (!text || len == 0) {
    return true;
  }
  if (!sb_reserve(builder, len)) {
    return false;
  }

  memcpy(builder->data + builder->len, text, len);
  builder->len += len;
  builder->data[builder->len] = '\0';
  return true;
}

static bool sb_append(StringBuilder *builder, const char *text) {
  return sb_append_n(builder, text, text ? strlen(text) : 0);
}

static bool sb_append_char(StringBuilder *builder, char ch) {
  return sb_append_n(builder, &ch, 1);
}

static bool sb_append_inline_text(StringBuilder *builder, const TextSpan *span) {
  const char *content = (span && span->text) ? span->text : "";

  if (span && span->code) {
    return sb_append(builder, "`") && sb_append(builder, content) &&
           sb_append(builder, "`");
  }

  if (span && span->strikethrough && !sb_append(builder, "~~")) {
    return false;
  }
  if (span && span->has_highlight && !sb_append(builder, "==")) {
    return false;
  }
  if (span && span->has_underline && !sb_append(builder, "++")) {
    return false;
  }

  if (span && span->bold && span->italic) {
    if (!sb_append(builder, "***") || !sb_append(builder, content) ||
        !sb_append(builder, "***")) {
      return false;
    }
  } else if (span && span->bold) {
    if (!sb_append(builder, "**") || !sb_append(builder, content) ||
        !sb_append(builder, "**")) {
      return false;
    }
  } else if (span && span->italic) {
    if (!sb_append(builder, "*") || !sb_append(builder, content) ||
        !sb_append(builder, "*")) {
      return false;
    }
  } else if (!sb_append(builder, content)) {
    return false;
  }

  if (span && span->has_underline && !sb_append(builder, "++")) {
    return false;
  }
  if (span && span->has_highlight && !sb_append(builder, "==")) {
    return false;
  }
  if (span && span->strikethrough && !sb_append(builder, "~~")) {
    return false;
  }

  return true;
}

static bool sb_append_span_markdown(StringBuilder *builder, const TextSpan *span) {
  if (!span) {
    return true;
  }

  if (span->is_image && span->image_src) {
    const char *alt = span->image_alt ? span->image_alt : "";
    return sb_append(builder, "![") && sb_append(builder, alt) &&
           sb_append(builder, "](") && sb_append(builder, span->image_src) &&
           sb_append_char(builder, ')');
  }

  if (span->is_link && span->link_href) {
    TextSpan inner = *span;
    inner.is_link = false;
    inner.link_href = NULL;
    inner.is_image = false;
    inner.image_src = NULL;
    inner.image_alt = NULL;

    return sb_append_char(builder, '[') &&
           sb_append_inline_text(builder, &inner) &&
           sb_append(builder, "](") && sb_append(builder, span->link_href) &&
           sb_append_char(builder, ')');
  }

  return sb_append_inline_text(builder, span);
}

static char *render_text_as_markdown(const ElementText *text) {
  if (!text) {
    return strdup_safe("");
  }

  StringBuilder builder = {0};
  if (!sb_reserve(&builder, 32)) {
    return NULL;
  }
  builder.data[0] = '\0';

  if (text->spans && text->spans_count > 0) {
    for (size_t i = 0; i < text->spans_count; i++) {
      if (!sb_append_span_markdown(&builder, &text->spans[i])) {
        free(builder.data);
        return NULL;
      }
    }
    return builder.data;
  }

  TextSpan span;
  memset(&span, 0, sizeof(span));
  span.text = text->text;
  span.bold = text->bold;
  span.italic = text->italic;
  span.has_highlight = text->has_highlight;
  span.highlight_color = text->highlight_color;
  span.has_underline = text->has_underline;
  span.underline_color = text->underline_color;
  span.underline_gap = text->underline_gap;

  if (!sb_append_inline_text(&builder, &span)) {
    free(builder.data);
    return NULL;
  }

  return builder.data;
}

static void append_bytes(Document *doc, const char *bytes, size_t len) {
  if (!ensure_line_capacity(doc, doc->current_line_len + len + 1)) {
    return;
  }
  memcpy(doc->current_line + doc->current_line_len, bytes, len);
  doc->current_line_len += len;
  doc->current_line[doc->current_line_len] = '\0';
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

  if (ensure_line_capacity(doc, 64)) {
    doc->current_line[0] = '\0';
  }
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
  }

  return elem;
}

static void init_table_cell(ElementText *cell, const Document *doc,
                            const char *content, bool bold) {
  memset(cell, 0, sizeof(*cell));
  cell->text = strdup_safe(content ? content : "");
  cell->align = ALIGN_LEFT;
  cell->font_size = doc->default_fontsize;
  cell->color = doc->default_text_color;
  cell->level = 0;
  cell->bold = bold;
  markdown_populate_text_spans(cell);
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

    append_bytes(doc, utf8, (size_t)len);
  } else {
    char ch = (char)codepoint;
    append_bytes(doc, &ch, 1);
  }
}

static bool is_pipe_line(const char *line) {
  const char *p = line;
  while (*p && isspace((unsigned char)*p))
    p++;
  return *p == '|' || strchr(p, '|') != NULL;
}

void editor_commit_line(Document *doc) {
  if (!doc->current_line || doc->current_line_len == 0) {
    if (ensure_line_capacity(doc, 64)) {
      doc->current_line[0] = '\0';
    }
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
      // Check for pipes in original text directly
      bool has_pipes = prev_text->text ? is_pipe_line(prev_text->text) : false;

      if (has_pipes) {
        // Convert to table
        ElementTable table;
        memset(&table, 0, sizeof(table));

        table.grid_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
        table.background_color = (RGBA){1.0f, 1.0f, 1.0f, 1.0f};
        table.grid_size = 1;

        char *rendered_text = render_text_as_markdown(prev_text);
        const char *original_text =
            rendered_text ? rendered_text
                          : (prev_text->text ? prev_text->text : "");

        // Parse header row
        int col_count;
        char **header_cols = split_table_row(original_text, &col_count);

        table.cols = col_count;
        table.rows = 1;
        table.cells = malloc(sizeof(ElementText **));
        table.cells[0] = malloc(col_count * sizeof(ElementText *));

        for (int c = 0; c < col_count; c++) {
          table.cells[0][c] = malloc(sizeof(ElementText));
          init_table_cell(table.cells[0][c], doc,
                          header_cols[c] ? header_cols[c] : "", true);
        }

        // Clean up header cols
        for (int c = 0; c < col_count; c++) {
          free(header_cols[c]);
        }
        free(header_cols);
        free(rendered_text);

        // Calculate column widths for consistent rendering
        table_calculate_column_widths(&table);
        
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

  // Check if this line should be added to an existing table
  if (doc->elements_len > 0 && 
      doc->elements[doc->elements_len - 1].kind == T_TABLE &&
      is_pipe_line(doc->current_line) &&
      !is_table_separator_line(doc->current_line)) {
    
    ElementTable *table = &doc->elements[doc->elements_len - 1].as.table;
    
    // Parse the new row
    int col_count;
    char **row_cells = split_table_row(doc->current_line, &col_count);
    
    if (row_cells && col_count > 0) {
      // Expand table to add new row
      table->rows++;
      ElementText ***new_cells =
          realloc(table->cells, table->rows * sizeof(ElementText **));
      if (!new_cells) {
        for (int c = 0; c < col_count; c++) {
          free(row_cells[c]);
        }
        free(row_cells);
        return;
      }
      table->cells = new_cells;
      table->cells[table->rows - 1] = malloc(table->cols * sizeof(ElementText *));
      
      // Fill the new row with data
      for (size_t c = 0; c < table->cols; c++) {
        table->cells[table->rows - 1][c] = malloc(sizeof(ElementText));
        const char *cell_content = (c < (size_t)col_count && row_cells[c]) ? row_cells[c] : "";
        init_table_cell(table->cells[table->rows - 1][c], doc, cell_content,
                        false);
      }
      
      // Clean up row parsing
      for (int c = 0; c < col_count; c++) {
        free(row_cells[c]);
      }
      free(row_cells);
      
      // Recalculate column widths with new data
      table_calculate_column_widths(table);
      
      // Line processed as table row, clear and return
      doc->current_line[0] = '\0';
      doc->current_line_len = 0;
      doc->updated = time(NULL);
      return;
    }
  }

  ElementImage image;
  if (parse_image_line(doc->current_line, &image) == 0) {
    if (!ensure_elements_capacity(doc, doc->elements_len + 1)) {
      free_element_image(&image);
      return;
    }
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
    markdown_populate_text_spans(&text_elem);

    if (!ensure_elements_capacity(doc, doc->elements_len + 1)) {
      free_element_text(&text_elem);
      return;
    }
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
      free(text->spans[i].link_href);
      free(text->spans[i].image_src);
      free(text->spans[i].image_alt);
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
  free(table->column_align);
  free(table->column_align_defined);
  free(table->column_widths);
  free(table->column_min_widths);
  free(table->column_max_widths);
}

static void free_element_code(ElementCode *code) {
  free(code->language);
  free(code->content);
}

static void free_element_list(ElementList *list) {
  if (!list || !list->items)
    return;
  for (size_t i = 0; i < list->item_count; i++) {
    free_element_text(&list->items[i].text);
    if (list->items[i].is_definition) {
      free_element_text(&list->items[i].term);
      free_element_text(&list->items[i].definition);
    }
  }
  free(list->items);
  list->items = NULL;
  list->item_count = 0;
  list->item_capacity = 0;
}

static void free_element_quote(ElementQuote *quote) {
  if (!quote || !quote->items)
    return;
  for (size_t i = 0; i < quote->item_count; i++) {
    free_element_text(&quote->items[i]);
  }
  free(quote->items);
  quote->items = NULL;
  quote->item_count = 0;
  quote->item_capacity = 0;
}

static void free_element_divider(ElementDivider *divider) {
  (void)divider;
}

static void free_element_settings(ElementSettings *settings) {
  if (!settings)
    return;
  free(settings->name);
  free(settings->value);
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
    case T_CODE:
      free_element_code(&doc->elements[i].as.code);
      break;
    case T_LIST:
      free_element_list(&doc->elements[i].as.list);
      break;
    case T_QUOTE:
      free_element_quote(&doc->elements[i].as.quote);
      break;
    case T_DIVIDER:
      free_element_divider(&doc->elements[i].as.divider);
      break;
    case T_SETTINGS:
      free_element_settings(&doc->elements[i].as.settings);
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

// Table column width calculation functions
int table_get_cell_content_width(const ElementText *cell) {
  if (!cell || !cell->text) {
    return 0;
  }
  
  // Calculate width based on text length and formatting
  int width = 0;
  
  if (cell->spans && cell->spans_count > 0) {
    // Use spans for accurate width calculation
    for (size_t i = 0; i < cell->spans_count; i++) {
      if (cell->spans[i].text) {
        int span_width = strlen(cell->spans[i].text);
        // Add extra width for formatting (bold, italic, code)
        if (cell->spans[i].bold || cell->spans[i].italic) {
          span_width += 2; // **text** or *text* markers
        }
        if (cell->spans[i].code) {
          span_width += 2; // `text` markers
        }
        width += span_width;
      }
    }
  } else {
    // Fallback to simple text length
    width = strlen(cell->text);
  }
  
  // Minimum width of 3 characters for any cell
  return width < 3 ? 3 : width;
}

int table_calculate_column_widths(ElementTable *table) {
  if (!table || table->cols == 0 || table->rows == 0) {
    return -1;
  }
  
  // Allocate width arrays if not already done
  if (!table->column_widths) {
    table->column_widths = calloc(table->cols, sizeof(int));
    table->column_min_widths = calloc(table->cols, sizeof(int));
    table->column_max_widths = calloc(table->cols, sizeof(int));
    
    if (!table->column_widths || !table->column_min_widths || !table->column_max_widths) {
      return -1;
    }
  }
  
  // Initialize with minimum widths
  for (size_t c = 0; c < table->cols; c++) {
    table->column_min_widths[c] = 3;  // Minimum 3 characters
    table->column_max_widths[c] = 50; // Maximum 50 characters
    table->column_widths[c] = table->column_min_widths[c];
  }
  
  // Calculate optimal width for each column
  for (size_t r = 0; r < table->rows; r++) {
    for (size_t c = 0; c < table->cols; c++) {
      if (table->cells[r][c]) {
        int cell_width = table_get_cell_content_width(table->cells[r][c]);
        
        // Update column width to fit this cell
        if (cell_width > table->column_widths[c]) {
          table->column_widths[c] = cell_width < table->column_max_widths[c] 
                                     ? cell_width 
                                     : table->column_max_widths[c];
        }
      }
    }
  }
  
  // Calculate total content width
  table->total_content_width = 0;
  for (size_t c = 0; c < table->cols; c++) {
    table->total_content_width += table->column_widths[c];
  }
  
  // Add separator space: | col1 | col2 | col3 |
  // That's: cols + 1 separators + 2*cols padding
  table->total_content_width += (table->cols + 1) + (2 * table->cols);
  
  table->widths_calculated = true;
  return 0;
}

void table_apply_width_constraints(ElementTable *table, int max_total_width) {
  if (!table || !table->widths_calculated || max_total_width <= 0) {
    return;
  }
  
  // If total width fits, no adjustment needed
  if (table->total_content_width <= max_total_width) {
    return;
  }
  
  // Calculate available width for content (excluding separators and padding)
  int separator_overhead = (table->cols + 1) + (2 * table->cols);
  int available_content_width = max_total_width - separator_overhead;
  
  if (available_content_width < (int)(table->cols * 3)) {
    // Not enough space even for minimum widths
    for (size_t c = 0; c < table->cols; c++) {
      table->column_widths[c] = 3;
    }
    return;
  }
  
  // Proportionally reduce column widths
  int current_total = 0;
  for (size_t c = 0; c < table->cols; c++) {
    current_total += table->column_widths[c];
  }
  
  for (size_t c = 0; c < table->cols; c++) {
    double ratio = (double)table->column_widths[c] / current_total;
    int new_width = (int)(ratio * available_content_width);
    
    // Ensure minimum width
    table->column_widths[c] = new_width < table->column_min_widths[c] 
                               ? table->column_min_widths[c] 
                               : new_width;
  }
  
  // Recalculate total width
  table->total_content_width = separator_overhead;
  for (size_t c = 0; c < table->cols; c++) {
    table->total_content_width += table->column_widths[c];
  }
}
