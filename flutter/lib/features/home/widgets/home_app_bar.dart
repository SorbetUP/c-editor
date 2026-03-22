import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';

import '../../../core/routing/app_router.dart';
import '../../../core/storage/storage_service.dart';
import '../../../core/state/app_state.dart';

class HomeAppBar extends ConsumerWidget implements PreferredSizeWidget {
  const HomeAppBar({super.key});

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return AppBar(
      title: const Text('Mes Notes'),
      actions: [
        IconButton(
          onPressed: () => _showImportOptions(context, ref),
          icon: const Icon(Icons.file_download),
          tooltip: 'Importer',
        ),
        IconButton(
          onPressed: () => _showFolderOptions(context, ref),
          icon: const Icon(Icons.folder_open),
          tooltip: 'Dossier',
        ),
        IconButton(
          onPressed: () => AppNavigation.toSettings(context),
          icon: const Icon(Icons.settings),
          tooltip: 'Paramètres',
        ),
      ],
    );
  }

  void _showImportOptions(BuildContext context, WidgetRef ref) {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Importer',
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            ListTile(
              leading: const Icon(Icons.description),
              title: const Text('Importer Markdown'),
              subtitle: const Text('Convertir un fichier .md en note'),
              onTap: () {
                Navigator.pop(context);
                _importMarkdown(context, ref);
              },
            ),
            ListTile(
              leading: const Icon(Icons.data_object),
              title: const Text('Ouvrir JSON'),
              subtitle: const Text('Ouvrir un fichier note.json'),
              onTap: () {
                Navigator.pop(context);
                _openJson(context, ref);
              },
            ),
            const SizedBox(height: 16),
          ],
        ),
      ),
    );
  }

  void _showFolderOptions(BuildContext context, WidgetRef ref) {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Padding(
              padding: EdgeInsets.all(16),
              child: Text(
                'Dossier',
                style: TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            ListTile(
              leading: const Icon(Icons.folder),
              title: const Text('Changer de dossier'),
              subtitle: const Text('Sélectionner un autre dossier de notes'),
              onTap: () {
                Navigator.pop(context);
                _pickNotesFolder(context, ref);
              },
            ),
            ListTile(
              leading: const Icon(Icons.refresh),
              title: const Text('Actualiser'),
              subtitle: const Text('Recharger la liste des notes'),
              onTap: () {
                Navigator.pop(context);
                _refreshNotes(ref);
              },
            ),
            const SizedBox(height: 16),
          ],
        ),
      ),
    );
  }

  Future<void> _importMarkdown(BuildContext context, WidgetRef ref) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      final document = await storageService.importMarkdown();
      
      if (document != null) {
        ref.read(uIStateProvider.notifier).showSnackBar('Markdown importé avec succès');
        // TODO: Navigate to viewer with imported document
      }
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur d\'import: $e');
    }
  }

  Future<void> _openJson(BuildContext context, WidgetRef ref) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      final filePath = await storageService.pickNoteFile();
      
      if (filePath != null) {
        AppNavigation.toViewer(context, notePath: filePath);
      }
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur d\'ouverture: $e');
    }
  }

  Future<void> _pickNotesFolder(BuildContext context, WidgetRef ref) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      final folderPath = await storageService.pickNotesDirectory();
      
      if (folderPath != null) {
        final currentConfig = await ref.read(appStateProvider.future);
        final newConfig = currentConfig.copyWith(defaultNotesPath: folderPath);
        
        await ref.read(appStateProvider.notifier).updateConfig(newConfig);
        ref.read(uIStateProvider.notifier).showSnackBar('Dossier changé: $folderPath');
        
        _refreshNotes(ref);
      }
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError('Erreur de sélection: $e');
    }
  }

  void _refreshNotes(WidgetRef ref) {
    // TODO: Refresh notes list
    ref.read(uIStateProvider.notifier).showSnackBar('Liste actualisée');
  }
}