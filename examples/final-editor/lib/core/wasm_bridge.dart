import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'package:web/web.dart' as web;

/// WASM bridge for the C core editor
class WasmBridge {
  static WasmBridge? _instance;
  static WasmBridge get instance => _instance!;
  
  late final JSObject _wasmModule;
  bool _initialized = false;
  
  static Future<void> initialize() async {
    _instance = WasmBridge._();
    await _instance!._loadWasm();
  }
  
  WasmBridge._();
  
  Future<void> _loadWasm() async {
    try {
      // Load WASM module using relative path for GitHub Pages compatibility
      final wasmPath = 'assets/core/note_core.wasm';
      
      // Check if WebAssembly is supported
      if (!web.window.has('WebAssembly')) {
        throw Exception('WebAssembly not supported in this browser');
      }
      
      // Load the WASM file
      final response = await web.window.fetch(wasmPath.toJS).toDart;
      if (!response.ok) {
        throw Exception('Failed to fetch WASM file: ${response.status}');
      }
      
      final wasmBytes = await response.arrayBuffer().toDart;
      final wasmModule = await web.WebAssembly.instantiate(wasmBytes).toDart;
      
      _wasmModule = wasmModule.instance;
      _initialized = true;
      
      print('✅ WASM Core loaded successfully');
      
      // Test version call
      final version = getVersion();
      print('📦 Core version: ${version.major}.${version.minor}.${version.patch}');
      
    } catch (e) {
      print('❌ Failed to load WASM: $e');
      rethrow;
    }
  }
  
  /// Convert Markdown to JSON
  String mdToJson(String markdown) {
    if (!_initialized) throw StateError('WASM not initialized');
    
    try {
      // Allocate memory for input string
      final inputPtr = _allocateString(markdown);
      final outputPtr = _callFunction('note_md_to_json', [inputPtr, markdown.length]);
      
      // Read output string
      final result = _readString(outputPtr);
      
      // Free allocated memory
      _callFunction('note_free', [outputPtr]);
      _callFunction('note_free', [inputPtr]);
      
      return result;
    } catch (e) {
      print('❌ Error in mdToJson: $e');
      rethrow;
    }
  }
  
  /// Convert JSON to Markdown  
  String jsonToMd(String json) {
    if (!_initialized) throw StateError('WASM not initialized');
    
    try {
      final inputPtr = _allocateString(json);
      final outputPtr = _callFunction('note_json_to_md', [inputPtr, json.length]);
      
      final result = _readString(outputPtr);
      
      _callFunction('note_free', [outputPtr]);
      _callFunction('note_free', [inputPtr]);
      
      return result;
    } catch (e) {
      print('❌ Error in jsonToMd: $e');
      rethrow;
    }
  }
  
  /// Canonicalize JSON
  String canonicalizeJson(String json) {
    if (!_initialized) throw StateError('WASM not initialized');
    
    try {
      final inputPtr = _allocateString(json);
      final outputPtr = _callFunction('note_json_canonicalize', [inputPtr, json.length]);
      
      final result = _readString(outputPtr);
      
      _callFunction('note_free', [outputPtr]);
      _callFunction('note_free', [inputPtr]);
      
      return result;
    } catch (e) {
      print('❌ Error in canonicalizeJson: $e');
      rethrow;
    }
  }
  
  /// Get core version
  CoreVersion getVersion() {
    if (!_initialized) throw StateError('WASM not initialized');
    
    try {
      // Allocate memory for version integers
      final majorPtr = _callFunction('malloc', [4]);
      final minorPtr = _callFunction('malloc', [4]);  
      final patchPtr = _callFunction('malloc', [4]);
      
      // Call version function
      _callFunction('note_version', [majorPtr, minorPtr, patchPtr]);
      
      // Read version numbers
      final major = _readInt32(majorPtr);
      final minor = _readInt32(minorPtr);
      final patch = _readInt32(patchPtr);
      
      // Free allocated memory
      _callFunction('free', [majorPtr]);
      _callFunction('free', [minorPtr]);
      _callFunction('free', [patchPtr]);
      
      return CoreVersion(major: major, minor: minor, patch: patch);
    } catch (e) {
      print('❌ Error getting version: $e');
      return CoreVersion(major: 0, minor: 0, patch: 0);
    }
  }
  
  /// Allocate string in WASM memory
  int _allocateString(String str) {
    final bytes = str.codeUnits;
    final ptr = _callFunction('malloc', [bytes.length + 1]);
    
    // Copy string bytes to WASM memory
    for (int i = 0; i < bytes.length; i++) {
      _writeUint8(ptr + i, bytes[i]);
    }
    _writeUint8(ptr + bytes.length, 0); // null terminator
    
    return ptr;
  }
  
  /// Read string from WASM memory
  String _readString(int ptr) {
    final bytes = <int>[];
    int offset = 0;
    
    while (true) {
      final byte = _readUint8(ptr + offset);
      if (byte == 0) break; // null terminator
      bytes.add(byte);
      offset++;
    }
    
    return String.fromCharCodes(bytes);
  }
  
  /// Call WASM function
  int _callFunction(String name, List<int> args) {
    final function = _wasmModule.getProperty(name.toJS) as JSFunction;
    final jsArgs = args.map((arg) => arg.toJS).toList();
    final result = function.callAsFunction(null, jsArgs.toJS);
    return (result as JSNumber).toDartInt;
  }
  
  /// Read uint8 from WASM memory
  int _readUint8(int ptr) {
    final memory = _wasmModule.getProperty('memory'.toJS) as JSObject;
    final buffer = memory.getProperty('buffer'.toJS) as JSArrayBuffer;
    final view = web.Uint8Array(buffer);
    return view.getAt(ptr).toDartInt;
  }
  
  /// Write uint8 to WASM memory
  void _writeUint8(int ptr, int value) {
    final memory = _wasmModule.getProperty('memory'.toJS) as JSObject;
    final buffer = memory.getProperty('buffer'.toJS) as JSArrayBuffer;
    final view = web.Uint8Array(buffer);
    view.setAt(ptr, value.toJS);
  }
  
  /// Read int32 from WASM memory
  int _readInt32(int ptr) {
    final memory = _wasmModule.getProperty('memory'.toJS) as JSObject;
    final buffer = memory.getProperty('buffer'.toJS) as JSArrayBuffer;
    final view = web.Int32Array(buffer);
    return view.getAt(ptr ~/ 4).toDartInt;
  }
}

/// Core version information
class CoreVersion {
  final int major;
  final int minor;
  final int patch;
  
  const CoreVersion({
    required this.major,
    required this.minor,
    required this.patch,
  });
  
  @override
  String toString() => '$major.$minor.$patch';
}