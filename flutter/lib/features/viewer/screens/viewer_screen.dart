import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:c_editor_flutter/models/models.dart';

import '../../../core/routing/app_router.dart';
import '../../../features/home/providers/notes_provider.dart';
import '../../../shared/widgets/loading_widget.dart';
import '../widgets/document_renderer.dart';
import '../widgets/viewer_app_bar.dart';

class ViewerScreen extends ConsumerStatefulWidget {
  final String? notePath;

  const ViewerScreen({
    super.key,
    this.notePath,
  });

  @override
  ConsumerState<ViewerScreen> createState() => _ViewerScreenState();
}

class _ViewerScreenState extends ConsumerState<ViewerScreen> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.notePath == null) {
      return Scaffold(
        appBar: AppBar(
          title: const Text('Viewer'),
        ),
        body: const Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.description_outlined,
                size: 64,
                color: Colors.grey,
              ),
              SizedBox(height: 16),
              Text(
                'Aucune note sélectionnée',
                style: TextStyle(
                  fontSize: 18,
                  color: Colors.grey,
                ),
              ),
            ],
          ),
        ),
      );
    }

    final noteDetails = ref.watch(noteDetailsProvider(widget.notePath!));

    return Scaffold(
      appBar: ViewerAppBar(
        notePath: widget.notePath!,
      ),
      body: noteDetails.when(
        loading: () => const LoadingWidget(),
        error: (error, stackTrace) => _buildErrorView(error),
        data: (document) => document == null
            ? _buildNotFoundView()
            : _buildDocumentView(document),
      ),
    );
  }

  Widget _buildErrorView(Object error) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.error_outline,
            size: 64,
            color: Colors.red,
          ),
          const SizedBox(height: 16),
          Text(
            'Erreur de chargement',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              error.toString(),
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                onPressed: () => AppNavigation.toHome(context),
                child: const Text('Retour'),
              ),
              const SizedBox(width: 16),
              OutlinedButton(
                onPressed: () {
                  ref.invalidate(noteDetailsProvider(widget.notePath!));
                },
                child: const Text('Réessayer'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildNotFoundView() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.search_off,
            size: 64,
            color: Colors.orange,
          ),
          const SizedBox(height: 16),
          Text(
            'Note introuvable',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          const Text(
            'Cette note n\'existe pas ou a été supprimée.',
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => AppNavigation.toHome(context),
            child: const Text('Retour à l\'accueil'),
          ),
        ],
      ),
    );
  }

  Widget _buildDocumentView(Document document) {
    return Column(
      children: [
        // Document info bar
        if (document.meta != null) _buildDocumentInfo(document.meta!),
        
        // Main content
        Expanded(
          child: DocumentRenderer(
            document: document,
            scrollController: _scrollController,
          ),
        ),
      ],
    );
  }

  Widget _buildDocumentInfo(DocumentMeta meta) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).dividerColor,
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (meta.title != null)
                  Text(
                    meta.title!,
                    style: Theme.of(context).textTheme.titleSmall,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                if (meta.modified != null)
                  Text(
                    'Modifié ${_formatDate(meta.modified!)}',
                    style: Theme.of(context).textTheme.labelSmall,
                  ),
              ],
            ),
          ),
          if (meta.author != null) ...[
            const SizedBox(width: 8),
            Chip(
              label: Text(
                meta.author!,
                style: const TextStyle(fontSize: 12),
              ),
              materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
          ],
        ],
      ),
    );
  }

  String _formatDate(DateTime date) {
    final now = DateTime.now();
    final difference = now.difference(date);

    if (difference.inDays == 0) {
      if (difference.inHours == 0) {
        if (difference.inMinutes < 1) {
          return 'à l\'instant';
        }
        return 'il y a ${difference.inMinutes} min';
      }
      return 'il y a ${difference.inHours} h';
    } else if (difference.inDays == 1) {
      return 'hier';
    } else if (difference.inDays < 7) {
      return 'il y a ${difference.inDays} jours';
    } else {
      return '${date.day}/${date.month}/${date.year}';
    }
  }
}