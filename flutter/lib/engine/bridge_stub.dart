import 'editor_api.dart';

/// Stub implementation for testing and analysis
/// This is never used in production - only when neither FFI nor WASM is available
class EditorApiStub implements EditorApi {
  @override 
  Future<String> mdToJson(String md) async => 
      throw UnsupportedError('No backend available - stub implementation');
      
  @override 
  Future<String> jsonToMd(String json) async => 
      throw UnsupportedError('No backend available - stub implementation');
      
  @override 
  Future<String> canonicalize(String json) async => 
      throw UnsupportedError('No backend available - stub implementation');
      
  @override 
  Future<(int, int, int)> version() async => 
      throw UnsupportedError('No backend available - stub implementation');
}

/// Factory function exposed by conditional exports
EditorApi createEditorApi() => EditorApiStub();