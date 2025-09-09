import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/services.dart';
import 'package:c_editor_flutter/core/editor/editor_controller.dart';
import 'package:c_editor_flutter/models/models.dart';

// Mock EditorController that doesn't use FFI for testing
class MockEditorController {
  EditorState _state = const EditorState(
    text: '',
    selection: TextSelection.collapsed(offset: 0),
    isDirty: false,
    undoStack: [],
    redoStack: [],
  );

  final List<EditorState> _undoStack = [];
  final List<EditorState> _redoStack = [];
  static const int maxUndoSteps = 50;

  EditorState get state => _state;
  bool get canUndo => _undoStack.isNotEmpty;
  bool get canRedo => _redoStack.isNotEmpty;

  void setContent(String content) {
    _state = _state.copyWith(
      text: content,
      selection: const TextSelection.collapsed(offset: 0),
      isDirty: false,
      undoStack: [],
      redoStack: [],
    );
    _undoStack.clear();
    _redoStack.clear();
  }

  void insertText(String text) {
    _saveState();
    final currentText = _state.text;
    final selection = _state.selection;
    
    final newText = currentText.replaceRange(
      selection.start,
      selection.end,
      text,
    );
    
    final newOffset = selection.start + text.length;
    _updateState(
      text: newText,
      selection: TextSelection.collapsed(offset: newOffset),
      isDirty: true,
    );
  }

  void insertFormatting(String prefix, String suffix) {
    _saveState();
    final currentText = _state.text;
    final selection = _state.selection;
    
    if (selection.isCollapsed) {
      // Insert formatting markers at cursor
      final newText = currentText.replaceRange(
        selection.start,
        selection.start,
        '$prefix$suffix',
      );
      final newOffset = selection.start + prefix.length;
      _updateState(
        text: newText,
        selection: TextSelection.collapsed(offset: newOffset),
        isDirty: true,
      );
    } else {
      // Wrap selected text with formatting
      final selectedText = currentText.substring(selection.start, selection.end);
      final formattedText = '$prefix$selectedText$suffix';
      final newText = currentText.replaceRange(
        selection.start,
        selection.end,
        formattedText,
      );
      
      _updateState(
        text: newText,
        selection: TextSelection(
          baseOffset: selection.start + prefix.length,
          extentOffset: selection.start + prefix.length + selectedText.length,
        ),
        isDirty: true,
      );
    }
  }

  void updateSelection(TextSelection selection) {
    _state = _state.copyWith(selection: selection);
  }

  void undo() {
    if (_undoStack.isNotEmpty) {
      _redoStack.add(_state);
      _state = _undoStack.removeLast();
      if (_redoStack.length > maxUndoSteps) {
        _redoStack.removeAt(0);
      }
    }
  }

  void redo() {
    if (_redoStack.isNotEmpty) {
      _undoStack.add(_state);
      _state = _redoStack.removeLast();
      if (_undoStack.length > maxUndoSteps) {
        _undoStack.removeAt(0);
      }
    }
  }

  void replaceText(int start, int end, String replacement) {
    if (start < 0 || end > _state.text.length || start > end) return;
    
    _saveState();
    final newText = _state.text.replaceRange(start, end, replacement);
    final newOffset = start + replacement.length;
    _updateState(
      text: newText,
      selection: TextSelection.collapsed(offset: newOffset),
      isDirty: true,
    );
  }

  String getTextRange(int start, int end) {
    if (start < 0 || end > _state.text.length || start > end) return '';
    return _state.text.substring(start, end);
  }

  String getCurrentLine() {
    final text = _state.text;
    final offset = _state.selection.start;
    
    int lineStart = text.lastIndexOf('\n', offset - 1) + 1;
    int lineEnd = text.indexOf('\n', offset);
    if (lineEnd == -1) lineEnd = text.length;
    
    return text.substring(lineStart, lineEnd);
  }

  String getCurrentWord() {
    final text = _state.text;
    final offset = _state.selection.start;
    
    if (offset == 0 || offset >= text.length) return '';
    
    // Check if we're at a word boundary
    final char = text[offset];
    if (!RegExp(r'\w').hasMatch(char)) return '';
    
    int start = offset;
    int end = offset;
    
    // Find word boundaries
    while (start > 0 && RegExp(r'\w').hasMatch(text[start - 1])) {
      start--;
    }
    while (end < text.length && RegExp(r'\w').hasMatch(text[end])) {
      end++;
    }
    
    return text.substring(start, end);
  }

