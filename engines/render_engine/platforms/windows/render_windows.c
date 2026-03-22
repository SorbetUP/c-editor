#include "render_windows.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#if defined(_WIN32) || defined(_WIN64)

#pragma comment(lib, "d2d1.lib")
#pragma comment(lib, "dwrite.lib")
#pragma comment(lib, "windowscodecs.lib")
#pragma comment(lib, "gdi32.lib")
#pragma comment(lib, "user32.lib")

#ifdef DIRECTX_SUPPORT
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d3dcompiler.lib")
#endif

int render_windows_init(render_context_t* ctx) {
    if (!ctx) return -1;
    
    render_windows_backend_t* backend = calloc(1, sizeof(render_windows_backend_t));
    if (!backend) return -1;
    
    backend->width = ctx->width;
    backend->height = ctx->height;
    backend->vsync_enabled = true;
    backend->events_enabled = false;
    
    // Initialize COM
    HRESULT hr = CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE) {
        free(backend);
        return -1;
    }
    
    // Set default font
    backend->current_font_family = _wcsdup(L"Segoe UI");
    backend->current_font_size = 14.0f;
    backend->font_weight = DWRITE_FONT_WEIGHT_NORMAL;
    backend->font_style = DWRITE_FONT_STYLE_NORMAL;
    
    // Detect and initialize best available backend
    backend->backend_type = render_windows_detect_best_backend();
    
    int result = -1;
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            result = render_windows_init_direct2d(backend, ctx->width, ctx->height);
            break;
#ifdef DIRECTX_SUPPORT
        case RENDER_WINDOWS_BACKEND_DIRECTX:
            result = render_windows_init_directx(backend, ctx->width, ctx->height);
            break;
#endif
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            result = render_windows_init_gdi(backend, ctx->width, ctx->height);
            break;
    }
    
    if (result != 0) {
        free(backend->current_font_family);
        free(backend);
        return -1;
    }
    
    // Setup context drawing functions
    ctx->backend_data = backend;
    ctx->clear_screen = render_windows_clear_screen;
    ctx->draw_rect = render_windows_draw_rect;
    ctx->draw_text = render_windows_draw_text;
    ctx->draw_image = render_windows_draw_image;
    ctx->present = render_windows_present;
    
    printf("Windows render backend initialized: %s (%dx%d)\n", 
           backend->backend_type == RENDER_WINDOWS_BACKEND_DIRECT2D ? "Direct2D" :
           backend->backend_type == RENDER_WINDOWS_BACKEND_DIRECTX ? "DirectX" : "GDI",
           ctx->width, ctx->height);
    
    return 0;
}

void render_windows_destroy(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    // Backend-specific cleanup
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            render_windows_cleanup_direct2d(backend);
            break;
#ifdef DIRECTX_SUPPORT
        case RENDER_WINDOWS_BACKEND_DIRECTX:
            render_windows_cleanup_directx(backend);
            break;
#endif
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            render_windows_cleanup_gdi(backend);
            break;
    }
    
    free(backend->current_font_family);
    free(backend);
    ctx->backend_data = NULL;
    
    CoUninitialize();
    
    printf("Windows render backend destroyed\n");
}

void render_windows_resize(render_context_t* ctx, int width, int height) {
    if (!ctx || !ctx->backend_data) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    backend->width = width;
    backend->height = height;
    
    // Resize backend-specific resources
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_hwnd_render_target) {
                D2D1_SIZE_U size = D2D1::SizeU(width, height);
                backend->d2d_hwnd_render_target->Resize(&size);
            }
            break;
#ifdef DIRECTX_SUPPORT
        case RENDER_WINDOWS_BACKEND_DIRECTX:
            if (backend->dxgi_swap_chain) {
                SAFE_RELEASE(backend->d3d_render_target_view);
                backend->dxgi_swap_chain->ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, 0);
                
                // Recreate render target view
                ID3D11Texture2D* back_buffer;
                backend->dxgi_swap_chain->GetBuffer(0, __uuidof(ID3D11Texture2D), (LPVOID*)&back_buffer);
                backend->d3d_device->CreateRenderTargetView(back_buffer, NULL, &backend->d3d_render_target_view);
                back_buffer->Release();
            }
            break;
