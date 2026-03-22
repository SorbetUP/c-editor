# Makefile for Markdown Engine
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g
EMCC = emcc

# Include paths
INCLUDES = -I../editor

# Source files
SOURCES = markdown.c json.c
HEADERS = markdown.h json.h

# Output files
STATIC_LIB = libmarkdown.a
SHARED_LIB = libmarkdown.so
WASM_OUTPUT = markdown.wasm.js
WASM_WASM = markdown.wasm

# Debug flags
DEBUG_FLAGS = -DDEBUG_MARKDOWN=1 -DDEBUG_VERBOSE=1
RELEASE_FLAGS = -DDEBUG_MARKDOWN=0 -DNDEBUG

# WASM compilation flags
WASM_FLAGS = -std=c11 -O2 -s WASM=1 \
             -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' \
             -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 \
             -s EXPORT_NAME="MarkdownModule" --no-entry \
             -s EXPORTED_FUNCTIONS='["_malloc","_free","_markdown_to_json","_json_to_markdown","_json_stringify","_json_parse","_parse_inline_styles"]'

.PHONY: all clean static shared wasm test debug help

# Default target
all: static

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c markdown.c -o markdown.o
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) -c json.c -o json.o
	ar rcs $(STATIC_LIB) markdown.o json.o
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
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c markdown.c -o markdown_debug.o  
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) -c json.c -o json_debug.o
	ar rcs libmarkdown_debug.a markdown_debug.o json_debug.o
	@echo "✅ Debug static library: libmarkdown_debug.a"

debug-wasm:
	$(EMCC) $(WASM_FLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(SOURCES) -o markdown_debug.wasm.js
	@echo "✅ Debug WebAssembly: markdown_debug.wasm.js"

# Test program
test: markdown_test

markdown_test: test.c $(STATIC_LIB)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) test.c -L. -lmarkdown -o markdown_test
	@echo "✅ Test program built: markdown_test"
	@echo "Run with: ./markdown_test"

# Install headers and library
install: static
	mkdir -p /usr/local/include/markdown
	mkdir -p /usr/local/lib
	cp $(HEADERS) /usr/local/include/markdown/
	cp $(STATIC_LIB) /usr/local/lib/
	@echo "📦 Installed to system"

# Clean
clean:
	rm -f $(STATIC_LIB) $(SHARED_LIB) libmarkdown_debug.a
	rm -f $(WASM_OUTPUT) $(WASM_WASM) markdown_debug.wasm.*
	rm -f *.o markdown_test
	@echo "🧹 Cleaned build artifacts"

# Help
help:
	@echo "Markdown Engine Build System"
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