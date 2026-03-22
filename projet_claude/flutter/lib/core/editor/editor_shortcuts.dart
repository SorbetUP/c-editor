import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'editor_controller.dart';
import 'span_text_controller.dart';

/// Editor keyboard shortcuts and actions
class EditorShortcuts extends StatelessWidget {
  final Widget child;
  final EditorController controller;
  final VoidCallback? onSave;
  final VoidCallback? onOpen;
  final VoidCallback? onNew;
  final VoidCallback? onFind;
  final VoidCallback? onReplace;
  
  const EditorShortcuts({
    Key? key,
    required this.child,
    required this.controller,
    this.onSave,
    this.onOpen,
    this.onNew,
    this.onFind,
    this.onReplace,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: _getShortcuts(),
      child: Actions(
        actions: _getActions(context),
        child: child,
      ),
    );
  }
  
  /// Get keyboard shortcuts map
  Map<LogicalKeySet, Intent> _getShortcuts() {
    return {
      // File operations
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyS): SaveIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyO): OpenIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyN): NewIntent(),
      
      // Edit operations
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyZ): UndoIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyY): RedoIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.shift, LogicalKeyboardKey.keyZ): RedoIntent(),
      
      // Text formatting
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyB): BoldIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyI): ItalicIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyU): UnderlineIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyK): LinkIntent(),
      
      // Find & Replace
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyF): FindIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyH): ReplaceIntent(),
      
      // Headers
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.digit1): Header1Intent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.digit2): Header2Intent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.digit3): Header3Intent(),
      
      // Special formatting
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.quote): CodeIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.shift, LogicalKeyboardKey.period): QuoteIntent(),
      
      // View operations
      LogicalKeySet(LogicalKeyboardKey.f11): FullscreenIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.backquote): PreviewIntent(),
      
      // Selection
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyA): SelectAllIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyD): DuplicateLineIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyL): SelectLineIntent(),
      
      // Navigation
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyG): GoToLineIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.arrowUp): ScrollUpIntent(),
      LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.arrowDown): ScrollDownIntent(),
      
      // Tab operations
      LogicalKeySet(LogicalKeyboardKey.tab): TabIntent(),
      LogicalKeySet(LogicalKeyboardKey.shift, LogicalKeyboardKey.tab): UnindentIntent(),
    };
  }
  
  /// Get actions map
  Map<Type, Action<Intent>> _getActions(BuildContext context) {
    return {
      // File operations
      SaveIntent: CallbackAction<SaveIntent>(
        onInvoke: (intent) {
          controller.markSaved();
          onSave?.call();
          return null;
        },
      ),
      OpenIntent: CallbackAction<OpenIntent>(
        onInvoke: (intent) {
          onOpen?.call();
          return null;
        },
      ),
      NewIntent: CallbackAction<NewIntent>(
        onInvoke: (intent) {
          controller.clear();
          onNew?.call();
          return null;
        },
      ),
      
      // Edit operations
      UndoIntent: CallbackAction<UndoIntent>(
        onInvoke: (intent) {
          controller.undo();
          return null;
        },
      ),
      RedoIntent: CallbackAction<RedoIntent>(
        onInvoke: (intent) {
          controller.redo();
          return null;
        },
      ),
      
      // Text formatting
      BoldIntent: CallbackAction<BoldIntent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.bold);
          return null;
        },
      ),
      ItalicIntent: CallbackAction<ItalicIntent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.italic);
          return null;
        },
      ),
      UnderlineIntent: CallbackAction<UnderlineIntent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.underline);
          return null;
        },
      ),
      LinkIntent: CallbackAction<LinkIntent>(
        onInvoke: (intent) {
          _showLinkDialog(context);
          return null;
        },
      ),
      
      // Find & Replace
      FindIntent: CallbackAction<FindIntent>(
        onInvoke: (intent) {
          onFind?.call();
          return null;
        },
      ),
      ReplaceIntent: CallbackAction<ReplaceIntent>(
        onInvoke: (intent) {
          onReplace?.call();
          return null;
        },
      ),
      
      // Headers
      Header1Intent: CallbackAction<Header1Intent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.header1);
          return null;
        },
      ),
      Header2Intent: CallbackAction<Header2Intent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.header2);
          return null;
        },
      ),
      Header3Intent: CallbackAction<Header3Intent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.header3);
          return null;
        },
      ),
      
      // Special formatting
      CodeIntent: CallbackAction<CodeIntent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.code);
          return null;
        },
      ),
      QuoteIntent: CallbackAction<QuoteIntent>(
        onInvoke: (intent) {
          _applyFormat(MarkdownFormat.quote);
          return null;
        },
      ),
      
      // View operations
      FullscreenIntent: CallbackAction<FullscreenIntent>(
        onInvoke: (intent) {
          // Implementation would toggle fullscreen
          return null;
        },
      ),
      PreviewIntent: CallbackAction<PreviewIntent>(
        onInvoke: (intent) {
          // Implementation would toggle preview
          return null;
        },
      ),
      
      // Selection
      SelectAllIntent: CallbackAction<SelectAllIntent>(
        onInvoke: (intent) {
          controller.selectAll();
          return null;
        },
      ),
      DuplicateLineIntent: CallbackAction<DuplicateLineIntent>(
        onInvoke: (intent) {
          controller.duplicateLine();
          return null;
        },
      ),
      SelectLineIntent: CallbackAction<SelectLineIntent>(
        onInvoke: (intent) {
          _selectCurrentLine();
          return null;
        },
      ),
      
      // Navigation
      GoToLineIntent: CallbackAction<GoToLineIntent>(
        onInvoke: (intent) {
          _showGoToLineDialog(context);
          return null;
        },
      ),
      ScrollUpIntent: CallbackAction<ScrollUpIntent>(
        onInvoke: (intent) {
          // Implementation would scroll up
          return null;
        },
      ),
      ScrollDownIntent: CallbackAction<ScrollDownIntent>(
        onInvoke: (intent) {
          // Implementation would scroll down
          return null;
        },
      ),
      
      // Tab operations
      TabIntent: CallbackAction<TabIntent>(
        onInvoke: (intent) {
          _handleTab();
          return null;
        },
      ),
      UnindentIntent: CallbackAction<UnindentIntent>(
        onInvoke: (intent) {
          _handleUnindent();
          return null;
        },
      ),
    };
  }
  
  /// Apply markdown formatting
  void _applyFormat(MarkdownFormat format) {
    final textController = controller.textController;
    if (textController is SpanTextEditingController) {
      textController.applyFormatting(format);
    } else {
      // Fallback for regular TextEditingController
      _applyFallbackFormat(format);
    }
  }
  
  /// Fallback formatting for regular TextEditingController
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
      case MarkdownFormat.boldItalic:
        prefix = suffix = '***';
        break;
    }
    
    controller.insertFormatting(prefix, suffix);
  }
  
  /// Show link insertion dialog
  void _showLinkDialog(BuildContext context) {
    final textController = TextEditingController();
    final urlController = TextEditingController();
    
    // Pre-fill with selected text if any
    final selection = controller.textController.selection;
    if (selection.isValid && !selection.isCollapsed) {
      textController.text = selection.textInside(controller.state.text);
    }
    
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Insert Link'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: textController,
              decoration: const InputDecoration(labelText: 'Link Text'),
              autofocus: true,
            ),
            const SizedBox(height: 8.0),
            TextField(
              controller: urlController,
              decoration: const InputDecoration(labelText: 'URL'),
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
              final text = textController.text;
              final url = urlController.text;
              if (text.isNotEmpty && url.isNotEmpty) {
                controller.insertText('[$text]($url)');
              }
              Navigator.pop(context);
            },
            child: const Text('Insert'),
          ),
        ],
      ),
    );
  }
  
  /// Show go to line dialog
  void _showGoToLineDialog(BuildContext context) {
    final lineController = TextEditingController();
    
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Go to Line'),
        content: TextField(
          controller: lineController,
          decoration: const InputDecoration(labelText: 'Line Number'),
          keyboardType: TextInputType.number,
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              final lineNumber = int.tryParse(lineController.text);
              if (lineNumber != null && lineNumber > 0) {
                _goToLine(lineNumber - 1); // Convert to 0-based index
              }
              Navigator.pop(context);
            },
            child: const Text('Go'),
          ),
        ],
      ),
    );
  }
  
  /// Go to specific line
  void _goToLine(int lineNumber) {
    final text = controller.state.text;
    final lines = text.split('\n');
    
    if (lineNumber >= lines.length) return;
    
    int offset = 0;
    for (int i = 0; i < lineNumber; i++) {
      offset += lines[i].length + 1; // +1 for newline
    }
    
    controller.updateSelection(TextSelection.collapsed(offset: offset));
  }
  
  /// Select current line
  void _selectCurrentLine() {
    final text = controller.state.text;
    final selection = controller.state.selection;
    final offset = selection.start;
    
    // Find line boundaries
    int lineStart = offset;
    while (lineStart > 0 && text[lineStart - 1] != '\n') {
      lineStart--;
    }
    
    int lineEnd = offset;
    while (lineEnd < text.length && text[lineEnd] != '\n') {
      lineEnd++;
    }
    
    controller.updateSelection(TextSelection(
      baseOffset: lineStart,
      extentOffset: lineEnd,
    ));
  }
  
  /// Handle tab insertion
  void _handleTab() {
    final selection = controller.state.selection;
    if (selection.isCollapsed) {
      // Insert tab or spaces
      controller.insertText('  '); // 2 spaces
    } else {
      // Indent selected lines
      _indentSelection();
    }
  }
  
  /// Handle unindent
  void _handleUnindent() {
    final selection = controller.state.selection;
    if (!selection.isCollapsed) {
      _unindentSelection();
    }
  }
  
  /// Indent selected lines
  void _indentSelection() {
    final text = controller.state.text;
    final selection = controller.state.selection;
    
    final selectedText = selection.textInside(text);
    final lines = selectedText.split('\n');
    final indentedLines = lines.map((line) => '  $line').toList();
    final indentedText = indentedLines.join('\n');
    
    controller.replaceText(selection.start, selection.end, indentedText);
  }
  
  /// Unindent selected lines
  void _unindentSelection() {
    final text = controller.state.text;
    final selection = controller.state.selection;
    
    final selectedText = selection.textInside(text);
    final lines = selectedText.split('\n');
    final unindentedLines = lines.map((line) {
      if (line.startsWith('  ')) {
        return line.substring(2);
      } else if (line.startsWith('\t')) {
        return line.substring(1);
      }
      return line;
    }).toList();
    final unindentedText = unindentedLines.join('\n');
    
    controller.replaceText(selection.start, selection.end, unindentedText);
  }
}

