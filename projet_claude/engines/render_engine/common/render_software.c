#include "render_software.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>

// Simple 8x8 monospace font bitmap (ASCII 32-127)
// Each character is 8 bytes, each byte represents a row
const uint8_t render_software_font_8x8[256][8] = {
    // Space (32)
    [32] = {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // ! (33)
    [33] = {0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00},
    // " (34)
    [34] = {0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
    // # (35)
    [35] = {0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00},
    // A (65)
    [65] = {0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00},
    // B (66)
    [66] = {0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00},
    // C (67)
    [67] = {0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00},
    // Add more characters as needed...
    // For brevity, we'll just add a few common ones
};

int render_software_init(render_context_t* ctx) {
    if (!ctx) return -1;
    
    render_software_backend_t* backend = calloc(1, sizeof(render_software_backend_t));
    if (!backend) return -1;
    
    backend->width = ctx->width;
    backend->height = ctx->height;
    backend->pitch = ctx->width * sizeof(uint32_t);
    backend->double_buffered = true;
    
    // Allocate framebuffer
    size_t buffer_size = backend->width * backend->height * sizeof(uint32_t);
    backend->framebuffer = malloc(buffer_size);
    if (!backend->framebuffer) {
        free(backend);
        return -1;
    }
    
    // Allocate back buffer for double buffering
    if (backend->double_buffered) {
        backend->back_buffer = malloc(buffer_size);
        if (!backend->back_buffer) {
            free(backend->framebuffer);
            free(backend);
            return -1;
        }
    }
    
    // Initialize clipping rectangle to full screen
    backend->clip_rect = (render_rect_t){0, 0, backend->width, backend->height};
    
    // Set default drawing state
    backend->current_color = (render_color_t){255, 255, 255, 255}; // White
    backend->current_font = (render_font_t){
        .family = "monospace",
        .size = 8,
        .bold = false,
        .italic = false,
        .color = {0, 0, 0, 255} // Black
    };
    
    // Initialize font cache
    backend->font_cache.char_width = 8;
    backend->font_cache.char_height = 8;
    backend->font_cache.cache_valid = false;
    memset(backend->font_cache.bitmap_cache, 0, sizeof(backend->font_cache.bitmap_cache));
    
    // Clear both buffers
    render_software_clear_buffer(backend, backend->framebuffer, 
                               (render_color_t){240, 240, 240, 255}); // Light gray
    if (backend->back_buffer) {
        render_software_clear_buffer(backend, backend->back_buffer, 
                                   (render_color_t){240, 240, 240, 255});
    }
    
    // Setup context drawing functions
    ctx->backend_data = backend;
    ctx->clear_screen = render_software_clear_screen;
    ctx->draw_rect = render_software_draw_rect;
    ctx->draw_text = render_software_draw_text;
    ctx->draw_image = render_software_draw_image;
    ctx->present = render_software_present;
    
    printf("Software render backend initialized: %dx%d\n", ctx->width, ctx->height);
    
    return 0;
}

void render_software_destroy(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    // Clear font cache
    render_software_clear_font_cache(backend);
    
    // Free buffers
    free(backend->framebuffer);
    free(backend->back_buffer);
    free(backend);
    
    ctx->backend_data = NULL;
    
    printf("Software render backend destroyed\n");
}

void render_software_resize(render_context_t* ctx, int width, int height) {
    if (!ctx || !ctx->backend_data) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    // Free old buffers
    free(backend->framebuffer);
    free(backend->back_buffer);
    
    // Update dimensions
    backend->width = width;
    backend->height = height;
    backend->pitch = width * sizeof(uint32_t);
    
    // Allocate new buffers
    size_t buffer_size = width * height * sizeof(uint32_t);
    backend->framebuffer = malloc(buffer_size);
    if (backend->double_buffered) {
        backend->back_buffer = malloc(buffer_size);
    }
    
    // Update clip rectangle
    backend->clip_rect = (render_rect_t){0, 0, width, height};
    
    printf("Software render backend resized: %dx%d\n", width, height);
}

void render_software_clear_screen(render_context_t* ctx, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    uint32_t* target = backend->double_buffered ? backend->back_buffer : backend->framebuffer;
    
    render_software_clear_buffer(backend, target, color);
}

void render_software_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    // Clip rectangle to screen bounds
    rect = render_software_clip_rect_to_bounds(backend, rect);
    
    uint32_t pixel_color = render_software_color_to_argb(color);
    uint32_t* target = backend->double_buffered ? backend->back_buffer : backend->framebuffer;
    
    for (int y = rect.y; y < rect.y + rect.height; y++) {
        for (int x = rect.x; x < rect.x + rect.width; x++) {
            if (!render_software_is_point_clipped(backend, x, y)) {
                target[y * backend->width + x] = pixel_color;
            }
        }
    }
}

void render_software_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font) {
    if (!ctx || !ctx->backend_data || !text) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    // Update font cache if needed
    if (!backend->font_cache.cache_valid || 
        backend->current_font.size != font.size ||
        strcmp(backend->current_font.family, font.family) != 0) {
        backend->current_font = font;
        render_software_generate_font_cache(backend, font);
    }
    
    int x = pos.x;
    int y = pos.y;
    
    for (const char* c = text; *c; c++) {
        render_software_draw_char(backend, *c, (render_point_t){x, y}, font);
        x += backend->font_cache.char_width;
        
        // Simple line wrapping
        if (x >= backend->width - backend->font_cache.char_width) {
            x = pos.x;
            y += backend->font_cache.char_height;
        }
    }
}

