#ifndef RENDER_WINDOWS_H
#define RENDER_WINDOWS_H

#include "../../render_engine.h"

#if defined(_WIN32) || defined(_WIN64)

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <wingdi.h>
#include <d2d1.h>
#include <d2d1helper.h>
#include <dwrite.h>
#include <wincodec.h>

// DirectX support
#ifdef DIRECTX_SUPPORT
#include <d3d11.h>
#include <dxgi.h>
#include <d3dcompiler.h>
#endif

#ifndef SAFE_RELEASE
#define SAFE_RELEASE(p) { if(p) { (p)->Release(); (p) = NULL; } }
#endif

typedef enum {
    RENDER_WINDOWS_BACKEND_GDI,
    RENDER_WINDOWS_BACKEND_DIRECT2D,
    RENDER_WINDOWS_BACKEND_DIRECTX
} render_windows_backend_type_t;

typedef struct {
    render_windows_backend_type_t backend_type;
    
    // Common Windows members
    HWND hwnd;
    HDC hdc;
    HBITMAP hbitmap;
    HBITMAP old_bitmap;
    void* bitmap_bits;
    BITMAPINFO bitmap_info;
    
    // GDI-specific members
    HPEN current_pen;
    HBRUSH current_brush;
    HFONT current_font;
    COLORREF current_color;
    
    // Direct2D-specific members
    ID2D1Factory* d2d_factory;
    ID2D1RenderTarget* d2d_render_target;
    ID2D1HwndRenderTarget* d2d_hwnd_render_target;
    ID2D1BitmapRenderTarget* d2d_bitmap_render_target;
    ID2D1SolidColorBrush* d2d_brush;
    
    // DirectWrite for text rendering
    IDWriteFactory* dwrite_factory;
    IDWriteTextFormat* dwrite_text_format;
    
    // WIC for image loading
    IWICImagingFactory* wic_factory;
    
    // DirectX-specific members (optional)
#ifdef DIRECTX_SUPPORT
    ID3D11Device* d3d_device;
    ID3D11DeviceContext* d3d_context;
    IDXGISwapChain* dxgi_swap_chain;
    ID3D11RenderTargetView* d3d_render_target_view;
    ID3D11VertexShader* vertex_shader;
    ID3D11PixelShader* pixel_shader;
    ID3D11InputLayout* input_layout;
    ID3D11Buffer* vertex_buffer;
    ID3D11Buffer* constant_buffer;
#endif
    
    // Common rendering state
    int width;
    int height;
    bool vsync_enabled;
    
    // Font management
    WCHAR* current_font_family;
    FLOAT current_font_size;
    DWRITE_FONT_WEIGHT font_weight;
    DWRITE_FONT_STYLE font_style;
    
    // Color management
    D2D1_COLOR_F current_d2d_color;
    
    // Event handling
    bool events_enabled;
    WNDPROC original_wndproc;
    void (*event_callback)(UINT message, WPARAM wparam, LPARAM lparam);
} render_windows_backend_t;

// Platform-specific functions
int render_windows_init(render_context_t* ctx);
void render_windows_destroy(render_context_t* ctx);
void render_windows_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_windows_clear_screen(render_context_t* ctx, render_color_t color);
void render_windows_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_windows_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_windows_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_windows_present(render_context_t* ctx);

// Backend-specific initialization
int render_windows_init_gdi(render_windows_backend_t* backend, int width, int height);
int render_windows_init_direct2d(render_windows_backend_t* backend, int width, int height);
#ifdef DIRECTX_SUPPORT
int render_windows_init_directx(render_windows_backend_t* backend, int width, int height);
#endif

// Backend-specific cleanup
void render_windows_cleanup_gdi(render_windows_backend_t* backend);
void render_windows_cleanup_direct2d(render_windows_backend_t* backend);
#ifdef DIRECTX_SUPPORT
void render_windows_cleanup_directx(render_windows_backend_t* backend);
#endif

// GDI utility functions
COLORREF render_windows_colorref_from_render_color(render_color_t color);
HFONT render_windows_create_gdi_font(render_font_t font);
void render_windows_set_gdi_color(render_windows_backend_t* backend, render_color_t color);

// Direct2D utility functions
D2D1_COLOR_F render_windows_d2d_color_from_render_color(render_color_t color);
D2D1_RECT_F render_windows_d2d_rect_from_render_rect(render_rect_t rect);
HRESULT render_windows_create_dwrite_text_format(render_windows_backend_t* backend, render_font_t font);
HRESULT render_windows_load_image_d2d(render_windows_backend_t* backend, const WCHAR* image_path, ID2D1Bitmap** bitmap);

// DirectX utility functions (optional)
#ifdef DIRECTX_SUPPORT
HRESULT render_windows_compile_shader(LPCWSTR filename, LPCSTR entry_point, LPCSTR shader_model, ID3DBlob** blob);
HRESULT render_windows_create_vertex_buffer(render_windows_backend_t* backend);
#endif

// Window management
HWND render_windows_create_window(int width, int height, const WCHAR* title);
LRESULT CALLBACK render_windows_wndproc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam);

// Event handling
void render_windows_enable_events(render_windows_backend_t* backend, void (*callback)(UINT, WPARAM, LPARAM));
void render_windows_process_events(render_windows_backend_t* backend);

// Backend detection
render_windows_backend_type_t render_windows_detect_best_backend(void);
BOOL render_windows_is_direct2d_available(void);
BOOL render_windows_is_directx_available(void);

// Utility functions
WCHAR* render_windows_utf8_to_wide(const char* utf8_str);
char* render_windows_wide_to_utf8(const WCHAR* wide_str);

#endif // _WIN32 || _WIN64

#endif // RENDER_WINDOWS_H