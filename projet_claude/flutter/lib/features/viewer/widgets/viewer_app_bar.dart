import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';
import 'package:path/path.dart' as path;

import '../../../core/routing/app_router.dart';
import '../../../core/state/app_state.dart';
import '../../../features/home/providers/notes_provider.dart';

class ViewerAppBar extends ConsumerWidget implements PreferredSizeWidget {
  final String notePath;

  const ViewerAppBar({
    super.key,
    required this.notePath,
  });

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final noteDetails = ref.watch(noteDetailsProvider(notePath));
    final noteName = path.basenameWithoutExtension(notePath);
    
    final title = noteDetails.when(
      data: (document) => document?.meta?.title ?? noteName,
      loading: () => noteName,
      error: (_, __) => noteName,
    );

    return AppBar(
      title: Text(
        title,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      leading: IconButton(
        onPressed: () => AppNavigation.toHome(context),
        icon: const Icon(Icons.arrow_back),
      ),
      actions: [
        IconButton(
          onPressed: () => _reloadFromDisk(context, ref),
          icon: const Icon(Icons.refresh),
          tooltip: 'Recharger depuis le disque',
        ),
        IconButton(
          onPressed: () => _exportMarkdown(context, ref),
          icon: const Icon(Icons.file_download),
          tooltip: 'Exporter en Markdown',
        ),
        IconButton(
          onPressed: () => _showDocumentInfo(context, ref),
          icon: const Icon(Icons.info_outline),
          tooltip: 'Informations du document',
        ),
        PopupMenuButton<String>(
          onSelected: (value) => _handleMenuAction(context, ref, value),
          itemBuilder: (context) => [
            const PopupMenuItem(
              value: 'reload',
              child: ListTile(
                leading: Icon(Icons.refresh),
                title: Text('Recharger'),
                contentPadding: EdgeInsets.zero,
              ),
            ),
            const PopupMenuItem(
              value: 'export',
              child: ListTile(
                leading: Icon(Icons.file_download),
                title: Text('Exporter MD'),
                contentPadding: EdgeInsets.zero,
              ),
            ),
            const PopupMenuItem(
              value: 'info',
              child: ListTile(
                leading: Icon(Icons.info),
                title: Text('Informations'),
                contentPadding: EdgeInsets.zero,
              ),
            ),
            const PopupMenuDivider(),
            const PopupMenuItem(
              value: 'settings',
              child: ListTile(
                leading: Icon(Icons.settings),
                title: Text('Paramètres'),
                contentPadding: EdgeInsets.zero,
              ),
            ),
          ],
        ),
      ],
    );
  }

  void _handleMenuAction(BuildContext context, WidgetRef ref, String action) {
    switch (action) {
      case 'reload':
        _reloadFromDisk(context, ref);
        break;
      case 'export':
        _exportMarkdown(context, ref);
        break;
      case 'info':
        _showDocumentInfo(context, ref);
        break;
      case 'settings':
        AppNavigation.toSettings(context);
        break;
    }
  }

  void _reloadFromDisk(BuildContext context, WidgetRef ref) {
    // Invalidate the note provider to force reload
    ref.invalidate(noteDetailsProvider(notePath));
    ref.read(uIStateProvider.notifier).showSnackBar('Document rechargé');
  }

  void _exportMarkdown(BuildContext context, WidgetRef ref) async {
    try {
      ref.read(uIStateProvider.notifier).setLoading(true);
      
      final exportPath = await ref
          .read(noteDetailsProvider(notePath).notifier)
          .exportMarkdown();
      
      if (exportPath != null) {
        ref.read(uIStateProvider.notifier).showSnackBar(
          'Markdown exporté: ${path.basename(exportPath)}',
        );
      } else {
        ref.read(uIStateProvider.notifier).setError(
          'Impossible d\'exporter le document',
        );
      }
    } catch (e) {
      ref.read(uIStateProvider.notifier).setError(
        'Erreur lors de l\'export: $e',
      );
    } finally {
      ref.read(uIStateProvider.notifier).setLoading(false);
    }
  }

  void _showDocumentInfo(BuildContext context, WidgetRef ref) {
    final noteDetails = ref.read(noteDetailsProvider(notePath));
    
    noteDetails.when(
      data: (document) {
        if (document == null) {
          ref.read(uIStateProvider.notifier).setError('Document non chargé');
          return;
        }

        showDialog(
          context: context,
          builder: (context) => _DocumentInfoDialog(
            document: document,
            notePath: notePath,
          ),
        );
      },
      loading: () {
        ref.read(uIStateProvider.notifier).showSnackBar('Chargement...');
      },
      error: (error, _) {
        ref.read(uIStateProvider.notifier).setError('Erreur: $error');
      },
    );
  }
}

class _DocumentInfoDialog extends StatelessWidget {
  final Document document;
  final String notePath;

  const _DocumentInfoDialog({
    required this.document,
    required this.notePath,
  });

  @override
  Widget build(BuildContext context) {
    final meta = document.meta;
    
    return AlertDialog(
      title: const Text('Informations du document'),
      content: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildInfoRow('Chemin', path.basename(notePath)),
            if (meta?.title != null)
              _buildInfoRow('Titre', meta!.title!),
            if (meta?.author != null)
              _buildInfoRow('Auteur', meta!.author!),
            if (meta?.created != null)
              _buildInfoRow('Créé', _formatDateTime(meta!.created!)),
            if (meta?.modified != null)
              _buildInfoRow('Modifié', _formatDateTime(meta!.modified!)),
            _buildInfoRow('Éléments', document.elements.length.toString()),
            _buildInfoRow(
              'Types d\'éléments',
              _getElementTypes(document.elements),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Fermer'),
        ),
      ],
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: Text(
              '$label:',
              style: const TextStyle(fontWeight: FontWeight.w500),
            ),
          ),
          Expanded(
            child: Text(value),
          ),
        ],
      ),
    );
  }

  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.day}/${dateTime.month}/${dateTime.year} '
           'à ${dateTime.hour}:${dateTime.minute.toString().padLeft(2, '0')}';
  }

  String _getElementTypes(List<DocElement> elements) {
    final types = <String, int>{};
    
    for (final element in elements) {
      final typeName = switch (element) {
        DocTextElement() => 'Texte',
        DocImageElement() => 'Image',
        DocTableElement() => 'Tableau',
        _ => 'Inconnu',
      };
      
      types[typeName] = (types[typeName] ?? 0) + 1;
    }
    
    return types.entries
        .map((entry) => '${entry.value} ${entry.key}')
        .join(', ');
  }
}