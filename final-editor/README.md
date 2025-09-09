# Final Editor

A web-based WYSIWYG editor built with Flutter that integrates with a C core via WebAssembly for markdown and JSON processing.

## 🌟 Features

- **WYSIWYG Editor**: Direct visual editing with real-time rendering
- **Dual Format Support**: Import/export both Markdown (`.md`) and JSON (`.json`) files
- **Live Panels**: Side-by-side Markdown and JSON editing panels with real-time synchronization  
- **IndexedDB Persistence**: Automatic saving with crash recovery
- **GitHub Pages Ready**: Optimized for static web deployment
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Keyboard Shortcuts**: Full keyboard support for power users

## 🚀 Live Demo

The editor is automatically deployed to GitHub Pages: [c-editor.github.io/c-editor/](https://your-username.github.io/c-editor/)

## 🛠️ Architecture

### Core Components

- **WYSIWYG Editor** (`wysiwyg_editor.dart`): Visual document renderer with support for paragraphs, headings, lists, code blocks, and blockquotes
- **Editor Service** (`editor_service.dart`): State management using Riverpod with document operations
- **WASM Bridge** (`wasm_bridge.dart`): WebAssembly integration for C core functions
- **Persistence Service** (`persistence_service.dart`): IndexedDB storage with autosave and crash recovery
- **File Service** (`file_service.dart`): File import/export operations

### Panels

- **Markdown Panel**: Live Markdown editing with syntax highlighting
- **JSON Panel**: Direct JSON structure editing with validation
- **Editor Toolbar**: Import/export controls and panel toggles

## 🔧 Development

### Prerequisites

- Flutter 3.24.0 or later
- Dart 3.4.0 or later
- Emscripten (for WASM compilation)

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd c-editor/final-editor
   ```

2. **Install dependencies**
   ```bash
   flutter pub get
   ```

3. **Build WASM core** (for local development)
   ```bash
   cd ../src
   emcc -O3 -s WASM=1 -s EXPORTED_FUNCTIONS='[
     "_note_md_to_json",
     "_note_json_to_md", 
     "_note_json_canonicalize",
     "_note_version",
     "_malloc",
     "_free"
   ]' -s EXPORTED_RUNTIME_METHODS='["ccall","cwrap"]' \
   -s ALLOW_MEMORY_GROWTH=1 \
   -s MODULARIZE=1 \
   -s EXPORT_NAME='createNoteCore' \
   -I. \
   markdown.c json.c editor.c \
   -o ../final-editor/web/assets/core/note_core.js
   ```

4. **Run the development server**
   ```bash
   flutter run -d web-server --web-port 8080
   ```

### Testing

```bash
# Run all tests
flutter test

# Run golden tests (UI snapshots)
flutter test test/golden/

# Update golden files
flutter test test/golden/ --update-goldens
```

## 📦 Building for Production

### Local Build

```bash
flutter build web --web-renderer canvaskit --base-href "/" --release
```

### GitHub Pages Deployment

The project includes a GitHub Actions workflow (`.github/workflows/deploy.yml`) that automatically:

1. Builds the WASM core from C source
2. Compiles the Flutter web app with CanvasKit renderer
3. Deploys to GitHub Pages with proper base href configuration

To deploy:

1. Push changes to the `main` branch
2. GitHub Actions will automatically build and deploy
3. The site will be available at `https://your-username.github.io/c-editor/`

## 🎯 Usage

### Keyboard Shortcuts

- `Ctrl/Cmd + B`: Toggle bold formatting
- `Ctrl/Cmd + I`: Toggle italic formatting
- `Ctrl/Cmd + U`: Toggle underline formatting
- `Ctrl/Cmd + Z`: Undo
- `Ctrl/Cmd + Y`: Redo
- `Ctrl/Cmd + A`: Select all
- `Ctrl/Cmd + S`: Export as JSON
- `Ctrl/Cmd + O`: Import Markdown file

### File Operations

- **Import**: Click toolbar buttons to import `.md` or `.json` files
- **Export**: Export current document as Markdown or JSON
- **Auto-save**: Documents are automatically saved to IndexedDB
- **Crash Recovery**: Automatic recovery of unsaved changes

### Panels

- **Markdown Panel**: Live markdown editor with real-time sync
- **JSON Panel**: Direct JSON structure editing with validation
- Toggle panels on/off using toolbar buttons

## 🧪 Testing Strategy

The project includes comprehensive testing:

- **Unit Tests**: Service logic and data models
- **Widget Tests**: UI components and interactions  
- **Golden Tests**: Visual regression testing across different screen sizes
- **Integration Tests**: End-to-end workflows

## 🏗️ Project Structure

```
final-editor/
├── lib/
│   ├── core/
│   │   └── wasm_bridge.dart           # WebAssembly integration
│   ├── features/
│   │   └── final_editor/
│   │       ├── screens/
│   │       │   └── final_editor_screen.dart
│   │       ├── widgets/
│   │       │   ├── wysiwyg_editor.dart
│   │       │   ├── editor_toolbar.dart
│   │       │   ├── markdown_panel.dart
│   │       │   └── json_panel.dart
│   │       └── services/
│   │           ├── editor_service.dart
│   │           ├── persistence_service.dart
│   │           └── file_service.dart
│   └── main.dart
├── test/
│   ├── golden/                        # Golden tests
│   ├── services/                      # Service tests
│   └── widgets/                       # Widget tests
├── assets/
│   ├── core/                          # WASM files
│   └── examples/                      # Sample files
├── web/
│   └── assets/                        # Web-specific assets
└── .github/
    └── workflows/
        └── deploy.yml                 # GitHub Pages deployment
```

## 🎨 Design System

The editor uses Material Design 3 with:

- **Primary Color**: Blue (#1976D2)  
- **Font**: RobotoMono for code, Roboto for UI
- **Dark/Light Theme**: Automatic system preference detection
- **Responsive Layout**: Adapts to different screen sizes

## 📱 Browser Support

- Chrome/Chromium 88+
- Firefox 78+
- Safari 14+
- Edge 88+

WebAssembly and modern JavaScript features are required.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Update golden files if UI changes
6. Submit a pull request

## 📄 License

This project is part of the c-editor suite. See the main repository for license information.

## 🔗 Related

- [C Editor Core](../src/) - The underlying C library
- [Main Flutter App](../) - Desktop and mobile versions