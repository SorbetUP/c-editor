import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/routing/app_router.dart';
import '../../../core/state/app_state.dart';
import '../../../shared/widgets/loading_widget.dart';
import '../providers/notes_provider.dart';
import '../widgets/note_card.dart';
import '../widgets/home_app_bar.dart';
import '../widgets/empty_state.dart';

class HomeScreen extends ConsumerStatefulWidget {
  const HomeScreen({super.key});

  @override
  ConsumerState<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends ConsumerState<HomeScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(notesProvider.notifier).loadNotes();
    });
  }

  @override
  Widget build(BuildContext context) {
    final appState = ref.watch(appStateProvider);
    final notesState = ref.watch(notesProvider);
    final uiState = ref.watch(uIStateProvider);

    return Scaffold(
      appBar: const HomeAppBar(),
      body: appState.when(
        loading: () => const LoadingWidget(),
        error: (error, _) => _buildErrorState(error.toString()),
        data: (config) => _buildHomeContent(notesState, uiState),
      ),
      floatingActionButton: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          FloatingActionButton.small(
            onPressed: () => AppNavigation.toEditor(context),
            heroTag: 'editor',
            tooltip: 'New Editor',
            child: const Icon(Icons.edit),
          ),
          const SizedBox(height: 8.0),
          FloatingActionButton(
            onPressed: () => _createNewNote(context),
            heroTag: 'note',
            tooltip: 'Créer une nouvelle note',
            child: const Icon(Icons.add),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(String error) {
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
            'Erreur de configuration',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              error,
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 16),
          ElevatedButton(
            onPressed: () => AppNavigation.toSettings(context),
            child: const Text('Paramètres'),
          ),
        ],
      ),
    );
  }

  Widget _buildHomeContent(AsyncValue<List<String>> notesState, UIData uiState) {
    if (uiState.isLoading) {
      return const LoadingWidget();
    }

    return notesState.when(
      loading: () => const LoadingWidget(),
      error: (error, _) => Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.folder_open,
              size: 64,
              color: Colors.orange,
            ),
            const SizedBox(height: 16),
            Text(
              'Erreur de chargement',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              error.toString(),
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () => ref.read(notesProvider.notifier).loadNotes(),
              child: const Text('Réessayer'),
            ),
          ],
        ),
      ),
      data: (notes) => notes.isEmpty 
          ? const EmptyState()
          : _buildNotesList(notes),
    );
  }

  Widget _buildNotesList(List<String> notes) {
    return RefreshIndicator(
      onRefresh: () => ref.read(notesProvider.notifier).loadNotes(),
      child: CustomScrollView(
        slivers: [
          SliverPadding(
            padding: const EdgeInsets.all(16),
            sliver: SliverGrid(
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 350,
                childAspectRatio: 1.2,
                crossAxisSpacing: 16,
                mainAxisSpacing: 16,
              ),
              delegate: SliverChildBuilderDelegate(
                (context, index) {
                  final notePath = notes[index];
                  return NoteCard(
                    notePath: notePath,
                    onTap: () => _openNote(context, notePath),
                    onDelete: () => _deleteNote(notePath),
                  );
                },
                childCount: notes.length,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _createNewNote(BuildContext context) async {
    final result = await showDialog<String>(
      context: context,
      builder: (context) => const _NewNoteDialog(),
    );

    if (result != null && result.isNotEmpty) {
      final notePath = await ref.read(notesProvider.notifier).createNote(result);
      if (notePath != null && context.mounted) {
        AppNavigation.toViewer(context, notePath: notePath);
      }
    }
  }

  void _openNote(BuildContext context, String notePath) {
    ref.read(currentNoteStateProvider.notifier).setCurrentNote(notePath);
    AppNavigation.toViewer(context, notePath: notePath);
  }

  void _deleteNote(String notePath) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Supprimer la note'),
        content: const Text('Êtes-vous sûr de vouloir supprimer cette note ? Cette action ne peut pas être annulée.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Annuler'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: ElevatedButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Supprimer'),
          ),
        ],
      ),
    );

    if (confirmed == true) {
      await ref.read(notesProvider.notifier).deleteNote(notePath);
    }
  }
}

class _NewNoteDialog extends StatefulWidget {
  const _NewNoteDialog();

  @override
  State<_NewNoteDialog> createState() => _NewNoteDialogState();
}

class _NewNoteDialogState extends State<_NewNoteDialog> {
  final _controller = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Nouvelle note'),
      content: Form(
        key: _formKey,
        child: TextFormField(
          controller: _controller,
          decoration: const InputDecoration(
            labelText: 'Nom de la note',
            hintText: 'Ma nouvelle note',
          ),
          autofocus: true,
          validator: (value) {
            if (value == null || value.trim().isEmpty) {
              return 'Veuillez entrer un nom';
            }
            return null;
          },
          onFieldSubmitted: (value) => _submit(),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Annuler'),
        ),
        ElevatedButton(
          onPressed: _submit,
          child: const Text('Créer'),
        ),
      ],
    );
  }

  void _submit() {
    if (_formKey.currentState?.validate() == true) {
      Navigator.of(context).pop(_controller.text.trim());
    }
  }
}