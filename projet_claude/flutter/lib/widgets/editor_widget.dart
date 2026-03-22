import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'dart:async';

import 'adapters.dart';
import '../core/editor/editor_controller.dart';
import '../core/editor/span_text_controller.dart';
import '../core/editor/editor_sync.dart';
import '../core/editor/editor_shortcuts.dart';
import '../features/editor/widgets/editor_toolbar.dart';

/// Advanced interactive editor widget with real-time markdown parsing
class EditorWidget extends ConsumerStatefulWidget {
  final String? initialContent;
  final bool showPreview;
  final bool showToolbar;
  final bool enableSyntaxHighlighting;
  final bool enableSync;
  final ValueChanged<String>? onChanged;
  final ValueChanged<Document>? onDocumentChanged;
  final VoidCallback? onSave;
  final VoidCallback? onOpen;
  final VoidCallback? onNew;
  
  const EditorWidget({
    Key? key,
    this.initialContent,
    this.showPreview = true,
    this.showToolbar = true,
    this.enableSyntaxHighlighting = true,
    this.enableSync = true,
    this.onChanged,
    this.onDocumentChanged,
    this.onSave,
    this.onOpen,
    this.onNew,
  }) : super(key: key);

  @override
  ConsumerState<EditorWidget> createState() => _EditorWidgetState();
}

class _EditorWidgetState extends ConsumerState<EditorWidget> 
    with EditorSyncMixin<EditorWidget> {
  late final SpanTextEditingController _textController;
  late final ScrollController _editorScrollController;
  late final ScrollController _previewScrollController;
  late final EditorController _editorController;
  
  
  @override
  void initState() {
    super.initState();
    
    // Initialize controllers
    _textController = SpanTextEditingController(
      text: widget.initialContent ?? '',
      enableSyntaxHighlighting: widget.enableSyntaxHighlighting,
    );
    _editorScrollController = ScrollController();
    _previewScrollController = ScrollController();
    // EditorController will be accessed via ref.watch in build method
    
    
    _initializeEditor();
  }
  
  @override
  void dispose() {
    _textController.dispose();
    _editorScrollController.dispose();
    _previewScrollController.dispose();
    disposeSync();
    super.dispose();
  }
  
  Future<void> _initializeEditor() async {
    // Initialize sync if enabled
    if (widget.enableSync) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          initializeSync(ref, _editorScrollController);
        }
      });
    }
  }
  
  @override
  void onScrollChanged(double offset) {
    if (widget.enableSync) {
      updateSyncScroll(offset, SyncDirection.editorToPreview);
    }
  }
  
  
  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorStateProvider);
    final editorController = ref.watch(editorStateProvider.notifier);
    
    // Set initial content if provided and controller is empty
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (widget.initialContent != null && 
          editorState.text.isEmpty && 
          widget.initialContent!.isNotEmpty) {
        editorController.setContent(widget.initialContent!);
      }
    });
    
    return EditorShortcuts(
      controller: editorController,
      onSave: widget.onSave,
      onOpen: widget.onOpen,
      onNew: widget.onNew,
      child: Column(
        children: [
          if (widget.showToolbar) 
            EditorToolbar(
              controller: editorController,
              onUndo: () => editorController.undo(),
              onRedo: () => editorController.redo(),
            ),
          
          Expanded(
            child: widget.showPreview 
                ? _buildSplitView(context, editorState)
                : _buildEditorOnly(context, editorState),
          ),
          
          if (editorState.error != null) _buildErrorBar(context, editorState),
        ],
      ),
    );
  }
  
  
  Widget _buildSplitView(BuildContext context, EditorState editorState) {
    return Row(
      children: [
        Expanded(
          flex: 1,
          child: _buildEditor(context, editorState),
        ),
        
        VerticalDivider(
          width: 1,
          color: Theme.of(context).colorScheme.outline,
        ),
        
        Expanded(
          flex: 1,
          child: _buildPreview(context, editorState),
        ),
      ],
    );
  }
  
  Widget _buildEditorOnly(BuildContext context, EditorState editorState) {
    return _buildEditor(context, editorState);
  }
  
  Widget _buildEditor(BuildContext context, EditorState editorState) {
    // Update text controller with latest document for syntax highlighting
    if (editorState.document != null && widget.enableSyntaxHighlighting) {
      _textController.updateDocument(editorState.document);
    }
    
    return Container(
      padding: const EdgeInsets.all(16.0),
      child: Scrollbar(
        controller: _editorScrollController,
        child: TextField(
          controller: _textController,
          scrollController: _editorScrollController,
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
          onChanged: (text) {
            widget.onChanged?.call(text);
            if (editorState.document != null) {
              widget.onDocumentChanged?.call(editorState.document!);
            }
          },
        ),
      ),
    );
  }
  
  Widget _buildPreview(BuildContext context, EditorState editorState) {
    return Container(
      padding: const EdgeInsets.all(16.0),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Scrollbar(
        controller: _previewScrollController,
        child: SingleChildScrollView(
          controller: _previewScrollController,
          child: editorState.isLoading
              ? const Center(
                  child: Padding(
                    padding: EdgeInsets.all(20.0),
                    child: CircularProgressIndicator(),
                  ),
                )
              : editorState.document != null
                  ? DocumentRenderer(document: editorState.document!)
                  : const Text(
                      'Preview will appear here...',
                      style: TextStyle(
                        fontStyle: FontStyle.italic,
                        color: Colors.grey,
                      ),
                    ),
        ),
      ),
    );
  }
  
  Widget _buildErrorBar(BuildContext context, EditorState editorState) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      width: double.infinity,
      padding: const EdgeInsets.all(8.0),
      color: Theme.of(context).colorScheme.errorContainer,
      child: Row(
        children: [
          Icon(
            Icons.error_outline,
            color: Theme.of(context).colorScheme.onErrorContainer,
            size: 16.0,
          ),
          const SizedBox(width: 8.0),
          Expanded(
            child: Text(
              'Parse Error: ${editorState.error}',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onErrorContainer,
                fontSize: 12.0,
              ),
            ),
          ),
          IconButton(
            onPressed: () {
              // Clear error - would need to update controller
            },
            icon: Icon(
              Icons.close,
              color: Theme.of(context).colorScheme.onErrorContainer,
              size: 16.0,
            ),
            iconSize: 16.0,
            constraints: const BoxConstraints(),
            padding: EdgeInsets.zero,
          ),
        ],
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