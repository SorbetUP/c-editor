import 'dart:async';
import 'dart:convert';
import 'package:c_editor_flutter/models/models.dart';
import 'engine/editor_api.dart' as core;
import 'engine/bridge.dart' as bridge;

/// Results from editor operations
class EditorResult<T> {
  final T? data;
  final String? error;
  final bool success;
  
  const EditorResult._({this.data, this.error, required this.success});
  
  /// Check if the result is successful
  bool get isSuccess => success;
  
  /// Check if the result is a failure
  bool get isFailure => !success;
  
  factory EditorResult.success(T data) => 
      EditorResult._(data: data, success: true);
  
  factory EditorResult.failure(String error) => 
      EditorResult._(error: error, success: false);
}

/// Abstract interface for the editor API
abstract class EditorApi {
  /// Initialize the editor backend
  Future<EditorResult<void>> initialize();
  
  /// Parse markdown string into structured document
  Future<EditorResult<Document>> parseMarkdown(String markdown);
  
  /// Export document to markdown format
  Future<EditorResult<String>> exportToMarkdown(Document document);
  
  /// Export document to JSON format
  Future<EditorResult<String>> exportToJson(Document document);
  
  /// Simulate character-by-character editor input
  Future<EditorResult<Document>> simulateEditor(List<String> characters);
  
  /// Check if the backend is ready
  bool get isReady;
  
  /// Dispose resources
  Future<void> dispose();
}

/// Factory for creating platform-appropriate editor API
class EditorApiFactory {
  static EditorApi? _instance;
  
  /// Get singleton instance of the appropriate editor API
  static EditorApi instance() {
    return _instance ??= _EditorApiAdapter(bridge.createEditorApi());
  }
  
  /// Reset instance (for testing)
  static void reset() {
    _instance?.dispose();
    _instance = null;
  }
}

/// Adapter between the new core.EditorApi and the legacy EditorApi interface
class _EditorApiAdapter implements EditorApi {
  final core.EditorApi _coreApi;
  bool _isInitialized = false;
  
  _EditorApiAdapter(this._coreApi);

  @override
  bool get isReady => _isInitialized;

  @override
  Future<EditorResult<void>> initialize() async {
    try {
      // Try to get version to test connectivity
      await _coreApi.version();
      _isInitialized = true;
      return EditorResult.success(null);
    } catch (e) {
      return EditorResult.failure('Failed to initialize: $e');
    }
  }

  @override
  Future<EditorResult<Document>> parseMarkdown(String markdown) async {
    try {
      final json = await _coreApi.mdToJson(markdown);
      final document = Document.fromJson(jsonDecode(json));
      return EditorResult.success(document);
    } catch (e) {
      return EditorResult.failure('Failed to parse markdown: $e');
    }
  }

  @override
  Future<EditorResult<String>> exportToMarkdown(Document document) async {
    try {
      final json = jsonEncode(document.toJson());
      final markdown = await _coreApi.jsonToMd(json);
      return EditorResult.success(markdown);
    } catch (e) {
      return EditorResult.failure('Failed to export to markdown: $e');
    }
  }

  @override
  Future<EditorResult<String>> exportToJson(Document document) async {
    try {
      final json = jsonEncode(document.toJson());
      final canonical = await _coreApi.canonicalize(json);
      return EditorResult.success(canonical);
    } catch (e) {
      return EditorResult.failure('Failed to export to JSON: $e');
    }
  }

  @override
  Future<EditorResult<Document>> simulateEditor(List<String> characters) async {
    try {
      // Build markdown incrementally
      final markdown = characters.join('');
      return parseMarkdown(markdown);
    } catch (e) {
      return EditorResult.failure('Failed to simulate editor: $e');
    }
  }

  @override
  Future<void> dispose() async {
    _isInitialized = false;
  }
}

/// Exception thrown by editor operations
class EditorException implements Exception {
  final String message;
  final String? operation;
  
  const EditorException(this.message, {this.operation});
  
  @override
  String toString() {
    final op = operation != null ? ' in $operation' : '';
    return 'EditorException$op: $message';
  }
}

/// Editor capabilities that may vary by platform
class EditorCapabilities {
  final bool supportsFileIO;
  final bool supportsMultithreading;
  final bool supportsMemoryMapping;
  final int maxDocumentSize;
  final Set<String> supportedFormats;
  
  const EditorCapabilities({
    required this.supportsFileIO,
    required this.supportsMultithreading,
    required this.supportsMemoryMapping,
    required this.maxDocumentSize,
    required this.supportedFormats,
  });
  
  static const EditorCapabilities web = EditorCapabilities(
    supportsFileIO: false,
    supportsMultithreading: false,
    supportsMemoryMapping: false,
    maxDocumentSize: 10 * 1024 * 1024, // 10MB
    supportedFormats: {'markdown', 'json'},
  );
  
  static const EditorCapabilities native = EditorCapabilities(
    supportsFileIO: true,
    supportsMultithreading: true,
    supportsMemoryMapping: true,
    maxDocumentSize: 100 * 1024 * 1024, // 100MB
    supportedFormats: {'markdown', 'json'},
  );
}