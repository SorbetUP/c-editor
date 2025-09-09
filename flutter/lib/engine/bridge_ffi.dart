import 'dart:ffi' as ffi;
import 'dart:io' show Platform;
import 'package:ffi/ffi.dart';
import 'editor_api.dart';

// FFI function signatures - Native side
typedef CStrFnNative = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, ffi.Int);
typedef CFreeFnNative = ffi.Void Function(ffi.Pointer<Utf8>);
typedef CVersionFnNative = ffi.Void Function(ffi.Pointer<ffi.Int>, ffi.Pointer<ffi.Int>, ffi.Pointer<ffi.Int>);

// FFI function signatures - Dart side
typedef CStrFnDart = ffi.Pointer<Utf8> Function(ffi.Pointer<Utf8>, int);
typedef CFreeFnDart = void Function(ffi.Pointer<Utf8>);
typedef CVersionFnDart = void Function(ffi.Pointer<ffi.Int>, ffi.Pointer<ffi.Int>, ffi.Pointer<ffi.Int>);

/// FFI implementation for desktop and mobile platforms
class EditorApiFfi implements EditorApi {
  final ffi.DynamicLibrary _lib;
  
  late final CStrFnDart _mdToJson = _lib.lookupFunction<CStrFnNative, CStrFnDart>('note_md_to_json');
  late final CStrFnDart _jsonToMd = _lib.lookupFunction<CStrFnNative, CStrFnDart>('note_json_to_md');
  late final CStrFnDart _canon = _lib.lookupFunction<CStrFnNative, CStrFnDart>('note_json_canonicalize');
  late final CFreeFnDart _free = _lib.lookupFunction<CFreeFnNative, CFreeFnDart>('note_free');
  late final CVersionFnDart _version = _lib.lookupFunction<CVersionFnNative, CVersionFnDart>('note_version');

  EditorApiFfi(this._lib);

  @override
  Future<String> mdToJson(String md) async {
    final p = md.toNativeUtf8();
    try {
      final out = _mdToJson(p, md.length);
      final s = out.cast<Utf8>().toDartString();
      _free(out);
      return s;
    } finally {
      calloc.free(p);
    }
  }

  @override
  Future<String> jsonToMd(String json) async {
    final p = json.toNativeUtf8();
    try {
      final out = _jsonToMd(p, json.length);
      final s = out.cast<Utf8>().toDartString();
      _free(out);
      return s;
    } finally {
      calloc.free(p);
    }
  }

  @override
  Future<String> canonicalize(String json) async {
    final p = json.toNativeUtf8();
    try {
      final out = _canon(p, json.length);
      final s = out.cast<Utf8>().toDartString();
      _free(out);
      return s;
    } finally {
      calloc.free(p);
    }
  }

  @override
  Future<(int, int, int)> version() async {
    final a = calloc<ffi.Int>();
    final b = calloc<ffi.Int>();
    final c = calloc<ffi.Int>();
    
    try {
      _version(a, b, c);
      return (a.value, b.value, c.value);
    } finally {
      calloc.free(a);
      calloc.free(b);
      calloc.free(c);
    }
  }
}

/// Load the core library based on current platform
ffi.DynamicLibrary _loadCoreLib() {
  if (Platform.isMacOS) return ffi.DynamicLibrary.open('libnote_core.dylib');
  if (Platform.isLinux) return ffi.DynamicLibrary.open('libnote_core.so');
  if (Platform.isWindows) return ffi.DynamicLibrary.open('note_core.dll');
  if (Platform.isAndroid) return ffi.DynamicLibrary.open('libnote_core.so');
  if (Platform.isIOS) return ffi.DynamicLibrary.process();
  throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
}

/// Factory function exposed by conditional exports
EditorApi createEditorApi() {
  final lib = _loadCoreLib();
  return EditorApiFfi(lib);
}