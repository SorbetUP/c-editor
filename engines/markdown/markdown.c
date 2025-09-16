#include "markdown.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *strdup_safe(const char *s) {
  if (!s)
    return NULL;
  size_t len = strlen(s);
  char *copy = malloc(len + 1);
  memcpy(copy, s, len + 1);
  return copy;
}

static void trim_whitespace(char *str) {
  char *end = str + strlen(str) - 1;
  while (end > str && isspace(*end)) {
    *end = '\0';
    end--;
  }

  char *start = str;
  while (*start && isspace(*start)) {
    start++;
  }

  if (start != str) {
    memmove(str, start, strlen(start) + 1);
  }
}

// Removed uppercase conversion for idempotence

int parse_inline_styles(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0)
    return 0;

  size_t len = strlen(text);
  if (len == 0)
    return 0;

  size_t span_count = 0;
  size_t i = 0;

  while (i < len && span_count < max_spans) {
    bool matched = false;

    // Parse *** (bold+italic) with higher priority
    if (i + 2 < len && text[i] == '*' && text[i + 1] == '*' &&
        text[i + 2] == '*') {
      size_t start = i;
      for (size_t j = i + 3; j + 2 < len; j++) {
        if (text[j] == '*' && text[j + 1] == '*' && text[j + 2] == '*') {
          spans[span_count].style = INLINE_BOLD_ITALIC;
          spans[span_count].start = start;
          spans[span_count].end = j + 3;
          span_count++;
          i = j + 3;
          matched = true;
          break;
        }
      }
    } 
    // Parse ** (bold)
    else if (i + 1 < len && text[i] == '*' && text[i + 1] == '*') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '*' && text[j + 1] == '*') {
          spans[span_count].style = INLINE_BOLD;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    } 
    // Parse * (italic)
    else if (text[i] == '*') {
      size_t start = i;
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == '*') {
          spans[span_count].style = INLINE_ITALIC;
          spans[span_count].start = start;
          spans[span_count].end = j + 1;
          span_count++;
          i = j + 1;
          matched = true;
          break;
        }
      }
    } 
    // Parse == (highlight)
    else if (i + 1 < len && text[i] == '=' && text[i + 1] == '=') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '=' && text[j + 1] == '=') {
          spans[span_count].style = INLINE_HIGHLIGHT;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    } 
    // Parse ++ (underline)
    else if (i + 1 < len && text[i] == '+' && text[i + 1] == '+') {
      size_t start = i;
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '+' && text[j + 1] == '+') {
          spans[span_count].style = INLINE_UNDERLINE;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 2;
          matched = true;
          break;
        }
      }
    }

    if (!matched) {
      i++;
    }
  }

  return (int)span_count;
}

static char *strip_all_markers(const char *text) {
  size_t len = strlen(text);
  char *result = malloc(len + 1);
  size_t write_pos = 0;

  for (size_t i = 0; i < len;) {
    bool skipped = false;

    // Skip *** markers (bold+italic)
    if (i + 2 < len && text[i] == '*' && text[i + 1] == '*' &&
        text[i + 2] == '*') {
      i += 3;
      skipped = true;
    }
    // Skip ** markers (bold)
    else if (i + 1 < len && text[i] == '*' && text[i + 1] == '*') {
      i += 2;
      skipped = true;
    }
    // Skip * markers (italic)
    else if (text[i] == '*') {
      i++;
      skipped = true;
    }
    // Skip == markers
    else if (i + 1 < len && text[i] == '=' && text[i + 1] == '=') {
      i += 2;
      skipped = true;
    }
    // Skip ++ markers
    else if (i + 1 < len && text[i] == '+' && text[i + 1] == '+') {
      i += 2;
      skipped = true;
    }

    if (!skipped) {
      result[write_pos++] = text[i];
      i++;
    }
  }

  result[write_pos] = '\0';
  return result;
}

