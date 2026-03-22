/// Conditional exports for platform-specific file service implementations
/// This file contains NO platform-specific imports

export 'file_service_stub.dart'
    if (dart.library.html) 'file_service_web.dart'
    if (dart.library.io) 'file_service_io.dart';