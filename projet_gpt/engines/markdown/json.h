#pragma once
#include "editor.h"

int json_write_document(const Document *doc, char **out_json);
int json_read_document(const char *json_str, Document *doc);

typedef struct {
  const char *json;
  size_t pos;
  size_t len;
} JsonParser;

typedef enum {
  JSON_NULL,
  JSON_BOOL,
  JSON_NUMBER,
  JSON_STRING,
  JSON_ARRAY,
  JSON_OBJECT
} JsonType;

typedef struct {
  JsonType type;
  const char *start;
  size_t len;
  size_t children_count;
  size_t child_offset;
} JsonToken;

int json_parse_tokens(const char *json, JsonToken *tokens, size_t max_tokens);
int json_find_key(const char *json, const JsonToken *tokens,
                  const JsonToken *parent, const char *key);
int json_parse_string(const char *json, const JsonToken *token, char **out);
int json_parse_number(const char *json, const JsonToken *token, double *out);
int json_parse_bool(const char *json, const JsonToken *token, bool *out);
RGBA json_parse_rgba_array(const char *json, const JsonToken *tokens,
                           const JsonToken *array_token);