#endif
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            // Recreate GDI bitmap
            if (backend->hbitmap) {
                SelectObject(backend->hdc, backend->old_bitmap);
                DeleteObject(backend->hbitmap);
            }
            
            backend->bitmap_info.bmiHeader.biWidth = width;
            backend->bitmap_info.bmiHeader.biHeight = -height; // Top-down
            
            backend->hbitmap = CreateDIBSection(backend->hdc, &backend->bitmap_info, DIB_RGB_COLORS, 
                                               &backend->bitmap_bits, NULL, 0);
            backend->old_bitmap = (HBITMAP)SelectObject(backend->hdc, backend->hbitmap);
            break;
    }
    
    printf("Windows render backend resized: %dx%d\n", width, height);
}

void render_windows_clear_screen(render_context_t* ctx, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_render_target) {
                D2D1_COLOR_F d2d_color = render_windows_d2d_color_from_render_color(color);
                backend->d2d_render_target->Clear(&d2d_color);
            }
            break;
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            if (backend->hdc) {
                COLORREF gdi_color = render_windows_colorref_from_render_color(color);
                HBRUSH brush = CreateSolidBrush(gdi_color);
                RECT rect = {0, 0, backend->width, backend->height};
                FillRect(backend->hdc, &rect, brush);
                DeleteObject(brush);
            }
            break;
    }
}

void render_windows_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_render_target && backend->d2d_brush) {
                D2D1_COLOR_F d2d_color = render_windows_d2d_color_from_render_color(color);
                backend->d2d_brush->SetColor(&d2d_color);
                
                D2D1_RECT_F d2d_rect = render_windows_d2d_rect_from_render_rect(rect);
                backend->d2d_render_target->FillRectangle(&d2d_rect, backend->d2d_brush);
            }
            break;
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            if (backend->hdc) {
                COLORREF gdi_color = render_windows_colorref_from_render_color(color);
                HBRUSH brush = CreateSolidBrush(gdi_color);
                RECT gdi_rect = {rect.x, rect.y, rect.x + rect.width, rect.y + rect.height};
                FillRect(backend->hdc, &gdi_rect, brush);
                DeleteObject(brush);
            }
            break;
    }
}

void render_windows_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font) {
    if (!ctx || !ctx->backend_data || !text) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_render_target && backend->d2d_brush && backend->dwrite_factory) {
                // Create text format if needed
                render_windows_create_dwrite_text_format(backend, font);
                
                if (backend->dwrite_text_format) {
                    D2D1_COLOR_F text_color = render_windows_d2d_color_from_render_color(font.color);
                    backend->d2d_brush->SetColor(&text_color);
                    
                    WCHAR* wide_text = render_windows_utf8_to_wide(text);
                    if (wide_text) {
                        D2D1_RECT_F text_rect = D2D1::RectF(
                            (FLOAT)pos.x, (FLOAT)pos.y, 
                            (FLOAT)(pos.x + 1000), (FLOAT)(pos.y + font.size + 10)
                        );
                        
                        backend->d2d_render_target->DrawText(
                            wide_text, 
                            (UINT32)wcslen(wide_text),
                            backend->dwrite_text_format,
                            &text_rect,
                            backend->d2d_brush
                        );
                        
                        free(wide_text);
                    }
                }
            }
            break;
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            if (backend->hdc) {
                HFONT gdi_font = render_windows_create_gdi_font(font);
                HFONT old_font = (HFONT)SelectObject(backend->hdc, gdi_font);
                
                COLORREF text_color = render_windows_colorref_from_render_color(font.color);
                SetTextColor(backend->hdc, text_color);
                SetBkMode(backend->hdc, TRANSPARENT);
                
                WCHAR* wide_text = render_windows_utf8_to_wide(text);
                if (wide_text) {
                    TextOutW(backend->hdc, pos.x, pos.y, wide_text, (int)wcslen(wide_text));
                    free(wide_text);
                }
                
                SelectObject(backend->hdc, old_font);
                DeleteObject(gdi_font);
            }
            break;
    }
}

