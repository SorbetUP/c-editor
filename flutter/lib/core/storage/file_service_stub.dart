import 'file_service.dart';

/// Stub implementation for testing and analysis
class FileServiceStub implements FileService {
  @override
  Future<bool> exists(String path) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<String> readAsString(String path) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<void> writeAsString(String path, String content) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<void> delete(String path) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<List<String>> listFiles(String directoryPath) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<void> createDirectory(String path) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<String?> pickFileForOpen({
    String? dialogTitle,
    List<String>? allowedExtensions,
  }) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<String?> pickFileForSave({
    String? dialogTitle,
    String? defaultName,
    List<String>? allowedExtensions,
  }) async =>
      throw UnsupportedError('No file service available - stub implementation');

  @override
  Future<String> getDefaultDocumentsPath() async =>
      throw UnsupportedError('No file service available - stub implementation');
}

/// Factory function exposed by conditional exports
FileService createFileService() => FileServiceStub();