TextSpan *convert_spans_to_text_spans(const char *text, const InlineSpan *spans,
                                      size_t span_count, size_t *out_count) {
  if (span_count == 0) {
    TextSpan *result = malloc(sizeof(TextSpan));
    result[0].text = strip_all_markers(text); // Clean up unmatched markers
    result[0].bold = false;
    result[0].italic = false;
    result[0].has_highlight = false;
    result[0].has_underline = false;
    *out_count = 1;
    return result;
  }

  TextSpan *result = malloc((span_count * 2 + 1) * sizeof(TextSpan));
  size_t result_count = 0;
  size_t text_pos = 0;

  for (size_t i = 0; i < span_count; i++) {
    if (text_pos < spans[i].start) {
      size_t len = spans[i].start - text_pos;
      char *temp_text = malloc(len + 1);
      memcpy(temp_text, text + text_pos, len);
      temp_text[len] = '\0';

      char *plain_text = strip_all_markers(temp_text);
      free(temp_text);

      result[result_count].text = plain_text;
      result[result_count].bold = false;
      result[result_count].italic = false;
      result[result_count].has_highlight = false;
      result[result_count].has_underline = false;
      result_count++;
    }

    size_t content_start = spans[i].start;
    size_t content_end = spans[i].end;

    switch (spans[i].style) {
    case INLINE_BOLD:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_ITALIC:
      content_start += 1;
      content_end -= 1;
      break;
    case INLINE_BOLD_ITALIC:
      content_start += 3;
      content_end -= 3;
      break;
    case INLINE_HIGHLIGHT:
      content_start += 2;
      content_end -= 2;
      break;
    case INLINE_UNDERLINE:
      content_start += 2;
      content_end -= 2;
      break;
    default:
      break;
    }

    size_t styled_len = content_end - content_start;
    char *temp_styled = malloc(styled_len + 1);
    memcpy(temp_styled, text + content_start, styled_len);
    temp_styled[styled_len] = '\0';

    char *styled_text = strip_all_markers(temp_styled);
    free(temp_styled);

    result[result_count].text = styled_text;
    result[result_count].bold =
        (spans[i].style == INLINE_BOLD || spans[i].style == INLINE_BOLD_ITALIC);
    result[result_count].italic = (spans[i].style == INLINE_ITALIC ||
                                   spans[i].style == INLINE_BOLD_ITALIC);
    result[result_count].has_highlight = (spans[i].style == INLINE_HIGHLIGHT);
    if (result[result_count].has_highlight) {
      result[result_count].highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
    }
    result[result_count].has_underline = (spans[i].style == INLINE_UNDERLINE);
    if (result[result_count].has_underline) {
      result[result_count].underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
      result[result_count].underline_gap = 7;
    }
    result_count++;

    text_pos = spans[i].end;
  }

  if (text_pos < strlen(text)) {
    size_t len = strlen(text) - text_pos;
    char *temp_remaining = malloc(len + 1);
    memcpy(temp_remaining, text + text_pos, len);
    temp_remaining[len] = '\0';

    char *remaining_text = strip_all_markers(temp_remaining);
    free(temp_remaining);

    result[result_count].text = remaining_text;
    result[result_count].bold = false;
    result[result_count].italic = false;
    result[result_count].has_highlight = false;
    result[result_count].has_underline = false;
    result_count++;
  }

  *out_count = result_count;
  return result;
}

int parse_image_line(const char *line, ElementImage *image) {
  memset(image, 0, sizeof(*image));
  image->alpha = 1.0f;
  image->align = ALIGN_LEFT;

  const char *p = line;
  while (*p && isspace(*p))
    p++;

  if (p[0] != '!' || p[1] != '[') {
    return -1;
  }
  p += 2;

  const char *alt_start = p;
  while (*p && *p != ']')
    p++;
  if (*p != ']')
    return -1;

  size_t alt_len = p - alt_start;
  image->alt = malloc(alt_len + 1);
  memcpy(image->alt, alt_start, alt_len);
  image->alt[alt_len] = '\0';

  p++;
  if (*p != '(') {
    free(image->alt);
    image->alt = NULL;
    return -1;
  }
  p++;

  const char *src_start = p;
  while (*p && *p != ')')
    p++;
  if (*p != ')') {
    free(image->alt);
    image->alt = NULL;
    return -1;
  }

  size_t src_len = p - src_start;
  image->src = malloc(src_len + 1);
  memcpy(image->src, src_start, src_len);
  image->src[src_len] = '\0';

  p++;

  if (*p == '{') {
    p++;
    while (*p && *p != '}') {
      while (*p && isspace(*p))
        p++;

      if (strncmp(p, "w=", 2) == 0) {
        p += 2;
        image->width = atoi(p);
        while (*p && isdigit(*p))
          p++;
      } else if (strncmp(p, "h=", 2) == 0) {
        p += 2;
        image->height = atoi(p);
        while (*p && isdigit(*p))
          p++;
      } else if (strncmp(p, "a=", 2) == 0) {
        p += 2;
        image->alpha = atof(p);
        while (*p && (isdigit(*p) || *p == '.'))
          p++;
      } else if (strncmp(p, "align=", 6) == 0) {
        p += 6;
        if (strncmp(p, "left", 4) == 0) {
          image->align = ALIGN_LEFT;
          p += 4;
        } else if (strncmp(p, "center", 6) == 0) {
          image->align = ALIGN_CENTER;
          p += 6;
        } else if (strncmp(p, "right", 5) == 0) {
          image->align = ALIGN_RIGHT;
          p += 5;
        }
      } else {
        while (*p && !isspace(*p) && *p != '}')
          p++;
      }

      while (*p && isspace(*p))
        p++;
    }
  }

  return 0;
}

