import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:c_editor_flutter/models/models.dart';
import '../../../engine/bridge.dart';
import '../../../features/viewer/widgets/document_renderer.dart';

/// Interactive playground with 3 panels: Markdown editor, rendered view, JSON output
class PlaygroundScreen extends StatefulWidget {
  const PlaygroundScreen({super.key});

  @override
  State<PlaygroundScreen> createState() => _PlaygroundScreenState();
}

class _PlaygroundScreenState extends State<PlaygroundScreen> {
  final _mdController = TextEditingController(
    text: '''# Hello World

This is a **bold** text with *italics* and some `inline code`.

## Features

- [x] Markdown parsing
- [x] Real-time preview  
- [x] JSON export
- [ ] Advanced styling

### Table Example

| Feature | Status | Priority |
|---------|--------|----------|
| Editor | ✅ Done | High |
| Preview | ✅ Done | High |
| Export | ✅ Done | Medium |

> This is a quote block with some **bold** text inside.

```javascript
const hello = () => {
  console.log("Hello from code block!");
};
```

![Sample Image](https://via.placeholder.com/300x200/4FC3F7/FFFFFF?text=Sample+Image)
''',
  );

  final _jsonController = TextEditingController();
  
  String _jsonOutput = '';
  Document? _document;
  String? _parseError;
  Timer? _debounceTimer;
  bool _isLoading = false;
  
  // Performance tracking
  int _parseTimeMs = 0;
  int _renderTimeMs = 0;
  
  @override
  void initState() {
    super.initState();
    _mdController.addListener(_onMarkdownChanged);
    _recomputeDocument();
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    _mdController.dispose();
    _jsonController.dispose();
    super.dispose();
  }

