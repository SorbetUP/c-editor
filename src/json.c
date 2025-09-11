#include "json.h"
#include <ctype.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

static int write_element_table(FILE *fp, const ElementTable *table) {
  fprintf(fp, "{\"type\":\"table\",\"grid_color\":");
  write_rgba_array(fp, &table->grid_color);
  fprintf(fp, ",\"grid_size\":%d", table->grid_size);
  fprintf(fp, ",\"background_color\":");
  write_rgba_array(fp, &table->background_color);
  fprintf(fp, ",\"rows\":[");

  for (size_t r = 0; r < table->rows; r++) {
    if (r > 0)
      fprintf(fp, ",");
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

  fprintf(fp, "]}");
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

  JsonParser obj_parser = *parser;
  if (find_object_key(&obj_parser, "text") == 0) {
    parse_string_value(&obj_parser, &text->text);
  }

  obj_parser = *parser;
  if (find_object_key(&obj_parser, "level") == 0) {
    double level;
    if (parse_number_value(&obj_parser, &level) == 0) {
      text->level = (int)level;
    }
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
  if (find_object_key(&obj_parser, "color") == 0) {
    parse_rgba_array(&obj_parser, &text->color);
  }

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
      if (parse_element_text(&elem_parser, &text) == 0) {
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
    } else if (type_str && strcmp(type_str, "image") == 0) {
      elem_parser.pos = 0;
      ElementImage image;
      if (parse_element_image(&elem_parser, &image) == 0) {
        if (doc->elements_len >= doc->elements_capacity) {
          size_t new_cap =
              doc->elements_capacity == 0 ? 8 : doc->elements_capacity * 2;
          doc->elements = realloc(doc->elements, new_cap * sizeof(Element));
          doc->elements_capacity = new_cap;
        }

        doc->elements[doc->elements_len].kind = T_IMAGE;
        doc->elements[doc->elements_len].as.image = image;
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
