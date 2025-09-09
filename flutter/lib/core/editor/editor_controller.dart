import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';
import '../../editor_api.dart';

/// Editor state management
class EditorState {
  final String text;
  final Document? document;
  final String? error;
  final bool isLoading;
  final TextSelection selection;
  final bool isDirty;
  final List<EditorState> undoStack;
  final List<EditorState> redoStack;
  
  const EditorState({
    required this.text,
    this.document,
    this.error,
    this.isLoading = false,
    this.selection = const TextSelection.collapsed(offset: 0),
    this.isDirty = false,
    this.undoStack = const [],
    this.redoStack = const [],
  });
  
  EditorState copyWith({
    String? text,
    Document? document,
    String? error,
    bool? isLoading,
    TextSelection? selection,
    bool? isDirty,
    List<EditorState>? undoStack,
    List<EditorState>? redoStack,
    bool clearError = false,
  }) {
    return EditorState(
      text: text ?? this.text,
      document: document ?? this.document,
      error: clearError ? null : (error ?? this.error),
      isLoading: isLoading ?? this.isLoading,
      selection: selection ?? this.selection,
      isDirty: isDirty ?? this.isDirty,
      undoStack: undoStack ?? this.undoStack,
      redoStack: redoStack ?? this.redoStack,
    );
  }
  
  /// Create a snapshot for undo/redo
  EditorState snapshot() {
    return copyWith(
      undoStack: [],
      redoStack: [],
    );
  }
}

/// Provider for editor state
final editorStateProvider = StateNotifierProvider<EditorController, EditorState>((ref) {
  return EditorController();
});

/// Advanced editor controller with undo/redo, real-time parsing, and cursor management
class EditorController extends StateNotifier<EditorState> {
  final EditorApi _api = EditorApiFactory.instance();
  late final TextEditingController _textController;
  Timer? _parseTimer;
  bool _isInitialized = false;
  
  static const int maxUndoSteps = 50;
  static const Duration parseDelay = Duration(milliseconds: 300);
  
  EditorController() : super(const EditorState(text: '')) {
    _textController = TextEditingController();
    _textController.addListener(_onTextChanged);
    _initialize();
  }
  
  TextEditingController get textController => _textController;
  bool get isInitialized => _isInitialized;
  bool get canUndo => state.undoStack.isNotEmpty;
  bool get canRedo => state.redoStack.isNotEmpty;
  
  @override
  void dispose() {
    _parseTimer?.cancel();
    _textController.dispose();
    super.dispose();
  }
  
  /// Initialize the editor API
  Future<void> _initialize() async {
    final result = await _api.initialize();
    _isInitialized = result.isSuccess;
    
    if (!result.isSuccess) {
      state = state.copyWith(error: result.error);
    }
  }
  
  /// Set initial content
  void setContent(String content) {
    _textController.text = content;
    state = state.copyWith(
      text: content,
      isDirty: false,
      undoStack: [],
      redoStack: [],
    );
    _scheduleParseMarkdown();
  }
  
  /// Handle text changes from the controller
  void _onTextChanged() {
    if (_textController.text == state.text) return;
    
    state = state.copyWith(
      text: _textController.text,
      isDirty: true,
      selection: _textController.selection,
      clearError: true,
    );
    
    _scheduleParseMarkdown();
  }
  
  /// Update cursor position
  void updateSelection(TextSelection selection) {
    if (selection == state.selection) return;
    
    state = state.copyWith(selection: selection);
    _textController.selection = selection;
  }
  
  /// Schedule markdown parsing with debouncing
  void _scheduleParseMarkdown() {
    _parseTimer?.cancel();
    _parseTimer = Timer(parseDelay, _parseMarkdown);
  }
  
  /// Parse markdown content
  Future<void> _parseMarkdown() async {
    if (!_isInitialized) return;
    
    state = state.copyWith(isLoading: true);
    
    try {
      if (state.text.isEmpty) {
        state = state.copyWith(
          document: null,
          isLoading: false,
          clearError: true,
        );
        return;
      }
      
      final result = await _api.parseMarkdown(state.text);
      
      if (result.isSuccess) {
        state = state.copyWith(
          document: result.data,
          isLoading: false,
          clearError: true,
        );
      } else {
        state = state.copyWith(
          error: result.error,
          isLoading: false,
        );
      }
    } catch (e) {
      state = state.copyWith(
        error: 'Unexpected error: $e',
        isLoading: false,
      );
    }
  }
  
  /// Insert text at cursor position
  void insertText(String text) {
    final selection = state.selection;
    final currentText = state.text;
    
    // Save current state for undo
    _pushUndo();
    
    final newText = currentText.replaceRange(
      selection.start, 
      selection.end, 
      text
    );
    
    final newOffset = selection.start + text.length;
    final newSelection = TextSelection.collapsed(offset: newOffset);
    
    _updateText(newText, newSelection);
  }
  
