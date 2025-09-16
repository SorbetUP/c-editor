CC = clang
CFLAGS = -std=c11 -Wall -Wextra -O2 -g -fPIC

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Platform-specific settings
ifeq ($(UNAME_S),Darwin)
    PLATFORM = MACOS
    PLATFORM_CFLAGS = -framework CoreGraphics -framework CoreText -framework ApplicationServices -framework ImageIO
    ifeq ($(shell uname -m),arm64)
        # Apple Silicon Mac
        PLATFORM_CFLAGS += -target arm64-apple-macos11
    endif
endif

ifeq ($(UNAME_S),Linux)
    PLATFORM = LINUX
    PLATFORM_CFLAGS = -lX11 -lcairo -lpango-1.0 -lpangocairo-1.0 -lgdk_pixbuf-2.0 -lgobject-2.0 -lglib-2.0
    PLATFORM_CFLAGS += $(shell pkg-config --cflags --libs cairo pango pangocairo gdk-pixbuf-2.0)
endif

# Base source files
COMMON_SOURCES = render_engine.c common/render_software.c
HEADERS = render_engine.h common/render_software.h

# Platform-specific sources
ifeq ($(PLATFORM),MACOS)
    PLATFORM_SOURCES = platforms/macos/render_macos.c
    PLATFORM_HEADERS = platforms/macos/render_macos.h
    CFLAGS += -DPLATFORM_MACOS=1
endif

ifeq ($(PLATFORM),LINUX)
    PLATFORM_SOURCES = platforms/linux/render_linux.c
    PLATFORM_HEADERS = platforms/linux/render_linux.h
    CFLAGS += -DPLATFORM_LINUX=1
endif

# For cross-compilation or when building all platforms
ifeq ($(BUILD_ALL_PLATFORMS),1)
    PLATFORM_SOURCES = platforms/macos/render_macos.c \
                      platforms/ios/render_ios.c \
                      platforms/android/render_android.c \
                      platforms/linux/render_linux.c \
                      platforms/windows/render_windows.c
    PLATFORM_HEADERS = platforms/macos/render_macos.h \
                      platforms/ios/render_ios.h \
                      platforms/android/render_android.h \
                      platforms/linux/render_linux.h \
                      platforms/windows/render_windows.h
    CFLAGS += -DBUILD_ALL_PLATFORMS=1
endif

ALL_SOURCES = $(COMMON_SOURCES) $(PLATFORM_SOURCES)
ALL_HEADERS = $(HEADERS) $(PLATFORM_HEADERS)
OBJECTS = $(ALL_SOURCES:.c=.o)
TARGET = librender_engine.a

# Default target
all: $(TARGET)

# Static library
$(TARGET): $(OBJECTS)
	ar rcs $(TARGET) $(OBJECTS)
	@echo "✅ Render engine library built: $(TARGET)"
	@echo "📦 Platform: $(PLATFORM)"
	@echo "📄 Objects: $(OBJECTS)"

# Compile source files
%.o: %.c $(ALL_HEADERS)
	$(CC) $(CFLAGS) -c $< -o $@

# Individual platform targets
macos:
	$(MAKE) PLATFORM=MACOS PLATFORM_SOURCES=platforms/macos/render_macos.c PLATFORM_HEADERS=platforms/macos/render_macos.h

ios:
	$(MAKE) PLATFORM=IOS PLATFORM_SOURCES=platforms/ios/render_ios.c PLATFORM_HEADERS=platforms/ios/render_ios.h CFLAGS="$(CFLAGS) -DTARGET_OS_IPHONE=1"

android:
	$(MAKE) PLATFORM=ANDROID PLATFORM_SOURCES=platforms/android/render_android.c PLATFORM_HEADERS=platforms/android/render_android.h CFLAGS="$(CFLAGS) -D__ANDROID__=1"

