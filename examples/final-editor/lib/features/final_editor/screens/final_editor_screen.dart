import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../widgets/editor_toolbar.dart';
import '../widgets/wysiwyg_editor.dart';
import '../widgets/markdown_panel.dart';
import '../widgets/json_panel.dart';
import '../services/editor_service.dart';

class FinalEditorScreen extends ConsumerStatefulWidget {
  const FinalEditorScreen({super.key});

  @override
  ConsumerState<FinalEditorScreen> createState() => _FinalEditorScreenState();
}

class _FinalEditorScreenState extends ConsumerState<FinalEditorScreen> {
  bool _showMarkdownPanel = false;
  bool _showJsonPanel = true;
  
  @override
  void initState() {
    super.initState();
    
    // Initialize editor service
    Future.microtask(() {
      ref.read(editorServiceProvider.notifier).initialize();
    });
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorServiceProvider);
    
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.surface,
      body: CallbackShortcuts(
        bindings: _buildKeyboardShortcuts(),
        child: Focus(
          autofocus: true,
          child: Column(
            children: [
              // Toolbar
              EditorToolbar(
                onImportMarkdown: () => _importFile('markdown'),
                onImportJson: () => _importFile('json'),
                onExportMarkdown: () => _exportFile('markdown'),
                onExportJson: () => _exportFile('json'),
                onToggleMarkdown: () => setState(() {
                  _showMarkdownPanel = !_showMarkdownPanel;
                }),
                onToggleJson: () => setState(() {
                  _showJsonPanel = !_showJsonPanel;
                }),
                showMarkdownPanel: _showMarkdownPanel,
                showJsonPanel: _showJsonPanel,
              ),
              
              // Main editor area
              Expanded(
                child: Row(
                  children: [
                    // Main WYSIWYG editor
                    Expanded(
                      flex: _showMarkdownPanel || _showJsonPanel ? 2 : 1,
                      child: Container(
                        margin: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: Theme.of(context).colorScheme.background,
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
                          ),
                        ),
                        child: const WysiwygEditor(),
                      ),
                    ),
                    
                    // Side panels
                    if (_showMarkdownPanel || _showJsonPanel)
                      SizedBox(
                        width: 400,
                        child: Column(
                          children: [
                            // Markdown panel
                            if (_showMarkdownPanel)
                              Expanded(
                                flex: _showJsonPanel ? 1 : 1,
                                child: Container(
                                  margin: const EdgeInsets.fromLTRB(0, 8, 8, 4),
                                  child: MarkdownPanel(
                                    onClose: () => setState(() {
                                      _showMarkdownPanel = false;
                                    }),
                                  ),
                                ),
                              ),
                            
                            // JSON panel
                            if (_showJsonPanel)
                              Expanded(
                                flex: _showMarkdownPanel ? 1 : 1,
                                child: Container(
                                  margin: EdgeInsets.fromLTRB(
                                    0,
                                    _showMarkdownPanel ? 4 : 8,
                                    8,
                                    8,
                                  ),
                                  child: JsonPanel(
                                    onClose: () => setState(() {
                                      _showJsonPanel = false;
                                    }),
                                  ),
                                ),
                              ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
              
              // Status bar
              _buildStatusBar(editorState),
            ],
          ),
        ),
      ),
    );
  }
  
  Map<ShortcutActivator, VoidCallback> _buildKeyboardShortcuts() {
    return {
      // Text formatting
      const SingleActivator(LogicalKeyboardKey.keyB, control: true):
          () => ref.read(editorServiceProvider.notifier).toggleBold(),
      const SingleActivator(LogicalKeyboardKey.keyB, meta: true):
          () => ref.read(editorServiceProvider.notifier).toggleBold(),
      
      const SingleActivator(LogicalKeyboardKey.keyI, control: true):
          () => ref.read(editorServiceProvider.notifier).toggleItalic(),
      const SingleActivator(LogicalKeyboardKey.keyI, meta: true):
          () => ref.read(editorServiceProvider.notifier).toggleItalic(),
          
      const SingleActivator(LogicalKeyboardKey.keyU, control: true):
          () => ref.read(editorServiceProvider.notifier).toggleUnderline(),
      const SingleActivator(LogicalKeyboardKey.keyU, meta: true):
          () => ref.read(editorServiceProvider.notifier).toggleUnderline(),
      
      // Undo/Redo
      const SingleActivator(LogicalKeyboardKey.keyZ, control: true):
          () => ref.read(editorServiceProvider.notifier).undo(),
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true):
          () => ref.read(editorServiceProvider.notifier).undo(),
          
      const SingleActivator(LogicalKeyboardKey.keyY, control: true):
          () => ref.read(editorServiceProvider.notifier).redo(),
      const SingleActivator(LogicalKeyboardKey.keyZ, control: true, shift: true):
          () => ref.read(editorServiceProvider.notifier).redo(),
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true):
          () => ref.read(editorServiceProvider.notifier).redo(),
          
      // Export
      const SingleActivator(LogicalKeyboardKey.keyS, control: true):
          () => _exportFile('json'),
      const SingleActivator(LogicalKeyboardKey.keyS, meta: true):
          () => _exportFile('json'),
          
      // Select all
      const SingleActivator(LogicalKeyboardKey.keyA, control: true):
          () => ref.read(editorServiceProvider.notifier).selectAll(),
      const SingleActivator(LogicalKeyboardKey.keyA, meta: true):
          () => ref.read(editorServiceProvider.notifier).selectAll(),
    };
  }
  
  Widget _buildStatusBar(EditorState state) {
    return Container(
      height: 32,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant.withOpacity(0.3),
        border: Border(
          top: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: Row(
        children: [
          // Auto-save status
          if (state.lastAutosave != null)
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.check_circle,
                  size: 14,
                  color: Colors.green,
                ),
                const SizedBox(width: 4),
                Text(
                  'Sauvegardé · ${_formatTime(state.lastAutosave!)}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Colors.green,
                  ),
                ),
              ],
            )
          else if (state.hasUnsavedChanges)
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: 12,
                  height: 12,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    valueColor: AlwaysStoppedAnimation(
                      Theme.of(context).colorScheme.primary,
                    ),
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  'Sauvegarde...',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          
          const Spacer(),
          
          // Document stats
          if (state.documentStats != null)
            Text(
              '${state.documentStats!.characters} chars · '
              '${state.documentStats!.words} words · '
              '${state.documentStats!.elements} elements',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            
          const SizedBox(width: 16),
          
          // Core version
          Text(
            'Core ${state.coreVersion}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ],
      ),
    );
  }
  
  String _formatTime(DateTime dateTime) {
    final now = DateTime.now();
    final diff = now.difference(dateTime);
    
    if (diff.inSeconds < 60) {
      return '${diff.inSeconds}s';
    } else if (diff.inMinutes < 60) {
      return '${diff.inMinutes}m';
    } else {
      return '${dateTime.hour.toString().padLeft(2, '0')}:'
             '${dateTime.minute.toString().padLeft(2, '0')}:'
             '${dateTime.second.toString().padLeft(2, '0')}';
    }
  }
  
  Future<void> _importFile(String type) async {
    try {
      await ref.read(editorServiceProvider.notifier).importFile(type);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Erreur lors de l\'import: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }
  
  Future<void> _exportFile(String type) async {
    try {
      await ref.read(editorServiceProvider.notifier).exportFile(type);
      
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Fichier exporté avec succès'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Erreur lors de l\'export: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }
}