import 'dart:js_interop';
import 'editor_api.dart';

/// JavaScript interop declarations for WASM module
@JS('noteCore')
external JSObject? get _noteCore;

@JS()
external JSPromise<JSString> _mdToJson(JSString md, JSNumber len);

@JS()
external JSPromise<JSString> _jsonToMd(JSString json, JSNumber len);

@JS()
external JSPromise<JSString> _canon(JSString json, JSNumber len);

@JS()
external JSPromise<JSArray<JSNumber>> _version();

/// WASM implementation for web platform
class EditorApiWasm implements EditorApi {
  bool _isInitialized = false;

  /// Initialize the WASM module
  Future<void> init() async {
    if (_isInitialized) return;
    
    // TODO: Load WASM module from assets/core/note_core.wasm
    // This would typically involve:
    // 1. Fetch the WASM binary
    // 2. Instantiate with WebAssembly.instantiateStreaming
    // 3. Expose functions via window.noteCore
    
    // For now, mock initialization
    _isInitialized = true;
  }

  void _ensureInitialized() {
    if (!_isInitialized) {
      throw StateError('WASM module not initialized. Call init() first.');
    }
    if (_noteCore == null) {
      throw StateError('noteCore not available. WASM module failed to load.');
    }
  }

  @override
  Future<String> mdToJson(String md) async {
    _ensureInitialized();
    try {
      final result = await _mdToJson(md.toJS, md.length.toJS).toDart;
      return result.toDart;
    } catch (e) {
      throw Exception('Failed to convert markdown to JSON: $e');
    }
  }

  @override
  Future<String> jsonToMd(String json) async {
    _ensureInitialized();
    try {
      final result = await _jsonToMd(json.toJS, json.length.toJS).toDart;
      return result.toDart;
    } catch (e) {
      throw Exception('Failed to convert JSON to markdown: $e');
    }
  }

  @override
  Future<String> canonicalize(String json) async {
    _ensureInitialized();
    try {
      final result = await _canon(json.toJS, json.length.toJS).toDart;
      return result.toDart;
    } catch (e) {
      throw Exception('Failed to canonicalize JSON: $e');
    }
  }

  @override
  Future<(int, int, int)> version() async {
    _ensureInitialized();
    try {
      final result = await _version().toDart;
      if (result.length < 3) {
        throw Exception('Invalid version response from WASM');
      }
      return (
        result[0].toDartInt,
        result[1].toDartInt,
        result[2].toDartInt,
      );
    } catch (e) {
      throw Exception('Failed to get version: $e');
    }
  }
}

/// Factory function exposed by conditional exports
EditorApi createEditorApi() => EditorApiWasm();