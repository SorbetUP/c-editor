import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:path/path.dart' as path;
import 'package:c_editor_flutter/models/models.dart';
import 'file_service.dart';
import 'file_bridge.dart' as file_bridge;

final storageServiceProvider = Provider<StorageService>((ref) {
  return _StorageServiceAdapter();
});

/// Adapter using the new FileService bridge
class _StorageServiceAdapter implements StorageService {
  final FileService _fileService = file_bridge.createFileService();
  
  @override
  Future<void> initialize() async {
    // No initialization needed for FileService
  }
  
  @override
  Future<AppConfig> loadConfig() async {
    try {
      final configPath = await _getConfigPath();
      if (await _fileService.exists(configPath)) {
        final content = await _fileService.readAsString(configPath);
        return AppConfig.fromJson(jsonDecode(content));
      }
    } catch (e) {
      print('Failed to load config: $e');
    }
    // Default config with default notes path
    final defaultPath = await _fileService.getDefaultDocumentsPath();
    return AppConfig.defaultConfig.copyWith(defaultNotesPath: defaultPath);
  }
  
  @override
  Future<void> saveConfig(AppConfig config) async {
    try {
      final configPath = await _getConfigPath();
      final content = jsonEncode(config.toJson());
      await _fileService.writeAsString(configPath, content);
    } catch (e) {
      print('Failed to save config: $e');
    }
  }
  
  Future<String> _getConfigPath() async {
    final docsPath = await _fileService.getDefaultDocumentsPath();
    return path.join(docsPath, 'config.json');
  }
  
  @override
  Future<List<String>> listNotes(String? directory) async {
    try {
      final notesPath = directory ?? await _fileService.getDefaultDocumentsPath();
      return await _fileService.listFiles(notesPath);
    } catch (e) {
      print('Failed to list notes: $e');
      return [];
    }
  }
  
  @override
  Future<Document?> loadNote(String notePath) async {
    try {
      if (await _fileService.exists(notePath)) {
        final content = await _fileService.readAsString(notePath);
        return Document.fromJson(jsonDecode(content));
      }
    } catch (e) {
      print('Failed to load note: $e');
    }
    return null;
  }
  
  @override
  Future<void> saveNote(String notePath, Document document) async {
    try {
      final content = jsonEncode(document.toJson());
      await _fileService.writeAsString(notePath, content);
    } catch (e) {
      print('Failed to save note: $e');
      throw Exception('Failed to save note: $e');
    }
  }
  
  @override
  Future<String?> exportMarkdown(String notePath, Document document) async {
    try {
      // TODO: Implement proper Markdown export
      final markdown = document.elements.map((e) {
        switch (e) {
          case DocTextElement():
            return e.spans.map((s) => s.text).join('');
          case DocImageElement():
            return '![${e.alt}](${e.src})';
          case DocTableElement():
            return '<!-- Table element -->';
          default:
            return '';
        }
      }).join('\n\n');
      
      final exportPath = notePath.replaceAll('.json', '.md');
      await _fileService.writeAsString(exportPath, markdown);
      return exportPath;
    } catch (e) {
      print('Failed to export markdown: $e');
      return null;
    }
  }
  
  @override
  Future<String?> pickNotesDirectory() async {
    return _fileService.pickFileForSave(
      dialogTitle: 'Select Notes Directory',
      defaultName: 'notes',
    );
  }
  
  @override
  Future<String?> pickNoteFile() async {
    return _fileService.pickFileForOpen(
      dialogTitle: 'Open Note',
      allowedExtensions: ['json', 'md'],
    );
  }
  
  @override
  Future<Document?> importMarkdown() async {
    // TODO: Implement markdown import
    return null;
  }
}

abstract class StorageService {
  Future<void> initialize();
  Future<AppConfig> loadConfig();
  Future<void> saveConfig(AppConfig config);
  Future<List<String>> listNotes(String? directory);
  Future<Document?> loadNote(String notePath);
  Future<void> saveNote(String notePath, Document document);
  Future<String?> exportMarkdown(String notePath, Document document);
  Future<String?> pickNotesDirectory();
  Future<String?> pickNoteFile();
  Future<Document?> importMarkdown();
}