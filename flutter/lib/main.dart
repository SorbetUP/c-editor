import 'package:flutter/material.dart';
import 'widgets/editor_widget.dart';
import 'models/document.dart';

void main() {
  runApp(const EditorApp());
}

class EditorApp extends StatelessWidget {
  const EditorApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'C Editor Flutter',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
        fontFamily: 'Roboto',
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
        fontFamily: 'Roboto',
      ),
      home: const EditorScreen(),
    );
  }
}

class EditorScreen extends StatefulWidget {
  const EditorScreen({Key? key}) : super(key: key);

  @override
  State<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends State<EditorScreen> {
  bool _showPreview = true;
  bool _showToolbar = true;
  String _content = '';
  Document? _document;

  final String _sampleContent = '''# Welcome to C Editor Flutter

This is a **markdown editor** with live preview, powered by a *C core library* with multiplatform support.

## Features

- **Bold text** and *italic text*
- ==Highlighted text== and ++underlined text++
- ***Bold and italic*** combined
- Real-time parsing and preview
- Cross-platform: FFI for native, WASM for web

## Example Table

| Feature | Native (FFI) | Web (WASM) |
|---------|-------------|------------|
| Performance | ⚡ Excellent | 🔥 Good |
| Memory | 💾 Direct | 📦 Sandboxed |
| File I/O | ✅ Full | ❌ Limited |

## Image Support

![Sample Image](https://via.placeholder.com/200x100){w=200 h=100 align=center}

Try editing this content to see the live preview update!
''';

  @override
  void initState() {
    super.initState();
    _content = _sampleContent;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('C Editor Flutter'),
        actions: [
          IconButton(
            onPressed: () {
              setState(() {
                _showToolbar = !_showToolbar;
              });
            },
            icon: Icon(_showToolbar ? Icons.build : Icons.build_outlined),
            tooltip: 'Toggle Toolbar',
          ),
          IconButton(
            onPressed: () {
              setState(() {
                _showPreview = !_showPreview;
              });
            },
            icon: Icon(_showPreview ? Icons.visibility : Icons.visibility_off),
            tooltip: 'Toggle Preview',
          ),
          IconButton(
            onPressed: _showInfo,
            icon: const Icon(Icons.info_outline),
            tooltip: 'About',
          ),
        ],
      ),
      body: EditorWidget(
        initialContent: _content,
        showPreview: _showPreview,
        showToolbar: _showToolbar,
        onChanged: (content) {
          setState(() {
            _content = content;
          });
        },
        onDocumentChanged: (document) {
          setState(() {
            _document = document;
          });
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _showDocumentInfo,
        tooltip: 'Document Info',
        child: const Icon(Icons.analytics),
      ),
    );
  }

  void _showInfo() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('C Editor Flutter'),
        content: const Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('A multiplatform markdown editor built with:'),
            SizedBox(height: 8),
            Text('• C11 core library for parsing'),
            Text('• FFI bindings for native platforms'),
            Text('• WASM compilation for web'),
            Text('• Flutter UI framework'),
            SizedBox(height: 16),
            Text('Features:'),
            SizedBox(height: 8),
            Text('• Real-time parsing and preview'),
            Text('• Inline formatting (bold, italic, etc.)'),
            Text('• Header detection and styling'),
            Text('• Table and image support'),
            Text('• Memory-safe C implementation'),
            Text('• Property-based testing'),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  void _showDocumentInfo() {
    if (_document == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No document parsed yet')),
      );
      return;
    }

    final textElements = _document!.elements.whereType<TextElement>().length;
    final imageElements = _document!.elements.whereType<ImageElement>().length;
    final tableElements = _document!.elements.whereType<TableElement>().length;
    final totalElements = _document!.elements.length;

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Document Statistics'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Total Elements: $totalElements'),
            Text('Text Elements: $textElements'),
            Text('Image Elements: $imageElements'),
            Text('Table Elements: $tableElements'),
            const SizedBox(height: 16),
            Text('Content Length: ${_content.length} characters'),
            Text('Lines: ${_content.split('\n').length}'),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}