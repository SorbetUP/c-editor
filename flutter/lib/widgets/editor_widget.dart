import 'package:flutter/material.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'dart:async';

import '../editor_api.dart';
import 'adapters.dart';

/// Main editor widget with live markdown editing and preview
class EditorWidget extends StatefulWidget {
  final String? initialContent;
  final bool showPreview;
  final bool showToolbar;
  final ValueChanged<String>? onChanged;
  final ValueChanged<Document>? onDocumentChanged;
  
  const EditorWidget({
    Key? key,
    this.initialContent,
    this.showPreview = true,
    this.showToolbar = true,
    this.onChanged,
    this.onDocumentChanged,
  }) : super(key: key);

  @override
  State<EditorWidget> createState() => _EditorWidgetState();
}

class _EditorWidgetState extends State<EditorWidget> {
  late final TextEditingController _controller;
  late final EditorApi _editorApi;
  
  Document? _parsedDocument;
  String? _parseError;
  bool _isInitialized = false;
  Timer? _parseTimer;
  
  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialContent ?? '');
    _editorApi = EditorApiFactory.instance();
    
    // Listen to text changes
    _controller.addListener(_onTextChanged);
    
    _initializeEditor();
  }
  
  @override
  void dispose() {
    _parseTimer?.cancel();
    _controller.dispose();
    super.dispose();
  }
  
  Future<void> _initializeEditor() async {
    final result = await _editorApi.initialize();
    
    if (mounted) {
      setState(() {
        _isInitialized = result.success;
        if (!result.success) {
          _parseError = result.error;
        }
      });
      
      // Parse initial content
      if (_isInitialized && _controller.text.isNotEmpty) {
        _scheduleUpdate();
      }
    }
  }
  
  void _onTextChanged() {
    widget.onChanged?.call(_controller.text);
    _scheduleUpdate();
  }
  
  void _scheduleUpdate() {
    _parseTimer?.cancel();
    _parseTimer = Timer(const Duration(milliseconds: 300), _parseMarkdown);
  }
  
  Future<void> _parseMarkdown() async {
    if (!_isInitialized || _controller.text.isEmpty) {
      if (mounted) {
        setState(() {
          _parsedDocument = null;
          _parseError = null;
        });
      }
      return;
    }
    
    final result = await _editorApi.parseMarkdown(_controller.text);
    
    if (mounted) {
      setState(() {
        if (result.success) {
          _parsedDocument = result.data;
          _parseError = null;
          widget.onDocumentChanged?.call(_parsedDocument!);
        } else {
          _parseError = result.error;
          _parsedDocument = null;
        }
      });
    }
  }
  
  void _insertFormatting(String prefix, String suffix) {
    final selection = _controller.selection;
    final text = _controller.text;
    
    if (selection.isValid) {
      final selectedText = selection.textInside(text);
      final newText = prefix + selectedText + suffix;
      
      _controller.text = text.replaceRange(selection.start, selection.end, newText);
      
      // Update cursor position
      final newStart = selection.start + prefix.length;
      final newEnd = newStart + selectedText.length;
      _controller.selection = TextSelection(
        baseOffset: newStart,
        extentOffset: newEnd,
      );
    }
  }
  
  @override
  Widget build(BuildContext context) {
    if (!_isInitialized) {
      return const Center(
        child: CircularProgressIndicator(),
      );
    }
    
    return Column(
      children: [
        if (widget.showToolbar) _buildToolbar(context),
        
        Expanded(
          child: widget.showPreview 
              ? _buildSplitView(context)
              : _buildEditorOnly(context),
        ),
        
        if (_parseError != null) _buildErrorBar(context),
      ],
    );
  }
  
  Widget _buildToolbar(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).colorScheme.outline,
          ),
        ),
      ),
      child: Wrap(
        spacing: 8.0,
        children: [
          IconButton(
            onPressed: () => _insertFormatting('**', '**'),
            icon: const Icon(Icons.format_bold),
            tooltip: 'Bold',
          ),
          IconButton(
            onPressed: () => _insertFormatting('*', '*'),
            icon: const Icon(Icons.format_italic),
            tooltip: 'Italic',
          ),
          IconButton(
            onPressed: () => _insertFormatting('***', '***'),
            icon: const Icon(Icons.format_bold),
            tooltip: 'Bold + Italic',
          ),
          IconButton(
            onPressed: () => _insertFormatting('==', '=='),
            icon: const Icon(Icons.highlight),
            tooltip: 'Highlight',
          ),
          IconButton(
            onPressed: () => _insertFormatting('++', '++'),
            icon: const Icon(Icons.format_underlined),
            tooltip: 'Underline',
          ),
          const VerticalDivider(),
          IconButton(
            onPressed: () => _insertFormatting('# ', ''),
            icon: const Icon(Icons.title),
            tooltip: 'Header 1',
          ),
          IconButton(
            onPressed: () => _insertFormatting('## ', ''),
            icon: const Icon(Icons.title),
            tooltip: 'Header 2',
          ),
        ],
      ),
    );
  }
  
  Widget _buildSplitView(BuildContext context) {
    return Row(
      children: [
        Expanded(
          flex: 1,
          child: _buildEditor(context),
        ),
        
        VerticalDivider(
          width: 1,
          color: Theme.of(context).colorScheme.outline,
        ),
        
        Expanded(
          flex: 1,
          child: _buildPreview(context),
        ),
      ],
    );
  }
  
  Widget _buildEditorOnly(BuildContext context) {
    return _buildEditor(context);
  }
  
  Widget _buildEditor(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16.0),
      child: TextField(
        controller: _controller,
        maxLines: null,
        expands: true,
        style: const TextStyle(
          fontFamily: 'monospace',
          fontSize: 14.0,
        ),
        decoration: const InputDecoration(
          hintText: 'Enter your markdown here...',
          border: InputBorder.none,
        ),
        textAlignVertical: TextAlignVertical.top,
      ),
    );
  }
  
  Widget _buildPreview(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16.0),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: SingleChildScrollView(
        child: _parsedDocument != null
            ? DocumentRenderer(document: _parsedDocument!)
            : const Text(
                'Preview will appear here...',
                style: TextStyle(
                  fontStyle: FontStyle.italic,
                  color: Colors.grey,
                ),
              ),
      ),
    );
  }
  
  Widget _buildErrorBar(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(8.0),
      color: Theme.of(context).colorScheme.errorContainer,
      child: Text(
        'Parse Error: $_parseError',
        style: TextStyle(
          color: Theme.of(context).colorScheme.onErrorContainer,
          fontSize: 12.0,
        ),
      ),
    );
  }
}

