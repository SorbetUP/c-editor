#pragma once
#include <stddef.h>
#include <stdbool.h>

typedef struct {
    float r, g, b, a;
} RGBA;

typedef enum { 
    ALIGN_LEFT, 
    ALIGN_CENTER, 
    ALIGN_RIGHT, 
    ALIGN_JUSTIFY 
} Align;

typedef struct {
    char *text;
    bool bold;
    bool italic;
    bool has_highlight;
    RGBA highlight_color;
    bool has_underline;
    RGBA underline_color;
    int underline_gap;
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
} ElementTable;

typedef enum { 
    T_TEXT, 
    T_IMAGE, 
    T_TABLE 
} ElementKind;

typedef struct {
    ElementKind kind;
    union { 
        ElementText text; 
        ElementImage image; 
        ElementTable table; 
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

void doc_free(Document *doc);
