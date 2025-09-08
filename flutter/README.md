# Flutter UI Multiplatform Architecture

This directory contains the Flutter UI implementation for the C Editor with multiplatform support.

## Architecture Overview

### Core Components

1. **C Core Library** (`../src/`)
   - Markdown parsing and JSON serialization
   - Memory-safe C11 implementation
   - Property-tested and fuzz-tested

2. **FFI Bridge** (`lib/ffi/`)
   - Native bindings for mobile/desktop platforms
   - Direct C library integration via dart:ffi
   - Platform: iOS, Android, macOS, Windows, Linux

3. **WASM Bridge** (`lib/wasm/`)
   - Web-compatible interface
   - Compiled C library to WebAssembly
   - Platform: Web browsers

4. **Unified API** (`lib/editor_api.dart`)
   - Abstract interface hiding platform differences
   - Consistent API across all platforms
   - Automatic platform detection and routing

## Directory Structure

```
flutter/
├── lib/
│   ├── editor_api.dart          # Unified cross-platform API
│   ├── ffi/
│   │   ├── editor_ffi.dart      # FFI bindings
│   │   └── generated/           # Generated FFI bindings
│   ├── wasm/
│   │   ├── editor_wasm.dart     # WASM interface
│   │   └── js_interop.dart      # JavaScript interop
│   ├── widgets/
│   │   ├── editor_widget.dart   # Main editor widget
│   │   ├── toolbar.dart         # Formatting toolbar
│   │   └── preview.dart         # Live preview pane
│   └── models/
│       ├── document.dart        # Document data model
│       └── text_span.dart       # Text span model
├── web/
│   ├── editor_wasm.js          # WASM loader
│   └── editor.wasm             # Compiled C library
├── ios/
│   └── libeditor.a             # iOS static library
├── android/
│   └── src/main/jniLibs/       # Android native libraries
├── macos/
│   └── libeditor.dylib         # macOS dynamic library
├── windows/
│   └── editor.dll              # Windows DLL
└── linux/
    └── libeditor.so            # Linux shared library
```

## Platform-Specific Implementation

### Native Platforms (FFI)
- **Mobile**: iOS, Android
- **Desktop**: macOS, Windows, Linux
- **Technology**: dart:ffi with platform-specific native libraries
- **Benefits**: Maximum performance, full C library access

### Web Platform (WASM)
- **Platform**: Web browsers
- **Technology**: WebAssembly + JavaScript interop
- **Benefits**: No plugin required, runs in any modern browser

## API Design

### Unified Editor API
```dart
abstract class EditorApi {
  Future<void> initialize();
  Future<Document> parseMarkdown(String markdown);
  Future<String> exportToMarkdown(Document doc);
  Future<String> exportToJson(Document doc);
  void dispose();
}

class EditorApiFactory {
  static EditorApi create() {
    if (kIsWeb) {
      return WasmEditorApi();
    } else {
      return FfiEditorApi();
    }
  }
}
```

### Document Model
```dart
class Document {
  final List<Element> elements;
  Document(this.elements);
}

abstract class Element {
  ElementType get type;
}

class TextElement extends Element {
  final List<TextSpan> spans;
  final int level; // Header level (0 = normal text)
  TextElement(this.spans, {this.level = 0});
}

class TextSpan {
  final String text;
  final bool bold;
  final bool italic;
  final bool highlight;
  final bool underline;
  final Color? highlightColor;
  final Color? underlineColor;
  
  TextSpan({
    required this.text,
    this.bold = false,
    this.italic = false,
    this.highlight = false,
    this.underline = false,
    this.highlightColor,
    this.underlineColor,
  });
}
```

## Implementation Plan

### Phase 1: Project Setup
1. Create Flutter project with multiplatform support
2. Setup build scripts for native libraries
3. Configure WASM compilation pipeline
4. Setup platform-specific build configurations

### Phase 2: FFI Implementation
1. Generate FFI bindings from C headers
2. Implement FfiEditorApi class
3. Create platform-specific library loading
4. Test on mobile and desktop platforms

### Phase 3: WASM Implementation  
1. Compile C library to WebAssembly
2. Create JavaScript interop layer
3. Implement WasmEditorApi class
4. Test web deployment

### Phase 4: UI Implementation
1. Create editor widget with syntax highlighting
2. Implement formatting toolbar
3. Add live preview functionality
4. Handle platform-specific input methods

### Phase 5: Testing & Polish
1. Cross-platform testing suite
2. Performance optimization
3. Error handling and recovery
4. Documentation and examples

## Build Requirements

### For Native Platforms
- C11-compatible compiler (gcc/clang)
- Platform-specific build tools
- Flutter SDK with desktop/mobile support

### For Web Platform
- Emscripten for WASM compilation
- Web-compatible Flutter build
- Modern browser with WASM support

## Performance Considerations

1. **Memory Management**: Proper cleanup of native resources
2. **Async Operations**: Non-blocking UI during parsing
3. **Incremental Updates**: Efficient text editing updates
4. **Platform Optimization**: Leverage native performance where available

## Security Considerations

1. **Input Validation**: Sanitize all user input before C library calls
2. **Memory Safety**: Use AddressSanitizer in debug builds
3. **Sandboxing**: Web platform naturally sandboxed via WASM
4. **Error Handling**: Graceful degradation on parsing errors