  void duplicateLine() {
    final currentLine = getCurrentLine();
    final text = _state.text;
    final offset = _state.selection.start;
    
    int lineStart = text.lastIndexOf('\n', offset - 1) + 1;
    int lineEnd = text.indexOf('\n', offset);
    
    if (lineEnd == -1) {
      // Last line, add newline and duplicate
      insertText('\n$currentLine');
    } else {
      // Insert after current line
      _saveState();
      final newText = text.substring(0, lineEnd) + '\n$currentLine' + text.substring(lineEnd);
      _updateState(
        text: newText,
        selection: _state.selection,
        isDirty: true,
      );
    }
  }

  void selectAll() {
    _state = _state.copyWith(
      selection: TextSelection(baseOffset: 0, extentOffset: _state.text.length),
    );
  }

  void clear() {
    _saveState();
    _updateState(
      text: '',
      selection: const TextSelection.collapsed(offset: 0),
      isDirty: true,
    );
  }

  void markSaved() {
    _state = _state.copyWith(isDirty: false);
  }

  void dispose() {
    // Mock disposal
  }

  void _saveState() {
    _undoStack.add(_state);
    _redoStack.clear();
    if (_undoStack.length > maxUndoSteps) {
      _undoStack.removeAt(0);
    }
  }

  void _updateState({
    String? text,
    TextSelection? selection,
    bool? isDirty,
  }) {
    _state = _state.copyWith(
      text: text ?? _state.text,
      selection: selection ?? _state.selection,
      isDirty: isDirty ?? _state.isDirty,
    );
  }
}

