# C Editor - Modular Markdown Editor with Advanced Cursor Management

A high-performance, modular markdown editor built in C with WebAssembly bindings, featuring intelligent cursor management and multiple interfaces.

## 🏗️ Architecture Overview

```
c-editor/
├── engines/          # Core C engines
│   ├── markdown/     # Markdown parsing & rendering
│   ├── cursor/       # Intelligent cursor management  
│   └── editor/       # Editor core & ABI
├── tools/           # Development & testing tools
│   ├── tui/         # Terminal UI editor
│   ├── debug/       # Debug utilities
│   └── build/       # Build systems & Makefiles
├── web/             # Web interfaces
│   ├── site/        # Web editor interface
│   └── wasm/        # WebAssembly bindings
├── flutter/         # Flutter mobile/desktop app
├── tests/           # Test suites
│   ├── unit/        # Unit tests
│   ├── integration/ # Integration tests  
│   └── fixtures/    # Test data
├── examples/        # Example files & demos
├── scripts/         # Automation scripts
└── docs/           # Documentation
```

## 🚀 Quick Start

### Build Core Engines
```bash
# Build markdown engine
cd engines/markdown && make

# Build cursor management
cd engines/cursor && make  

# Build editor core
cd engines/editor && make
```

### Try the TUI Editor
```bash
# Terminal-based editor with C cursor management
cd tools/tui && make run
```

### Launch Web Interface
```bash
# Web editor with WebAssembly backend
cd web/site && python3 -m http.server 8001
```

### Build Flutter App
```bash
cd flutter && flutter run
```

## 🔧 Core Engines

### 📝 Markdown Engine (`engines/markdown/`)
- Fast markdown parsing & rendering
- JSON intermediate representation
- Support for tables, lists, formatting
- WebAssembly exports

### 🎯 Cursor Management (`engines/cursor/`)
- **Intelligent positioning** - Avoids splitting formatting markers
- **Smart Enter key** - Preserves formatting across line breaks  
- **Smart line merging** - Reconnects split formatting
- **Context detection** - Knows when cursor is inside formatting
- **WebAssembly bindings** for web integration

### ⚙️ Editor Core (`engines/editor/`)
- Document management
- Undo/redo system
- File I/O operations
- Cross-platform ABI

## 🛠️ Development Tools

### 🖥️ TUI Editor (`tools/tui/`)
- Full-featured terminal editor
- Real-time cursor context display
- Direct testing of C cursor algorithms
- Scriptable for automated testing

### 🐛 Debug Tools (`tools/debug/`)
- Cursor position debugging
- Formatting analysis
- Memory leak detection
- Performance profiling

### 🔨 Build System (`tools/build/`)
- Modular Makefiles
- WebAssembly compilation
- Cross-platform builds
- Testing automation

## 🌐 Web Interfaces

### 🌐 Web Site (`web/site/`)
- Interactive markdown editor
- Real-time preview
- Hybrid edit/render modes
- WebAssembly-powered cursor management

### 🔗 WebAssembly (`web/wasm/`)
- C engine bindings
- JavaScript interfaces
- Memory management
- Error handling

## 📱 Flutter App (`flutter/`)
- Cross-platform mobile/desktop app
- Native performance with C engines
- Touch-optimized interface
- Synchronized with web editor

## 🧪 Testing

### Unit Tests (`tests/unit/`)
```bash
cd tests/unit && make test
```

### Integration Tests (`tests/integration/`)
```bash
cd tests/integration && ./run-all-tests.sh
```

### TUI Testing (`tools/tui/`)
```bash
cd tools/tui && make test
```

## 📖 Features

- **🎯 Advanced Cursor Management**: Smart positioning, formatting preservation
- **⚡ High Performance**: C core with WebAssembly bindings
- **🔧 Multi-Interface**: TUI, Web, Flutter mobile/desktop
- **🧪 Comprehensive Testing**: Unit, integration, and interactive testing
- **📱 Cross-Platform**: Linux, macOS, Windows, iOS, Android, Web
- **🔗 Modular Design**: Independently buildable components

## 🏆 Why This Architecture?

1. **Modularity**: Each engine can be developed and tested independently
2. **Performance**: C core for maximum speed
3. **Reusability**: Same engines power TUI, web, and mobile interfaces
4. **Testability**: Isolated components, comprehensive test suites
5. **Maintainability**: Clear separation of concerns
6. **Extensibility**: Easy to add new interfaces or engines

## 🤝 Contributing

1. Pick a module (engine, tool, interface)
2. Read the module-specific README
3. Build and test your changes
4. Submit focused PRs per module

## 📜 License

[Your License Here]

---

**Built with ❤️ and C** - High-performance markdown editing across all platforms