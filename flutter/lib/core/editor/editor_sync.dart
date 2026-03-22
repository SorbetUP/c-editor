import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'editor_controller.dart';

/// Synchronization state between editor and preview
class SyncState {
  final double editorScrollOffset;
  final double previewScrollOffset;
  final int cursorLine;
  final int previewLine;
  final bool isScrollSyncing;
  final SyncDirection lastSyncDirection;
  
  const SyncState({
    this.editorScrollOffset = 0.0,
    this.previewScrollOffset = 0.0,
    this.cursorLine = 0,
    this.previewLine = 0,
    this.isScrollSyncing = false,
    this.lastSyncDirection = SyncDirection.none,
  });
  
  SyncState copyWith({
    double? editorScrollOffset,
    double? previewScrollOffset,
    int? cursorLine,
    int? previewLine,
    bool? isScrollSyncing,
    SyncDirection? lastSyncDirection,
  }) {
    return SyncState(
      editorScrollOffset: editorScrollOffset ?? this.editorScrollOffset,
      previewScrollOffset: previewScrollOffset ?? this.previewScrollOffset,
      cursorLine: cursorLine ?? this.cursorLine,
      previewLine: previewLine ?? this.previewLine,
      isScrollSyncing: isScrollSyncing ?? this.isScrollSyncing,
      lastSyncDirection: lastSyncDirection ?? this.lastSyncDirection,
    );
  }
}

/// Sync direction enumeration
enum SyncDirection {
  none,
  editorToPreview,
  previewToEditor,
}

/// Provider for editor-preview synchronization
final editorSyncProvider = StateNotifierProvider<EditorSyncController, SyncState>((ref) {
  final editorController = ref.watch(editorStateProvider.notifier);
  return EditorSyncController(editorController);
});

/// Controller for synchronizing editor and preview scroll positions
class EditorSyncController extends StateNotifier<SyncState> {
  final EditorController _editorController;
  Timer? _syncTimer;
  
  static const Duration syncDelay = Duration(milliseconds: 100);
  static const double scrollThreshold = 5.0; // Minimum scroll difference to trigger sync
  
  EditorSyncController(this._editorController) : super(const SyncState()) {
    // Note: EditorController is a StateNotifier, we'll listen via ref.listen instead
  }
  
  @override
  void dispose() {
    _syncTimer?.cancel();
    super.dispose();
  }
  
  /// Handle editor state changes (called from widget)
  void onEditorStateChanged(EditorState editorState) {
    final cursorLine = _getCursorLine(editorState.text, editorState.selection.start);
    
    if (cursorLine != state.cursorLine) {
      state = state.copyWith(cursorLine: cursorLine);
      _scheduleSyncToPreview();
    }
  }
  
  /// Update editor scroll position
  void updateEditorScroll(double offset) {
    if ((state.editorScrollOffset - offset).abs() > scrollThreshold) {
      state = state.copyWith(
        editorScrollOffset: offset,
        lastSyncDirection: SyncDirection.editorToPreview,
      );
      _scheduleSyncToPreview();
    }
  }
  
  /// Update preview scroll position
  void updatePreviewScroll(double offset) {
    if ((state.previewScrollOffset - offset).abs() > scrollThreshold) {
      state = state.copyWith(
        previewScrollOffset: offset,
        lastSyncDirection: SyncDirection.previewToEditor,
      );
      _scheduleSyncToEditor();
    }
  }
  
  /// Schedule synchronization from editor to preview
  void _scheduleSyncToPreview() {
    if (state.lastSyncDirection == SyncDirection.previewToEditor) return;
    
    _syncTimer?.cancel();
    _syncTimer = Timer(syncDelay, () {
      if (mounted) {
        _syncToPreview();
      }
    });
  }
  
  /// Schedule synchronization from preview to editor
  void _scheduleSyncToEditor() {
    if (state.lastSyncDirection == SyncDirection.editorToPreview) return;
    
    _syncTimer?.cancel();
    _syncTimer = Timer(syncDelay, () {
      if (mounted) {
        _syncToEditor();
      }
    });
  }
  
  /// Synchronize editor to preview
  void _syncToPreview() {
    final editorState = _editorController.state;
    if (editorState.document == null) return;
    
    state = state.copyWith(isScrollSyncing: true);
    
    // Calculate preview scroll position based on cursor line
    final previewOffset = _calculatePreviewOffset(
      editorState.document!,
      state.cursorLine,
    );
    
    state = state.copyWith(
      previewScrollOffset: previewOffset,
      previewLine: state.cursorLine,
      isScrollSyncing: false,
    );
  }
  
  /// Synchronize preview to editor
  void _syncToEditor() {
    final editorState = _editorController.state;
    if (editorState.document == null) return;
    
    state = state.copyWith(isScrollSyncing: true);
    
    // Calculate editor scroll position based on preview scroll
    final editorLine = _calculateEditorLine(
      editorState.document!,
      state.previewScrollOffset,
    );
    
    final editorOffset = _calculateEditorOffset(
      editorState.text,
      editorLine,
    );
    
    state = state.copyWith(
      editorScrollOffset: editorOffset,
      cursorLine: editorLine,
      isScrollSyncing: false,
    );
  }
  
  /// Calculate cursor line from text and offset
  int _getCursorLine(String text, int offset) {
    if (offset <= 0) return 0;
    if (offset >= text.length) {
      return text.split('\n').length;
    }
    
    int line = 0;
    for (int i = 0; i < offset && i < text.length; i++) {
      if (text[i] == '\n') {
        line++;
      }
    }
    return line;
  }
  