void render_windows_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect) {
    if (!ctx || !ctx->backend_data || !image_path) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_render_target && backend->wic_factory) {
                WCHAR* wide_path = render_windows_utf8_to_wide(image_path);
                if (wide_path) {
                    ID2D1Bitmap* bitmap = NULL;
                    HRESULT hr = render_windows_load_image_d2d(backend, wide_path, &bitmap);
                    if (SUCCEEDED(hr) && bitmap) {
                        D2D1_RECT_F dest_rect = render_windows_d2d_rect_from_render_rect(rect);
                        backend->d2d_render_target->DrawBitmap(bitmap, &dest_rect);
                        bitmap->Release();
                    }
                    free(wide_path);
                }
            }
            break;
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            // For GDI, we would need to load the image using GDI+ or similar
            // This is a simplified placeholder
            printf("GDI image loading not implemented for: %s\n", image_path);
            break;
    }
}

void render_windows_present(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_windows_backend_t* backend = (render_windows_backend_t*)ctx->backend_data;
    
    switch (backend->backend_type) {
        case RENDER_WINDOWS_BACKEND_DIRECT2D:
            if (backend->d2d_render_target) {
                backend->d2d_render_target->EndDraw();
                backend->d2d_render_target->BeginDraw();
            }
            break;
#ifdef DIRECTX_SUPPORT
        case RENDER_WINDOWS_BACKEND_DIRECTX:
            if (backend->dxgi_swap_chain) {
                backend->dxgi_swap_chain->Present(backend->vsync_enabled ? 1 : 0, 0);
            }
            break;
#endif
        case RENDER_WINDOWS_BACKEND_GDI:
        default:
            // For GDI, we would typically BitBlt to the window DC
            // This is handled externally in a real implementation
            break;
    }
    
    // Process Windows events
    if (backend->events_enabled) {
        render_windows_process_events(backend);
    }
}

int render_windows_init_gdi(render_windows_backend_t* backend, int width, int height) {
    // Create compatible DC
    HDC screen_dc = GetDC(NULL);
    backend->hdc = CreateCompatibleDC(screen_dc);
    ReleaseDC(NULL, screen_dc);
    
    if (!backend->hdc) {
        return -1;
    }
    
    // Setup bitmap info
    ZeroMemory(&backend->bitmap_info, sizeof(BITMAPINFO));
    backend->bitmap_info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    backend->bitmap_info.bmiHeader.biWidth = width;
    backend->bitmap_info.bmiHeader.biHeight = -height; // Top-down
    backend->bitmap_info.bmiHeader.biPlanes = 1;
    backend->bitmap_info.bmiHeader.biBitCount = 32;
    backend->bitmap_info.bmiHeader.biCompression = BI_RGB;
    
    // Create DIB section
    backend->hbitmap = CreateDIBSection(backend->hdc, &backend->bitmap_info, DIB_RGB_COLORS, 
                                       &backend->bitmap_bits, NULL, 0);
    if (!backend->hbitmap) {
        DeleteDC(backend->hdc);
        return -1;
    }
    
    backend->old_bitmap = (HBITMAP)SelectObject(backend->hdc, backend->hbitmap);
    
    return 0;
}

int render_windows_init_direct2d(render_windows_backend_t* backend, int width, int height) {
    HRESULT hr;
    
    // Create Direct2D factory
    hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &backend->d2d_factory);
    if (FAILED(hr)) return -1;
    
    // Create bitmap render target
    D2D1_SIZE_F size = D2D1::SizeF((FLOAT)width, (FLOAT)height);
    D2D1_RENDER_TARGET_PROPERTIES props = D2D1::RenderTargetProperties();
    props.pixelFormat = D2D1::PixelFormat(DXGI_FORMAT_B8G8R8A8_UNORM, D2D1_ALPHA_MODE_PREMULTIPLIED);
    
    hr = backend->d2d_factory->CreateDCRenderTarget(&props, &backend->d2d_render_target);
    if (FAILED(hr)) {
        SAFE_RELEASE(backend->d2d_factory);
        return -1;
    }
    
    // Create solid color brush
    hr = backend->d2d_render_target->CreateSolidColorBrush(
        D2D1::ColorF(D2D1::ColorF::Black), &backend->d2d_brush);
    if (FAILED(hr)) {
        SAFE_RELEASE(backend->d2d_render_target);
        SAFE_RELEASE(backend->d2d_factory);
        return -1;
    }
    
    // Create DirectWrite factory
    hr = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory),
                           reinterpret_cast<IUnknown**>(&backend->dwrite_factory));
    if (FAILED(hr)) {
        SAFE_RELEASE(backend->d2d_brush);
        SAFE_RELEASE(backend->d2d_render_target);
        SAFE_RELEASE(backend->d2d_factory);
        return -1;
    }
    
    // Create WIC factory
    hr = CoCreateInstance(CLSID_WICImagingFactory, NULL, CLSCTX_INPROC_SERVER,
                         IID_IWICImagingFactory, (LPVOID*)&backend->wic_factory);
    if (FAILED(hr)) {
        SAFE_RELEASE(backend->dwrite_factory);
        SAFE_RELEASE(backend->d2d_brush);
        SAFE_RELEASE(backend->d2d_render_target);
        SAFE_RELEASE(backend->d2d_factory);
        return -1;
    }
    
    backend->d2d_render_target->BeginDraw();
    
    return 0;
}

