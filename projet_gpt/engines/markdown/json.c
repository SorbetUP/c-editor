#include "json.h"
#include <ctype.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *json_strdup(const char *str) {
  if (!str)
    return NULL;
  size_t len = strlen(str);
  char *copy = malloc(len + 1);
  if (!copy)
    return NULL;
  memcpy(copy, str, len + 1);
  return copy;
}

static bool json_ensure_document_capacity(Document *doc, size_t extra) {
  if (!doc)
    return false;

  size_t needed = doc->elements_len + extra;
  if (doc->elements_capacity >= needed)
    return true;

  size_t new_cap = doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
  while (new_cap < needed)
    new_cap *= 2;

  Element *new_elements = realloc(doc->elements, new_cap * sizeof(Element));
  if (!new_elements)
    return false;

  doc->elements = new_elements;
  doc->elements_capacity = new_cap;
  return true;
}

static void skip_whitespace(JsonParser *parser) {
  while (parser->pos < parser->len && (parser->json[parser->pos] == ' ' ||
                                       parser->json[parser->pos] == '\t' ||
                                       parser->json[parser->pos] == '\r' ||
                                       parser->json[parser->pos] == '\n')) {
    parser->pos++;
  }
}

static int parse_string_value(JsonParser *parser, char **out) {
  skip_whitespace(parser);
  if (parser->pos >= parser->len || parser->json[parser->pos] != '"') {
    return -1;
  }

  parser->pos++;
  size_t start = parser->pos;

  while (parser->pos < parser->len && parser->json[parser->pos] != '"') {
    if (parser->json[parser->pos] == '\\') {
      parser->pos++;
      if (parser->pos >= parser->len)
        return -1;
    }
    parser->pos++;
  }

  if (parser->pos >= parser->len)
    return -1;

  size_t len = parser->pos - start;
  *out = malloc(len + 1);
  if (!*out)
    return -1;
  memcpy(*out, parser->json + start, len);
  (*out)[len] = '\0';

  parser->pos++;
  return 0;
}

static int parse_number_value(JsonParser *parser, double *out) {
  skip_whitespace(parser);
  char *endptr;
  *out = strtod(parser->json + parser->pos, &endptr);

  if (endptr == parser->json + parser->pos) {
    return -1;
  }

  parser->pos = endptr - parser->json;
  return 0;
}

static int parse_bool_value(JsonParser *parser, bool *out) {
  skip_whitespace(parser);

  if (parser->pos + 4 <= parser->len &&
      strncmp(parser->json + parser->pos, "true", 4) == 0) {
    *out = true;
    parser->pos += 4;
    return 0;
  } else if (parser->pos + 5 <= parser->len &&
             strncmp(parser->json + parser->pos, "false", 5) == 0) {
    *out = false;
    parser->pos += 5;
    return 0;
  }

  return -1;
}

static int parse_rgba_array(JsonParser *parser, RGBA *rgba) {
  skip_whitespace(parser);
  if (parser->pos >= parser->len || parser->json[parser->pos] != '[') {
    return -1;
  }
  parser->pos++;

  double values[4];
  for (int i = 0; i < 4; i++) {
    skip_whitespace(parser);
    if (parse_number_value(parser, &values[i]) != 0) {
      return -1;
    }

    skip_whitespace(parser);
    if (i < 3) {
      if (parser->pos >= parser->len || parser->json[parser->pos] != ',') {
        return -1;
      }
      parser->pos++;
    }
  }

  skip_whitespace(parser);
  if (parser->pos >= parser->len || parser->json[parser->pos] != ']') {
    return -1;
  }
  parser->pos++;

  rgba->r = (float)values[0];
  rgba->g = (float)values[1];
  rgba->b = (float)values[2];
  rgba->a = (float)values[3];

  return 0;
}

static int find_object_key(JsonParser *parser, const char *key) {
  skip_whitespace(parser);
  if (parser->pos >= parser->len || parser->json[parser->pos] != '{') {
    return -1;
  }
  parser->pos++;

  while (parser->pos < parser->len) {
    skip_whitespace(parser);

    if (parser->json[parser->pos] == '}') {
      return -1;
    }

    char *found_key;
    if (parse_string_value(parser, &found_key) != 0) {
      return -1;
    }

    skip_whitespace(parser);
    if (parser->pos >= parser->len || parser->json[parser->pos] != ':') {
      free(found_key);
      return -1;
    }
    parser->pos++;

    if (strcmp(found_key, key) == 0) {
      free(found_key);
      return 0;
    }

    free(found_key);

    int depth = 0;
    while (parser->pos < parser->len) {
      char c = parser->json[parser->pos];
      if (c == '{' || c == '[')
        depth++;
      else if (c == '}' || c == ']') {
        if (depth == 0)
          break;
        depth--;
      } else if (c == ',' && depth == 0) {
        parser->pos++;
        break;
      } else if (c == '"') {
        parser->pos++;
        while (parser->pos < parser->len && parser->json[parser->pos] != '"') {
          if (parser->json[parser->pos] == '\\')
            parser->pos++;
          parser->pos++;
        }
      }
      parser->pos++;
    }
  }

  return -1;
}

static inline float clamp01f(float x) {
  return x < 0.0f ? 0.0f : x > 1.0f ? 1.0f : x;
}

static void rgba_normalize(RGBA *c) {
  c->r = clamp01f(c->r);
  c->g = clamp01f(c->g);
  c->b = clamp01f(c->b);
  c->a = clamp01f(c->a);
}

