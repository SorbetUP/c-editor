#include "render_macos.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#ifdef __APPLE__
#include <TargetConditionals.h>
#if TARGET_OS_MAC && !TARGET_OS_IPHONE

#include <CoreText/CoreText.h>
#include <ImageIO/ImageIO.h>

int render_macos_init(render_context_t* ctx) {
    if (!ctx) return -1;
    
    render_macos_backend_t* backend = calloc(1, sizeof(render_macos_backend_t));
    if (!backend) return -1;
    
    // Create color space
    backend->color_space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    if (!backend->color_space) {
        free(backend);
        return -1;
    }
    
    // Setup bitmap dimensions
    backend->bitmap_width = ctx->width;
    backend->bitmap_height = ctx->height;
    backend->bytes_per_row = backend->bitmap_width * 4; // RGBA
    
    // Allocate bitmap data
    size_t data_size = backend->bytes_per_row * backend->bitmap_height;
    backend->bitmap_data = malloc(data_size);
    if (!backend->bitmap_data) {
        CGColorSpaceRelease(backend->color_space);
        free(backend);
        return -1;
    }
    
    // Clear bitmap to white
    memset(backend->bitmap_data, 255, data_size);
    
    // Create CGContext
    backend->cg_context = CGBitmapContextCreate(
        backend->bitmap_data,
        backend->bitmap_width,
        backend->bitmap_height,
        8,                          // bits per component
        backend->bytes_per_row,
        backend->color_space,
        kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big
    );
    
    if (!backend->cg_context) {
        free(backend->bitmap_data);
        CGColorSpaceRelease(backend->color_space);
        free(backend);
        return -1;
    }
    
    backend->owns_context = true;
    
    // Initialize drawing state
    backend->transform = CGAffineTransformIdentity;
    backend->clip_rect = CGRectMake(0, 0, ctx->width, ctx->height);
    backend->line_width = 1.0f;
    
    // Set default colors
    CGFloat black[] = {0.0f, 0.0f, 0.0f, 1.0f};
    CGFloat white[] = {1.0f, 1.0f, 1.0f, 1.0f};
    backend->stroke_color = CGColorCreate(backend->color_space, black);
    backend->fill_color = CGColorCreate(backend->color_space, white);
    backend->font_color = CGColorCreate(backend->color_space, black);
    
    // Set default font
    render_font_t default_font = {
        .family = "system",
        .size = 14,
        .bold = false,
        .italic = false,
        .color = {0, 0, 0, 255}
    };
    render_macos_set_font(backend, default_font);
    
    // Setup context drawing functions
    ctx->backend_data = backend;
    ctx->clear_screen = render_macos_clear_screen;
    ctx->draw_rect = render_macos_draw_rect;
    ctx->draw_text = render_macos_draw_text;
    ctx->draw_image = render_macos_draw_image;
    ctx->present = render_macos_present;
    
    return 0;
}

void render_macos_destroy(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    // Release Core Graphics objects
    if (backend->cg_context) {
        CGContextRelease(backend->cg_context);
    }
    if (backend->current_image) {
        CGImageRelease(backend->current_image);
    }
    if (backend->data_provider) {
        CGDataProviderRelease(backend->data_provider);
    }
    if (backend->color_space) {
        CGColorSpaceRelease(backend->color_space);
    }
    if (backend->current_font) {
        CFRelease(backend->current_font);
    }
    if (backend->stroke_color) {
        CGColorRelease(backend->stroke_color);
    }
    if (backend->fill_color) {
        CGColorRelease(backend->fill_color);
    }
    if (backend->font_color) {
        CGColorRelease(backend->font_color);
    }
    
    // Free bitmap data
    free(backend->bitmap_data);
    free(backend);
    
    ctx->backend_data = NULL;
}

void render_macos_resize(render_context_t* ctx, int width, int height) {
    if (!ctx || !ctx->backend_data) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    // Release old context and bitmap
    if (backend->cg_context) {
        CGContextRelease(backend->cg_context);
    }
    free(backend->bitmap_data);
    
    // Update dimensions
    backend->bitmap_width = width;
    backend->bitmap_height = height;
    backend->bytes_per_row = width * 4;
    
    // Allocate new bitmap
    size_t data_size = backend->bytes_per_row * backend->bitmap_height;
    backend->bitmap_data = malloc(data_size);
    if (!backend->bitmap_data) return;
    
    memset(backend->bitmap_data, 255, data_size);
    
    // Create new context
    backend->cg_context = CGBitmapContextCreate(
        backend->bitmap_data,
        backend->bitmap_width,
        backend->bitmap_height,
        8,
        backend->bytes_per_row,
        backend->color_space,
        kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big
    );
    
    backend->clip_rect = CGRectMake(0, 0, width, height);
}

void render_macos_clear_screen(render_context_t* ctx, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    CGColorRef cg_color = render_macos_color_from_render_color(color);
    CGContextSetFillColorWithColor(backend->cg_context, cg_color);
    CGContextFillRect(backend->cg_context, backend->clip_rect);
    CGColorRelease(cg_color);
}

void render_macos_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    CGRect cg_rect = CGRectMake(rect.x, rect.y, rect.width, rect.height);
    CGColorRef cg_color = render_macos_color_from_render_color(color);
    
    CGContextSetFillColorWithColor(backend->cg_context, cg_color);
    CGContextFillRect(backend->cg_context, cg_rect);
    
    CGColorRelease(cg_color);
}