void render_windows_cleanup_gdi(render_windows_backend_t* backend) {
    if (backend->hbitmap) {
        SelectObject(backend->hdc, backend->old_bitmap);
        DeleteObject(backend->hbitmap);
    }
    if (backend->hdc) {
        DeleteDC(backend->hdc);
    }
    if (backend->current_pen) {
        DeleteObject(backend->current_pen);
    }
    if (backend->current_brush) {
        DeleteObject(backend->current_brush);
    }
    if (backend->current_font) {
        DeleteObject(backend->current_font);
    }
}

void render_windows_cleanup_direct2d(render_windows_backend_t* backend) {
    SAFE_RELEASE(backend->dwrite_text_format);
    SAFE_RELEASE(backend->wic_factory);
    SAFE_RELEASE(backend->dwrite_factory);
    SAFE_RELEASE(backend->d2d_brush);
    SAFE_RELEASE(backend->d2d_bitmap_render_target);
    SAFE_RELEASE(backend->d2d_hwnd_render_target);
    SAFE_RELEASE(backend->d2d_render_target);
    SAFE_RELEASE(backend->d2d_factory);
}

COLORREF render_windows_colorref_from_render_color(render_color_t color) {
    return RGB(color.r, color.g, color.b);
}

D2D1_COLOR_F render_windows_d2d_color_from_render_color(render_color_t color) {
    return D2D1::ColorF(color.r / 255.0f, color.g / 255.0f, color.b / 255.0f, color.a / 255.0f);
}

D2D1_RECT_F render_windows_d2d_rect_from_render_rect(render_rect_t rect) {
    return D2D1::RectF((FLOAT)rect.x, (FLOAT)rect.y, 
                       (FLOAT)(rect.x + rect.width), (FLOAT)(rect.y + rect.height));
}

HFONT render_windows_create_gdi_font(render_font_t font) {
    WCHAR* wide_family = render_windows_utf8_to_wide(font.family);
    
    HFONT hfont = CreateFontW(
        font.size,                    // Height
        0,                            // Width
        0,                            // Escapement
        0,                            // Orientation
        font.bold ? FW_BOLD : FW_NORMAL, // Weight
        font.italic ? TRUE : FALSE,   // Italic
        FALSE,                        // Underline
        FALSE,                        // StrikeOut
        DEFAULT_CHARSET,              // CharSet
        OUT_DEFAULT_PRECIS,           // OutPrecision
        CLIP_DEFAULT_PRECIS,          // ClipPrecision
        DEFAULT_QUALITY,              // Quality
        DEFAULT_PITCH | FF_DONTCARE,  // PitchAndFamily
        wide_family ? wide_family : L"Segoe UI" // FaceName
    );
    
    free(wide_family);
    return hfont;
}

HRESULT render_windows_create_dwrite_text_format(render_windows_backend_t* backend, render_font_t font) {
    if (backend->dwrite_text_format) {
        backend->dwrite_text_format->Release();
        backend->dwrite_text_format = NULL;
    }
    
    WCHAR* wide_family = render_windows_utf8_to_wide(font.family);
    
    HRESULT hr = backend->dwrite_factory->CreateTextFormat(
        wide_family ? wide_family : L"Segoe UI",
        NULL,
        font.bold ? DWRITE_FONT_WEIGHT_BOLD : DWRITE_FONT_WEIGHT_NORMAL,
        font.italic ? DWRITE_FONT_STYLE_ITALIC : DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_STRETCH_NORMAL,
        (FLOAT)font.size,
        L"en-us",
        &backend->dwrite_text_format
    );
    
    free(wide_family);
    return hr;
}