void main() {
  group('MockEditorController', () {
    late MockEditorController controller;

    setUp(() {
      controller = MockEditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    group('Basic Operations', () {
      test('should initialize with empty state', () {
        expect(controller.state.text, isEmpty);
        expect(controller.state.selection, const TextSelection.collapsed(offset: 0));
        expect(controller.state.isDirty, false);
        expect(controller.canUndo, false);
        expect(controller.canRedo, false);
      });

      test('should set content correctly', () {
        const content = '# Hello World\n\nThis is a test.';
        controller.setContent(content);

        expect(controller.state.text, content);
        expect(controller.state.isDirty, false);
        expect(controller.canUndo, false);
      });

      test('should insert text at cursor position', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection.collapsed(offset: 5));
        
        controller.insertText(' beautiful');
        
        expect(controller.state.text, 'Hello beautiful world');
        expect(controller.state.selection.start, 15);
        expect(controller.state.isDirty, true);
      });

      test('should replace selected text', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection(baseOffset: 6, extentOffset: 11));
        
        controller.insertText('Flutter');
        
        expect(controller.state.text, 'Hello Flutter');
        expect(controller.state.selection.start, 13);
      });
    });

    group('Formatting Operations', () {
      test('should apply bold formatting around selection', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection(baseOffset: 6, extentOffset: 11));
        
        controller.insertFormatting('**', '**');
        
        expect(controller.state.text, 'Hello **world**');
        expect(controller.state.selection.baseOffset, 8);
        expect(controller.state.selection.extentOffset, 13);
      });

      test('should insert formatting at cursor when no selection', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection.collapsed(offset: 6));
        
        controller.insertFormatting('**', '**');
        
        expect(controller.state.text, 'Hello ****world');
        expect(controller.state.selection.start, 8);
      });
    });

    group('Undo/Redo System', () {
      test('should create undo point on text insertion', () {
        controller.setContent('Initial text');
        expect(controller.canUndo, false);
        
        controller.insertText(' added');
        expect(controller.canUndo, true);
        expect(controller.canRedo, false);
        
        controller.undo();
        expect(controller.state.text, 'Initial text');
        expect(controller.canUndo, false);
        expect(controller.canRedo, true);
      });

      test('should handle multiple undo/redo operations', () {
        controller.setContent('Start');
        
        controller.insertText(' step1');
        controller.insertText(' step2');
        controller.insertText(' step3');
        
        expect(controller.state.text, 'Start step1 step2 step3');
        
        controller.undo();
        expect(controller.state.text, 'Start step1 step2');
        
        controller.undo();
        expect(controller.state.text, 'Start step1');
        
        controller.undo();
        expect(controller.state.text, 'Start');
        
        controller.redo();
        expect(controller.state.text, 'Start step1');
        
        controller.redo();
        expect(controller.state.text, 'Start step1 step2');
      });

      test('should clear redo stack on new action after undo', () {
        controller.setContent('Start');
        controller.insertText(' step1');
        controller.insertText(' step2');
        
        controller.undo();
        expect(controller.canRedo, true);
        
        controller.insertText(' new');
        expect(controller.canRedo, false);
        expect(controller.state.text, 'Start new');
      });
    });

    group('Text Manipulation', () {
      test('should replace text in range correctly', () {
        controller.setContent('Hello beautiful world');
        
        controller.replaceText(6, 15, 'amazing');
        
        expect(controller.state.text, 'Hello amazing world');
        expect(controller.state.selection.start, 13);
      });

      test('should handle invalid range gracefully', () {
        controller.setContent('Hello world');
        
        controller.replaceText(-1, 5, 'test');
        expect(controller.state.text, 'Hello world');
        
        controller.replaceText(5, 100, 'test');
        expect(controller.state.text, 'Hello world');
        
        controller.replaceText(8, 5, 'test');
        expect(controller.state.text, 'Hello world');
      });

      test('should get text range correctly', () {
        controller.setContent('Hello world test');
        
        expect(controller.getTextRange(0, 5), 'Hello');
        expect(controller.getTextRange(6, 11), 'world');
        expect(controller.getTextRange(12, 16), 'test');
        
        expect(controller.getTextRange(-1, 5), '');
        expect(controller.getTextRange(5, 100), '');
        expect(controller.getTextRange(8, 5), '');
      });
    });

    group('Line Operations', () {
      test('should get current line correctly', () {
        controller.setContent('Line 1\nLine 2\nLine 3');
        
        controller.updateSelection(const TextSelection.collapsed(offset: 9));
        expect(controller.getCurrentLine(), 'Line 2');
        
        controller.updateSelection(const TextSelection.collapsed(offset: 6));
        expect(controller.getCurrentLine(), 'Line 1');
      });

      test('should get current word correctly', () {
        controller.setContent('Hello beautiful world');
        
        controller.updateSelection(const TextSelection.collapsed(offset: 8));
        expect(controller.getCurrentWord(), 'beautiful');
        
        controller.updateSelection(const TextSelection.collapsed(offset: 5));
        expect(controller.getCurrentWord(), '');
      });

      test('should duplicate line correctly', () {
        controller.setContent('Line 1\nLine 2\nLine 3');
        
        controller.updateSelection(const TextSelection.collapsed(offset: 9));
        controller.duplicateLine();
        
        expect(controller.state.text, 'Line 1\nLine 2\nLine 2\nLine 3');
        expect(controller.canUndo, true);
      });
    });

    group('Selection Management', () {
      test('should update selection correctly', () {
        controller.setContent('Hello world');
        
        const selection = TextSelection(baseOffset: 2, extentOffset: 7);
        controller.updateSelection(selection);
        
        expect(controller.state.selection, selection);
      });

      test('should select all text', () {
        controller.setContent('Hello world');
        controller.selectAll();
        
        expect(controller.state.selection.baseOffset, 0);
        expect(controller.state.selection.extentOffset, 11);
      });

      test('should clear content', () {
        controller.setContent('Hello world');
        controller.clear();
        
        expect(controller.state.text, '');
        expect(controller.state.selection, const TextSelection.collapsed(offset: 0));
        expect(controller.canUndo, true);
      });
    });

    group('State Management', () {
      test('should mark as saved correctly', () {
        controller.setContent('Hello world');
        controller.insertText(' test');
        
        expect(controller.state.isDirty, true);
        
        controller.markSaved();
        expect(controller.state.isDirty, false);
      });

      test('should track dirty state correctly', () {
        controller.setContent('Hello');
        expect(controller.state.isDirty, false);
        
        controller.insertText(' world');
        expect(controller.state.isDirty, true);
        
        controller.undo();
        expect(controller.state.isDirty, false);
      });
    });
  });
}