#include "render_android.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#ifdef __ANDROID__

int render_android_init(render_context_t* ctx) {
    if (!ctx) return -1;
    
    render_android_backend_t* backend = calloc(1, sizeof(render_android_backend_t));
    if (!backend) return -1;
    
    backend->canvas_width = ctx->width;
    backend->canvas_height = ctx->height;
    backend->current_color = 0xFF000000; // Black
    backend->text_size = 14.0f;
    backend->anti_alias = true;
    backend->bitmap_locked = false;
    
    // In a real Android implementation, JNI environment would be passed
    // For now, we'll set up the structure for when it's available
    backend->env = NULL; // Will be set externally
    
    // Setup context drawing functions
    ctx->backend_data = backend;
    ctx->clear_screen = render_android_clear_screen;
    ctx->draw_rect = render_android_draw_rect;
    ctx->draw_text = render_android_draw_text;
    ctx->draw_image = render_android_draw_image;
    ctx->present = render_android_present;
    
    LOGI("Android render backend initialized: %dx%d", ctx->width, ctx->height);
    
    return 0;
}

void render_android_destroy(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    // Release bitmap if locked
    if (backend->bitmap_locked) {
        render_android_unlock_bitmap(backend);
    }
    
    // Release bitmap
    render_android_release_bitmap(backend);
    
    // Cleanup JNI references
    render_android_cleanup_jni(backend);
    
    free(backend);
    ctx->backend_data = NULL;
    
    LOGI("Android render backend destroyed");
}

void render_android_resize(render_context_t* ctx, int width, int height) {
    if (!ctx || !ctx->backend_data) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    // Release old bitmap
    if (backend->bitmap_locked) {
        render_android_unlock_bitmap(backend);
    }
    render_android_release_bitmap(backend);
    
    // Update dimensions
    backend->canvas_width = width;
    backend->canvas_height = height;
    
    // Create new bitmap
    render_android_create_bitmap(backend, width, height);
    
    LOGI("Android render backend resized: %dx%d", width, height);
}

void render_android_clear_screen(render_context_t* ctx, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    if (!backend->env || !backend->canvas) {
        LOGE("JNI environment or canvas not available for clear_screen");
        return;
    }
    
    // Set paint color
    jint android_color = render_android_color_from_render_color(color);
    (*backend->env)->CallVoidMethod(backend->env, backend->paint, 
                                   backend->paint_set_color_method, android_color);
    
    // Clear entire canvas
    jobject rect = render_android_create_rectf(backend->env, 
        (render_rect_t){0, 0, backend->canvas_width, backend->canvas_height});
    
    (*backend->env)->CallVoidMethod(backend->env, backend->canvas,
                                   backend->canvas_draw_rect_method, rect, backend->paint);
    
    (*backend->env)->DeleteLocalRef(backend->env, rect);
}

void render_android_draw_rect(render_context_t* ctx, render_rect_t rect, render_color_t color) {
    if (!ctx || !ctx->backend_data) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    if (!backend->env || !backend->canvas) {
        LOGE("JNI environment or canvas not available for draw_rect");
        return;
    }
    
    // Set paint color
    jint android_color = render_android_color_from_render_color(color);
    (*backend->env)->CallVoidMethod(backend->env, backend->paint,
                                   backend->paint_set_color_method, android_color);
    
    // Create rectangle and draw
    jobject rectf = render_android_create_rectf(backend->env, rect);
    (*backend->env)->CallVoidMethod(backend->env, backend->canvas,
                                   backend->canvas_draw_rect_method, rectf, backend->paint);
    
    (*backend->env)->DeleteLocalRef(backend->env, rectf);
}

void render_android_draw_text(render_context_t* ctx, const char* text, render_point_t pos, render_font_t font) {
    if (!ctx || !ctx->backend_data || !text) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    if (!backend->env || !backend->canvas) {
        LOGE("JNI environment or canvas not available for draw_text");
        return;
    }
    
    // Set paint properties
    render_android_set_paint_properties(backend, font.color, font.size);
    
    // Create Java string
    jstring java_text = (*backend->env)->NewStringUTF(backend->env, text);
    
    // Draw text
    (*backend->env)->CallVoidMethod(backend->env, backend->canvas,
                                   backend->canvas_draw_text_method,
                                   java_text, (jfloat)pos.x, (jfloat)pos.y, backend->paint);
    
    (*backend->env)->DeleteLocalRef(backend->env, java_text);
}

void render_android_draw_image(render_context_t* ctx, const char* image_path, render_rect_t rect) {
    if (!ctx || !ctx->backend_data || !image_path) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    if (!backend->env || !backend->canvas) {
        LOGE("JNI environment or canvas not available for draw_image");
        return;
    }
    
    // In a real implementation, we would:
    // 1. Load image from assets or file system using Android APIs
    // 2. Create bitmap from image data
    // 3. Draw bitmap to canvas at specified rectangle
    
    // For now, just log the operation
    LOGI("Drawing image: %s at (%d,%d,%d,%d)", image_path, rect.x, rect.y, rect.width, rect.height);
    
    // TODO: Implement image loading and drawing
    // This would involve BitmapFactory, asset management, etc.
}

