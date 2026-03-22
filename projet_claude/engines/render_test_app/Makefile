CC = clang
CFLAGS = -std=c11 -Wall -Wextra -O2 -g
OBJCFLAGS = -fobjc-arc -std=gnu99 -Wall -Wextra -O2 -g
FRAMEWORKS = -framework Cocoa -framework CoreGraphics -framework CoreText -framework ApplicationServices -framework ImageIO -framework QuartzCore

# Paths
RENDER_ENGINE_DIR = ../render_engine
RENDER_ENGINE_LIB = $(RENDER_ENGINE_DIR)/librender_engine.a

# Sources
SOURCES = main.m
TARGET = RenderTestApp

# Default target
all: $(TARGET)

# Check if render engine library exists
$(RENDER_ENGINE_LIB):
	@echo "🔧 Building render engine library..."
	@cd $(RENDER_ENGINE_DIR) && make
	@echo "✅ Render engine library ready"

# Build the macOS app
$(TARGET): $(RENDER_ENGINE_LIB) $(SOURCES)
	@echo "🚀 Building macOS test application..."
	$(CC) $(OBJCFLAGS) $(FRAMEWORKS) -I$(RENDER_ENGINE_DIR) $(SOURCES) $(RENDER_ENGINE_LIB) -o $(TARGET)
	@echo "✅ Application built: ./$(TARGET)"
	@echo ""
	@echo "🎯 To run the app:"
	@echo "   ./$(TARGET)"
	@echo ""
	@echo "📋 App features:"
	@echo "   • Native macOS window with render engine integration"
	@echo "   • Demo content showcasing web-like layout"
	@echo "   • Mouse interaction logging"
	@echo "   • Real-time rendering with display link"
	@echo "   • Window resizing support"

# Create app bundle (optional)
bundle: $(TARGET)
	@echo "📦 Creating app bundle..."
	@mkdir -p $(TARGET).app/Contents/MacOS
	@mkdir -p $(TARGET).app/Contents/Resources
	@cp $(TARGET) $(TARGET).app/Contents/MacOS/
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(TARGET).app/Contents/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(TARGET).app/Contents/Info.plist
	@echo '<plist version="1.0"><dict>' >> $(TARGET).app/Contents/Info.plist
	@echo '<key>CFBundleExecutable</key><string>$(TARGET)</string>' >> $(TARGET).app/Contents/Info.plist
	@echo '<key>CFBundleIdentifier</key><string>com.example.rendertestapp</string>' >> $(TARGET).app/Contents/Info.plist
	@echo '<key>CFBundleName</key><string>Render Test App</string>' >> $(TARGET).app/Contents/Info.plist
	@echo '<key>CFBundleVersion</key><string>1.0</string>' >> $(TARGET).app/Contents/Info.plist
	@echo '<key>NSHighResolutionCapable</key><true/>' >> $(TARGET).app/Contents/Info.plist
	@echo '</dict></plist>' >> $(TARGET).app/Contents/Info.plist
	@echo "✅ App bundle created: $(TARGET).app"
	@echo "🚀 You can now run: open $(TARGET).app"

# Run the application
run: $(TARGET)
	@echo "🎬 Launching test application..."
	@./$(TARGET)

# Test with verbose output
test: $(TARGET)
	@echo "🧪 Running test application with debug output..."
	@./$(TARGET) 2>&1 | tee test_output.log

# Clean build files
clean:
	@echo "🧹 Cleaning build files..."
	@rm -f $(TARGET)
	@rm -rf $(TARGET).app
	@rm -f test_output.log
	@echo "✅ Clean complete"

# Clean everything including render engine
clean-all: clean
	@echo "🧹 Cleaning render engine..."
	@cd $(RENDER_ENGINE_DIR) && make clean
	@echo "✅ Full clean complete"

# Status check
status:
	@echo "📊 Render Test App Status"
	@echo "========================="
	@echo "Target: $(TARGET)"
	@echo "Sources: $(SOURCES)"
	@echo "Render Engine: $(RENDER_ENGINE_LIB)"
	@echo ""
	@if [ -f $(TARGET) ]; then \
		echo "📱 App Status: Built ($(shell ls -lh $(TARGET) | awk '{print $$5}'))"; \
	else \
		echo "📱 App Status: Not built"; \
	fi
	@if [ -f $(RENDER_ENGINE_LIB) ]; then \
		echo "🔧 Render Engine: Ready ($(shell ls -lh $(RENDER_ENGINE_LIB) | awk '{print $$5}'))"; \
	else \
		echo "🔧 Render Engine: Not built"; \
	fi
	@echo ""
	@echo "🎯 Available commands:"
	@echo "   make          - Build the application"
	@echo "   make bundle   - Create app bundle"
	@echo "   make run      - Build and run"
	@echo "   make test     - Run with debug output"
	@echo "   make clean    - Clean build files"
	@echo "   make status   - Show this status"

# Help
help:
	@echo "🚀 Render Test App Build System"
	@echo "================================"
	@echo ""
	@echo "This application demonstrates the C render engine integrated"
	@echo "into a native macOS app using Objective-C and Cocoa."
	@echo ""
	@echo "Features:"
	@echo "  • Native Cocoa window and event handling"
	@echo "  • Render engine integration with Core Graphics"
	@echo "  • Real-time rendering with CVDisplayLink"
	@echo "  • Mouse interaction and window resizing"
	@echo "  • Demo content showcasing web-like layout"
	@echo ""
	@echo "Commands:"
	@echo "  make          - Build the application"
	@echo "  make bundle   - Create .app bundle"
	@echo "  make run      - Build and run immediately"
	@echo "  make test     - Run with debug logging"
	@echo "  make clean    - Remove build files"
	@echo "  make clean-all- Clean everything"
	@echo "  make status   - Show build status"
	@echo "  make help     - Show this help"

.PHONY: all bundle run test clean clean-all status help