static void write_rgba_array(FILE *fp, const RGBA *rgba) {
  RGBA normalized = *rgba;
  rgba_normalize(&normalized);
  fprintf(fp, "[%.3f,%.3f,%.3f,%.3f]", normalized.r, normalized.g, normalized.b,
          normalized.a);
}

static void write_escaped_string(FILE *fp, const char *str) {
  fputc('"', fp);
  while (*str) {
    switch (*str) {
    case '"':
      fputs("\\\"", fp);
      break;
    case '\\':
      fputs("\\\\", fp);
      break;
    case '\n':
      fputs("\\n", fp);
      break;
    case '\r':
      fputs("\\r", fp);
      break;
    case '\t':
      fputs("\\t", fp);
      break;
    default:
      fputc(*str, fp);
      break;
    }
    str++;
  }
  fputc('"', fp);
}

static void json_free_text(ElementText *text) {
  if (!text)
    return;

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

static int write_element_text(FILE *fp, const ElementText *text) {
  fprintf(fp, "{\"type\":\"text\",\"text\":");
  write_escaped_string(fp, text->text ? text->text : "");

  const char *align_str = "left";
  switch (text->align) {
  case ALIGN_CENTER:
    align_str = "center";
    break;
  case ALIGN_RIGHT:
    align_str = "right";
    break;
  case ALIGN_JUSTIFY:
    align_str = "justify";
    break;
  case ALIGN_LEFT:
  default:
    align_str = "left";
    break;
  }
  fprintf(fp, ",\"align\":\"%s\"", align_str);

  if (text->font) {
    fprintf(fp, ",\"font\":");
    write_escaped_string(fp, text->font);
  }

  if (text->font_size > 0) {
    fprintf(fp, ",\"font_size\":%d", text->font_size);
  }

  fprintf(fp, ",\"color\":");
  write_rgba_array(fp, &text->color);

  fprintf(fp, ",\"bold\":%s", text->bold ? "true" : "false");
  fprintf(fp, ",\"italic\":%s", text->italic ? "true" : "false");

  if (text->has_underline) {
    fprintf(fp, ",\"underline\":{\"color\":");
    write_rgba_array(fp, &text->underline_color);
    fprintf(fp, ",\"gap\":%d}", text->underline_gap);
  }

  if (text->has_highlight) {
    fprintf(fp, ",\"highlight\":{\"color\":");
    write_rgba_array(fp, &text->highlight_color);
    fprintf(fp, "}");
  }

  if (text->spans && text->spans_count > 0) {
    fprintf(fp, ",\"spans\":[");
    for (size_t i = 0; i < text->spans_count; i++) {
      if (i > 0)
        fprintf(fp, ",");
      const TextSpan *span = &text->spans[i];
      fprintf(fp, "{");

      fprintf(fp, "\"text\":");
      write_escaped_string(fp, span->text ? span->text : "");

      fprintf(fp, ",\"bold\":%s", span->bold ? "true" : "false");
      fprintf(fp, ",\"italic\":%s", span->italic ? "true" : "false");
      fprintf(fp, ",\"code\":%s", span->code ? "true" : "false");
      fprintf(fp, ",\"strikethrough\":%s", span->strikethrough ? "true" : "false");

      if (span->has_underline) {
        fprintf(fp, ",\"has_underline\":true,\"underline_color\":");
        write_rgba_array(fp, &span->underline_color);
        fprintf(fp, ",\"underline_gap\":%d", span->underline_gap);
      } else {
        fprintf(fp, ",\"has_underline\":false");
      }

      if (span->has_highlight) {
        fprintf(fp, ",\"has_highlight\":true,\"highlight_color\":");
        write_rgba_array(fp, &span->highlight_color);
      } else {
        fprintf(fp, ",\"has_highlight\":false");
      }

      if (span->is_link && span->link_href) {
        fprintf(fp, ",\"link\":true,\"href\":");
        write_escaped_string(fp, span->link_href);
        fprintf(fp, ",\"note_link\":%s", span->is_note_link ? "true" : "false");
      } else {
        fprintf(fp, ",\"link\":false,\"note_link\":false");
      }

      if (span->is_image && span->image_src) {
        fprintf(fp, ",\"image\":true,\"src\":");
        write_escaped_string(fp, span->image_src);
        fprintf(fp, ",\"alt\":");
        write_escaped_string(fp, span->image_alt ? span->image_alt : "");
      } else {
        fprintf(fp, ",\"image\":false");
      }

      fprintf(fp, "}");
    }
    fprintf(fp, "]");
  }

  fprintf(fp, ",\"level\":%d}", text->level);
  return 0;
}

static int write_element_image(FILE *fp, const ElementImage *image) {
  fprintf(fp, "{\"type\":\"image\",\"src\":");
  write_escaped_string(fp, image->src ? image->src : "");
  fprintf(fp, ",\"alt\":");
  write_escaped_string(fp, image->alt ? image->alt : "");

  const char *align_str = "left";
  switch (image->align) {
  case ALIGN_CENTER:
    align_str = "center";
    break;
  case ALIGN_RIGHT:
    align_str = "right";
    break;
  case ALIGN_LEFT:
  default:
    align_str = "left";
    break;
  case ALIGN_JUSTIFY:
    align_str = "left";
    break;
  }
  fprintf(fp, ",\"align\":\"%s\"", align_str);

  fprintf(fp, ",\"width\":%d", image->width);
  fprintf(fp, ",\"height\":%d", image->height);
  fprintf(fp, ",\"alpha\":%.3f}", image->alpha);
  return 0;
}

static int write_element_code(FILE *fp, const ElementCode *code) {
  fprintf(fp, "{\"type\":\"code\",\"content\":");
  write_escaped_string(fp, code->content ? code->content : "");
  fprintf(fp, ",\"language\":");
  write_escaped_string(fp, (code->language && code->language[0]) ? code->language : "");
  fprintf(fp, ",\"fenced\":%s}", code->fenced ? "true" : "false");
  return 0;
}

static const char *list_kind_to_string(ListKind kind) {
  switch (kind) {
  case LIST_KIND_ORDERED:
    return "ordered";
  case LIST_KIND_TASK:
    return "task";
  case LIST_KIND_DEFINITION:
    return "definition";
  case LIST_KIND_BULLET:
  default:
    return "bullet";
  }
}

static int write_element_list(FILE *fp, const ElementList *list) {
  fprintf(fp,
          "{\"type\":\"list\",\"kind\":\"%s\",\"ordered\":%s,\"start\":%d,\"items\":[",
          list_kind_to_string(list->kind), list->ordered ? "true" : "false",
          list->start_index > 0 ? list->start_index : 1);

  for (size_t i = 0; i < list->item_count; i++) {
    if (i > 0)
      fprintf(fp, ",");
    const ElementListItem *item = &list->items[i];
    fprintf(fp,
            "{\"indent\":%d,\"checkbox\":%s,\"checked\":%s,\"isTask\":%s,\"isDefinition\":%s,\"number\":%d,\"text\":",
            item->indent_level, item->has_checkbox ? "true" : "false",
            item->checkbox_checked ? "true" : "false",
            item->is_task ? "true" : "false",
            item->is_definition ? "true" : "false", item->number);
    write_element_text(fp, &item->text);
    if (item->is_definition) {
      fprintf(fp, ",\"term\":");
      write_element_text(fp, &item->term);
      fprintf(fp, ",\"definition\":");
      write_element_text(fp, &item->definition);
    }
    fprintf(fp, "}");
  }

  fprintf(fp, "]}");
  return 0;
}

static int write_element_quote(FILE *fp, const ElementQuote *quote) {
  fprintf(fp, "{\"type\":\"quote\",\"items\":[");
  for (size_t i = 0; i < quote->item_count; i++) {
    if (i > 0)
      fprintf(fp, ",");
    write_element_text(fp, &quote->items[i]);
  }
  fprintf(fp, "]}");
  return 0;
}

static int write_element_divider(FILE *fp, const ElementDivider *divider) {
  fprintf(fp, "{\"type\":\"divider\",\"thickness\":%d,\"color\":",
          divider->thickness);
  write_rgba_array(fp, &divider->color);
  fprintf(fp, "}");
  return 0;
}

static int write_element_settings(FILE *fp, const ElementSettings *settings) {
  fprintf(fp, "{\"type\":\"settings\",\"name\":");
  write_escaped_string(fp, settings->name ? settings->name : "");
  fprintf(fp, ",\"value\":");
  write_escaped_string(fp, settings->value ? settings->value : "");
  fprintf(fp, "}");
  return 0;
}

static int parse_text_span(JsonParser *parser, TextSpan *span) {
  memset(span, 0, sizeof(*span));

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "text") == 0) {
    parse_string_value(&obj_parser, &span->text);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "bold") == 0) {
    parse_bool_value(&obj_parser, &span->bold);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "italic") == 0) {
    parse_bool_value(&obj_parser, &span->italic);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "code") == 0) {
    parse_bool_value(&obj_parser, &span->code);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "strikethrough") == 0) {
    parse_bool_value(&obj_parser, &span->strikethrough);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "has_underline") == 0) {
    bool has_underline = false;
    parse_bool_value(&obj_parser, &has_underline);
    span->has_underline = has_underline;
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "underline_color") == 0) {
    parse_rgba_array(&obj_parser, &span->underline_color);
    span->has_underline = true;
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "underline_gap") == 0) {
    double gap;
    if (parse_number_value(&obj_parser, &gap) == 0) {
      span->underline_gap = (int)gap;
      if (span->underline_gap < 0)
        span->underline_gap = 0;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "has_highlight") == 0) {
    bool has_highlight = false;
    parse_bool_value(&obj_parser, &has_highlight);
    span->has_highlight = has_highlight;
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "highlight_color") == 0) {
    parse_rgba_array(&obj_parser, &span->highlight_color);
    span->has_highlight = true;
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "link") == 0) {
    bool is_link = false;
    parse_bool_value(&obj_parser, &is_link);
    span->is_link = is_link;
  }

  obj_parser = *parser;
  if (span->is_link && find_object_key(&obj_parser, "href") == 0) {
    parse_string_value(&obj_parser, &span->link_href);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "note_link") == 0) {
    bool note_link = false;
    parse_bool_value(&obj_parser, &note_link);
    span->is_note_link = note_link;
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "image") == 0) {
    bool is_image = false;
    parse_bool_value(&obj_parser, &is_image);
    span->is_image = is_image;
  }

  obj_parser = *parser;
  if (span->is_image && find_object_key(&obj_parser, "src") == 0) {
    parse_string_value(&obj_parser, &span->image_src);
  }

  obj_parser = *parser;
  if (span->is_image && find_object_key(&obj_parser, "alt") == 0) {
    parse_string_value(&obj_parser, &span->image_alt);
  }

  if (!span->text) {
    span->text = json_strdup("");
  }
  if (span->is_link && span->is_note_link && !span->link_href) {
    span->is_note_link = false;
  }
  if (span->is_image && !span->image_alt) {
    span->image_alt = json_strdup("");
  }

  return 0;
}

