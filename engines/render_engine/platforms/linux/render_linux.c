#include "render_linux.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#ifdef __linux__

int render_linux_init(render_context_t* ctx) {
    if (!ctx) return -1;
    
    render_linux_backend_t* backend = calloc(1, sizeof(render_linux_backend_t));
    if (!backend) return -1;
    
    backend->width = ctx->width;
    backend->height = ctx->height;
    backend->has_transparency = false;
    backend->events_enabled = false;
    
    // Set default colors
    backend->current_red = 0.0;
    backend->current_green = 0.0;
    backend->current_blue = 0.0;
    backend->current_alpha = 1.0;
    
    // Set default font
    backend->current_font_family = strdup("Sans");
    backend->current_font_size = 12.0;
    backend->font_bold = false;
    backend->font_italic = false;
    
    // Detect and initialize best available backend
    backend->backend_type = render_linux_detect_best_backend();
    
    int result = -1;
    switch (backend->backend_type) {
        case RENDER_LINUX_BACKEND_X11:
            result = render_linux_init_x11(backend, ctx->width, ctx->height);
            break;
#ifdef WAYLAND_SUPPORT
        case RENDER_LINUX_BACKEND_WAYLAND:
            result = render_linux_init_wayland(backend, ctx->width, ctx->height);
            break;
#endif
        case RENDER_LINUX_BACKEND_SOFTWARE:
        default:
            // Fallback to software rendering using Cairo image surface
            backend->cairo_surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, 
                                                               ctx->width, ctx->height);
            backend->cairo_context = cairo_create(backend->cairo_surface);
            result = (cairo_status(backend->cairo_context) == CAIRO_STATUS_SUCCESS) ? 0 : -1;
            break;
    }
    
    if (result != 0) {
        free(backend->current_font_family);
        free(backend);
        return -1;
    }
    
    // Initialize Pango layout
    backend->pango_layout = pango_cairo_create_layout(backend->cairo_context);
    
    // Setup context drawing functions
    ctx->backend_data = backend;
    ctx->clear_screen = render_linux_clear_screen;
    ctx->draw_rect = render_linux_draw_rect;
    ctx->draw_text = render_linux_draw_text;
    ctx->draw_image = render_linux_draw_image;
    ctx->present = render_linux_present;
    
    printf("Linux render backend initialized: %s (%dx%d)\n", 
           backend->backend_type == RENDER_LINUX_BACKEND_X11 ? "X11" :
           backend->backend_type == RENDER_LINUX_BACKEND_WAYLAND ? "Wayland" : "Software",
           ctx->width, ctx->height);
    
    return 0;
}

void render_linux_destroy(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    // Cleanup Pango
    if (backend->pango_layout) {
        g_object_unref(backend->pango_layout);
    }
    if (backend->current_font_desc) {
        pango_font_description_free(backend->current_font_desc);
    }
    
    // Cleanup Cairo
    if (backend->cairo_context) {
        cairo_destroy(backend->cairo_context);
    }
    if (backend->cairo_surface) {
        cairo_surface_destroy(backend->cairo_surface);
    }
    
    // Backend-specific cleanup
    switch (backend->backend_type) {
        case RENDER_LINUX_BACKEND_X11:
            render_linux_cleanup_x11(backend);
            break;
#ifdef WAYLAND_SUPPORT
        case RENDER_LINUX_BACKEND_WAYLAND:
            render_linux_cleanup_wayland(backend);
            break;
#endif
        default:
            break;
    }
    
    free(backend->current_font_family);
    free(backend);
    ctx->backend_data = NULL;
    
    printf("Linux render backend destroyed\n");
}