void render_android_present(render_context_t* ctx) {
    if (!ctx || !ctx->backend_data) return;
    
    render_android_backend_t* backend = (render_android_backend_t*)ctx->backend_data;
    
    // In a real Android implementation, this would:
    // 1. Finish drawing to the canvas
    // 2. Trigger a view invalidation/refresh
    // 3. Present the rendered frame to the screen
    
    LOGI("Presenting frame");
}

int render_android_setup_jni(render_android_backend_t* backend, JNIEnv* env, jobject activity) {
    if (!backend || !env) return -1;
    
    backend->env = env;
    backend->activity = (*env)->NewGlobalRef(env, activity);
    
    // Get class references
    jclass local_canvas_class = (*env)->FindClass(env, "android/graphics/Canvas");
    backend->canvas_class = (*env)->NewGlobalRef(env, local_canvas_class);
    (*env)->DeleteLocalRef(env, local_canvas_class);
    
    jclass local_paint_class = (*env)->FindClass(env, "android/graphics/Paint");
    backend->paint_class = (*env)->NewGlobalRef(env, local_paint_class);
    (*env)->DeleteLocalRef(env, local_paint_class);
    
    jclass local_bitmap_class = (*env)->FindClass(env, "android/graphics/Bitmap");
    backend->bitmap_class = (*env)->NewGlobalRef(env, local_bitmap_class);
    (*env)->DeleteLocalRef(env, local_bitmap_class);
    
    jclass local_rect_class = (*env)->FindClass(env, "android/graphics/Rect");
    backend->rect_class = (*env)->NewGlobalRef(env, local_rect_class);
    (*env)->DeleteLocalRef(env, local_rect_class);
    
    jclass local_rectf_class = (*env)->FindClass(env, "android/graphics/RectF");
    backend->rectf_class = (*env)->NewGlobalRef(env, local_rectf_class);
    (*env)->DeleteLocalRef(env, local_rectf_class);
    
    // Cache method IDs
    backend->canvas_draw_rect_method = (*env)->GetMethodID(env, backend->canvas_class,
        "drawRect", "(Landroid/graphics/RectF;Landroid/graphics/Paint;)V");
    backend->canvas_draw_text_method = (*env)->GetMethodID(env, backend->canvas_class,
        "drawText", "(Ljava/lang/String;FFLandroid/graphics/Paint;)V");
    backend->canvas_draw_bitmap_method = (*env)->GetMethodID(env, backend->canvas_class,
        "drawBitmap", "(Landroid/graphics/Bitmap;Landroid/graphics/Rect;Landroid/graphics/RectF;Landroid/graphics/Paint;)V");
    
    backend->paint_set_color_method = (*env)->GetMethodID(env, backend->paint_class,
        "setColor", "(I)V");
    backend->paint_set_text_size_method = (*env)->GetMethodID(env, backend->paint_class,
        "setTextSize", "(F)V");
    backend->paint_set_anti_alias_method = (*env)->GetMethodID(env, backend->paint_class,
        "setAntiAlias", "(Z)V");
    
    // Create Paint object
    jmethodID paint_constructor = (*env)->GetMethodID(env, backend->paint_class, "<init>", "()V");
    jobject local_paint = (*env)->NewObject(env, backend->paint_class, paint_constructor);
    backend->paint = (*env)->NewGlobalRef(env, local_paint);
    (*env)->DeleteLocalRef(env, local_paint);
    
    // Set initial paint properties
    (*env)->CallVoidMethod(env, backend->paint, backend->paint_set_anti_alias_method, JNI_TRUE);
    
    return 0;
}

void render_android_cleanup_jni(render_android_backend_t* backend) {
    if (!backend || !backend->env) return;
    
    JNIEnv* env = backend->env;
    
    // Delete global references
    if (backend->activity) (*env)->DeleteGlobalRef(env, backend->activity);
    if (backend->canvas) (*env)->DeleteGlobalRef(env, backend->canvas);
    if (backend->bitmap) (*env)->DeleteGlobalRef(env, backend->bitmap);
    if (backend->paint) (*env)->DeleteGlobalRef(env, backend->paint);
    if (backend->canvas_class) (*env)->DeleteGlobalRef(env, backend->canvas_class);
    if (backend->paint_class) (*env)->DeleteGlobalRef(env, backend->paint_class);
    if (backend->bitmap_class) (*env)->DeleteGlobalRef(env, backend->bitmap_class);
    if (backend->rect_class) (*env)->DeleteGlobalRef(env, backend->rect_class);
    if (backend->rectf_class) (*env)->DeleteGlobalRef(env, backend->rectf_class);
}

jint render_android_color_from_render_color(render_color_t color) {
    // Android color format: ARGB
    return (jint)((color.a << 24) | (color.r << 16) | (color.g << 8) | color.b);
}

