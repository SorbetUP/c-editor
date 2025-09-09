import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:c_editor_flutter/models/models.dart';
import '../../../core/editor/editor_controller.dart';
import '../../../core/storage/storage_service.dart';
import '../../../widgets/editor_widget.dart';

/// Full-featured editor screen with document management
class EditorScreen extends ConsumerStatefulWidget {
  final String? documentPath;
  
  const EditorScreen({
    Key? key,
    this.documentPath,
  }) : super(key: key);

  @override
  ConsumerState<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends ConsumerState<EditorScreen> {
  Document? _currentDocument;
  String? _currentPath;
  bool _hasUnsavedChanges = false;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    if (widget.documentPath != null) {
      _loadDocument(widget.documentPath!);
    }
  }

  @override
  Widget build(BuildContext context) {
    final editorState = ref.watch(editorStateProvider);
    
    return Scaffold(
      appBar: AppBar(
        title: _buildTitle(),
        actions: [
          if (_hasUnsavedChanges)
            IconButton(
              onPressed: _saveDocument,
              icon: const Icon(Icons.save),
              tooltip: 'Save (Ctrl+S)',
            ),
          IconButton(
            onPressed: _exportDocument,
            icon: const Icon(Icons.download),
            tooltip: 'Export',
          ),
          IconButton(
            onPressed: () => _showSettings(context),
            icon: const Icon(Icons.settings),
            tooltip: 'Settings',
          ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : EditorWidget(
              initialContent: _currentDocument != null 
                  ? _documentToMarkdown(_currentDocument!)
                  : null,
              onChanged: _onTextChanged,
              onDocumentChanged: _onDocumentChanged,
              onSave: _saveDocument,
              onOpen: _openDocument,
              onNew: _newDocument,
            ),
      floatingActionButton: _buildFloatingActionButton(),
    );
  }

  Widget _buildTitle() {
    String title = 'Editor';
    
    if (_currentPath != null) {
      final fileName = _currentPath!.split('/').last;
      title = fileName;
      if (_hasUnsavedChanges) {
        title += ' •';
      }
    } else if (_hasUnsavedChanges) {
      title = 'Untitled •';
    }
    
    return Text(title);
  }

  Widget _buildFloatingActionButton() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        FloatingActionButton.small(
          onPressed: _newDocument,
          heroTag: 'new',
          child: const Icon(Icons.add),
          tooltip: 'New Document',
        ),
        const SizedBox(height: 8.0),
        FloatingActionButton.small(
          onPressed: _openDocument,
          heroTag: 'open',
          child: const Icon(Icons.folder_open),
          tooltip: 'Open Document',
        ),
      ],
    );
  }

  void _onTextChanged(String text) {
    setState(() {
      _hasUnsavedChanges = true;
    });
  }

  void _onDocumentChanged(Document document) {
    _currentDocument = document;
  }