void render_linux_resize(render_context_t* ctx, int width, int height) {
    if (!ctx || !ctx->backend_data) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    backend->width = width;
    backend->height = height;
    
    // Recreate Cairo surface with new dimensions
    if (backend->cairo_context) {
        cairo_destroy(backend->cairo_context);
    }
    if (backend->cairo_surface) {
        cairo_surface_destroy(backend->cairo_surface);
    }
    
    switch (backend->backend_type) {
        case RENDER_LINUX_BACKEND_X11:
            // Resize X11 pixmap
            if (backend->x11_pixmap) {
                XFreePixmap(backend->x11_display, backend->x11_pixmap);
            }
            backend->x11_pixmap = XCreatePixmap(backend->x11_display, backend->x11_window,
                                               width, height, DefaultDepth(backend->x11_display, backend->x11_screen));
            backend->cairo_surface = cairo_xlib_surface_create(backend->x11_display, backend->x11_pixmap,
                                                              backend->x11_visual, width, height);
            break;
        default:
            backend->cairo_surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, width, height);
            break;
    }
    
    backend->cairo_context = cairo_create(backend->cairo_surface);
    
    // Update Pango layout
    if (backend->pango_layout) {
        g_object_unref(backend->pango_layout);
    }
    backend->pango_layout = pango_cairo_create_layout(backend->cairo_context);
    
    printf("Linux render backend resized: %dx%d\n", width, height);
}

void render_linux_clear_screen(render_context_t* ctx, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    render_linux_set_cairo_color(backend->cairo_context, color);
    cairo_paint(backend->cairo_context);
}

void render_linux_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    render_linux_set_cairo_color(backend->cairo_context, color);
    cairo_rectangle(backend->cairo_context, rect.x, rect.y, rect.width, rect.height);
    cairo_fill(backend->cairo_context);
}

void render_linux_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font) {
    if (!ctx || !ctx->backend_data || !text) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    render_linux_draw_text_with_pango(backend, text, pos, font);
}

void render_linux_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect) {
    if (!ctx || !ctx->backend_data || !image_path) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    cairo_surface_t* image_surface = render_linux_load_image_surface(image_path);
    if (!image_surface) {
        printf("Failed to load image: %s\n", image_path);
        return;
    }
    
    // Scale and position the image
    cairo_save(backend->cairo_context);
    cairo_translate(backend->cairo_context, rect.x, rect.y);
    
    double image_width = cairo_image_surface_get_width(image_surface);
    double image_height = cairo_image_surface_get_height(image_surface);
    
    double scale_x = rect.width / image_width;
    double scale_y = rect.height / image_height;
    
    cairo_scale(backend->cairo_context, scale_x, scale_y);
    cairo_set_source_surface(backend->cairo_context, image_surface, 0, 0);
    cairo_paint(backend->cairo_context);
    cairo_restore(backend->cairo_context);
    
    cairo_surface_destroy(image_surface);
}

void render_linux_present(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_linux_backend_t* backend = (render_linux_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_LINUX_BACKEND_X11:
            // Copy from pixmap to window
            if (backend->x11_window && backend->x11_pixmap) {
                XCopyArea(backend->x11_display, backend->x11_pixmap, backend->x11_window,
                         backend->x11_gc, 0, 0, backend->width, backend->height, 0, 0);
                XFlush(backend->x11_display);
            }
            break;
#ifdef WAYLAND_SUPPORT
        case RENDER_LINUX_BACKEND_WAYLAND:
            // Swap EGL buffers for Wayland
            if (backend->egl_display && backend->egl_surface) {
                eglSwapBuffers(backend->egl_display, backend->egl_surface);
            }
            break;
#endif
        default:
            // For software rendering, the surface is ready for export/display
            break;
    }
    
    // Process any pending events
    if (backend->events_enabled) {
        render_linux_process_events(backend);
    }
}