static const char *align_to_string(Align align) {
  switch (align) {
  case ALIGN_CENTER:
    return "center";
  case ALIGN_RIGHT:
    return "right";
  case ALIGN_LEFT:
  case ALIGN_JUSTIFY:
  default:
    return "left";
  }
}

static int write_element_table(FILE *fp, const ElementTable *table) {
  fprintf(fp, "{\"type\":\"table\"");

  fprintf(fp, ",\"align\":[");
  for (size_t c = 0; c < table->cols; c++) {
    if (c > 0)
      fprintf(fp, ",");
    bool defined = table->column_align_defined &&
                   c < table->column_align_count &&
                   table->column_align_defined[c];
    Align align = ALIGN_LEFT;
    if (table->column_align && c < table->column_align_count) {
      align = table->column_align[c];
    }
    if (!defined) {
      fprintf(fp, "null");
    } else {
      fprintf(fp, "\"%s\"", align_to_string(align));
    }
  }
  fprintf(fp, "]");

  size_t header_rows = 0;
  if (table->header_rows > 0 && table->header_rows <= table->rows) {
    header_rows = table->header_rows;
  } else if (table->rows > 0) {
    header_rows = 1;
  }

  fprintf(fp, ",\"header\":[");
  if (header_rows > 0) {
    size_t header_index = 0;
    for (size_t c = 0; c < table->cols; c++) {
      if (c > 0)
        fprintf(fp, ",");
      fprintf(fp, "[");
      if (table->cells[header_index][c]) {
        write_element_text(fp, table->cells[header_index][c]);
      } else {
        fprintf(fp, "{\"type\":\"text\",\"text\":\"\",\"level\":0}");
      }
      fprintf(fp, "]");
    }
  }
  fprintf(fp, "]");

  fprintf(fp, ",\"rows\":[");
  bool first_row_written = false;
  for (size_t r = header_rows; r < table->rows; r++) {
    if (first_row_written)
      fprintf(fp, ",");
    first_row_written = true;
    fprintf(fp, "[");
    for (size_t c = 0; c < table->cols; c++) {
      if (c > 0)
        fprintf(fp, ",");
      fprintf(fp, "[");
      if (table->cells[r][c]) {
        write_element_text(fp, table->cells[r][c]);
      } else {
        fprintf(fp, "{\"type\":\"text\",\"text\":\"\",\"level\":0}");
      }
      fprintf(fp, "]");
    }
    fprintf(fp, "]");
  }
  
  // Add column width specifications for consistent line-by-line rendering
  fprintf(fp, ",\"columnWidths\":[");
  if (table->column_widths && table->widths_calculated) {
    for (size_t c = 0; c < table->cols; c++) {
      if (c > 0) fprintf(fp, ",");
      fprintf(fp, "%d", table->column_widths[c]);
    }
  } else {
    for (size_t c = 0; c < table->cols; c++) {
      if (c > 0) fprintf(fp, ",");
      fprintf(fp, "10"); // Default width
    }
  }
  fprintf(fp, "]");
  
  fprintf(fp, ",\"totalWidth\":%d", 
          table->widths_calculated ? table->total_content_width : 
          (int)(table->cols * 10 + (table->cols + 1) + (2 * table->cols)));
          
  fprintf(fp, "}");
  return 0;
}

