#ifndef RENDER_IOS_H
#define RENDER_IOS_H

#include "../../render_engine.h"

#ifdef __APPLE__
#include <TargetConditionals.h>
#if TARGET_OS_IPHONE

#include <CoreGraphics/CoreGraphics.h>
#include <CoreText/CoreText.h>
#include <UIKit/UIKit.h>

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
    
    // iOS-specific references
    UIView* target_view;
    CALayer* target_layer;
    
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
    
    // Touch/gesture support
    bool touch_enabled;
    CGPoint last_touch_point;
    
    // Metal layer support (for advanced rendering)
    CAMetalLayer* metal_layer;
    bool use_metal;
} render_ios_backend_t;

// Platform-specific functions
int render_ios_init(render_context_t* ctx);
void render_ios_destroy(render_context_t* ctx);
void render_ios_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_ios_clear_screen(render_context_t* ctx, render_color_t color);
void render_ios_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_ios_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_ios_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_ios_present(render_context_t* ctx);

// iOS-specific functions
int render_ios_setup_view_target(render_ios_backend_t* backend, UIView* view);
int render_ios_setup_metal_layer(render_ios_backend_t* backend, CAMetalLayer* layer);
void render_ios_handle_touch(render_ios_backend_t* backend, CGPoint point, int touch_phase);

// Utility functions
CGColorRef render_ios_color_from_render_color(render_color_t color);
CTFontRef render_ios_font_from_render_font(render_font_t font);
void render_ios_set_font(render_ios_backend_t* backend, render_font_t font);
UIColor* render_ios_uicolor_from_render_color(render_color_t color);

// Auto Layout and constraints support
void render_ios_update_view_constraints(render_ios_backend_t* backend);
CGSize render_ios_measure_text(const char* text, render_font_t font);

#endif // TARGET_OS_IPHONE
#endif // __APPLE__

#endif // RENDER_IOS_H