/// Widget for rendering parsed documents
class DocumentRenderer extends StatelessWidget {
  final Document document;
  
  const DocumentRenderer({
    Key? key,
    required this.document,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: document.elements.map((element) {
        return _renderElement(context, element);
      }).toList(),
    );
  }
  
  Widget _renderElement(BuildContext context, DocElement element) {
    switch (element) {
      case DocTextElement():
        return _renderTextElement(context, element);
      case DocImageElement():
        return _renderImageElement(context, element);
      case DocTableElement():
        return _renderTableElement(context, element);
      default:
        return const SizedBox.shrink();
    }
  }
  
  Widget _renderTextElement(BuildContext context, DocTextElement element) {
    final style = element.level > 0
        ? Theme.of(context).textTheme.headlineMedium?.copyWith(
            fontSize: (24 - element.level * 2).toDouble(),
          )
        : Theme.of(context).textTheme.bodyMedium;
    
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: RichText(
        textAlign: mapDocAlign(element.align),
        text: TextSpan(
          style: style,
          children: element.spans.map((span) => mapDocSpan(span)).toList(),
        ),
      ),
    );
  }
  
  Widget _renderImageElement(BuildContext context, DocImageElement element) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Column(
        crossAxisAlignment: _alignmentFromString(element.align),
        children: [
          Image.network(
            element.src,
            width: element.width?.toDouble(),
            height: element.height?.toDouble(),
            opacity: AlwaysStoppedAnimation(element.alpha),
            errorBuilder: (context, error, stackTrace) {
              return Container(
                padding: const EdgeInsets.all(8.0),
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.grey),
                  borderRadius: BorderRadius.circular(4.0),
                ),
                child: Text('Image load error: ${element.src}'),
              );
            },
          ),
          if (element.alt.isNotEmpty) ...[
            const SizedBox(height: 4.0),
            Text(
              element.alt,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ],
      ),
    );
  }
  
  Widget _renderTableElement(BuildContext context, DocTableElement element) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Table(
        border: TableBorder.all(color: Theme.of(context).colorScheme.outline),
        children: element.rows.asMap().entries.map((rowEntry) {
          final isHeader = rowEntry.key == 0;
          final row = rowEntry.value;
          
          return TableRow(
            decoration: isHeader
                ? BoxDecoration(
                    color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  )
                : null,
            children: row.map((cell) {
              return TableCell(
                child: Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: RichText(
                    text: TextSpan(
                      children: cell.map((span) => mapDocSpan(span)).toList(),
                    ),
                  ),
                ),
              );
            }).toList(),
          );
        }).toList(),
      ),
    );
  }
  
  CrossAxisAlignment _alignmentFromString(String align) {
    switch (align) {
      case 'left':
        return CrossAxisAlignment.start;
      case 'center':
        return CrossAxisAlignment.center;
      case 'right':
        return CrossAxisAlignment.end;
      case 'justify':
        return CrossAxisAlignment.stretch;
      default:
        return CrossAxisAlignment.start;
    }
  }
}