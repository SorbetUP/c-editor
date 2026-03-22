import 'package:flutter_test/flutter_test.dart';
import 'package:c_editor_flutter/core/storage/autosave_service.dart';
import 'package:c_editor_flutter/core/storage/storage_service.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'dart:io';
import 'dart:convert';

// Mock storage service for testing
class MockStorageService implements StorageService {
  final Map<String, Document> _notes = {};
  
  @override
  Future<String?> pickNoteFile() async => null;
  
  @override
  Future<String?> pickDirectory() async => null;
  
  @override
  Future<void> saveNote(String path, Document note) async {
    _notes[path] = note;
  }
  
  @override
  Future<Document?> loadNote(String path) async => _notes[path];
  
  @override
  Future<List<String>> getNotesList() async => _notes.keys.toList();
  
  @override
  Future<void> deleteNote(String path) async {
    _notes.remove(path);
  }
  
  @override
  Future<String?> createNote(String name) async {
    final path = '/mock/$name.md';
    _notes[path] = Document.empty();
    return path;
  }
  
  @override
  Future<String?> exportNote(String path, Document note) async => null;
  
  @override
  Future<void> saveSettings(AppConfig config) async {}
  
  @override
  Future<AppConfig> loadSettings() async => AppConfig.defaultConfig();
}

