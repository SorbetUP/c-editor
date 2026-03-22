# Makefile for Cursor Management Engine
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g
EMCC = emcc

# Source files
SOURCES = cursor_manager.c cursor_wasm.c
HEADERS = cursor_manager.h

# Output files
STATIC_LIB = libcursor.a
SHARED_LIB = libcursor.so
WASM_OUTPUT = cursor.wasm.js
WASM_WASM = cursor.wasm

# Debug flags
DEBUG_FLAGS = -DDEBUG_CURSOR=1 -DDEBUG_VERBOSE=1
RELEASE_FLAGS = -DDEBUG_CURSOR=0 -DNDEBUG

# WASM compilation flags
WASM_FLAGS = -std=c11 -O2 -s WASM=1 \
             -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' \
             -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 \
             -s EXPORT_NAME="CursorModule" --no-entry \
             -s EXPORTED_FUNCTIONS='["_malloc","_free","_cursor_wasm_html_to_markdown","_cursor_wasm_adjust_for_formatting","_cursor_wasm_is_inside_formatting","_cursor_wasm_get_formatting_type","_cursor_wasm_handle_enter_key","_cursor_wasm_split_line","_cursor_wasm_merge_lines","_cursor_wasm_validate_position","_cursor_wasm_find_safe_position","_cursor_wasm_free","_cursor_wasm_debug"]'

.PHONY: all clean static shared wasm test debug help

# Default target
all: static

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): cursor_manager.c $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) -c cursor_manager.c -o cursor_manager.o
	ar rcs $(STATIC_LIB) cursor_manager.o
	@echo "✅ Static library built: $(STATIC_LIB)"

# Shared library  
shared: $(SHARED_LIB)

$(SHARED_LIB): cursor_manager.c $(HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) -fPIC -shared cursor_manager.c -o $(SHARED_LIB)
	@echo "✅ Shared library built: $(SHARED_LIB)"

# WebAssembly module
wasm: $(WASM_OUTPUT)

$(WASM_OUTPUT): $(SOURCES) $(HEADERS)
	$(EMCC) $(WASM_FLAGS) $(RELEASE_FLAGS) $(SOURCES) -o $(WASM_OUTPUT)
	@echo "✅ WebAssembly module built: $(WASM_OUTPUT)"

# Debug versions
debug-static:
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) -c cursor_manager.c -o cursor_manager_debug.o  
	ar rcs libcursor_debug.a cursor_manager_debug.o
	@echo "✅ Debug static library: libcursor_debug.a"

debug-wasm:
	$(EMCC) $(WASM_FLAGS) $(DEBUG_FLAGS) $(SOURCES) -o cursor_debug.wasm.js
	@echo "✅ Debug WebAssembly: cursor_debug.wasm.js"

# Test program
test: cursor_test_demo

cursor_test_demo: cursor_test_demo.c $(STATIC_LIB)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) cursor_test_demo.c -L. -lcursor -o cursor_test_demo
	@echo "✅ Test program built: cursor_test_demo"
	@echo "Run with: ./cursor_test_demo"

# Benchmark  
benchmark: cursor_manager.c benchmark.c
	$(CC) $(CFLAGS) -O3 -DNDEBUG -DBENCHMARK cursor_manager.c benchmark.c -o benchmark
	@echo "🚀 Benchmark built: ./benchmark"

# Install headers and library
install: static
	mkdir -p /usr/local/include/cursor
	mkdir -p /usr/local/lib
	cp $(HEADERS) /usr/local/include/cursor/
	cp $(STATIC_LIB) /usr/local/lib/
	@echo "📦 Installed to system"

# Clean
clean:
	rm -f $(STATIC_LIB) $(SHARED_LIB) libcursor_debug.a
	rm -f $(WASM_OUTPUT) $(WASM_WASM) cursor_debug.wasm.*
	rm -f *.o cursor_test_demo benchmark
	@echo "🧹 Cleaned build artifacts"

# Help
help:
	@echo "Cursor Management Engine Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Build static library (default)"
	@echo "  static      - Build static library (.a)"  
	@echo "  shared      - Build shared library (.so)"
	@echo "  wasm        - Build WebAssembly module"
	@echo "  test        - Build and run test program"
	@echo "  debug-*     - Debug versions with verbose output"
	@echo "  benchmark   - Build performance benchmarks"
	@echo "  install     - Install to system"
	@echo "  clean       - Remove build artifacts"
	@echo "  help        - Show this help"
	@echo ""
	@echo "Usage:"
	@echo "  make            # Build static library"
	@echo "  make wasm       # For web integration"  
	@echo "  make test       # Test the engine"