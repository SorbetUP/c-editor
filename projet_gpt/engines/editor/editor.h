#pragma once
#include <stdbool.h>
#include <stddef.h>

typedef struct {
  float r, g, b, a;
} RGBA;

typedef enum { ALIGN_LEFT, ALIGN_CENTER, ALIGN_RIGHT, ALIGN_JUSTIFY } Align;

typedef struct {
  char *text;
  bool bold;
  bool italic;
  bool has_highlight;
  RGBA highlight_color;
  bool has_underline;
  RGBA underline_color;
  int underline_gap;
  bool code;
  bool strikethrough;
  bool is_link;
  char *link_href;
  bool is_note_link;
  bool is_image;
  char *image_src;
  char *image_alt;
} TextSpan;

typedef struct {
  char *text;
  char *font;
  Align align;
  int font_size;
  RGBA color;
  bool bold, italic;
  bool has_underline, has_highlight;
  RGBA underline_color;
  int underline_gap;
  RGBA highlight_color;
  int level;
  TextSpan *spans;
  size_t spans_count;
} ElementText;

typedef struct {
  char *language;
  char *content;
  bool fenced;
} ElementCode;

typedef enum {
  LIST_KIND_BULLET = 0,
  LIST_KIND_ORDERED = 1,
  LIST_KIND_TASK = 2,
  LIST_KIND_DEFINITION = 3
} ListKind;

typedef struct {
  ElementText text;
  bool has_checkbox;
  bool checkbox_checked;
  bool is_task;
  bool is_definition;
  ElementText term;
  ElementText definition;
  int indent_level;
  int number;
} ElementListItem;

typedef struct {
  bool ordered;
  int start_index;
  ListKind kind;
  ElementListItem *items;
  size_t item_count;
  size_t item_capacity;
} ElementList;

typedef struct {
  ElementText *items;
  size_t item_count;
  size_t item_capacity;
} ElementQuote;

typedef struct {
  int thickness;
  RGBA color;
} ElementDivider;

typedef struct {
  char *name;
  char *value;
} ElementSettings;

typedef struct {
  char *src;
  char *alt;
  Align align;
  int width, height;
  float alpha;
} ElementImage;

typedef struct {
  size_t rows, cols;
  ElementText ***cells;
  RGBA grid_color, background_color;
  int grid_size;
  Align *column_align;
  bool *column_align_defined;
  size_t column_align_count;
  size_t header_rows;
  
  // Column width specifications for consistent line-by-line rendering
  int *column_widths;           // Calculated optimal widths in characters
  int *column_min_widths;       // Minimum required widths
  int *column_max_widths;       // Maximum allowed widths
  bool widths_calculated;       // Whether column widths have been computed
  int total_content_width;      // Total width needed for all content
} ElementTable;

typedef enum {
  T_TEXT,
  T_IMAGE,
  T_TABLE,
  T_CODE,
  T_LIST,
  T_QUOTE,
  T_DIVIDER,
  T_SETTINGS
} ElementKind;

typedef struct {
  ElementKind kind;
  union {
    ElementText text;
    ElementImage image;
    ElementTable table;
    ElementCode code;
    ElementList list;
    ElementQuote quote;
    ElementDivider divider;
    ElementSettings settings;
  } as;
} Element;

typedef struct {
  char *name;
  char *default_font;
  int default_fontsize;
  RGBA default_text_color;
  RGBA default_highlight_color;
  RGBA default_underline_color;
  int default_underline_gap;
  long created, updated;
  Element *elements;
  size_t elements_len;
  size_t elements_capacity;

  char *current_line;
  size_t current_line_len;
  size_t current_line_capacity;
} Document;

void editor_init(Document *doc);
void editor_feed_char(Document *doc, unsigned codepoint);
void editor_commit_line(Document *doc);

int json_export_markdown(const Document *doc, char **out_md);
int json_import_markdown(const char *md, Document *out_doc);

int json_stringify(const Document *doc, char **out_json);
int json_parse(const char *json, Document *out_doc);

// Table column width calculation functions
int table_calculate_column_widths(ElementTable *table);
int table_get_cell_content_width(const ElementText *cell);
void table_apply_width_constraints(ElementTable *table, int max_total_width);

void doc_free(Document *doc);
