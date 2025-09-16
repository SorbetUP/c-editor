CC = clang
CFLAGS = -std=c11 -Wall -Wextra -O2 -fPIC
LDFLAGS = -shared

# Directories
SRC_DIR = .
BUILD_DIR = build
INCLUDE_DIR = .

# Source files
SOURCES = file_manager.c
PRO_SOURCES = file_manager.c professional_file_manager.c
OBJECTS = $(SOURCES:%.c=$(BUILD_DIR)/%.o)
PRO_OBJECTS = $(PRO_SOURCES:%.c=$(BUILD_DIR)/%.o)
HEADERS = file_manager.h professional_file_manager.h

# Library names
LIB_NAME = libfile_manager
PRO_LIB_NAME = libprofessional_file_manager
STATIC_LIB = $(BUILD_DIR)/$(LIB_NAME).a
SHARED_LIB = $(BUILD_DIR)/$(LIB_NAME).so
PRO_STATIC_LIB = $(BUILD_DIR)/$(PRO_LIB_NAME).a
PRO_SHARED_LIB = $(BUILD_DIR)/$(PRO_LIB_NAME).so

# Professional features require OpenSSL
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl 2>/dev/null || echo "-I/opt/homebrew/include")
OPENSSL_LIBS = $(shell pkg-config --libs openssl 2>/dev/null || echo "-L/opt/homebrew/lib -lssl -lcrypto")
PRO_CFLAGS = $(CFLAGS) $(OPENSSL_CFLAGS)
PRO_LIBS = $(OPENSSL_LIBS) -lpthread

# Test executable
TEST_SRC = test_file_manager.c
TEST_BIN = $(BUILD_DIR)/test_file_manager

.PHONY: all clean test install static shared professional

all: static professional

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

# Professional libraries
professional: $(PRO_STATIC_LIB)

$(PRO_STATIC_LIB): $(PRO_OBJECTS)
	ar rcs $@ $(PRO_OBJECTS)

$(PRO_SHARED_LIB): $(PRO_OBJECTS)
	$(CC) $(LDFLAGS) -o $@ $(PRO_OBJECTS) $(PRO_LIBS)

# Professional object files (with OpenSSL)
$(BUILD_DIR)/professional_file_manager.o: professional_file_manager.c $(HEADERS) | $(BUILD_DIR)
	$(CC) $(PRO_CFLAGS) -I$(INCLUDE_DIR) -c $< -o $@

# Test executable
test: $(TEST_BIN)

$(TEST_BIN): $(TEST_SRC) $(STATIC_LIB) | $(BUILD_DIR)
	$(CC) $(CFLAGS) -I$(INCLUDE_DIR) -o $@ $< -L$(BUILD_DIR) -lfile_manager

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
	@echo "  all      - Build static and shared libraries"
	@echo "  static   - Build static library only"
	@echo "  shared   - Build shared library only"
	@echo "  test     - Build test executable"
	@echo "  check    - Run tests"
	@echo "  clean    - Remove build files"
	@echo "  install  - Install libraries and headers"
	@echo "  debug    - Build with debug symbols"
	@echo "  help     - Show this help"