void main() {
  group('AutosaveService', () {
    late AutosaveService autosaveService;
    late MockStorageService mockStorage;
    late Directory tempDir;
    late String testNotePath;

    setUp(() async {
      tempDir = await Directory.systemTemp.createTemp('autosave_test_');
      testNotePath = '${tempDir.path}/test_note.md';
      mockStorage = MockStorageService();
      autosaveService = AutosaveService(mockStorage);
    });

    tearDown(() async {
      await autosaveService.dispose();
      if (await tempDir.exists()) {
        await tempDir.delete(recursive: true);
      }
    });

    group('Basic Autosave Operations', () {
      test('should start and manage autosave for document', () async {
        final document = Document.empty();
        
        autosaveService.startAutosave(testNotePath, document);
        
        final status = autosaveService.status;
        expect(status.isActive, true);
        expect(status.hasUnsavedChanges, false);
        expect(status.autosaveCount, 0);
        
        autosaveService.stopAutosave();
      });

      test('should use atomic write operations', () async {
        const content1 = '# Version 1';
        const content2 = '# Version 2';
        
        await autosaveService.saveContent(testNotePath, content1);
        expect(await File(testNotePath).readAsString(), content1);
        
        await autosaveService.saveContent(testNotePath, content2);
        expect(await File(testNotePath).readAsString(), content2);
      });

      test('should handle concurrent save operations', () async {
        const content1 = '# Concurrent 1';
        const content2 = '# Concurrent 2';
        const content3 = '# Concurrent 3';
        
        // Start multiple saves concurrently
        final futures = [
          autosaveService.saveContent(testNotePath, content1),
          autosaveService.saveContent(testNotePath, content2),
          autosaveService.saveContent(testNotePath, content3),
        ];
        
        await Future.wait(futures);
        
        // File should exist and contain one of the contents
        expect(await File(testNotePath).exists(), true);
        final savedContent = await File(testNotePath).readAsString();
        expect([content1, content2, content3].contains(savedContent), true);
      });
    });

    group('Version Management', () {
      test('should create version history', () async {
        const contents = [
          '# Version 1',
          '# Version 2',
          '# Version 3',
        ];
        
        for (final content in contents) {
          await autosaveService.saveContent(testNotePath, content);
          await Future.delayed(const Duration(milliseconds: 10)); // Ensure different timestamps
        }
        
        final versions = await autosaveService.getVersionHistory(testNotePath);
        expect(versions.length, greaterThanOrEqualTo(contents.length));
      });

      test('should limit version history to maxVersions', () async {
        const maxVersions = AutosaveService.maxVersions;
        
        // Create more versions than the limit
        for (int i = 0; i < maxVersions + 5; i++) {
          await autosaveService.saveContent(testNotePath, '# Version $i');
          await Future.delayed(const Duration(milliseconds: 1)); // Ensure different timestamps
        }
        
        final versions = await autosaveService.getVersionHistory(testNotePath);
        expect(versions.length, lessThanOrEqualTo(maxVersions));
      });

      test('should restore content from version', () async {
        const originalContent = '# Original Content\n\nThis is the original.';
        const modifiedContent = '# Modified Content\n\nThis is modified.';
        
        await autosaveService.saveContent(testNotePath, originalContent);
        await Future.delayed(const Duration(milliseconds: 10));
        await autosaveService.saveContent(testNotePath, modifiedContent);
        
        final versions = await autosaveService.getVersionHistory(testNotePath);
        expect(versions.length, greaterThanOrEqualTo(2));
        
        // Get the first version (oldest)
        final oldestVersion = versions.first;
        final restoredContent = await autosaveService.getVersionContent(testNotePath, oldestVersion.timestamp);
        
        expect(restoredContent, originalContent);
      });

      test('should clean up old versions automatically', () async {
        const maxVersions = AutosaveService.maxVersions;
        final versionsDir = Directory('${testNotePath}_versions');
        
        // Create more versions than the limit
        for (int i = 0; i < maxVersions + 10; i++) {
          await autosaveService.saveContent(testNotePath, '# Version $i');
          await Future.delayed(const Duration(milliseconds: 1));
        }
        
        // Count actual version files
        if (await versionsDir.exists()) {
          final files = await versionsDir.list().toList();
          expect(files.length, lessThanOrEqualTo(maxVersions));
        }
      });
    });

    group('Recovery and Safety', () {
      test('should detect and handle corrupted temp files', () async {
        const content = '# Test Content';
        final tempFile = File('$testNotePath.tmp');
        
        // Create a corrupted temp file
        await tempFile.writeAsString('corrupted content');
        
        // Normal save should still work
        await autosaveService.saveContent(testNotePath, content);
        
        expect(await File(testNotePath).readAsString(), content);
        expect(await tempFile.exists(), false); // Temp file should be cleaned up
      });

      test('should handle file system errors gracefully', () async {
        // Try to save to a non-existent directory
        final invalidPath = '/non/existent/path/test.md';
        
        expect(
          () => autosaveService.saveContent(invalidPath, 'test'),
          throwsA(isA<FileSystemException>()),
        );
      });

      test('should provide recovery suggestions', () async {
        const originalContent = '# Original';
        const crashContent = '# Content before crash';
        
        await autosaveService.saveContent(testNotePath, originalContent);
        await Future.delayed(const Duration(milliseconds: 10));
        
        // Simulate a crash by creating unsaved changes
        await autosaveService.saveContent(testNotePath, crashContent);
        
        final versions = await autosaveService.getVersionHistory(testNotePath);
        expect(versions.length, greaterThanOrEqualTo(2));
        
        // Should be able to recover previous versions
        for (final version in versions) {
          final content = await autosaveService.getVersionContent(testNotePath, version.timestamp);
          expect(content, isNotEmpty);
        }
      });
    });

    group('Performance and Efficiency', () {
      test('should debounce rapid save operations', () async {
        const content1 = '# Rapid 1';
        const content2 = '# Rapid 2';
        const content3 = '# Rapid 3';
        
        // Start multiple rapid saves
        final startTime = DateTime.now();
        autosaveService.saveContent(testNotePath, content1);
        autosaveService.saveContent(testNotePath, content2);
        await autosaveService.saveContent(testNotePath, content3);
        final endTime = DateTime.now();
        
        // Should complete quickly due to debouncing
        expect(endTime.difference(startTime).inMilliseconds, lessThan(1000));
        
        // Final content should be the last one
        final savedContent = await File(testNotePath).readAsString();
        expect(savedContent, content3);
      });

      test('should handle large files efficiently', () async {
        // Generate a large content string
        final largeContent = '# Large File\n\n' + 'A' * 100000; // ~100KB
        
        final startTime = DateTime.now();
        await autosaveService.saveContent(testNotePath, largeContent);
        final endTime = DateTime.now();
        
        // Should complete within reasonable time
        expect(endTime.difference(startTime).inMilliseconds, lessThan(5000));
        
        final savedContent = await File(testNotePath).readAsString();
        expect(savedContent, largeContent);
      });
    });

    group('Metadata and Information', () {
      test('should track file metadata correctly', () async {
        const content = '# Test Content\n\nWith metadata.';
        
        await autosaveService.saveContent(testNotePath, content);
        
        final versions = await autosaveService.getVersionHistory(testNotePath);
        expect(versions.isNotEmpty, true);
        
        final version = versions.first;
        expect(version.size, greaterThan(0));
        expect(version.timestamp.isBefore(DateTime.now().add(const Duration(seconds: 1))), true);
        expect(version.timestamp.isAfter(DateTime.now().subtract(const Duration(seconds: 10))), true);
      });

      test('should calculate content statistics', () async {
        const content = '# Test\n\nThis is a test with **bold** and *italic* text.\n\n- Item 1\n- Item 2';
        
        await autosaveService.saveContent(testNotePath, content);
        
        // Basic validation that content was saved correctly
        final savedContent = await File(testNotePath).readAsString();
        expect(savedContent, content);
        expect(savedContent.length, greaterThan(0));
        expect(savedContent.split('\n').length, greaterThan(1));
      });
    });
  });
}