int json_stringify(const Document *doc, char **out_json) {
  FILE *fp = tmpfile();
  if (!fp)
    return -1;

  fprintf(fp, "{\"name\":");
  write_escaped_string(fp, doc->name ? doc->name : "new note");

  fprintf(fp, ",\"meta\":{");
  fprintf(fp, "\"default\":{");
  fprintf(fp, "\"fontsize\":%d", doc->default_fontsize);
  fprintf(fp, ",\"font\":");
  write_escaped_string(fp, doc->default_font ? doc->default_font : "Helvetica");
  fprintf(fp, ",\"text_color\":");
  write_rgba_array(fp, &doc->default_text_color);
  fprintf(fp, ",\"highlight_color\":");
  write_rgba_array(fp, &doc->default_highlight_color);
  fprintf(fp, "}");
  fprintf(fp, ",\"icon\":\"\"");
  fprintf(fp, ",\"updated\":%ld", doc->updated);
  fprintf(fp, ",\"created\":%ld", doc->created);
  fprintf(fp, "}");

  fprintf(fp, ",\"elements\":[");
  for (size_t i = 0; i < doc->elements_len; i++) {
    if (i > 0)
      fprintf(fp, ",");

    switch (doc->elements[i].kind) {
    case T_TEXT:
      write_element_text(fp, &doc->elements[i].as.text);
      break;
    case T_IMAGE:
      write_element_image(fp, &doc->elements[i].as.image);
      break;
    case T_TABLE:
      write_element_table(fp, &doc->elements[i].as.table);
      break;
    case T_CODE:
      write_element_code(fp, &doc->elements[i].as.code);
      break;
    case T_LIST:
      write_element_list(fp, &doc->elements[i].as.list);
      break;
    case T_QUOTE:
      write_element_quote(fp, &doc->elements[i].as.quote);
      break;
    case T_DIVIDER:
      write_element_divider(fp, &doc->elements[i].as.divider);
      break;
    case T_SETTINGS:
      write_element_settings(fp, &doc->elements[i].as.settings);
      break;
    }
  }
  fprintf(fp, "]}");

  fseek(fp, 0, SEEK_END);
  long len = ftell(fp);
  fseek(fp, 0, SEEK_SET);

  *out_json = malloc(len + 1);
  size_t bytes_read = fread(*out_json, 1, len, fp);
  (void)bytes_read; // Suppress unused variable warning
  (*out_json)[len] = '\0';

  fclose(fp);
  return 0;
}