void render_software_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect) {
    if (!ctx || !ctx->backend_data || !image_path) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    uint32_t* image_data = NULL;
    int img_width, img_height;
    
    if (render_software_load_image_data(image_path, &image_data, &img_width, &img_height) == 0) {
        render_software_draw_image_data(backend, image_data, img_width, img_height, rect);
        render_software_free_image_data(image_data);
    } else {
        // Draw placeholder rectangle for missing images
        render_color_t placeholder_color = {200, 200, 200, 255}; // Light gray
        render_software_draw_rect_with_alpha(backend, rect, placeholder_color);
        
        // Draw X pattern to indicate missing image
        render_color_t x_color = {100, 100, 100, 255}; // Dark gray
        render_software_draw_line(backend, 
            (render_point_t){rect.x, rect.y}, 
            (render_point_t){rect.x + rect.width, rect.y + rect.height}, x_color);
        render_software_draw_line(backend, 
            (render_point_t){rect.x + rect.width, rect.y}, 
            (render_point_t){rect.x, rect.y + rect.height}, x_color);
    }
}

void render_software_present(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_software_backend_t* backend = (render_software_backend_t*)ctx->backend_data;
    
    if (backend->double_buffered) {
        render_software_swap_buffers(backend);
    }
    
    // In a real implementation, this would copy the framebuffer to the screen
    // For now, the framebuffer is ready for external display
}

void render_software_set_pixel(render_software_backend_t* backend, int x, int y, render_color_t color) {
    if (!backend || render_software_is_point_clipped(backend, x, y)) return;
    
    uint32_t* target = backend->double_buffered ? backend->back_buffer : backend->framebuffer;
    uint32_t pixel_color = render_software_color_to_argb(color);
    
    if (color.a == 255) {
        // Opaque pixel
        target[y * backend->width + x] = pixel_color;
    } else {
        // Alpha blending required
        uint32_t existing = target[y * backend->width + x];
        target[y * backend->width + x] = render_software_blend_pixels(pixel_color, existing, color.a);
    }
}

render_color_t render_software_get_pixel(render_software_backend_t* backend, int x, int y) {
    if (!backend || x < 0 || x >= backend->width || y < 0 || y >= backend->height) {
        return (render_color_t){0, 0, 0, 0};
    }
    
    uint32_t* target = backend->double_buffered ? backend->back_buffer : backend->framebuffer;
    uint32_t argb = target[y * backend->width + x];
    return render_software_argb_to_color(argb);
}

uint32_t render_software_color_to_argb(render_color_t color) {
    return (color.a << 24) | (color.r << 16) | (color.g << 8) | color.b;
}

