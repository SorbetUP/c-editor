#include "render_engine.h"
#include "common/render_software.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Platform detection
#ifdef __APPLE__
    #include "TargetConditionals.h"
    #if TARGET_OS_MAC && !TARGET_OS_IPHONE
        #ifndef PLATFORM_MACOS
            #define PLATFORM_MACOS
        #endif
    #elif TARGET_OS_IPHONE
        #define PLATFORM_IOS
    #endif
#elif defined(__ANDROID__)
    #define PLATFORM_ANDROID
#elif defined(__linux__)
    #define PLATFORM_LINUX
#elif defined(_WIN32) || defined(_WIN64)
    #define PLATFORM_WINDOWS
#endif

// Platform-specific implementations
#ifdef PLATFORM_MACOS
    #include "platforms/macos/render_macos.h"
#elif PLATFORM_IOS
    #include "platforms/ios/render_ios.h"
#elif PLATFORM_ANDROID
    #include "platforms/android/render_android.h"
#elif PLATFORM_LINUX
    #include "platforms/linux/render_linux.h"
#elif PLATFORM_WINDOWS
    #include "platforms/windows/render_windows.h"
#else
    #include "common/render_software.h"
#endif

// Global context management
static render_context_t* g_current_context = NULL;

// ============= CONTEXT MANAGEMENT =============

render_context_t* render_engine_create_context(render_backend_type_t backend, int width, int height) {
    render_context_t* ctx = calloc(1, sizeof(render_context_t));
    if (!ctx) return NULL;
    
    ctx->backend_type = backend;
    ctx->width = width;
    ctx->height = height;
    ctx->needs_layout = true;
    ctx->needs_repaint = true;
    
    // Initialize platform-specific backend
    int result = -1;
    
#ifdef PLATFORM_MACOS
    if (backend == RENDER_BACKEND_FRAMEBUFFER) {
        result = render_macos_init(ctx);
    }
#elif PLATFORM_IOS
    if (backend == RENDER_BACKEND_FRAMEBUFFER) {
        result = render_ios_init(ctx);
    }
#elif PLATFORM_ANDROID
    if (backend == RENDER_BACKEND_FRAMEBUFFER) {
        result = render_android_init(ctx);
    }
#elif PLATFORM_LINUX
    if (backend == RENDER_BACKEND_FRAMEBUFFER) {
        result = render_linux_init(ctx);
    }
#elif PLATFORM_WINDOWS
    if (backend == RENDER_BACKEND_FRAMEBUFFER) {
        result = render_windows_init(ctx);
    }
#endif
    
    // Fallback to software renderer
    if (result != 0) {
        result = render_software_init(ctx);
        if (result != 0) {
            free(ctx);
            return NULL;
        }
    }
    
    g_current_context = ctx;
    return ctx;
}

void render_engine_destroy_context(render_context_t* ctx) {
    if (!ctx) return;
    
    // Clean up root element
    if (ctx->root) {
        render_engine_destroy_element(ctx->root);
    }
    
    // Clean up stylesheets
    if (ctx->stylesheets) {
        for (int i = 0; i < ctx->stylesheet_count; i++) {
            free(ctx->stylesheets[i]);
        }
        free(ctx->stylesheets);
    }
    
    // Platform-specific cleanup
    if (ctx->backend_data) {
#ifdef PLATFORM_MACOS
        render_macos_destroy(ctx);
#elif PLATFORM_IOS
        render_ios_destroy(ctx);
#elif PLATFORM_ANDROID
        render_android_destroy(ctx);
#elif PLATFORM_LINUX
        render_linux_destroy(ctx);
#elif PLATFORM_WINDOWS
        render_windows_destroy(ctx);
#endif
    }
    
    if (g_current_context == ctx) {
        g_current_context = NULL;
    }
    
    free(ctx);
}

void render_engine_resize(render_context_t* ctx, int width, int height) {
    if (!ctx) return;
    
    ctx->width = width;
    ctx->height = height;
    ctx->needs_layout = true;
    ctx->needs_repaint = true;
    
    // Platform-specific resize
#ifdef PLATFORM_MACOS
    render_macos_resize(ctx, width, height);
#elif PLATFORM_IOS
    render_ios_resize(ctx, width, height);
#elif PLATFORM_ANDROID
    render_android_resize(ctx, width, height);
#elif PLATFORM_LINUX
    render_linux_resize(ctx, width, height);
#elif PLATFORM_WINDOWS
    render_windows_resize(ctx, width, height);
#endif
}

// ============= ELEMENT MANAGEMENT =============

render_element_t* render_engine_create_element(render_element_type_t type, const char* id) {
    render_element_t* element = calloc(1, sizeof(render_element_t));
    if (!element) return NULL;
    
    element->type = type;
    element->id = id ? strdup(id) : NULL;
    element->max_children = 16;
    element->children = malloc(element->max_children * sizeof(render_element_t*));
    element->needs_layout = true;
    
    // Initialize default style
    element->style.visible = true;
    element->style.opacity = 1.0f;
    element->style.background_color = (render_color_t){255, 255, 255, 255}; // White
    element->style.font.color = (render_color_t){0, 0, 0, 255}; // Black
    element->style.font.size = 14;
    element->style.font.family = strdup("system");
    
    return element;
}