HRESULT render_windows_load_image_d2d(render_windows_backend_t* backend, const WCHAR* image_path, ID2D1Bitmap** bitmap) {
    IWICBitmapDecoder* decoder = NULL;
    IWICBitmapFrameDecode* source = NULL;
    IWICFormatConverter* converter = NULL;
    
    HRESULT hr = backend->wic_factory->CreateDecoderFromFilename(
        image_path, NULL, GENERIC_READ, WICDecodeMetadataCacheOnLoad, &decoder);
    
    if (SUCCEEDED(hr)) {
        hr = decoder->GetFrame(0, &source);
    }
    
    if (SUCCEEDED(hr)) {
        hr = backend->wic_factory->CreateFormatConverter(&converter);
    }
    
    if (SUCCEEDED(hr)) {
        hr = converter->Initialize(source, GUID_WICPixelFormat32bppPBGRA,
                                 WICBitmapDitherTypeNone, NULL, 0.0, WICBitmapPaletteTypeMedianCut);
    }
    
    if (SUCCEEDED(hr)) {
        hr = backend->d2d_render_target->CreateBitmapFromWicBitmap(converter, NULL, bitmap);
    }
    
    SAFE_RELEASE(converter);
    SAFE_RELEASE(source);
    SAFE_RELEASE(decoder);
    
    return hr;
}

render_windows_backend_type_t render_windows_detect_best_backend(void) {
    // Try Direct2D first
    if (render_windows_is_direct2d_available()) {
        return RENDER_WINDOWS_BACKEND_DIRECT2D;
    }
    
    // Fall back to GDI
    return RENDER_WINDOWS_BACKEND_GDI;
}

BOOL render_windows_is_direct2d_available(void) {
    ID2D1Factory* factory = NULL;
    HRESULT hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &factory);
    if (SUCCEEDED(hr)) {
        factory->Release();
        return TRUE;
    }
    return FALSE;
}

WCHAR* render_windows_utf8_to_wide(const char* utf8_str) {
    if (!utf8_str) return NULL;
    
    int wide_len = MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, NULL, 0);
    if (wide_len <= 0) return NULL;
    
    WCHAR* wide_str = (WCHAR*)malloc(wide_len * sizeof(WCHAR));
    if (!wide_str) return NULL;
    
    MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, wide_str, wide_len);
    return wide_str;
}

char* render_windows_wide_to_utf8(const WCHAR* wide_str) {
    if (!wide_str) return NULL;
    
    int utf8_len = WideCharToMultiByte(CP_UTF8, 0, wide_str, -1, NULL, 0, NULL, NULL);
    if (utf8_len <= 0) return NULL;
    
    char* utf8_str = (char*)malloc(utf8_len);
    if (!utf8_str) return NULL;
    
    WideCharToMultiByte(CP_UTF8, 0, wide_str, -1, utf8_str, utf8_len, NULL, NULL);
    return utf8_str;
}

void render_windows_process_events(render_windows_backend_t* backend) {
    MSG msg;
    while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
        if (backend->event_callback) {
            backend->event_callback(msg.message, msg.wParam, msg.lParam);
        }
        
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

LRESULT CALLBACK render_windows_wndproc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) {
    // Default window procedure
    return DefWindowProc(hwnd, message, wparam, lparam);
}

#ifdef DIRECTX_SUPPORT
BOOL render_windows_is_directx_available(void) {
    ID3D11Device* device = NULL;
    D3D_FEATURE_LEVEL feature_level;
    
    HRESULT hr = D3D11CreateDevice(
        NULL, D3D_DRIVER_TYPE_HARDWARE, NULL, 0,
        NULL, 0, D3D11_SDK_VERSION,
        &device, &feature_level, NULL
    );
    
    if (SUCCEEDED(hr)) {
        device->Release();
        return TRUE;
    }
    return FALSE;
}
#endif

#endif // _WIN32 || _WIN64