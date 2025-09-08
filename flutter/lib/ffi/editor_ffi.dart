import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import '../editor_api.dart';
import '../models/document.dart';

/// FFI-based implementation for native platforms
class FfiEditorApi implements EditorApi {
  DynamicLibrary? _library;
  bool _initialized = false;
  
  @override
  bool get isReady => _initialized && _library != null;
  
  @override
  EditorCapabilities get capabilities => EditorCapabilities.native;
  
  @override
  Future<EditorResult<void>> initialize() async {
    try {
      _library = _loadLibrary();
      _initialized = true;
      return EditorResult.success(null);
    } catch (e) {
      return EditorResult.failure('Failed to initialize FFI: $e');
    }
  }
  
  @override
  Future<EditorResult<Document>> parseMarkdown(String markdown) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      // Allocate native string
      final markdownPtr = _allocateString(markdown);
      final jsonPtrPtr = calloc<Pointer<Utf8>>();
      
      try {
        // Call native function
        final result = _markdownToJson(markdownPtr, jsonPtrPtr);
        
        if (result != 0) {
          return EditorResult.failure('Parse failed with code $result');
        }
        
        // Read JSON result
        final jsonPtr = jsonPtrPtr.value;
        if (jsonPtr == nullptr) {
          return EditorResult.failure('No JSON output received');
        }
        
        final jsonString = jsonPtr.toDartString();
        final jsonData = jsonDecode(jsonString);
        final document = Document.fromJson(jsonData);
        
        // Free native memory
        _free(jsonPtr);
        
        return EditorResult.success(document);
        
      } finally {
        calloc.free(markdownPtr);
        calloc.free(jsonPtrPtr);
      }
      
    } catch (e) {
      return EditorResult.failure('Parse error: $e');
    }
  }
  
  @override
  Future<EditorResult<String>> exportToMarkdown(Document document) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      // Convert document to JSON
      final jsonString = jsonEncode(document.toJson());
      final jsonPtr = _allocateString(jsonString);
      final markdownPtrPtr = calloc<Pointer<Utf8>>();
      
      try {
        // Call native function  
        final result = _jsonToMarkdown(jsonPtr, markdownPtrPtr);
        
        if (result != 0) {
          return EditorResult.failure('Export failed with code $result');
        }
        
        // Read markdown result
        final markdownPtr = markdownPtrPtr.value;
        if (markdownPtr == nullptr) {
          return EditorResult.failure('No markdown output received');
        }
        
        final markdown = markdownPtr.toDartString();
        
        // Free native memory
        _free(markdownPtr);
        
        return EditorResult.success(markdown);
        
      } finally {
        calloc.free(jsonPtr);
        calloc.free(markdownPtrPtr);
      }
      
    } catch (e) {
      return EditorResult.failure('Export error: $e');
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
    
    try {
      // Initialize editor state
      final editorPtr = _editorInit();
      if (editorPtr == nullptr) {
        return EditorResult.failure('Failed to initialize editor state');
      }
      
      try {
        // Process characters one by one
        for (final char in characters) {
          if (char.isNotEmpty) {
            final charCode = char.codeUnitAt(0);
            _editorInput(editorPtr, charCode);
          }
        }
        
        // Get final document
        final jsonPtrPtr = calloc<Pointer<Utf8>>();
        try {
          final result = _editorGetDocument(editorPtr, jsonPtrPtr);
          
          if (result != 0) {
            return EditorResult.failure('Failed to get document with code $result');
          }
          
          final jsonPtr = jsonPtrPtr.value;
          if (jsonPtr == nullptr) {
            return EditorResult.failure('No document output received');
          }
          
          final jsonString = jsonPtr.toDartString();
          final jsonData = jsonDecode(jsonString);
          final document = Document.fromJson(jsonData);
          
          _free(jsonPtr);
          return EditorResult.success(document);
          
        } finally {
          calloc.free(jsonPtrPtr);
        }
        
      } finally {
        _editorFree(editorPtr);
      }
      
    } catch (e) {
      return EditorResult.failure('Editor simulation error: $e');
    }
  }
  
  @override
  Future<void> dispose() async {
    _initialized = false;
    _library = null;
  }
  
  // Private helper methods
  
  DynamicLibrary _loadLibrary() {
    if (Platform.isAndroid) {
      return DynamicLibrary.open('libeditor.so');
    } else if (Platform.isIOS) {
      return DynamicLibrary.executable();
    } else if (Platform.isMacOS) {
      return DynamicLibrary.open('libeditor.dylib');
    } else if (Platform.isWindows) {
      return DynamicLibrary.open('editor.dll');
    } else if (Platform.isLinux) {
      return DynamicLibrary.open('libeditor.so');
    } else {
      throw UnsupportedError('Platform not supported');
    }
  }
  
  Pointer<Utf8> _allocateString(String str) {
    final units = utf8.encode(str);
    final ptr = calloc<Uint8>(units.length + 1);
    final bytes = ptr.asTypedList(units.length + 1);
    bytes.setRange(0, units.length, units);
    bytes[units.length] = 0; // null terminator
    return ptr.cast<Utf8>();
  }
  
  // Native function bindings
  late final _markdownToJson = _library!
      .lookup<NativeFunction<Int32 Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>)>>('markdown_to_json')
      .asFunction<int Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>)>();
  
  late final _jsonToMarkdown = _library!
      .lookup<NativeFunction<Int32 Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>)>>('json_to_markdown')
      .asFunction<int Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>)>();
  
  late final _editorInit = _library!
      .lookup<NativeFunction<Pointer<Void> Function()>>('editor_init')
      .asFunction<Pointer<Void> Function()>();
  
  late final _editorInput = _library!
      .lookup<NativeFunction<Void Function(Pointer<Void>, Int32)>>('editor_input')
      .asFunction<void Function(Pointer<Void>, int)>();
  
  late final _editorGetDocument = _library!
      .lookup<NativeFunction<Int32 Function(Pointer<Void>, Pointer<Pointer<Utf8>>)>>('editor_get_document')
      .asFunction<int Function(Pointer<Void>, Pointer<Pointer<Utf8>>)>();
  
  late final _editorFree = _library!
      .lookup<NativeFunction<Void Function(Pointer<Void>)>>('editor_free')
      .asFunction<void Function(Pointer<Void>)>();
  
  late final _free = _library!
      .lookup<NativeFunction<Void Function(Pointer<Void>)>>('free')
      .asFunction<void Function(Pointer<Void>)>();
}