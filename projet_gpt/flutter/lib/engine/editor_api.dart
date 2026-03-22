/// Clean interface for editor core operations
/// NO platform-specific imports (dart:ffi, dart:js*, dart:html, dart:io)
abstract class EditorApi {
  /// Convert Markdown text to JSON document
  Future<String> mdToJson(String md);
  
  /// Convert JSON document to Markdown text  
  Future<String> jsonToMd(String json);
  
  /// Canonicalize JSON document (normalize format)
  Future<String> canonicalize(String json);
  
  /// Get core library version (major, minor, patch)
  Future<(int, int, int)> version();
}

/// Result wrapper for editor operations
sealed class EditorResult<T> {
  const EditorResult();
}

class EditorSuccess<T> extends EditorResult<T> {
  final T data;
  const EditorSuccess(this.data);
}

class EditorFailure<T> extends EditorResult<T> {
  final String error;
  const EditorFailure(this.error);
}

extension EditorResultExt<T> on EditorResult<T> {
  bool get isSuccess => this is EditorSuccess<T>;
  bool get isFailure => this is EditorFailure<T>;
  T? get data => isSuccess ? (this as EditorSuccess<T>).data : null;
  String? get error => isFailure ? (this as EditorFailure<T>).error : null;
}