static int parse_element_text(JsonParser *parser, ElementText *text) {
  memset(text, 0, sizeof(*text));
  text->align = ALIGN_LEFT;
  text->font_size = 0;
  text->color = (RGBA){0.0f, 0.0f, 0.0f, 1.0f};
  text->highlight_color = (RGBA){1.0f, 1.0f, 0.0f, 0.3f};
  text->underline_color = (RGBA){0.0f, 0.0f, 0.0f, 0.4f};
  text->underline_gap = 7;

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "text") == 0) {
    parse_string_value(&obj_parser, &text->text);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "align") == 0) {
    char *align_str = NULL;
    if (parse_string_value(&obj_parser, &align_str) == 0 && align_str) {
      if (strcmp(align_str, "center") == 0) {
        text->align = ALIGN_CENTER;
      } else if (strcmp(align_str, "right") == 0) {
        text->align = ALIGN_RIGHT;
      } else if (strcmp(align_str, "justify") == 0) {
        text->align = ALIGN_JUSTIFY;
      } else {
        text->align = ALIGN_LEFT;
      }
      free(align_str);
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "font") == 0) {
    parse_string_value(&obj_parser, &text->font);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "font_size") == 0) {
    double size;
    if (parse_number_value(&obj_parser, &size) == 0) {
      text->font_size = (int)size;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "color") == 0) {
    parse_rgba_array(&obj_parser, &text->color);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "bold") == 0) {
    parse_bool_value(&obj_parser, &text->bold);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "italic") == 0) {
    parse_bool_value(&obj_parser, &text->italic);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "underline") == 0) {
    skip_whitespace(&obj_parser);
    if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == '{') {
      size_t under_start = obj_parser.pos;
      int depth = 0;
      while (obj_parser.pos < obj_parser.len) {
        char ch = obj_parser.json[obj_parser.pos];
        if (ch == '{') {
          depth++;
        } else if (ch == '}') {
          depth--;
          if (depth == 0) {
            obj_parser.pos++;
            break;
          }
        } else if (ch == '"') {
          obj_parser.pos++;
          while (obj_parser.pos < obj_parser.len &&
                 obj_parser.json[obj_parser.pos] != '"') {
            if (obj_parser.json[obj_parser.pos] == '\\')
              obj_parser.pos++;
            obj_parser.pos++;
          }
        }
        obj_parser.pos++;
      }
      size_t under_end = obj_parser.pos;
      JsonParser under_parser = {parser->json, under_start, under_end - under_start};
      JsonParser tmp = under_parser;
      if (find_object_key(&tmp, "color") == 0) {
        parse_rgba_array(&tmp, &text->underline_color);
        text->has_underline = true;
      }
      tmp = under_parser;
      if (find_object_key(&tmp, "gap") == 0) {
        double gap;
        if (parse_number_value(&tmp, &gap) == 0) {
          text->underline_gap = (int)gap;
          text->has_underline = true;
        }
      }
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "highlight") == 0) {
    skip_whitespace(&obj_parser);
    if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == '{') {
      size_t high_start = obj_parser.pos;
      int depth = 0;
      while (obj_parser.pos < obj_parser.len) {
        char ch = obj_parser.json[obj_parser.pos];
        if (ch == '{') {
          depth++;
        } else if (ch == '}') {
          depth--;
          if (depth == 0) {
            obj_parser.pos++;
            break;
          }
        } else if (ch == '"') {
          obj_parser.pos++;
          while (obj_parser.pos < obj_parser.len &&
                 obj_parser.json[obj_parser.pos] != '"') {
            if (obj_parser.json[obj_parser.pos] == '\\')
              obj_parser.pos++;
            obj_parser.pos++;
          }
        }
        obj_parser.pos++;
      }
      size_t high_end = obj_parser.pos;
      JsonParser high_parser = {parser->json, high_start, high_end - high_start};
      JsonParser tmp = high_parser;
      if (find_object_key(&tmp, "color") == 0) {
        parse_rgba_array(&tmp, &text->highlight_color);
        text->has_highlight = true;
      }
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "level") == 0) {
    double level;
    if (parse_number_value(&obj_parser, &level) == 0) {
      text->level = (int)level;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "spans") == 0) {
    skip_whitespace(&obj_parser);
    if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == '[') {
      obj_parser.pos++;
      while (obj_parser.pos < obj_parser.len) {
        skip_whitespace(&obj_parser);
        if (obj_parser.json[obj_parser.pos] == ']') {
          obj_parser.pos++;
          break;
        }
        if (obj_parser.json[obj_parser.pos] != '{') {
          break;
        }
        size_t span_start = obj_parser.pos;
        int depth = 0;
        while (obj_parser.pos < obj_parser.len) {
          char ch = obj_parser.json[obj_parser.pos];
          if (ch == '{') {
            depth++;
          } else if (ch == '}') {
            depth--;
            if (depth == 0) {
              obj_parser.pos++;
              break;
            }
          } else if (ch == '"') {
            obj_parser.pos++;
            while (obj_parser.pos < obj_parser.len &&
                   obj_parser.json[obj_parser.pos] != '"') {
              if (obj_parser.json[obj_parser.pos] == '\\')
                obj_parser.pos++;
              obj_parser.pos++;
            }
          }
          obj_parser.pos++;
        }
        size_t span_end = obj_parser.pos;
        JsonParser span_parser = {parser->json, span_start, span_end - span_start};
        TextSpan span;
        parse_text_span(&span_parser, &span);
        TextSpan *new_spans = realloc(text->spans, (text->spans_count + 1) * sizeof(TextSpan));
        if (!new_spans) {
          free(span.text);
          free(span.link_href);
          free(span.image_src);
          free(span.image_alt);
          break;
        }
        text->spans = new_spans;
        text->spans[text->spans_count++] = span;

        skip_whitespace(&obj_parser);
        if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == ',') {
          obj_parser.pos++;
        }
      }
    }
  }

  if (!text->text) {
    text->text = json_strdup("");
  }

  return 0;
}



