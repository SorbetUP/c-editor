import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;

import 'package:c_editor_flutter/models/models.dart';
import '../providers/notes_provider.dart';

class NoteCard extends ConsumerWidget {
  final String notePath;
  final VoidCallback onTap;
  final VoidCallback onDelete;

  const NoteCard({
    super.key,
    required this.notePath,
    required this.onTap,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final noteDetails = ref.watch(noteDetailsProvider(notePath));
    final noteName = path.basename(notePath);

    return Card(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: noteDetails.when(
            loading: () => const _LoadingCard(),
            error: (error, _) => _ErrorCard(
              noteName: noteName,
              error: error.toString(),
            ),
            data: (document) => _NoteCardContent(
              document: document,
              noteName: noteName,
              onDelete: onDelete,
            ),
          ),
        ),
      ),
    );
  }
}

class _LoadingCard extends StatelessWidget {
  const _LoadingCard();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          height: 20,
          width: double.infinity,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surface,
            borderRadius: BorderRadius.circular(4),
          ),
        ),
        const SizedBox(height: 8),
        Container(
          height: 16,
          width: 150,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surface,
            borderRadius: BorderRadius.circular(4),
          ),
        ),
        const Spacer(),
        const Center(
          child: CircularProgressIndicator(),
        ),
      ],
    );
  }
}

class _ErrorCard extends StatelessWidget {
  final String noteName;
  final String error;

  const _ErrorCard({
    required this.noteName,
    required this.error,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          noteName,
          style: Theme.of(context).textTheme.titleMedium,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        const SizedBox(height: 8),
        Text(
          'Erreur de chargement',
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
            color: Theme.of(context).colorScheme.error,
          ),
        ),
        const Spacer(),
        const Center(
          child: Icon(
            Icons.error_outline,
            color: Colors.red,
            size: 32,
          ),
        ),
      ],
    );
  }
}

class _NoteCardContent extends StatelessWidget {
  final Document? document;
  final String noteName;
  final VoidCallback onDelete;

  const _NoteCardContent({
    required this.document,
    required this.noteName,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    final title = document?.meta?.title ?? noteName;
    final preview = _getPreview(document);
    final lastModified = document?.meta?.modified;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                title,
                style: Theme.of(context).textTheme.titleMedium,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            PopupMenuButton<String>(
              onSelected: (value) {
                if (value == 'delete') {
                  onDelete();
                }
              },
              itemBuilder: (context) => [
                const PopupMenuItem(
                  value: 'delete',
                  child: ListTile(
                    leading: Icon(Icons.delete),
                    title: Text('Supprimer'),
                    contentPadding: EdgeInsets.zero,
                  ),
                ),
              ],
              child: const Icon(Icons.more_vert),
            ),
          ],
        ),
        const SizedBox(height: 8),
        if (preview.isNotEmpty) ...[
          Expanded(
            child: Text(
              preview,
              style: Theme.of(context).textTheme.bodySmall,
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ] else ...[
          const Spacer(),
          Center(
            child: Icon(
              Icons.description,
              color: Theme.of(context).colorScheme.onSurface.withOpacity(0.5),
              size: 32,
            ),
          ),
        ],
        const SizedBox(height: 8),
        if (lastModified != null) ...[
          Text(
            _formatDate(lastModified),
            style: Theme.of(context).textTheme.labelSmall,
          ),
        ],
      ],
    );
  }

  String _getPreview(Document? document) {
    if (document == null || document.elements.isEmpty) {
      return '';
    }

    final textElements = document.elements
        .whereType<DocTextElement>()
        .where((e) => e.level == 0) // Exclude headers
        .take(3);

    if (textElements.isEmpty) {
      return '';
    }

    return textElements
        .map((e) => e.spans.map((s) => s.text).join(' '))
        .where((text) => text.isNotEmpty)
        .join(' ')
        .trim();
  }

  String _formatDate(DateTime date) {
    final now = DateTime.now();
    final difference = now.difference(date);

    if (difference.inDays == 0) {
      if (difference.inHours == 0) {
        return 'Il y a ${difference.inMinutes} min';
      }
      return 'Il y a ${difference.inHours} h';
    } else if (difference.inDays == 1) {
      return 'Hier';
    } else if (difference.inDays < 7) {
      return 'Il y a ${difference.inDays} jours';
    } else {
      return '${date.day}/${date.month}/${date.year}';
    }
  }
}