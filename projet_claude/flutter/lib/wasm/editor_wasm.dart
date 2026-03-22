import 'dart:async';
import 'dart:convert';
import 'dart:js' as js;
import 'dart:js_util' as js_util;

import '../editor_api.dart';
import 'package:c_editor_flutter/models/models.dart';

/// WASM-based implementation for web platform
class WasmEditorApi implements EditorApi {
  bool _initialized = false;
  js.JsObject? _wasmModule;
  
  @override
  bool get isReady => _initialized && _wasmModule != null;
  
  @override
  EditorCapabilities get capabilities => EditorCapabilities.web;
  
  @override
  Future<EditorResult<void>> initialize() async {
    try {
      // Load WASM module
      await _loadWasmModule();
      _initialized = true;
      return EditorResult.success(null);
    } catch (e) {
      return EditorResult.failure('Failed to initialize WASM: $e');
    }
  }
  
  @override
  Future<EditorResult<Document>> parseMarkdown(String markdown) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      // Call WASM function
      final jsonResult = _wasmModule!.callMethod('parseMarkdown', [markdown]);
      
      if (jsonResult == null) {
        return EditorResult.failure('WASM parse returned null');
      }
      
      final jsonData = jsonDecode(jsonResult);
      final document = Document.fromJson(jsonData);
      
      return EditorResult.success(document);
      
    } catch (e) {
      return EditorResult.failure('WASM parse error: $e');
    }
  }
  
  @override
  Future<EditorResult<String>> exportToMarkdown(Document document) async {
    if (!isReady) {
      return EditorResult.failure('Editor not initialized');
    }
    
    try {
      final jsonString = jsonEncode(document.toJson());
      final markdownResult = _wasmModule!.callMethod('jsonToMarkdown', [jsonString]);
      
      if (markdownResult == null) {
        return EditorResult.failure('WASM export returned null');
      }
      
      return EditorResult.success(markdownResult.toString());
      
    } catch (e) {
      return EditorResult.failure('WASM export error: $e');
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
      // Initialize editor
      final editorId = _wasmModule!.callMethod('editorInit', []);
      if (editorId == null) {
        return EditorResult.failure('Failed to initialize WASM editor');
      }
      
      try {
        // Process characters
        for (final char in characters) {
          if (char.isNotEmpty) {
            final charCode = char.codeUnitAt(0);
            _wasmModule!.callMethod('editorInput', [editorId, charCode]);
          }
        }
        
        // Get final document
        final jsonResult = _wasmModule!.callMethod('editorGetDocument', [editorId]);
        
        if (jsonResult == null) {
          return EditorResult.failure('Failed to get WASM editor document');
        }
        
        final jsonData = jsonDecode(jsonResult);
        final document = Document.fromJson(jsonData);
        
        return EditorResult.success(document);
        
      } finally {
        _wasmModule!.callMethod('editorFree', [editorId]);
      }
      
    } catch (e) {
      return EditorResult.failure('WASM editor simulation error: $e');
    }
  }
  
  @override
  Future<void> dispose() async {
    _initialized = false;
    _wasmModule = null;
  }
  
  // Private methods
  
  Future<void> _loadWasmModule() async {
    // Check if module already loaded
    if (js.context.hasProperty('EditorWasm')) {
      _wasmModule = js.context['EditorWasm'];
      return;
    }
    
    // Load WASM module via JavaScript
    final completer = Completer<void>();
    
    js.context['_editorWasmCallback'] = js.allowInterop((js.JsObject module) {
      _wasmModule = module;
      completer.complete();
    });
    
    js.context['_editorWasmErrorCallback'] = js.allowInterop((String error) {
      completer.completeError(EditorException('WASM load failed: $error'));
    });
    
    // Trigger WASM load
    js.context.callMethod('loadEditorWasm', []);
    
    await completer.future;
  }
}