void render_engine_destroy_element(render_element_t* element) {
    if (!element) return;
    
    // Clean up children
    if (element->children) {
        for (int i = 0; i < element->child_count; i++) {
            render_engine_destroy_element(element->children[i]);
        }
        free(element->children);
    }
    
    // Clean up strings
    free(element->id);
    free(element->class);
    free(element->text_content);
    free(element->style.font.family);
    
    // Element-specific cleanup
    switch (element->type) {
        case RENDER_ELEMENT_IMAGE:
            free(element->data.image.src);
            break;
        case RENDER_ELEMENT_LINK:
            free(element->data.link.href);
            free(element->data.link.target);
            break;
        case RENDER_ELEMENT_INPUT:
            free(element->data.input.value);
            free(element->data.input.placeholder);
            break;
        case RENDER_ELEMENT_CODE_BLOCK:
            free(element->data.code_block.language);
            break;
        default:
            break;
    }
    
    free(element);
}

void render_engine_add_child(render_element_t* parent, render_element_t* child) {
    if (!parent || !child) return;
    
    // Resize children array if needed
    if (parent->child_count >= parent->max_children) {
        parent->max_children *= 2;
        parent->children = realloc(parent->children, 
                                 parent->max_children * sizeof(render_element_t*));
        if (!parent->children) return;
    }
    
    parent->children[parent->child_count++] = child;
    child->parent = parent;
    parent->needs_layout = true;
}

void render_engine_remove_child(render_element_t* parent, render_element_t* child) {
    if (!parent || !child) return;
    
    for (int i = 0; i < parent->child_count; i++) {
        if (parent->children[i] == child) {
            // Shift remaining children
            for (int j = i; j < parent->child_count - 1; j++) {
                parent->children[j] = parent->children[j + 1];
            }
            parent->child_count--;
            child->parent = NULL;
            parent->needs_layout = true;
            break;
        }
    }
}

// ============= ELEMENT PROPERTIES =============

void render_engine_set_text(render_element_t* element, const char* text) {
    if (!element) return;
    
    free(element->text_content);
    element->text_content = text ? strdup(text) : NULL;
    element->needs_layout = true;
}

void render_engine_set_attribute(render_element_t* element, const char* attr, const char* value) {
    if (!element || !attr) return;
    
    if (strcmp(attr, "class") == 0) {
        free(element->class);
        element->class = value ? strdup(value) : NULL;
    } else if (strcmp(attr, "src") == 0 && element->type == RENDER_ELEMENT_IMAGE) {
        free(element->data.image.src);
        element->data.image.src = value ? strdup(value) : NULL;
    } else if (strcmp(attr, "href") == 0 && element->type == RENDER_ELEMENT_LINK) {
        free(element->data.link.href);
        element->data.link.href = value ? strdup(value) : NULL;
    } else if (strcmp(attr, "value") == 0 && element->type == RENDER_ELEMENT_INPUT) {
        free(element->data.input.value);
        element->data.input.value = value ? strdup(value) : NULL;
    }
    
    element->needs_layout = true;
}

// ============= STYLING =============

render_color_t render_engine_parse_color(const char* color_str) {
    render_color_t color = {0, 0, 0, 255}; // Default to black
    
    if (!color_str) return color;
    
    // Simple color parsing
    if (strcmp(color_str, "white") == 0) {
        color = (render_color_t){255, 255, 255, 255};
    } else if (strcmp(color_str, "black") == 0) {
        color = (render_color_t){0, 0, 0, 255};
    } else if (strcmp(color_str, "red") == 0) {
        color = (render_color_t){255, 0, 0, 255};
    } else if (strcmp(color_str, "green") == 0) {
        color = (render_color_t){0, 255, 0, 255};
    } else if (strcmp(color_str, "blue") == 0) {
        color = (render_color_t){0, 0, 255, 255};
    } else if (color_str[0] == '#') {
        // Parse hex color
        unsigned int hex;
        if (sscanf(color_str + 1, "%x", &hex) == 1) {
            if (strlen(color_str) == 7) { // #RRGGBB
                color.r = (hex >> 16) & 0xFF;
                color.g = (hex >> 8) & 0xFF;
                color.b = hex & 0xFF;
                color.a = 255;
            }
        }
    }
    
    return color;
}

render_font_t render_engine_create_font(const char* family, int size, bool bold, bool italic) {
    render_font_t font = {0};
    font.family = family ? strdup(family) : strdup("system");
    font.size = size > 0 ? size : 14;
    font.bold = bold;
    font.italic = italic;
    font.color = (render_color_t){0, 0, 0, 255}; // Black
    return font;
}

// ============= BASIC LAYOUT ENGINE =============