  /// Insert formatting around selection
  void insertFormatting(String prefix, String suffix) {
    final selection = state.selection;
    final currentText = state.text;
    
    if (!selection.isValid) return;
    
    // Save current state for undo
    _pushUndo();
    
    final selectedText = selection.textInside(currentText);
    final formattedText = prefix + selectedText + suffix;
    
    final newText = currentText.replaceRange(
      selection.start,
      selection.end,
      formattedText,
    );
    
    // Position cursor after formatting
    final newStart = selection.start + prefix.length;
    final newEnd = newStart + selectedText.length;
    final newSelection = TextSelection(
      baseOffset: newStart,
      extentOffset: newEnd,
    );
    
    _updateText(newText, newSelection);
  }
  
  /// Replace text in range
  void replaceText(int start, int end, String replacement) {
    if (start < 0 || end > state.text.length || start > end) return;
    
    // Save current state for undo
    _pushUndo();
    
    final newText = state.text.replaceRange(start, end, replacement);
    final newOffset = start + replacement.length;
    final newSelection = TextSelection.collapsed(offset: newOffset);
    
    _updateText(newText, newSelection);
  }
  
  /// Update text and sync with controller
  void _updateText(String newText, TextSelection newSelection) {
    // Update controller without triggering listener
    _textController.removeListener(_onTextChanged);
    _textController.text = newText;
    _textController.selection = newSelection;
    _textController.addListener(_onTextChanged);
    
    // Update state
    state = state.copyWith(
      text: newText,
      selection: newSelection,
      isDirty: true,
      clearError: true,
    );
    
    _scheduleParseMarkdown();
  }
  
  /// Push current state to undo stack
  void _pushUndo() {
    final snapshot = state.snapshot();
    final newUndoStack = [...state.undoStack, snapshot];
    
    // Limit undo stack size
    if (newUndoStack.length > maxUndoSteps) {
      newUndoStack.removeAt(0);
    }
    
    state = state.copyWith(
      undoStack: newUndoStack,
      redoStack: [], // Clear redo stack on new action
    );
  }
  
  /// Undo last action
  void undo() {
    if (!canUndo) return;
    
    final undoStack = [...state.undoStack];
    final previousState = undoStack.removeLast();
    
    final redoStack = [...state.redoStack, state.snapshot()];
    
    // Restore previous state
    _updateText(previousState.text, previousState.selection);
    
    state = state.copyWith(
      undoStack: undoStack,
      redoStack: redoStack,
      isDirty: previousState.isDirty,
    );
  }
  
  /// Redo last undone action
  void redo() {
    if (!canRedo) return;
    
    final redoStack = [...state.redoStack];
    final nextState = redoStack.removeLast();
    
    final undoStack = [...state.undoStack, state.snapshot()];
    
    // Restore next state
    _updateText(nextState.text, nextState.selection);
    
    state = state.copyWith(
      undoStack: undoStack,
      redoStack: redoStack,
      isDirty: nextState.isDirty,
    );
  }
  
  /// Mark as saved (no longer dirty)
  void markSaved() {
    state = state.copyWith(isDirty: false);
  }
  
  /// Get text range
  String getTextRange(int start, int end) {
    if (start < 0 || end > state.text.length || start > end) {
      return '';
    }
    return state.text.substring(start, end);
  }
  
  /// Get current line
  String getCurrentLine() {
    final text = state.text;
    final offset = state.selection.start;
    
    int lineStart = offset;
    while (lineStart > 0 && text[lineStart - 1] != '\n') {
      lineStart--;
    }
    
    int lineEnd = offset;
    while (lineEnd < text.length && text[lineEnd] != '\n') {
      lineEnd++;
    }
    
    return text.substring(lineStart, lineEnd);
  }
  
  /// Get current word
  String getCurrentWord() {
    final text = state.text;
    final offset = state.selection.start;
    
    if (offset >= text.length) return '';
    
    int wordStart = offset;
    while (wordStart > 0 && _isWordChar(text[wordStart - 1])) {
      wordStart--;
    }
    
    int wordEnd = offset;
    while (wordEnd < text.length && _isWordChar(text[wordEnd])) {
      wordEnd++;
    }
    
    return text.substring(wordStart, wordEnd);
  }
  
  bool _isWordChar(String char) {
    return RegExp(r'\w').hasMatch(char);
  }
  
  /// Clear all content
  void clear() {
    _pushUndo();
    _updateText('', const TextSelection.collapsed(offset: 0));
  }
  
  /// Select all content
  void selectAll() {
    final newSelection = TextSelection(
      baseOffset: 0,
      extentOffset: state.text.length,
    );
    updateSelection(newSelection);
  }
  
  /// Duplicate current line
  void duplicateLine() {
    final currentLine = getCurrentLine();
    final selection = state.selection;
    final text = state.text;
    
    // Find line boundaries
    final offset = selection.start;
    int lineStart = offset;
    while (lineStart > 0 && text[lineStart - 1] != '\n') {
      lineStart--;
    }
    
    int lineEnd = offset;
    while (lineEnd < text.length && text[lineEnd] != '\n') {
      lineEnd++;
    }
    
    _pushUndo();
    
    // Insert duplicate line
    final duplicatedLine = '\n$currentLine';
    final newText = text.substring(0, lineEnd) + 
                   duplicatedLine + 
                   text.substring(lineEnd);
    
    final newOffset = lineEnd + duplicatedLine.length;
    final newSelection = TextSelection.collapsed(offset: newOffset);
    
    _updateText(newText, newSelection);
  }
}