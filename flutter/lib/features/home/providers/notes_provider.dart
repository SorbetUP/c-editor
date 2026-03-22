import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:path/path.dart' as path;

import '../../../core/storage/storage_service.dart';
import '../../../core/state/app_state.dart';
import '../../../core/editor/platform_editor.dart';
import 'package:c_editor_flutter/models/models.dart';

part 'notes_provider.g.dart';

@riverpod
class Notes extends _$Notes {
  @override
  FutureOr<List<String>> build() async {
    return [];
  }

  Future<void> loadNotes() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() async {
      final storageService = ref.read(storageServiceProvider);
      final appConfig = await ref.read(appStateProvider.future);
      
      return await storageService.listNotes(appConfig.defaultNotesPath);
    });
  }

  Future<String?> createNote(String title) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      final appConfig = await ref.read(appStateProvider.future);
      
      // Create note slug from title
      final slug = _createSlug(title);
      final notePath = path.join(appConfig.defaultNotesPath, slug);
      
      // Create basic document
      final document = Document(
        name: title,
        elements: [
          DocTextElement(
            spans: [DocTextSpan(text: title)],
            level: 1,
          ),
          DocTextElement(
            spans: [DocTextSpan(text: 'Commencez à écrire votre note ici...')],
          ),
        ],
        meta: DocumentMeta(
          title: title,
          created: DateTime.now(),
          modified: DateTime.now(),
        ),
      );
      
      // Save the note
      await storageService.saveNote(notePath, document);
      
      // Refresh notes list
      await loadNotes();
      
      return notePath;
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur lors de la création: $e');
      return null;
    }
  }

  Future<void> deleteNote(String notePath) async {
    try {
      // For now, we'll just refresh the list
      // TODO: Implement actual deletion in storage service
      await loadNotes();
      ref.read(uIStateProvider.notifier).showSnackBar('Note supprimée');
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur lors de la suppression: $e');
    }
  }

  String _createSlug(String title) {
    return title
        .toLowerCase()
        .replaceAll(RegExp(r'[^\w\s-]'), '')
        .replaceAll(RegExp(r'[-\s]+'), '-')
        .trim();
  }
}

// Individual note provider
@riverpod
class NoteDetails extends _$NoteDetails {
  String? _notePath;
  
  @override
  FutureOr<Document?> build(String notePath) async {
    _notePath = notePath;
    final storageService = ref.read(storageServiceProvider);
    return await storageService.loadNote(notePath);
  }
  
  String get notePath => _notePath ?? '';

  Future<void> saveNote(Document document) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      await storageService.saveNote(notePath, document);
      
      // Update the state
      state = AsyncValue.data(document);
      
      ref.read(uIStateProvider.notifier).showSnackBar('Note sauvegardée');
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur lors de la sauvegarde: $e');
    }
  }

  Future<String?> exportMarkdown() async {
    try {
      final currentDoc = state.value;
      if (currentDoc == null) return null;

      // Use platform editor to convert to markdown
      final editor = PlatformEditor.instance;
      await editor.initialize();
      
      final result = await editor.exportToMarkdown(currentDoc);
      if (!result.isSuccess) {
        throw Exception(result.error);
      }

      final storageService = ref.read(storageServiceProvider);
      final exportPath = await storageService.exportMarkdown(notePath, currentDoc);
      
      if (exportPath != null) {
        ref.read(uIStateProvider.notifier).showSnackBar('Markdown exporté: $exportPath');
      }
      
      return exportPath;
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur lors de l\'export: $e');
      return null;
    }
  }
}