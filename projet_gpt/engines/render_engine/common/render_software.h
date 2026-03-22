#ifndef RENDER_SOFTWARE_H
#define RENDER_SOFTWARE_H

#include "../render_engine.h"
#include <stdint.h>

typedef struct {
    uint32_t* framebuffer;
    int width;
    int height;
    int pitch; // bytes per row
    
    // Current drawing state
    render_color_t current_color;
    render_font_t current_font;
    
    // Software font rendering
    struct {
        uint8_t* bitmap_cache[256]; // ASCII character cache
        int char_width;
        int char_height;
        bool cache_valid;
    } font_cache;
    
    // Clipping rectangle
    render_rect_t clip_rect;
    
    // Double buffering
    uint32_t* back_buffer;
    bool double_buffered;
} render_software_backend_t;

// Platform-independent software renderer
int render_software_init(render_context_t* ctx);
void render_software_destroy(render_context_t* ctx);
void render_software_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_software_clear_screen(render_context_t* ctx, render_color_t color);
void render_software_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_software_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_software_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_software_present(render_context_t* ctx);

// Low-level pixel operations
void render_software_set_pixel(render_software_backend_t* backend, int x, int y, render_color_t color);
render_color_t render_software_get_pixel(render_software_backend_t* backend, int x, int y);
uint32_t render_software_color_to_argb(render_color_t color);
render_color_t render_software_argb_to_color(uint32_t argb);

// Primitive drawing functions
void render_software_draw_line(render_software_backend_t* backend, 
                              render_point_t start, render_point_t end, render_color_t color);
void render_software_draw_circle(render_software_backend_t* backend, 
                                render_point_t center, int radius, render_color_t color);
void render_software_fill_circle(render_software_backend_t* backend, 
                                render_point_t center, int radius, render_color_t color);

// Text rendering with software font rasterization
void render_software_draw_char(render_software_backend_t* backend, 
                              char c, render_point_t pos, render_font_t font);
void render_software_generate_font_cache(render_software_backend_t* backend, render_font_t font);
void render_software_clear_font_cache(render_software_backend_t* backend);

// Image loading and manipulation
int render_software_load_image_data(const char* image_path, uint32_t** image_data, 
                                   int* width, int* height);
void render_software_free_image_data(uint32_t* image_data);
void render_software_draw_image_data(render_software_backend_t* backend, 
                                    uint32_t* image_data, int img_width, int img_height,
                                    render_rect_t dest_rect);

// Blending and alpha operations
uint32_t render_software_blend_pixels(uint32_t src, uint32_t dst, uint8_t alpha);
void render_software_draw_rect_with_alpha(render_software_backend_t* backend, 
                                         render_rect_t rect, render_color_t color);

// Clipping support
void render_software_set_clip_rect(render_software_backend_t* backend, render_rect_t rect);
bool render_software_is_point_clipped(render_software_backend_t* backend, int x, int y);
render_rect_t render_software_clip_rect_to_bounds(render_software_backend_t* backend, 
                                                 render_rect_t rect);

// Buffer operations
void render_software_swap_buffers(render_software_backend_t* backend);
void render_software_clear_buffer(render_software_backend_t* backend, uint32_t* buffer, 
                                 render_color_t color);

// Simple built-in fonts
extern const uint8_t render_software_font_8x8[256][8];
extern const uint8_t render_software_font_16x16[256][32];

#endif // RENDER_SOFTWARE_H