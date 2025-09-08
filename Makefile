CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -Werror -pedantic -g
SRCDIR = src
TESTDIR = tests
OBJDIR = obj

SOURCES = $(wildcard $(SRCDIR)/*.c)
OBJECTS = $(SOURCES:$(SRCDIR)/%.c=$(OBJDIR)/%.o)
HEADERS = $(wildcard $(SRCDIR)/*.h)

TEST_SOURCES = $(wildcard $(TESTDIR)/*.c)
TEST_OBJECTS = $(TEST_SOURCES:$(TESTDIR)/%.c=$(OBJDIR)/test_%.o)

LIBRARY = libeditor.a
TEST_EXECUTABLE = run_tests

.PHONY: all clean test debug sanitize

all: $(LIBRARY)

$(LIBRARY): $(OBJECTS) | $(OBJDIR)
	ar rcs $@ $^
	ranlib $@

$(OBJDIR)/%.o: $(SRCDIR)/%.c $(HEADERS) | $(OBJDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJDIR)/test_%.o: $(TESTDIR)/%.c $(HEADERS) | $(OBJDIR)
	$(CC) $(CFLAGS) -I$(SRCDIR) -c $< -o $@

$(OBJDIR):
	mkdir -p $(OBJDIR)

$(TEST_EXECUTABLE): $(TEST_OBJECTS) $(LIBRARY)
	$(CC) $(CFLAGS) $^ -o $@ -lm

test: $(TEST_EXECUTABLE)
	./$(TEST_EXECUTABLE)

debug: CFLAGS += -DDEBUG -O0
debug: $(LIBRARY) $(TEST_EXECUTABLE)

sanitize: CFLAGS += -fsanitize=address -fsanitize=undefined -fno-omit-frame-pointer
sanitize: $(LIBRARY) $(TEST_EXECUTABLE)

valgrind: $(TEST_EXECUTABLE)
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./$(TEST_EXECUTABLE)

format:
	clang-format -i $(SRCDIR)/*.c $(SRCDIR)/*.h $(TESTDIR)/*.c

clean:
	rm -rf $(OBJDIR) $(LIBRARY) $(TEST_EXECUTABLE)

install: $(LIBRARY)
	cp $(LIBRARY) /usr/local/lib/
	cp $(SRCDIR)/*.h /usr/local/include/

uninstall:
	rm -f /usr/local/lib/$(LIBRARY)
	rm -f /usr/local/include/editor.h /usr/local/include/json.h /usr/local/include/markdown.h

help:
	@echo "Available targets:"
	@echo "  all       - Build the library (default)"
	@echo "  test      - Build and run tests"
	@echo "  debug     - Build with debug flags"
	@echo "  sanitize  - Build with address/undefined sanitizers"
	@echo "  valgrind  - Run tests with valgrind"
	@echo "  format    - Format code with clang-format"
	@echo "  clean     - Remove build files"
	@echo "  install   - Install library and headers"
	@echo "  uninstall - Remove installed files"
	@echo "  help      - Show this help message"