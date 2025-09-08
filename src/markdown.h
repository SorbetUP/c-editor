#pragma once
#include "editor.h"

int markdown_to_json(const char *markdown, Document *doc);
int json_to_markdown(const Document *doc, char **out_markdown);

typedef struct {
    const char *text;
    size_t pos;
    size_t len;
} MarkdownParser;

typedef enum {
    INLINE_NONE,
    INLINE_BOLD,
    INLINE_ITALIC, 
    INLINE_BOLD_ITALIC,
    INLINE_HIGHLIGHT,
    INLINE_UNDERLINE
} InlineStyle;

typedef struct {
    InlineStyle style;
    size_t start;
    size_t end;
} InlineSpan;

int parse_inline_styles(const char *text, InlineSpan *spans, size_t max_spans);
int parse_table_block(MarkdownParser *parser, ElementTable *table);
int parse_image_line(const char *line, ElementImage *image);
int parse_header_line(const char *line, ElementText *text);
bool is_table_separator_line(const char *line);
char **split_table_row(const char *line, int *col_count);
TextSpan *convert_spans_to_text_spans(const char *text, const InlineSpan *spans, size_t span_count, size_t *out_count);
