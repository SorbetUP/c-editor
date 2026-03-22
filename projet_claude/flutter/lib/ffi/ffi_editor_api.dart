import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

import '../editor_api.dart';
import 'package:c_editor_flutter/models/models.dart';

/// FFI-based implementation for native platforms
class FfiEditorApi implements EditorApi {
  bool _initialized = false;
  DynamicLibrary? _dylib;
  
  // Function signatures
  late int Function() _initLibrary;
  late Pointer<Utf8> Function(Pointer<Utf8>) _parseMarkdown;
  late Pointer<Utf8> Function(Pointer<Utf8>) _jsonToMarkdown;
  late Pointer<Utf8> Function(Pointer<Utf8>) _jsonCanonicalize;
  late void Function(Pointer<Utf8>) _freeString;
  late Pointer<Utf8> Function() _getVersion;
  
  @override
  bool get isReady => _initialized && _dylib != null;
  
  @override
  Future<EditorResult<void>> initialize() async {
    try {
      _loadLibrary();
      _bindFunctions();
      
      final result = _initLibrary();
      if (result != 0) {
        return EditorResult.failure('Failed to initialize library: $result');
      }
      
      _initialized = true;
      return EditorResult.success(null);
    } catch (e) {
      return EditorResult.failure('FFI initialization failed: $e');
    }
  }
  
  @override
  Future<EditorResult<Document>> parseMarkdown(String markdown) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      final markdownPtr = markdown.toNativeUtf8();
      
      try {
        final resultPtr = _parseMarkdown(markdownPtr);
        final jsonString = resultPtr.toDartString();
        _freeString(resultPtr);
        
        final jsonData = jsonDecode(jsonString);
        final document = Document.fromJson(jsonData);
        
        return EditorResult.success(document);
      } finally {
        calloc.free(markdownPtr);
      }
    } catch (e) {
      return EditorResult.failure('FFI parse error: $e');
    }
  }
  
  @override
  Future<EditorResult<String>> exportToMarkdown(Document document) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      final jsonString = jsonEncode(document.toJson());
      final jsonPtr = jsonString.toNativeUtf8();
      
      try {
        final resultPtr = _jsonToMarkdown(jsonPtr);
        final markdown = resultPtr.toDartString();
        _freeString(resultPtr);
        
        return EditorResult.success(markdown);
      } finally {
        calloc.free(jsonPtr);
      }
    } catch (e) {
      return EditorResult.failure('FFI export error: $e');
    }
  }
  
  @override
  Future<EditorResult<String>> exportToJson(Document document) async {
    try {
      final jsonString = jsonEncode(document.toJson());
      return EditorResult.success(jsonString);
    } catch (e) {
      return EditorResult.failure('JSON serialization error: $e');
    }
  }
  
  @override
  Future<EditorResult<Document>> simulateEditor(List<String> characters) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    // For now, just concatenate characters and parse as markdown
    try {
      final text = characters.join('');
      return await parseMarkdown(text);
    } catch (e) {
      return EditorResult.failure('FFI editor simulation error: $e');
    }
  }
  
  @override
  Future<void> dispose() async {
    _initialized = false;
    _dylib = null;
  }
  
  // Private methods
  
  void _loadLibrary() {
    if (Platform.isWindows) {
      _dylib = DynamicLibrary.open('libeditor.dll');
    } else if (Platform.isMacOS) {
      _dylib = DynamicLibrary.open('../libeditor.dylib');
    } else if (Platform.isLinux) {
      _dylib = DynamicLibrary.open('../libeditor.so');
    } else {
      throw UnsupportedError('Platform not supported for FFI');
    }
  }
  
  void _bindFunctions() {
    if (_dylib == null) throw StateError('Library not loaded');
    
    _initLibrary = _dylib!
        .lookup<NativeFunction<Int32 Function()>>('editor_library_init')
        .asFunction();
    
    _parseMarkdown = _dylib!
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>)>>('editor_parse_markdown')
        .asFunction();
    
    _jsonToMarkdown = _dylib!
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>)>>('editor_json_to_markdown')
        .asFunction();
    
    _jsonCanonicalize = _dylib!
        .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>)>>('editor_json_canonicalize')
        .asFunction();
    
    _freeString = _dylib!
        .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>('editor_free_string')
        .asFunction();
    
    _getVersion = _dylib!
        .lookup<NativeFunction<Pointer<Utf8> Function()>>('editor_version')
        .asFunction();
  }
}