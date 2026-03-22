#ifndef RENDER_ENGINE_H
#define RENDER_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

// Forward declarations
typedef struct render_context render_context_t;
typedef struct render_element render_element_t;
typedef struct render_style render_style_t;

// Color structure (RGBA)
typedef struct {
    uint8_t r, g, b, a;
} render_color_t;

// Point structure
typedef struct {
    int x, y;
} render_point_t;

// Rectangle structure
typedef struct {
    int x, y, width, height;
} render_rect_t;

// Font structure
typedef struct {
    char* family;
    int size;
    bool bold;
    bool italic;
    render_color_t color;
} render_font_t;

// Element types
typedef enum {
    RENDER_ELEMENT_TEXT,
    RENDER_ELEMENT_BOX,
    RENDER_ELEMENT_IMAGE,
    RENDER_ELEMENT_LINE,
    RENDER_ELEMENT_BUTTON,
    RENDER_ELEMENT_INPUT,
    RENDER_ELEMENT_LIST,
    RENDER_ELEMENT_TABLE,
    RENDER_ELEMENT_LINK,
    RENDER_ELEMENT_CODE_BLOCK
} render_element_type_t;

// Layout types
typedef enum {
    RENDER_LAYOUT_BLOCK,
    RENDER_LAYOUT_INLINE,
    RENDER_LAYOUT_FLEX,
    RENDER_LAYOUT_GRID,
    RENDER_LAYOUT_ABSOLUTE
} render_layout_type_t;

// Text alignment
typedef enum {
    RENDER_ALIGN_LEFT,
    RENDER_ALIGN_CENTER,
    RENDER_ALIGN_RIGHT,
    RENDER_ALIGN_JUSTIFY
} render_align_t;

// Style structure (CSS-like but simplified)
typedef struct render_style {
    // Box model
    render_rect_t margin;
    render_rect_t padding;
    render_rect_t border;
    
    // Colors and appearance
    render_color_t background_color;
    render_color_t border_color;
    int border_radius;
    
    // Typography
    render_font_t font;
    render_align_t text_align;
    int line_height;
    
    // Layout
    render_layout_type_t layout;
    int width, height;
    bool visible;
    float opacity;
    
    // Positioning
    render_point_t position;
    bool position_absolute;
} render_style_t;

// Element structure (DOM-like but simplified)
typedef struct render_element {
    render_element_type_t type;
    char* id;
    char* class;
    char* text_content;
    
    // Style
    render_style_t style;
    render_style_t computed_style; // After CSS cascade
    
    // Tree structure
    struct render_element* parent;
    struct render_element** children;
    int child_count;
    int max_children;
    
    // Element-specific data
    union {
        struct {
            char* src;
            int width, height;
        } image;
        
        struct {
            char* href;
            char* target;
        } link;
        
        struct {
            char* value;
            char* placeholder;
            bool readonly;
        } input;
        
        struct {
            char* language;
            bool syntax_highlighting;
        } code_block;
    } data;
    
    // Event handlers (simplified)
    void (*on_click)(struct render_element* element);
    void (*on_hover)(struct render_element* element);
    
    // Layout data
    render_rect_t computed_rect;
    bool needs_layout;
} render_element_t;

// Rendering backends
typedef enum {
    RENDER_BACKEND_TERMINAL,  // Terminal/ASCII rendering
    RENDER_BACKEND_FRAMEBUFFER, // Direct framebuffer
    RENDER_BACKEND_CAIRO,     // Cairo graphics (optional)
    RENDER_BACKEND_SOFTWARE   // Software renderer
} render_backend_type_t;

// Render context (like a browser context)
typedef struct render_context {
    render_backend_type_t backend_type;
    
    // Canvas/screen dimensions
    int width, height;
    
    // Root element (like HTML root)
    render_element_t* root;
    
    // Style sheets (simplified CSS)
    char** stylesheets;
    int stylesheet_count;
    
    // Backend-specific data
    void* backend_data;
    
    // Rendering functions (backend-specific)
    void (*draw_text)(struct render_context* ctx, const char* text, render_point_t pos, render_font_t font);
    void (*draw_rect)(struct render_context* ctx, render_rect_t rect, render_color_t color);
    void (*draw_line)(struct render_context* ctx, render_point_t start, render_point_t end, render_color_t color);
    void (*draw_image)(struct render_context* ctx, const char* src, render_rect_t rect);
    void (*clear_screen)(struct render_context* ctx, render_color_t color);
    void (*present)(struct render_context* ctx);
    
    // Layout engine
    bool needs_layout;
    bool needs_repaint;
} render_context_t;

// Simple CSS parser structures
typedef struct {
    char* selector;
    render_style_t* properties;
    int property_count;
} css_rule_t;

typedef struct {
    css_rule_t* rules;
    int rule_count;
} css_stylesheet_t;

// API Functions

// Context management
render_context_t* render_engine_create_context(render_backend_type_t backend, int width, int height);
void render_engine_destroy_context(render_context_t* ctx);
void render_engine_resize(render_context_t* ctx, int width, int height);

// Element creation and management
render_element_t* render_engine_create_element(render_element_type_t type, const char* id);
void render_engine_destroy_element(render_element_t* element);
void render_engine_add_child(render_element_t* parent, render_element_t* child);
void render_engine_remove_child(render_element_t* parent, render_element_t* child);

// Element properties
void render_engine_set_text(render_element_t* element, const char* text);
void render_engine_set_attribute(render_element_t* element, const char* attr, const char* value);
void render_engine_set_style_property(render_element_t* element, const char* property, const char* value);

// HTML-like parsing (simplified)
render_element_t* render_engine_parse_html(const char* html);
render_element_t* render_engine_parse_markdown(const char* markdown);

// CSS parsing and styling
css_stylesheet_t* render_engine_parse_css(const char* css);
void render_engine_apply_stylesheet(render_context_t* ctx, css_stylesheet_t* stylesheet);
void render_engine_compute_styles(render_context_t* ctx);

// Layout engine
void render_engine_compute_layout(render_context_t* ctx);
void render_engine_invalidate_layout(render_element_t* element);

// Rendering
void render_engine_render(render_context_t* ctx);
void render_engine_render_element(render_context_t* ctx, render_element_t* element);

// Event handling
void render_engine_handle_click(render_context_t* ctx, render_point_t point);
void render_engine_handle_hover(render_context_t* ctx, render_point_t point);
render_element_t* render_engine_element_at_point(render_context_t* ctx, render_point_t point);

// Utility functions
render_color_t render_engine_parse_color(const char* color_str);
render_font_t render_engine_create_font(const char* family, int size, bool bold, bool italic);
char* render_engine_element_to_html(render_element_t* element);

// Backend-specific functions
int render_engine_init_terminal_backend(render_context_t* ctx);
int render_engine_init_framebuffer_backend(render_context_t* ctx);
int render_engine_init_software_backend(render_context_t* ctx);

// Performance and debugging
void render_engine_print_tree(render_element_t* element, int depth);
void render_engine_get_stats(render_context_t* ctx, int* element_count, int* draw_calls);
void render_engine_benchmark(render_context_t* ctx, int iterations);

#endif // RENDER_ENGINE_H