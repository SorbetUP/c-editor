# Makefile for Vault Manager
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -O2 -fPIC
INCLUDES = -I/opt/homebrew/include
LIBS = -L/opt/homebrew/lib -ljson-c

# Directories
BUILD_DIR = build
SRC_DIR = .

# Source files
SOURCES = vault_manager.c
OBJECTS = $(SOURCES:%.c=$(BUILD_DIR)/%.o)
HEADERS = vault_manager.h

# Library names
LIB_NAME = libvault_manager
STATIC_LIB = $(BUILD_DIR)/$(LIB_NAME).a
SHARED_LIB = $(BUILD_DIR)/$(LIB_NAME).so

# Test executable
TEST_SRC = test_vault_manager.c
TEST_BIN = $(BUILD_DIR)/test_vault_manager

.PHONY: all clean test install static shared

all: static

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Object files
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c $(HEADERS) | $(BUILD_DIR)
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

# Static library
static: $(STATIC_LIB)

$(STATIC_LIB): $(OBJECTS)
	ar rcs $@ $(OBJECTS)

# Shared library
shared: $(SHARED_LIB)

$(SHARED_LIB): $(OBJECTS)
	$(CC) -shared -o $@ $(OBJECTS) $(LIBS)

# Test executable
test: $(TEST_BIN)

$(TEST_BIN): $(TEST_SRC) $(STATIC_LIB) | $(BUILD_DIR)
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $< -L$(BUILD_DIR) -lvault_manager $(LIBS)

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

# Help
help:
	@echo "Available targets:"
	@echo "  all      - Build static library"
	@echo "  static   - Build static library only"
	@echo "  shared   - Build shared library only"
	@echo "  test     - Build test executable"
	@echo "  check    - Run tests"
	@echo "  clean    - Remove build files"
	@echo "  install  - Install libraries and headers"
	@echo "  debug    - Build with debug symbols"
	@echo "  help     - Show this help"