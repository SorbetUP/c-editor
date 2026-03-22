import 'package:flutter_test/flutter_test.dart';
import 'package:c_editor_flutter/core/storage/autosave_service.dart';
import 'package:c_editor_flutter/core/storage/storage_service.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'dart:io';

// Simple mock storage service for testing basic autosave functionality
class SimpleStorageService implements StorageService {
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
    final path = '/test/$name.md';
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
  group('AutosaveService Basic Tests', () {
    late AutosaveService autosaveService;
    late SimpleStorageService storageService;
    
    setUp(() {
      storageService = SimpleStorageService();
      autosaveService = AutosaveService(storageService);
    });
    
    tearDown(() {
      autosaveService.dispose();
    });

    test('should initialize correctly', () {
      expect(autosaveService.status.isActive, false);
      expect(autosaveService.status.hasUnsavedChanges, false);
      expect(autosaveService.status.autosaveCount, 0);
      expect(autosaveService.status.versionCount, 0);
    });

    test('should start autosave for a document', () {
      final document = Document.empty();
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      
      expect(autosaveService.status.isActive, true);
      expect(autosaveService.status.hasUnsavedChanges, false);
    });

    test('should detect document changes', () {
      final document = Document.empty();
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      
      // Update document
      final updatedDoc = Document.empty().copyWith(
        elements: [const TextElement(id: '1', text: 'Updated content')]
      );
      
      autosaveService.updateDocument(updatedDoc);
      
      expect(autosaveService.status.hasUnsavedChanges, true);
    });

    test('should force save immediately', () async {
      final document = Document.empty();
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      
      // Update and force save
      final updatedDoc = Document.empty().copyWith(
        elements: [const TextElement(id: '1', text: 'Content to save')]
      );
      
      autosaveService.updateDocument(updatedDoc);
      await autosaveService.forceSave();
      
      // Should have saved (no unsaved changes)
      expect(autosaveService.status.hasUnsavedChanges, false);
      expect(autosaveService.status.autosaveCount, 1);
    });

    test('should handle version saves', () async {
      final document = Document.empty().copyWith(
        elements: [const TextElement(id: '1', text: 'Version content')]
      );
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      await autosaveService.forceVersionSave();
      
      expect(autosaveService.status.versionCount, 1);
      expect(autosaveService.status.lastVersion, isNotNull);
    });

    test('should stop autosave correctly', () {
      final document = Document.empty();
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      expect(autosaveService.status.isActive, true);
      
      autosaveService.stopAutosave();
      expect(autosaveService.status.isActive, false);
    });

    test('should check for crash recovery', () async {
      const path = '/test/document.md';
      
      final recovery = await autosaveService.checkCrashRecovery(path);
      // Should return null since no crash files exist
      expect(recovery, isNull);
    });

    test('should get version list', () async {
      const path = '/test/document.md';
      
      final versions = await autosaveService.getVersions(path);
      // Should return empty list initially
      expect(versions, isEmpty);
    });

    test('should handle dispose correctly', () {
      final document = Document.empty();
      const path = '/test/document.md';
      
      autosaveService.startAutosave(path, document);
      expect(autosaveService.status.isActive, true);
      
      autosaveService.dispose();
      expect(autosaveService.status.isActive, false);
    });
  });
}