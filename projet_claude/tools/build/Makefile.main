# Makefile for C Markdown Editor macOS App

APP_NAME = CMarkdownEditor
BUNDLE_NAME = $(APP_NAME).app
CONTENTS_DIR = $(BUNDLE_NAME)/Contents
MACOS_DIR = $(CONTENTS_DIR)/MacOS
RESOURCES_DIR = $(CONTENTS_DIR)/Resources

# Compiler and linker settings
CC = clang
OBJC_FLAGS = -x objective-c -fobjc-arc -fobjc-weak
CFLAGS = -std=c11 -Wall -Wextra -O2 -g
FRAMEWORKS = -framework Cocoa -framework Foundation -framework UniformTypeIdentifiers

# Include paths for our C engines
INCLUDES = -I../editor -I../markdown -I../render_engine -I../cursor -I../search_engine -I../backup_engine -I../crypto_engine

# Library paths and libraries - DISABLED FOR TESTING
# LIBS = -L../editor -leditor \
#        -L../markdown -lmarkdown \
#        -L../cursor -lcursor \
#        -L../search_engine -lsearch_engine \
#        -L../crypto_engine -lcrypto_engine
LIBS =

# Source files
SOURCES = MarkdownEditorApp/AppDelegate.m \
          MarkdownEditorApp/main.m

# Object files
OBJECTS = $(SOURCES:.m=.o)

# Default target
all: $(BUNDLE_NAME)

# Create the app bundle
$(BUNDLE_NAME): $(OBJECTS)
	@echo "🏗️ Creating app bundle..."
	@mkdir -p $(MACOS_DIR)
	@mkdir -p $(RESOURCES_DIR)
	
	@echo "🔗 Linking executable..."
	$(CC) $(FRAMEWORKS) $(OBJECTS) $(LIBS) -o $(MACOS_DIR)/$(APP_NAME)
	
	@echo "📋 Copying Info.plist..."
	@cp MarkdownEditorApp/Info.plist $(CONTENTS_DIR)/Info.plist
	
	@echo "✅ App bundle created: $(BUNDLE_NAME)"

# Compile Objective-C source files
%.o: %.m
	@echo "🔨 Compiling $<..."
	$(CC) $(OBJC_FLAGS) $(CFLAGS) $(INCLUDES) -c $< -o $@

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	@rm -rf $(BUNDLE_NAME)
	@rm -f $(OBJECTS)
	@echo "✅ Cleaned"

# Install/run the app
run: $(BUNDLE_NAME)
	@echo "🚀 Launching $(APP_NAME)..."
	@open $(BUNDLE_NAME)

# Debug info
debug: $(BUNDLE_NAME)
	@echo "📝 App Info:"
	@echo "Bundle: $(BUNDLE_NAME)"
	@echo "Executable: $(MACOS_DIR)/$(APP_NAME)"
	@file $(MACOS_DIR)/$(APP_NAME)
	@ls -la $(BUNDLE_NAME)

.PHONY: all clean run debug

# Check dependencies
check-deps:
	@echo "🔍 Checking C engine libraries..."
	@for dir in editor markdown render_engine cursor search_engine backup_engine crypto_engine; do \
		lib="../$$dir/lib$$dir.a"; \
		if [ -f "$$lib" ]; then \
			echo "✅ Found $$lib"; \
		else \
			echo "❌ Missing $$lib"; \
		fi; \
	done