int parse_header_line(const char *line, ElementText *text) {
  memset(text, 0, sizeof(*text));

  int level = 0;
  const char *p = line;

  while (*p == '#' && level < 6) {
    level++;
    p++;
  }

  if (level == 0 || (*p != ' ' && *p != '\t')) {
    return -1;
  }

  while (*p && (*p == ' ' || *p == '\t'))
    p++;

  text->text = strdup_safe(p);
  text->level = level;
  text->bold = true;
  text->align = ALIGN_LEFT;
  text->font_size = 28 - (level - 1) * 4;
  text->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};

  return 0;
}

bool is_table_separator_line(const char *line) {
  const char *p = line;
  while (*p && isspace(*p))
    p++;

  if (*p == '|')
    p++;

  while (*p) {
    while (*p && isspace(*p))
      p++;

    if (*p == ':')
      p++;

    bool has_dash = false;
    while (*p && *p == '-') {
      has_dash = true;
      p++;
    }

    if (!has_dash)
      return false;

    if (*p == ':')
      p++;

    while (*p && isspace(*p))
      p++;

    if (*p == '|') {
      p++;
    } else if (*p == '\0') {
      break;
    } else {
      return false;
    }
  }

  return true;
}

static int count_table_columns(const char *line) {
  int count = 0;
  const char *p = line;

  while (*p && isspace(*p))
    p++;
  if (*p == '|')
    p++;

  while (*p) {
    while (*p && *p != '|' && *p != '\0')
      p++;
    count++;

    if (*p == '|') {
      p++;
    } else {
      break;
    }
  }

  return count;
}

char **split_table_row(const char *line, int *col_count) {
  int max_cols = count_table_columns(line);
  char **cells = malloc(max_cols * sizeof(char *));

  const char *p = line;
  while (*p && isspace(*p))
    p++;
  if (*p == '|')
    p++;

  int col = 0;
  while (*p && col < max_cols) {
    const char *start = p;
    while (*p && *p != '|')
      p++;

    size_t len = p - start;
    cells[col] = malloc(len + 1);
    memcpy(cells[col], start, len);
    cells[col][len] = '\0';
    trim_whitespace(cells[col]);

    col++;
    if (*p == '|')
      p++;
  }

  *col_count = col;
  return cells;
}

