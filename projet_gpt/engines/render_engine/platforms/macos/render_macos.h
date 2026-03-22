#ifndef RENDER_MACOS_H
#define RENDER_MACOS_H

#include "../../render_engine.h"

#ifdef __APPLE__
#include <TargetConditionals.h>
#if TARGET_OS_MAC && !TARGET_OS_IPHONE

#include <CoreGraphics/CoreGraphics.h>
#include <ApplicationServices/ApplicationServices.h>

typedef struct {
    CGContextRef cg_context;
    CGColorSpaceRef color_space;
    void* bitmap_data;
    size_t bitmap_width;
    size_t bitmap_height;
    size_t bytes_per_row;
    CGDataProviderRef data_provider;
    CGImageRef current_image;
    bool owns_context;
    
    // Font management
    CTFontRef current_font;
    CGFloat font_size;
    CGColorRef font_color;
    
    // Drawing state
    CGAffineTransform transform;
    CGRect clip_rect;
    CGFloat line_width;
    CGColorRef stroke_color;
    CGColorRef fill_color;
} render_macos_backend_t;

// Platform-specific functions
int render_macos_init(render_context_t* ctx);
void render_macos_destroy(render_context_t* ctx);
void render_macos_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_macos_clear_screen(render_context_t* ctx, render_color_t color);
void render_macos_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_macos_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_macos_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_macos_present(render_context_t* ctx);

// Utility functions
CGColorRef render_macos_color_from_render_color(render_color_t color);
CTFontRef render_macos_font_from_render_font(render_font_t font);
void render_macos_set_font(render_macos_backend_t* backend, render_font_t font);

#endif // TARGET_OS_MAC && !TARGET_OS_IPHONE
#endif // __APPLE__

#endif // RENDER_MACOS_H