void render_macos_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font) {
    if (!ctx || !ctx->backend_data || !text) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    // Set font if different from current
    render_macos_set_font(backend, font);
    
    // Create attributed string
    CFStringRef cf_string = CFStringCreateWithCString(kCFAllocatorDefault, text, kCFStringEncodingUTF8);
    if (!cf_string) return;
    
    // Create attributes dictionary
    CGColorRef text_color = render_macos_color_from_render_color(font.color);
    
    CFStringRef keys[] = { kCTFontAttributeName, kCTForegroundColorAttributeName };
    CFTypeRef values[] = { backend->current_font, text_color };
    
    CFDictionaryRef attributes = CFDictionaryCreate(
        kCFAllocatorDefault,
        (const void**)keys,
        (const void**)values,
        2,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks
    );
    
    CFAttributedStringRef attributed_string = CFAttributedStringCreate(
        kCFAllocatorDefault,
        cf_string,
        attributes
    );
    
    // Create line and draw
    CTLineRef line = CTLineCreateWithAttributedString(attributed_string);
    
    CGContextSetTextPosition(backend->cg_context, pos.x, backend->bitmap_height - pos.y - font.size);
    CTLineDraw(line, backend->cg_context);
    
    // Cleanup
    CFRelease(line);
    CFRelease(attributed_string);
    CFRelease(attributes);
    CFRelease(cf_string);
    CGColorRelease(text_color);
}

void render_macos_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect) {
    if (!ctx || !ctx->backend_data || !image_path) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    // Create URL from path
    CFStringRef path_string = CFStringCreateWithCString(kCFAllocatorDefault, image_path, kCFStringEncodingUTF8);
    CFURLRef image_url = CFURLCreateWithFileSystemPath(kCFAllocatorDefault, path_string, kCFURLPOSIXPathStyle, false);
    
    if (!image_url) {
        CFRelease(path_string);
        return;
    }
    
    // Create image source and image
    CGImageSourceRef image_source = CGImageSourceCreateWithURL(image_url, NULL);
    if (!image_source) {
        CFRelease(image_url);
        CFRelease(path_string);
        return;
    }
    
    CGImageRef image = CGImageSourceCreateImageAtIndex(image_source, 0, NULL);
    if (!image) {
        CFRelease(image_source);
        CFRelease(image_url);
        CFRelease(path_string);
        return;
    }
    
    // Draw image
    CGRect cg_rect = CGRectMake(rect.x, backend->bitmap_height - rect.y - rect.height, rect.width, rect.height);
    CGContextDrawImage(backend->cg_context, cg_rect, image);
    
    // Cleanup
    CGImageRelease(image);
    CFRelease(image_source);
    CFRelease(image_url);
    CFRelease(path_string);
}

void render_macos_present(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_macos_backend_t* backend = (render_macos_backend_t*)ctx->backend_data;
    
    // Create image from context
    if (backend->current_image) {
        CGImageRelease(backend->current_image);
    }
    
    backend->current_image = CGBitmapContextCreateImage(backend->cg_context);
    
    // In a real implementation, this would present to a window or view
    // For now, we just maintain the image for potential export/display
}

CGColorRef render_macos_color_from_render_color(render_color_t color) {
    CGFloat components[4] = {
        color.r / 255.0f,
        color.g / 255.0f,
        color.b / 255.0f,
        color.a / 255.0f
    };
    
    CGColorSpaceRef color_space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGColorRef cg_color = CGColorCreate(color_space, components);
    CGColorSpaceRelease(color_space);
    
    return cg_color;
}

CTFontRef render_macos_font_from_render_font(render_font_t font) {
    CFStringRef font_name;
    
    // Map font family names
    if (strcmp(font.family, "system") == 0) {
        font_name = CFSTR(".AppleSystemUIFont");
    } else if (strcmp(font.family, "monospace") == 0) {
        font_name = CFSTR("Menlo");
    } else if (strcmp(font.family, "serif") == 0) {
        font_name = CFSTR("Times New Roman");
    } else {
        font_name = CFStringCreateWithCString(kCFAllocatorDefault, font.family, kCFStringEncodingUTF8);
    }
    
    CTFontRef base_font = CTFontCreateWithName(font_name, font.size, NULL);
    
    if (font_name != CFSTR(".AppleSystemUIFont") && 
        font_name != CFSTR("Menlo") && 
        font_name != CFSTR("Times New Roman")) {
        CFRelease(font_name);
    }
    
    // Apply bold/italic traits
    if (font.bold || font.italic) {
        CTFontSymbolicTraits traits = 0;
        if (font.bold) traits |= kCTFontTraitBold;
        if (font.italic) traits |= kCTFontTraitItalic;
        
        CTFontRef styled_font = CTFontCreateCopyWithSymbolicTraits(base_font, font.size, NULL, traits, traits);
        if (styled_font) {
            CFRelease(base_font);
            return styled_font;
        }
    }
    
    return base_font;
}

void render_macos_set_font(render_macos_backend_t* backend, render_font_t font) {
    if (!backend) return;
    
    // Only update if font changed
    if (backend->current_font && backend->font_size == font.size) {
        return; // Simplified check - in real implementation, compare full font properties
    }
    
    if (backend->current_font) {
        CFRelease(backend->current_font);
    }
    if (backend->font_color) {
        CGColorRelease(backend->font_color);
    }
    
    backend->current_font = render_macos_font_from_render_font(font);
    backend->font_size = font.size;
    backend->font_color = render_macos_color_from_render_color(font.color);
}

#endif // TARGET_OS_MAC && !TARGET_OS_IPHONE
#endif // __APPLE__