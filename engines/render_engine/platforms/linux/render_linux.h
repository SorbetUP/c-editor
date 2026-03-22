#ifndef RENDER_LINUX_H
#define RENDER_LINUX_H

#include "../../render_engine.h"

#ifdef __linux__

#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/extensions/Xrender.h>
#include <cairo/cairo.h>
#include <cairo/cairo-xlib.h>
#include <pango/pangocairo.h>
#include <gdk-pixbuf/gdk-pixbuf.h>

// Wayland support
#ifdef WAYLAND_SUPPORT
#include <wayland-client.h>
#include <wayland-egl.h>
#include <EGL/egl.h>
#include <GLES2/gl2.h>
#endif

typedef enum {
    RENDER_LINUX_BACKEND_X11,
    RENDER_LINUX_BACKEND_WAYLAND,
    RENDER_LINUX_BACKEND_SOFTWARE
} render_linux_backend_type_t;

typedef struct {
    render_linux_backend_type_t backend_type;
    
    // X11-specific members
    Display* x11_display;
    Window x11_window;
    int x11_screen;
    Visual* x11_visual;
    Colormap x11_colormap;
    GC x11_gc;
    Pixmap x11_pixmap;
    
    // Cairo rendering context
    cairo_surface_t* cairo_surface;
    cairo_t* cairo_context;
    
    // Pango font rendering
    PangoLayout* pango_layout;
    PangoFontDescription* current_font_desc;
    
    // Wayland-specific members
#ifdef WAYLAND_SUPPORT
    struct wl_display* wl_display;
    struct wl_registry* wl_registry;
    struct wl_compositor* wl_compositor;
    struct wl_shell* wl_shell;
    struct wl_surface* wl_surface;
    struct wl_shell_surface* wl_shell_surface;
    struct wl_egl_window* wl_egl_window;
    EGLDisplay egl_display;
    EGLConfig egl_config;
    EGLContext egl_context;
    EGLSurface egl_surface;
#endif
    
    // Common members
    int width;
    int height;
    bool has_transparency;
    
    // Color management
    double current_red, current_green, current_blue, current_alpha;
    
    // Font management
    char* current_font_family;
    double current_font_size;
    bool font_bold;
    bool font_italic;
    
    // Event handling
    bool events_enabled;
    void (*event_callback)(int event_type, void* event_data);
} render_linux_backend_t;

// Platform-specific functions
int render_linux_init(render_context_t* ctx);
void render_linux_destroy(render_context_t* ctx);
void render_linux_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_linux_clear_screen(render_context_t* ctx, render_color_t color);
void render_linux_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_linux_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_linux_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_linux_present(render_context_t* ctx);

// X11-specific functions
int render_linux_init_x11(render_linux_backend_t* backend, int width, int height);
void render_linux_cleanup_x11(render_linux_backend_t* backend);
void render_linux_handle_x11_events(render_linux_backend_t* backend);

// Wayland-specific functions
#ifdef WAYLAND_SUPPORT
int render_linux_init_wayland(render_linux_backend_t* backend, int width, int height);
void render_linux_cleanup_wayland(render_linux_backend_t* backend);
void render_linux_handle_wayland_events(render_linux_backend_t* backend);
#endif

// Cairo utility functions
void render_linux_set_cairo_color(cairo_t* cr, render_color_t color);
void render_linux_set_cairo_font(render_linux_backend_t* backend, render_font_t font);
cairo_surface_t* render_linux_load_image_surface(const char* image_path);

// Pango text rendering
void render_linux_draw_text_with_pango(render_linux_backend_t* backend, 
                                       const char* text, 
                                       render_point_t pos, 
                                       render_font_t font);
PangoFontDescription* render_linux_create_font_description(render_font_t font);

// Event handling
void render_linux_enable_events(render_linux_backend_t* backend, void (*callback)(int, void*));
void render_linux_process_events(render_linux_backend_t* backend);

// Backend detection and selection
render_linux_backend_type_t render_linux_detect_best_backend(void);
int render_linux_is_wayland_available(void);
int render_linux_is_x11_available(void);

#endif // __linux__

#endif // RENDER_LINUX_H