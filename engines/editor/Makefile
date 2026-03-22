# Makefile for Editor Engine
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g
EMCC = emcc

# Include paths  
INCLUDES = -I../markdown -I../render_ext -I.

# Source files
SOURCES = editor.c editor_abi.c ../render_ext/render_ext.c
WASM_SOURCES = editor.c editor_abi.c ../render_ext/render_ext.c ../markdown/markdown.c ../markdown/json.c
HEADERS = editor.h editor_abi.h

# Output files
STATIC_LIB = libeditor.a
SHARED_LIB = libeditor.so
WASM_OUTPUT = editor.js
WASM_WASM = editor.wasm

# Debug flags
DEBUG_FLAGS = -DDEBUG_EDITOR=1 -DDEBUG_VERBOSE=1
RELEASE_FLAGS = -DDEBUG_EDITOR=0 -DNDEBUG

# WASM compilation flags
WASM_FLAGS = -std=c11 -O2 -s WASM=1 \
             -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' \
             -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 \
             -s EXPORT_NAME="EditorModule" --no-entry \
             -s EXPORTED_FUNCTIONS='["_malloc","_free","_editor_library_init","_editor_library_cleanup","_editor_get_version_string","_editor_parse_markdown","_editor_parse_markdown_simple","_editor_markdown_to_html","_editor_export_markdown","_editor_state_create","_editor_state_destroy","_editor_state_reset","_editor_state_input_char","_editor_state_input_string","_editor_state_backspace","_editor_state_delete","_editor_state_get_document","_editor_state_get_markdown","_editor_free_string","_editor_get_error_message","_editor_enable_debug_logging"]'

.PHONY: all clean static shared wasm test debug help

# Default target
all: static

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c editor.c -o editor.o
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c editor_abi.c -o editor_abi.o
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c ../render_ext/render_ext.c -o render_ext.o
	ar rcs $(STATIC_LIB) editor.o editor_abi.o render_ext.o
	@echo "✅ Static library built: $(STATIC_LIB)"

# Shared library  
shared: $(SHARED_LIB)

$(SHARED_LIB): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -fPIC -shared $(SOURCES) -o $(SHARED_LIB)
	@echo "✅ Shared library built: $(SHARED_LIB)"

# WebAssembly module
wasm: $(WASM_OUTPUT)

$(WASM_OUTPUT): $(SOURCES) $(HEADERS)
	$(EMCC) $(WASM_FLAGS) $(RELEASE_FLAGS) $(INCLUDES) $(WASM_SOURCES) -o $(WASM_OUTPUT)
	@echo "✅ WebAssembly module built: $(WASM_OUTPUT)"

# Debug versions
debug-static:
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c editor.c -o editor_debug.o  
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c editor_abi.c -o editor_abi_debug.o
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c ../render_ext/render_ext.c -o render_ext_debug.o
	ar rcs libeditor_debug.a editor_debug.o editor_abi_debug.o render_ext_debug.o
	@echo "✅ Debug static library: libeditor_debug.a"

debug-wasm:
	$(EMCC) $(WASM_FLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(WASM_SOURCES) -o editor_debug.js
	@echo "✅ Debug WebAssembly: editor_debug.js"

# Test program
test: editor_test

editor_test: test.c $(STATIC_LIB)
	$(MAKE) -C ../markdown static
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) test.c -L. -leditor -L../markdown -lmarkdown -o editor_test
	@echo "✅ Test program built: editor_test"
	@echo "Run with: ./editor_test"

# Install headers and library
install: static
	mkdir -p /usr/local/include/editor
	mkdir -p /usr/local/lib
	cp $(HEADERS) /usr/local/include/editor/
	cp $(STATIC_LIB) /usr/local/lib/
	@echo "📦 Installed to system"

# Clean
clean:
	rm -f $(STATIC_LIB) $(SHARED_LIB) libeditor_debug.a
	rm -f $(WASM_OUTPUT) $(WASM_WASM) editor_debug.wasm.*
	rm -f *.o render_ext.o render_ext_debug.o editor_test
	@echo "🧹 Cleaned build artifacts"

# Help
help:
	@echo "Editor Engine Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Build static library (default)"
	@echo "  static      - Build static library (.a)"  
	@echo "  shared      - Build shared library (.so)"
	@echo "  wasm        - Build WebAssembly module"
	@echo "  test        - Build and run test program"
	@echo "  debug-*     - Debug versions with verbose output"
	@echo "  install     - Install to system"
	@echo "  clean       - Remove build artifacts"
	@echo "  help        - Show this help"
	@echo ""
	@echo "Usage:"
	@echo "  make            # Build static library"
	@echo "  make wasm       # For web integration"  
	@echo "  make test       # Test the engine"
