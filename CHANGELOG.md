# Changelog

All notable changes to the C Editor project will be documented in this file.

## [1.3.0-pages-final] - 2024-01-09

### 🌟 **Major: Final Web Editor Release**

Complete web-based WYSIWYG editor optimized for GitHub Pages deployment.

#### ✨ Added
- **Final Web Editor**: Separate web-optimized editor in `final-editor/` directory
- **WYSIWYG Rendering**: Real-time visual document rendering with support for:
  - Headings (H1-H6) with proper typography
  - Paragraphs with rich text support
  - Ordered and unordered lists with nesting
  - Code blocks with syntax highlighting hints
  - Blockquotes with visual styling
  - Inline code elements
- **3-Panel Interface**: 
  - Main WYSIWYG editor
  - Toggleable Markdown editor panel
  - Toggleable JSON structure editor panel
- **File Operations**:
  - Import Markdown (`.md`) and JSON (`.json`) files
  - Export to both formats with proper file naming
  - File validation and error handling
- **IndexedDB Persistence**:
  - Automatic document saving
  - Crash recovery system
  - Session restoration
  - Multiple document management
- **WASM Integration**: WebAssembly bridge for C core functions
- **GitHub Actions Workflow**: Automated deployment to GitHub Pages
- **Code Coverage Gate**: 80% minimum coverage requirement in CI
- **Optional Telemetry**: Anonymized performance and error monitoring

#### 🧪 Testing
- **Golden Tests**: Visual regression testing across multiple screen sizes
- **Round-trip Tests**: MD↔JSON↔MD conversion validation
- **Integration Tests**: End-to-end workflow testing
- **Performance Tests**: Large document handling validation
- **Test Fixtures**: Sample documents for lists, code blocks, and blockquotes

#### 🎨 UI/UX
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Material Design 3**: Modern UI with light/dark theme support
- **Keyboard Shortcuts**: Full keyboard navigation and formatting
- **Status Bar**: Real-time document stats and save status
- **Error Handling**: User-friendly error messages and recovery

#### 🚀 Deployment
- **GitHub Pages Ready**: Optimized build configuration
- **CanvasKit Renderer**: Consistent text rendering across browsers
- **Relative Assets**: Proper base-href configuration
- **WASM Compilation**: Automated C core to WebAssembly build

#### 📊 Metrics & Monitoring
- **Coverage Reporting**: Automated test coverage analysis
- **Performance Tracking**: Parse time and document size monitoring
- **Error Telemetry**: Anonymous error reporting for production health
- **Usage Analytics**: Optional feature usage tracking

### 🐛 Known Limitations
- WASM functions are mocked in tests (actual C integration in production)
- Some deprecated Material Design APIs used (will be updated)
- Telemetry endpoint is placeholder (needs actual analytics service)
- Font loading depends on system fonts for golden tests

### 📍 Live Demo
- **GitHub Pages**: https://your-username.github.io/c-editor/ (after deployment)
- **Source Code**: [final-editor/](./final-editor/)
- **Test Coverage**: 80%+ required by CI

### 🔗 Links
- [Final Editor README](./final-editor/README.md)
- [GitHub Actions Workflow](./final-editor/.github/workflows/deploy.yml)
- [Test Suite Documentation](./final-editor/test/)

---

## [Previous versions]

### [1.2.0] - 2024-01-08
- Desktop Flutter application with FFI bridge
- Playground implementation
- Comprehensive test suites
- Platform support (macOS, iOS, Android)

### [1.1.0] - 2024-01-07  
- Core C library implementation
- Markdown to JSON conversion
- Basic test framework

### [1.0.0] - 2024-01-06
- Initial project structure
- Basic markdown parsing