int parse_table_block(MarkdownParser *parser, ElementTable *table) {
  memset(table, 0, sizeof(*table));

  table->grid_color = (RGBA){0.0f, 0.0f, 0.0f, 0.0f};
  table->background_color = (RGBA){1.0f, 1.0f, 1.0f, 1.0f};
  table->grid_size = 1;

  size_t start_pos = parser->pos;
  const char *line_start = parser->text + parser->pos;
  const char *line_end = strchr(line_start, '\n');
  if (!line_end)
    line_end = parser->text + parser->len;

  char *first_line = malloc(line_end - line_start + 1);
  memcpy(first_line, line_start, line_end - line_start);
  first_line[line_end - line_start] = '\0';

  int first_row_cols;
  char **first_row = split_table_row(first_line, &first_row_cols);

  parser->pos = line_end - parser->text;
  if (*line_end == '\n')
    parser->pos++;

  if (parser->pos >= parser->len) {
    for (int i = 0; i < first_row_cols; i++) {
      free(first_row[i]);
    }
    free(first_row);
    free(first_line);
    return -1;
  }

  line_start = parser->text + parser->pos;
  line_end = strchr(line_start, '\n');
  if (!line_end)
    line_end = parser->text + parser->len;

  char *sep_line = malloc(line_end - line_start + 1);
  memcpy(sep_line, line_start, line_end - line_start);
  sep_line[line_end - line_start] = '\0';

  if (!is_table_separator_line(sep_line)) {
    for (int i = 0; i < first_row_cols; i++) {
      free(first_row[i]);
    }
    free(first_row);
    free(first_line);
    free(sep_line);
    parser->pos = start_pos;
    return -1;
  }

  parser->pos = line_end - parser->text;
  if (*line_end == '\n')
    parser->pos++;

  table->cols = first_row_cols;
  table->rows = 1;

  char ***all_rows = malloc(sizeof(char **));
  all_rows[0] = first_row;

  // Track column counts per row
  int *row_col_counts = malloc(sizeof(int));
  row_col_counts[0] = first_row_cols;

  while (parser->pos < parser->len) {
    line_start = parser->text + parser->pos;
    line_end = strchr(line_start, '\n');
    if (!line_end)
      line_end = parser->text + parser->len;

    char *line = malloc(line_end - line_start + 1);
    memcpy(line, line_start, line_end - line_start);
    line[line_end - line_start] = '\0';

    // Check if this line looks like a table row (must contain '|' or start with
    // '|')
    bool is_table_line = false;
    const char *p = line;
    while (*p && isspace(*p))
      p++; // skip whitespace
    if (*p == '|') {
      is_table_line = true;
    } else {
      // Check if line contains any '|' characters
      while (*p && *p != '\n') {
        if (*p == '|') {
          is_table_line = true;
          break;
        }
        p++;
      }
    }

    if (!is_table_line || count_table_columns(line) == 0) {
      free(line);
      break;
    }

    int row_cols;
    char **row = split_table_row(line, &row_cols);

    all_rows = realloc(all_rows, (table->rows + 1) * sizeof(char **));
    all_rows[table->rows] = row;

    row_col_counts = realloc(row_col_counts, (table->rows + 1) * sizeof(int));
    row_col_counts[table->rows] = row_cols;

    table->rows++;

    parser->pos = line_end - parser->text;
    if (*line_end == '\n')
      parser->pos++;

    free(line);
  }

  table->cells = malloc(table->rows * sizeof(ElementText **));
  for (size_t r = 0; r < table->rows; r++) {
    table->cells[r] = malloc(table->cols * sizeof(ElementText *));

    for (size_t c = 0; c < table->cols; c++) {
      table->cells[r][c] = malloc(sizeof(ElementText));
      memset(table->cells[r][c], 0, sizeof(ElementText));

      if (c < (size_t)row_col_counts[r] && all_rows[r][c]) {
        table->cells[r][c]->text = strdup_safe(all_rows[r][c]);
        table->cells[r][c]->level = 0;
        table->cells[r][c]->align = ALIGN_LEFT;
        table->cells[r][c]->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      } else {
        table->cells[r][c]->text = strdup_safe("");
        table->cells[r][c]->level = 0;
        table->cells[r][c]->align = ALIGN_LEFT;
        table->cells[r][c]->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      }
    }
  }

  for (size_t r = 0; r < table->rows; r++) {
    for (int c = 0; c < row_col_counts[r]; c++) {
      free(all_rows[r][c]);
    }
    free(all_rows[r]);
  }
  free(all_rows);
  free(row_col_counts);
  free(first_line);
  free(sep_line);

  return 0;
}

