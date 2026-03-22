#ifndef RENDER_ANDROID_H
#define RENDER_ANDROID_H

#include "../../render_engine.h"

#ifdef __ANDROID__

#include <jni.h>
#include <android/bitmap.h>
#include <android/log.h>

#define LOG_TAG "RenderEngine"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

typedef struct {
    // JNI environment and objects
    JNIEnv* env;
    jobject activity;
    jobject canvas;
    jobject bitmap;
    jobject paint;
    
    // Canvas dimensions
    int canvas_width;
    int canvas_height;
    
    // Bitmap data
    void* bitmap_pixels;
    AndroidBitmapInfo bitmap_info;
    bool bitmap_locked;
    
    // Drawing state
    jint current_color;
    float text_size;
    bool anti_alias;
    
    // Java method IDs (cached for performance)
    jmethodID canvas_draw_rect_method;
    jmethodID canvas_draw_text_method;
    jmethodID canvas_draw_bitmap_method;
    jmethodID paint_set_color_method;
    jmethodID paint_set_text_size_method;
    jmethodID paint_set_anti_alias_method;
    
    // Class references
    jclass canvas_class;
    jclass paint_class;
    jclass bitmap_class;
    jclass rect_class;
    jclass rectf_class;
} render_android_backend_t;

// Platform-specific functions
int render_android_init(render_context_t* ctx);
void render_android_destroy(render_context_t* ctx);
void render_android_resize(render_context_t* ctx, int width, int height);

// Drawing functions
void render_android_clear_screen(render_context_t* ctx, render_color_t color);
void render_android_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color);
void render_android_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font);
void render_android_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect);
void render_android_present(render_context_t* ctx);

// JNI helper functions
int render_android_setup_jni(render_android_backend_t* backend, JNIEnv* env, jobject activity);
void render_android_cleanup_jni(render_android_backend_t* backend);
jint render_android_color_from_render_color(render_color_t color);
jobject render_android_create_rect(JNIEnv* env, render_rect_t rect);
jobject render_android_create_rectf(JNIEnv* env, render_rect_t rect);

// Bitmap management
int render_android_create_bitmap(render_android_backend_t* backend, int width, int height);
void render_android_release_bitmap(render_android_backend_t* backend);
int render_android_lock_bitmap(render_android_backend_t* backend);
void render_android_unlock_bitmap(render_android_backend_t* backend);

// Utility functions
void render_android_set_paint_properties(render_android_backend_t* backend, render_color_t color, float text_size);

#endif // __ANDROID__

#endif // RENDER_ANDROID_H