render_color_t render_software_argb_to_color(uint32_t argb) {
    return (render_color_t){
        .a = (argb >> 24) & 0xFF,
        .r = (argb >> 16) & 0xFF,
        .g = (argb >> 8) & 0xFF,
        .b = argb & 0xFF
    };
}

void render_software_draw_line(render_software_backend_t* backend, 
                              render_point_t start, render_point_t end, render_color_t color) {
    // Bresenham's line algorithm
    int dx = abs(end.x - start.x);
    int dy = abs(end.y - start.y);
    int sx = start.x < end.x ? 1 : -1;
    int sy = start.y < end.y ? 1 : -1;
    int err = dx - dy;
    
    int x = start.x;
    int y = start.y;
    
    while (1) {
        render_software_set_pixel(backend, x, y, color);
        
        if (x == end.x && y == end.y) break;
        
        int e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            x += sx;
        }
        if (e2 < dx) {
            err += dx;
            y += sy;
        }
    }
}

void render_software_draw_char(render_software_backend_t* backend, 
                              char c, render_point_t pos, render_font_t font) {
    if (c < 32 || c > 126) c = '?'; // Replace non-printable with question mark
    
    const uint8_t* char_bitmap = render_software_font_8x8[(unsigned char)c];
    
    for (int y = 0; y < 8; y++) {
        uint8_t row = char_bitmap[y];
        for (int x = 0; x < 8; x++) {
            if (row & (0x80 >> x)) {
                render_software_set_pixel(backend, pos.x + x, pos.y + y, font.color);
            }
        }
    }
}

void render_software_generate_font_cache(render_software_backend_t* backend, render_font_t font) {
    // For simplicity, we'll just use the built-in 8x8 font
    // In a real implementation, this would rasterize the specified font
    backend->font_cache.char_width = 8;
    backend->font_cache.char_height = 8;
    backend->font_cache.cache_valid = true;
}

void render_software_clear_font_cache(render_software_backend_t* backend) {
    for (int i = 0; i < 256; i++) {
        free(backend->font_cache.bitmap_cache[i]);
        backend->font_cache.bitmap_cache[i] = NULL;
    }
    backend->font_cache.cache_valid = false;
}

int render_software_load_image_data(const char* image_path, uint32_t** image_data, 
                                   int* width, int* height) {
    // Placeholder implementation
    // In a real implementation, this would use a library like stb_image
    (void)image_path;
    (void)image_data;
    (void)width;
    (void)height;
    
    printf("Image loading not implemented: %s\n", image_path);
    return -1;
}

void render_software_free_image_data(uint32_t* image_data) {
    free(image_data);
}

void render_software_draw_image_data(render_software_backend_t* backend, 
                                    uint32_t* image_data, int img_width, int img_height,
                                    render_rect_t dest_rect) {
    if (!backend || !image_data) return;
    
    // Simple nearest-neighbor scaling
    float scale_x = (float)img_width / dest_rect.width;
    float scale_y = (float)img_height / dest_rect.height;
    
    for (int y = 0; y < dest_rect.height; y++) {
        for (int x = 0; x < dest_rect.width; x++) {
            int src_x = (int)(x * scale_x);
            int src_y = (int)(y * scale_y);
            
            if (src_x >= 0 && src_x < img_width && src_y >= 0 && src_y < img_height) {
                uint32_t pixel = image_data[src_y * img_width + src_x];
                render_color_t color = render_software_argb_to_color(pixel);
                render_software_set_pixel(backend, dest_rect.x + x, dest_rect.y + y, color);
            }
        }
    }
}

