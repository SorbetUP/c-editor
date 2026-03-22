import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/wasm_bridge.dart';
import 'persistence_service.dart';
import 'file_service.dart';
import 'dart:convert';

/// Main editor service that coordinates all editor functionality
final editorServiceProvider = StateNotifierProvider<EditorService, EditorState>(
  (ref) => EditorService(ref),
);

class EditorService extends StateNotifier<EditorState> {
  final Ref _ref;
  late final PersistenceService _persistence;
  late final FileService _fileService;
  
  EditorService(this._ref) : super(const EditorState()) {
    _persistence = PersistenceService();
    _fileService = FileService();
  }
  
  /// Initialize the editor service
  Future<void> initialize() async {
    try {
      // Initialize persistence
      await _persistence.initialize();
      
      // Get core version
      final version = WasmBridge.instance.getVersion();
      
      // Check for crash recovery
      final crashRecovery = await _persistence.getCrashRecovery();
      
      // Load last session or create new document
      DocumentData? document;
      if (crashRecovery != null) {
        document = crashRecovery;
        // Show crash recovery notification would be handled by UI
      } else {
        document = await _persistence.loadLastSession();
      }
      
      document ??= DocumentData.empty();
      
      state = state.copyWith(
        document: document,
        coreVersion: version.toString(),
        lastAutosave: DateTime.now(),
        hasUnsavedChanges: false,
      );
      
      // Start autosave timer
      _startAutosave();
      
    } catch (e) {
      print('❌ Failed to initialize editor service: $e');
    }
  }
  
  /// Update document content and trigger autosave
  void updateDocument(DocumentData document) {
    state = state.copyWith(
      document: document,
      hasUnsavedChanges: true,
      documentStats: _calculateStats(document),
    );
  }
  
  /// Update text content from markdown editing
  void updateFromMarkdown(String markdown) {
    try {
      final json = WasmBridge.instance.mdToJson(markdown);
      final document = DocumentData.fromJson(jsonDecode(json));
      updateDocument(document);
    } catch (e) {
      print('❌ Error parsing markdown: $e');
    }
  }
  
  /// Update document from JSON editing  
  void updateFromJson(String json) {
    try {
      final canonicalJson = WasmBridge.instance.canonicalizeJson(json);
      final document = DocumentData.fromJson(jsonDecode(canonicalJson));
      updateDocument(document);
    } catch (e) {
      print('❌ Error parsing JSON: $e');
    }
  }
  
  /// Toggle bold formatting
  void toggleBold() {
    // Implementation would depend on selection/cursor position
    // For now, just trigger an update
    _triggerChange();
  }
  
  /// Toggle italic formatting
  void toggleItalic() {
    _triggerChange();
  }
  
  /// Toggle underline formatting  
  void toggleUnderline() {
    _triggerChange();
  }
  
  /// Undo last action
  void undo() {
    // Implementation would use undo stack
    _triggerChange();
  }
  
  /// Redo last undone action
  void redo() {
    // Implementation would use redo stack
    _triggerChange();
  }
  
  /// Select all content
  void selectAll() {
    // Implementation would update selection state
    _triggerChange();
  }
  
  /// Import file (markdown or JSON)
  Future<void> importFile(String type) async {
    try {
      final content = await _fileService.importFile(type);
      if (content != null) {
        if (type == 'markdown') {
          updateFromMarkdown(content);
        } else if (type == 'json') {
          updateFromJson(content);
        }
      }
    } catch (e) {
      print('❌ Error importing file: $e');
      rethrow;
    }
  }
  
  /// Export file (markdown or JSON)
  Future<void> exportFile(String type) async {
    try {
      String content;
      String filename;
      
      if (type == 'markdown') {
        content = getMarkdown();
        filename = 'document.md';
      } else {
        content = getCanonicalJson();
        filename = 'document.json';
      }
      
      await _fileService.exportFile(content, filename, type);
      
      // Mark as saved
      state = state.copyWith(
        lastAutosave: DateTime.now(),
        hasUnsavedChanges: false,
      );
      
    } catch (e) {
      print('❌ Error exporting file: $e');
      rethrow;
    }
  }
  
  /// Get current document as markdown
  String getMarkdown() {
    try {
      final json = jsonEncode(state.document.toJson());
      return WasmBridge.instance.jsonToMd(json);
    } catch (e) {
      print('❌ Error converting to markdown: $e');
      return '';
    }
  }
  
  /// Get current document as canonical JSON
  String getCanonicalJson() {
    try {
      final json = jsonEncode(state.document.toJson());
      return WasmBridge.instance.canonicalizeJson(json);
    } catch (e) {
      print('❌ Error canonicalizing JSON: $e');
      return '';
    }
  }
  
  /// Start autosave timer
  void _startAutosave() {
    // Would start a periodic timer for autosave
    // For now, just save immediately when changes occur
  }
  
  /// Trigger a generic change for testing
  void _triggerChange() {
    state = state.copyWith(
      hasUnsavedChanges: true,
      documentStats: _calculateStats(state.document),
    );
  }
  
  /// Calculate document statistics
  DocumentStats _calculateStats(DocumentData document) {
    // This would analyze the document structure
    // For now, return mock stats
    return DocumentStats(
      characters: 150,
      words: 25,
      elements: 3,
    );
  }
  
  @override
  void dispose() {
    // Clean up resources
    super.dispose();
  }
}

/// Editor state
class EditorState {
  final DocumentData document;
  final String coreVersion;
  final bool hasUnsavedChanges;
  final DateTime? lastAutosave;
  final DocumentStats? documentStats;
  final String? errorMessage;
  
  const EditorState({
    this.document = const DocumentData.empty(),
    this.coreVersion = '0.0.0',
    this.hasUnsavedChanges = false,
    this.lastAutosave,
    this.documentStats,
    this.errorMessage,
  });
  
  EditorState copyWith({
    DocumentData? document,
    String? coreVersion,
    bool? hasUnsavedChanges,
    DateTime? lastAutosave,
    DocumentStats? documentStats,
    String? errorMessage,
  }) {
    return EditorState(
      document: document ?? this.document,
      coreVersion: coreVersion ?? this.coreVersion,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
      lastAutosave: lastAutosave ?? this.lastAutosave,
      documentStats: documentStats ?? this.documentStats,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

/// Document data structure
class DocumentData {
  final String name;
  final Map<String, dynamic> metadata;
  final List<dynamic> elements;
  
  const DocumentData({
    required this.name,
    required this.metadata,
    required this.elements,
  });
  
  const DocumentData.empty()
      : name = 'Untitled',
        metadata = const {},
        elements = const [];
  
  factory DocumentData.fromJson(Map<String, dynamic> json) {
    return DocumentData(
      name: json['name'] as String? ?? 'Untitled',
      metadata: json['meta'] as Map<String, dynamic>? ?? {},
      elements: json['elements'] as List<dynamic>? ?? [],
    );
  }
  
  Map<String, dynamic> toJson() {
    return {
      'name': name,
      'meta': metadata,
      'elements': elements,
    };
  }
}

/// Document statistics
class DocumentStats {
  final int characters;
  final int words;
  final int elements;
  
  const DocumentStats({
    required this.characters,
    required this.words,
    required this.elements,
  });
}