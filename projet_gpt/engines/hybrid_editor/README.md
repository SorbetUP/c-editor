# Hybrid Editor Core C Library

A platform-agnostic C library for implementing hybrid markdown editors. This library provides core functionality that can be used across different platforms (macOS, iOS, Android, Windows, Linux, WASM).

## Overview

The hybrid editor core library enables building markdown editors with a "hybrid" editing experience:
- **Current line**: Shows raw markdown syntax for editing
- **Other lines**: Shows rendered output with markdown characters hidden/styled

## Features

- ✅ **Platform-agnostic C core** - Works on all platforms
- ✅ **Markdown format detection** - Detects headers, bold, italic, highlight, lists
- ✅ **Line-based parsing** - Efficient text parsing into lines
- ✅ **Cursor position tracking** - Maps cursor position to line numbers
- ✅ **Markup character hiding** - Finds ranges of markdown characters to hide
- ✅ **HTML integration** - Works with existing `editor_markdown_to_html()` 
- ✅ **Configurable** - Enable/disable features per platform needs
- ✅ **Memory safe** - Proper allocation/deallocation
- ✅ **Well tested** - Comprehensive test suite

## API Overview

### Core Functions

```c
// Text parsing
TextLines* hybrid_parse_text(const char* text);
int hybrid_get_line_at_cursor(const TextLines* lines, int cursor_pos);
LineInfo* hybrid_get_line_info(const TextLines* lines, int line_index);

// Format detection  
MarkdownFormat hybrid_detect_line_format(const char* line);
LineFormats* hybrid_analyze_markdown_line(const char* line);

// Text manipulation
char* hybrid_strip_markdown_markup(const char* line);
TextRange* hybrid_find_markup_ranges(const char* line, int* range_count);

// Memory management
void hybrid_free_text_lines(TextLines* lines);
void hybrid_free_line_formats(LineFormats* formats);
```

### Markdown Format Types

```c
typedef enum {
    MD_FORMAT_NONE = 0,
    MD_FORMAT_BOLD = 1,
    MD_FORMAT_ITALIC = 2, 
    MD_FORMAT_HIGHLIGHT = 4,
    MD_FORMAT_HEADER1 = 8,
    MD_FORMAT_HEADER2 = 16,
    MD_FORMAT_HEADER3 = 32,
    MD_FORMAT_LIST = 64
} MarkdownFormat;
```

## Building

```bash
# Build static and shared libraries
make all

# Run tests
make check

# Build for WASM
make wasm

# Install system-wide
sudo make install
```

## Usage Example

```c
#include "hybrid_editor_core.h"

// Parse text into lines
const char* text = "# Header\n**Bold** text\n*Italic*";
TextLines* lines = hybrid_parse_text(text);

// Get current line from cursor
int current_line = hybrid_get_line_at_cursor(lines, cursor_pos);

// Analyze markdown formatting
char* line_content = hybrid_get_line_content(text, 1);
MarkdownFormat format = hybrid_detect_line_format(line_content);

if (format & MD_FORMAT_BOLD) {
    // Apply bold styling
    LineFormats* details = hybrid_analyze_markdown_line(line_content);
    // Use details->formats to get exact ranges
    hybrid_free_line_formats(details);
}

// Clean up
free(line_content);
hybrid_free_text_lines(lines);
```

## Integration Examples

### macOS/iOS (Objective-C)
See `example_integration.m` - Shows how to integrate with NSTextView for a complete hybrid editor.

### WASM/JavaScript
```javascript
// Load WASM module
const lines = Module._hybrid_parse_text(textPtr);
const currentLine = Module._hybrid_get_line_at_cursor(lines, cursorPos);
const format = Module._hybrid_detect_line_format(linePtr);
```

### Android (JNI)
```java
// Native method declarations
public native long parseText(String text);
public native int getLineAtCursor(long lines, int cursor);
public native int detectLineFormat(String line);
```

## Platform-Specific Benefits

- **Flutter**: Single C library shared between iOS/Android
- **React Native**: WASM version for cross-platform consistency  
- **Electron**: Native performance with WASM fallback
- **Mobile**: Optimized memory usage and battery efficiency
- **Desktop**: Maximum performance with native compilation

## Configuration

```c
HybridConfig config = {
    .enable_bold = true,
    .enable_italic = true, 
    .enable_highlight = true,
    .enable_headers = true,
    .enable_lists = true,
    .strict_markdown = true,
    .max_line_length = 10000
};
hybrid_set_config(&config);
```

## Testing

The library includes comprehensive tests covering:
- Text parsing edge cases
- Cursor position mapping
- Markdown format detection
- Memory management
- Platform-specific scenarios

Run tests with: `make check`

## License

Part of the c-editor project. See main repository for license information.

## Contributing

This library is designed to be the foundation for hybrid markdown editors across all platforms. Contributions should maintain platform-agnosticism and focus on core functionality.