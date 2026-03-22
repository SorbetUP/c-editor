import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;

import '../../../core/state/app_state.dart';
import '../../../core/storage/storage_service.dart';
import 'package:c_editor_flutter/models/models.dart';

class StorageSettings extends ConsumerWidget {
  final AppConfig config;

  const StorageSettings({
    super.key,
    required this.config,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Column(
      children: [
        ListTile(
          title: const Text('Dossier par défaut'),
          subtitle: Text(
            config.defaultNotesPath.isNotEmpty
                ? path.basename(config.defaultNotesPath)
                : 'Aucun dossier sélectionné',
          ),
          leading: const Icon(Icons.folder),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _selectDefaultFolder(context, ref),
        ),
        
        ListTile(
          title: const Text('Emplacement complet'),
          subtitle: Text(
            config.defaultNotesPath.isNotEmpty
                ? config.defaultNotesPath
                : 'Non défini',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          isThreeLine: true,
        ),

        ListTile(
          title: const Text('Notes récentes'),
          subtitle: Text('${config.recentNotes.length} notes récentes'),
          leading: const Icon(Icons.history),
          trailing: TextButton(
            onPressed: () => _clearRecentNotes(ref),
            child: const Text('Effacer'),
          ),
        ),

        ListTile(
          title: const Text('Sauvegarde automatique'),
          subtitle: const Text('Sauvegarder automatiquement lors de modifications'),
          leading: const Icon(Icons.save),
          trailing: Switch(
            value: true, // TODO: Add to config
            onChanged: (value) {
              // TODO: Implement auto-save setting
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Sauvegarde automatique bientôt disponible'),
                ),
              );
            },
          ),
        ),

        ListTile(
          title: const Text('Format d\'export'),
          subtitle: const Text('Format par défaut pour l\'export'),
          leading: const Icon(Icons.file_download),
          trailing: DropdownButton<String>(
            value: 'markdown',
            onChanged: (value) {
              // TODO: Implement export format setting
            },
            items: const [
              DropdownMenuItem(
                value: 'markdown',
                child: Text('Markdown (.md)'),
              ),
              DropdownMenuItem(
                value: 'html',
                child: Text('HTML (.html)'),
              ),
              DropdownMenuItem(
                value: 'pdf',
                child: Text('PDF (.pdf)'),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Future<void> _selectDefaultFolder(BuildContext context, WidgetRef ref) async {
    try {
      final storageService = ref.read(storageServiceProvider);
      final selectedPath = await storageService.pickNotesDirectory();
      
      if (selectedPath != null && context.mounted) {
        final newConfig = config.copyWith(defaultNotesPath: selectedPath);
        await ref.read(appStateProvider.notifier).updateConfig(newConfig);
        
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Dossier changé: ${path.basename(selectedPath)}'),
          ),
        );
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Erreur: $e'),
            backgroundColor: Theme.of(context).colorScheme.error,
          ),
        );
      }
    }
  }

  void _clearRecentNotes(WidgetRef ref) {
    final newConfig = config.copyWith(recentNotes: []);
    ref.read(appStateProvider.notifier).updateConfig(newConfig);
  }
}