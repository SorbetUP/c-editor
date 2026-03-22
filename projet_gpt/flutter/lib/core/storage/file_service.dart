/// Abstract interface for file operations
/// NO platform-specific imports (dart:io, dart:html, file_picker)
abstract class FileService {
  /// Check if a file exists at the given path
  Future<bool> exists(String path);
  
  /// Read text content from a file
  Future<String> readAsString(String path);
  
  /// Write text content to a file
  Future<void> writeAsString(String path, String content);
  
  /// Delete a file
  Future<void> delete(String path);
  
  /// List files in a directory
  Future<List<String>> listFiles(String directoryPath);
  
  /// Create a directory (and parent directories if needed)
  Future<void> createDirectory(String path);
  
  /// Pick a file for opening (returns path or null if cancelled)
  Future<String?> pickFileForOpen({
    String? dialogTitle,
    List<String>? allowedExtensions,
  });
  
  /// Pick a location for saving (returns path or null if cancelled)
  Future<String?> pickFileForSave({
    String? dialogTitle,
    String? defaultName,
    List<String>? allowedExtensions,
  });
  
  /// Get platform-specific default documents directory
  Future<String> getDefaultDocumentsPath();
}