int markdown_to_json(const char *markdown, Document *doc) {
  editor_init(doc);

  const char *pos = markdown;
  const char *end = markdown + strlen(markdown);

  while (pos < end) {
    const char *line_end = strchr(pos, '\n');
    if (!line_end)
      line_end = end;

    size_t line_len = line_end - pos;
    if (line_len == 0) {
      // Create empty text element for empty line
      if (doc->elements_len >= doc->elements_capacity) {
        size_t new_cap =
            doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
        doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
        doc->elements_capacity = new_cap;
      }

      doc->elements[doc->elements_len].kind = T_TEXT;
      ElementText *text_element = &doc->elements[doc->elements_len].as.text;
      text_element->text = strdup_safe("");
      text_element->font = NULL;
      text_element->align = ALIGN_LEFT;
      text_element->level = 0;
      text_element->bold = false;
      text_element->italic = false;
      text_element->has_highlight = false;
      text_element->has_underline = false;
      text_element->font_size = 16;
      text_element->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      text_element->underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
      text_element->underline_gap = 7;
      text_element->highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
      text_element->spans = NULL;
      text_element->spans_count = 0;
      doc->elements_len++;

      pos = line_end < end ? line_end + 1 : end;
      continue;
    }

    char *line = malloc(line_len + 1);
    memcpy(line, pos, line_len);
    line[line_len] = '\0';
    trim_whitespace(line);

    if (strlen(line) == 0) {
      // Treat empty lines as empty text elements
      if (doc->elements_len >= doc->elements_capacity) {
        size_t new_cap =
            doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
        doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
        doc->elements_capacity = new_cap;
      }

      doc->elements[doc->elements_len].kind = T_TEXT;
      ElementText *text_element = &doc->elements[doc->elements_len].as.text;
      text_element->text = strdup_safe("");
      text_element->font = NULL;
      text_element->align = ALIGN_LEFT;
      text_element->level = 0;
      text_element->bold = false;
      text_element->italic = false;
      text_element->has_highlight = false;
      text_element->has_underline = false;
      text_element->font_size = 16;
      text_element->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      text_element->underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
      text_element->underline_gap = 7;
      text_element->highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
      text_element->spans = NULL;
      text_element->spans_count = 0;
      doc->elements_len++;

      free(line);
      pos = line_end < end ? line_end + 1 : end;
      continue;
    }

    ElementImage image;
    if (parse_image_line(line, &image) == 0) {
      if (doc->elements_len >= doc->elements_capacity) {
        size_t new_cap =
            doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
        doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
        doc->elements_capacity = new_cap;
      }

      doc->elements[doc->elements_len].kind = T_IMAGE;
      doc->elements[doc->elements_len].as.image = image;
      doc->elements_len++;
    } else if (strchr(line, '|')) {
      // Check if this could be a table
      MarkdownParser parser;
      parser.text = pos;
      parser.pos = 0;
      parser.len = end - pos;

      ElementTable table;
      if (parse_table_block(&parser, &table) == 0) {
        if (doc->elements_len >= doc->elements_capacity) {
          size_t new_cap =
              doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
          doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
          doc->elements_capacity = new_cap;
        }

        doc->elements[doc->elements_len].kind = T_TABLE;
        doc->elements[doc->elements_len].as.table = table;
        doc->elements_len++;

        // Advance position to after the table
        pos = pos + parser.pos;
        free(line);
        continue;
      }
    } else {
      ElementText text;
      if (parse_header_line(line, &text) != 0) {
        memset(&text, 0, sizeof(text));
        text.text = strdup_safe(line);
        text.level = 0;
        text.align = ALIGN_LEFT;
        text.color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
      }

      InlineSpan spans[32];
      int span_count = parse_inline_styles(text.text, spans, 32);
      if (span_count > 0) {
        text.spans = convert_spans_to_text_spans(text.text, spans, span_count,
                                                 &text.spans_count);
      } else {
        text.spans =
            convert_spans_to_text_spans(text.text, NULL, 0, &text.spans_count);
      }

      // Headers keep original case for idempotence

      // Replace text.text with cleaned text (without markdown markers)
      free(text.text);
      text.text = NULL;

      // Reconstruct clean text from spans
      size_t total_len = 0;
      for (size_t i = 0; i < text.spans_count; i++) {
        if (text.spans[i].text) {
          total_len += strlen(text.spans[i].text);
        }
      }

      text.text = malloc(total_len + 1);
      text.text[0] = '\0';
      for (size_t i = 0; i < text.spans_count; i++) {
        if (text.spans[i].text) {
          strcat(text.text, text.spans[i].text);
        }
      }

      // Set global style flags based on spans (preserve existing flags for
      // headers)
      bool preserve_bold = text.bold; // Headers already have bold=true
      text.bold = preserve_bold;
      text.italic = false;
      text.has_highlight = false;
      text.has_underline = false;
      for (size_t i = 0; i < text.spans_count; i++) {
        if (text.spans[i].bold)
          text.bold = true;
        if (text.spans[i].italic)
          text.italic = true;
        if (text.spans[i].has_highlight) {
          text.has_highlight = true;
          text.highlight_color = text.spans[i].highlight_color;
        }
        if (text.spans[i].has_underline) {
          text.has_underline = true;
          text.underline_color = text.spans[i].underline_color;
          text.underline_gap = text.spans[i].underline_gap;
        }
      }

      if (doc->elements_len >= doc->elements_capacity) {
        size_t new_cap =
            doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
        doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
        doc->elements_capacity = new_cap;
      }

      doc->elements[doc->elements_len].kind = T_TEXT;
      doc->elements[doc->elements_len].as.text = text;
      doc->elements_len++;
    }

    free(line);
    pos = line_end < end ? line_end + 1 : end;
  }

  return 0;
}