int render_linux_init_x11(render_linux_backend_t* backend, int width, int height) {
    backend->x11_display = XOpenDisplay(NULL);
    if (!backend->x11_display) {
        printf("Cannot open X11 display\n");
        return -1;
    }
    
    backend->x11_screen = DefaultScreen(backend->x11_display);
    backend->x11_visual = DefaultVisual(backend->x11_display, backend->x11_screen);
    backend->x11_colormap = DefaultColormap(backend->x11_display, backend->x11_screen);
    
    // Create window (for testing purposes)
    backend->x11_window = XCreateSimpleWindow(
        backend->x11_display,
        RootWindow(backend->x11_display, backend->x11_screen),
        0, 0, width, height, 1,
        BlackPixel(backend->x11_display, backend->x11_screen),
        WhitePixel(backend->x11_display, backend->x11_screen)
    );
    
    // Create pixmap for off-screen rendering
    backend->x11_pixmap = XCreatePixmap(backend->x11_display, backend->x11_window,
                                       width, height, DefaultDepth(backend->x11_display, backend->x11_screen));
    
    // Create graphics context
    backend->x11_gc = XCreateGC(backend->x11_display, backend->x11_window, 0, NULL);
    
    // Create Cairo surface
    backend->cairo_surface = cairo_xlib_surface_create(backend->x11_display, backend->x11_pixmap,
                                                      backend->x11_visual, width, height);
    backend->cairo_context = cairo_create(backend->cairo_surface);
    
    if (cairo_status(backend->cairo_context) != CAIRO_STATUS_SUCCESS) {
        render_linux_cleanup_x11(backend);
        return -1;
    }
    
    return 0;
}

void render_linux_cleanup_x11(render_linux_backend_t* backend) {
    if (backend->x11_gc) {
        XFreeGC(backend->x11_display, backend->x11_gc);
    }
    if (backend->x11_pixmap) {
        XFreePixmap(backend->x11_display, backend->x11_pixmap);
    }
    if (backend->x11_window) {
        XDestroyWindow(backend->x11_display, backend->x11_window);
    }
    if (backend->x11_display) {
        XCloseDisplay(backend->x11_display);
    }
}

#ifdef WAYLAND_SUPPORT
int render_linux_init_wayland(render_linux_backend_t* backend, int width, int height) {
    // Wayland initialization - simplified implementation
    backend->wl_display = wl_display_connect(NULL);
    if (!backend->wl_display) {
        printf("Cannot connect to Wayland display\n");
        return -1;
    }
    
    // In a real implementation, this would involve:
    // - Registry setup and global object binding
    // - Surface creation
    // - EGL context setup
    // - Buffer management
    
    printf("Wayland support not fully implemented\n");
    return -1;
}

void render_linux_cleanup_wayland(render_linux_backend_t* backend) {
    if (backend->wl_display) {
        wl_display_disconnect(backend->wl_display);
    }
}
#endif

void render_linux_set_cairo_color(cairo_t* cr, render_color_t color) {
    cairo_set_source_rgba(cr, 
                         color.r / 255.0,
                         color.g / 255.0,
                         color.b / 255.0,
                         color.a / 255.0);
}

cairo_surface_t* render_linux_load_image_surface(const char* image_path) {
    GError* error = NULL;
    GdkPixbuf* pixbuf = gdk_pixbuf_new_from_file(image_path, &error);
    
    if (!pixbuf) {
        if (error) {
            printf("Error loading image: %s\n", error->message);
            g_error_free(error);
        }
        return NULL;
    }
    
    int width = gdk_pixbuf_get_width(pixbuf);
    int height = gdk_pixbuf_get_height(pixbuf);
    int stride = cairo_format_stride_for_width(CAIRO_FORMAT_ARGB32, width);
    
    cairo_surface_t* surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, width, height);
    unsigned char* data = cairo_image_surface_get_data(surface);
    
    // Convert GdkPixbuf to Cairo surface format
    unsigned char* pixbuf_data = gdk_pixbuf_get_pixels(pixbuf);
    int pixbuf_stride = gdk_pixbuf_get_rowstride(pixbuf);
    int channels = gdk_pixbuf_get_n_channels(pixbuf);
    
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            unsigned char* src = pixbuf_data + y * pixbuf_stride + x * channels;
            unsigned char* dst = data + y * stride + x * 4;
            
            if (channels == 4) {
                // RGBA -> ARGB (premultiplied)
                unsigned char a = src[3];
                dst[0] = (src[2] * a) / 255; // B
                dst[1] = (src[1] * a) / 255; // G
                dst[2] = (src[0] * a) / 255; // R
                dst[3] = a;                   // A
            } else {
                // RGB -> ARGB
                dst[0] = src[2]; // B
                dst[1] = src[1]; // G
                dst[2] = src[0]; // R
                dst[3] = 255;    // A
            }
        }
    }
    
    cairo_surface_mark_dirty(surface);
    g_object_unref(pixbuf);
    
    return surface;
}

