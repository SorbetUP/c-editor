import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/editor/editor_controller.dart';
import '../../../core/editor/span_text_controller.dart';

/// Comprehensive editor toolbar with formatting options
class EditorToolbar extends ConsumerWidget {
  final EditorController controller;
  final bool isCompact;
  final VoidCallback? onUndo;
  final VoidCallback? onRedo;
  
  const EditorToolbar({
    Key? key,
    required this.controller,
    this.isCompact = false,
    this.onUndo,
    this.onRedo,
  }) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(editorStateProvider);
    
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: isCompact ? _buildCompactToolbar(context, state) : _buildFullToolbar(context, state),
    );
  }
  
  Widget _buildFullToolbar(BuildContext context, EditorState state) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8.0, vertical: 4.0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Top row: Undo/Redo, File operations
          _buildTopRow(context, state),
          const SizedBox(height: 4.0),
          
          // Main row: Text formatting
          _buildMainRow(context, state),
          const SizedBox(height: 4.0),
          
          // Bottom row: Advanced formatting
          _buildBottomRow(context, state),
        ],
      ),
    );
  }
  
  Widget _buildCompactToolbar(BuildContext context, EditorState state) {
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Wrap(
        spacing: 4.0,
        runSpacing: 4.0,
        children: [
          // Essential buttons only
          _buildUndoRedoButtons(context, state),
          const SizedBox(width: 8.0),
          _buildTextFormatButtons(context),
          const SizedBox(width: 8.0),
          _buildHeaderButtons(context),
        ],
      ),
    );
  }
  
  Widget _buildTopRow(BuildContext context, EditorState state) {
    return Row(
      children: [
        _buildUndoRedoButtons(context, state),
        const SizedBox(width: 16.0),
        _buildFileButtons(context, state),
        const Spacer(),
        _buildViewButtons(context, state),
      ],
    );
  }
  
  Widget _buildMainRow(BuildContext context, EditorState state) {
    return Row(
      children: [
        _buildTextFormatButtons(context),
        const SizedBox(width: 16.0),
        _buildHeaderButtons(context),
        const SizedBox(width: 16.0),
        _buildListButtons(context),
        const Spacer(),
        _buildInsertButtons(context),
      ],
    );
  }
  
  Widget _buildBottomRow(BuildContext context, EditorState state) {
    return Row(
      children: [
        _buildTableButtons(context),
        const SizedBox(width: 16.0),
        _buildAdvancedFormatButtons(context),
        const Spacer(),
        _buildUtilityButtons(context, state),
      ],
    );
  }
  
  Widget _buildUndoRedoButtons(BuildContext context, EditorState state) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: controller.canUndo ? () {
            controller.undo();
            onUndo?.call();
          } : null,
          icon: const Icon(Icons.undo),
          tooltip: 'Undo (Ctrl+Z)',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: controller.canRedo ? () {
            controller.redo();
            onRedo?.call();
          } : null,
          icon: const Icon(Icons.redo),
          tooltip: 'Redo (Ctrl+Y)',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildFileButtons(BuildContext context, EditorState state) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _showFileMenu(context),
          icon: const Icon(Icons.folder_open),
          tooltip: 'Open file',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: state.isDirty ? () => _saveDocument(context) : null,
          icon: const Icon(Icons.save),
          tooltip: 'Save (Ctrl+S)',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _exportDocument(context),
          icon: const Icon(Icons.download),
          tooltip: 'Export',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildViewButtons(BuildContext context, EditorState state) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _togglePreview(context),
          icon: const Icon(Icons.preview),
          tooltip: 'Toggle preview',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _toggleFullscreen(context),
          icon: const Icon(Icons.fullscreen),
          tooltip: 'Fullscreen',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildTextFormatButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildFormatButton(
          context,
          icon: Icons.format_bold,
          tooltip: 'Bold (Ctrl+B)',
          format: MarkdownFormat.bold,
        ),
        _buildFormatButton(
          context,
          icon: Icons.format_italic,
          tooltip: 'Italic (Ctrl+I)',
          format: MarkdownFormat.italic,
        ),
        _buildFormatButton(
          context,
          icon: Icons.format_underlined,
          tooltip: 'Underline (Ctrl+U)',
          format: MarkdownFormat.underline,
        ),
        _buildFormatButton(
          context,
          icon: Icons.highlight,
          tooltip: 'Highlight',
          format: MarkdownFormat.highlight,
        ),
        _buildFormatButton(
          context,
          icon: Icons.code,
          tooltip: 'Code (`)',
          format: MarkdownFormat.code,
        ),
      ],
    );
  }
  
  Widget _buildHeaderButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildFormatButton(
          context,
          icon: Icons.title,
          tooltip: 'Header 1',
          format: MarkdownFormat.header1,
          text: 'H1',
        ),
        _buildFormatButton(
          context,
          icon: Icons.title,
          tooltip: 'Header 2',
          format: MarkdownFormat.header2,
          text: 'H2',
        ),
        _buildFormatButton(
          context,
          icon: Icons.title,
          tooltip: 'Header 3',
          format: MarkdownFormat.header3,
          text: 'H3',
        ),
      ],
    );
  }
  
  Widget _buildListButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _insertList(context, ordered: false),
          icon: const Icon(Icons.format_list_bulleted),
          tooltip: 'Bullet list',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertList(context, ordered: true),
          icon: const Icon(Icons.format_list_numbered),
          tooltip: 'Numbered list',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertQuote(context),
          icon: const Icon(Icons.format_quote),
          tooltip: 'Quote',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildInsertButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _insertLink(context),
          icon: const Icon(Icons.link),
          tooltip: 'Insert link',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertImage(context),
          icon: const Icon(Icons.image),
          tooltip: 'Insert image',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertTable(context),
          icon: const Icon(Icons.table_chart),
          tooltip: 'Insert table',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildTableButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _insertTableRow(context),
          icon: const Icon(Icons.table_rows),
          tooltip: 'Insert table row',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertTableColumn(context),
          icon: const Icon(Icons.view_column),
          tooltip: 'Insert table column',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _deleteTableRow(context),
          icon: const Icon(Icons.delete_sweep),
          tooltip: 'Delete table row',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildAdvancedFormatButtons(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _insertCodeBlock(context),
          icon: const Icon(Icons.code_off),
          tooltip: 'Code block',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertHorizontalRule(context),
          icon: const Icon(Icons.horizontal_rule),
          tooltip: 'Horizontal rule',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _insertFootnote(context),
          icon: const Icon(Icons.note),
          tooltip: 'Footnote',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildUtilityButtons(BuildContext context, EditorState state) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        IconButton(
          onPressed: () => _findReplace(context),
          icon: const Icon(Icons.search),
          tooltip: 'Find & Replace (Ctrl+F)',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _showWordCount(context, state),
          icon: const Icon(Icons.text_fields),
          tooltip: 'Word count',
          iconSize: 20.0,
        ),
        IconButton(
          onPressed: () => _showSettings(context),
          icon: const Icon(Icons.settings),
          tooltip: 'Editor settings',
          iconSize: 20.0,
        ),
      ],
    );
  }
  
  Widget _buildFormatButton(
    BuildContext context, {
    required IconData icon,
    required String tooltip,
    required MarkdownFormat format,
    String? text,
  }) {
    return SizedBox(
      width: 32.0,
      height: 32.0,
      child: IconButton(
        onPressed: () => _applyFormat(format),
        icon: text != null ? Text(
          text,
          style: const TextStyle(fontSize: 11.0, fontWeight: FontWeight.bold),
        ) : Icon(icon),
        tooltip: tooltip,
        iconSize: 20.0,
        padding: EdgeInsets.zero,
      ),
    );
  }
  
  // Format operations
  void _applyFormat(MarkdownFormat format) {
    final textController = controller.textController;
    if (textController is SpanTextEditingController) {
      textController.applyFormatting(format);
    } else {
      // Fallback for regular TextEditingController
      _applyFallbackFormat(format);
    }
  }
  
  void _applyFallbackFormat(MarkdownFormat format) {
    String prefix = '';
    String suffix = '';
    
    switch (format) {
      case MarkdownFormat.bold:
        prefix = suffix = '**';
        break;
      case MarkdownFormat.italic:
        prefix = suffix = '*';
        break;
      case MarkdownFormat.boldItalic:
        prefix = suffix = '***';
        break;
      case MarkdownFormat.underline:
        prefix = suffix = '++';
        break;
      case MarkdownFormat.highlight:
        prefix = suffix = '==';
        break;
      case MarkdownFormat.code:
        prefix = suffix = '`';
        break;
      case MarkdownFormat.header1:
        prefix = '# ';
        break;
      case MarkdownFormat.header2:
        prefix = '## ';
        break;
      case MarkdownFormat.header3:
        prefix = '### ';
        break;
      case MarkdownFormat.quote:
        prefix = '> ';
        break;
    }
    
    controller.insertFormatting(prefix, suffix);
  }
  
  // Insert operations
  void _insertList(BuildContext context, {required bool ordered}) {
    final marker = ordered ? '1. ' : '- ';
    controller.insertText('\n$marker');
  }
  
  void _insertQuote(BuildContext context) {
    controller.insertFormatting('\n> ', '');
  }
  
  void _insertLink(BuildContext context) {
    _showInsertDialog(
      context,
      title: 'Insert Link',
      fields: ['Text', 'URL'],
      onInsert: (values) {
        final text = values[0];
        final url = values[1];
        controller.insertText('[$text]($url)');
      },
    );
  }
  
  void _insertImage(BuildContext context) {
    _showInsertDialog(
      context,
      title: 'Insert Image',
      fields: ['Alt text', 'URL'],
      onInsert: (values) {
        final alt = values[0];
        final url = values[1];
        controller.insertText('![$alt]($url)');
      },
    );
  }
  
  void _insertTable(BuildContext context) {
    _showTableDialog(context);
  }
  
  void _insertTableRow(BuildContext context) {
    controller.insertText('\n| Column 1 | Column 2 | Column 3 |');
  }
  
  void _insertTableColumn(BuildContext context) {
    // Complex operation - would need cursor position analysis
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Table column insertion not implemented yet')),
    );
  }
  
  void _deleteTableRow(BuildContext context) {
    // Complex operation - would need line detection
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Table row deletion not implemented yet')),
    );
  }
  
  void _insertCodeBlock(BuildContext context) {
    controller.insertText('\n```\ncode here\n```\n');
  }
  
  void _insertHorizontalRule(BuildContext context) {
    controller.insertText('\n---\n');
  }
  
  void _insertFootnote(BuildContext context) {
    controller.insertText('[^1]\n\n[^1]: Footnote text');
  }
  
  // Dialog operations
  void _showInsertDialog(
    BuildContext context, {
    required String title,
    required List<String> fields,
    required Function(List<String>) onInsert,
  }) {
    final controllers = fields.map((_) => TextEditingController()).toList();
    
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: fields.asMap().entries.map((entry) {
            final index = entry.key;
            final field = entry.value;
            return Padding(
              padding: const EdgeInsets.only(bottom: 8.0),
              child: TextField(
                controller: controllers[index],
                decoration: InputDecoration(labelText: field),
              ),
            );
          }).toList(),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              final values = controllers.map((c) => c.text).toList();
              onInsert(values);
              Navigator.pop(context);
            },
            child: const Text('Insert'),
          ),
        ],
      ),
    );
  }
  
  void _showTableDialog(BuildContext context) {
    int rows = 3;
    int cols = 3;
    
    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Insert Table'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                children: [
                  const Text('Rows: '),
                  Slider(
                    value: rows.toDouble(),
                    min: 2,
                    max: 10,
                    divisions: 8,
                    label: rows.toString(),
                    onChanged: (value) => setState(() => rows = value.round()),
                  ),
                ],
              ),
              Row(
                children: [
                  const Text('Columns: '),
                  Slider(
                    value: cols.toDouble(),
                    min: 2,
                    max: 10,
                    divisions: 8,
                    label: cols.toString(),
                    onChanged: (value) => setState(() => cols = value.round()),
                  ),
                ],
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () {
                _generateTable(rows, cols);
                Navigator.pop(context);
              },
              child: const Text('Insert'),
            ),
          ],
        ),
      ),
    );
  }
  
  void _generateTable(int rows, int cols) {
    final header = '| ${List.filled(cols, 'Header').join(' | ')} |';
    final separator = '| ${List.filled(cols, '---').join(' | ')} |';
    final dataRows = List.generate(rows - 1, (i) => 
        '| ${List.filled(cols, 'Data').join(' | ')} |').join('\n');
    
    final table = '$header\n$separator\n$dataRows';
    controller.insertText('\n$table\n');
  }
  
  // Menu operations
  void _showFileMenu(BuildContext context) {
    // Implementation would show file operations menu
  }
  
  void _saveDocument(BuildContext context) {
    controller.markSaved();
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Document saved')),
    );
  }
  
  void _exportDocument(BuildContext context) {
    // Implementation would show export options
  }
  
  void _togglePreview(BuildContext context) {
    // Implementation would toggle preview pane
  }
  
  void _toggleFullscreen(BuildContext context) {
    // Implementation would toggle fullscreen mode
  }
  
  void _findReplace(BuildContext context) {
    // Implementation would show find/replace dialog
  }
  
  void _showWordCount(BuildContext context, EditorState state) {
    final words = state.text.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).length;
    final chars = state.text.length;
    final lines = state.text.split('\n').length;
    
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Document Statistics'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Words: $words'),
            Text('Characters: $chars'),
            Text('Lines: $lines'),
            if (state.document != null) ...[
              const SizedBox(height: 8.0),
              Text('Elements: ${state.document!.elements.length}'),
            ],
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
  
  void _showSettings(BuildContext context) {
    // Implementation would show editor settings
  }
}