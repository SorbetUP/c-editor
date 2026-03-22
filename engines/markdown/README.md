# Markdown Engine

High-performance markdown parser and renderer with JSON intermediate representation.

## 🚀 Features

- **Fast parsing** - Optimized C implementation
- **Rich formatting** - Bold, italic, headers, lists, tables, code blocks
- **JSON output** - Structured intermediate representation
- **HTML rendering** - Clean, semantic HTML generation
- **WebAssembly exports** - Use from JavaScript
- **Memory efficient** - Minimal allocations, explicit memory management

## 🏗️ Files

```
markdown/
├── markdown.c          # Core parsing & rendering
├── markdown.h          # Public API
├── json.c              # JSON generation utilities  
├── json.h              # JSON API
├── Makefile            # Build system
└── README.md          # This file
```

## 🔧 API Overview

### Core Functions

```c
// Parse markdown to JSON representation
char* markdown_to_json(const char* markdown_text);

// Render JSON to HTML
char* json_to_html(const char* json_text);

// Direct markdown to HTML (convenience)
char* markdown_to_html(const char* markdown_text);

// Memory management
void markdown_free_result(char* result);
```

### Supported Markdown

- **Headers**: `# H1`, `## H2`, etc.
- **Formatting**: `**bold**`, `*italic*`, `==highlight==`, `++underline++`
- **Lists**: `- item`, `1. numbered`
- **Code**: `` `inline` ``, ``` blocks ```
- **Tables**: `| col1 | col2 |`
- **Links**: `[text](url)`
- **Images**: `![alt](src)`

## 📊 JSON Structure

The engine produces structured JSON:

```json
{
  "name": "document",
  "meta": {
    "default": {
      "fontsize": 11,
      "font": "Helvetica",
      "text_color": [0,0,0,1],
      "highlight_color": [1,1,0,0.3]
    }
  },
  "elements": [
    {
      "type": "text",
      "text": "Hello World",
      "bold": true,
      "spans": [
        {"text": "Hello", "bold": true},
        {"text": " World", "bold": true}
      ]
    }
  ]
}
```

## 🚀 Quick Start

### Build
```bash
make
```

### Basic Usage
```c
#include "markdown.h"

const char* markdown = "# Hello\n**Bold text**";

// Parse to JSON
char* json = markdown_to_json(markdown);
printf("JSON: %s\n", json);

// Render to HTML
char* html = json_to_html(json);  
printf("HTML: %s\n", html);

// Cleanup
markdown_free_result(json);
markdown_free_result(html);
```

### WebAssembly Usage
```javascript
// After loading WASM module
const json = Module.ccall('markdown_to_json', 'string', ['string'], [markdown]);
const html = Module.ccall('json_to_html', 'string', ['string'], [json]);
Module.ccall('markdown_free_result', 'void', ['string'], [json]);
Module.ccall('markdown_free_result', 'void', ['string'], [html]);
```

## 🧪 Testing

### Unit Tests
```bash
make test
```

### Benchmarks  
```bash
make benchmark
```

## 📈 Performance Characteristics

- **Linear parsing** - O(n) time complexity
- **Minimal memory** - ~2x input size peak memory
- **Zero-copy spans** - Text spans reference original input
- **Fast tables** - Optimized table parsing
- **Streaming friendly** - Processes input incrementally

## 🔧 Configuration

### Debug Mode
```bash
make debug
```

### Memory Debugging
```bash  
make valgrind
```

## 🏗️ Architecture

### Two-Stage Process
1. **Parse** markdown → JSON (structured representation)
2. **Render** JSON → HTML (or other formats)

### Benefits
- **Flexibility** - JSON can be rendered to multiple formats
- **Inspection** - Intermediate representation is human-readable  
- **Caching** - JSON can be cached between renders
- **Editing** - JSON can be modified programmatically

### Parser Design
- **Recursive descent** - Clean, maintainable parsing logic
- **Span-based** - Tracks text formatting spans precisely
- **Memory pools** - Efficient allocation patterns
- **Error recovery** - Graceful handling of malformed input

## 🎨 Rendering Features

### HTML Output
- **Semantic markup** - Uses proper HTML5 elements
- **CSS classes** - Consistent class names for styling
- **Accessibility** - ARIA attributes where appropriate
- **Clean output** - Well-formatted, readable HTML

### Span Processing
- **Nested formatting** - Handles overlapping styles correctly
- **Boundary detection** - Precise marker boundary tracking
- **Optimization** - Merges adjacent spans with same formatting

## 🔗 Integration

### Used By
- **Web editor** - Powers the web interface
- **TUI editor** - Provides markdown rendering in terminal
- **Flutter app** - Native markdown processing
- **Testing tools** - Validates markdown processing

### Dependencies
- **json.c** - JSON generation utilities
- **Standard C library** - No external dependencies

## 🐛 Debug Features

### Verbose Logging
```c
#define DEBUG_MARKDOWN 1
```

Shows:
- Parse tree construction
- Span boundary calculations  
- Memory allocations
- Rendering decisions

### Memory Tracking
- Allocation/deallocation logging
- Leak detection
- Usage statistics

## 🤝 Contributing

1. **Add new syntax** - Extend parser for new markdown features
2. **Optimize performance** - Profile and improve hot paths
3. **Output formats** - Add renderers for other formats (PDF, etc.)
4. **Testing** - Add test cases for edge cases
5. **Documentation** - Improve API documentation

## 📜 License

Part of the C Editor project.