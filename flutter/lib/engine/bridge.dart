/// Conditional exports for platform-specific editor implementations
/// This file contains NO platform-specific imports - they are delegated to implementation files
/// 
/// Selection logic:
/// - dart.library.html = Web (uses WASM)
/// - dart.library.io = Desktop/Mobile (uses FFI)
/// - Neither = Stub for testing/analysis

export 'bridge_stub.dart'
    if (dart.library.html) 'bridge_wasm.dart'
    if (dart.library.io) 'bridge_ffi.dart';