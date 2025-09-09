# Examples

This directory contains example applications and demonstrations for the c-editor-flutter library.

## Playground

The playground example demonstrates the core capabilities of the markdown editor:

- **3-Panel View**: Markdown input, rendered preview, and JSON output
- **Real-time Parsing**: See changes instantly as you type
- **Performance Metrics**: View parse and render times
- **Sample Content**: Pre-loaded examples to test various markdown features

### Features Demonstrated

- **Text Styling**: Bold, italic, code, strikethrough
- **Headers**: H1, H2, H3 with proper hierarchy
- **Lists**: Ordered, unordered, and task lists with nesting
- **Tables**: Multi-column tables with formatting
- **Code Blocks**: Syntax-highlighted code examples
- **Links & Images**: External resources and media
- **Quotes**: Block quotes with nested formatting

### Running the Playground

To run the playground example:

1. Navigate to the main flutter directory
2. Uncomment the playground route in `lib/core/routing/app_router.dart`
3. Run the application: `flutter run -d web`
4. Navigate to `/playground` or click the science icon in the home screen

### Architecture

The playground demonstrates the separation of concerns:
- **Markdown Input**: Raw text editing with monospace font
- **C Core Processing**: Real-time parsing via FFI/WASM bridge
- **Flutter Rendering**: Rich document display using custom widgets
- **JSON Output**: Canonical document structure for debugging

This example showcases the full pipeline from markdown input to rendered output, making it useful for:
- Testing new markdown features
- Debugging parsing issues  
- Understanding the document structure
- Performance testing with large documents