jobject render_android_create_rect(JNIEnv* env, render_rect_t rect) {
    jclass rect_class = (*env)->FindClass(env, "android/graphics/Rect");
    jmethodID constructor = (*env)->GetMethodID(env, rect_class, "<init>", "(IIII)V");
    
    jobject java_rect = (*env)->NewObject(env, rect_class, constructor,
                                         (jint)rect.x, (jint)rect.y,
                                         (jint)(rect.x + rect.width), (jint)(rect.y + rect.height));
    
    (*env)->DeleteLocalRef(env, rect_class);
    return java_rect;
}

jobject render_android_create_rectf(JNIEnv* env, render_rect_t rect) {
    jclass rectf_class = (*env)->FindClass(env, "android/graphics/RectF");
    jmethodID constructor = (*env)->GetMethodID(env, rectf_class, "<init>", "(FFFF)V");
    
    jobject java_rect = (*env)->NewObject(env, rectf_class, constructor,
                                         (jfloat)rect.x, (jfloat)rect.y,
                                         (jfloat)(rect.x + rect.width), (jfloat)(rect.y + rect.height));
    
    (*env)->DeleteLocalRef(env, rectf_class);
    return java_rect;
}

int render_android_create_bitmap(render_android_backend_t* backend, int width, int height) {
    if (!backend || !backend->env) return -1;
    
    JNIEnv* env = backend->env;
    
    // Get Bitmap.Config.ARGB_8888
    jclass config_class = (*env)->FindClass(env, "android/graphics/Bitmap$Config");
    jfieldID argb_8888_field = (*env)->GetStaticFieldID(env, config_class, "ARGB_8888", 
                                                        "Landroid/graphics/Bitmap$Config;");
    jobject config = (*env)->GetStaticObjectField(env, config_class, argb_8888_field);
    
    // Create bitmap
    jmethodID create_bitmap_method = (*env)->GetStaticMethodID(env, backend->bitmap_class,
        "createBitmap", "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;");
    
    jobject local_bitmap = (*env)->CallStaticObjectMethod(env, backend->bitmap_class,
                                                          create_bitmap_method, width, height, config);
    
    backend->bitmap = (*env)->NewGlobalRef(env, local_bitmap);
    
    // Create canvas with bitmap
    jmethodID canvas_constructor = (*env)->GetMethodID(env, backend->canvas_class,
                                                      "<init>", "(Landroid/graphics/Bitmap;)V");
    jobject local_canvas = (*env)->NewObject(env, backend->canvas_class, canvas_constructor, backend->bitmap);
    backend->canvas = (*env)->NewGlobalRef(env, local_canvas);
    
    // Cleanup local references
    (*env)->DeleteLocalRef(env, local_bitmap);
    (*env)->DeleteLocalRef(env, local_canvas);
    (*env)->DeleteLocalRef(env, config);
    (*env)->DeleteLocalRef(env, config_class);
    
    return 0;
}

void render_android_release_bitmap(render_android_backend_t* backend) {
    if (!backend || !backend->env) return;
    
    if (backend->bitmap) {
        (*backend->env)->DeleteGlobalRef(backend->env, backend->bitmap);
        backend->bitmap = NULL;
    }
    if (backend->canvas) {
        (*backend->env)->DeleteGlobalRef(backend->env, backend->canvas);
        backend->canvas = NULL;
    }
}

int render_android_lock_bitmap(render_android_backend_t* backend) {
    if (!backend || !backend->bitmap || backend->bitmap_locked) return -1;
    
    int result = AndroidBitmap_getInfo(backend->env, backend->bitmap, &backend->bitmap_info);
    if (result != ANDROID_BITMAP_RESULT_SUCCESS) {
        LOGE("Failed to get bitmap info: %d", result);
        return -1;
    }
    
    result = AndroidBitmap_lockPixels(backend->env, backend->bitmap, &backend->bitmap_pixels);
    if (result != ANDROID_BITMAP_RESULT_SUCCESS) {
        LOGE("Failed to lock bitmap pixels: %d", result);
        return -1;
    }
    
    backend->bitmap_locked = true;
    return 0;
}

void render_android_unlock_bitmap(render_android_backend_t* backend) {
    if (!backend || !backend->bitmap_locked) return;
    
    AndroidBitmap_unlockPixels(backend->env, backend->bitmap);
    backend->bitmap_locked = false;
}

void render_android_set_paint_properties(render_android_backend_t* backend, render_color_t color, float text_size) {
    if (!backend || !backend->env || !backend->paint) return;
    
    jint android_color = render_android_color_from_render_color(color);
    (*backend->env)->CallVoidMethod(backend->env, backend->paint,
                                   backend->paint_set_color_method, android_color);
    (*backend->env)->CallVoidMethod(backend->env, backend->paint,
                                   backend->paint_set_text_size_method, (jfloat)text_size);
    
    backend->current_color = android_color;
    backend->text_size = text_size;
}

#endif // __ANDROID__