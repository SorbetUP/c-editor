import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/state/app_state.dart';
import '../shared/widgets/loading_widget.dart';

class NoteEditorApp extends ConsumerStatefulWidget {
  const NoteEditorApp({super.key});

  @override
  ConsumerState<NoteEditorApp> createState() => _NoteEditorAppState();
}

class _NoteEditorAppState extends ConsumerState<NoteEditorApp> {
  @override
  void initState() {
    super.initState();
    // Initialize the app
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(appStateProvider.notifier).initialize();
    });
  }

  @override
  Widget build(BuildContext context) {
    final appState = ref.watch(appStateProvider);
    
    return appState.when(
      loading: () => const MaterialApp(
        home: Scaffold(
          backgroundColor: Color(0xFF202124),
          body: LoadingWidget(),
        ),
      ),
      error: (error, _) => MaterialApp(
        home: Scaffold(
          backgroundColor: Color(0xFF202124),
          body: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(
                  Icons.error_outline,
                  color: Colors.red,
                  size: 64,
                ),
                const SizedBox(height: 16),
                Text(
                  'Erreur d\'initialisation',
                  style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    color: Colors.white,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  error.toString(),
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Colors.white70,
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
      data: (data) => const SizedBox(), // Main app will be rendered by router
    );
  }
}