// Intent classes for keyboard shortcuts
class SaveIntent extends Intent {}
class OpenIntent extends Intent {}
class NewIntent extends Intent {}
class UndoIntent extends Intent {}
class RedoIntent extends Intent {}
class BoldIntent extends Intent {}
class ItalicIntent extends Intent {}
class UnderlineIntent extends Intent {}
class LinkIntent extends Intent {}
class FindIntent extends Intent {}
class ReplaceIntent extends Intent {}
class Header1Intent extends Intent {}
class Header2Intent extends Intent {}
class Header3Intent extends Intent {}
class CodeIntent extends Intent {}
class QuoteIntent extends Intent {}
class FullscreenIntent extends Intent {}
class PreviewIntent extends Intent {}
class SelectAllIntent extends Intent {}
class DuplicateLineIntent extends Intent {}
class SelectLineIntent extends Intent {}
class GoToLineIntent extends Intent {}
class ScrollUpIntent extends Intent {}
class ScrollDownIntent extends Intent {}
class TabIntent extends Intent {}
class UnindentIntent extends Intent {}

/// Utility widget for displaying keyboard shortcuts help
class ShortcutsHelpDialog extends StatelessWidget {
  const ShortcutsHelpDialog({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Keyboard Shortcuts'),
      content: SizedBox(
        width: 400.0,
        height: 500.0,
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildShortcutSection('File Operations', [
                _ShortcutItem('Ctrl+S', 'Save'),
                _ShortcutItem('Ctrl+O', 'Open'),
                _ShortcutItem('Ctrl+N', 'New'),
              ]),
              _buildShortcutSection('Edit Operations', [
                _ShortcutItem('Ctrl+Z', 'Undo'),
                _ShortcutItem('Ctrl+Y', 'Redo'),
                _ShortcutItem('Ctrl+A', 'Select All'),
                _ShortcutItem('Ctrl+D', 'Duplicate Line'),
              ]),
              _buildShortcutSection('Text Formatting', [
                _ShortcutItem('Ctrl+B', 'Bold'),
                _ShortcutItem('Ctrl+I', 'Italic'),
                _ShortcutItem('Ctrl+U', 'Underline'),
                _ShortcutItem('Ctrl+K', 'Insert Link'),
                _ShortcutItem('Ctrl+\'', 'Code'),
              ]),
              _buildShortcutSection('Headers', [
                _ShortcutItem('Ctrl+1', 'Header 1'),
                _ShortcutItem('Ctrl+2', 'Header 2'),
                _ShortcutItem('Ctrl+3', 'Header 3'),
              ]),
              _buildShortcutSection('Navigation', [
                _ShortcutItem('Ctrl+F', 'Find'),
                _ShortcutItem('Ctrl+H', 'Replace'),
                _ShortcutItem('Ctrl+G', 'Go to Line'),
                _ShortcutItem('F11', 'Fullscreen'),
              ]),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
  
  Widget _buildShortcutSection(String title, List<_ShortcutItem> shortcuts) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 16.0, bottom: 8.0),
          child: Text(
            title,
            style: const TextStyle(fontWeight: FontWeight.bold),
          ),
        ),
        ...shortcuts.map((item) => Padding(
          padding: const EdgeInsets.symmetric(vertical: 2.0),
          child: Row(
            children: [
              SizedBox(
                width: 100.0,
                child: Text(
                  item.shortcut,
                  style: const TextStyle(fontFamily: 'monospace'),
                ),
              ),
              Text(item.description),
            ],
          ),
        )),
      ],
    );
  }
}

class _ShortcutItem {
  final String shortcut;
  final String description;
  
  const _ShortcutItem(this.shortcut, this.description);
}