static int parse_element_code(JsonParser *parser, ElementCode *code) {
  memset(code, 0, sizeof(*code));
  code->fenced = true;

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "content") == 0) {
    parse_string_value(&obj_parser, &code->content);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "language") == 0) {
    parse_string_value(&obj_parser, &code->language);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "fenced") == 0) {
    parse_bool_value(&obj_parser, &code->fenced);
  }

  if (!code->content)
    code->content = json_strdup("");
  if (!code->language)
    code->language = json_strdup("");

  return 0;
}

static int parse_element_image(JsonParser *parser, ElementImage *image) {
  memset(image, 0, sizeof(*image));
  image->alpha = 1.0f;

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "src") == 0) {
    parse_string_value(&obj_parser, &image->src);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "alt") == 0) {
    parse_string_value(&obj_parser, &image->alt);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "width") == 0) {
    double width;
    if (parse_number_value(&obj_parser, &width) == 0) {
      image->width = (int)width;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "height") == 0) {
    double height;
    if (parse_number_value(&obj_parser, &height) == 0) {
      image->height = (int)height;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "alpha") == 0) {
    double alpha;
    if (parse_number_value(&obj_parser, &alpha) == 0) {
      image->alpha = (float)alpha;
    }
  }

  return 0;
}

static int parse_element_list(JsonParser *parser, ElementList *list) {
  memset(list, 0, sizeof(*list));
  list->start_index = 1;

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "ordered") == 0) {
    parse_bool_value(&obj_parser, &list->ordered);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "start") == 0) {
    double start;
    if (parse_number_value(&obj_parser, &start) == 0) {
      list->start_index = (int)start;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "kind") == 0) {
    char *kind_str = NULL;
    if (parse_string_value(&obj_parser, &kind_str) == 0 && kind_str) {
      if (strcmp(kind_str, "ordered") == 0) {
        list->kind = LIST_KIND_ORDERED;
      } else if (strcmp(kind_str, "task") == 0) {
        list->kind = LIST_KIND_TASK;
      } else if (strcmp(kind_str, "definition") == 0) {
        list->kind = LIST_KIND_DEFINITION;
      } else {
        list->kind = LIST_KIND_BULLET;
      }
      free(kind_str);
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "items") == 0) {
    skip_whitespace(&obj_parser);
    if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == '[') {
      obj_parser.pos++;
      while (obj_parser.pos < obj_parser.len) {
        skip_whitespace(&obj_parser);
        if (obj_parser.json[obj_parser.pos] == ']') {
          obj_parser.pos++;
          break;
        }
        if (obj_parser.json[obj_parser.pos] != '{') {
          break;
        }

        size_t item_start = obj_parser.pos;
        int depth = 0;
        while (obj_parser.pos < obj_parser.len) {
          char ch = obj_parser.json[obj_parser.pos];
          if (ch == '{') {
            depth++;
          } else if (ch == '}') {
            depth--;
            if (depth == 0) {
              obj_parser.pos++;
              break;
            }
          } else if (ch == '"') {
            obj_parser.pos++;
            while (obj_parser.pos < obj_parser.len &&
                   obj_parser.json[obj_parser.pos] != '"') {
              if (obj_parser.json[obj_parser.pos] == '\\')
                obj_parser.pos++;
              obj_parser.pos++;
            }
          }
          obj_parser.pos++;
        }
        size_t item_end = obj_parser.pos;
        JsonParser item_parser = {parser->json, item_start, item_end - item_start};

        ElementListItem item;
        memset(&item, 0, sizeof(item));

        JsonParser field_parser = item_parser;
        if (find_object_key(&field_parser, "indent") == 0) {
          double indent;
          if (parse_number_value(&field_parser, &indent) == 0) {
            item.indent_level = (int)indent;
          }
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "checkbox") == 0) {
          parse_bool_value(&field_parser, &item.has_checkbox);
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "isTask") == 0) {
          parse_bool_value(&field_parser, &item.is_task);
        } else {
          item.is_task = item.has_checkbox;
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "isDefinition") == 0) {
          parse_bool_value(&field_parser, &item.is_definition);
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "checked") == 0) {
          parse_bool_value(&field_parser, &item.checkbox_checked);
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "number") == 0) {
          double num;
          if (parse_number_value(&field_parser, &num) == 0) {
            item.number = (int)num;
          }
        }

        field_parser = item_parser;
        if (find_object_key(&field_parser, "text") == 0) {
          skip_whitespace(&field_parser);
          if (field_parser.pos < field_parser.len &&
              field_parser.json[field_parser.pos] == '{') {
            size_t text_start = field_parser.pos;
            int depth_text = 0;
            while (field_parser.pos < field_parser.len) {
              char ch = field_parser.json[field_parser.pos];
              if (ch == '{') {
                depth_text++;
              } else if (ch == '}') {
                depth_text--;
                if (depth_text == 0) {
                  field_parser.pos++;
                  break;
                }
              } else if (ch == '"') {
                field_parser.pos++;
                while (field_parser.pos < field_parser.len &&
                       field_parser.json[field_parser.pos] != '"') {
                  if (field_parser.json[field_parser.pos] == '\\')
                    field_parser.pos++;
                  field_parser.pos++;
                }
              }
              field_parser.pos++;
            }
            size_t text_end = field_parser.pos;
            JsonParser text_parser = {parser->json, text_start, text_end - text_start};
            parse_element_text(&text_parser, &item.text);
          }
        }

        if (item.is_definition) {
          field_parser = item_parser;
          if (find_object_key(&field_parser, "term") == 0) {
            skip_whitespace(&field_parser);
            if (field_parser.pos < field_parser.len &&
                field_parser.json[field_parser.pos] == '{') {
              size_t term_start = field_parser.pos;
              int depth_term = 0;
              while (field_parser.pos < field_parser.len) {
                char ch = field_parser.json[field_parser.pos];
                if (ch == '{') {
                  depth_term++;
                } else if (ch == '}') {
                  depth_term--;
                  if (depth_term == 0) {
                    field_parser.pos++;
                    break;
                  }
                } else if (ch == '"') {
                  field_parser.pos++;
                  while (field_parser.pos < field_parser.len &&
                         field_parser.json[field_parser.pos] != '"') {
                    if (field_parser.json[field_parser.pos] == '\\')
                      field_parser.pos++;
                    field_parser.pos++;
                  }
                }
                field_parser.pos++;
              }
              size_t term_end = field_parser.pos;
              JsonParser term_parser = {parser->json, term_start,
                                       term_end - term_start};
              parse_element_text(&term_parser, &item.term);
            }
          }

          field_parser = item_parser;
          if (find_object_key(&field_parser, "definition") == 0) {
            skip_whitespace(&field_parser);
            if (field_parser.pos < field_parser.len &&
                field_parser.json[field_parser.pos] == '{') {
              size_t def_start = field_parser.pos;
              int depth_def = 0;
              while (field_parser.pos < field_parser.len) {
                char ch = field_parser.json[field_parser.pos];
                if (ch == '{') {
                  depth_def++;
                } else if (ch == '}') {
                  depth_def--;
                  if (depth_def == 0) {
                    field_parser.pos++;
                    break;
                  }
                } else if (ch == '"') {
                  field_parser.pos++;
                  while (field_parser.pos < field_parser.len &&
                         field_parser.json[field_parser.pos] != '"') {
                    if (field_parser.json[field_parser.pos] == '\\')
                      field_parser.pos++;
                    field_parser.pos++;
                  }
                }
                field_parser.pos++;
              }
              size_t def_end = field_parser.pos;
              JsonParser def_parser = {parser->json, def_start,
                                       def_end - def_start};
              parse_element_text(&def_parser, &item.definition);
            }
          }
        }

        ElementListItem *new_items =
            realloc(list->items, (list->item_count + 1) * sizeof(ElementListItem));
        if (!new_items) {
          json_free_text(&item.text);
          if (item.is_definition) {
            json_free_text(&item.term);
            json_free_text(&item.definition);
          }
          break;
        }
        list->items = new_items;
        list->items[list->item_count++] = item;

        skip_whitespace(&obj_parser);
        if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == ',') {
          obj_parser.pos++;
        }
      }
    }
  }

  if (list->kind == LIST_KIND_BULLET) {
    if (list->ordered) {
      list->kind = LIST_KIND_ORDERED;
    } else {
      for (size_t i = 0; i < list->item_count; i++) {
        if (list->items[i].has_checkbox || list->items[i].is_task) {
          list->kind = LIST_KIND_TASK;
          break;
        }
        if (list->items[i].is_definition) {
          list->kind = LIST_KIND_DEFINITION;
          break;
        }
      }
    }
  }

  list->item_capacity = list->item_count;
  return 0;
}

