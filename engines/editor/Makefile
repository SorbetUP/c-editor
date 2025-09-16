# Makefile for Editor Engine
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g
EMCC = emcc

# Include paths  
INCLUDES = -I../markdown -I.

# Source files
SOURCES = editor.c editor_abi.c
HEADERS = editor.h editor_abi.h

# Output files
STATIC_LIB = libeditor.a
SHARED_LIB = libeditor.so
WASM_OUTPUT = editor.wasm.js
WASM_WASM = editor.wasm

# Debug flags
DEBUG_FLAGS = -DDEBUG_EDITOR=1 -DDEBUG_VERBOSE=1
RELEASE_FLAGS = -DDEBUG_EDITOR=0 -DNDEBUG

# WASM compilation flags
WASM_FLAGS = -std=c11 -O2 -s WASM=1 \
             -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' \
             -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 \
             -s EXPORT_NAME="EditorModule" --no-entry \
             -s EXPORTED_FUNCTIONS='["_malloc","_free","_editor_create","_editor_destroy","_editor_insert_text","_editor_delete_text","_editor_get_content","_editor_set_content","_editor_undo","_editor_redo"]'

.PHONY: all clean static shared wasm test debug help

# Default target
all: static

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c editor.c -o editor.o
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c editor_abi.c -o editor_abi.o
	ar rcs $(STATIC_LIB) editor.o editor_abi.o
	@echo "✅ Static library built: $(STATIC_LIB)"

# Shared library  
shared: $(SHARED_LIB)

$(SHARED_LIB): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -fPIC -shared $(SOURCES) -o $(SHARED_LIB)
	@echo "✅ Shared library built: $(SHARED_LIB)"

# WebAssembly module
wasm: $(WASM_OUTPUT)

$(WASM_OUTPUT): $(SOURCES) $(HEADERS)
	$(EMCC) $(WASM_FLAGS) $(RELEASE_FLAGS) $(INCLUDES) $(SOURCES) -o $(WASM_OUTPUT)
	@echo "✅ WebAssembly module built: $(WASM_OUTPUT)"

# Debug versions
debug-static:
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c editor.c -o editor_debug.o  
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c editor_abi.c -o editor_abi_debug.o
	ar rcs libeditor_debug.a editor_debug.o editor_abi_debug.o
	@echo "✅ Debug static library: libeditor_debug.a"

debug-wasm:
	$(EMCC) $(WASM_FLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(SOURCES) -o editor_debug.wasm.js
	@echo "✅ Debug WebAssembly: editor_debug.wasm.js"

# Test program
test: editor_test

editor_test: test.c $(STATIC_LIB)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) test.c -L. -leditor -o editor_test
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
	rm -f *.o editor_test
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