import 'dart:ffi';
import 'dart:convert';
import 'dart:io';
import 'package:ffi/ffi.dart';

import 'package:c_editor_flutter/models/models.dart';

// C function signatures
typedef EditorInitializeC = Int32 Function();
typedef EditorInitializeDart = int Function();

typedef EditorParseMarkdownC = Int32 Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);
typedef EditorParseMarkdownDart = int Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);

typedef EditorConvertToMarkdownC = Int32 Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);
typedef EditorConvertToMarkdownDart = int Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);

typedef EditorCanonicalizeC = Int32 Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);
typedef EditorCanonicalizeDart = int Function(Pointer<Utf8>, Pointer<Pointer<Utf8>>);

typedef EditorFreeStringC = Void Function(Pointer<Utf8>);
typedef EditorFreeStringDart = void Function(Pointer<Utf8>);

typedef EditorVersionC = Pointer<Utf8> Function();
typedef EditorVersionDart = Pointer<Utf8> Function();

class EditorFFI {
  static EditorFFI? _instance;
  late DynamicLibrary _dylib;
  
  // Function bindings
  late EditorInitializeDart _initialize;
  late EditorParseMarkdownDart _parseMarkdown;
  late EditorConvertToMarkdownDart _convertToMarkdown;
  late EditorCanonicalizeDart _canonicalize;
  late EditorFreeStringDart _freeString;
  late EditorVersionDart _getVersion;

  EditorFFI._();

  static EditorFFI get instance {
    _instance ??= EditorFFI._();
    return _instance!;
  }

  Future<void> initialize() async {
    // Load the native library
    if (Platform.isWindows) {
      _dylib = DynamicLibrary.open('libeditor.dll');
    } else if (Platform.isMacOS) {
      _dylib = DynamicLibrary.open('../libeditor.a');
    } else if (Platform.isLinux) {
      _dylib = DynamicLibrary.open('../libeditor.so');
    } else {
      throw UnsupportedError('Platform not supported for FFI');
    }

    // Bind functions
    _initialize = _dylib
        .lookup<NativeFunction<EditorInitializeC>>('editor_library_init')
        .asFunction();

    _parseMarkdown = _dylib
        .lookup<NativeFunction<EditorParseMarkdownC>>('editor_parse_markdown')
        .asFunction();

    _convertToMarkdown = _dylib
        .lookup<NativeFunction<EditorConvertToMarkdownC>>('editor_json_to_markdown')
        .asFunction();

    _canonicalize = _dylib
        .lookup<NativeFunction<EditorCanonicalizeC>>('editor_json_canonicalize')
        .asFunction();

    _freeString = _dylib
        .lookup<NativeFunction<EditorFreeStringC>>('editor_free_string')
        .asFunction();

    _getVersion = _dylib
        .lookup<NativeFunction<EditorVersionC>>('editor_version')
        .asFunction();

    // Initialize the library
    final result = _initialize();
    if (result != 0) {
      throw Exception('Failed to initialize editor library: $result');
    }
  }

  String getVersion() {
    final versionPtr = _getVersion();
    final version = versionPtr.toDartString();
    return version;
  }

  Document parseMarkdown(String markdown) {
    final markdownPtr = markdown.toNativeUtf8();
    final outputPtr = calloc<Pointer<Utf8>>();

    try {
      final result = _parseMarkdown(markdownPtr, outputPtr);
      if (result != 0) {
        throw Exception('Failed to parse markdown: $result');
      }

      final jsonString = outputPtr.value.toDartString();
      _freeString(outputPtr.value);

      final json = jsonDecode(jsonString) as Map<String, dynamic>;
      return Document.fromJson(json);
    } finally {
      calloc.free(markdownPtr);
      calloc.free(outputPtr);
    }
  }

  String convertToMarkdown(Document document) {
    final jsonString = jsonEncode(document.toJson());
    final jsonPtr = jsonString.toNativeUtf8();
    final outputPtr = calloc<Pointer<Utf8>>();

    try {
      final result = _convertToMarkdown(jsonPtr, outputPtr);
      if (result != 0) {
        throw Exception('Failed to convert to markdown: $result');
      }

      final markdown = outputPtr.value.toDartString();
      _freeString(outputPtr.value);

      return markdown;
    } finally {
      calloc.free(jsonPtr);
      calloc.free(outputPtr);
    }
  }

  Document canonicalizeDocument(Document document) {
    final jsonString = jsonEncode(document.toJson());
    final jsonPtr = jsonString.toNativeUtf8();
    final outputPtr = calloc<Pointer<Utf8>>();

    try {
      final result = _canonicalize(jsonPtr, outputPtr);
      if (result != 0) {
        throw Exception('Failed to canonicalize document: $result');
      }

      final canonicalJsonString = outputPtr.value.toDartString();
      _freeString(outputPtr.value);

      final json = jsonDecode(canonicalJsonString) as Map<String, dynamic>;
      return Document.fromJson(json);
    } finally {
      calloc.free(jsonPtr);
      calloc.free(outputPtr);
    }
  }

  void dispose() {
    // No explicit cleanup needed for FFI
  }
}

// Helper extension
extension PointerUtf8Extension on Pointer<Utf8> {
  String toDartString() {
    return cast<Utf8>().toDartString();
  }
}