static int parse_element_quote(JsonParser *parser, ElementQuote *quote) {
  memset(quote, 0, sizeof(*quote));

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "items") == 0) {
    skip_whitespace(&obj_parser);
    if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == '[') {
      obj_parser.pos++;
      while (obj_parser.pos < obj_parser.len) {
        skip_whitespace(&obj_parser);
        if (obj_parser.json[obj_parser.pos] == ']') {
          obj_parser.pos++;
          break;
        }
        if (obj_parser.json[obj_parser.pos] != '{') {
          break;
        }
        size_t text_start = obj_parser.pos;
        int depth = 0;
        while (obj_parser.pos < obj_parser.len) {
          char ch = obj_parser.json[obj_parser.pos];
          if (ch == '{') {
            depth++;
          } else if (ch == '}') {
            depth--;
            if (depth == 0) {
              obj_parser.pos++;
              break;
            }
          } else if (ch == '"') {
            obj_parser.pos++;
            while (obj_parser.pos < obj_parser.len &&
                   obj_parser.json[obj_parser.pos] != '"') {
              if (obj_parser.json[obj_parser.pos] == '\\')
                obj_parser.pos++;
              obj_parser.pos++;
            }
          }
          obj_parser.pos++;
        }
        size_t text_end = obj_parser.pos;
        JsonParser text_parser = {parser->json, text_start, text_end - text_start};
        ElementText inner_text;
        parse_element_text(&text_parser, &inner_text);

        ElementText *new_items =
            realloc(quote->items, (quote->item_count + 1) * sizeof(ElementText));
        if (!new_items) {
          json_free_text(&inner_text);
          break;
        }
        quote->items = new_items;
        quote->items[quote->item_count++] = inner_text;

        skip_whitespace(&obj_parser);
        if (obj_parser.pos < obj_parser.len && obj_parser.json[obj_parser.pos] == ',') {
          obj_parser.pos++;
        }
      }
    }
  }

  quote->item_capacity = quote->item_count;
  return 0;
}