static void write_inline_markup(FILE *fp, const char *text, bool bold,
                                bool italic, bool highlight, bool underline) {
  if (bold && italic) {
    fprintf(fp, "***%s***", text);
  } else if (bold) {
    fprintf(fp, "**%s**", text);
  } else if (italic) {
    fprintf(fp, "*%s*", text);
  } else if (highlight) {
    fprintf(fp, "==%s==", text);
  } else if (underline) {
    fprintf(fp, "++%s++", text);
  } else {
    fprintf(fp, "%s", text);
  }
}

int json_to_markdown(const Document *doc, char **out_markdown) {
  FILE *fp = tmpfile();
  if (!fp)
    return -1;

  for (size_t i = 0; i < doc->elements_len; i++) {
    const Element *elem = &doc->elements[i];

    switch (elem->kind) {
    case T_TEXT: {
      const ElementText *text = &elem->as.text;

      // Check if this text element has any actual content
      bool has_content = false;

      // Check if element has explicit text content (for empty lines)
      if (text->text && strlen(text->text) == 0 && text->spans_count == 0) {
        // This is an empty line - output as empty line
        fprintf(fp, "\n");
        break;
      }

      // Check spans for content
      for (size_t s = 0; s < text->spans_count; s++) {
        const char *span_text = text->spans[s].text ? text->spans[s].text : "";
        if (strlen(span_text) > 0) {
          has_content = true;
          break;
        }
      }

      // Only output if there's actual content or it's a header (level > 0)
      if (has_content || text->level > 0) {
        if (text->level > 0) {
          for (int j = 0; j < text->level; j++) {
            fputc('#', fp);
          }
          fputc(' ', fp);
        }

        // Reconstruct content from spans
        for (size_t s = 0; s < text->spans_count; s++) {
          const char *span_text =
              text->spans[s].text ? text->spans[s].text : "";
          write_inline_markup(
              fp, span_text, text->spans[s].bold, text->spans[s].italic,
              text->spans[s].has_highlight, text->spans[s].has_underline);
        }

        fprintf(fp, "\n");
      }
      break;
    }

    case T_IMAGE: {
      const ElementImage *image = &elem->as.image;
      fprintf(fp, "![%s](%s)", image->alt ? image->alt : "",
              image->src ? image->src : "");

      if (image->width > 0 || image->height > 0 || image->alpha != 1.0f ||
          image->align != ALIGN_LEFT) {
        fprintf(fp, "{");
        bool first = true;

        if (image->width > 0) {
          fprintf(fp, "w=%d", image->width);
          first = false;
        }
        if (image->height > 0) {
          if (!first)
            fprintf(fp, " ");
          fprintf(fp, "h=%d", image->height);
          first = false;
        }
        if (image->alpha != 1.0f) {
          if (!first)
            fprintf(fp, " ");
          fprintf(fp, "a=%.3f", image->alpha);
          first = false;
        }
        if (image->align != ALIGN_LEFT) {
          if (!first)
            fprintf(fp, " ");
          const char *align_str = "left";
          switch (image->align) {
          case ALIGN_CENTER:
            align_str = "center";
            break;
          case ALIGN_RIGHT:
            align_str = "right";
            break;
          default:
            align_str = "left";
            break;
          }
          fprintf(fp, "align=%s", align_str);
        }

        fprintf(fp, "}");
      }

      fprintf(fp, "\n");
      break;
    }

    case T_TABLE: {
      const ElementTable *table = &elem->as.table;

      for (size_t c = 0; c < table->cols; c++) {
        fprintf(fp, "| ");
        if (table->cells[0] && table->cells[0][c] && table->cells[0][c]->text) {
          write_inline_markup(
              fp, table->cells[0][c]->text, table->cells[0][c]->bold,
              table->cells[0][c]->italic, table->cells[0][c]->has_highlight,
              table->cells[0][c]->has_underline);
        }
        fprintf(fp, " ");
      }
      fprintf(fp, "|\n");

      for (size_t c = 0; c < table->cols; c++) {
        fprintf(fp, "|---");
      }
      fprintf(fp, "|\n");

      for (size_t r = 1; r < table->rows; r++) {
        for (size_t c = 0; c < table->cols; c++) {
          fprintf(fp, "| ");
          if (table->cells[r] && table->cells[r][c] &&
              table->cells[r][c]->text) {
            write_inline_markup(
                fp, table->cells[r][c]->text, table->cells[r][c]->bold,
                table->cells[r][c]->italic, table->cells[r][c]->has_highlight,
                table->cells[r][c]->has_underline);
          }
          fprintf(fp, " ");
        }
        fprintf(fp, "|\n");
      }
      break;
    }
    }
  }

  fseek(fp, 0, SEEK_END);
  long len = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  *out_markdown = malloc(len + 1);
  size_t bytes_read = fread(*out_markdown, 1, len, fp);
  (void)bytes_read; // Suppress unused variable warning
  (*out_markdown)[len] = '\0';

  fclose(fp);
  return 0;
}