void render_linux_draw_text_with_pango(render_linux_backend_t* backend, 
                                       const char* text, 
                                       render_point_t pos, 
                                       render_font_t font) {
    render_linux_set_cairo_color(backend->cairo_context, font.color);
    
    // Update font if needed
    if (backend->current_font_desc) {
        pango_font_description_free(backend->current_font_desc);
    }
    backend->current_font_desc = render_linux_create_font_description(font);
    
    pango_layout_set_font_description(backend->pango_layout, backend->current_font_desc);
    pango_layout_set_text(backend->pango_layout, text, -1);
    
    cairo_move_to(backend->cairo_context, pos.x, pos.y);
    pango_cairo_show_layout(backend->cairo_context, backend->pango_layout);
}

PangoFontDescription* render_linux_create_font_description(render_font_t font) {
    PangoFontDescription* desc = pango_font_description_new();
    
    pango_font_description_set_family(desc, font.family);
    pango_font_description_set_size(desc, font.size * PANGO_SCALE);
    
    PangoWeight weight = font.bold ? PANGO_WEIGHT_BOLD : PANGO_WEIGHT_NORMAL;
    pango_font_description_set_weight(desc, weight);
    
    PangoStyle style = font.italic ? PANGO_STYLE_ITALIC : PANGO_STYLE_NORMAL;
    pango_font_description_set_style(desc, style);
    
    return desc;
}

render_linux_backend_type_t render_linux_detect_best_backend(void) {
    // Check for Wayland first
    if (getenv("WAYLAND_DISPLAY") && render_linux_is_wayland_available()) {
        return RENDER_LINUX_BACKEND_WAYLAND;
    }
    
    // Fall back to X11
    if (getenv("DISPLAY") && render_linux_is_x11_available()) {
        return RENDER_LINUX_BACKEND_X11;
    }
    
    // Software fallback
    return RENDER_LINUX_BACKEND_SOFTWARE;
}

int render_linux_is_wayland_available(void) {
#ifdef WAYLAND_SUPPORT
    struct wl_display* display = wl_display_connect(NULL);
    if (display) {
        wl_display_disconnect(display);
        return 1;
    }
#endif
    return 0;
}

int render_linux_is_x11_available(void) {
    Display* display = XOpenDisplay(NULL);
    if (display) {
        XCloseDisplay(display);
        return 1;
    }
    return 0;
}

void render_linux_handle_x11_events(render_linux_backend_t* backend) {
    if (!backend->x11_display) return;
    
    while (XPending(backend->x11_display)) {
        XEvent event;
        XNextEvent(backend->x11_display, &event);
        
        // Handle events and call callback if registered
        if (backend->event_callback) {
            backend->event_callback(event.type, &event);
        }
    }
}

void render_linux_enable_events(render_linux_backend_t* backend, void (*callback)(int, void*)) {
    backend->events_enabled = true;
    backend->event_callback = callback;
}

void render_linux_process_events(render_linux_backend_t* backend) {
    switch (backend->backend_type) {
        case RENDER_LINUX_BACKEND_X11:
            render_linux_handle_x11_events(backend);
            break;
#ifdef WAYLAND_SUPPORT
        case RENDER_LINUX_BACKEND_WAYLAND:
            render_linux_handle_wayland_events(backend);
            break;
#endif
        default:
            break;
    }
}

#ifdef WAYLAND_SUPPORT
void render_linux_handle_wayland_events(render_linux_backend_t* backend) {
    if (!backend->wl_display) return;
    
    wl_display_dispatch_pending(backend->wl_display);
}
#endif

#endif // __linux__