import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/editor/editor_controller.dart';
import '../../../core/storage/autosave_service.dart';
import '../../../engine/bridge.dart';

/// Status bar showing editor information, cursor position, autosave status, etc.
class EditorStatusBar extends ConsumerWidget {
  final EditorController controller;
  
  const EditorStatusBar({
    Key? key,
    required this.controller,
  }) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editorState = ref.watch(editorStateProvider);
    final autosaveService = ref.watch(autosaveServiceProvider);
    
    return Container(
      height: 28.0,
      padding: const EdgeInsets.symmetric(horizontal: 12.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border(
          top: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: Row(
        children: [
          // Cursor position
          _buildCursorInfo(context, editorState),
          
          const SizedBox(width: 16.0),
          _buildSeparator(context),
          const SizedBox(width: 16.0),
          
          // Document stats
          _buildDocumentStats(context, editorState),
          
          const SizedBox(width: 16.0),
          _buildSeparator(context),
          const SizedBox(width: 16.0),
          
          // Parsing status
          _buildParsingStatus(context, editorState),
          
          const Spacer(),
          
          // Autosave status
          _buildAutosaveStatus(context, autosaveService.status),
          
          const SizedBox(width: 16.0),
          _buildSeparator(context),
          const SizedBox(width: 16.0),
          
          // Core version
          _buildCoreVersion(context),
        ],
      ),
    );
  }

  Widget _buildCursorInfo(BuildContext context, EditorState state) {
    final position = _getCursorPosition(state.text, state.selection.start);
    
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          Icons.edit_location,
          size: 14.0,
          color: Theme.of(context).textTheme.bodySmall?.color,
        ),
        const SizedBox(width: 4.0),
        Text(
          'Line ${position.line}, Col ${position.column}',
          style: Theme.of(context).textTheme.bodySmall,
        ),
        if (state.selection.isCollapsed) ...[
          Text(
            ' | Offset ${state.selection.start}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).textTheme.bodySmall?.color?.withOpacity(0.7),
            ),
          ),
        ] else ...[
          Text(
            ' | Selected ${state.selection.end - state.selection.start} chars',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildDocumentStats(BuildContext context, EditorState state) {
    final stats = _getDocumentStats(state);
    
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          Icons.article_outlined,
          size: 14.0,
          color: Theme.of(context).textTheme.bodySmall?.color,
        ),
        const SizedBox(width: 4.0),
        Text(
          '${stats.characters} chars, ${stats.words} words',
          style: Theme.of(context).textTheme.bodySmall,
        ),
        if (state.document != null) ...[
          Text(
            ' | ${state.document!.elements.length} elements',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).textTheme.bodySmall?.color?.withOpacity(0.7),
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildParsingStatus(BuildContext context, EditorState state) {
    Color statusColor;
    IconData statusIcon;
    String statusText;
    
    if (state.isLoading) {
      statusColor = Theme.of(context).colorScheme.primary;
      statusIcon = Icons.refresh;
      statusText = 'Parsing...';
    } else if (state.error != null) {
      statusColor = Theme.of(context).colorScheme.error;
      statusIcon = Icons.error_outline;
      statusText = 'Parse Error';
    } else if (state.document != null) {
      statusColor = Colors.green;
      statusIcon = Icons.check_circle_outline;
      statusText = 'Valid';
    } else {
      statusColor = Theme.of(context).textTheme.bodySmall?.color ?? Colors.grey;
      statusIcon = Icons.radio_button_unchecked;
      statusText = 'Empty';
    }
    
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          statusIcon,
          size: 14.0,
          color: statusColor,
        ),
        const SizedBox(width: 4.0),
        Text(
          statusText,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: statusColor,
          ),
        ),
      ],
    );
  }

  Widget _buildAutosaveStatus(BuildContext context, AutosaveStatus status) {
    String statusText;
    Color statusColor;
    IconData statusIcon;
    
    if (status.hasUnsavedChanges) {
      statusColor = Theme.of(context).colorScheme.primary;
      statusIcon = Icons.circle;
      statusText = 'Unsaved';
    } else if (status.lastAutosave != null) {
      statusColor = Colors.green;
      statusIcon = Icons.cloud_done;
      final timeDiff = DateTime.now().difference(status.lastAutosave!);
      if (timeDiff.inMinutes < 1) {
        statusText = 'Saved now';
      } else if (timeDiff.inMinutes < 60) {
        statusText = 'Saved ${timeDiff.inMinutes}m ago';
      } else {
        statusText = 'Saved ${timeDiff.inHours}h ago';
      }
    } else {
      statusColor = Theme.of(context).textTheme.bodySmall?.color ?? Colors.grey;
      statusIcon = Icons.save_outlined;
      statusText = 'Not saved';
    }
    
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(
          statusIcon,
          size: 14.0,
          color: statusColor,
        ),
        const SizedBox(width: 4.0),
        Text(
          statusText,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: statusColor,
          ),
        ),
        if (status.isActive) ...[
          const SizedBox(width: 4.0),
          SizedBox(
            width: 10.0,
            height: 10.0,
            child: CircularProgressIndicator(
              strokeWidth: 1.5,
              valueColor: AlwaysStoppedAnimation<Color>(statusColor),
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildCoreVersion(BuildContext context) {
    return FutureBuilder<String>(
      future: _getCoreVersion(),
      builder: (context, snapshot) {
        final version = snapshot.data ?? '?.?.?';
        return Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.code,
              size: 14.0,
              color: Theme.of(context).textTheme.bodySmall?.color,
            ),
            const SizedBox(width: 4.0),
            Text(
              'Core v$version',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontFamily: 'monospace',
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _buildSeparator(BuildContext context) {
    return Container(
      width: 1.0,
      height: 16.0,
      color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
    );
  }

  CursorPosition _getCursorPosition(String text, int offset) {
    if (offset <= 0) {
      return const CursorPosition(line: 1, column: 1);
    }
    
    int line = 1;
    int column = 1;
    
    for (int i = 0; i < offset && i < text.length; i++) {
      if (text[i] == '\n') {
        line++;
        column = 1;
      } else {
        column++;
      }
    }
    
    return CursorPosition(line: line, column: column);
  }

  DocumentStats _getDocumentStats(EditorState state) {
    final text = state.text;
    final characters = text.length;
    
    // Count words (split by whitespace, filter empty)
    final words = text
        .split(RegExp(r'\s+'))
        .where((word) => word.isNotEmpty)
        .length;
    
    // Count lines
    final lines = text.isEmpty ? 0 : text.split('\n').length;
    
    // Count elements from document
    final elements = state.document?.elements.length ?? 0;
    
    return DocumentStats(
      characters: characters,
      words: words,
      lines: lines,
      elements: elements,
    );
  }

  Future<String> _getCoreVersion() async {
    try {
      final editorApi = createEditorApi();
      final version = await editorApi.version();
      return version;
    } catch (e) {
      return 'error';
    }
  }
}

/// Cursor position in text
class CursorPosition {
  final int line;
  final int column;
  
  const CursorPosition({
    required this.line,
    required this.column,
  });
}

/// Document statistics
class DocumentStats {
  final int characters;
  final int words;
  final int lines;
  final int elements;
  
  const DocumentStats({
    required this.characters,
    required this.words,
    required this.lines,
    required this.elements,
  });
}

/// Compact status bar for smaller screens
class CompactStatusBar extends ConsumerWidget {
  final EditorController controller;
  
  const CompactStatusBar({
    Key? key,
    required this.controller,
  }) : super(key: key);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editorState = ref.watch(editorStateProvider);
    final autosaveService = ref.watch(autosaveServiceProvider);
    
    return Container(
      height: 24.0,
      padding: const EdgeInsets.symmetric(horizontal: 8.0),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border(
          top: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: Row(
        children: [
          // Essential info only
          Text(
            'L${_getLineNumber(editorState)} • ${editorState.text.length} chars',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          
          const Spacer(),
          
          // Status indicators
          if (editorState.isLoading)
            const SizedBox(
              width: 12.0,
              height: 12.0,
              child: CircularProgressIndicator(strokeWidth: 2.0),
            )
          else if (editorState.error != null)
            Icon(
              Icons.error,
              size: 12.0,
              color: Theme.of(context).colorScheme.error,
            )
          else if (autosaveService.status.hasUnsavedChanges)
            Icon(
              Icons.circle,
              size: 8.0,
              color: Theme.of(context).colorScheme.primary,
            )
          else
            Icon(
              Icons.check,
              size: 12.0,
              color: Colors.green,
            ),
        ],
      ),
    );
  }

  int _getLineNumber(EditorState state) {
    final offset = state.selection.start;
    if (offset <= 0) return 1;
    
    int line = 1;
    for (int i = 0; i < offset && i < state.text.length; i++) {
      if (state.text[i] == '\n') line++;
    }
    return line;
  }
}