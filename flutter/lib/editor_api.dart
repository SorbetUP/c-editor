import 'dart:async';
import 'package:flutter/foundation.dart';

import 'models/document.dart';
import 'ffi/editor_ffi.dart';
import 'wasm/editor_wasm.dart';

/// Results from editor operations
class EditorResult<T> {
  final T? data;
  final String? error;
  final bool success;
  
  const EditorResult._({this.data, this.error, required this.success});
  
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
    return _instance ??= _create();
  }
  
  static EditorApi _create() {
    if (kIsWeb) {
      return WasmEditorApi();
    } else {
      return FfiEditorApi();
    }
  }
  
  /// Reset instance (for testing)
  static void reset() {
    _instance?.dispose();
    _instance = null;
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