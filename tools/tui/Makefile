# Makefile for TUI Editor Tools
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g

# Paths to engines
CURSOR_DIR = ../../engines/cursor
MARKDOWN_DIR = ../../engines/markdown

# Source files
TUI_SOURCES = tui_editor.c
SCRIPTABLE_SOURCES = scriptable_tui.c  
DEMO_SOURCES = cursor_test_demo.c

# Engine dependencies
CURSOR_SOURCES = $(CURSOR_DIR)/cursor_manager.c
ENGINE_HEADERS = $(CURSOR_DIR)/cursor_manager.h

# Output executables
TUI_EDITOR = tui_editor
SCRIPTABLE_TUI = scriptable_tui
CURSOR_DEMO = cursor_demo

# Debug flags
DEBUG_FLAGS = -DDEBUG_CURSOR=1 -DDEBUG_VERBOSE=1
RELEASE_FLAGS = -DDEBUG_CURSOR=0 -DNDEBUG

# Include paths
INCLUDES = -I$(CURSOR_DIR) -I$(MARKDOWN_DIR)

.PHONY: all clean run demo test debug help

# Default target
all: $(TUI_EDITOR) $(SCRIPTABLE_TUI) $(CURSOR_DEMO)

# Interactive TUI editor  
$(TUI_EDITOR): $(TUI_SOURCES) $(CURSOR_SOURCES) $(ENGINE_HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) $(TUI_SOURCES) $(CURSOR_SOURCES) -o $(TUI_EDITOR)
	@echo "✅ Interactive TUI editor built: $(TUI_EDITOR)"

# Scriptable TUI for testing
$(SCRIPTABLE_TUI): $(SCRIPTABLE_SOURCES) $(CURSOR_SOURCES) $(ENGINE_HEADERS)
	$(CC) $(CFLAGS) $(RELEASE_FLAGS) $(INCLUDES) $(SCRIPTABLE_SOURCES) $(CURSOR_SOURCES) -o $(SCRIPTABLE_TUI)  
	@echo "✅ Scriptable TUI built: $(SCRIPTABLE_TUI)"

# Cursor demo program
$(CURSOR_DEMO): $(DEMO_SOURCES) $(CURSOR_SOURCES) $(ENGINE_HEADERS)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(DEMO_SOURCES) $(CURSOR_SOURCES) -o $(CURSOR_DEMO)
	@echo "✅ Cursor demo built: $(CURSOR_DEMO)"

# Debug versions
debug: $(TUI_EDITOR)_debug $(SCRIPTABLE_TUI)_debug

$(TUI_EDITOR)_debug: $(TUI_SOURCES) $(CURSOR_SOURCES) $(ENGINE_HEADERS)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(TUI_SOURCES) $(CURSOR_SOURCES) -o $(TUI_EDITOR)_debug
	@echo "✅ Debug TUI editor: $(TUI_EDITOR)_debug"

$(SCRIPTABLE_TUI)_debug: $(SCRIPTABLE_SOURCES) $(CURSOR_SOURCES) $(ENGINE_HEADERS)
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) $(INCLUDES) $(SCRIPTABLE_SOURCES) $(CURSOR_SOURCES) -o $(SCRIPTABLE_TUI)_debug
	@echo "✅ Debug scriptable TUI: $(SCRIPTABLE_TUI)_debug"

# Run interactive editor
run: $(TUI_EDITOR)
	@echo "🚀 Starting TUI Editor..."
	@echo "   Controls: Arrows=move, Enter=split, Backspace=merge, Ctrl+Q=quit"
	@echo "   Status bar shows cursor context in real-time"
	@echo ""
	./$(TUI_EDITOR)

# Run demo/test scenarios  
demo: $(SCRIPTABLE_TUI)
	@echo "🎬 Running automated cursor management demo..."
	@echo ""
	./$(SCRIPTABLE_TUI)

# Run cursor function tests
test: $(CURSOR_DEMO)
	@echo "🧪 Testing cursor management functions..."
	@echo ""
	./$(CURSOR_DEMO)

# Run debug versions
debug-run: $(TUI_EDITOR)_debug
	@echo "🐛 Starting TUI Editor in debug mode..."
	./$(TUI_EDITOR)_debug

debug-demo: $(SCRIPTABLE_TUI)_debug  
	@echo "🐛 Running debug scriptable demo..."
	./$(SCRIPTABLE_TUI)_debug

# Memory testing with valgrind
valgrind: $(TUI_EDITOR)
	@echo "🔍 Running TUI editor under valgrind..."
	valgrind --leak-check=full --show-leak-kinds=all ./$(TUI_EDITOR)

valgrind-demo: $(SCRIPTABLE_TUI)
	@echo "🔍 Running scriptable demo under valgrind..."  
	valgrind --leak-check=full --show-leak-kinds=all ./$(SCRIPTABLE_TUI)

# Address sanitizer builds
asan: 
	$(CC) $(CFLAGS) $(DEBUG_FLAGS) -fsanitize=address -fno-omit-frame-pointer \
		$(INCLUDES) $(TUI_SOURCES) $(CURSOR_SOURCES) -o $(TUI_EDITOR)_asan
	@echo "✅ AddressSanitizer build: $(TUI_EDITOR)_asan"

# Performance profiling
profile:
	$(CC) $(CFLAGS) -pg -O2 $(INCLUDES) $(TUI_SOURCES) $(CURSOR_SOURCES) -o $(TUI_EDITOR)_prof
	@echo "🚀 Profiling build: $(TUI_EDITOR)_prof"
	@echo "Run and use 'gprof $(TUI_EDITOR)_prof gmon.out' to analyze"

# Benchmark cursor performance
benchmark: $(CURSOR_DEMO)
	@echo "📊 Benchmarking cursor management performance..."
	@for i in {1..1000}; do ./$(CURSOR_DEMO) > /dev/null; done
	@echo "Completed 1000 iterations"

# Clean all builds
clean:
	rm -f $(TUI_EDITOR) $(TUI_EDITOR)_debug $(TUI_EDITOR)_asan $(TUI_EDITOR)_prof
	rm -f $(SCRIPTABLE_TUI) $(SCRIPTABLE_TUI)_debug
	rm -f $(CURSOR_DEMO)
	rm -f gmon.out *.o core
	@echo "🧹 Cleaned all TUI build artifacts"

# Help
help:
	@echo "TUI Editor Tools Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build all TUI tools (default)"
	@echo "  run          - Build and run interactive TUI editor"
	@echo "  demo         - Build and run scriptable demo"
	@echo "  test         - Build and run cursor function tests"  
	@echo "  debug        - Build debug versions"
	@echo "  debug-run    - Run interactive editor in debug mode"
	@echo "  debug-demo   - Run scriptable demo in debug mode"
	@echo "  valgrind     - Run with memory checking"
	@echo "  asan         - Build with AddressSanitizer"
	@echo "  benchmark    - Performance benchmarking"
	@echo "  clean        - Remove build artifacts"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Quick Start:"
	@echo "  make run     - Interactive terminal editor"
	@echo "  make demo    - Automated test scenarios"
	@echo "  make test    - Function-level testing"
	@echo ""
	@echo "The TUI editor provides real-time cursor context in the status bar"
	@echo "and directly uses the C cursor management engine for smart editing."