// ============= NEW ADVANCED MARKDOWN FUNCTIONS =============

// Parse inline code (`code`)
int parse_code_blocks(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i < len && span_count < max_spans; i++) {
    if (text[i] == '`') {
      size_t start = i;
      
      // Find closing `
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == '`') {
          spans[span_count].style = INLINE_CODE;
          spans[span_count].start = start;
          spans[span_count].end = j + 1;
          span_count++;
          i = j;
          break;
        }
      }
    }
  }
  
  return span_count;
}

// Parse strikethrough (~~text~~)
int parse_strikethrough(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i + 1 < len && span_count < max_spans; i++) {
    if (text[i] == '~' && text[i + 1] == '~') {
      size_t start = i;
      
      // Find closing ~~
      for (size_t j = i + 2; j + 1 < len; j++) {
        if (text[j] == '~' && text[j + 1] == '~') {
          spans[span_count].style = INLINE_STRIKETHROUGH;
          spans[span_count].start = start;
          spans[span_count].end = j + 2;
          span_count++;
          i = j + 1;
          break;
        }
      }
    }
  }
  
  return span_count;
}

// Parse links [text](url) and images ![alt](url)
int parse_links_and_images(const char *text, InlineSpan *spans, size_t max_spans) {
  if (!text || !spans || max_spans == 0) return 0;
  
  size_t len = strlen(text);
  size_t span_count = 0;
  
  for (size_t i = 0; i < len && span_count < max_spans; i++) {
    // Check for image ![
    if (i + 1 < len && text[i] == '!' && text[i + 1] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_start = 0;
      size_t paren_end = 0;
      
      // Find closing ]
      for (size_t j = i + 2; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      
      // Find opening (
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        paren_start = bracket_end + 1;
        
        // Find closing )
        for (size_t j = paren_start + 1; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        
        if (paren_end > paren_start) {
          spans[span_count].style = INLINE_IMAGE_REF;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end;
        }
      }
    }
    // Check for link [
    else if (text[i] == '[') {
      size_t start = i;
      size_t bracket_end = 0;
      size_t paren_start = 0;
      size_t paren_end = 0;
      
      // Find closing ]
      for (size_t j = i + 1; j < len; j++) {
        if (text[j] == ']') {
          bracket_end = j;
          break;
        }
      }
      
      // Find opening (
      if (bracket_end > 0 && bracket_end + 1 < len && text[bracket_end + 1] == '(') {
        paren_start = bracket_end + 1;
        
        // Find closing )
        for (size_t j = paren_start + 1; j < len; j++) {
          if (text[j] == ')') {
            paren_end = j;
            break;
          }
        }
        
        if (paren_end > paren_start) {
          spans[span_count].style = INLINE_LINK;
          spans[span_count].start = start;
          spans[span_count].end = paren_end + 1;
          span_count++;
          i = paren_end;
        }
      }
    }
  }
  
  return span_count;
}

// Validate URL format
bool is_valid_url(const char *url) {
  if (!url || strlen(url) < 4) return false;
  
  // Check for common protocols
  if (strncmp(url, "http://", 7) == 0 || 
      strncmp(url, "https://", 8) == 0 ||
      strncmp(url, "ftp://", 6) == 0 ||
      strncmp(url, "file://", 7) == 0) {
    return true;
  }
  
  // Check for relative URLs starting with /
  if (url[0] == '/') return true;
  
  // Check for anchor links
  if (url[0] == '#') return true;
  
  return false;
}

// Check if line is a markdown heading
bool is_markdown_heading(const char *line) {
  if (!line) return false;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  return (*line == '#');
}

// Get heading level (1-6)
int get_heading_level(const char *line) {
  if (!line) return 0;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  int level = 0;
  while (*line == '#' && level < 6) {
    level++;
    line++;
  }
  
  // Must be followed by space or end of line
  if (*line == ' ' || *line == '\t' || *line == '\0' || *line == '\n') {
    return level;
  }
  
  return 0;
}

// Extract heading text (without # symbols)
char* extract_heading_text(const char *line) {
  if (!line) return NULL;
  
  // Skip leading whitespace
  while (*line == ' ' || *line == '\t') line++;
  
  // Skip # symbols
  while (*line == '#') line++;
  
  // Skip space after #
  while (*line == ' ' || *line == '\t') line++;
  
  // Find end of line
  const char *end = line;
  while (*end != '\0' && *end != '\n' && *end != '\r') {
    end++;
  }
  
  // Remove trailing whitespace
  while (end > line && (*(end - 1) == ' ' || *(end - 1) == '\t')) {
    end--;
  }
  
  // Create result string
  size_t len = end - line;
  char *result = malloc(len + 1);
  if (result) {
    strncpy(result, line, len);
    result[len] = '\0';
  }
  
  return result;
}

// Validate markdown structure
bool validate_markdown_structure(const char *markdown) {
  if (!markdown) return false;
  
  // Check for balanced brackets
  int bracket_count = 0;
  int paren_count = 0;
  int code_block_count = 0;
  
  const char *p = markdown;
  while (*p) {
    switch (*p) {
      case '[':
        bracket_count++;
        break;
      case ']':
        bracket_count--;
        if (bracket_count < 0) return false;
        break;
      case '(':
        paren_count++;
        break;
      case ')':
        paren_count--;
        if (paren_count < 0) return false;
        break;
      case '`':
        // Check for triple backticks
        if (p[1] == '`' && p[2] == '`') {
          code_block_count++;
          p += 2; // Skip the other two backticks
        }
        break;
    }
    p++;
  }
  
  // All brackets and parentheses should be balanced
  // Code blocks should be even (paired)
  return (bracket_count == 0 && paren_count == 0 && (code_block_count % 2) == 0);
}

// Enhance markdown formatting
char* enhance_markdown_formatting(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  // Allocate extra space for enhancements
  char *result = malloc(len * 2 + 1000);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  
  while (*src) {
    // Auto-format headings
    if (*src == '#') {
      // Count heading level
      int level = 0;
      const char *p = src;
      while (*p == '#' && level < 6) {
        level++;
        p++;
      }
      
      // Add proper spacing
      for (int i = 0; i < level; i++) {
        *dst++ = '#';
      }
      
      if (*p != ' ' && *p != '\0' && *p != '\n') {
        *dst++ = ' ';
      }
      
      src = p;
      continue;
    }
    
    // Auto-format list items
    if ((*src == '-' || *src == '*' || *src == '+') && 
        (src == markdown || *(src - 1) == '\n')) {
      *dst++ = *src++;
      if (*src != ' ') {
        *dst++ = ' ';
      }
      continue;
    }
    
    *dst++ = *src++;
  }
  
  *dst = '\0';
  return result;
}

// Auto-format lists
char* auto_format_lists(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  char *result = malloc(len * 2 + 1);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  bool at_line_start = true;
  
  while (*src) {
    if (at_line_start && (*src == '-' || *src == '*' || *src == '+')) {
      *dst++ = '-'; // Standardize to dash
      *dst++ = ' ';
      src++;
      // Skip existing space if present
      if (*src == ' ') src++;
      at_line_start = false;
      continue;
    }
    
    *dst++ = *src;
    at_line_start = (*src == '\n');
    src++;
  }
  
  *dst = '\0';
  return result;
}

// Fix markdown spacing
char* fix_markdown_spacing(const char *markdown) {
  if (!markdown) return NULL;
  
  size_t len = strlen(markdown);
  char *result = malloc(len * 2 + 1);
  if (!result) return NULL;
  
  const char *src = markdown;
  char *dst = result;
  char prev_char = '\0';
  
  while (*src) {
    char curr_char = *src;
    
    // Add proper spacing around formatting markers
    if ((curr_char == '*' || curr_char == '_' || curr_char == '`') &&
        prev_char != ' ' && prev_char != '\0' && prev_char != '\n' &&
        !isalnum(prev_char)) {
      // Don't add space if it's part of markdown syntax
    }
    
    // Remove multiple consecutive newlines (max 2)
    if (curr_char == '\n' && prev_char == '\n') {
      // Look ahead for more newlines
      const char *p = src + 1;
      while (*p == '\n') p++;
      if (p > src + 1) {
        // Skip extra newlines, keep only one more
        src = p - 1;
      }
    }
    
    *dst++ = *src;
    prev_char = curr_char;
    src++;
  }
  
  *dst = '\0';
  return result;
}