linux:
	$(MAKE) PLATFORM=LINUX PLATFORM_SOURCES=platforms/linux/render_linux.c PLATFORM_HEADERS=platforms/linux/render_linux.h

windows:
	$(MAKE) PLATFORM=WINDOWS PLATFORM_SOURCES=platforms/windows/render_windows.c PLATFORM_HEADERS=platforms/windows/render_windows.h CFLAGS="$(CFLAGS) -D_WIN32=1"

# Cross-platform build
all-platforms:
	$(MAKE) BUILD_ALL_PLATFORMS=1

# Test program
test: $(TARGET)
	@echo "🧪 Building test program..."
	$(CC) $(CFLAGS) $(PLATFORM_CFLAGS) test_render.c $(TARGET) -o test_render
	@echo "✅ Test program built: ./test_render"

# Demo program
demo: $(TARGET)
	@echo "🎮 Building demo program..."
	$(CC) $(CFLAGS) $(PLATFORM_CFLAGS) demo_render.c $(TARGET) -o demo_render
	@echo "✅ Demo program built: ./demo_render"

# Create test file if it doesn't exist
test_render.c:
	@echo "📝 Creating test file..."
	@echo '#include "render_engine.h"' > test_render.c
	@echo '#include <stdio.h>' >> test_render.c
	@echo '' >> test_render.c
	@echo 'int main() {' >> test_render.c
	@echo '    printf("Testing render engine...\\n");' >> test_render.c
	@echo '    render_context_t* ctx = render_engine_create_context(RENDER_BACKEND_FRAMEBUFFER, 800, 600);' >> test_render.c
	@echo '    if (!ctx) {' >> test_render.c
	@echo '        printf("❌ Failed to create render context\\n");' >> test_render.c
	@echo '        return 1;' >> test_render.c
	@echo '    }' >> test_render.c
	@echo '    printf("✅ Render context created: %dx%d\\n", ctx->width, ctx->height);' >> test_render.c
	@echo '    ctx->root = render_engine_create_element(RENDER_ELEMENT_BOX, "root");' >> test_render.c
	@echo '    if (ctx->root) {' >> test_render.c
	@echo '        printf("✅ Root element created\\n");' >> test_render.c
	@echo '        render_element_t* text = render_engine_create_element(RENDER_ELEMENT_TEXT, "hello");' >> test_render.c
	@echo '        render_engine_set_text(text, "Hello, Render Engine!");' >> test_render.c
	@echo '        render_engine_add_child(ctx->root, text);' >> test_render.c
	@echo '        printf("✅ Text element added\\n");' >> test_render.c
	@echo '        render_engine_render(ctx);' >> test_render.c
	@echo '        printf("✅ Frame rendered\\n");' >> test_render.c
	@echo '    }' >> test_render.c
	@echo '    render_engine_destroy_context(ctx);' >> test_render.c
	@echo '    printf("✅ Render context destroyed\\n");' >> test_render.c
	@echo '    return 0;' >> test_render.c
	@echo '}' >> test_render.c

# Clean build files
clean:
	rm -f $(OBJECTS) $(TARGET) test_render demo_render test_render.c demo_render.c
	@echo "🧹 Cleaned build files"

# Status report
status:
	@echo "📊 Render Engine Status"
	@echo "======================"
	@echo "Platform: $(PLATFORM)"
	@echo "Target: $(TARGET)"
	@echo "Sources: $(ALL_SOURCES)"
	@echo "Headers: $(ALL_HEADERS)"
	@echo "Objects: $(OBJECTS)"
	@echo ""
	@echo "📁 Directory structure:"
	@find . -type f -name "*.c" -o -name "*.h" | head -20
	@echo ""
	@echo "📏 Library size:"
	@if [ -f $(TARGET) ]; then ls -lh $(TARGET); else echo "Not built yet"; fi

static: $(TARGET)

.PHONY: all macos ios android linux windows all-platforms test demo clean status static