  Future<void> _loadDocument(String path) async {
    setState(() {
      _isLoading = true;
    });

    try {
      final storageService = ref.read(storageServiceProvider);
      final document = await storageService.loadNote(path);
      
      if (document != null && mounted) {
        setState(() {
          _currentDocument = document;
          _currentPath = path;
          _hasUnsavedChanges = false;
        });
        
        // Set content in editor
        final editorController = ref.read(editorStateProvider.notifier);
        final markdown = _documentToMarkdown(document);
        editorController.setContent(markdown);
      }
    } catch (e) {
      if (mounted) {
        _showError('Failed to load document: $e');
      }
    } finally {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _saveDocument() async {
    if (_currentDocument == null) {
      await _saveAsDocument();
      return;
    }

    try {
      final storageService = ref.read(storageServiceProvider);
      final path = _currentPath ?? await _pickSavePath();
      
      if (path != null) {
        await storageService.saveNote(path, _currentDocument!);
        
        if (mounted) {
          setState(() {
            _currentPath = path;
            _hasUnsavedChanges = false;
          });
          
          _showSuccess('Document saved');
          
          // Mark as saved in editor controller
          final editorController = ref.read(editorStateProvider.notifier);
          editorController.markSaved();
        }
      }
    } catch (e) {
      _showError('Failed to save document: $e');
    }
  }

  Future<void> _saveAsDocument() async {
    try {
      final path = await _pickSavePath();
      if (path != null && _currentDocument != null) {
        final storageService = ref.read(storageServiceProvider);
        await storageService.saveNote(path, _currentDocument!);
        
        if (mounted) {
          setState(() {
            _currentPath = path;
            _hasUnsavedChanges = false;
          });
          
          _showSuccess('Document saved as $path');
        }
      }
    } catch (e) {
      _showError('Failed to save document: $e');
    }
  }

  Future<String?> _pickSavePath() async {
    final storageService = ref.read(storageServiceProvider);
    return await storageService.pickNotesDirectory();
  }

  Future<void> _openDocument() async {
    if (_hasUnsavedChanges) {
      final shouldContinue = await _showUnsavedChangesDialog();
      if (!shouldContinue) return;
    }

    try {
      final storageService = ref.read(storageServiceProvider);
      final path = await storageService.pickNoteFile();
      
      if (path != null) {
        await _loadDocument(path);
      }
    } catch (e) {
      _showError('Failed to open document: $e');
    }
  }

  Future<void> _newDocument() async {
    if (_hasUnsavedChanges) {
      final shouldContinue = await _showUnsavedChangesDialog();
      if (!shouldContinue) return;
    }

    setState(() {
      _currentDocument = null;
      _currentPath = null;
      _hasUnsavedChanges = false;
    });

    // Clear editor
    final editorController = ref.read(editorStateProvider.notifier);
    editorController.clear();
  }

  Future<void> _exportDocument() async {
    if (_currentDocument == null) {
      _showError('No document to export');
      return;
    }

    try {
      final storageService = ref.read(storageServiceProvider);
      final path = _currentPath ?? 'untitled.json';
      
      final exportPath = await storageService.exportMarkdown(path, _currentDocument!);
      
      if (exportPath != null) {
        _showSuccess('Document exported to $exportPath');
      } else {
        _showError('Failed to export document');
      }
    } catch (e) {
      _showError('Failed to export document: $e');
    }
  }

  String _documentToMarkdown(Document document) {
    // Simple conversion - in practice would use the C core
    final buffer = StringBuffer();
    
    for (final element in document.elements) {
      switch (element) {
        case DocTextElement():
          if (element.level > 0) {
            buffer.write('${'#' * element.level} ');
          }
          
          for (final span in element.spans) {
            String text = span.text;
            
            if (span.bold && span.italic) {
              text = '***$text***';
            } else if (span.bold) {
              text = '**$text**';
            } else if (span.italic) {
              text = '*$text*';
            }
            
            if (span.underline != null) {
              text = '++$text++';
            }
            
            if (span.highlight != null) {
              text = '==$text==';
            }
            
            buffer.write(text);
          }
          
          buffer.writeln();
          break;
          
        case DocImageElement():
          buffer.writeln('![${element.alt}](${element.src})');
          break;
          
        case DocTableElement():
          // Simple table representation
          for (int i = 0; i < element.rows.length; i++) {
            final row = element.rows[i];
            buffer.write('| ');
            
            for (final cell in row) {
              final cellText = cell.map((span) => span.text).join('');
              buffer.write('$cellText | ');
            }
            
            buffer.writeln();
            
            // Add separator after header
            if (i == 0) {
              buffer.write('| ');
              for (int j = 0; j < row.length; j++) {
                buffer.write('--- | ');
              }
              buffer.writeln();
            }
          }
          break;
      }
      
      buffer.writeln();
    }
    
    return buffer.toString();
  }

  Future<bool> _showUnsavedChangesDialog() async {
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Unsaved Changes'),
        content: const Text('You have unsaved changes. Do you want to continue without saving?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () async {
              await _saveDocument();
              if (mounted) {
                Navigator.pop(context, true);
              }
            },
            child: const Text('Save'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Discard'),
          ),
        ],
      ),
    );
    
    return result ?? false;
  }

  void _showSettings(BuildContext context) {
    context.go('/settings');
  }

  void _showError(String message) {
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(message),
          backgroundColor: Theme.of(context).colorScheme.error,
        ),
      );
    }
  }

  void _showSuccess(String message) {
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(message),
          backgroundColor: Colors.green,
        ),
      );
    }
  }
}