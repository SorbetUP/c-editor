import 'package:flutter/material.dart';
import 'dart:js' as js;
import 'dart:convert';
import 'dart:typed_data';

void main() {
  runApp(const CEditorApp());
}

class CEditorApp extends StatelessWidget {
  const CEditorApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'C Editor Web',
      theme: ThemeData.dark(),
      debugShowCheckedModeBanner: false,
      home: const EditorScreen(),
    );
  }
}

class EditorScreen extends StatefulWidget {
  const EditorScreen({super.key});

  @override
  State<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends State<EditorScreen> {
  final TextEditingController _controller = TextEditingController();
  String _renderedContent = '';
  bool _isWasmLoaded = false;

  @override
  void initState() {
    super.initState();
    _initializeWasm();
    _controller.addListener(_onTextChanged);
  }

  void _initializeWasm() {
    try {
      // Check if WASM module is available
      if (js.context.hasProperty('EditorModule')) {
        setState(() {
          _isWasmLoaded = true;
        });
      }
    } catch (e) {
      print('WASM not available: $e');
    }
  }

  void _onTextChanged() {
    final text = _controller.text;
    if (_isWasmLoaded && text.isNotEmpty) {
      try {
        // Call C editor function to render markdown
        final result = js.context.callMethod('parseMarkdown', [text]);
        setState(() {
          _renderedContent = result?.toString() ?? text;
        });
      } catch (e) {
        // Fallback to simple rendering
        setState(() {
          _renderedContent = _simpleMarkdownRender(text);
        });
      }
    } else {
      setState(() {
        _renderedContent = _simpleMarkdownRender(text);
      });
    }
  }

  String _simpleMarkdownRender(String text) {
    // Simple client-side markdown rendering for preview
    String result = text;
    
    // Headers
    result = result.replaceAll(RegExp(r'^# (.+)', multiLine: true), r'<h1>$1</h1>');
    result = result.replaceAll(RegExp(r'^## (.+)', multiLine: true), r'<h2>$1</h2>');
    result = result.replaceAll(RegExp(r'^### (.+)', multiLine: true), r'<h3>$1</h3>');
    
    // Bold and italic
    result = result.replaceAll(RegExp(r'\*\*(.+?)\*\*'), r'<b>$1</b>');
    result = result.replaceAll(RegExp(r'\*(.+?)\*'), r'<i>$1</i>');
    
    // Line breaks
    result = result.replaceAll('\n', '<br>');
    
    return result;
  }

  void _exportJson() {
    final text = _controller.text;
    if (_isWasmLoaded) {
      try {
        final json = js.context.callMethod('markdownToJson', [text]);
        _downloadFile('export.json', json?.toString() ?? '{}');
      } catch (e) {
        _downloadFile('export.json', jsonEncode({'content': text}));
      }
    } else {
      _downloadFile('export.json', jsonEncode({'content': text}));
    }
  }

  void _exportMarkdown() {
    _downloadFile('export.md', _controller.text);
  }

  void _exportRendered() {
    _downloadFile('export.html', '<html><body>$_renderedContent</body></html>');
  }

  void _downloadFile(String filename, String content) {
    final bytes = Uint8List.fromList(utf8.encode(content));
    final blob = js.context['Blob'].callMethod('', [
      [bytes],
      {'type': 'application/octet-stream'}
    ]);
    final url = js.context['URL'].callMethod('createObjectURL', [blob]);
    final anchor = js.context['document'].callMethod('createElement', ['a']);
    anchor['href'] = url;
    anchor['download'] = filename;
    anchor.callMethod('click');
    js.context['URL'].callMethod('revokeObjectURL', [url]);
  }

  void _importFile() {
    final input = js.context['document'].callMethod('createElement', ['input']);
    input['type'] = 'file';
    input['accept'] = '.md,.json,.txt';
    
    input.callMethod('addEventListener', ['change', js.allowInterop((event) {
      final file = event.target.files[0];
      if (file != null) {
        final reader = js.context['FileReader'].callMethod('');
        reader.callMethod('addEventListener', ['load', js.allowInterop((e) {
          final content = reader['result'];
          if (content != null) {
            setState(() {
              _controller.text = content.toString();
            });
            _onTextChanged();
          }
        })]);
        reader.callMethod('readAsText', [file]);
      }
    })]);
    
    input.callMethod('click');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('C Editor Web'),
        backgroundColor: Colors.grey[900],
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.file_download),
            tooltip: 'Export',
            onSelected: (value) {
              switch (value) {
                case 'json':
                  _exportJson();
                  break;
                case 'markdown':
                  _exportMarkdown();
                  break;
                case 'rendered':
                  _exportRendered();
                  break;
              }
            },
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'json', child: Text('Export JSON')),
              const PopupMenuItem(value: 'markdown', child: Text('Export Markdown')),
              const PopupMenuItem(value: 'rendered', child: Text('Export Rendered HTML')),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.file_upload),
            tooltip: 'Import',
            onPressed: _importFile,
          ),
        ],
      ),
      body: Row(
        children: [
          // Editor pane
          Expanded(
            flex: 1,
            child: Container(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Editor',
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  const SizedBox(height: 8),
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      maxLines: null,
                      expands: true,
                      style: const TextStyle(fontFamily: 'monospace'),
                      decoration: const InputDecoration(
                        hintText: '# Titre\n\nEcrivez votre markdown ici...',
                        border: OutlineInputBorder(),
                        filled: true,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const VerticalDivider(width: 1),
          // Preview pane
          Expanded(
            flex: 1,
            child: Container(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Text(
                        'Preview',
                        style: Theme.of(context).textTheme.headlineSmall,
                      ),
                      const Spacer(),
                      if (_isWasmLoaded)
                        const Icon(Icons.check_circle, color: Colors.green, size: 16)
                      else
                        const Icon(Icons.warning, color: Colors.orange, size: 16),
                      const SizedBox(width: 4),
                      Text(
                        _isWasmLoaded ? 'WASM Loaded' : 'Fallback Mode',
                        style: const TextStyle(fontSize: 12),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Expanded(
                    child: Container(
                      width: double.infinity,
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        border: Border.all(color: Colors.grey[600]!),
                        borderRadius: BorderRadius.circular(4),
                        color: Colors.grey[850],
                      ),
                      child: SingleChildScrollView(
                        child: Text(
                          _renderedContent.isEmpty ? 'Le rendu apparaîtra ici...' : _renderedContent,
                          style: const TextStyle(fontSize: 16),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
      bottomNavigationBar: Container(
        padding: const EdgeInsets.all(8),
        color: Colors.grey[900],
        child: Text(
          'C Editor Web - Powered by Emscripten WASM',
          textAlign: TextAlign.center,
          style: TextStyle(color: Colors.grey[400], fontSize: 12),
        ),
      ),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}