# Cursor Management Engine

Advanced cursor positioning and management for markdown editors with intelligent formatting preservation.

## 🎯 Features

- **Smart positioning** - Avoids breaking formatting markers
- **Context awareness** - Knows when cursor is inside bold, italic, etc.
- **Intelligent Enter key** - Preserves formatting across line breaks
- **Smart line merging** - Reconnects split formatting (e.g. "**Gr" + "as**" → "**Gras**")
- **WebAssembly exports** - Use from JavaScript/web interfaces

## 🏗️ Architecture

```
cursor/
├── cursor_manager.c    # Core cursor management algorithms
├── cursor_manager.h    # Public API & data structures
├── cursor_wasm.c       # WebAssembly bindings
├── Makefile            # Build system
└── README.md          # This file
```

## 🔧 API Overview

### Core Functions

```c
// Analyze formatting context at cursor position
formatting_context_t cursor_analyze_formatting(const char* content, int position);

// Adjust cursor position to avoid breaking formatting
cursor_position_t cursor_adjust_for_formatting(int position, const char* content, bool is_markdown_mode);

// Handle Enter key with smart formatting preservation
cursor_operation_result_t cursor_handle_enter_key(int position, const char* content, bool is_markdown_mode);

// Merge two lines with intelligent formatting reconnection
cursor_operation_result_t cursor_merge_lines(const char* line1, const char* line2, bool add_space);
```

### Data Structures

```c
typedef enum {
    MARKER_NONE = 0,
    MARKER_BOLD,        // **text**
    MARKER_ITALIC,      // *text*
    MARKER_HIGHLIGHT,   // ==text==
    MARKER_UNDERLINE,   // ++text++
    MARKER_HEADER       // # text
} formatting_marker_t;

typedef struct {
    formatting_marker_t type;
    int start_pos;
    int end_pos;
    int marker_length;
    bool inside_marker;
} formatting_context_t;
```

## 🚀 Quick Start

### Build Library
```bash
make
```

### Build with WebAssembly
```bash
make wasm
```

### Run Tests
```bash
make test
```

## 🧪 Testing

### Unit Tests
```bash
./cursor_test_demo
```

### Interactive TUI Testing
```bash
cd ../../tools/tui && make run
```

## 📊 Examples

### Basic Usage
```c
#include "cursor_manager.h"

const char* text = "**Bold text**";
int cursor_pos = 4;  // Inside "Bold"

// Analyze context
formatting_context_t ctx = cursor_analyze_formatting(text, cursor_pos);
if (ctx.inside_marker && ctx.type == MARKER_BOLD) {
    printf("Cursor is inside bold formatting\n");
}

// Handle Enter key
cursor_operation_result_t result = cursor_handle_enter_key(cursor_pos, text, true);
if (result.success) {
    printf("Before: %s\n", result.before_cursor);  // "**Bold"
    printf("After: %s\n", result.after_cursor);    // " text**"
}
```

### Smart Line Merging
```c
// Reconnect split formatting
cursor_operation_result_t result = cursor_merge_lines("**Gr", "as**", false);
if (result.success) {
    printf("Merged: %s\n", result.before_cursor);  // "**Gras**"
    printf("Cursor at: %d\n", result.new_position.position);  // 4
}
```

## 🔗 Integration

### JavaScript/Web
```javascript
// WebAssembly module loaded as CursorModule
const manager = new CCursorManager(CursorModule);

// Check if cursor is inside formatting
const isInside = manager.isInsideFormatting("**text**", 3);

// Smart Enter key handling  
const result = manager.handleEnterKey(4, "**Bold text**");
```

### Flutter/Dart
```dart
// FFI bindings to C library
final result = CursorManager.handleEnterKey(position, content);
```

## 🎨 Algorithm Details

### Smart Enter Key
1. Analyze formatting context at cursor position
2. If inside formatting, intelligently split preserving markers
3. Handle list prefixes and indentation
4. Position cursor optimally on new line

### Intelligent Line Merging  
1. Detect if lines contain split formatting
2. Reconstruct formatting markers (e.g. "**Gr" + "as**" → "**Gras**")
3. Handle multiple formatting types (bold, italic, highlight)
4. Position cursor at merge point

### Context Analysis
1. Scan text for formatting markers
2. Determine marker boundaries and nesting
3. Calculate cursor position relative to markers
4. Provide rich context information

## 📈 Performance

- **Zero-copy analysis** - Works directly on input strings
- **Minimal allocations** - Only for result structures
- **O(n) complexity** - Linear scan of content
- **Cache-friendly** - Sequential memory access patterns

## 🔧 Debug Mode

Enable detailed logging:
```bash
make debug
```

Debug output shows:
- Formatting marker detection
- Position adjustment decisions  
- Memory allocation/deallocation
- Algorithm execution traces

## 🧬 WebAssembly Integration

The cursor engine compiles to WebAssembly for web integration:

```bash
# Compile to WASM
emcc cursor_manager.c cursor_wasm.c -o cursor.wasm.js \
     -s EXPORTED_FUNCTIONS='["_cursor_wasm_analyze"]' \
     -s MODULARIZE=1
```

## 🤝 Contributing

1. Add new formatting types to `formatting_marker_t`
2. Implement detection in `cursor_analyze_formatting()`  
3. Add handling in `cursor_handle_enter_key()` and `cursor_merge_lines()`
4. Write tests and update documentation

## 📜 License

Part of the C Editor project.