  /// Calculate preview scroll offset based on editor line
  double _calculatePreviewOffset(Document document, int targetLine) {
    // Estimate based on document elements
    double offset = 0.0;
    int currentLine = 0;
    
    for (final element in document.elements) {
      switch (element) {
        case DocTextElement():
          final elementLines = _countElementLines(element);
          if (currentLine + elementLines >= targetLine) {
            // Found target line within this element
            final linesIntoElement = targetLine - currentLine;
            final elementHeight = _getElementHeight(element);
            offset += elementHeight * (linesIntoElement / elementLines);
            break;
          }
          currentLine += elementLines;
          offset += _getElementHeight(element);
          break;
          
        case DocImageElement():
          if (currentLine >= targetLine) break;
          currentLine += 1;
          offset += _getImageHeight(element);
          break;
          
        case DocTableElement():
          final tableLines = element.rows.length;
          if (currentLine + tableLines >= targetLine) {
            final linesIntoTable = targetLine - currentLine;
            offset += _getTableHeight(element) * (linesIntoTable / tableLines);
            break;
          }
          currentLine += tableLines;
          offset += _getTableHeight(element);
          break;
      }
      
      if (currentLine >= targetLine) break;
    }
    
    return offset;
  }
  
  /// Calculate editor line based on preview scroll offset
  int _calculateEditorLine(Document document, double targetOffset) {
    double currentOffset = 0.0;
    int currentLine = 0;
    
    for (final element in document.elements) {
      final elementHeight = _getElementHeight(element);
      
      if (currentOffset + elementHeight >= targetOffset) {
        // Target is within this element
        final offsetIntoElement = targetOffset - currentOffset;
        final progressRatio = offsetIntoElement / elementHeight;
        
        switch (element) {
          case DocTextElement():
            final elementLines = _countElementLines(element);
            return currentLine + (elementLines * progressRatio).round();
            
          case DocImageElement():
            return currentLine;
            
          case DocTableElement():
            final tableLines = element.rows.length;
            return currentLine + (tableLines * progressRatio).round();
        }
      }
      
      currentOffset += elementHeight;
      currentLine += _getElementLineCount(element);
    }
    
    return currentLine;
  }
  
  /// Calculate editor scroll offset for a specific line
  double _calculateEditorOffset(String text, int targetLine) {
    const double lineHeight = 20.0; // Estimated line height
    return targetLine * lineHeight;
  }
  
  /// Count lines in a text element
  int _countElementLines(DocTextElement element) {
    final text = element.spans.map((s) => s.text).join('');
    return text.split('\n').length;
  }
  
  /// Get element height estimate
  double _getElementHeight(DocElement element) {
    switch (element) {
      case DocTextElement():
        final lines = _countElementLines(element);
        final baseHeight = element.level > 0 ? 32.0 : 20.0; // Headers taller
        return lines * baseHeight;
        
      case DocImageElement():
        return element.height?.toDouble() ?? 200.0;
        
      case DocTableElement():
        return element.rows.length * 40.0; // Estimated row height
    }
    return 20.0; // Default fallback
  }
  
  /// Get image height estimate
  double _getImageHeight(DocImageElement element) {
    return element.height?.toDouble() ?? 200.0;
  }
  
  /// Get table height estimate  
  double _getTableHeight(DocTableElement element) {
    return element.rows.length * 40.0 + 20.0; // Header + padding
  }
  
  /// Get line count for any element
  int _getElementLineCount(DocElement element) {
    switch (element) {
      case DocTextElement():
        return _countElementLines(element);
      case DocImageElement():
        return 1;
      case DocTableElement():
        return element.rows.length;
    }
    return 1; // Default fallback
  }
  
  /// Force sync editor to preview
  void forceSyncToPreview() {
    state = state.copyWith(lastSyncDirection: SyncDirection.editorToPreview);
    _syncToPreview();
  }
  
  /// Force sync preview to editor
  void forceSyncToEditor() {
    state = state.copyWith(lastSyncDirection: SyncDirection.previewToEditor);
    _syncToEditor();
  }
  
  /// Toggle sync direction
  void toggleSyncDirection() {
    final newDirection = state.lastSyncDirection == SyncDirection.editorToPreview
        ? SyncDirection.previewToEditor
        : SyncDirection.editorToPreview;
        
    state = state.copyWith(lastSyncDirection: newDirection);
    
    if (newDirection == SyncDirection.editorToPreview) {
      _syncToPreview();
    } else {
      _syncToEditor();
    }
  }
  
  /// Reset sync state
  void reset() {
    state = const SyncState();
  }
}

/// Mixin for widgets that participate in editor-preview sync
mixin EditorSyncMixin<T extends StatefulWidget> on State<T> {
  late EditorSyncController _syncController;
  ScrollController? _scrollController;
  
  /// Initialize sync
  void initializeSync(WidgetRef ref, ScrollController scrollController) {
    _syncController = ref.read(editorSyncProvider.notifier);
    _scrollController = scrollController;
    _scrollController?.addListener(_onScroll);
  }
  
  /// Dispose sync
  void disposeSync() {
    _scrollController?.removeListener(_onScroll);
  }
  
  /// Handle scroll events
  void _onScroll() {
    if (_scrollController != null) {
      onScrollChanged(_scrollController!.offset);
    }
  }
  
  /// Override in implementation
  void onScrollChanged(double offset);
  
  /// Update sync controller with scroll offset
  void updateSyncScroll(double offset, SyncDirection direction) {
    switch (direction) {
      case SyncDirection.editorToPreview:
        _syncController.updateEditorScroll(offset);
        break;
      case SyncDirection.previewToEditor:
        _syncController.updatePreviewScroll(offset);
        break;
      case SyncDirection.none:
        break;
    }
  }
}