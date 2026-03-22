CC = clang
CFLAGS = -std=c11 -Wall -Wextra -O2 -fPIC
LDFLAGS = -shared

# Directories
SRC_DIR = .
BUILD_DIR = build
INCLUDE_DIR = .

# Source files
SOURCES = hybrid_editor_core.c
OBJECTS = $(SOURCES:%.c=$(BUILD_DIR)/%.o)
HEADERS = hybrid_editor_core.h

# Library name
LIB_NAME = libhybrid_editor
STATIC_LIB = $(BUILD_DIR)/$(LIB_NAME).a
SHARED_LIB = $(BUILD_DIR)/$(LIB_NAME).so

# Test executable
TEST_SRC = test_hybrid_editor.c
TEST_BIN = $(BUILD_DIR)/test_hybrid_editor

.PHONY: all clean test install static shared

all: static shared test

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Object files
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c $(HEADERS) | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(INCLUDE_DIR) -c $< -o $@

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): $(OBJECTS)
	ar rcs $@ $(OBJECTS)

# Shared library
shared: $(SHARED_LIB)

$(SHARED_LIB): $(OBJECTS)
	$(CC) $(LDFLAGS) -o $@ $(OBJECTS)

# Test executable
test: $(TEST_BIN)

$(TEST_BIN): $(TEST_SRC) $(STATIC_LIB) | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(INCLUDE_DIR) -o $@ $< -L$(BUILD_DIR) -lhybrid_editor

# Run tests
check: $(TEST_BIN)
	./$(TEST_BIN)

# Clean build files
clean:
	rm -rf $(BUILD_DIR)

# Install (basic install to /usr/local)
install: static shared
	mkdir -p /usr/local/lib
	mkdir -p /usr/local/include
	cp $(STATIC_LIB) /usr/local/lib/
	cp $(SHARED_LIB) /usr/local/lib/
	cp $(HEADERS) /usr/local/include/

# Debug build
debug: CFLAGS += -g -DDEBUG
debug: all

# WASM build
wasm: CC = emcc
wasm: CFLAGS = -std=c11 -O3 -s WASM=1 -s EXPORTED_FUNCTIONS='["_hybrid_parse_text","_hybrid_get_line_at_cursor","_hybrid_detect_line_format"]'
wasm: 
	$(CC) $(CFLAGS) -o $(BUILD_DIR)/hybrid_editor.js $(SOURCES)

# Help
help:
	@echo "Available targets:"
	@echo "  all      - Build static library, shared library, and tests"
	@echo "  static   - Build static library only"
	@echo "  shared   - Build shared library only"
	@echo "  test     - Build test executable"
	@echo "  check    - Run tests"
	@echo "  clean    - Remove build files"
	@echo "  install  - Install libraries and headers"
	@echo "  debug    - Build with debug symbols"
	@echo "  wasm     - Build WebAssembly version"
	@echo "  help     - Show this help"