  void _onMarkdownChanged() {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 300), _recomputeDocument);
  }

  Future<void> _recomputeDocument() async {
    if (!mounted) return;
    
    setState(() {
      _isLoading = true;
      _parseError = null;
    });

    try {
      final markdown = _mdController.text;
      if (markdown.isEmpty) {
        setState(() {
          _document = null;
          _jsonOutput = '';
          _isLoading = false;
        });
        return;
      }

      // Parse markdown to JSON
      final parseStart = DateTime.now();
      final editorApi = createEditorApi();
      final jsonResult = await editorApi.mdToJson(markdown);
      final parseEnd = DateTime.now();
      _parseTimeMs = parseEnd.difference(parseStart).inMilliseconds;

      // Canonicalize JSON
      final canonicalJson = await editorApi.canonicalize(jsonResult);
      
      // Parse to Document for rendering
      final renderStart = DateTime.now();
      final document = Document.fromJson(jsonDecode(canonicalJson));
      final renderEnd = DateTime.now();
      _renderTimeMs = renderEnd.difference(renderStart).inMilliseconds;

      if (mounted) {
        setState(() {
          _document = document;
          _jsonOutput = _prettyFormatJson(canonicalJson);
          _jsonController.text = _jsonOutput;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _parseError = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  String _prettyFormatJson(String json) {
    try {
      final decoded = jsonDecode(json);
      const encoder = JsonEncoder.withIndent('  ');
      return encoder.convert(decoded);
    } catch (e) {
      return json;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: _buildAppBar(),
      body: Column(
        children: [
          _buildStatusBar(),
          Expanded(child: _buildMainContent()),
          if (_parseError != null) _buildErrorBar(),
        ],
      ),
    );
  }

  PreferredSizeWidget _buildAppBar() {
    return AppBar(
      title: const Text('Playground: MD ↔ JSON ↔ View'),
      actions: [
        IconButton(
          onPressed: _copyJson,
          icon: const Icon(Icons.copy),
          tooltip: 'Copy JSON',
        ),
        IconButton(
          onPressed: _exportMarkdown,
          icon: const Icon(Icons.download),
          tooltip: 'Export Markdown',
        ),
        IconButton(
          onPressed: _loadSample,
          icon: const Icon(Icons.refresh),
          tooltip: 'Load Sample',
        ),
      ],
    );
  }

  Widget _buildStatusBar() {
    final mdLength = _mdController.text.length;
    final elementCount = _document?.elements.length ?? 0;
    
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        children: [
          Text(
            'Characters: $mdLength',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(width: 16.0),
          Text(
            'Elements: $elementCount',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(width: 16.0),
          Text(
            'Parse: ${_parseTimeMs}ms',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(width: 16.0),
          Text(
            'Render: ${_renderTimeMs}ms',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const Spacer(),
          if (_isLoading)
            const SizedBox(
              width: 16.0,
              height: 16.0,
              child: CircularProgressIndicator(strokeWidth: 2.0),
            ),
        ],
      ),
    );
  }

  Widget _buildMainContent() {
    return Row(
      children: [
        // Markdown Editor Panel
        Expanded(
          flex: 1,
          child: _buildMarkdownPanel(),
        ),
        
        const VerticalDivider(width: 1),
        
        // Rendered View Panel
        Expanded(
          flex: 1,
          child: _buildViewPanel(),
        ),
        
        const VerticalDivider(width: 1),
        
        // JSON Output Panel
        Expanded(
          flex: 1,
          child: _buildJsonPanel(),
        ),
      ],
    );
  }

  Widget _buildMarkdownPanel() {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(12.0),
          width: double.infinity,
          color: Theme.of(context).colorScheme.primary.withOpacity(0.1),
          child: Text(
            'Markdown Input',
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        Expanded(
          child: Container(
            padding: const EdgeInsets.all(16.0),
            child: TextField(
              controller: _mdController,
              maxLines: null,
              expands: true,
              style: const TextStyle(
                fontFamily: 'monospace',
                fontSize: 14.0,
                height: 1.4,
              ),
              decoration: const InputDecoration(
                hintText: 'Enter your markdown here...',
                border: InputBorder.none,
                contentPadding: EdgeInsets.zero,
              ),
              textAlignVertical: TextAlignVertical.top,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildViewPanel() {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(12.0),
          width: double.infinity,
          color: Theme.of(context).colorScheme.secondary.withOpacity(0.1),
          child: Text(
            'Rendered View',
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        Expanded(
          child: Container(
            color: const Color(0xFF202124),
            padding: const EdgeInsets.all(16.0),
            child: _buildPreviewContent(),
          ),
        ),
      ],
    );
  }

  Widget _buildPreviewContent() {
    if (_isLoading) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16.0),
            Text('Parsing...', style: TextStyle(color: Colors.grey)),
          ],
        ),
      );
    }

    if (_document == null) {
      return const Center(
        child: Text(
          'Preview will appear here...',
          style: TextStyle(
            color: Colors.grey,
            fontStyle: FontStyle.italic,
          ),
        ),
      );
    }

    return SingleChildScrollView(
      child: DocumentRenderer(document: _document!),
    );
  }

  Widget _buildJsonPanel() {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(12.0),
          width: double.infinity,
          color: Theme.of(context).colorScheme.tertiary.withOpacity(0.1),
          child: Text(
            'Canonical JSON',
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        Expanded(
          child: Container(
            padding: const EdgeInsets.all(16.0),
            child: SingleChildScrollView(
              child: SelectableText(
                _jsonOutput.isEmpty ? '{}' : _jsonOutput,
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12.0,
                  height: 1.3,
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildErrorBar() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(12.0),
      color: Theme.of(context).colorScheme.errorContainer,
      child: Row(
        children: [
          Icon(
            Icons.error_outline,
            color: Theme.of(context).colorScheme.onErrorContainer,
            size: 20.0,
          ),
          const SizedBox(width: 8.0),
          Expanded(
            child: Text(
              'Parse Error: $_parseError',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onErrorContainer,
                fontSize: 13.0,
              ),
            ),
          ),
          TextButton(
            onPressed: () {
              setState(() {
                _parseError = null;
              });
            },
            child: Text(
              'Dismiss',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onErrorContainer,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _copyJson() {
    if (_jsonOutput.isNotEmpty) {
      Clipboard.setData(ClipboardData(text: _jsonOutput));
      _showSnackBar('JSON copied to clipboard');
    }
  }

  void _exportMarkdown() {
    // For now, copy to clipboard - in real app would trigger download
    final markdown = _mdController.text;
    if (markdown.isNotEmpty) {
      Clipboard.setData(ClipboardData(text: markdown));
      _showSnackBar('Markdown copied to clipboard');
    }
  }

  void _loadSample() {
    const sampleMarkdown = '''# Advanced Markdown Sample

## Text Styling

Normal text with **bold**, *italic*, and ***bold italic*** combinations.
You can also use `inline code` and ~~strikethrough~~ text.

## Lists

### Unordered List
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

### Ordered List
1. First numbered item
2. Second numbered item
   1. Nested numbered item
   2. Another nested numbered item
3. Third numbered item

### Task List
- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

## Code Blocks

```javascript
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10)); // 55
```

## Tables

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| **Bold** | *Italic* | `Code`   |
| Long content here | Short | Med |

## Quotes

> This is a blockquote.
> 
> It can span multiple lines and contain **formatting**.
> 
> > And even nested quotes!

## Links and Images

Check out [this link](https://flutter.dev) and this image:

![Flutter Logo](https://docs.flutter.dev/assets/images/shared/brand/flutter/logo/flutter-logomark-320px.png)

## Horizontal Rule

---

That's all for now!
''';

    _mdController.text = sampleMarkdown;
    _recomputeDocument();
  }

  void _showSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}