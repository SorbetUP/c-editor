import 'dart:html' as html;
import 'dart:convert';
import 'package:file_picker/file_picker.dart';
import 'file_service.dart';

/// Web implementation using browser storage APIs
class FileServiceWeb implements FileService {
  static const String _storagePrefix = 'note_editor_';
  
  /// Get storage key for a file path
  String _getStorageKey(String path) => '$_storagePrefix$path';
  
  @override
  Future<bool> exists(String path) async {
    final key = _getStorageKey(path);
    return html.window.localStorage.containsKey(key);
  }

  @override
  Future<String> readAsString(String path) async {
    final key = _getStorageKey(path);
    final content = html.window.localStorage[key];
    if (content == null) {
      throw Exception('File not found: $path');
    }
    return content;
  }

  @override
  Future<void> writeAsString(String path, String content) async {
    final key = _getStorageKey(path);
    html.window.localStorage[key] = content;
  }

  @override
  Future<void> delete(String path) async {
    final key = _getStorageKey(path);
    html.window.localStorage.remove(key);
  }

  @override
  Future<List<String>> listFiles(String directoryPath) async {
    final files = <String>[];
    final prefix = _getStorageKey(directoryPath);
    
    for (final key in html.window.localStorage.keys) {
      if (key.startsWith(prefix)) {
        // Extract the relative path
        final filePath = key.substring(_storagePrefix.length);
        files.add(filePath);
      }
    }
    
    return files..sort();
  }

  @override
  Future<void> createDirectory(String path) async {
    // No-op on web - directories are implicit in localStorage keys
  }

  @override
  Future<String?> pickFileForOpen({
    String? dialogTitle,
    List<String>? allowedExtensions,
  }) async {
    try {
      final result = await FilePicker.platform.pickFiles(
        dialogTitle: dialogTitle ?? 'Open File',
        type: allowedExtensions != null ? FileType.custom : FileType.any,
        allowedExtensions: allowedExtensions,
        allowMultiple: false,
      );
      
      if (result != null && result.files.isNotEmpty) {
        final file = result.files.first;
        if (file.bytes != null) {
          // Store the file content in localStorage with a temporary path
          final tempPath = 'temp/${file.name}';
          final content = utf8.decode(file.bytes!);
          await writeAsString(tempPath, content);
          return tempPath;
        }
      }
      return null;
    } catch (e) {
      print('Error picking file: $e');
      return null;
    }
  }

  @override
  Future<String?> pickFileForSave({
    String? dialogTitle,
    String? defaultName,
    List<String>? allowedExtensions,
  }) async {
    // On web, we can't actually save to a specific path
    // Instead, we return a path that indicates where the file would be saved
    // The actual saving will trigger a browser download
    return 'downloads/${defaultName ?? 'file.md'}';
  }

  @override
  Future<String> getDefaultDocumentsPath() async {
    // On web, use a virtual path in localStorage
    return 'documents';
  }
  
  /// Trigger a download of file content (web-specific utility)
  Future<void> downloadFile(String filename, String content) async {
    final bytes = utf8.encode(content);
    final blob = html.Blob([bytes]);
    final url = html.Url.createObjectUrlFromBlob(blob);
    
    final anchor = html.AnchorElement(href: url)
      ..setAttribute('download', filename)
      ..click();
    
    html.Url.revokeObjectUrl(url);
  }
}

/// Factory function exposed by conditional exports
FileService createFileService() => FileServiceWeb();