uint32_t render_software_blend_pixels(uint32_t src, uint32_t dst, uint8_t alpha) {
    if (alpha == 0) return dst;
    if (alpha == 255) return src;
    
    uint8_t src_a = (src >> 24) & 0xFF;
    uint8_t src_r = (src >> 16) & 0xFF;
    uint8_t src_g = (src >> 8) & 0xFF;
    uint8_t src_b = src & 0xFF;
    
    uint8_t dst_a = (dst >> 24) & 0xFF;
    uint8_t dst_r = (dst >> 16) & 0xFF;
    uint8_t dst_g = (dst >> 8) & 0xFF;
    uint8_t dst_b = dst & 0xFF;
    
    // Simple alpha blending
    uint8_t inv_alpha = 255 - alpha;
    uint8_t out_a = dst_a + ((src_a * alpha) >> 8);
    uint8_t out_r = ((dst_r * inv_alpha) + (src_r * alpha)) >> 8;
    uint8_t out_g = ((dst_g * inv_alpha) + (src_g * alpha)) >> 8;
    uint8_t out_b = ((dst_b * inv_alpha) + (src_b * alpha)) >> 8;
    
    return (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
}

void render_software_draw_rect_with_alpha(render_software_backend_t* backend, 
                                         render_rect_t rect, render_color_t color) {
    rect = render_software_clip_rect_to_bounds(backend, rect);
    
    for (int y = rect.y; y < rect.y + rect.height; y++) {
        for (int x = rect.x; x < rect.x + rect.width; x++) {
            render_software_set_pixel(backend, x, y, color);
        }
    }
}

void render_software_set_clip_rect(render_software_backend_t* backend, render_rect_t rect) {
    if (!backend) return;
    
    // Ensure clip rect is within screen bounds
    rect.x = rect.x < 0 ? 0 : rect.x;
    rect.y = rect.y < 0 ? 0 : rect.y;
    rect.width = rect.x + rect.width > backend->width ? backend->width - rect.x : rect.width;
    rect.height = rect.y + rect.height > backend->height ? backend->height - rect.y : rect.height;
    
    backend->clip_rect = rect;
}

bool render_software_is_point_clipped(render_software_backend_t* backend, int x, int y) {
    if (!backend) return true;
    
    return (x < backend->clip_rect.x || x >= backend->clip_rect.x + backend->clip_rect.width ||
            y < backend->clip_rect.y || y >= backend->clip_rect.y + backend->clip_rect.height);
}

render_rect_t render_software_clip_rect_to_bounds(render_software_backend_t* backend, 
                                                 render_rect_t rect) {
    if (!backend) return rect;
    
    // Clip to screen bounds
    if (rect.x < 0) {
        rect.width += rect.x;
        rect.x = 0;
    }
    if (rect.y < 0) {
        rect.height += rect.y;
        rect.y = 0;
    }
    if (rect.x + rect.width > backend->width) {
        rect.width = backend->width - rect.x;
    }
    if (rect.y + rect.height > backend->height) {
        rect.height = backend->height - rect.y;
    }
    
    // Ensure non-negative dimensions
    rect.width = rect.width < 0 ? 0 : rect.width;
    rect.height = rect.height < 0 ? 0 : rect.height;
    
    return rect;
}

void render_software_swap_buffers(render_software_backend_t* backend) {
    if (!backend || !backend->double_buffered) return;
    
    // Copy back buffer to front buffer
    memcpy(backend->framebuffer, backend->back_buffer, 
           backend->width * backend->height * sizeof(uint32_t));
}

void render_software_clear_buffer(render_software_backend_t* backend, uint32_t* buffer, 
                                 render_color_t color) {
    if (!backend || !buffer) return;
    
    uint32_t pixel_color = render_software_color_to_argb(color);
    size_t pixel_count = backend->width * backend->height;
    
    for (size_t i = 0; i < pixel_count; i++) {
        buffer[i] = pixel_color;
    }
}

void render_software_fill_circle(render_software_backend_t* backend, 
                                render_point_t center, int radius, render_color_t color) {
    int x = 0;
    int y = radius;
    int d = 1 - radius;
    
    while (x <= y) {
        // Draw horizontal lines for filled circle
        for (int i = center.x - x; i <= center.x + x; i++) {
            render_software_set_pixel(backend, i, center.y + y, color);
            render_software_set_pixel(backend, i, center.y - y, color);
        }
        for (int i = center.x - y; i <= center.x + y; i++) {
            render_software_set_pixel(backend, i, center.y + x, color);
            render_software_set_pixel(backend, i, center.y - x, color);
        }
        
        if (d < 0) {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y--;
        }
        x++;
    }
}