static int parse_element_divider(JsonParser *parser, ElementDivider *divider) {
  memset(divider, 0, sizeof(*divider));
  divider->thickness = 1;
  divider->color = (RGBA){0.7f, 0.7f, 0.7f, 1.0f};

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "thickness") == 0) {
    double thick;
    if (parse_number_value(&obj_parser, &thick) == 0) {
      divider->thickness = (int)thick;
    }
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "color") == 0) {
    parse_rgba_array(&obj_parser, &divider->color);
  }

  return 0;
}

static int parse_element_settings(JsonParser *parser, ElementSettings *settings) {
  memset(settings, 0, sizeof(*settings));

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "name") == 0) {
    parse_string_value(&obj_parser, &settings->name);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "value") == 0) {
    parse_string_value(&obj_parser, &settings->value);
  }

  if (!settings->name)
    settings->name = json_strdup("");
  if (!settings->value)
    settings->value = json_strdup("");

  return 0;
}


int json_parse(const char *json_str, Document *doc) {
  JsonParser parser = {json_str, 0, strlen(json_str)};

  editor_init(doc);

  if (find_object_key(&parser, "name") == 0) {
    char *name;
    if (parse_string_value(&parser, &name) == 0) {
      free(doc->name);
      doc->name = name;
    }
  }

  JsonParser elements_parser = {json_str, 0, strlen(json_str)};
  if (find_object_key(&elements_parser, "elements") != 0) {
    return -1;
  }

  skip_whitespace(&elements_parser);
  if (elements_parser.pos >= elements_parser.len ||
      elements_parser.json[elements_parser.pos] != '[') {
    return -1;
  }
  elements_parser.pos++;

  while (elements_parser.pos < elements_parser.len) {
    skip_whitespace(&elements_parser);

    if (elements_parser.json[elements_parser.pos] == ']') {
      break;
    }

    if (elements_parser.json[elements_parser.pos] != '{') {
      return -1;
    }

    size_t obj_start = elements_parser.pos;
    int depth = 0;
    while (elements_parser.pos < elements_parser.len) {
      char c = elements_parser.json[elements_parser.pos];
      if (c == '{')
        depth++;
      else if (c == '}') {
        depth--;
        if (depth == 0) {
          elements_parser.pos++;
          break;
        }
      } else if (c == '"') {
        elements_parser.pos++;
        while (elements_parser.pos < elements_parser.len &&
               elements_parser.json[elements_parser.pos] != '"') {
          if (elements_parser.json[elements_parser.pos] == '\\')
            elements_parser.pos++;
          elements_parser.pos++;
        }
      }
      elements_parser.pos++;
    }

    JsonParser elem_parser = {json_str, obj_start,
                              elements_parser.pos - obj_start};

    char *type_str = NULL;
    if (find_object_key(&elem_parser, "type") == 0) {
      parse_string_value(&elem_parser, &type_str);
    }

    if (type_str && strcmp(type_str, "text") == 0) {
      elem_parser.pos = 0;
      ElementText text;
      if (parse_element_text(&elem_parser, &text) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_TEXT;
        doc->elements[doc->elements_len].as.text = text;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "image") == 0) {
      elem_parser.pos = 0;
      ElementImage image;
      if (parse_element_image(&elem_parser, &image) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_IMAGE;
        doc->elements[doc->elements_len].as.image = image;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "code") == 0) {
      elem_parser.pos = 0;
      ElementCode code;
      if (parse_element_code(&elem_parser, &code) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_CODE;
        doc->elements[doc->elements_len].as.code = code;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "list") == 0) {
      elem_parser.pos = 0;
      ElementList list_elem;
      if (parse_element_list(&elem_parser, &list_elem) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_LIST;
        doc->elements[doc->elements_len].as.list = list_elem;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "quote") == 0) {
      elem_parser.pos = 0;
      ElementQuote quote_elem;
      if (parse_element_quote(&elem_parser, &quote_elem) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_QUOTE;
        doc->elements[doc->elements_len].as.quote = quote_elem;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "divider") == 0) {
      elem_parser.pos = 0;
      ElementDivider divider;
      if (parse_element_divider(&elem_parser, &divider) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_DIVIDER;
        doc->elements[doc->elements_len].as.divider = divider;
        doc->elements_len++;
      }
    } else if (type_str && strcmp(type_str, "settings") == 0) {
      elem_parser.pos = 0;
      ElementSettings settings_elem;
      if (parse_element_settings(&elem_parser, &settings_elem) == 0 &&
          json_ensure_document_capacity(doc, 1)) {
        doc->elements[doc->elements_len].kind = T_SETTINGS;
        doc->elements[doc->elements_len].as.settings = settings_elem;
        doc->elements_len++;
      }
    }

    free(type_str);

    skip_whitespace(&elements_parser);
    if (elements_parser.pos < elements_parser.len &&
        elements_parser.json[elements_parser.pos] == ',') {
      elements_parser.pos++;
    }
  }

  return 0;
}
