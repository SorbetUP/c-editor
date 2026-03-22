import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as path;
import 'package:path_provider/path_provider.dart';
import 'file_service.dart';

/// Native implementation for desktop and mobile platforms using dart:io
class FileServiceNative implements FileService {
  @override
  Future<bool> exists(String filePath) async {
    return File(filePath).exists();
  }

  @override
  Future<String> readAsString(String filePath) async {
    final file = File(filePath);
    if (!await file.exists()) {
      throw FileSystemException('File not found', filePath);
    }
    return file.readAsString();
  }

  @override
  Future<void> writeAsString(String filePath, String content) async {
    final file = File(filePath);
    await file.parent.create(recursive: true);
    await file.writeAsString(content);
  }

  @override
  Future<void> delete(String filePath) async {
    final file = File(filePath);
    if (await file.exists()) {
      await file.delete();
    }
  }

  @override
  Future<List<String>> listFiles(String directoryPath) async {
    final directory = Directory(directoryPath);
    if (!await directory.exists()) {
      return [];
    }
    
    final files = <String>[];
    await for (final entity in directory.list()) {
      if (entity is File) {
        files.add(entity.path);
      }
    }
    return files..sort();
  }

  @override
  Future<void> createDirectory(String directoryPath) async {
    final directory = Directory(directoryPath);
    await directory.create(recursive: true);
  }

  @override
  Future<String?> pickFileForOpen({
    String? dialogTitle,
    List<String>? allowedExtensions,
  }) async {
    final result = await FilePicker.platform.pickFiles(
      dialogTitle: dialogTitle ?? 'Open File',
      type: allowedExtensions != null ? FileType.custom : FileType.any,
      allowedExtensions: allowedExtensions,
      allowMultiple: false,
    );
    
    return result?.files.first.path;
  }

  @override
  Future<String?> pickFileForSave({
    String? dialogTitle,
    String? defaultName,
    List<String>? allowedExtensions,
  }) async {
    final result = await FilePicker.platform.saveFile(
      dialogTitle: dialogTitle ?? 'Save File',
      fileName: defaultName,
      type: allowedExtensions != null ? FileType.custom : FileType.any,
      allowedExtensions: allowedExtensions,
    );
    
    return result;
  }

  @override
  Future<String> getDefaultDocumentsPath() async {
    try {
      final directory = await getApplicationDocumentsDirectory();
      final notesDir = path.join(directory.path, 'Notes');
      await createDirectory(notesDir);
      return notesDir;
    } catch (e) {
      // Fallback to current directory if path_provider fails
      final current = Directory.current.path;
      final notesDir = path.join(current, 'notes');
      await createDirectory(notesDir);
      return notesDir;
    }
  }
}

/// Factory function exposed by conditional exports
FileService createFileService() => FileServiceNative();