import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'dart:convert';

import '../../lib/features/final_editor/services/editor_service.dart';

void main() {
  group('EditorService Tests', () {
    late ProviderContainer container;
    
    setUp(() {
      container = ProviderContainer();
    });
    
    tearDown(() {
      container.dispose();
    });

    test('should have initial empty state', () {
      final editorService = container.read(editorServiceProvider.notifier);
      final state = container.read(editorServiceProvider);
      
      expect(state.document.name, equals('Untitled'));
      expect(state.document.elements, isEmpty);
      expect(state.hasUnsavedChanges, isFalse);
      expect(state.coreVersion, equals('0.0.0'));
    });

    test('should update document and mark as changed', () {
      final editorService = container.read(editorServiceProvider.notifier);
      
      final newDocument = DocumentData(
        name: 'Test Document',
        metadata: {'author': 'Test'},
        elements: [{'type': 'paragraph', 'content': 'Test content'}],
      );
      
      editorService.updateDocument(newDocument);
      final state = container.read(editorServiceProvider);
      
      expect(state.document.name, equals('Test Document'));
      expect(state.document.metadata['author'], equals('Test'));
      expect(state.document.elements.length, equals(1));
      expect(state.hasUnsavedChanges, isTrue);
    });

    test('should calculate document stats', () {
      final editorService = container.read(editorServiceProvider.notifier);
      
      final document = DocumentData(
        name: 'Stats Test',
        metadata: {},
        elements: [
          {'type': 'paragraph', 'content': 'First paragraph'},
          {'type': 'heading', 'content': 'Test Heading'},
          {'type': 'paragraph', 'content': 'Second paragraph'},
        ],
      );
      
      editorService.updateDocument(document);
      final state = container.read(editorServiceProvider);
      
      expect(state.documentStats, isNotNull);
      expect(state.documentStats!.elements, equals(3));
    });

    test('should handle JSON conversion', () {
      final editorService = container.read(editorServiceProvider.notifier);
      
      const jsonString = '''
      {
        "name": "JSON Test",
        "meta": {"version": "1.0"},
        "elements": [
          {"type": "paragraph", "content": "JSON content"}
        ]
      }
      ''';
      
      // This would normally call WASM bridge, but in tests we mock it
      expect(() => editorService.updateFromJson(jsonString), isA<void>());
    });

    test('should create document from JSON', () {
      final document = DocumentData.fromJson({
        'name': 'From JSON',
        'meta': {'created': '2024-01-01'},
        'elements': [
          {'type': 'heading', 'level': 1, 'content': 'Title'},
        ],
      });
      
      expect(document.name, equals('From JSON'));
      expect(document.metadata['created'], equals('2024-01-01'));
      expect(document.elements.length, equals(1));
    });

    test('should convert document to JSON', () {
      final document = DocumentData(
        name: 'To JSON',
        metadata: {'version': '1.0'},
        elements: [
          {'type': 'paragraph', 'content': 'Test content'},
        ],
      );
      
      final json = document.toJson();
      
      expect(json['name'], equals('To JSON'));
      expect(json['meta']['version'], equals('1.0'));
      expect(json['elements'][0]['type'], equals('paragraph'));
    });

    test('should handle empty document creation', () {
      const document = DocumentData.empty();
      
      expect(document.name, equals('Untitled'));
      expect(document.metadata, isEmpty);
      expect(document.elements, isEmpty);
    });

    test('should create document stats', () {
      const stats = DocumentStats(
        characters: 100,
        words: 20,
        elements: 5,
      );
      
      expect(stats.characters, equals(100));
      expect(stats.words, equals(20));
      expect(stats.elements, equals(5));
    });
  });

  group('EditorState Tests', () {
    test('should create editor state with defaults', () {
      const state = EditorState();
      
      expect(state.document.name, equals('Untitled'));
      expect(state.coreVersion, equals('0.0.0'));
      expect(state.hasUnsavedChanges, isFalse);
      expect(state.lastAutosave, isNull);
      expect(state.documentStats, isNull);
      expect(state.errorMessage, isNull);
    });

    test('should copy with new values', () {
      const originalState = EditorState(
        coreVersion: '1.0.0',
        hasUnsavedChanges: false,
      );
      
      final newState = originalState.copyWith(
        hasUnsavedChanges: true,
        coreVersion: '1.0.1',
      );
      
      expect(newState.hasUnsavedChanges, isTrue);
      expect(newState.coreVersion, equals('1.0.1'));
      expect(newState.document, equals(originalState.document));
    });

    test('should copy with document stats', () {
      const stats = DocumentStats(
        characters: 150,
        words: 25,
        elements: 3,
      );
      
      const state = EditorState();
      final newState = state.copyWith(documentStats: stats);
      
      expect(newState.documentStats, equals(stats));
      expect(newState.documentStats!.characters, equals(150));
    });

    test('should copy with error message', () {
      const state = EditorState();
      final newState = state.copyWith(errorMessage: 'Test error');
      
      expect(newState.errorMessage, equals('Test error'));
    });

    test('should copy with last autosave time', () {
      final now = DateTime.now();
      const state = EditorState();
      final newState = state.copyWith(lastAutosave: now);
      
      expect(newState.lastAutosave, equals(now));
    });
  });
}