static void compute_element_layout(render_element_t* element, render_rect_t available_rect) {
    if (!element || !element->style.visible) return;
    
    // Simple block layout for now
    element->computed_rect = available_rect;
    
    // Apply margins and padding
    element->computed_rect.x += element->style.margin.x;
    element->computed_rect.y += element->style.margin.y;
    element->computed_rect.width -= element->style.margin.width + element->style.padding.width;
    element->computed_rect.height -= element->style.margin.height + element->style.padding.height;
    
    // Layout children
    if (element->child_count > 0) {
        render_rect_t child_rect = element->computed_rect;
        child_rect.x += element->style.padding.x;
        child_rect.y += element->style.padding.y;
        
        int current_y = child_rect.y;
        
        for (int i = 0; i < element->child_count; i++) {
            render_element_t* child = element->children[i];
            if (!child->style.visible) continue;
            
            render_rect_t child_available = child_rect;
            child_available.y = current_y;
            child_available.height = child_rect.height - (current_y - child_rect.y);
            
            compute_element_layout(child, child_available);
            
            // For block layout, stack vertically
            current_y = child->computed_rect.y + child->computed_rect.height + child->style.margin.height;
        }
    }
    
    element->needs_layout = false;
}

void render_engine_compute_layout(render_context_t* ctx) {
    if (!ctx || !ctx->root) return;
    
    render_rect_t viewport = {0, 0, ctx->width, ctx->height};
    compute_element_layout(ctx->root, viewport);
    ctx->needs_layout = false;
}

// ============= RENDERING =============

static void render_element_recursive(render_context_t* ctx, render_element_t* element) {
    if (!ctx || !element || !element->style.visible) return;
    
    // Draw background
    if (element->style.background_color.a > 0) {
        if (ctx->draw_rect) {
            ctx->draw_rect(ctx, element->computed_rect, element->style.background_color);
        }
    }
    
    // Draw content based on element type
    switch (element->type) {
        case RENDER_ELEMENT_TEXT:
            if (element->text_content && ctx->draw_text) {
                render_point_t pos = {element->computed_rect.x, element->computed_rect.y};
                ctx->draw_text(ctx, element->text_content, pos, element->style.font);
            }
            break;
            
        case RENDER_ELEMENT_BOX:
            // Background already drawn
            break;
            
        case RENDER_ELEMENT_IMAGE:
            if (element->data.image.src && ctx->draw_image) {
                ctx->draw_image(ctx, element->data.image.src, element->computed_rect);
            }
            break;
            
        default:
            // Handle other element types
            break;
    }
    
    // Draw border
    if (element->style.border.width > 0) {
        // TODO: Implement border drawing
    }
    
    // Render children
    for (int i = 0; i < element->child_count; i++) {
        render_element_recursive(ctx, element->children[i]);
    }
}

void render_engine_render(render_context_t* ctx) {
    if (!ctx) return;
    
    // Compute layout if needed
    if (ctx->needs_layout) {
        render_engine_compute_layout(ctx);
    }
    
    // Clear screen
    if (ctx->clear_screen) {
        render_color_t bg = {240, 240, 240, 255}; // Light gray
        ctx->clear_screen(ctx, bg);
    }
    
    // Render from root
    if (ctx->root) {
        render_element_recursive(ctx, ctx->root);
    }
    
    // Present to screen
    if (ctx->present) {
        ctx->present(ctx);
    }
    
    ctx->needs_repaint = false;
}

// ============= UTILITY FUNCTIONS =============

void render_engine_print_tree(render_element_t* element, int depth) {
    if (!element) return;
    
    for (int i = 0; i < depth; i++) printf("  ");
    
    printf("Element[%d]: type=%d, visible=%s, rect=(%d,%d,%d,%d)\n",
           depth,
           element->type,
           element->style.visible ? "true" : "false",
           element->computed_rect.x,
           element->computed_rect.y,
           element->computed_rect.width,
           element->computed_rect.height);
    
    for (int i = 0; i < element->child_count; i++) {
        render_engine_print_tree(element->children[i], depth + 1);
    }
}

// ============= STUB IMPLEMENTATIONS =============

// These will be implemented in platform-specific files
render_element_t* render_engine_parse_html(const char* html) {
    (void)html;
    return NULL; // TODO: Implement HTML parsing
}

render_element_t* render_engine_parse_markdown(const char* markdown) {
    (void)markdown;
    return NULL; // TODO: Implement Markdown parsing
}

css_stylesheet_t* render_engine_parse_css(const char* css) {
    (void)css;
    return NULL; // TODO: Implement CSS parsing
}

void render_engine_apply_stylesheet(render_context_t* ctx, css_stylesheet_t* stylesheet) {
    (void)ctx; (void)stylesheet;
    // TODO: Implement CSS application
}

void render_engine_compute_styles(render_context_t* ctx) {
    (void)ctx;
    // TODO: Implement style computation
}

void render_engine_handle_click(render_context_t* ctx, render_point_t point) {
    (void)ctx; (void)point;
    // TODO: Implement click handling
}

render_element_t* render_engine_element_at_point(render_context_t* ctx, render_point_t point) {
    (void)ctx; (void)point;
    return NULL; // TODO: Implement hit testing
}